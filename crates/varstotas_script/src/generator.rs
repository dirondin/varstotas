//! Running generators (see `varstotas_kernel::generator`): `build(p)` runs in an empty model
//! (with the document's materials and tags), and what it made becomes the content of the
//! generated definition, in the component's local axes.

use crate::{ScriptCtx, run_with};
use varstotas_kernel::glam::{DMat4, DVec3};
use varstotas_kernel::model::Definition;
use varstotas_kernel::{DefId, GenKind, GenValue, GenValues, Generated, Generator, Instance, InstanceId, Model};
use std::cell::Cell;
use std::collections::HashSet;
use std::time::Duration;
use varstotas_i18n::tr;

thread_local! {
    /// Generators building generators: nesting depth.
    static DEPTH: Cell<u32> = const { Cell::new(0) };
}

/// The values as the script's `p` map: ints as integers, lengths / angles / numbers as floats,
/// flags as bools, choices and material names as strings.
pub fn values_map(g: &Generator, values: &GenValues) -> rhai::Map {
    let mut m = rhai::Map::new();
    for p in &g.params {
        let v = values.get(&p.key).unwrap_or(&p.default);
        let d = match (v, &p.kind) {
            (GenValue::Num(x), GenKind::Int) => rhai::Dynamic::from_int(*x as i64),
            (GenValue::Num(x), _) => rhai::Dynamic::from_float(*x),
            (GenValue::Bool(b), _) => rhai::Dynamic::from_bool(*b),
            (GenValue::Text(s), _) => rhai::Dynamic::from(s.clone()),
        };
        m.insert(p.key.as_str().into(), d);
    }
    m
}

/// A Rhai value as a generator value (numbers, flags, texts).
pub(crate) fn to_value(d: &rhai::Dynamic) -> Option<GenValue> {
    if let Ok(x) = d.as_float() {
        Some(GenValue::Num(x))
    } else if let Ok(i) = d.as_int() {
        Some(GenValue::Num(i as f64))
    } else if let Ok(b) = d.as_bool() {
        Some(GenValue::Bool(b))
    } else {
        d.clone().into_string().ok().map(GenValue::Text)
    }
}

/// `min`, `max`, `clamp` over mixed numbers and `ceil` (also in the script API).
pub(crate) fn register_math(e: &mut rhai::Engine) {
    type R = Result<f64, Box<rhai::EvalAltResult>>;
    fn f(d: &rhai::Dynamic) -> Result<f64, Box<rhai::EvalAltResult>> {
        d.as_float().or_else(|_| d.as_int().map(|i| i as f64)).map_err(|t| tr!("script-err-expected-a-number-got-t", t = t.to_string()).into())
    }
    e.register_fn("min", |a: rhai::Dynamic, b: rhai::Dynamic| -> R { Ok(f(&a)?.min(f(&b)?)) });
    e.register_fn("max", |a: rhai::Dynamic, b: rhai::Dynamic| -> R { Ok(f(&a)?.max(f(&b)?)) });
    e.register_fn("clamp", |x: rhai::Dynamic, a: rhai::Dynamic, b: rhai::Dynamic| -> R { Ok(f(&x)?.max(f(&a)?).min(f(&b)?)) });
    e.register_fn("ceil", |x: rhai::Dynamic| -> R { Ok(f(&x)?.ceil()) });
}

/// The engine for `@formula` / `@check` expressions: Rhai's standard functions plus
/// [`register_math`].
pub(crate) fn expr_engine() -> rhai::Engine {
    let mut e = rhai::Engine::new();
    e.set_max_operations(100_000);
    e.set_max_string_size(1024 * 1024);
    e.set_max_array_size(10_000);
    e.set_max_map_size(1_000);
    e.set_max_call_levels(64);
    e.set_max_expr_depths(64, 64);
    register_math(&mut e);
    e
}

/// The parameter values (normalized) with the `@formula` values after them, as the script's
/// `p` map; `Err` names a failing formula or `@check`.
pub fn derive(g: &Generator, values: &GenValues) -> Result<rhai::Map, String> {
    let mut map = values_map(g, values);
    if g.formulas.is_empty() && g.checks.is_empty() {
        return Ok(map);
    }
    let engine = expr_engine();
    let mut scope = rhai::Scope::new();
    for (k, v) in &map {
        scope.push_dynamic(k.to_string(), v.clone());
    }
    for f in &g.formulas {
        let mut v = engine.eval_with_scope::<rhai::Dynamic>(&mut scope, &f.expr).map_err(|e| format!("{}: formula {}: {e}", g.name, f.key))?;
        if f.kind == GenKind::Int
            && let Ok(x) = v.as_float()
        {
            v = rhai::Dynamic::from_int(x.round() as i64);
        }
        scope.push_dynamic(f.key.clone(), v.clone());
        map.insert(f.key.as_str().into(), v);
    }
    for c in &g.checks {
        let ok = engine.eval_with_scope::<rhai::Dynamic>(&mut scope, &c.expr).map_err(|e| format!("{}: check `{}`: {e}", g.name, c.expr))?;
        if !ok.as_bool().map_err(|_| tr!("script-err-name-check-expr-is-not-true", name = g.name.to_string(), expr = c.expr.to_string()))? {
            return Err(format!("{}: {}", g.name, c.message));
        }
    }
    Ok(map)
}

/// The formula values for `values` (normalized), for showing: (formula, value).
pub fn formula_values(g: &Generator, values: &GenValues) -> Result<Vec<(varstotas_kernel::GenFormula, GenValue)>, String> {
    let map = derive(g, values)?;
    Ok(g.formulas.iter().filter_map(|f| map.get(f.key.as_str()).and_then(to_value).map(|v| (f.clone(), v))).collect())
}

/// Run the generator with `values` (normalized) into a fresh model.
fn build(model: &Model, g: &Generator, values: &GenValues) -> Result<Model, String> {
    if DEPTH.get() >= 4 {
        return Err(tr!("script-err-name-generators-nested-too-deep", name = g.name.to_string()));
    }
    let p = derive(g, values)?;
    let mut tmp = Model::new();
    tmp.unit = model.unit;
    tmp.materials = model.materials.clone();
    tmp.tags = model.tags.clone();
    tmp.generators = model.generators.clone();
    let ctx = ScriptCtx { path: Vec::new(), selection: Vec::new(), time_limit: Some(Duration::from_secs(10)),
        trusted_io: false};
    let src = format!("{}\nbuild(p);\n", g.source);
    DEPTH.set(DEPTH.get() + 1);
    let (run, _) = run_with(&mut tmp, &ctx, &src, vec![("p".into(), rhai::Dynamic::from_map(p))], |_| ());
    DEPTH.set(DEPTH.get() - 1);
    match run.error {
        Some(e) => Err(format!("{}: {e}", g.name)),
        None => Ok(tmp),
    }
}

fn generator(model: &Model, name: &str) -> Result<Generator, String> {
    model.generators.get(name).cloned().ok_or_else(|| tr!("script-err-no-generator-name-in-the-document", name = name.to_string()))
}

/// The definition of generator `name` with `values` (normalized): the shared one, else a new
/// one built now.
pub fn def_for(model: &mut Model, name: &str, values: &GenValues) -> Result<DefId, String> {
    let g = generator(model, name)?;
    let values = g.normalize(values)?;
    if let Some(d) = model.generated_def(name, &values) {
        return Ok(d);
    }
    let built = build(model, &g, &values)?;
    let d = model.defs.insert(Definition {
        // called as the generator, whatever the values (the Outliner shows what it is)
        name: name.to_string(),
        mesh: Default::default(),
        is_group: false,
        guides: Default::default(),
        params: None,
        generated: Some(Generated { generator: name.to_string(), values }),
        source: None,
        rig: None,
        rigged: None,
        preview: None,
    });
    model.graft_content(d, &built);
    Ok(d)
}

/// Add generator `src` to the document (replacing one of the same name; its definitions are
/// rebuilt from the new source). Returns its name.
pub fn add(model: &mut Model, src: &str) -> Result<String, String> {
    let g = Generator::parse(src)?;
    let name = g.name.clone();
    if model.generators.get(&name).is_some_and(|old| old.source == g.source) {
        return Ok(name);
    }
    let rebuild = model.generators.contains_key(&name);
    model.add_generator(g);
    if rebuild {
        rebuild_all(model, &name)?;
    }
    Ok(name)
}

/// Rebuild every definition of generator `name` (after its source changed). Values the new
/// header rejects fall back to the defaults.
pub fn rebuild_all(model: &mut Model, name: &str) -> Result<(), String> {
    let g = generator(model, name)?;
    let defs: Vec<(DefId, GenValues)> = model
        .defs
        .iter()
        .filter_map(|(id, d)| d.generated.as_ref().filter(|x| x.generator == name).map(|x| (id, x.values.clone())))
        .collect();
    for (d, old) in defs {
        let kept: GenValues = old.into_iter().filter(|(k, v)| g.param(k).is_some_and(|p| p.normalize(v).is_ok())).collect();
        let values = g.normalize(&kept)?;
        let built = build(model, &g, &values)?;
        model.graft_content(d, &built);
        model.defs[d].generated = Some(Generated { generator: name.to_string(), values });
    }
    Ok(())
}

/// Place generator `name` with `values` (over the defaults) in the context `path` with the
/// local transform `xf`.
pub fn place(model: &mut Model, path: &[InstanceId], name: &str, values: &GenValues, xf: DMat4) -> Result<InstanceId, String> {
    let d = def_for(model, name, values)?;
    let mesh = model.mesh_at_mut(path).ok_or_else(|| tr!("script-err-no-such-context"))?;
    Ok(mesh.instances.insert(Instance { def: d, transform: xf, name: String::new(), material: None, tag: None, hidden: false, locked: false }))
}

/// Change the values of generated instance `inst` (`values` over its current ones, or over the
/// defaults with `reset`). The instance moves to the definition with the new values: the shared
/// one, its own rebuilt in place when nothing else uses it, else a new one.
pub fn set_values(model: &mut Model, path: &[InstanceId], inst: InstanceId, values: &GenValues, reset: bool) -> Result<(), String> {
    let cur = model.instance_generated(path, inst).cloned().ok_or_else(|| tr!("script-err-not-a-generated-component"))?;
    let g = generator(model, &cur.generator)?;
    let mut merged = if reset { GenValues::new() } else { cur.values.clone() };
    merged.extend(values.iter().map(|(k, v)| (k.clone(), v.clone())));
    let values = g.normalize(&merged)?;
    if values == cur.values {
        return Ok(());
    }
    let old = model.mesh_at(path).and_then(|m| m.instances.get(inst)).map(|i| i.def).ok_or_else(|| tr!("script-err-no-such-instance"))?;
    let target = match model.generated_def(&g.name, &values) {
        Some(d) => d,
        None if model.instance_count(old) == 1 => {
            let built = build(model, &g, &values)?;
            model.graft_content(old, &built);
            model.defs[old].generated = Some(Generated { generator: g.name.clone(), values });
            return Ok(());
        }
        None => def_for(model, &g.name, &values)?,
    };
    if let Some(i) = model.mesh_at_mut(path).and_then(|m| m.instances.get_mut(inst)) {
        i.def = target;
    }
    model.remove_unused(HashSet::from([old]));
    Ok(())
}

/// Scale tool on a generated instance: the world scale `xf` (about an anchor) becomes new
/// width / depth / height values (`SIZE_KEYS`) and a moved instance; axes without such a
/// parameter keep their size. `None` when no scaled axis has a size parameter.
pub fn scale(model: &mut Model, path: &[InstanceId], inst: InstanceId, xf: &DMat4) -> Option<Result<(), String>> {
    let cur = model.instance_generated(path, inst)?.clone();
    let g = model.generators.get(&cur.generator)?.clone();
    let world = model.transform_at(path);
    let t = model.mesh_at(path)?.instances.get(inst)?.transform;
    let x = world * t;
    let mut values = GenValues::new();
    for k in 0..3 {
        let Some(p) = g.size_param(k) else { continue };
        let axis = x.transform_vector3([DVec3::X, DVec3::Y, DVec3::Z][k]);
        let f = xf.transform_vector3(axis).length() / axis.length().max(1e-12);
        if (f - 1.0).abs() < 1e-9 {
            continue;
        }
        let v = cur.values.get(&p.key).and_then(|v| v.num())?;
        values.insert(p.key.clone(), GenValue::Num(v * f));
    }
    if values.is_empty() {
        return None;
    }
    let r = set_values(model, path, inst, &values, false).map(|()| {
        // The local origin moves as the scale moves it; orientation stays.
        let o = world.inverse().transform_point3(xf.transform_point3(x.transform_point3(DVec3::ZERO)));
        if let Some(i) = model.mesh_at_mut(path).and_then(|m| m.instances.get_mut(inst)) {
            i.transform.w_axis = o.extend(1.0);
        }
    });
    Some(r)
}
