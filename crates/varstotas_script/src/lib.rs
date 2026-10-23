//! Rhai scripting over the Varstotas kernel: the Script Console, script files and (later) AI
//! agents use the same API. See `docs/SCRIPTING.md` for the language-level reference.
//!
//! A run works on a copy of the model: on success the copy replaces the model (one undo step
//! in the app), on an error nothing changes. Coordinates are world millimetres; new geometry
//! goes into the current context (the open group, or `enter(group)` inside the script).

mod api;
pub mod generator;
pub mod rig;
pub mod help;
mod json;
pub mod network;
pub mod process;

pub use api::list_files;
pub use json::{EvalJson, entity_id, eval_json, parse_entity_id};

use varstotas_kernel::{ContextPath, EntityRef, Model};
use std::cell::RefCell;
use std::time::Duration;

/// Where a script runs: the open editing context and the selection in it.
#[derive(Clone, Debug, Default)]
pub struct ScriptCtx {
    pub path: ContextPath,
    pub selection: Vec<EntityRef>,
    /// Stop the script after this long (default 30 s).
    pub time_limit: Option<Duration>,
    /// Files/network are for explicitly invoked local scripts and installed plugins only.
    pub trusted_io: bool,
}

/// Result of a run.
#[derive(Clone, Debug, Default)]
pub struct ScriptRun {
    /// Text printed by the script (`print`, `debug`), then the value of the last expression.
    pub output: String,
    /// The error that stopped the script (the model is unchanged then).
    pub error: Option<String>,
    /// The model was changed (and written back).
    pub changed: bool,
    /// New selection in the starting context, when the script called `select`.
    pub selection: Option<Vec<EntityRef>>,
}

/// An entity handle in a script: an entity of the mesh reached through `path`.
#[derive(Clone, Debug, PartialEq)]
pub struct Entity {
    pub path: ContextPath,
    pub r: EntityRef,
}

pub(crate) struct State {
    pub model: Model,
    pub path: ContextPath,
    pub start_path: ContextPath,
    pub selection: Vec<Entity>,
    pub selection_set: bool,
    pub out: String,
    pub dirty: bool,
    pub deadline: std::time::Instant,
}

thread_local! {
    pub(crate) static STATE: RefCell<Option<State>> = const { RefCell::new(None) };
}

/// Bound recursive result expansion before formatting or JSON conversion.
fn check_value(value: &rhai::Dynamic) -> Result<(), Box<rhai::EvalAltResult>> {
    fn visit(v: &rhai::Dynamic, depth: usize, nodes: &mut usize, bytes: &mut usize) -> bool {
        if depth > 64 || *nodes >= 100_000 || *bytes > 8 * 1024 * 1024 { return false; }
        *nodes += 1;
        if let Some(a) = v.read_lock::<rhai::Array>() {
            return a.iter().all(|x| visit(x, depth + 1, nodes, bytes));
        }
        if let Some(m) = v.read_lock::<rhai::Map>() {
            return m.iter().all(|(k, x)| { *bytes = bytes.saturating_add(k.len()); visit(x, depth + 1, nodes, bytes) });
        }
        if let Some(s) = v.read_lock::<rhai::ImmutableString>() { *bytes = bytes.saturating_add(s.len()); }
        *bytes <= 8 * 1024 * 1024
    }
    if visit(value, 0, &mut 0, &mut 0) { Ok(()) } else {
        Err(Box::new(rhai::EvalAltResult::ErrorRuntime("Script result exceeds expansion limits".into(), rhai::Position::NONE)))
    }
}

/// Run `src` against `model`.
pub fn run(model: &mut Model, ctx: &ScriptCtx, src: &str) -> ScriptRun {
    let (mut run, last) = run_with(model, ctx, src, Vec::new(), |v| (!v.is_unit()).then(|| api::show(v)));
    if let Some(Some(s)) = last {
        let room = (1024 * 1024usize).saturating_sub(run.output.len());
        run.output.push_str(&s[..s.floor_char_boundary(room.saturating_sub(1).min(s.len()))]);
        if room > 0 {
            run.output.push('\n');
        }
    }
    run
}

/// [`run`] with extra variables in scope; `finish` sees the last value while the script
/// state (the changed model) is still available. The value is not added to the output.
pub(crate) fn run_with<T>(
    model: &mut Model,
    ctx: &ScriptCtx,
    src: &str,
    vars: Vec<(String, rhai::Dynamic)>,
    finish: impl FnOnce(&rhai::Dynamic) -> T,
) -> (ScriptRun, Option<T>) {
    if src.len() > 1024 * 1024 {
        return (
            ScriptRun {
                error: Some("Script source exceeds 1 MiB".into()),
                ..Default::default()
            },
            None,
        );
    }
    let mut deadline =
        std::time::Instant::now() + ctx.time_limit.unwrap_or(Duration::from_secs(30));
    STATE.with(|s| {
        if let Some(outer) = s.borrow().as_ref() {
            deadline = deadline.min(outer.deadline);
        }
    });
    let state = State {
        model: model.clone(),
        path: ctx.path.clone(),
        start_path: ctx.path.clone(),
        selection: ctx.selection.iter().filter(|r| !model.entity_locked(&ctx.path, **r)).map(|r| Entity { path: ctx.path.clone(), r: *r }).collect(),
        selection_set: false,
        out: String::new(),
        dirty: false,
        deadline,
    };
    // A generator built from inside a script nests: the outer state comes back afterwards.
    let outer = STATE.with(|s| s.borrow_mut().replace(state));
    let engine = api::engine(
        deadline.saturating_duration_since(std::time::Instant::now()),
        ctx.trusted_io,
    );
    let mut scope = api::scope();
    for (k, v) in vars {
        scope.push_dynamic(k, v);
    }
    let result = engine.eval_with_scope::<rhai::Dynamic>(&mut scope, src)
        .and_then(|v| {
            check_value(&v)?;
            STATE.with(|s| {
                let s = s.borrow();
                model.validate_locked_changes(&s.as_ref().expect("script state").model)
            }).map_err(|e| Box::new(rhai::EvalAltResult::ErrorRuntime(e.into(), rhai::Position::NONE)))?;
            if !ctx.trusted_io {
                let validation = STATE.with(|s| {
                    let s = s.borrow();
                    let st = s.as_ref().expect("script state");
                    if st.dirty { varstotas_kernel::io::validate_model(&st.model) } else { Ok(()) }
                });
                validation.map_err(|e| Box::new(rhai::EvalAltResult::ErrorRuntime(e.to_string().into(), rhai::Position::NONE)))?;
            }
            if std::time::Instant::now() > deadline {
                Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                    "Script deadline exceeded".into(),
                    rhai::Position::NONE,
                )))
            } else {
                Ok(v)
            }
        });
    let value = result.as_ref().ok().map(finish);
    let state = STATE.with(|s| std::mem::replace(&mut *s.borrow_mut(), outer)).expect("script state");
    let mut out = ScriptRun { output: state.out, ..Default::default() };
    match result {
        Ok(_) => {
            if state.selection_set {
                out.selection = Some(state.selection.iter().filter(|e| e.path == state.start_path).map(|e| e.r).collect());
            }
            if state.dirty {
                *model = state.model;
                out.changed = true;
            }
        }
        Err(e) => {
            out.error = Some(match *e {
                rhai::EvalAltResult::ErrorTerminated(token, pos) => format!("{} ({pos})", api::show(&token)),
                e => e.to_string(),
            })
        }
    }
    (out, value)
}

/// Names of the API functions (for completion and the console's help line).
pub fn api_names() -> Vec<&'static str> {
    api::NAMES.to_vec()
}
