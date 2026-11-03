//! Reading GPU buffers back. A buffer is copied to and mapped by the GPU in its own time: the
//! desktop builds can wait for it (`Pending::wait`, `poll` with `wait`), a browser cannot — its
//! WebGPU answers only once the page's frame has returned, so there the work is polled again on
//! the next frames until it is in.

use crate::wgpu;
use std::sync::Arc;
use std::sync::atomic::{AtomicU8, Ordering};

const WAITING: u8 = 0;
const MAPPED: u8 = 1;
const FAILED: u8 = 2;

/// One buffer the GPU is copying to, mapped for reading once it is done.
struct Readback {
    buf: wgpu::Buffer,
    state: Arc<AtomicU8>,
    submission: wgpu::SubmissionIndex,
}

impl Readback {
    /// Submit `enc` (which copies into `buf`) and ask for `buf` to be mapped.
    fn start(queue: &wgpu::Queue, buf: wgpu::Buffer, enc: wgpu::CommandEncoder) -> Self {
        let submission = queue.submit(Some(enc.finish()));
        let state = Arc::new(AtomicU8::new(WAITING));
        let mapped = state.clone();
        buf.slice(..).map_async(wgpu::MapMode::Read, move |r| {
            mapped.store(if r.is_ok() { MAPPED } else { FAILED }, Ordering::SeqCst);
        });
        Self { buf, state, submission }
    }

    /// The buffer's bytes, once mapped.
    fn take(&self, device: &wgpu::Device, wait: bool) -> Option<Result<Vec<u8>, String>> {
        let poll = if wait {
            wgpu::PollType::Wait { submission_index: Some(self.submission.clone()), timeout: None }
        } else {
            wgpu::PollType::Poll
        };
        if let Err(e) = device.poll(poll) {
            return Some(Err(e.to_string()));
        }
        match self.state.load(Ordering::SeqCst) {
            WAITING => None,
            FAILED => Some(Err("the GPU could not map the buffer".into())), // i18n-ignore
            _ => {
                let data = self.buf.slice(..).get_mapped_range().map(|v| v.to_vec()).map_err(|e| e.to_string());
                self.buf.unmap();
                Some(data)
            }
        }
    }
}

/// What makes the result from the bytes of the buffers.
type Then<T> = Box<dyn FnOnce(Vec<Vec<u8>>) -> T>;

/// Work waiting for buffers read back from the GPU, then made into a `T` from their bytes.
pub struct Pending<T> {
    reads: Vec<(Readback, Option<Vec<u8>>)>,
    then: Option<Then<T>>,
}

impl<T> Pending<T> {
    pub(crate) fn new(then: impl FnOnce(Vec<Vec<u8>>) -> T + 'static) -> Self {
        Self { reads: Vec::new(), then: Some(Box::new(then)) }
    }

    /// Submit `enc`, which copies into `buf`, and read `buf` back (its bytes come to `then` in
    /// the order the buffers were added).
    pub(crate) fn read(&mut self, queue: &wgpu::Queue, buf: wgpu::Buffer, enc: wgpu::CommandEncoder) {
        self.reads.push((Readback::start(queue, buf, enc), None));
    }

    /// The result once every buffer is in; `wait` waits for the GPU (not in a browser, where
    /// this is called again on a later frame).
    pub fn poll(&mut self, device: &wgpu::Device, wait: bool) -> Option<Result<T, String>> {
        let wait = wait && cfg!(not(target_arch = "wasm32"));
        let mut all = true;
        for (r, got) in &mut self.reads {
            if got.is_none() {
                match r.take(device, wait) {
                    None => all = false,
                    Some(Err(e)) => return Some(Err(e)),
                    Some(Ok(data)) => *got = Some(data),
                }
            }
        }
        if !all {
            return None;
        }
        let then = self.then.take()?;
        Some(Ok(then(self.reads.iter_mut().filter_map(|(_, got)| got.take()).collect())))
    }

    /// Wait for the result (the desktop builds; in a browser an error unless it is already in).
    pub fn wait(mut self, device: &wgpu::Device) -> Result<T, String> {
        self.poll(device, true).unwrap_or_else(|| Err("the GPU has not answered yet".into())) // i18n-ignore
    }
}
