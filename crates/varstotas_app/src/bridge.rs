//! The bridge: a local server for external scripts (Python, see `python/varstotas.py`) and AI
//! agents (the MCP server `python/varstotas_mcp.py`). It listens on 127.0.0.1 only, on a random
//! port; the port and a random token are written to `~/.varstotas/bridge.json` (mode 0600).
//!
//! Protocol: JSON requests `{"id", "token", "method", "params"}`, one per line, answered by
//! `{"id", "result"}` or `{"id", "error": {"message"}}` lines. The same JSON can be sent as an
//! HTTP `POST /rpc` (token in the body or `Authorization: Bearer <token>`).
//!
//! Network threads only parse and forward requests; the app executes them on the UI thread
//! (`VarstotasApp::bridge_tick`), so every call sees and changes the live document.

use serde_json::{Value, json};
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::sync::mpsc::{Receiver, Sender, channel};
use std::time::Duration;

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

fn random_token() -> String {
    use std::collections::hash_map::RandomState;
    use std::hash::{BuildHasher, Hasher};
    let t = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0);
    (0..2)
        .map(|i| {
            let mut h = RandomState::new().build_hasher();
            h.write_u128(t);
            h.write_u32(std::process::id());
            h.write_u8(i);
            format!("{:016x}", h.finish())
        })
        .collect()
}

impl Bridge {
    /// Listen on 127.0.0.1 (random port), write `~/.varstotas/bridge.json` and the Python modules.
    pub fn start(ctx: egui::Context) -> std::io::Result<Self> {
        let listener = TcpListener::bind(("127.0.0.1", 0))?;
        let port = listener.local_addr()?.port();
        let token = random_token();
        let (tx, rx) = channel();
        let tok = token.clone();
        std::thread::Builder::new().name("bridge".into()).spawn(move || {
            for (conn, stream) in (1u64..).zip(listener.incoming().flatten()) {
                let (tx, tok, ctx) = (tx.clone(), tok.clone(), ctx.clone());
                let _ = std::thread::Builder::new()
                    .name("bridge-conn".into())
                    .spawn(move || serve(stream, conn, &tok, &tx, &ctx));
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
        let mut opts = std::fs::OpenOptions::new();
        opts.write(true).create(true).truncate(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            opts.mode(0o600);
        }
        let mut f = opts.open(&path)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            f.set_permissions(std::fs::Permissions::from_mode(0o600))?;
        }
        f.write_all(info.to_string().as_bytes())
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
        self.rx.try_iter().collect()
    }
}

/// Forward one call to the UI thread and wait for the answer.
fn call(tx: &Sender<Request>, ctx: &egui::Context, conn: u64, method: &str, params: Value) -> Result<Value, String> {
    let (reply, wait) = channel();
    tx.send(Request { method: method.to_string(), params, conn, reply }).map_err(|_| "the app is closing".to_string())?;
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
fn handle(req: &Value, authed: &mut bool, token: &str, tx: &Sender<Request>, ctx: &egui::Context, conn: u64) -> Value {
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

fn serve(stream: TcpStream, conn: u64, token: &str, tx: &Sender<Request>, ctx: &egui::Context) {
    let Ok(write) = stream.try_clone() else { return };
    let mut w = std::io::BufWriter::new(write);
    let mut r = BufReader::new(stream);
    let mut authed = false;
    let mut line = String::new();
    loop {
        line.clear();
        match r.read_line(&mut line) {
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
        if writeln!(w, "{resp}").and_then(|_| w.flush()).is_err() {
            break;
        }
    }
    // Close an operation the client left open (it crashed or forgot): roll it back.
    let _ = tx.send(Request { method: "$disconnect".into(), params: Value::Null, conn, reply: channel().0 });
    ctx.request_repaint();
}

/// One HTTP request (`POST /rpc`), then the connection closes.
fn http(
    first: &str,
    r: &mut BufReader<TcpStream>,
    w: &mut impl Write,
    token: &str,
    tx: &Sender<Request>,
    ctx: &egui::Context,
    conn: u64,
) {
    let mut len = 0usize;
    let mut bearer = None;
    let mut origin = false;
    let mut h = String::new();
    loop {
        h.clear();
        if r.read_line(&mut h).unwrap_or(0) == 0 {
            return;
        }
        let t = h.trim();
        if t.is_empty() {
            break;
        }
        if let Some((k, v)) = t.split_once(':') {
            let (k, v) = (k.trim().to_ascii_lowercase(), v.trim());
            match k.as_str() {
                "content-length" => len = v.parse().unwrap_or(0),
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
    if origin || !first.starts_with("POST ") || len > 64 << 20 {
        return reply(w, "403 Forbidden", &json!({ "error": { "message": "POST /rpc from a local client only" } }));
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
