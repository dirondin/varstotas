//! The web build: the app runs in the page's canvas (`web/index.html`, id `varstotas`) on
//! WebGPU only; a browser without it gets the page's own message instead of a blank canvas.

use wasm_bindgen::JsCast;

/// Start the app in the page (called by the generated JS glue on load).
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn start() {
    crate::applog::init();
    varstotas_kernel::fs::set_on_write(on_write);
    warn_before_leaving();
    let Some(document) = web_sys::window().and_then(|w| w.document()) else { return };
    let Some(canvas) = document.get_element_by_id("varstotas").and_then(|e| e.dyn_into::<web_sys::HtmlCanvasElement>().ok()) else {
        log::error!("no canvas #varstotas in the page");
        return;
    };
    let mut wgpu_options = egui_wgpu::WgpuConfiguration::default();
    if let egui_wgpu::WgpuSetup::CreateNew(setup) = &mut wgpu_options.wgpu_setup {
        // the renderer reads storage buffers, which WebGL2 does not have
        setup.instance_descriptor.backends = egui_wgpu::wgpu::Backends::BROWSER_WEBGPU;
    }
    let options = eframe::WebOptions { wgpu_options, ..Default::default() };
    wasm_bindgen_futures::spawn_local(async move {
        let started = eframe::WebRunner::new()
            .start(
                canvas,
                options,
                Box::new(|cc| {
                    crate::file_dialog::CTX.with(|c| *c.borrow_mut() = Some(cc.egui_ctx.clone()));
                    Ok(Box::new(crate::app::VarstotasApp::new(cc, None)))
                }),
            )
            .await;
        if let Err(e) = started {
            log::error!("start failed: {e:?}");
            show_failure(&document);
        }
    });
}

/// The page's "this browser has no WebGPU" message (`#no-webgpu`) in place of the canvas.
fn show_failure(document: &web_sys::Document) {
    if let Some(e) = document.get_element_by_id("no-webgpu") {
        let _ = e.remove_attribute("hidden");
    }
}

/// The document has changes that are not saved (set every frame).
pub static DIRTY: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// Closing or reloading the tab with changes not saved asks first (the browser's own question).
fn warn_before_leaving() {
    let Some(window) = web_sys::window() else { return };
    let on_leave = wasm_bindgen::closure::Closure::<dyn FnMut(web_sys::BeforeUnloadEvent)>::new(|e: web_sys::BeforeUnloadEvent| {
        if DIRTY.load(std::sync::atomic::Ordering::Relaxed) {
            e.prevent_default();
            e.set_return_value("");
        }
    });
    let _ = window.add_event_listener_with_callback("beforeunload", on_leave.as_ref().unchecked_ref());
    on_leave.forget();
}

/// Folders the app keeps to itself: its data (the browser's storage) and temporary files.
const PRIVATE: [&str; 2] = ["/data", "/tmp"];

/// A file written anywhere else is one the user saved or exported: the browser downloads it.
fn on_write(path: &std::path::Path, data: &[u8]) {
    if PRIVATE.iter().any(|d| path.starts_with(d)) {
        return;
    }
    let name = path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
    if let Err(e) = download(&name, data) {
        log::error!("download of {name} failed: {e:?}");
    }
}

/// Hand `data` to the browser as a download called `name`.
fn download(name: &str, data: &[u8]) -> Result<(), wasm_bindgen::JsValue> {
    let document = web_sys::window().and_then(|w| w.document()).ok_or("no document")?;
    let parts = js_sys::Array::of1(&js_sys::Uint8Array::from(data));
    let blob = web_sys::Blob::new_with_u8_array_sequence(&parts)?;
    let url = web_sys::Url::create_object_url_with_blob(&blob)?;
    let a: web_sys::HtmlAnchorElement = document.create_element("a")?.dyn_into()?;
    a.set_href(&url);
    a.set_download(name);
    a.click();
    // the browser has taken the file once the click is handled
    let revoke = wasm_bindgen::closure::Closure::once_into_js(move || {
        let _ = web_sys::Url::revoke_object_url(&url);
    });
    web_sys::window().ok_or("no window")?.set_timeout_with_callback_and_timeout_and_arguments_0(revoke.unchecked_ref(), 10_000)?;
    Ok(())
}

/// Open a link in a new tab.
pub fn open_url(url: &str) -> std::io::Result<()> {
    let opened = web_sys::window().and_then(|w| w.open_with_url_and_target(url, "_blank").ok().flatten());
    opened.map(|_| ()).ok_or_else(|| std::io::Error::other("the browser did not open the link"))
}
