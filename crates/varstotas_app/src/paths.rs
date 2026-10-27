//! Where Varstotas keeps its own files: the per-user application data folder of the platform,
//! the one eframe keeps the window state in (`~/Library/Application Support/Varstotas`,
//! `$XDG_DATA_HOME/varstotas` or `~/.local/share/varstotas`, `%APPDATA%\Varstotas\data`; inside
//! the Flatpak `~/.var/app/io.github.dirondin.Varstotas/data/varstotas`). It holds the bridge
//! file and the Python modules, plugins, libraries, templates, user locales and `cache/`.
//! Nothing else outside it is written without the user choosing the place.
//!
//! `VARSTOTAS_DATA_DIR` replaces it (UI tests).

use std::path::{Path, PathBuf};

const APP: &str = "Varstotas"; // i18n-ignore

/// The data folder (not created here).
pub fn data_dir() -> Option<PathBuf> {
    if let Some(d) = std::env::var_os("VARSTOTAS_DATA_DIR").filter(|d| !d.is_empty()) {
        return Some(PathBuf::from(d));
    }
    eframe::storage_dir(APP)
}

/// `path` for the UI: the home folder shortened to `~` (except on Windows).
pub fn display(path: &Path) -> String {
    if !cfg!(windows)
        && let Some(home) = std::env::home_dir()
        && let Ok(rest) = path.strip_prefix(&home)
    {
        return Path::new("~").join(rest).display().to_string();
    }
    path.display().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_shortens_home() {
        if cfg!(windows) {
            return;
        }
        let home = std::env::home_dir().unwrap();
        assert_eq!(display(&home.join("x").join("y")), "~/x/y");
        assert_eq!(display(Path::new("/opt/v")), "/opt/v");
    }
}
