//! Files, folders and links open in the system's default program for them (the `open` crate:
//! ShellExecute on Windows, `open` on macOS, xdg-open / gio on Linux, which in a Flatpak hand
//! over to the OpenURI portal). Varstotas never runs a program path the user enters: the
//! slicer is the default app for `.3mf`, the image editor the default one for `.png`.
//! Scripted runs only record the target: another window would cover the app.

use std::ffi::OsStr;
use std::sync::Mutex;

/// What [`open`] opened last (the harness reads it).
pub static LAST_OPENED: Mutex<Option<String>> = Mutex::new(None);

/// Open `target` (a file, a folder or a URL) with the system's default program.
pub fn open(target: impl AsRef<OsStr>) -> std::io::Result<()> {
    let target = target.as_ref();
    if let Ok(mut l) = LAST_OPENED.lock() {
        *l = Some(target.to_string_lossy().into_owned());
    }
    if std::env::var_os("VARSTOTAS_SCRIPT").is_some() {
        log::info!("open {}", target.to_string_lossy());
        return Ok(());
    }
    open::that_detached(target)
}
