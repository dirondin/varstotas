//! The file dialogs. The desktop builds use rfd's blocking dialogs. A browser gives files only
//! asynchronously, so a file to open is asked for with a continuation
//! ([`VarstotasApp::pick_file`]): on the desktop it runs at once, in the web build when the file
//! has come in (put into the in-memory file system under `/uploads`). A file to save needs no
//! dialog there: it is written under `/downloads` and the page hands it to the browser as a
//! download (see `web.rs`).

use crate::app::VarstotasApp;
use std::path::PathBuf;

#[cfg(not(target_arch = "wasm32"))]
pub use rfd::FileDialog;

#[cfg(target_arch = "wasm32")]
pub use web::FileDialog;

/// What to do with the picked files.
type Then = Box<dyn FnOnce(&mut VarstotasApp, Vec<PathBuf>)>;

/// A folder for files the app writes (an export with its textures, renders, animation frames):
/// the one picked on the desktop, the downloads in the web build.
pub fn output_folder(d: FileDialog) -> Option<PathBuf> {
    #[cfg(not(target_arch = "wasm32"))]
    return d.pick_folder();
    #[cfg(target_arch = "wasm32")]
    {
        drop(d);
        Some(PathBuf::from(web::DOWNLOADS))
    }
}

impl VarstotasApp {
    /// Ask for a file to open, then go on with it.
    pub(crate) fn pick_file(&mut self, d: FileDialog, then: impl FnOnce(&mut Self, PathBuf) + 'static) {
        self.pick(d, false, Box::new(move |app, mut paths| {
            if let Some(p) = paths.pop() {
                then(app, p);
            }
        }));
    }

    /// Ask for files to open (one or more), then go on with them.
    pub(crate) fn pick_files(&mut self, d: FileDialog, then: impl FnOnce(&mut Self, Vec<PathBuf>) + 'static) {
        self.pick(d, true, Box::new(then));
    }

    fn pick(&mut self, d: FileDialog, many: bool, then: Then) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let paths = if many { d.pick_files() } else { d.pick_file().map(|p| vec![p]) };
            if let Some(paths) = paths.filter(|p| !p.is_empty()) {
                then(self, paths);
            }
        }
        #[cfg(target_arch = "wasm32")]
        web::pick(d, many, then);
    }

    /// Go on with the files that came in since the last frame (the web build; nothing to do on
    /// the desktop, where a dialog answers at once).
    pub(crate) fn picked_files(&mut self) {
        #[cfg(target_arch = "wasm32")]
        for (then, paths) in web::arrived() {
            then(self, paths);
        }
    }
}

#[cfg(target_arch = "wasm32")]
mod web {
    use super::Then;
    use std::cell::RefCell;
    use std::path::{Path, PathBuf};
    use wasm_bindgen::JsCast;
    use wasm_bindgen::closure::Closure;

    /// Where files to save are written (each one becomes a download).
    pub const DOWNLOADS: &str = "/downloads";
    /// Where the files the user opens are put.
    pub const UPLOADS: &str = "/uploads";

    thread_local! {
        static ARRIVED: RefCell<Vec<(Then, Vec<PathBuf>)>> = const { RefCell::new(Vec::new()) };
        /// To draw a frame when files come in.
        pub static CTX: RefCell<Option<egui::Context>> = const { RefCell::new(None) };
    }

    /// The same calls as rfd's dialog; the filters become the browser's `accept`.
    #[derive(Default)]
    pub struct FileDialog {
        exts: Vec<String>,
        file_name: Option<String>,
    }

    impl FileDialog {
        pub fn new() -> Self {
            Self::default()
        }
        pub fn add_filter(mut self, _name: impl Into<String>, extensions: &[impl ToString]) -> Self {
            self.exts.extend(extensions.iter().map(|e| e.to_string()));
            self
        }
        pub fn set_file_name(mut self, name: impl Into<String>) -> Self {
            self.file_name = Some(name.into());
            self
        }
        pub fn set_directory<P: AsRef<Path>>(self, _path: P) -> Self {
            self
        }
        pub fn set_title(self, _title: impl Into<String>) -> Self {
            self
        }
        /// No blocking dialog in a browser: see [`super::VarstotasApp::pick_file`].
        pub fn pick_file(self) -> Option<PathBuf> {
            None
        }
        /// A browser lets a page read no folders.
        pub fn pick_folder(self) -> Option<PathBuf> {
            None
        }
        /// The file is written under the downloads (and downloaded), by the name it was given.
        pub fn save_file(self) -> Option<PathBuf> {
            let name = self.file_name.filter(|n| !n.trim().is_empty()).unwrap_or_else(|| "Untitled".into()); // i18n-ignore
            let name = crate::files::safe_name(&name, crate::files::MODEL);
            Some(Path::new(DOWNLOADS).join(name))
        }
    }

    /// Open the browser's file chooser; the files go into the file system, then to `then`.
    pub fn pick(d: FileDialog, many: bool, then: Then) {
        let Some(document) = web_sys::window().and_then(|w| w.document()) else { return };
        let Ok(input) = document.create_element("input").map(|e| e.unchecked_into::<web_sys::HtmlInputElement>()) else { return };
        input.set_type("file");
        input.set_multiple(many);
        if !d.exts.is_empty() {
            let accept: Vec<String> = d.exts.iter().map(|e| format!(".{e}")).collect();
            input.set_accept(&accept.join(","));
        }
        let then = RefCell::new(Some(then));
        let target = input.clone();
        let on_change = Closure::<dyn FnMut()>::new(move || {
            let Some(then) = then.borrow_mut().take() else { return };
            let Some(list) = target.files() else { return };
            let files: Vec<web_sys::File> = (0..list.length()).filter_map(|i| list.get(i)).collect();
            wasm_bindgen_futures::spawn_local(async move {
                let mut paths = Vec::new();
                for f in files {
                    let Ok(buf) = wasm_bindgen_futures::JsFuture::from(f.array_buffer()).await else { continue };
                    let bytes = js_sys::Uint8Array::new(&buf).to_vec();
                    let path = Path::new(UPLOADS).join(crate::files::safe_name(&f.name(), crate::files::MODEL));
                    varstotas_kernel::fs::insert(&path, bytes);
                    paths.push(path);
                }
                if !paths.is_empty() {
                    ARRIVED.with(|a| a.borrow_mut().push((then, paths)));
                    CTX.with(|c| c.borrow().as_ref().map(egui::Context::request_repaint));
                }
            });
        });
        input.set_onchange(Some(on_change.as_ref().unchecked_ref()));
        // the input lives as long as the page keeps it (until its change)
        on_change.forget();
        input.click();
    }

    pub fn arrived() -> Vec<(Then, Vec<PathBuf>)> {
        ARRIVED.with(|a| std::mem::take(&mut *a.borrow_mut()))
    }
}

#[cfg(target_arch = "wasm32")]
pub use web::CTX;
