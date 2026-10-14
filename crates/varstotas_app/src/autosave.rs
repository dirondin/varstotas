//! Autosave and crash recovery (classic).
//!
//! Every N minutes, if the document has unsaved changes, a copy is written next to the file as
//! `<name>.vrst~autosave` (or `Untitled.vrst~autosave` in the app data dir for untitled
//! documents). The model is cloned on the UI thread; JSON serialization and the file write
//! happen on a background worker that processes writes and deletes strictly in order, so a
//! late write can never resurrect an autosave that was deleted after a clean save.
//! On startup (and when opening a file) a newer autosave is offered for recovery.

use varstotas_kernel::{Model, io};
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{Receiver, Sender, channel};
use std::thread::JoinHandle;
use std::time::SystemTime;

pub const SUFFIX: &str = "~autosave";
pub const UNTITLED_FILE: &str = "Untitled.vrst~autosave";
pub const DEFAULT_MINUTES: u32 = 5;

/// Directory for untitled autosaves: `$VARSTOTAS_AUTOSAVE_DIR`, else the platform app-data dir.
pub fn data_dir() -> Option<PathBuf> {
    if let Some(d) = std::env::var_os("VARSTOTAS_AUTOSAVE_DIR").filter(|d| !d.is_empty()) {
        return Some(PathBuf::from(d));
    }
    eframe::storage_dir("Varstotas")
}

/// Where the autosave of a document lives: next to the file, or in `data_dir` when untitled.
pub fn path_for(doc_path: Option<&Path>, data_dir: Option<&Path>) -> Option<PathBuf> {
    match doc_path {
        Some(p) => {
            let mut name: OsString = p.file_name()?.to_os_string();
            name.push(SUFFIX);
            Some(p.with_file_name(name))
        }
        None => data_dir.map(|d| d.join(UNTITLED_FILE)),
    }
}

/// Is an autosave due? `now` / `last` are seconds; `last` is the time of the previous
/// autosave (or of the last open/save). Only dirty documents changed since the previous
/// autosave are written.
pub fn is_due(minutes: u32, now: f64, last: f64, dirty: bool, version: u64, autosaved_version: u64) -> bool {
    minutes > 0 && dirty && version != autosaved_version && now - last >= minutes as f64 * 60.0
}

/// Seconds until the next autosave check could succeed (for scheduling a repaint).
pub fn seconds_until_due(minutes: u32, now: f64, last: f64) -> f64 {
    (last + minutes as f64 * 60.0 - now).max(0.0)
}

fn mtime(p: &Path) -> Option<SystemTime> {
    std::fs::metadata(p).and_then(|m| m.modified()).ok()
}

/// An autosave that can be recovered.
#[derive(Clone, Debug, PartialEq)]
pub struct Recovery {
    /// The autosave file.
    pub autosave: PathBuf,
    /// The document it belongs to (None = untitled).
    pub original: Option<PathBuf>,
    pub modified: Option<SystemTime>,
}

impl Recovery {
    pub fn display_name(&self) -> String {
        self.original
            .as_ref()
            .and_then(|p| p.file_name())
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_else(|| "Untitled".into())
    }

    /// "5 minutes ago"-style age of the autosave.
    pub fn age_text(&self) -> String {
        let Some(secs) = self.modified.and_then(|t| SystemTime::now().duration_since(t).ok()).map(|d| d.as_secs())
        else {
            return "recently".into();
        };
        match secs {
            0..=59 => "less than a minute ago".into(),
            60..=119 => "1 minute ago".into(),
            120..=3599 => format!("{} minutes ago", secs / 60),
            3600..=7199 => "1 hour ago".into(),
            7200..=86_399 => format!("{} hours ago", secs / 3600),
            _ => format!("{} days ago", secs / 86_400),
        }
    }
}

/// The recoverable autosave for `original` (or the untitled one), if any: it must exist and be
/// newer than the original file (a missing original counts as older).
pub fn find_recovery(original: Option<&Path>, data_dir: Option<&Path>) -> Option<Recovery> {
    let autosave = path_for(original, data_dir)?;
    let modified = mtime(&autosave)?;
    if let Some(orig) = original
        && let Some(orig_time) = mtime(orig)
        && orig_time >= modified
    {
        return None;
    }
    Some(Recovery { autosave, original: original.map(Path::to_path_buf), modified: Some(modified) })
}

/// All recoverable autosaves at startup: the untitled one plus those of recent files.
pub fn startup_recoveries(recent: &[PathBuf], data_dir: Option<&Path>) -> Vec<Recovery> {
    let mut out = Vec::new();
    out.extend(find_recovery(None, data_dir));
    for p in recent {
        if let Some(r) = find_recovery(Some(p), data_dir)
            && !out.iter().any(|x: &Recovery| x.autosave == r.autosave)
        {
            out.push(r);
        }
    }
    out
}

/// Serialize and write atomically (temp file + rename).
pub fn write_model(model: &Model, path: &Path) -> Result<(), String> {
    let json = io::to_json(model).map_err(|e| e.to_string())?;
    if let Some(dir) = path.parent()
        && !dir.as_os_str().is_empty()
    {
        std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    }
    let mut tmp_name = path.file_name().map(OsString::from).unwrap_or_default();
    tmp_name.push(".tmp");
    let tmp = path.with_file_name(tmp_name);
    std::fs::write(&tmp, json).map_err(|e| e.to_string())?;
    std::fs::rename(&tmp, path).map_err(|e| {
        let _ = std::fs::remove_file(&tmp);
        e.to_string()
    })
}

pub fn remove(path: &Path) {
    match std::fs::remove_file(path) {
        Ok(()) => log::info!("autosave: removed {}", path.display()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
        Err(e) => log::warn!("autosave: cannot remove {}: {e}", path.display()),
    }
}

enum Job {
    Write(PathBuf, Box<Model>),
    Delete(PathBuf),
}

/// Result of a background autosave write.
#[derive(Debug)]
pub struct WriteResult {
    pub path: PathBuf,
    pub result: Result<(), String>,
}

/// Background writer. Jobs run in submission order on one thread.
pub struct Autosaver {
    tx: Option<Sender<Job>>,
    results: Receiver<WriteResult>,
    handle: Option<JoinHandle<()>>,
}

impl Default for Autosaver {
    fn default() -> Self {
        Self::new()
    }
}

impl Autosaver {
    pub fn new() -> Self {
        let (tx, rx) = channel::<Job>();
        let (rtx, results) = channel();
        let handle = std::thread::Builder::new()
            .name("varstotas-autosave".into())
            .spawn(move || {
                for job in rx {
                    match job {
                        Job::Write(path, model) => {
                            let t0 = std::time::Instant::now();
                            let result = crate::document::guarded(|| write_model(&model, &path))
                                .unwrap_or_else(Err);
                            match &result {
                                Ok(()) => log::info!("autosave: wrote {} in {:?}", path.display(), t0.elapsed()),
                                Err(e) => log::warn!("autosave: {} failed: {e}", path.display()),
                            }
                            let _ = rtx.send(WriteResult { path, result });
                        }
                        Job::Delete(path) => remove(&path),
                    }
                }
            })
            .ok();
        Self { tx: handle.as_ref().map(|_| tx), results, handle }
    }

    /// Queue a write of `model` (already cloned by the caller) to `path`.
    pub fn write(&self, path: PathBuf, model: Model) {
        match &self.tx {
            Some(tx) => {
                let _ = tx.send(Job::Write(path, Box::new(model)));
            }
            None => {
                if let Err(e) = write_model(&model, &path) {
                    log::warn!("autosave: {} failed: {e}", path.display());
                }
            }
        }
    }

    /// Queue removal of an autosave file (ordered after pending writes).
    pub fn delete(&self, path: PathBuf) {
        match &self.tx {
            Some(tx) => {
                let _ = tx.send(Job::Delete(path));
            }
            None => remove(&path),
        }
    }

    /// Finished writes since the last call.
    pub fn poll(&self) -> Vec<WriteResult> {
        self.results.try_iter().collect()
    }

    /// Finish all queued jobs (called on exit).
    pub fn flush(&mut self) {
        self.tx = None;
        if let Some(h) = self.handle.take() {
            let _ = h.join();
        }
    }
}

impl Drop for Autosaver {
    fn drop(&mut self) {
        self.flush();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use varstotas_kernel::glam::DVec3;

    fn tmp_dir(name: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("varstotas_autosave_test_{name}_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    fn sample_model() -> Model {
        let mut m = Model::new();
        m.root_mesh_mut().add_face(&[
            DVec3::new(0.0, 0.0, 0.0),
            DVec3::new(1000.0, 0.0, 0.0),
            DVec3::new(1000.0, 500.0, 0.0),
            DVec3::new(0.0, 500.0, 0.0),
        ]);
        m
    }

    fn set_mtime(p: &Path, t: SystemTime) {
        let f = std::fs::File::options().write(true).open(p).unwrap();
        f.set_modified(t).unwrap();
    }

    #[test]
    fn autosave_paths() {
        let p = path_for(Some(Path::new("/a/b/house.vrst")), None).unwrap();
        assert_eq!(p, PathBuf::from("/a/b/house.vrst~autosave"));
        let u = path_for(None, Some(Path::new("/data"))).unwrap();
        assert_eq!(u, PathBuf::from("/data/Untitled.vrst~autosave"));
        assert_eq!(path_for(None, None), None);
    }

    #[test]
    fn due_logic() {
        // Not dirty / unchanged since last autosave / interval not elapsed / disabled.
        assert!(!is_due(5, 1000.0, 0.0, false, 7, 1));
        assert!(!is_due(5, 1000.0, 0.0, true, 7, 7));
        assert!(!is_due(5, 299.0, 0.0, true, 7, 1));
        assert!(!is_due(0, 1e9, 0.0, true, 7, 1));
        assert!(is_due(5, 300.0, 0.0, true, 7, 1));
        assert!(is_due(1, 160.0, 100.0, true, 2, 1));
        assert_eq!(seconds_until_due(5, 100.0, 0.0), 200.0);
        assert_eq!(seconds_until_due(5, 1000.0, 0.0), 0.0);
    }

    #[test]
    fn recovery_requires_newer_autosave() {
        let d = tmp_dir("recovery");
        let file = d.join("house.vrst");
        let auto = path_for(Some(&file), None).unwrap();
        // No autosave: nothing to recover.
        std::fs::write(&file, "x").unwrap();
        assert_eq!(find_recovery(Some(&file), None), None);
        // Autosave older than the file: ignored.
        std::fs::write(&auto, "y").unwrap();
        let now = SystemTime::now();
        set_mtime(&auto, now - std::time::Duration::from_secs(120));
        set_mtime(&file, now - std::time::Duration::from_secs(60));
        assert_eq!(find_recovery(Some(&file), None), None);
        // Autosave newer: offered.
        set_mtime(&auto, now);
        let r = find_recovery(Some(&file), None).unwrap();
        assert_eq!(r.autosave, auto);
        assert_eq!(r.display_name(), "house.vrst");
        // Original deleted: still offered.
        std::fs::remove_file(&file).unwrap();
        assert!(find_recovery(Some(&file), None).is_some());
        // Untitled autosave in the data dir.
        assert!(find_recovery(None, Some(&d)).is_none());
        std::fs::write(d.join(UNTITLED_FILE), "z").unwrap();
        let all = startup_recoveries(&[file.clone(), file.clone()], Some(&d));
        assert_eq!(all.len(), 2, "{all:?}");
        assert_eq!(all[0].original, None);
        assert_eq!(all[1].original.as_deref(), Some(file.as_path()));
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn worker_writes_then_deletes_in_order() {
        let d = tmp_dir("worker");
        let auto = d.join("sub").join(UNTITLED_FILE);
        let mut w = Autosaver::new();
        w.write(auto.clone(), sample_model());
        w.flush();
        let loaded = io::load(&auto).expect("autosave is a valid .vrst file");
        assert_eq!(loaded.root_mesh().faces.len(), 1);
        assert!(!d.join("sub").join(format!("{UNTITLED_FILE}.tmp")).exists());

        // A delete queued right after a write must win.
        let mut w = Autosaver::new();
        w.write(auto.clone(), sample_model());
        w.delete(auto.clone());
        w.flush();
        assert!(!auto.exists());
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn worker_reports_results() {
        let d = tmp_dir("results");
        let auto = d.join(UNTITLED_FILE);
        let w = Autosaver::new();
        w.write(auto.clone(), sample_model());
        let t0 = std::time::Instant::now();
        let mut got = Vec::new();
        while got.is_empty() && t0.elapsed().as_secs() < 10 {
            got = w.poll();
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        assert_eq!(got.len(), 1);
        assert_eq!(got[0].path, auto);
        assert!(got[0].result.is_ok());
        let _ = std::fs::remove_dir_all(&d);
    }
}
