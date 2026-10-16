//! Rhai scripting over the Varstotas kernel: the Script Console, script files and (later) AI
//! agents use the same API. See `docs/SCRIPTING.md` for the language-level reference.
//!
//! A run works on a copy of the model: on success the copy replaces the model (one undo step
//! in the app), on an error nothing changes. Coordinates are world millimetres; new geometry
//! goes into the current context (the open group, or `enter(group)` inside the script).

mod api;
pub mod help;
mod json;

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
}

thread_local! {
    pub(crate) static STATE: RefCell<Option<State>> = const { RefCell::new(None) };
}

/// Run `src` against `model`.
pub fn run(model: &mut Model, ctx: &ScriptCtx, src: &str) -> ScriptRun {
    let (mut run, last) = run_with(model, ctx, src, Vec::new(), |v| (!v.is_unit()).then(|| api::show(v)));
    if let Some(Some(s)) = last {
        run.output.push_str(&s);
        run.output.push('\n');
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
    let state = State {
        model: model.clone(),
        path: ctx.path.clone(),
        start_path: ctx.path.clone(),
        selection: ctx.selection.iter().map(|r| Entity { path: ctx.path.clone(), r: *r }).collect(),
        selection_set: false,
        out: String::new(),
        dirty: false,
    };
    STATE.with(|s| *s.borrow_mut() = Some(state));
    let engine = api::engine(ctx.time_limit.unwrap_or(Duration::from_secs(30)));
    let mut scope = api::scope();
    for (k, v) in vars {
        scope.push_dynamic(k, v);
    }
    let result = engine.eval_with_scope::<rhai::Dynamic>(&mut scope, src);
    let value = result.as_ref().ok().map(finish);
    let state = STATE.with(|s| s.borrow_mut().take()).expect("script state");
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
