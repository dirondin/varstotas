//! Work off the frame: a thread of its own on the desktop. A browser page has one thread (a
//! thread there panics), so the web build runs the work on it after the frame has been drawn —
//! all at once (`spawn`), or a step at a time with the page drawing between them
//! (`spawn_steps`). The work sends its results as on the desktop (a channel the app polls).
//! It must not wait for the app: in the web build the app does not run until it returns.

/// Run `work` off the frame.
pub fn spawn(work: impl FnOnce() + Send + 'static) {
    #[cfg(not(target_arch = "wasm32"))]
    std::thread::spawn(work);
    #[cfg(target_arch = "wasm32")]
    later(Box::new(work));
}

/// Run `step` until it returns false: in a loop on a thread on the desktop, one call at a time
/// in the web build (the page draws a frame between them now and then).
#[cfg_attr(target_arch = "wasm32", allow(unused_mut))]
pub fn spawn_steps(mut step: impl FnMut() -> bool + Send + 'static) {
    #[cfg(not(target_arch = "wasm32"))]
    std::thread::spawn(move || while step() {});
    #[cfg(target_arch = "wasm32")]
    web::steps(Box::new(step));
}

/// After the current frame (a zero timeout: the browser draws first).
#[cfg(target_arch = "wasm32")]
fn later(work: Box<dyn FnOnce()>) {
    use wasm_bindgen::JsCast;
    let f = wasm_bindgen::closure::Closure::once_into_js(work);
    if let Some(w) = web_sys::window() {
        let _ = w.set_timeout_with_callback_and_timeout_and_arguments_0(f.unchecked_ref(), 0);
    }
}

#[cfg(target_arch = "wasm32")]
mod web {
    use std::time::Duration;
    use web_time::Instant;

    /// Steps run back to back for at most this long before the page gets its turn.
    const SLICE: Duration = Duration::from_millis(30);

    pub fn steps(mut step: Box<dyn FnMut() -> bool>) {
        super::later(Box::new(move || {
            let start = Instant::now();
            while start.elapsed() < SLICE {
                if !step() {
                    return;
                }
            }
            steps(step);
        }));
    }
}
