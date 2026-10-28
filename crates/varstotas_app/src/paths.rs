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

/// FNV-1a of the absolute path (stable across Rust versions, unlike DefaultHasher), 32 bits.
fn path_hash(doc: &Path) -> u32 {
    let abs = std::path::absolute(doc).unwrap_or_else(|_| doc.to_path_buf());
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in abs.as_os_str().as_encoded_bytes() {
        h = (h ^ u64::from(*b)).wrapping_mul(0x0100_0000_01b3);
    }
    h as u32
}

/// The name of a file kept about the document at `doc`: its file stem and a short hash of its
/// absolute path, so two `house.vrst` in different folders do not share it.
pub fn doc_key(doc: &Path) -> String {
    let stem = doc.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
    format!("{stem} ({:08x})", path_hash(doc))
}

/// Where an explicit save keeps the previous version of `doc`. `beside`: next to it as
/// `<file>.bak` (`house.vrst.bak`). Otherwise `backups/` in the data folder, named by the file
/// stem, the name of its folder and a hash of the full path (`house — client-a
/// (1a2b3c4d).bak.vrst`): two `house.vrst` in different folders never share a backup, and the
/// folder's name tells them apart at a glance.
pub fn backup_path(doc: &Path, beside: bool) -> Option<PathBuf> {
    if beside {
        let mut name = doc.file_name()?.to_os_string();
        name.push(".bak");
        return Some(doc.with_file_name(name));
    }
    let stem = doc.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
    let abs = std::path::absolute(doc).unwrap_or_else(|_| doc.to_path_buf());
    let folder = abs.parent().and_then(Path::file_name).map(|f| f.to_string_lossy().into_owned());
    let name = match folder {
        Some(f) if !f.is_empty() => format!("{stem} — {f} ({:08x}).bak.vrst", path_hash(doc)), // i18n-ignore
        _ => format!("{stem} ({:08x}).bak.vrst", path_hash(doc)), // i18n-ignore
    };
    Some(data_dir()?.join("backups").join(name))
}

/// `path` for a menu: the home folder as `~`, and when longer than `max` characters the middle
/// folders as `…`, keeping the start and as many of the last folders as fit.
pub fn short(path: &Path, max: usize) -> String {
    let full = display(path);
    if full.chars().count() <= max {
        return full;
    }
    let parts: Vec<String> = Path::new(&full).components().map(|c| c.as_os_str().to_string_lossy().into_owned()).collect();
    let sep = std::path::MAIN_SEPARATOR_STR;
    let Some((first, rest)) = parts.split_first() else { return full };
    let head = if first.ends_with(sep) { first.clone() } else { format!("{first}{sep}") };
    let mut tail: Vec<&str> = Vec::new();
    for p in rest.iter().rev() {
        let len = head.chars().count() + 2 + tail.iter().map(|t| t.chars().count() + 1).sum::<usize>() + p.chars().count();
        if len > max && !tail.is_empty() {
            break;
        }
        tail.insert(0, p);
    }
    if tail.len() == rest.len() {
        return full;
    }
    format!("{head}…{sep}{}", tail.join(sep))
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
    fn backups_beside_or_named_by_folder() {
        assert_eq!(backup_path(Path::new("/a/house.vrst"), true), Some(PathBuf::from("/a/house.vrst.bak")));
        if let Some(b) = backup_path(Path::new("/projects/client-a/house.vrst"), false) {
            let name = b.file_name().unwrap().to_string_lossy().into_owned();
            assert!(name.starts_with("house — client-a (") && name.ends_with(").bak.vrst"), "{name}");
            let other = backup_path(Path::new("/projects/b/client-a/house.vrst"), false).unwrap();
            assert_ne!(b, other, "same names, different folders");
        }
    }

    #[test]
    fn short_paths_keep_the_ends() {
        if cfg!(windows) {
            return;
        }
        let p = Path::new("/srv/work/clients/2026/kitchen/drawings/Untitled.vrst");
        assert_eq!(short(p, 200), "/srv/work/clients/2026/kitchen/drawings/Untitled.vrst");
        assert_eq!(short(p, 33), "/…/kitchen/drawings/Untitled.vrst");
        assert_eq!(short(p, 32), "/…/drawings/Untitled.vrst");
        assert_eq!(short(p, 10), "/…/Untitled.vrst");
    }

    #[test]
    fn doc_keys_differ_by_folder() {
        let a = doc_key(Path::new("/a/house.vrst"));
        assert!(a.starts_with("house ("), "{a}");
        assert_eq!(a, doc_key(Path::new("/a/house.vrst")));
        assert_ne!(a, doc_key(Path::new("/b/house.vrst")));
    }
}
