//! Menu integration of the AppImage (Linux). An AppImage installs nothing, so the desktop
//! (GNOME, KDE on Wayland) finds no `varstotas.desktop` for the window's app id: the dock shows a
//! generic icon and the bare id "varstotas", and `.vrst` files do not open with it. On the first
//! start of an AppImage the app offers to add the menu entry, the icons and the `.vrst` file
//! type under `~/.local/share` (what install.sh does, with Exec pointing at the AppImage);
//! Preferences → General adds or removes them later. A moved AppImage updates its entry on start.
use crate::app::VarstotasApp;
use egui::{Context, Id, Modal};
use std::path::{Path, PathBuf};
use varstotas_i18n::tr;

const DESKTOP: &str = include_str!("../../../packaging/linux/varstotas.desktop");
const MIME: &str = include_str!("../../../packaging/linux/varstotas-mime.xml");
/// Marks the entry this module wrote: only that one is updated or removed, never a packaged one.
const MARK: &str = "X-Varstotas-AppImage=true";
const ICON_SIZES: [u32; 3] = [64, 128, 256];

/// The running AppImage file (`$APPIMAGE`, set by the AppImage runtime).
pub(crate) fn running() -> Option<PathBuf> {
    if !cfg!(target_os = "linux") {
        return None;
    }
    std::env::var_os("APPIMAGE").map(PathBuf::from).filter(|p| p.is_absolute() && p.is_file())
}

fn data_home() -> Option<PathBuf> {
    std::env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .filter(|p| p.is_absolute())
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".local/share")))
}

fn entry_path() -> Option<PathBuf> {
    data_home().map(|d| d.join("applications/varstotas.desktop"))
}

/// The text of the entry this module wrote, if there is one.
fn our_entry() -> Option<String> {
    entry_path().and_then(|p| std::fs::read_to_string(p).ok()).filter(|t| t.lines().any(|l| l == MARK))
}

/// The AppImage's menu entry is in place.
pub(crate) fn installed() -> bool {
    our_entry().is_some()
}

/// Some `varstotas.desktop` exists (this module's, install.sh's, the .deb's): the desktop
/// already knows the app.
fn known() -> bool {
    let mut dirs: Vec<PathBuf> = data_home().into_iter().collect();
    let system = std::env::var("XDG_DATA_DIRS").ok().filter(|s| !s.is_empty()).unwrap_or_else(|| "/usr/local/share:/usr/share".into());
    dirs.extend(system.split(':').filter(|s| !s.is_empty()).map(PathBuf::from));
    dirs.iter().any(|d| d.join("applications/varstotas.desktop").is_file())
}

/// `path` as one argument of a desktop entry's Exec key: quoted, with the characters the
/// Exec quoting reserves escaped, and backslashes escaped again for the string value.
fn exec_arg(path: &Path) -> String {
    let mut s = String::from("\"");
    for c in path.to_string_lossy().chars() {
        match c {
            '\\' => s.push_str("\\\\\\\\"),
            '"' | '`' | '$' => {
                s.push_str("\\\\");
                s.push(c);
            }
            '%' => s.push_str("%%"),
            c => s.push(c),
        }
    }
    s.push('"');
    s
}

/// The menu entry for the AppImage `app`.
fn entry(app: &Path) -> String {
    let mut out: Vec<String> = DESKTOP
        .lines()
        .map(|l| if l.starts_with("Exec=") { format!("Exec={} %f", exec_arg(app)) } else { l.to_string() })
        .collect();
    out.push(MARK.to_string());
    out.join("\n") + "\n"
}

fn write(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    std::fs::write(path, bytes)
}

/// Let the desktop see the change (best effort: the tools may be missing).
fn refresh(home: &Path) {
    let run = |program: &str, args: &[&std::ffi::OsStr]| {
        let _ = std::process::Command::new(program)
            .args(args)
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status();
    };
    run("update-mime-database", &[home.join("mime").as_os_str()]);
    run("update-desktop-database", &[home.join("applications").as_os_str()]);
    run("gtk-update-icon-cache", &["-q".as_ref(), "-t".as_ref(), home.join("icons/hicolor").as_os_str()]);
}

/// Add the menu entry, icons and file type for the AppImage `app` (or point them at it).
pub(crate) fn install(app: &Path) -> Result<(), String> {
    let home = data_home().ok_or("no home folder")?;
    write(&home.join("applications/varstotas.desktop"), entry(app).as_bytes()).map_err(|e| e.to_string())?;
    write(&home.join("mime/packages/varstotas.xml"), MIME.as_bytes()).map_err(|e| e.to_string())?;
    // the icons inside the AppImage (mounted at $APPDIR while it runs)
    if let Some(appdir) = std::env::var_os("APPDIR").map(PathBuf::from) {
        for s in ICON_SIZES {
            let rel = format!("icons/hicolor/{s}x{s}/apps/varstotas.png");
            if let Ok(png) = std::fs::read(appdir.join("usr/share").join(&rel)) {
                write(&home.join(&rel), &png).map_err(|e| e.to_string())?;
            }
        }
    }
    refresh(&home);
    Ok(())
}

/// Remove what [`install`] added (nothing when the entry is not this module's).
pub(crate) fn remove() -> Result<(), String> {
    let home = data_home().ok_or("no home folder")?;
    if !installed() {
        return Ok(());
    }
    std::fs::remove_file(home.join("applications/varstotas.desktop")).map_err(|e| e.to_string())?;
    let _ = std::fs::remove_file(home.join("mime/packages/varstotas.xml"));
    for s in ICON_SIZES {
        let _ = std::fs::remove_file(home.join(format!("icons/hicolor/{s}x{s}/apps/varstotas.png")));
    }
    refresh(&home);
    Ok(())
}

/// On start: point an existing entry at this AppImage when the file was moved or replaced by a
/// newer one; true when the menu entry should be offered (none yet, never declined).
pub(crate) fn on_start(declined: bool) -> bool {
    let Some(app) = running() else { return false };
    match our_entry() {
        Some(text) if !text.contains(&entry(&app)) => {
            if let Err(e) = install(&app) {
                log::warn!("AppImage menu entry: {e}");
            }
            false
        }
        Some(_) => false,
        None => !declined && !known(),
    }
}

impl VarstotasApp {
    /// The one-time offer to add the AppImage to the applications menu.
    pub(crate) fn appimage_offer(&mut self, ctx: &Context) {
        if !self.appimage_offer || self.show_start {
            return;
        }
        let mut answer = None;
        Modal::new(Id::new("appimage_offer")).show(ctx, |ui| {
            ui.set_max_width(460.0);
            ui.heading(tr!("appimage-offer-title"));
            ui.add_space(6.0);
            ui.label(tr!("appimage-offer-text"));
            ui.add_space(10.0);
            ui.horizontal(|ui| {
                if ui.button(tr!("appimage-offer-add")).clicked() {
                    answer = Some(true);
                }
                if ui.button(tr!("appimage-offer-no")).clicked() {
                    answer = Some(false);
                }
            });
        });
        if let Some(add) = answer {
            self.appimage_offer = false;
            self.prefs.appimage_menu_declined = !add;
            if add {
                self.appimage_menu(true);
            }
        }
    }

    /// Add (`true`) or remove the AppImage's menu entry, with a message.
    pub(crate) fn appimage_menu(&mut self, add: bool) {
        let result = match (add, running()) {
            (true, Some(app)) => install(&app),
            (true, None) => Ok(()),
            (false, _) => remove(),
        };
        match result {
            Ok(()) if add => self.flash(tr!("appimage-added")),
            Ok(()) => self.flash(tr!("appimage-removed")),
            Err(e) => crate::applog::error("AppImage", &tr!("appimage-failed", error = e)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exec_quotes_the_appimage_path() {
        assert_eq!(exec_arg(Path::new("/home/a/Apps/Varstotas.AppImage")), "\"/home/a/Apps/Varstotas.AppImage\"");
        assert_eq!(exec_arg(Path::new("/home/a/My Apps/50% $x\"q")), "\"/home/a/My Apps/50%% \\\\$x\\\\\"q\"");
        let e = entry(Path::new("/opt/V.AppImage"));
        assert!(e.contains("\nExec=\"/opt/V.AppImage\" %f\n"));
        assert!(e.ends_with(&format!("{MARK}\n")));
        assert!(e.contains("Icon=varstotas"));
    }
}
