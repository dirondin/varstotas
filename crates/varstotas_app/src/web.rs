//! The web build: the app runs in the page's canvas (`web/index.html`, id `varstotas`) on
//! WebGPU only; a browser without it gets the page's own message instead of a blank canvas.

use wasm_bindgen::JsCast;

/// Start the app in the page (called by the generated JS glue on load).
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn start() {
    crate::applog::init();
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
            .start(canvas, options, Box::new(|cc| Ok(Box::new(crate::app::VarstotasApp::new(cc, None)))))
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

/// Open a link in a new tab.
pub fn open_url(url: &str) -> std::io::Result<()> {
    let opened = web_sys::window().and_then(|w| w.open_with_url_and_target(url, "_blank").ok().flatten());
    opened.map(|_| ()).ok_or_else(|| std::io::Error::other("the browser did not open the link"))
}
