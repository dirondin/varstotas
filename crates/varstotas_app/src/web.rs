//! The web build: the app runs in the page's canvas (`web/index.html`, id `varstotas`) on
//! WebGPU only; a browser without it gets the page's own message instead of a blank canvas.
//! The app's files live in memory (`varstotas_kernel::fs`): its data folder `/data` is kept in
//! the browser's IndexedDB (`web_store.js`), loaded before the app starts; a file written
//! anywhere else but `/tmp` is one the user saved and is downloaded.

use std::path::Path;
use varstotas_kernel::fs::Change;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen(module = "/src/web_store.js")]
extern "C" {
    async fn store_load() -> wasm_bindgen::JsValue;
    fn store_file(path: &str, data: &[u8]);
    fn store_dir(path: &str);
    fn store_delete(path: &str);
}

/// The data folder (see `paths::data_dir`).
pub const DATA: &str = "/data";

/// Start the app in the page (called by the generated JS glue on load).
#[wasm_bindgen(start)]
pub fn start() {
    crate::applog::init();
    varstotas_kernel::fs::set_on_change(on_change);
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
        load_data().await;
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

/// The preferences in a data archive (the page keeps them in its local storage, not in the data
/// folder).
const PREFS_FILE: &str = "preferences.json"; // i18n-ignore

/// The data folder (without its cache) and the preferences as a zip archive, to keep a copy or
/// to take them to another browser.
pub fn data_zip(prefs: &crate::app::Prefs) -> Vec<u8> {
    let mut files = Vec::new();
    collect(Path::new(DATA), &mut files);
    if let Ok(json) = serde_json::to_vec_pretty(prefs) {
        files.push((PREFS_FILE.to_string(), json));
    }
    let entries: Vec<(&str, &[u8])> = files.iter().map(|(n, d)| (n.as_str(), d.as_slice())).collect();
    varstotas_kernel::io::zip_stored(&entries)
}

fn collect(dir: &Path, out: &mut Vec<(String, Vec<u8>)>) {
    let Ok(entries) = varstotas_kernel::fs::read_dir(dir) else { return };
    for e in entries.flatten() {
        let p = e.path();
        if varstotas_kernel::fs::is_dir(&p) {
            if p != Path::new(DATA).join("cache") {
                collect(&p, out);
            }
        } else if let (Ok(data), Ok(name)) = (varstotas_kernel::fs::read(&p), p.strip_prefix(DATA)) {
            out.push((name.to_string_lossy().into_owned(), data));
        }
    }
}

/// Put the files of a data archive ([`data_zip`]) into the data folder, over those of the same
/// name: how many there were, and its preferences.
pub fn restore_data(zip: &[u8]) -> Result<(usize, Option<crate::app::Prefs>), String> {
    let (mut count, mut prefs) = (0, None);
    for (name, data) in varstotas_kernel::io::unzip(zip, |_| true)? {
        if name == PREFS_FILE {
            prefs = serde_json::from_slice(&data).ok();
            continue;
        }
        // only plain names inside the folder (no "..", no absolute paths)
        let inside = Path::new(&name);
        if name.ends_with('/') || !inside.components().all(|c| matches!(c, std::path::Component::Normal(_))) {
            continue;
        }
        varstotas_kernel::fs::write(Path::new(DATA).join(inside), data).map_err(|e| e.to_string())?;
        count += 1;
    }
    Ok((count, prefs))
}

/// Put the data folder kept from earlier visits into the file system.
async fn load_data() {
    let records = js_sys::Array::from(&store_load().await);
    let mut files = 0;
    for i in (0..records.length()).step_by(2) {
        let Some(path) = records.get(i).as_string() else { continue };
        let value = records.get(i + 1);
        if value.is_null() || value.is_undefined() {
            varstotas_kernel::fs::insert_dir(&path);
        } else {
            varstotas_kernel::fs::insert(&path, js_sys::Uint8Array::new(&value).to_vec());
            files += 1;
        }
    }
    log::info!("data folder: {files} files from the browser's storage");
}

/// A change in the data folder goes to the browser's storage; a file written anywhere else but
/// the temporary files is one the user saved or exported: the browser downloads it.
fn on_change(path: &Path, change: Change) {
    if path.starts_with(DATA) {
        let key = path.to_string_lossy();
        match change {
            Change::File(data) => store_file(&key, data),
            Change::Dir => store_dir(&key),
            Change::Removed => store_delete(&key),
        }
        return;
    }
    let Change::File(data) = change else { return };
    if path.starts_with("/tmp") {
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
