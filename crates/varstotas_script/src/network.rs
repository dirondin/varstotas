//! HTTP(S)-only downloads with bounded output. Local files are handled by explicit file APIs.
use std::time::Duration;
/// Progress of the current HTTP response body (not headers or redirect responses).
#[derive(Clone, Debug, Default)]
pub struct TransferProgress {
    pub received: u64,
    pub total: Option<u64>,
    pub file: String,
}

type Observer = std::sync::Arc<dyn Fn(TransferProgress) + Send + Sync>;
thread_local! {
    static OBSERVER: std::cell::RefCell<Option<Observer>> = const { std::cell::RefCell::new(None) };
}

/// Worker-local observer; its guard restores a previous observer, including on unwind.
pub struct ProgressGuard(Option<Observer>);
impl Drop for ProgressGuard {
    fn drop(&mut self) {
        OBSERVER.with(|o| *o.borrow_mut() = self.0.take());
    }
}
pub fn observe_progress(observer: Observer) -> ProgressGuard {
    ProgressGuard(OBSERVER.with(|o| o.borrow_mut().replace(observer)))
}

pub const MAX_DOWNLOAD: usize = 256 * 1024 * 1024;

/// One HTTP client for the whole app: connections (and TLS sessions) to a host are reused, so a
/// catalogue's thumbnails do not each pay for a new connection. Certificates are checked against
/// the system's store; proxies come from the usual environment variables (`HTTPS_PROXY`, …).
fn agent() -> &'static ureq::Agent {
    static AGENT: std::sync::OnceLock<ureq::Agent> = std::sync::OnceLock::new();
    AGENT.get_or_init(|| {
        use ureq::tls::{RootCerts, TlsConfig};
        ureq::Agent::config_builder()
            .user_agent("Varstotas")
            .timeout_connect(Some(Duration::from_secs(10)))
            .max_redirects(10)
            .tls_config(TlsConfig::builder().root_certs(RootCerts::PlatformVerifier).build())
            .build()
            .into()
    })
}

/// GET `url` (HTTP(S) only, redirects followed, HTTP errors fail) with extra request headers,
/// within `timeout`; at most [`MAX_DOWNLOAD`] bytes. `cancelled` is checked while the body
/// arrives; the worker's progress observer ([`observe_progress`]) sees the body bytes.
pub fn fetch(
    url: &str,
    headers: &[(String, String)],
    timeout: Duration,
    cancelled: impl Fn() -> bool,
) -> Result<Vec<u8>, String> {
    use std::io::Read;
    if !(url.starts_with("https://") || url.starts_with("http://"))
        || url.chars().any(char::is_control)
    {
        return Err("Only HTTP(S) URLs are allowed".into());
    }
    if timeout.is_zero() {
        return Err("Download deadline exceeded".into());
    }
    let mut request = agent().get(url);
    for (k, v) in headers {
        if k.contains(['\r', '\n', ':']) || v.contains(['\r', '\n']) {
            return Err("Invalid HTTP header".into());
        }
        request = request.header(k.as_str(), v.as_str());
    }
    let mut response = request
        .config()
        .timeout_global(Some(timeout))
        .build()
        .call()
        .map_err(|e| match e {
            ureq::Error::StatusCode(code) => format!("HTTP {code}: {url}"),
            e => format!("{e}: {url}"),
        })?;
    let total = response.body().content_length();
    let observer = OBSERVER.with(|o| o.borrow().clone());
    let file = url
        .split('?')
        .next()
        .unwrap_or(url)
        .rsplit('/')
        .next()
        .unwrap_or("download")
        .to_string();
    let report = |received: usize| {
        if let Some(observer) = &observer {
            observer(TransferProgress {
                received: received as u64,
                total,
                file: file.clone(),
            });
        }
    };
    report(0);
    let mut reader = response
        .body_mut()
        .with_config()
        .limit(MAX_DOWNLOAD as u64 + 1)
        .reader();
    let mut out = Vec::new();
    let mut buf = vec![0u8; 64 * 1024];
    loop {
        if cancelled() {
            return Err("Download cancelled".into());
        }
        let n = reader.read(&mut buf).map_err(|e| format!("{e}: {url}"))?;
        if n == 0 {
            break;
        }
        out.extend_from_slice(&buf[..n]);
        if out.len() > MAX_DOWNLOAD {
            return Err("Download exceeds size limit".into());
        }
        report(out.len());
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        io::{Read, Write},
        net::TcpListener,
        sync::{Arc, Mutex},
    };

    #[test]
    fn streaming_progress_counts_the_body() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let url = format!("http://{}/model.glb", listener.local_addr().unwrap());
        let server = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut request = [0; 2048];
            assert!(stream.read(&mut request).unwrap() > 0);
            stream
                .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 8\r\nConnection: close\r\n\r\nabcd")
                .unwrap();
            stream.flush().unwrap();
            std::thread::sleep(Duration::from_millis(250));
            stream.write_all(b"efgh").unwrap();
        });
        let updates = Arc::new(Mutex::new(Vec::new()));
        let record = updates.clone();
        let _guard = observe_progress(Arc::new(move |p| record.lock().unwrap().push(p)));
        assert_eq!(
            fetch(&url, &[], Duration::from_secs(3), || false).unwrap(),
            b"abcdefgh"
        );
        server.join().unwrap();
        let updates = updates.lock().unwrap();
        assert!(
            updates
                .iter()
                .any(|p| p.received == 4 && p.total == Some(8))
        );
        let last = updates.last().unwrap();
        assert_eq!((last.received, last.total), (8, Some(8)));
        assert_eq!(last.file, "model.glb");
    }
    #[test]
    fn redirect_and_chunked_progress_preserve_the_final_body() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let url = format!("http://{}/redirect", listener.local_addr().unwrap());
        let server = std::thread::spawn(move || {
            for redirect in [true, false] {
                let (mut stream, _) = listener.accept().unwrap();
                let mut request = [0; 2048];
                assert!(stream.read(&mut request).unwrap() > 0);
                let response: &[u8] = if redirect {
                    b"HTTP/1.1 302 Found\r\nLocation: /final\r\nContent-Length: 7\r\nConnection: close\r\n\r\nignored"
                } else {
                    b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\nConnection: close\r\n\r\n8\r\nabcdefgh\r\n0\r\n\r\n"
                };
                stream.write_all(response).unwrap();
            }
        });
        let updates = Arc::new(Mutex::new(Vec::new()));
        let record = updates.clone();
        let _guard = observe_progress(Arc::new(move |p| record.lock().unwrap().push(p)));
        assert_eq!(
            fetch(&url, &[], Duration::from_secs(3), || false).unwrap(),
            b"abcdefgh"
        );
        server.join().unwrap();
        let updates = updates.lock().unwrap();
        let last = updates.last().unwrap();
        assert_eq!((last.received, last.total), (8, None));
    }
}
