//! The app side of the bridge (`bridge.rs`): executes calls from Python scripts and AI agents
//! on the UI thread, asks the user before destructive ones, and runs `.py` files.

use crate::app::{ExportContent, VarstotasApp};
use crate::bridge::{Bridge, Request};
use crate::nav;
use crate::ui::console::LogLine;
use serde_json::{Value, json};
use std::collections::{HashSet, VecDeque};
use std::io::{BufRead, Read};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::mpsc::{Receiver, Sender, sync_channel};
use varstotas_kernel::Model;
use varstotas_kernel::glam::DVec3;
use varstotas_render::StandardView;

/// An operation opened by `begin`: one undo step for everything until `commit`.
pub struct Txn {
    conn: u64,
    label: String,
    model: Model,
    base_version: u64,
    path: varstotas_kernel::ContextPath,
    selection: Option<Vec<varstotas_kernel::EntityRef>>,
    started: std::time::Instant,
    changed: bool,
}

impl Txn {
    fn commit(self, doc: &mut crate::document::Document) -> Result<bool, String> {
        if self.base_version != doc.model_version || self.path != doc.active_path {
            return Err("document changed while operation was open; operation discarded".into());
        }
        if self.changed {
            let before = std::mem::replace(&mut doc.model, self.model);
            doc.record_step(&self.label, before);
        }
        if let Some(sel) = self.selection { doc.set_selection(sel); }
        Ok(self.changed)
    }
}

/// A screenshot waiting for the next paint.
struct Shot {
    path: PathBuf,
    base64: bool,
    reply: Sender<Result<Value, String>>,
}

/// A `.py` file started from the app; its output goes to the Script Console.
pub struct PythonRun {
    child: Child,
    out: Receiver<(bool, String)>,
    name: String,
    text: String,
    started: std::time::Instant,
    exceeded: std::sync::Arc<std::sync::atomic::AtomicBool>,
}

#[derive(Default)]
pub struct BridgeState {
    pub server: Option<Bridge>,
    pub txn: Option<Txn>,
    /// Destructive calls waiting for the user's answer.
    pub confirm: VecDeque<(String, Request, u64, Option<HashSet<PathBuf>>)>,
    shots: Vec<Shot>,
    pub python: Option<PythonRun>,
    /// Calls served (for the script harness dump).
    pub calls: u64,
    /// The script harness's answer to the pending confirmation.
    pub answer: Option<bool>,
}

enum Reply {
    Done(Result<Value, String>),
    /// Answered later (screenshot).
    Later,
    /// Ask the user first.
    Confirm(String, Option<HashSet<PathBuf>>),
}

impl Reply {
    fn confirm(message: String) -> Self { Self::Confirm(message, None) }
}

fn vec(v: &Value) -> Option<DVec3> {
    let a = v.as_array()?;
    let c = |i: usize| a.get(i).and_then(Value::as_f64);
    Some(DVec3::new(c(0)?, c(1)?, c(2).unwrap_or(0.0)))
}

fn jvec(v: DVec3) -> Value {
    json!([v.x, v.y, v.z])
}

fn base64(bytes: &[u8]) -> String {
    const T: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut s = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for c in bytes.chunks(3) {
        let n = (c[0] as u32) << 16 | (*c.get(1).unwrap_or(&0) as u32) << 8 | *c.get(2).unwrap_or(&0) as u32;
        for i in 0..4 {
            if i <= c.len() {
                s.push(T[(n >> (18 - 6 * i) & 63) as usize] as char);
            } else {
                s.push('=');
            }
        }
    }
    s
}

fn view_by_name(s: &str) -> Option<StandardView> {
    Some(match s {
        "iso" => StandardView::Iso,
        "top" => StandardView::Top,
        "bottom" => StandardView::Bottom,
        "front" => StandardView::Front,
        "back" => StandardView::Back,
        "left" => StandardView::Left,
        "right" => StandardView::Right,
        _ => return None,
    })
}

impl VarstotasApp {
    /// Start the bridge server (unless disabled with `VARSTOTAS_NO_BRIDGE`).
    pub(crate) fn start_bridge(&mut self, ctx: &egui::Context) {
        if std::env::var_os("VARSTOTAS_NO_BRIDGE").is_some_and(|v| !v.is_empty()) {
            return;
        }
        match Bridge::start(ctx.clone()) {
            Ok(b) => self.bridge.server = Some(b),
            Err(e) => log::warn!("bridge: cannot listen: {e}"),
        }
    }

    pub(crate) fn stop_bridge(&mut self) {
        if let Some(b) = &self.bridge.server {
            b.remove_file();
        }
        if let Some(p) = self.bridge.python.as_mut() {
            varstotas_script::process::terminate(&mut p.child);
        }
    }

    /// Serve calls that arrived, show the confirmation dialog, collect Python output.
    pub(crate) fn bridge_tick(&mut self, ctx: &egui::Context) {
        if self
            .bridge
            .txn
            .as_ref()
            .is_some_and(|t| t.started.elapsed().as_secs() > 300)
        {
            self.bridge.txn = None;
        }
        let reqs = self.bridge.server.as_ref().map(Bridge::poll).unwrap_or_default();
        for req in reqs {
            self.bridge.calls += 1;
            match self.bridge_exec(&req, false, None, ctx) {
                Reply::Done(r) => {
                    let _ = req.reply.send(r);
                }
                Reply::Later => {}
                Reply::Confirm(what, approved) => {
                    if self.bridge.confirm.len() >= 16 {
                        let _ = req.reply.send(Err("too many pending confirmations".into()));
                    } else {
                        self.bridge.confirm.push_back((what, req, self.doc.model_version, approved));
                    }
                }
            }
        }
        if let Some(a) = self.bridge.answer.take() {
            self.bridge_answer(a, ctx);
        }
        self.bridge_confirm_dialog(ctx);
        self.python_tick(ctx);
    }

    /// Answer the oldest confirmation (dialog button or script harness).
    pub(crate) fn bridge_answer(&mut self, allow: bool, ctx: &egui::Context) {
        let Some((_, req, version, approved)) = self.bridge.confirm.pop_front() else { return;
        };
        let r = if !allow { Reply::Done(Err("the user declined".into())) }
            else if req.method == "export" && version != self.doc.model_version {
                Reply::Done(Err("document changed while export confirmation was open; retry export".into()))
            } else { self.bridge_exec(&req, true, approved.as_ref(), ctx) };
        match r {
            Reply::Done(r) => {
                let _ = req.reply.send(r);
            }
            Reply::Later => {}
            Reply::Confirm(_, _) => {
                let _ = req.reply.send(Err("confirmation loop".into()));
            }
        }
    }

    fn bridge_confirm_dialog(&mut self, ctx: &egui::Context) {
        let Some((what, _, _, _)) = self.bridge.confirm.front() else { return;
        };
        let what = what.clone();
        let mut answer = None;
        egui::Modal::new(egui::Id::new("bridge_confirm")).show(ctx, |ui| {
            ui.set_width(380.0);
            ui.heading("External script");
            ui.add_space(4.0);
            ui.label(format!("A script or AI agent connected to Varstotas wants to:\n\n{what}"));
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                if ui.button("Allow").clicked() {
                    answer = Some(true);
                }
                if ui.button("Deny").clicked() {
                    answer = Some(false);
                }
            });
        });
        if let Some(a) = answer {
            self.bridge_answer(a, ctx);
        }
    }

    /// A finished viewport PNG: answer the screenshot waiting for it. True when it was one.
    pub(crate) fn bridge_shot_done(&mut self, r: &Result<PathBuf, String>) -> bool {
        let Some(i) = self.bridge.shots.iter().position(|s| r.as_ref().map_or(true, |p| *p == s.path)) else {
            return false;
        };
        let s = self.bridge.shots.remove(i);
        let out = match r {
            Ok(p) => {
                let mut v = json!({ "path": p.to_string_lossy() });
                if s.base64 {
                    match std::fs::read(p) {
                        Ok(b) => v["png_base64"] = Value::String(base64(&b)),
                        Err(e) => {
                            let _ = s.reply.send(Err(e.to_string()));
                            return true;
                        }
                    }
                }
                Ok(v)
            }
            Err(e) => Err(e.clone()),
        };
        let _ = s.reply.send(out);
        true
    }

    fn script_ctx(&self) -> varstotas_script::ScriptCtx {
        varstotas_script::ScriptCtx { path: self.doc.active_path.clone(), selection: self.doc.selection.clone(), time_limit: None,
            trusted_io: false,
        }
    }

    fn bridge_exec(&mut self, req: &Request, confirmed: bool, approved: Option<&HashSet<PathBuf>>, ctx: &egui::Context) -> Reply {
        let p = &req.params;
        let s = |k: &str| p.get(k).and_then(Value::as_str);
        let in_txn = self.bridge.txn.is_some();
        let done = Reply::Done;
        if self.bridge.txn.as_ref().is_some_and(|t| t.conn != req.conn)
            && matches!(
                req.method.as_str(),
                "eval"
                    | "begin"
                    | "commit"
                    | "abort"
                    | "undo"
                    | "redo"
                    | "save"
                    | "export"
                    | "open"
                    | "new"
            )
        {
            return done(Err("operation belongs to another connection".into()));
        }
        match req.method.as_str() {
            "hello" => done(Ok(json!({
                "app": "Varstotas",
                "version": env!("CARGO_PKG_VERSION"),
                "api": varstotas_script::api_names(),
            }))),
            "docs" => done(Ok(Value::String(include_str!("../../../docs/SCRIPTING.md").into(),
            ))),
            "info" => {
                let m = self
                    .bridge
                    .txn
                    .as_ref()
                    .filter(|t| t.conn == req.conn)
                    .map_or(&self.doc.model, |t| &t.model);
                done(Ok(self.bridge_info(m)))
            }
            "eval" => {
                let code = s("code").unwrap_or_default().to_string();
                let vars = p.get("vars").and_then(Value::as_object).cloned().unwrap_or_default();
                let label = s("label").unwrap_or("Python").to_string();
                let mut sc = self.script_ctx();
                if let Some(t) = &self.bridge.txn {
                    sc.path = t.path.clone();
                    if let Some(sel) = &t.selection {
                        sc.selection = sel.clone();
                    }
                }
                let r = if let Some(t) = self.bridge.txn.as_mut() {
                    let r = varstotas_script::eval_json(&mut t.model, &sc, &code, &vars);
                    t.changed |= r.run.changed;
                    if let Some(sel) = &r.run.selection {
                        t.selection = Some(sel.clone());
                    }
                    Some(r)
                } else {
                    self.doc.try_commit_if(
                        &label,
                        |m| Ok::<_, String>(varstotas_script::eval_json(m, &sc, &code, &vars)),
                        |r| r.run.changed,
                    )
                };
                let Some(r) = r else {
                    return done(Err(self.doc.last_error.take().unwrap_or_else(|| "script failed".into())));
                };
                if !in_txn && let Some(sel) = r.run.selection.clone() {
                    self.doc.set_selection(sel);
                }
                done(match r.run.error {
                    Some(e) if r.run.output.is_empty() => Err(e),
                    Some(e) => Err(format!("{}{e}", r.run.output)),
                    None => Ok(json!({ "value": r.value, "output": r.run.output, "changed": r.run.changed }),
                    ),
                })
            }
            "begin" => {
                if in_txn {
                    return done(Err("an operation is already open".into()));
                }
                let label = s("label").unwrap_or("Python").to_string();
                self.bridge.txn = Some(Txn { conn: req.conn, label,
                    model: self.doc.model.clone(),
                    base_version: self.doc.model_version,
                    path: self.doc.active_path.clone(),
                    selection: None,
                    started: std::time::Instant::now(),
                    changed: false,
                });
                done(Ok(Value::Null))
            }
            "commit" => match self.bridge.txn.take() {
                Some(t) => done(t.commit(&mut self.doc).map(|changed| json!(changed))),
                None => done(Err("no open operation".into())),
            },
            "abort" | "$disconnect" => {
                self.bridge.txn.take_if(|t| t.conn == req.conn);
                done(Ok(Value::Null))
            }
            "undo" | "redo" if in_txn => done(Err("finish the open operation first".into())),
            "undo" => done(Ok(self.doc.undo().map_or(Value::Null, Value::String))),
            "redo" => done(Ok(self.doc.redo().map_or(Value::Null, Value::String))),
            "view" => {
                let name = s("name").unwrap_or_default().to_ascii_lowercase();
                self.cam_anim = None;
                if name == "zoom_extents" {
                    self.camera = nav::zoom_extents_camera(&self.doc.model, &self.camera, self.aspect());
                } else if let Some(v) = view_by_name(&name) {
                    self.camera = nav::standard_view_camera(&self.doc.model, &self.camera, v, self.aspect());
                } else {
                    return done(Err(format!("unknown view {name}")));
                }
                done(Ok(self.camera_json()))
            }
            "camera" => {
                self.cam_anim = None;
                let c = &mut self.camera;
                if let Some(v) = p.get("eye").and_then(vec) {
                    c.eye = v;
                }
                if let Some(v) = p.get("target").and_then(vec) {
                    c.target = v;
                }
                if let Some(v) = p.get("up").and_then(vec) {
                    c.up = v;
                }
                if let Some(f) = p.get("fov").and_then(Value::as_f64) {
                    c.fov_y_deg = f.clamp(1.0, 150.0);
                }
                if let Some(b) = p.get("parallel").and_then(Value::as_bool) {
                    c.set_ortho(b);
                }
                done(Ok(self.camera_json()))
            }
            "screenshot" => {
                let path = match s("path") {
                    Some(x) => PathBuf::from(x),
                    None => match crate::bridge::random_token() {
                        Ok(token) => {
                            std::env::temp_dir().join(format!("varstotas-shot-{token}.png"))
                        }
                        Err(e) => return done(Err(e.to_string())),
                },
                };
                if !confirmed && path.exists() {
                    return Reply::confirm(format!("Overwrite the file\n{}", path.display()));
                }
                self.bridge_png(path, p, ctx, req.reply.clone(), p.get("base64").and_then(Value::as_bool).unwrap_or(false),
                )
            }
            "save" | "export" | "open" | "new" if in_txn => {
                done(Err("finish the open operation first".into()))
            }
            "save" => {
                let path = match s("path") {
                    Some(x) => PathBuf::from(x),
                    None => match self.doc.path.clone() {
                        Some(x) => x,
                        None => {
                            return done(Err("the document has no file yet: pass a path".into()));
                        }
                    },
                };
                if !confirmed && path.exists() && self.doc.path.as_deref() != Some(&path) {
                    return Reply::confirm(format!("Save the model over the existing file\n{}", path.display()));
                }
                if self.save_to(&path) { done(Ok(json!(path.to_string_lossy()))) } else { done(Err(self.flash_text("save failed"))) }
            }
            "export" => {
                let Some(path) = s("path").map(PathBuf::from) else { return done(Err("missing path".into()));
                };
                let files = varstotas_kernel::io::export_bundle(&self.doc.model, &path);
                let collisions: Vec<_> = files
                    .as_ref()
                    .map(|files| {
                        files
                            .iter()
                            .map(|(p, _)| p)
                            .filter(|p| p.exists())
                            .collect()
                    })
                    .unwrap_or_else(|| if path.exists() { vec![&path] } else { vec![] });
                if !confirmed && !collisions.is_empty() {
                    let message = format!("Overwrite existing export files\n{}", collisions.iter().map(|p| p.display().to_string()).collect::<Vec<_>>().join("\n"));
                    let paths = collisions.iter().map(|p| (*p).clone()).collect();
                    return Reply::Confirm(message, Some(paths));
                }
                let ext = path.extension().and_then(|e| e.to_str()).unwrap_or_default().to_ascii_lowercase();
                if ext == "png" {
                    return self.bridge_png(path, p, ctx, req.reply.clone(), false);
                }
                if !["stl", "obj", "dxf", "glb", "3mf", "dae", "ply", "json", "csv", "svg", "pdf",
                ].contains(&ext.as_str()) {
                    return done(Err(format!("unknown export format .{ext}")));
                }
                self.flash = None;
                let empty_approved = HashSet::new();
                self.export_to_approved(&path, Some(approved.unwrap_or(&empty_approved)));
                match &self.flash {
                    Some((m, _, _)) if m.starts_with("Exported") => {
                        done(Ok(json!(path.to_string_lossy())))
                    }
                    _ => done(Err(self.flash_text("export failed"))),
                }
            }
            "open" | "new" => {
                if !confirmed && self.doc.dirty {
                    let what = match s("path") {
                        Some(x) if req.method == "open" => format!("Open {x}"),
                        _ => "Start a new document".into(),
                    };
                    return Reply::confirm(format!("{what}, discarding the unsaved changes of “{}”", self.doc.display_name()));
                }
                if req.method == "new" {
                    let t = self.prefs.template.unwrap_or(crate::templates::Template::ArchitecturalMm);
                    self.new_from_template(t);
                    return done(Ok(Value::Null));
                }
                let Some(path) = s("path").map(PathBuf::from) else { return done(Err("missing path".into()));
                };
                self.flash = None;
                self.open_path(&path);
                if self.doc.path.as_deref() == Some(&path) {
                    self.actions.push(crate::app::Action::ZoomExtents);
                    done(Ok(json!(path.to_string_lossy())))
                } else {
                    done(Err(self.flash_text("open failed")))
                }
            }
            m => done(Err(format!("unknown method {m}"))),
        }
    }

    fn flash_text(&self, default: &str) -> String {
        self.flash.as_ref().map_or_else(|| default.to_string(), |f| f.0.clone())
    }

    fn bridge_png(&mut self, path: PathBuf, p: &Value, ctx: &egui::Context, reply: Sender<Result<Value, String>>, base64: bool,
    ) -> Reply {
        if !self.bridge.shots.is_empty() {
            return Reply::Done(Err("too many pending screenshots".into()));
        }
        if std::fs::symlink_metadata(&path).is_ok_and(|m| m.file_type().is_symlink()) {
            return Reply::Done(Err("refusing to overwrite a symbolic link".into()));
        }
        let ppp = ctx.pixels_per_point();
        let vp = self.viewport.size() * ppp;
        let dim = |k: &str, d: f32| {
            p.get(k).and_then(Value::as_u64).map_or(d.round() as u32, |v| v.clamp(16, 4096) as u32).clamp(16, 4096)
        };
        let size = [dim("width", vp.x.max(16.0)), dim("height", vp.y.max(16.0))];
        self.export_png_to(path.clone(), size, ExportContent::default());
        self.bridge.shots.push(Shot { path, base64, reply,
        });
        ctx.request_repaint();
        Reply::Later
    }

    fn camera_json(&self) -> Value {
        let c = &self.camera;
        json!({ "eye": jvec(c.eye), "target": jvec(c.target), "up": jvec(c.up), "fov": c.fov_y_deg, "parallel": c.ortho })
    }

    fn bridge_info(&self, m: &Model) -> Value {
        let root = m.root_mesh();
        let id = |r| varstotas_script::entity_id(&self.doc.active_path, r);
        let bounds = m.bounds().map(|b| json!({ "min": jvec(b.min), "max": jvec(b.max), "size": jvec(b.max - b.min) }),
        );
        let groups: Vec<Value> = root
            .instances
            .iter()
            .map(|(k, i)| json!({ "id": varstotas_script::entity_id(&Vec::new(), varstotas_kernel::EntityRef::Instance(k)), "name": i.name, "definition": m.defs.get(i.def).map(|d| d.name.clone()) }))
            .take(200)
            .collect();
        json!({
            "document": self.doc.display_name(),
            "path": self.doc.path.as_ref().map(|p| p.to_string_lossy()),
            "unsaved_changes": self.doc.dirty,
            "units": format!("{:?}", m.unit).to_lowercase(),
            "root": { "faces": root.faces.len(), "edges": root.edges.len(), "groups": root.instances.len() },
            "definitions": m.defs.len(),
            "bounds": bounds,
            "groups": groups,
            "materials": m.materials.values().map(|x| x.name.clone()).collect::<Vec<_>>(),
            "tags": m.tags.values().map(|x| x.name.clone()).collect::<Vec<_>>(),
            "context_depth": self.doc.active_path.len(),
            "selection": self.doc.selection.iter().map(|r| id(*r)).collect::<Vec<_>>(),
            "undo": self.doc.history.undo_label(),
            "camera": self.camera_json(),
        })
    }

    // ---- Python files -------------------------------------------------------------

    /// Run a `.py` file with `python3`; it connects back through the bridge.
    pub(crate) fn run_python_file(&mut self, path: &Path) {
        self.run_python_with(path, None);
    }

    /// [`Self::run_python_file`] with plugin values (JSON) in `VARSTOTAS_PARAMS`.
    pub(crate) fn run_python_with(&mut self, path: &Path, params: Option<String>) {
        let name = path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
        let Some(b) = &self.bridge.server else {
            self.flash_error("Python scripts need the bridge (disabled by VARSTOTAS_NO_BRIDGE)");
            return;
        };
        if self.bridge.python.is_some() {
            self.flash_error("A Python script is already running");
            return;
        }
        let py = std::env::var("VARSTOTAS_PYTHON").unwrap_or_else(|_| {
            if cfg!(windows) { "python".into() } else { "python3".into()
            }
        });
        let mut pypath = crate::bridge::python_dir().map(|d| d.into_os_string()).unwrap_or_default();
        if let Some(old) = std::env::var_os("PYTHONPATH").filter(|v| !v.is_empty()) {
            pypath.push(if cfg!(windows) { ";" } else { ":" });
            pypath.push(old);
        }
        let mut cmd = Command::new(&py);
        cmd.arg("-u")
            .arg(path)
            .env("VARSTOTAS_BRIDGE", format!("127.0.0.1:{}", b.port))
            .env("VARSTOTAS_TOKEN", &b.token)
            .env("PYTHONPATH", pypath)
            .env("VARSTOTAS_PARAMS", params.unwrap_or_default())
            .env("VARSTOTAS_UI_LANG", varstotas_i18n::language())
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        if let Some(dir) = path.parent().filter(|d| !d.as_os_str().is_empty()) {
            cmd.current_dir(dir);
        }
        varstotas_script::process::isolate_group(&mut cmd);
        let mut child = match cmd.spawn() {
            Ok(c) => c,
            Err(e) => {
                self.flash_error(format!("Cannot start {py}: {e}"));
                self.console.push(LogLine::Error(format!("cannot start {py}: {e} (set VARSTOTAS_PYTHON)")));
                self.console.open = true;
                return;
            }
        };
        let (tx, rx) = sync_channel(128);
        let exceeded = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let pipe = |r: Box<dyn std::io::Read + Send>, err: bool, tx: std::sync::mpsc::SyncSender<(bool, String)>| {
            let exceeded = exceeded.clone();
            std::thread::spawn(move || {
                let mut reader = std::io::BufReader::new(r.take(1024 * 1024 + 1));
                let mut total = 0usize;
                loop {
                    let mut bytes = Vec::new();
                    let Ok(n) = reader.by_ref().take(65537).read_until(b'\n', &mut bytes) else {
                        break;
                    };
                    if n == 0 {
                        break;
                    }
                    total += n;
                    if total > 1024 * 1024
                        || n > 65536
                        || tx.try_send((err, String::from_utf8_lossy(&bytes).trim_end().to_string()))
                            .is_err()
                    {
                        exceeded.store(true, std::sync::atomic::Ordering::Relaxed);
                        break;
                    }
                }
            });
        };
        if let Some(o) = child.stdout.take() {
            pipe(Box::new(o), false, tx.clone());
        }
        if let Some(e) = child.stderr.take() {
            pipe(Box::new(e), true, tx);
        }
        self.console.open = true;
        self.console.push(LogLine::Input(format!("python {name}")));
        self.bridge.python = Some(PythonRun { child, out: rx, name, text: String::new(),
            started: std::time::Instant::now(),
            exceeded,
        });
    }

    pub(crate) fn stop_python(&mut self) {
        if let Some(p) = self.bridge.python.as_mut() {
            varstotas_script::process::terminate(&mut p.child);
        }
    }

    fn python_tick(&mut self, ctx: &egui::Context) {
        let Some(p) = self.bridge.python.as_mut() else { return;
        };
        ctx.request_repaint_after(std::time::Duration::from_millis(50));
        if p.started.elapsed().as_secs() > 600
            || p.exceeded.load(std::sync::atomic::Ordering::Relaxed)
            || p.text.len() > 1024 * 1024
        {
            varstotas_script::process::terminate(&mut p.child);
        }
        for (err, l) in p.out.try_iter().take(128) {
            p.text.push_str(&l);
            p.text.push('\n');
            self.console.push(if err { LogLine::Error(l) } else { LogLine::Output(l) });
        }
        let Ok(Some(status)) = p.child.try_wait() else { return;
        };
        // Output still in the pipes arrives after the exit: collect it.
        std::thread::sleep(std::time::Duration::from_millis(20));
        for (err, l) in p.out.try_iter() {
            p.text.push_str(&l);
            p.text.push('\n');
            self.console.push(if err { LogLine::Error(l) } else { LogLine::Output(l) });
        }
        let p = self.bridge.python.take().expect("python run");
        if status.success() {
            self.console.last = Some((true, p.text.trim_end().to_string()));
            self.flash(format!("{} finished", p.name));
        } else {
            let msg = format!("{} exited with {status}", p.name);
            self.console.push(LogLine::Error(msg.clone()));
            self.console.last = Some((false, p.text.trim_end().to_string()));
            self.flash_error(msg);
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn base64_matches_the_standard_alphabet() {
        assert_eq!(super::base64(b""), "");
        assert_eq!(super::base64(b"f"), "Zg==");
        assert_eq!(super::base64(b"fo"), "Zm8=");
        assert_eq!(super::base64(b"foobar"), "Zm9vYmFy");
        assert_eq!(super::base64(&[0xff, 0xfe, 0x00, 0x7f]), "//4Afw==");
    }
}

#[cfg(test)]
mod transaction_security_tests {
    use super::*;
    use crate::document::Document;
    fn stage(doc: &Document) -> Txn {
        Txn { conn: 1, label: "Staged".into(), model: doc.model.clone(), base_version: doc.model_version,
            path: doc.active_path.clone(), selection: None, started: std::time::Instant::now(), changed: true }
    }
    #[test]
    fn transaction_commit_preserves_intervening_ui_edit() {
        let mut doc = Document::new(); let mut t = stage(&doc);
        t.model.add_material("agent", [1.0; 4]);
        doc.commit("UI", |m| { m.add_material("user", [1.0; 4]); });
        assert!(t.commit(&mut doc).is_err());
        assert!(doc.model.materials.values().any(|m| m.name == "user"));
        assert!(!doc.model.materials.values().any(|m| m.name == "agent"));
        assert_eq!(doc.history.undo_label(), Some("UI"));
    }
    #[test]
    fn successful_transaction_is_one_undo_step() {
        let mut doc = Document::new(); let mut t = stage(&doc);
        t.model.add_material("agent", [1.0; 4]);
        assert!(doc.model.materials.is_empty());
        assert!(t.commit(&mut doc).unwrap());
        assert_eq!(doc.history.undo_label(), Some("Staged"));
        doc.undo(); assert!(doc.model.materials.is_empty());
    }
}
