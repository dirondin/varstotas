//! Bounded child-process IO, deadlines and cancellation. Not an OS permission sandbox.
use std::io::{Read, Write};
use std::process::{Child, Command, Output, Stdio};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::time::{Duration, Instant};

pub fn isolate_group(cmd: &mut Command) {
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        cmd.process_group(0);
    }
}

pub fn terminate(child: &mut Child) {
    #[cfg(unix)]
    {
        // SAFETY: all children using this helper were started in their own process group.
        unsafe {
            libc::kill(-(child.id() as i32), libc::SIGKILL);
        }
    }
    let _ = child.kill();
    let _ = child.wait();
}

pub fn output(
    cmd: &mut Command,
    timeout: Duration,
    max_bytes: usize,
    cancelled: impl Fn() -> bool,
) -> Result<Output, String> {
    output_input(cmd, timeout, max_bytes, cancelled, None)
}

pub fn output_input(
    cmd: &mut Command,
    timeout: Duration,
    max_bytes: usize,
    cancelled: impl Fn() -> bool,
    input: Option<Vec<u8>>,
) -> Result<Output, String> {
    isolate_group(cmd);
    cmd.stdin(if input.is_some() { Stdio::piped() } else { Stdio::null() });
    let mut child = cmd
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| e.to_string())?;
    if let Some(input) = input && let Some(mut stdin) = child.stdin.take() {
        std::thread::spawn(move || { let _ = stdin.write_all(&input); });
    }
    let exceeded = Arc::new(AtomicBool::new(false));
    let read = |pipe: Box<dyn Read + Send>, limit: usize, flag: Arc<AtomicBool>| {
        let (tx, rx) = std::sync::mpsc::sync_channel(1);
        std::thread::spawn(move || {
            let mut data = Vec::new();
            let result = pipe.take(limit as u64 + 1).read_to_end(&mut data);
            if result.is_err() || data.len() > limit {
                flag.store(true, Ordering::Relaxed);
            }
            data.truncate(limit);
            let _ = tx.send(data);
        });
        rx
    };
    let stdout = read(
        Box::new(child.stdout.take().unwrap()),
        max_bytes,
        exceeded.clone(),
    );
    let stderr = read(
        Box::new(child.stderr.take().unwrap()),
        1024 * 1024,
        exceeded.clone(),
    );
    let deadline = Instant::now() + timeout;
    let mut error = None;
    let status = loop {
        if cancelled() || exceeded.load(Ordering::Relaxed) || Instant::now() >= deadline {
            error = Some("Process cancelled, timed out, or exceeded its output limit".to_string());
            terminate(&mut child);
            break None;
        }
        match child.try_wait() {
            Ok(Some(status)) => break Some(status),
            Ok(None) => std::thread::sleep(Duration::from_millis(5)),
            Err(e) => {
                error = Some(e.to_string());
                terminate(&mut child);
                break None;
            }
        }
    };
    // Descendants must not keep the output pipes alive after the main process exits.
    #[cfg(unix)]
    unsafe {
        libc::kill(-(child.id() as i32), libc::SIGKILL);
    }
    let stdout = stdout
        .recv_timeout(Duration::from_millis(100))
        .map_err(|_| "stdout pipe did not close")?;
    let stderr = stderr
        .recv_timeout(Duration::from_millis(100))
        .map_err(|_| "stderr pipe did not close")?;
    if exceeded.load(Ordering::Relaxed) {
        return Err("Process output limit exceeded".into());
    }
    if let Some(e) = error {
        return Err(e);
    }
    Ok(Output {
        status: status.ok_or("Process failed")?,
        stdout,
        stderr,
    })
}
