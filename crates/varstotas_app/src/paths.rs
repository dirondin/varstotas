//! Where Varstotas keeps its own files: the per-user application data folder of the platform,
//! the one eframe keeps the window state in (`~/Library/Application Support/Varstotas`,
//! `$XDG_DATA_HOME/varstotas` or `~/.local/share/varstotas`, `%APPDATA%\Varstotas\data`; inside
//! the Flatpak `~/.var/app/io.github.dirondin.Varstotas/data/varstotas`). It holds the bridge
//! file and the Python modules, plugins, libraries, templates, user locales and `cache/`.
//! Nothing else outside it is written without the user choosing the place.
//!
//! `VARSTOTAS_DATA_DIR` replaces it (UI tests). Versions up to 0.1 used `~/.varstotas`; its
//! contents are moved here once at startup ([`migrate_legacy`]).

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

/// What is not moved from the old folder: written again by the app, or only a cache.
const NOT_MOVED: [&str; 3] = ["bridge.json", "python", "cache"];

/// Moves the contents of the old `~/.varstotas` into [`data_dir`] once: entries that already
/// exist there are left in place, the old folder is removed when nothing is left in it.
/// Without `VARSTOTAS_DATA_DIR` only. Returns the names moved.
pub fn migrate_legacy() -> Vec<String> {
    if std::env::var_os("VARSTOTAS_DATA_DIR").is_some_and(|d| !d.is_empty()) {
        return Vec::new();
    }
    let (Some(home), Some(to)) = (std::env::home_dir(), data_dir()) else { return Vec::new() };
    move_contents(&home.join(".varstotas"), &to)
}

fn move_contents(from: &Path, to: &Path) -> Vec<String> {
    let Ok(entries) = std::fs::read_dir(from) else { return Vec::new() };
    let mut moved = Vec::new();
    for e in entries.flatten() {
        let name = e.file_name();
        let name_s = name.to_string_lossy().into_owned();
        let dst = to.join(&name);
        if NOT_MOVED.contains(&name_s.as_str()) || dst.exists() {
            continue;
        }
        if std::fs::create_dir_all(to).is_err() {
            break;
        }
        match std::fs::rename(e.path(), &dst) {
            Ok(()) => moved.push(name_s),
            Err(err) => log::warn!("cannot move {} to {}: {err}", e.path().display(), dst.display()),
        }
    }
    // the leftovers that are not moved are regenerated or caches
    for n in NOT_MOVED {
        let p = from.join(n);
        let _ = if p.is_dir() { std::fs::remove_dir_all(&p) } else { std::fs::remove_file(&p) };
    }
    let _ = std::fs::remove_dir(from);
    moved
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn legacy_contents_move_once_without_overwriting() {
        let tmp = std::env::temp_dir().join(format!("varstotas-paths-{}", std::process::id()));
        let (from, to) = (tmp.join("old"), tmp.join("new"));
        let _ = std::fs::remove_dir_all(&tmp);
        for d in ["plugins", "python", "cache/thumbs", "components"] {
            std::fs::create_dir_all(from.join(d)).unwrap();
        }
        std::fs::write(from.join("plugins/a.rhai"), "old").unwrap();
        std::fs::write(from.join("bridge.json"), "{}").unwrap();
        std::fs::write(from.join("sources.toml"), "old").unwrap();
        std::fs::create_dir_all(&to).unwrap();
        std::fs::write(to.join("sources.toml"), "new").unwrap();

        let mut moved = move_contents(&from, &to);
        moved.sort();
        assert_eq!(moved, ["components", "plugins"]);
        assert_eq!(std::fs::read_to_string(to.join("plugins/a.rhai")).unwrap(), "old");
        assert_eq!(std::fs::read_to_string(to.join("sources.toml")).unwrap(), "new", "an existing file is kept");
        assert!(!to.join("bridge.json").exists() && !to.join("python").exists() && !to.join("cache").exists());
        // the old sources.toml is left behind, so the old folder stays
        assert!(from.join("sources.toml").exists() && !from.join("plugins").exists());
        assert!(move_contents(&from, &to).is_empty());
        let _ = std::fs::remove_dir_all(&tmp);
    }

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
