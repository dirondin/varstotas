//! `VarstotasApp` file handling: new / open / save / export / import, templates, recent
//! files and autosave recovery.

use crate::app::{Action, MAX_RECENT, Pending, VarstotasApp};
use crate::nav;
use crate::tools::{self, ToolId};
use crate::ui::tray::TrayState;
use crate::document::{Document, guarded};
use crate::templates::Template;
use varstotas_i18n::tr;
use varstotas_kernel::io;
use std::path::{Path, PathBuf};

impl VarstotasApp {
    pub(crate) fn guard_dirty(&mut self, p: Pending) {
        if self.doc.dirty {
            self.confirm = Some(p);
        } else {
            self.proceed(p);
        }
    }

    /// Continue a pending operation after the user confirmed (or nothing was dirty).
    pub(crate) fn proceed(&mut self, p: Pending) {
        match p {
            Pending::New => {
                let t = self.prefs.template.unwrap_or(Template::ArchitecturalMm);
                self.new_from_template(t);
            }
            Pending::NewFrom(t) => self.new_from_template(t),
            Pending::NewFromFileTemplate(path) => self.new_from_file_template(&path),
            Pending::Open => {
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter(tr!("app-filter-model"), &["vrst"])
                    .pick_file()
                {
                    self.open_path(&path);
                }
            }
            Pending::OpenPath(path) => self.open_path(&path),
            Pending::Quit => {
                self.allow_close = true;
                self.tray.close_requested = true;
            }
        }
    }

    pub(crate) fn ensure_templates_dir(&self) {
        if let Some(dir) = crate::templates::directory(self.prefs.templates_dir.as_deref())
            && let Err(e) = std::fs::create_dir_all(&dir)
        {
            log::warn!("templates: cannot create {}: {e}", dir.display());
        }
    }

    pub(crate) fn new_from_file_template(&mut self, path: &Path) {
        match guarded(|| crate::templates::load_document(path)) {
            Ok(Ok(doc)) => {
                self.replace_document(doc);
                self.flash(tr!("template-created", path = path.display().to_string()));
            }
            Ok(Err(e)) => self.flash_error(tr!("template-load-failed", path = path.display().to_string(), error = e.to_string())),
            Err(e) => self.flash_error(tr!("template-load-failed", path = path.display().to_string(), error = e)),
        }
    }

    pub(crate) fn new_from_template(&mut self, t: Template) {
        let mut model = t.new_model();
        model.render = self.prefs.new_render;
        self.replace_document(Document::from_model(model, None));
        self.camera = t.camera(nav::startup_camera());
        self.apply_view_defaults();
    }

    pub(crate) fn replace_document(&mut self, doc: Document) {
        self.show_start = false;
        self.show_templates = false;
        // The old document was saved or its changes discarded: its autosave is obsolete.
        if let Some(p) = self.autosave.written.take() {
            self.autosave.worker.delete(p);
        }
        self.autosave.version = doc.model_version;
        self.autosave.last = self.now;
        self.doc = doc;
        self.viewport.invalidate();
        self.tool = tools::make_tool(ToolId::Select);
        self.vcb.clear();
        self.tray = TrayState::default();
        let mut cam = nav::startup_camera();
        // A document with scenes opens with the camera and the style of the first one.
        if let Some(s) = self.doc.model.scenes.first().cloned() {
            self.apply_scene_style(&s);
            cam.eye = s.eye;
            cam.target = s.target;
            cam.up = s.up;
            cam.fov_y_deg = s.fov_y_deg;
            cam.ortho = s.ortho;
            self.camera = cam;
        } else {
            self.camera = nav::zoom_extents_camera(&self.doc.model, &cam, self.aspect());
        }
        self.cam_anim = None;
        self.scene_player = None;
        self.tray.selected_scene = None;
    }

    pub(crate) fn open_path(&mut self, path: &Path) {
        self.show_start = false;
        self.show_templates = false;
        match guarded(|| io::load(path)) {
            Ok(Ok(model)) => {
                self.replace_document(Document::from_model(model, Some(path.to_path_buf())));
                self.add_recent(path);
                self.flash(tr!("app-opened", path = path.display().to_string()));
                if self.autosave.allowed
                    && let Some(r) = crate::autosave::find_recovery(Some(path), self.autosave.dir.as_deref())
                    && !self.autosave.recoveries.iter().any(|x| x.autosave == r.autosave)
                {
                    self.autosave.recoveries.insert(0, r);
                }
            }
            Ok(Err(e)) => self.flash_error(tr!("app-could-not-open", path = path.display().to_string(), error = e.to_string())),
            Err(e) => self.flash_error(tr!("app-could-not-open", path = path.display().to_string(), error = e)),
        }
    }

    pub(crate) fn add_recent(&mut self, path: &Path) {
        self.prefs.recent.retain(|p| p != path);
        self.prefs.recent.insert(0, path.to_path_buf());
        self.prefs.recent.truncate(MAX_RECENT);
    }

    /// Returns true if saved.
    pub(crate) fn save(&mut self) -> bool {
        match self.doc.path.clone() {
            Some(p) => self.save_to(&p),
            None => self.save_as(),
        }
    }

    pub(crate) fn save_as(&mut self) -> bool {
        let Some(mut path) = rfd::FileDialog::new()
            .add_filter(tr!("app-filter-model"), &["vrst"])
            .set_file_name(format!("{}.vrst", self.doc.display_name()))
            .save_file()
        else {
            return false;
        };
        if path.extension().is_none() {
            path.set_extension("vrst");
        }
        self.save_to(&path)
    }

    pub(crate) fn save_to(&mut self, path: &Path) -> bool {
        match guarded(|| {
            match crate::paths::backup_path(path).filter(|_| self.prefs.save_backups) {
                Some(backup) => {
                    if let Some(dir) = backup.parent() {
                        std::fs::create_dir_all(dir)?;
                    }
                    io::save_with_backup(&self.doc.model, path, &backup)
                }
                None => io::save(&self.doc.model, path),
            }
        }) {
            Ok(Ok(())) => {
                self.doc.path = Some(path.to_path_buf());
                self.doc.dirty = false;
                // Clean save: drop the autosave we wrote (untitled or old name) and any stale
                // autosave of the new file.
                if let Some(p) = self.autosave.written.take() {
                    self.autosave.worker.delete(p);
                }
                if let Some(p) = crate::autosave::path_for(Some(path), self.autosave.dir.as_deref()) {
                    self.autosave.worker.delete(p);
                }
                self.autosave.recoveries.retain(|r| r.original.as_deref() != Some(path));
                self.autosave.version = self.doc.model_version;
                self.autosave.last = self.now;
                self.add_recent(path);
                self.flash(tr!("app-saved", path = path.display().to_string()));
                true
            }
            Ok(Err(e)) => {
                self.flash_error(tr!("app-save-failed", error = write_error(e.to_string())));
                false
            }
            Err(e) => {
                self.flash_error(tr!("app-save-failed", error = e));
                false
            }
        }
    }

    pub(crate) fn export(&mut self, ext: &str) {
        let Some(mut path) = rfd::FileDialog::new()
            .add_filter(ext.to_uppercase(), &[ext])
            .set_file_name(format!("{}.{ext}", self.doc.display_name()))
            .save_file()
        else {
            return;
        };
        if path.extension().is_none() {
            path.set_extension(ext);
        }
        // Files beside it (OBJ's .mtl, textures) go next to it where the app may write; a
        // sandbox (Flatpak portal) lets it write only the file the dialog picked, so there the
        // folder for them is asked for too, and the export goes into that folder.
        let sidecars = io::export_bundle(&self.doc.model, &path).is_some_and(|f| f.len() > 1);
        if sidecars && !path.parent().is_some_and(writable_dir) {
            let Some(name) = path.file_name().map(|n| n.to_owned()) else { return };
            let mut dialog = rfd::FileDialog::new().set_title(tr!("app-export-folder", file = name.to_string_lossy().to_string()));
            if let Some(dir) = path.parent() {
                dialog = dialog.set_directory(dir);
            }
            let Some(dir) = dialog.pick_folder() else { return };
            self.export_into(&dir.join(name));
            return;
        }
        self.export_to(&path);
    }

    /// Export with its sidecar files to `path` in a picked folder: one question for every file
    /// it would replace, the model file too (no save dialog asked about it).
    fn export_into(&mut self, path: &Path) {
        let existing: std::collections::HashSet<PathBuf> = io::export_bundle(&self.doc.model, path)
            .unwrap_or_default()
            .into_iter()
            .map(|(p, _)| p)
            .filter(|p| p.exists())
            .collect();
        if !existing.is_empty() {
            let mut files: Vec<String> = existing.iter().map(|p| p.display().to_string()).collect();
            files.sort();
            let yes = rfd::MessageDialog::new()
                .set_title(tr!("app-export-replace-title"))
                .set_description(tr!("app-export-replace-all", files = files.join("\n")))
                .set_buttons(rfd::MessageButtons::YesNo)
                .show()
                == rfd::MessageDialogResult::Yes;
            if !yes {
                return;
            }
        }
        self.export_to_approved(path, Some(&existing));
    }

    /// Export the model to `path`; the format follows the extension.
    pub(crate) fn export_to(&mut self, path: &std::path::Path) {
        self.export_to_approved(path, None);
    }

    pub(crate) fn export_to_approved(&mut self, path: &std::path::Path, approved: Option<&std::collections::HashSet<PathBuf>>) {
        let model = &self.doc.model;
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or_default().to_ascii_lowercase();
        let bundle = io::export_bundle(model, path);
        let collisions: Vec<_> = bundle
            .as_ref()
            .map(|f| {
                f.iter()
                    .skip(1)
                    .filter(|(p, _)| p.exists())
                    .map(|(p, _)| p.display().to_string())
                    .collect()
            })
            .unwrap_or_default();
        let local_approved: std::collections::HashSet<PathBuf> = bundle.as_ref()
            .map(|f| f.iter().filter(|(p, _)| p.exists()).map(|(p, _)| p.clone()).collect()).unwrap_or_default();
        let allow_sidecars = if approved.is_none() && !collisions.is_empty() {
            rfd::MessageDialog::new()
                .set_title(tr!("app-export-replace-title"))
                .set_description(tr!(
                    "app-export-replace-files",
                    files = collisions.join("\n")
                ))
                .set_buttons(rfd::MessageButtons::YesNo)
                .show()
                == rfd::MessageDialogResult::Yes
        } else {
            approved.is_some()
        };
        if !collisions.is_empty() && !allow_sidecars {
            return;
        }
        let approved = approved.unwrap_or(&local_approved);
        let r = guarded(|| {
            if let Some(files) = &bundle {
                return io::write_bundle_approved(files, approved);
            }
            match ext.as_str() {
            "obj" => io::export_obj(model, path),
            "dxf" => io::export_dxf(model, path),
            "glb" => io::export_glb(model, path),
            "3mf" => io::export_3mf(model, path),
            "ply" => io::export_ply(model, path),
            "dae" => io::export_dae(model, path),
            "json" => io::export_faces_json(model, path),
            "csv" => varstotas_kernel::io::atomic_write(path, varstotas_kernel::report::report_csv(model)).map_err(Into::into),
            "txt" => {
                let lines: Vec<String> = model.attributions().iter().map(|a| a.line()).collect();
                varstotas_kernel::io::atomic_write(path, format!("Credits of the 3D models in {}\n\n{}\n", self.doc.display_name(), lines.join("\n"))).map_err(Into::into) // i18n-ignore
            }
            "svg" | "pdf" => {
                let vp = self.viewport.size();
                let d = crate::vector_export::build(model, &self.camera, varstotas_kernel::glam::DVec2::new(vp.x as f64, vp.y as f64), &self.vector_opts);
                self.last_vector = Some((d.lines.len(), d.texts.len()));
                let bytes = if ext == "svg" { crate::vector_export::to_svg(&d).into_bytes() } else { crate::vector_export::to_pdf(&d) };
                varstotas_kernel::io::atomic_write(path, bytes).map_err(Into::into)
            }
            _ => io::export_stl(model, path),
            }
        });
        match r {
            Ok(Ok(())) => match crate::downloads::licence_warning(&self.doc.model.attributions()) {
                Some(w) if ext != "txt" => self.flash_error(tr!("app-exported-warning", path = path.display().to_string(), warning = w)),
                _ => self.flash(tr!("app-exported", path = path.display().to_string())),
            },
            Ok(Err(e)) => self.flash_error(tr!("app-export-failed", error = write_error(e.to_string()))),
            Err(e) => self.flash_error(tr!("app-export-failed", error = e)),
        }
    }

    pub(crate) fn import(&mut self, exts: &[&str], label: &str) {
        let Some(path) = rfd::FileDialog::new().add_filter(label, exts).pick_file() else {
            return;
        };
        self.import_from(&path, label);
    }

    /// Import `path` (format by extension) as one undo step "Import <label>".
    pub(crate) fn import_from(&mut self, path: &std::path::Path, label: &str) {
        self.import_asking(path, label, true);
    }

    /// When files next to the model (textures, `.mtl`, `.bin`) could not be read — in a sandbox
    /// the app sees only the picked file — `ask` offers to choose the model's folder and
    /// imports again from there, in place of the first try.
    fn import_asking(&mut self, path: &std::path::Path, label: &str, ask: bool) {
        io::take_missing_resources();
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or_default().to_ascii_lowercase();
        let r = self.doc.try_commit(&tr!("app-undo-import", format = label), |m| match ext.as_str() {
            "obj" => io::import_obj(m, path),
            "dxf" => io::import_dxf(m, path),
            "glb" | "gltf" => io::import_gltf(m, path),
            "3mf" => io::import_3mf(m, path),
            "dae" => io::import_dae(m, path),
            "fbx" => io::import_fbx(m, path),
            "ply" => io::import_ply(m, path),
            "csv" => io::import_points_csv(m, path).map(|_| ()),
            _ => io::import_stl(m, path),
        });
        let missing = io::take_missing_resources();
        self.import_missing.clone_from(&missing);
        if !missing.is_empty() {
            log::warn!("import {}: missing {}", path.display(), missing.join(", "));
        }
        if !missing.is_empty() && ask && !self.debug.script_active()
            && let Some(folder) = self.ask_model_folder(path, &missing)
        {
            let Some(name) = path.file_name() else { return };
            let again = folder.join(name);
            if !again.is_file() {
                self.flash_error(tr!("app-import-not-in-folder", file = name.to_string_lossy().to_string(), folder = folder.display().to_string()));
            } else {
                if r.is_some() {
                    self.doc.undo_last_commit();
                }
                self.import_asking(&again, label, false);
                return;
            }
        }
        if r.is_some() {
            self.flash(tr!("app-imported", path = path.display().to_string()));
            self.actions.push(Action::ZoomExtents);
        } else if let Some(e) = self.doc.last_error.clone() {
            self.flash_error(e);
        }
    }

    /// "Files next to the model are missing: choose its folder?" → the folder, or None.
    fn ask_model_folder(&self, path: &std::path::Path, missing: &[String]) -> Option<PathBuf> {
        let name = path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
        let mut files: Vec<&str> = missing.iter().take(6).map(String::as_str).collect();
        if missing.len() > files.len() {
            files.push("…");
        }
        let yes = rfd::MessageDialog::new()
            .set_title(tr!("app-import-missing-title"))
            .set_description(tr!("app-import-missing", file = name, files = files.join("\n")))
            .set_buttons(rfd::MessageButtons::YesNo)
            .show()
            == rfd::MessageDialogResult::Yes;
        if !yes {
            return None;
        }
        let mut dialog = rfd::FileDialog::new();
        if let Some(dir) = path.parent() {
            dialog = dialog.set_directory(dir);
        }
        dialog.pick_folder()
    }

    // ---- autosave ----------------------------------------------------------

    /// Write an autosave now (if the document has changes since the last one).
    pub(crate) fn autosave_now(&mut self) {
        if !self.autosave.allowed {
            return;
        }
        let Some(path) = crate::autosave::path_for(self.doc.path.as_deref(), self.autosave.dir.as_deref()) else {
            return;
        };
        if let Some(old) = self.autosave.written.take()
            && old != path
        {
            self.autosave.worker.delete(old);
        }
        self.autosave.worker.write(path.clone(), self.doc.model.clone());
        self.autosave.written = Some(path);
        self.autosave.version = self.doc.model_version;
        self.autosave.last = self.now;
    }

    /// Per-frame autosave timer.
    pub(crate) fn autosave_tick(&mut self, ctx: &egui::Context) {
        for r in self.autosave.worker.poll() {
            if let Err(e) = r.result {
                self.flash_error(tr!("app-autosave-failed", path = r.path.display().to_string(), error = e.clone()));
            }
        }
        if !self.autosave.allowed || !self.prefs.autosave {
            return;
        }
        let minutes = self.prefs.autosave_minutes.max(1);
        // a physics run shows positions that are not the document's yet
        if !self.doc.dirty || self.doc.model_version == self.autosave.version || self.physics_live() {
            // Nothing new to protect: the interval starts at the first unsaved change.
            self.autosave.last = self.now;
            return;
        }
        // Don't hitch an in-progress drag; try again on the next frame.
        if !self.tool_pressed
            && crate::autosave::is_due(
                minutes,
                self.now,
                self.autosave.last,
                self.doc.dirty,
                self.doc.model_version,
                self.autosave.version,
            )
        {
            self.autosave_now();
        }
        let wait = crate::autosave::seconds_until_due(minutes, self.now, self.autosave.last);
        ctx.request_repaint_after(std::time::Duration::from_secs_f64(wait.max(0.5)));
    }

    /// Apply the user's choice for the first pending recovery.
    pub(crate) fn resolve_recovery(&mut self, recover: Option<bool>) {
        if self.autosave.recoveries.is_empty() {
            return;
        }
        let r = self.autosave.recoveries.remove(0);
        match recover {
            Some(true) => match guarded(|| io::load(&r.autosave)) {
                Ok(Ok(model)) => {
                    let mut doc = Document::from_model(model, r.original.clone());
                    doc.dirty = true;
                    doc.model_version = crate::document::next_model_version();
                    self.replace_document(doc);
                    // Keep the autosave until the recovered model is saved (or discarded).
                    self.autosave.written = Some(r.autosave.clone());
                    self.autosave.version = 0;
                    self.flash(tr!("app-recovered", name = r.display_name()));
                }
                Ok(Err(e)) => self.flash_error(tr!("app-could-not-recover", name = r.display_name(), error = e.to_string())),
                Err(e) => self.flash_error(tr!("app-could-not-recover", name = r.display_name(), error = e)),
            },
            Some(false) => self.autosave.worker.delete(r.autosave),
            None => {}
        }
    }
}

/// The app may create files in `dir` (not so in a sandbox that granted one picked file).
fn writable_dir(dir: &Path) -> bool {
    let probe = dir.join(format!(".varstotas-write-test-{}", std::process::id()));
    match std::fs::OpenOptions::new().write(true).create_new(true).open(&probe) {
        Ok(_) => std::fs::remove_file(&probe).is_ok(),
        Err(_) => false,
    }
}

/// A write error, with a hint where Windows refuses the user's own folders: Controlled Folder
/// Access (Defender ransomware protection) blocks unknown apps in Documents, Pictures, Desktop.
fn write_error(error: String) -> String {
    if cfg!(windows) && error.contains("(os error 5)") {
        format!("{error}\n{}", tr!("app-write-denied-windows"))
    } else {
        error
    }
}
