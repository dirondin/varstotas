//! The file dialogs. The desktop builds use rfd's blocking dialogs; the web build has none yet
//! (a browser only gives files asynchronously): its dialog takes the same calls and picks
//! nothing, until the web file layer answers them.

#[cfg(not(target_arch = "wasm32"))]
pub use rfd::FileDialog;

#[cfg(target_arch = "wasm32")]
pub use web::FileDialog;

#[cfg(target_arch = "wasm32")]
mod web {
    use std::path::{Path, PathBuf};

    #[derive(Default)]
    pub struct FileDialog;

    impl FileDialog {
        pub fn new() -> Self {
            Self
        }
        pub fn add_filter(self, _name: impl Into<String>, _extensions: &[impl ToString]) -> Self {
            self
        }
        pub fn set_file_name(self, _name: impl Into<String>) -> Self {
            self
        }
        pub fn set_directory<P: AsRef<Path>>(self, _path: P) -> Self {
            self
        }
        pub fn set_title(self, _title: impl Into<String>) -> Self {
            self
        }
        pub fn pick_file(self) -> Option<PathBuf> {
            None
        }
        pub fn pick_files(self) -> Option<Vec<PathBuf>> {
            None
        }
        pub fn pick_folder(self) -> Option<PathBuf> {
            None
        }
        pub fn save_file(self) -> Option<PathBuf> {
            None
        }
    }
}
