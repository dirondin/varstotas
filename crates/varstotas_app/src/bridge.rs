//! The bridge: a local server for external scripts (Python, see `python/varstotas.py`) and AI
//! agents (the MCP server `python/varstotas_mcp.py`). It listens on 127.0.0.1 only, on a random
//! port; the port and a random token are written to `~/.varstotas/bridge.json` (mode 0600).
//!
//! Protocol: JSON requests `{"id", "token", "method", "params"}`, one per line, answered by
//! `{"id", "result"}` or `{"id", "error": {"message"}}` lines. The same JSON can be sent as an
//! HTTP `POST /rpc` (`Authorization: Bearer <token>` required for HTTP).
//!
//! Network threads only parse and forward requests; the app executes them on the UI thread
//! (`VarstotasApp::bridge_tick`), so every call sees and changes the live document.

use serde_json::{Value, json};
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::sync::mpsc::{Receiver, Sender, SyncSender, channel, sync_channel};
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};
use std::time::Duration;
const MAX_REQUEST: usize = 1024 * 1024;
const MAX_CLIENTS: usize = 16;

/// The Python client module and the MCP server, installed next to `bridge.json`.
pub const PY_CLIENT: &str = include_str!("../../../python/varstotas.py");
pub const PY_MCP: &str = include_str!("../../../python/varstotas_mcp.py");

/// A call waiting for the UI thread.
pub struct Request {
    pub method: String,
    pub params: Value,
    /// Connection the call came from (transactions belong to a connection).
    pub conn: u64,
    pub reply: Sender<Result<Value, String>>,
}

pub struct Bridge {
    pub port: u16,
    pub token: String,
    rx: Receiver<Request>,
}

/// `~/.varstotas` (the bridge file and the Python modules).
pub fn home_dir() -> Option<PathBuf> {
    std::env::var_os("HOME").or_else(|| std::env::var_os("USERPROFILE")).map(|h| PathBuf::from(h).join(".varstotas"))
}

/// Directory with `varstotas.py` / `varstotas_mcp.py` (for `PYTHONPATH`).
pub fn python_dir() -> Option<PathBuf> {
    home_dir().map(|d| d.join("python"))
}

pub(crate) fn random_token() -> std::io::Result<String> {
    let mut bytes = [0u8; 32];
    getrandom::fill(&mut bytes).map_err(std::io::Error::other)?;
    Ok(bytes.iter().map(|b| format!("{b:02x}")).collect())
}

fn limited_line(r: &mut impl BufRead, line: &mut String, limit: usize) -> std::io::Result<usize> {
            let mut bytes = Vec::new();
    let n = r.take(limit as u64 + 1).read_until(b'\n', &mut bytes)?;
    if n > limit {
        return Err(std::io::Error::other("Bridge line too long"));
    }
    *line = String::from_utf8(bytes).map_err(std::io::Error::other)?;
    Ok(n)
}

struct ClientSlot(Arc<AtomicUsize>);
impl Drop for ClientSlot {
    fn drop(&mut self) {
        self.0.fetch_sub(1, Ordering::Relaxed);
    }
}

impl Bridge {
    /// Listen on 127.0.0.1 (random port), write `~/.varstotas/bridge.json` and the Python modules.
    pub fn start(ctx: egui::Context) -> std::io::Result<Self> {
        let listener = TcpListener::bind(("127.0.0.1", 0))?;
        let port = listener.local_addr()?.port();
        let token = random_token()?;
        let (tx, rx) = sync_channel(64);
        let clients = Arc::new(AtomicUsize::new(0));
        let tok = token.clone();
        std::thread::Builder::new().name("bridge".into()).spawn(move || {
            for (conn, stream) in (1u64..).zip(listener.incoming().flatten()) {
                    if clients
                        .try_update(Ordering::Relaxed, Ordering::Relaxed, |n| {
                            (n < MAX_CLIENTS).then_some(n + 1)
                        })
                        .is_err()
                    {
                        continue;
                    }
                    let slot = ClientSlot(clients.clone());
                    let _ = stream.set_read_timeout(Some(Duration::from_secs(10)));
                    let _ = stream.set_write_timeout(Some(Duration::from_secs(10)));
                    let (tx, tok, ctx) = (tx.clone(), tok.clone(), ctx.clone());
                let _ = std::thread::Builder::new()
                    .name("bridge-conn".into())
                    .spawn(move || {
                            let _slot = slot;
                            serve(stream, conn, &tok, &tx, &ctx);
                        });
            }
        })?;
        let b = Self { port, token, rx };
        if let Err(e) = b.write_files() {
            log::warn!("bridge: cannot write ~/.varstotas: {e}");
        }
        log::info!("bridge: listening on 127.0.0.1:{port}");
        Ok(b)
    }

    fn write_files(&self) -> std::io::Result<()> {
        let Some(dir) = home_dir() else { return Ok(()) };
        let py = dir.join("python");
        std::fs::create_dir_all(&py)?;
        std::fs::write(py.join("varstotas.py"), PY_CLIENT)?;
        std::fs::write(py.join("varstotas_mcp.py"), PY_MCP)?;
        let info = json!({ "host": "127.0.0.1", "port": self.port, "token": self.token, "pid": std::process::id() });
        let path = dir.join("bridge.json");
        varstotas_kernel::io::atomic_write(&path, info.to_string().as_bytes())
    }

    /// Remove `bridge.json` when it is still ours (the app is closing).
    pub fn remove_file(&self) {
        let Some(path) = home_dir().map(|d| d.join("bridge.json")) else { return };
        let ours = std::fs::read_to_string(&path)
            .ok()
            .and_then(|s| serde_json::from_str::<Value>(&s).ok())
            .is_some_and(|v| v["token"] == self.token.as_str());
        if ours {
            let _ = std::fs::remove_file(path);
        }
    }

    /// Calls that arrived since the last frame.
    pub fn poll(&self) -> Vec<Request> {
        self.rx.try_iter().take(32).collect()
    }
}

/// Forward one call to the UI thread and wait for the answer.
fn call(tx: &SyncSender<Request>, ctx: &egui::Context, conn: u64, method: &str, params: Value) -> Result<Value, String> {
    let (reply, wait) = channel();
    tx.try_send(Request { method: method.to_string(), params, conn, reply }).map_err(|_| "the app is closing".to_string())?;
    ctx.request_repaint();
    // Long enough for a confirmation dialog the user has to answer.
    wait.recv_timeout(Duration::from_secs(600)).map_err(|_| "no answer from the app".to_string())?
}

fn answer(id: &Value, r: Result<Value, String>) -> Value {
    match r {
        Ok(v) => json!({ "id": id, "result": v }),
        Err(e) => json!({ "id": id, "error": { "message": e } }),
    }
}

/// Handle one parsed request (token check, then the UI thread).
fn handle(req: &Value, authed: &mut bool, token: &str, tx: &SyncSender<Request>, ctx: &egui::Context, conn: u64) -> Value {
    let id = req.get("id").cloned().unwrap_or(Value::Null);
    if !*authed {
        if req.get("token").and_then(Value::as_str) != Some(token) {
            return answer(&id, Err("bad or missing token (see ~/.varstotas/bridge.json)".into()));
        }
        *authed = true;
    }
    let Some(method) = req.get("method").and_then(Value::as_str) else {
        return answer(&id, Err("missing method".into()));
    };
    let params = req.get("params").cloned().unwrap_or(Value::Null);
    answer(&id, call(tx, ctx, conn, method, params))
}

fn serve(stream: TcpStream, conn: u64, token: &str, tx: &SyncSender<Request>, ctx: &egui::Context) {
    let Ok(write) = stream.try_clone() else { return };
    let mut w = std::io::BufWriter::new(write);
    let mut r = BufReader::new(stream);
    let mut authed = false;
    let mut line = String::new();
    loop {
        line.clear();
        match limited_line(&mut r, &mut line, MAX_REQUEST) {
            Ok(0) | Err(_) => break,
            Ok(_) => {}
        }
        let l = line.trim();
        if l.is_empty() {
            continue;
        }
        if l.starts_with("POST ") || l.starts_with("GET ") || l.starts_with("OPTIONS ") {
            http(l, &mut r, &mut w, token, tx, ctx, conn);
            break;
        }
        let resp = match serde_json::from_str::<Value>(l) {
            Ok(req) => handle(&req, &mut authed, token, tx, ctx, conn),
            Err(e) => answer(&Value::Null, Err(format!("bad JSON: {e}"))),
        };
        if writeln!(w, "{resp}").and_then(|()| w.flush()).is_err() {
            break;
        }
    }
    // Close an operation the client left open (it crashed or forgot): roll it back.
    let _ = tx.try_send(Request { method: "$disconnect".into(), params: Value::Null, conn, reply: channel().0 });
    ctx.request_repaint();
}

/// One HTTP request (`POST /rpc`), then the connection closes.
fn http(
    first: &str,
    r: &mut BufReader<TcpStream>,
    w: &mut impl Write,
    token: &str,
    tx: &SyncSender<Request>,
    ctx: &egui::Context,
    conn: u64,
) {
    let mut len = 0usize;
    let mut seen_length = false;
    let mut bearer = None;
    let mut origin = false;
    let mut h = String::new();
    let mut header_bytes = 0usize;
    let mut host = None;
    let mut invalid = false;
    loop {
        h.clear();
        if limited_line(r, &mut h, 4096).unwrap_or(0) == 0 {
            return;
        }
        header_bytes += h.len();
        if header_bytes > 16 * 1024 {
            return;
        }
        let t = h.trim();
        if t.is_empty() {
            break;
        }
        if let Some((k, v)) = t.split_once(':') {
            let (k, v) = (k.trim().to_ascii_lowercase(), v.trim());
            match k.as_str() {
                "content-length" => {
                    if seen_length {
                        invalid = true;
                    }
                    seen_length = true;
                    len = v.parse().unwrap_or(usize::MAX);
                }
                "transfer-encoding" => invalid = true,
                "host" => host = Some(v.to_string()),
                "authorization" => bearer = v.strip_prefix("Bearer ").map(str::to_string),
                // Browsers send Origin: refuse pages trying to reach the local app.
                "origin" => origin = true,
                _ => {}
            }
        }
    }
    let reply = |w: &mut dyn Write, code: &str, body: &Value| {
        let b = body.to_string();
        let _ = write!(w, "HTTP/1.1 {code}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{b}", b.len());
        let _ = w.flush();
    };
    let local_host = host.as_deref().is_some_and(|h| {
        ["127.0.0.1", "localhost"].iter().any(|p| {
            h == *p
                || h.strip_prefix(p)
                    .is_some_and(|s| s.starts_with(':') && s[1..].parse::<u16>().is_ok())
        })
    });
    if origin || invalid
        || !local_host
        || !matches!(first, "POST /rpc HTTP/1.1" | "POST /rpc HTTP/1.0") || len > MAX_REQUEST
    {
        return reply(w, "403 Forbidden", &json!({ "error": { "message": "POST /rpc from a local client only" } })
        );
    }
    if bearer.as_deref() != Some(token) {
        return reply(
            w,
            "401 Unauthorized",
            &answer(&Value::Null, Err("Missing or invalid bearer token".into())));
    }
    let mut body = vec![0; len];
    if r.read_exact(&mut body).is_err() {
        return;
    }
    let mut req = match serde_json::from_slice::<Value>(&body) {
        Ok(v) => v,
        Err(e) => return reply(w, "400 Bad Request", &answer(&Value::Null, Err(format!("bad JSON: {e}")))),
    };
    if let (Some(b), Some(o)) = (bearer, req.as_object_mut()) {
        o.entry("token").or_insert(Value::String(b));
    }
    let mut authed = false;
    let resp = handle(&req, &mut authed, token, tx, ctx, conn);
    let code = if authed { "200 OK" } else { "401 Unauthorized" };
    reply(w, code, &resp);
}

#[cfg(test)]
mod security_tests {
    use super::*;
    #[test]
    fn tokens_use_random_256_bit_values() {
        let a = random_token().unwrap();
        let b = random_token().unwrap();
        assert_eq!(a.len(), 64);
        assert!(a.bytes().all(|b| b.is_ascii_hexdigit()));
        assert_ne!(a, b);
    }
    #[test]
    fn line_limit_applies_without_a_newline() {
        let mut r = std::io::Cursor::new(vec![b'x'; 65]);
        assert!(limited_line(&mut r, &mut String::new(), 64).is_err());
    }
    fn rejected_http(request: &[u8]) -> String {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let mut client = TcpStream::connect(listener.local_addr().unwrap()).unwrap();
        client
            .set_read_timeout(Some(Duration::from_secs(2)))
            .unwrap();
        let (server, _) = listener.accept().unwrap();
        server
            .set_read_timeout(Some(Duration::from_secs(2)))
            .unwrap();
        let (tx, rx) = sync_channel(1);
        let thread =
            std::thread::spawn(move || serve(server, 1, "secret", &tx, &egui::Context::default()));
        client.write_all(request).unwrap();
        let mut response = String::new();
        client.read_to_string(&mut response).unwrap();
        thread.join().unwrap();
        assert!(rx.try_iter().all(|r| r.method == "$disconnect"));
        response
    }
    #[test]
    fn http_checks_authorization_before_reading_body() {
        // Intentionally send no body: the response must arrive without waiting for it.
        let r = rejected_http(
            b"POST /rpc HTTP/1.1\r\nHost: localhost\r\nContent-Length: 1000000\r\n\r\n",
        );
        assert!(r.starts_with("HTTP/1.1 401"), "{r}");
    }
    #[test]
    fn browser_origin_routes_and_ambiguous_lengths_are_denied() {
        for request in [
            "POST /rpc HTTP/1.1\r\nHost: localhost\r\nOrigin: http://evil.test\r\nAuthorization: Bearer secret\r\nContent-Length: 0\r\n\r\n",
            "POST /other HTTP/1.1\r\nHost: localhost\r\nAuthorization: Bearer secret\r\nContent-Length: 0\r\n\r\n",
            "POST /rpc HTTP/1.1\r\nHost: localhost\r\nAuthorization: Bearer secret\r\nContent-Length: 0\r\nContent-Length: 1\r\n\r\n",
        ] {
            assert!(rejected_http(request.as_bytes()).starts_with("HTTP/1.1 403"));
        }
    }
}
