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

/// Parse curl's included response headers, skipping redirects and proxy/100 preambles.
fn body_headers(bytes: &[u8]) -> Option<(usize, Option<u64>)> {
    let mut offset = 0;
    loop {
        let rest = bytes.get(offset..)?;
        let end = rest.windows(4).position(|w| w == b"\r\n\r\n")? + 4;
        let header = String::from_utf8_lossy(&rest[..end]);
        let status = header.lines().next()?;
        if !status.starts_with("HTTP/") {
            return None;
        }
        let code: u16 = status.split_whitespace().nth(1)?.parse().ok()?;
        offset += end;
        if (100..200).contains(&code)
            || (300..400).contains(&code)
            || status
                .to_ascii_lowercase()
                .contains("connection established")
        {
            continue;
        }
        let mut total = None;
        let mut chunked = false;
        for line in header.lines().skip(1) {
            if let Some((name, value)) = line.split_once(':') {
                if name.eq_ignore_ascii_case("content-length") {
                    total = value.trim().parse().ok();
                }
                if name.eq_ignore_ascii_case("transfer-encoding") {
                    chunked = value.to_ascii_lowercase().contains("chunked");
                }
            }
        }
        return Some((offset, if chunked { None } else { total }));
    }
}

pub const MAX_DOWNLOAD: usize = 256 * 1024 * 1024;
pub fn fetch(
    url: &str,
    headers: &[(String, String)],
    timeout: Duration,
    cancelled: impl Fn() -> bool,
) -> Result<Vec<u8>, String> {
    if !(url.starts_with("https://") || url.starts_with("http://"))
        || url.chars().any(char::is_control)
    {
        return Err("Only HTTP(S) URLs are allowed".into());
    }
    if timeout.is_zero() {
        return Err("Download deadline exceeded".into());
    }
    let mut cmd = std::process::Command::new(if cfg!(windows) { "curl.exe" } else { "curl" });
    cmd.args([
        "-q",
        "-sSfL",
        "--proto",
        "=http,https",
        "--proto-redir",
        "=http,https",
        "--connect-timeout",
        "10",
        "--max-time",
    ])
    .arg(timeout.as_secs_f64().to_string())
    .args(["--max-filesize", "268435456", "--user-agent", "Varstotas"]);
    for (k, v) in headers {
        if k.contains(['\r', '\n', ':']) || v.contains(['\r', '\n']) {
            return Err("Invalid HTTP header".into());
        }
        cmd.arg("-H").arg(format!("{k}: {v}"));
    }
    let observer = OBSERVER.with(|o| o.borrow().clone());
    if observer.is_some() {
        cmd.args(["--include", "--no-buffer"]);
    }
    cmd.arg("--").arg(url);
    let out = if let Some(observer) = observer {
        let file = url
            .split('?')
            .next()
            .unwrap_or(url)
            .rsplit('/')
            .next()
            .unwrap_or("download")
            .to_string();
        observer(TransferProgress {
            received: 0,
            total: None,
            file: file.clone(),
        });
        let mut progress = |count: usize, prefix: &[u8]| {
            if let Some((offset, total)) = body_headers(prefix) {
                observer(TransferProgress {
                    received: count.saturating_sub(offset) as u64,
                    total,
                    file: file.clone(),
                });
            }
        };
        let mut out = crate::process::output_progress(
            &mut cmd,
            timeout,
            MAX_DOWNLOAD + 65536,
            cancelled,
            &mut progress,
        )?;
        if out.status.success() {
            let (offset, _) = body_headers(&out.stdout[..out.stdout.len().min(65536)])
                .ok_or("Invalid or oversized HTTP response headers")?;
            out.stdout.drain(..offset);
            if out.stdout.len() > MAX_DOWNLOAD {
                return Err("Download exceeds size limit".into());
            }
        }
        out
    } else {
        crate::process::output(&mut cmd, timeout, MAX_DOWNLOAD, cancelled)?
    };
    if !out.status.success() {
        return Err(String::from_utf8_lossy(&out.stderr).trim().to_string());
    }
    Ok(out.stdout)
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
    fn response_headers_use_final_length_and_ignore_chunked_lengths() {
        let headers = b"HTTP/1.1 200 Connection established\r\n\r\nHTTP/1.1 302 Found\r\nContent-Length: 999\r\n\r\nHTTP/2 200 OK\r\nContent-Length: 42\r\n\r\n";
        assert_eq!(body_headers(headers), Some((headers.len(), Some(42))));
        let chunked =
            b"HTTP/1.1 200 OK\r\nContent-Length: 99\r\nTransfer-Encoding: chunked\r\n\r\n";
        assert_eq!(body_headers(chunked), Some((chunked.len(), None)));
        assert_eq!(body_headers(b"HTTP/1.1 200 OK\r\nContent-Length:"), None);
    }

    #[test]
    fn streaming_progress_does_not_mix_headers_into_the_download() {
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
