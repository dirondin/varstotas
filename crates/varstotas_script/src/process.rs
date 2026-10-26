//! Bounded child-process IO, deadlines and cancellation. Not an OS permission sandbox.
use std::io::{Read, Write};
use std::process::{Child, Command, Output, Stdio};
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, AtomicUsize, Ordering},
};
use std::time::{Duration, Instant};

type OutputObserver<'a> = dyn FnMut(usize, &[u8]) + 'a;
type PrefixState = (Arc<AtomicUsize>, Arc<Mutex<Vec<u8>>>);

/// Starts the child in its own process group (Unix), so [`terminate`] kills its children too.
pub fn isolate_group(#[cfg_attr(not(unix), allow(unused_variables))] cmd: &mut Command) {
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
    output_inner(cmd, timeout, max_bytes, cancelled, input, None)
}

/// Observe stdout byte count and its first 64 KiB while preserving bounded IO and cancellation.
pub fn output_progress(
    cmd: &mut Command,
    timeout: Duration,
    max_bytes: usize,
    cancelled: impl Fn() -> bool,
    progress: &mut OutputObserver<'_>,
) -> Result<Output, String> {
    output_inner(cmd, timeout, max_bytes, cancelled, None, Some(progress))
}

fn output_inner(
    cmd: &mut Command,
    timeout: Duration,
    max_bytes: usize,
    cancelled: impl Fn() -> bool,
    input: Option<Vec<u8>>,
    mut progress: Option<&mut OutputObserver<'_>>,
) -> Result<Output, String> {
    isolate_group(cmd);
    cmd.stdin(if input.is_some() {
        Stdio::piped()
    } else {
        Stdio::null()
    });
    let mut child = cmd
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| e.to_string())?;
    if let Some(input) = input
        && let Some(mut stdin) = child.stdin.take()
    {
        std::thread::spawn(move || {
            let _ = stdin.write_all(&input);
        });
    }
    let exceeded = Arc::new(AtomicBool::new(false));
    let received = Arc::new(AtomicUsize::new(0));
    let prefix = Arc::new(Mutex::new(Vec::new()));
    let read = |mut pipe: Box<dyn Read + Send>,
                limit: usize,
                flag: Arc<AtomicBool>,
                observe: Option<PrefixState>| {
        let (tx, rx) = std::sync::mpsc::sync_channel(1);
        std::thread::spawn(move || {
            let mut data = Vec::new();
            let mut buf = [0u8; 8192];
            loop {
                match pipe.read(&mut buf) {
                    Ok(0) => break,
                    Ok(n) => {
                        let n = n.min((limit + 1).saturating_sub(data.len()));
                        data.extend_from_slice(&buf[..n]);
                        if let Some((count, prefix)) = &observe {
                            if let Ok(mut prefix) = prefix.lock() {
                                let keep = n.min(65536usize.saturating_sub(prefix.len()));
                                prefix.extend_from_slice(&buf[..keep]);
                            }
                            count.store(data.len(), Ordering::Relaxed);
                        }
                        if data.len() > limit {
                            flag.store(true, Ordering::Relaxed);
                            break;
                        }
                    }
                    Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
                    Err(_) => {
                        flag.store(true, Ordering::Relaxed);
                        break;
                    }
                }
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
        progress
            .is_some()
            .then(|| (received.clone(), prefix.clone())),
    );
    let stderr = read(
        Box::new(child.stderr.take().unwrap()),
        1024 * 1024,
        exceeded.clone(),
        None,
    );
    let deadline = Instant::now() + timeout;
    let mut error = None;
    let mut last_progress = Instant::now() - Duration::from_secs(1);
    let status = loop {
        if last_progress.elapsed() >= Duration::from_millis(80) {
            if let Some(progress) = progress.as_mut()
                && let Ok(prefix) = prefix.lock()
            {
                progress(received.load(Ordering::Relaxed), &prefix);
            }
            last_progress = Instant::now();
        }
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
    if let Some(progress) = progress.as_mut() {
        progress(stdout.len(), &stdout[..stdout.len().min(65536)]);
    }
    Ok(Output {
        status: status.ok_or("Process failed")?,
        stdout,
        stderr,
    })
}
