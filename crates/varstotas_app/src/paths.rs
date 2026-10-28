//! Where Varstotas keeps its own files: the per-user application data folder of the platform,
//! the one eframe keeps the window state in (`~/Library/Application Support/Varstotas`,
//! `$XDG_DATA_HOME/varstotas` or `~/.local/share/varstotas`, `%APPDATA%\Varstotas\data`; inside
//! the Flatpak `~/.var/app/io.github.dirondin.Varstotas/data/varstotas`). It holds the bridge
//! file and the Python modules, plugins, libraries, templates, user locales, `cache/`, and the
//! documents' autosaves (`autosave/`) and backups (`backups/`), and the graphics API found to work
//! (`gpu.json`). Nothing else outside it is
//! written without the user choosing the place: a sandbox (Flatpak) lets the app see only the
//! file picked in the dialog, not the folder around it.
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

/// The name of a file kept about the document at `doc`: its file stem and a short hash of its
/// absolute path, so two `house.vrst` in different folders do not share it.
pub fn doc_key(doc: &Path) -> String {
    let abs = std::path::absolute(doc).unwrap_or_else(|_| doc.to_path_buf());
    // FNV-1a: stable across Rust versions, unlike DefaultHasher
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in abs.as_os_str().as_encoded_bytes() {
        h = (h ^ u64::from(*b)).wrapping_mul(0x0100_0000_01b3);
    }
    let stem = doc.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
    format!("{stem} ({:08x})", h as u32)
}

/// Where an explicit save keeps the previous version of `doc`: `backups/` in the data folder.
pub fn backup_path(doc: &Path) -> Option<PathBuf> {
    Some(data_dir()?.join("backups").join(format!("{}.bak.vrst", doc_key(doc)))) // i18n-ignore
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

    #[test]
    fn doc_keys_differ_by_folder() {
        let a = doc_key(Path::new("/a/house.vrst"));
        assert!(a.starts_with("house ("), "{a}");
        assert_eq!(a, doc_key(Path::new("/a/house.vrst")));
        assert_ne!(a, doc_key(Path::new("/b/house.vrst")));
    }
}
