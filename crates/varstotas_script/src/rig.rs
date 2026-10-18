//! Rigs (parameters of ordinary components, see `varstotas_kernel::rig`): the values turned into
//! part operations — the joints, then the rig's script `update(p, parts)` — and the variants
//! built, shared, posed again and written back.
//!
//! Script functions (in `update`): `rotate(part, origin, axis, degrees)`,
//! `move(part, offset)` / `translate`, `scale(part, origin, factor or vec3)`,
//! `show(part, flag)`, `hide(part)`, `paint(part, material)`. `p` holds the values and formulas,
//! `parts` each part's rest-pose bounds by name (`min`, `max`, `center`, `size`).

use crate::generator::{derive, expr_engine, register_math};
use varstotas_kernel::glam::{DMat4, DVec3};
use varstotas_kernel::{DefId, GenKind, GenValues, Generator, InstanceId, JointKind, Model, PartOp, Rig};
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};

thread_local! {
    /// Operations collected by the running `update` script.
    static OPS: RefCell<Vec<PartOp>> = const { RefCell::new(Vec::new()) };
}

/// The rig as a generator (for normalizing values, formulas and Entity Info's fields).
pub fn generator_of(name: &str, rig: &Rig) -> Generator {
    Generator {
        name: name.to_string(),
        description: String::new(),
        params: rig.params.clone(),
        formulas: rig.formulas.clone(),
        checks: Vec::new(),
        source: String::new(),
    }
}

/// The generator of master `master` (`None` when it has no rig).
pub fn generator(model: &Model, master: DefId) -> Option<Generator> {
    let d = model.defs.get(master)?;
    Some(generator_of(&d.name, d.rig.as_ref()?))
}

fn rotation(origin: DVec3, axis: DVec3, deg: f64) -> DMat4 {
    let axis = axis.normalize_or(DVec3::Z);
    DMat4::from_translation(origin) * DMat4::from_axis_angle(axis, deg.to_radians()) * DMat4::from_translation(-origin)
}

fn number(d: &rhai::Dynamic) -> Option<f64> {
    d.as_float().ok().or_else(|| d.as_int().ok().map(|i| i as f64))
}

fn flag(d: &rhai::Dynamic) -> Option<bool> {
    d.as_bool().ok().or_else(|| number(d).map(|x| x.abs() > 1e-12))
}

type R<T> = Result<T, Box<rhai::EvalAltResult>>;

fn push(op: PartOp) {
    OPS.with(|o| o.borrow_mut().push(op));
}

/// The engine of `update` scripts: math, vectors and the part functions.
fn script_engine() -> rhai::Engine {
    use crate::api::{num, pt};
    use rhai::Dynamic;
    let mut e = rhai::Engine::new();
    e.set_max_operations(2_000_000);
    e.set_max_call_levels(64);
    register_math(&mut e);
    crate::api::register_vec(&mut e);
    e.register_fn("rotate", |part: &str, origin: Dynamic, axis: Dynamic, deg: Dynamic| -> R<()> {
        push(PartOp::Transform(part.to_string(), rotation(pt(&origin)?, pt(&axis)?, num(&deg)?)));
        Ok(())
    });
    for name in ["move", "translate"] {
        e.register_fn(name, |part: &str, off: Dynamic| -> R<()> {
            push(PartOp::Transform(part.to_string(), DMat4::from_translation(pt(&off)?)));
            Ok(())
        });
    }
    e.register_fn("scale", |part: &str, origin: Dynamic, f: Dynamic| -> R<()> {
        let o = pt(&origin)?;
        let k = if f.is::<DVec3>() || f.is_array() { pt(&f)? } else { DVec3::splat(num(&f)?) };
        push(PartOp::Transform(part.to_string(), DMat4::from_translation(o) * DMat4::from_scale(k) * DMat4::from_translation(-o)));
        Ok(())
    });
    e.register_fn("show", |part: &str, on: bool| push(PartOp::Visible(part.to_string(), on)));
    e.register_fn("show", |part: &str| push(PartOp::Visible(part.to_string(), true)));
    e.register_fn("hide", |part: &str| push(PartOp::Visible(part.to_string(), false)));
    e.register_fn("paint", |part: &str, mat: &str| push(PartOp::Paint(part.to_string(), mat.to_string())));
    e
}

/// The parts of `master` for the script: name → rest-pose bounds.
fn parts_map(model: &Model, master: DefId) -> rhai::Map {
    let mut m = rhai::Map::new();
    for (name, b) in model.rig_part_bounds(master) {
        let mut x = rhai::Map::new();
        x.insert("min".into(), rhai::Dynamic::from(b.min));
        x.insert("max".into(), rhai::Dynamic::from(b.max));
        x.insert("center".into(), rhai::Dynamic::from(b.center()));
        x.insert("size".into(), rhai::Dynamic::from(b.size()));
        m.insert(name.as_str().into(), rhai::Dynamic::from_map(x));
    }
    m
}

/// Run `update(p, parts)` of `script`: its operations.
fn run_script(script: &str, p: rhai::Map, parts: rhai::Map) -> Result<Vec<PartOp>, String> {
    let engine = script_engine();
    let ast = engine.compile(script).map_err(|e| format!("script: {e}"))?;
    if !ast.iter_functions().any(|f| f.name == "update" && f.params.len() == 2) {
        return Err("script: needs `fn update(p, parts) { ... }`".into());
    }
    OPS.with(|o| o.borrow_mut().clear());
    let r = engine.call_fn::<rhai::Dynamic>(&mut rhai::Scope::new(), &ast, "update", (rhai::Dynamic::from_map(p), rhai::Dynamic::from_map(parts)));
    let ops = OPS.with(|o| std::mem::take(&mut *o.borrow_mut()));
    let _ = r.map_err(|e| format!("script: {e}"))?;
    Ok(ops)
}

/// The part operations of rig `rig` (of master `master`) for `values` (normalized).
pub fn rig_ops(model: &Model, master: DefId, rig: &Rig, values: &GenValues) -> Result<Vec<PartOp>, String> {
    let name = model.defs.get(master).map(|d| d.name.clone()).unwrap_or_default();
    let g = generator_of(&name, rig);
    let p = derive(&g, values)?;
    let mut ops = Vec::new();
    if !rig.joints.is_empty() {
        let engine = expr_engine();
        let mut scope = rhai::Scope::new();
        for (k, v) in &p {
            scope.push_dynamic(k.to_string(), v.clone());
        }
        for j in &rig.joints {
            let what = || format!("{} ({})", j.part, j.kind.name());
            let v = if !j.expr.is_empty() {
                engine.eval_with_scope::<rhai::Dynamic>(&mut scope, &j.expr).map_err(|e| format!("{}: {e}", what()))?
            } else {
                p.get(j.param.as_str()).cloned().ok_or_else(|| format!("{}: no parameter `{}`", what(), j.param))?
            };
            match &j.kind {
                JointKind::Rotate { .. } | JointKind::Slide { .. } => {
                    let x = number(&v).ok_or_else(|| format!("{}: the value is not a number", what()))?;
                    let amount = match g.param(&j.param).filter(|_| j.expr.is_empty()) {
                        Some(pp) if pp.min.zip(pp.max).is_some_and(|(a, b)| b > a) => {
                            let (a, b) = (pp.min.unwrap(), pp.max.unwrap());
                            j.from + (x - a) / (b - a) * (j.to - j.from)
                        }
                        _ => x,
                    };
                    let m = match &j.kind {
                        JointKind::Rotate { origin, axis } => rotation(*origin, *axis, amount),
                        JointKind::Slide { dir } => DMat4::from_translation(dir.normalize_or_zero() * amount),
                        _ => unreachable!(),
                    };
                    ops.push(PartOp::Transform(j.part.clone(), m));
                }
                JointKind::Visible => {
                    let on = flag(&v).ok_or_else(|| format!("{}: the value is not true / false", what()))?;
                    ops.push(PartOp::Visible(j.part.clone(), on));
                }
                JointKind::Material => {
                    let s = v.clone().into_string().map_err(|_| format!("{}: the value is not a material name", what()))?;
                    ops.push(PartOp::Paint(j.part.clone(), s));
                }
            }
        }
    }
    if !rig.script.trim().is_empty() {
        ops.extend(run_script(&rig.script, p, parts_map(model, master))?);
    }
    Ok(ops)
}

/// [`rig_ops`] with the master's own rig.
pub fn ops(model: &Model, master: DefId, values: &GenValues) -> Result<Vec<PartOp>, String> {
    let rig = model.defs.get(master).and_then(|d| d.rig.clone()).ok_or("not a component with parameters")?;
    rig_ops(model, master, &rig, values)
}

/// Values over the rig's defaults, normalized.
pub fn normalize(model: &Model, master: DefId, values: &GenValues) -> Result<GenValues, String> {
    generator(model, master).ok_or("not a component with parameters")?.normalize(values)
}

/// The variant of `master` for `values` (over the defaults): the shared one, else a new one.
pub fn variant_for(model: &mut Model, master: DefId, values: &GenValues) -> Result<DefId, String> {
    let values = normalize(model, master, values)?;
    if let Some(v) = model.rig_variant(master, &values) {
        return Ok(v);
    }
    let ops = ops(model, master, &values)?;
    model.new_rig_variant(master, values, &ops).ok_or_else(|| "no such component".into())
}

/// Change the values of the rigged instance `inst` (over its current ones, or over the defaults
/// with `reset`): it moves to the variant with the new values — the shared one, its own posed
/// again when nothing else uses it, else a new one.
pub fn set_values(model: &mut Model, path: &[InstanceId], inst: InstanceId, values: &GenValues, reset: bool) -> Result<(), String> {
    let (old, cur) = model.instance_rigged(path, inst).map(|(d, r)| (d, r.clone())).ok_or("not a component with parameters")?;
    let mut merged = if reset { GenValues::new() } else { cur.values.clone() };
    merged.extend(values.iter().map(|(k, v)| (k.clone(), v.clone())));
    let values = normalize(model, cur.master, &merged)?;
    if values == cur.values {
        return Ok(());
    }
    let target = match model.rig_variant(cur.master, &values) {
        Some(d) => d,
        None if model.instance_count(old) == 1 => {
            let ops = ops(model, cur.master, &values)?;
            model.pose_rig(old, cur.master, &ops);
            if let Some(r) = model.defs[old].rigged.as_mut() {
                r.values = values;
            }
            return Ok(());
        }
        None => variant_for(model, cur.master, &values)?,
    };
    if let Some(i) = model.mesh_at_mut(path).and_then(|m| m.instances.get_mut(inst)) {
        i.def = target;
    }
    model.remove_unused(HashSet::from([old]));
    Ok(())
}

/// Pose every variant of `master` again (after its rig or content changed). Values the rig
/// rejects now fall back to the defaults; variants that end up equal are merged.
pub fn rebuild_all(model: &mut Model, master: DefId) -> Result<(), String> {
    let g = generator(model, master).ok_or("not a component with parameters")?;
    let mut seen: Vec<(GenValues, DefId)> = Vec::new();
    let mut merged: HashMap<DefId, DefId> = HashMap::new();
    for (v, old) in model.rig_variants(master) {
        let kept: GenValues = old.into_iter().filter(|(k, x)| g.param(k).is_some_and(|p| p.normalize(x).is_ok())).collect();
        let values = g.normalize(&kept)?;
        if let Some((_, k)) = seen.iter().find(|s| s.0 == values) {
            merged.insert(v, *k);
            continue;
        }
        let ops = ops(model, master, &values)?;
        model.pose_rig(v, master, &ops);
        if let Some(r) = model.defs[v].rigged.as_mut() {
            r.values = values.clone();
        }
        seen.push((values, v));
    }
    if !merged.is_empty() {
        for d in model.defs.values_mut() {
            for i in d.mesh.instances.values_mut() {
                if let Some(k) = merged.get(&i.def) {
                    i.def = *k;
                }
            }
        }
        model.remove_unused(merged.keys().copied().collect());
    }
    Ok(())
}

/// Instances placed with a master itself (pasted, placed from the Components card, made by a
/// script) are moved to its variant for the default values.
pub fn attach(model: &mut Model) {
    let masters: HashSet<DefId> = model.defs.iter().filter(|(_, d)| d.rig.is_some()).map(|(id, _)| id).collect();
    if masters.is_empty() {
        return;
    }
    let mut on: Vec<(DefId, InstanceId, DefId)> = Vec::new();
    for (holder, d) in &model.defs {
        for (id, i) in &d.mesh.instances {
            if masters.contains(&i.def) {
                on.push((holder, id, i.def));
            }
        }
    }
    let mut var: HashMap<DefId, Option<DefId>> = HashMap::new();
    for (holder, id, m) in on {
        let v = match var.get(&m) {
            Some(v) => *v,
            None => {
                let v = variant_for(model, m, &GenValues::new()).ok();
                var.insert(m, v);
                v
            }
        };
        if let Some(v) = v
            && let Some(i) = model.defs.get_mut(holder).and_then(|d| d.mesh.instances.get_mut(id))
        {
            i.def = v;
        }
    }
}

/// The variant whose content the context `path` edits directly, if any.
pub fn edited_variant(model: &Model, path: &[InstanceId]) -> Option<DefId> {
    let d = model.def_at(path)?;
    model.defs.get(d)?.rigged.as_ref().filter(|r| r.paint.is_none()).map(|_| d)
}

/// After an edit in the context `path`: when it is a variant, its content goes back to the
/// master and the other variants are posed again; instances on masters get variants.
pub fn after_edit(model: &mut Model, path: &[InstanceId]) -> Result<(), String> {
    let r = match edited_variant(model, path) {
        Some(v) => {
            let r = model.defs[v].rigged.clone().expect("a variant");
            let ops = ops(model, r.master, &r.values)?;
            model.rig_write_back(v, &ops);
            rebuild_all(model, r.master)
        }
        None => Ok(()),
    };
    attach(model);
    r
}

/// Set (or with `None` remove) the rig of the component `d` (a master, one of its variants or
/// a plain component, which becomes the master: its unnamed parts are named and its copies
/// move to the variant for the defaults). Removing puts the copies back on the master (as
/// drawn). Returns the master.
pub fn set_rig(model: &mut Model, d: DefId, rig: Option<Rig>) -> Result<DefId, String> {
    let master = model.rig_master(d).unwrap_or(d);
    let def = model.defs.get(master).ok_or("no such component")?;
    if def.is_group {
        return Err("groups can't have parameters: make it a component first".into());
    }
    if def.generated.is_some() || def.params.is_some() || def.rigged.is_some() {
        return Err("this component is built by a generator or a parametric group".into());
    }
    let (had, name) = (def.rig.is_some(), def.name.clone());
    match rig {
        Some(rig) => {
            validate(&rig)?;
            if !had {
                model.name_rig_parts(master);
            }
            let g = generator_of(&name, &rig);
            rig_ops(model, master, &rig, &g.defaults())?;
            model.defs[master].rig = Some(rig);
            rebuild_all(model, master)?;
            attach(model);
        }
        None => {
            let vs: HashSet<DefId> = model.rig_variants(master).into_iter().map(|(v, _)| v).collect();
            for x in model.defs.values_mut() {
                for i in x.mesh.instances.values_mut() {
                    if vs.contains(&i.def) {
                        i.def = master;
                    }
                }
            }
            model.defs[master].rig = None;
            model.remove_unused(vs);
        }
    }
    Ok(master)
}

/// Names unique, joints driven by something, formulas parsed.
fn validate(rig: &Rig) -> Result<(), String> {
    let mut keys = HashSet::new();
    for k in rig.params.iter().map(|p| &p.key).chain(rig.formulas.iter().map(|f| &f.key)) {
        if !keys.insert(k.as_str()) {
            return Err(format!("`{k}` twice (parameters and formulas share names)"));
        }
    }
    for j in &rig.joints {
        if j.part.trim().is_empty() {
            return Err("a joint without a part".into());
        }
        if j.expr.trim().is_empty() && !keys.contains(j.param.as_str()) {
            return Err(format!("{}: choose a parameter or write an expression", j.part));
        }
        if let JointKind::Rotate { axis, .. } = j.kind
            && axis.length() < 1e-9
        {
            return Err(format!("{}: the axis has no direction", j.part));
        }
        if let JointKind::Slide { dir } = j.kind
            && dir.length() < 1e-9
        {
            return Err(format!("{}: the direction is zero", j.part));
        }
        if matches!(j.kind, JointKind::Visible) && j.expr.is_empty() && rig.params.iter().find(|p| p.key == j.param).is_some_and(|p| matches!(p.kind, GenKind::Material | GenKind::Choice(_))) {
            return Err(format!("{}: show / hide needs a yes / no or number parameter", j.part));
        }
    }
    Ok(())
}

/// Problems that don't stop the rig (for the editor): joints naming parts that aren't there.
pub fn warnings(model: &Model, master: DefId, rig: &Rig) -> Vec<String> {
    let parts: HashSet<String> = model.rig_parts(master).into_iter().map(|(_, n)| n).collect();
    let mut seen = HashSet::new();
    rig.joints.iter().filter(|j| !parts.contains(&j.part) && seen.insert(j.part.clone())).map(|j| format!("no part `{}` in the component", j.part)).collect()
}
