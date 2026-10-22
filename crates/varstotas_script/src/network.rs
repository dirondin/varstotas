//! HTTP(S)-only downloads with bounded output. Local files are handled by explicit file APIs.
use std::time::Duration;
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
    cmd.arg("--").arg(url);
    let out = crate::process::output(&mut cmd, timeout, MAX_DOWNLOAD, cancelled)?;
    if !out.status.success() {
        return Err(String::from_utf8_lossy(&out.stderr).trim().to_string());
    }
    Ok(out.stdout)
}
