//! Rigs (parameters of ordinary components, see `varstotas_kernel::rig`): the values turned into
//! part operations — the joints — and the variants built, shared, posed again and written back.
//!
//! The rig's script `fn update(p) { … }` runs after the joints with the whole script API (as a
//! generator's `build`): the current context is the component's content in the pose of the
//! joints, `p` holds the values and formulas; parts are found with `child(name)` /
//! `child(parent, name, deep)` and changed with the usual functions (`rotate`, `move`, `scale`,
//! `hide`, `paint`, `set_params` …; a part's name also works in place of the part). It runs on
//! a copy of the document: definitions it changes become copies used by the variant, new
//! generated components and components with parameters are the document's for their values.
//! `update(p, parts)` of older documents still runs (`parts`: rest-pose bounds by name).
//!
//! Parameters of parts ([`PartOp::Values`]) are resolved into [`PartOp::Swap`]s: the part's
//! definition for its values in the master with these over them (built when new).

use crate::generator::{derive, expr_engine, to_value};
use crate::ScriptCtx;
use varstotas_kernel::glam::{DMat4, DVec3};
use varstotas_kernel::{DefId, GenKind, GenValues, Generator, InstanceId, JointKind, Mesh, Model, PartOp, Rig, Rigged};
use std::collections::{HashMap, HashSet};
use std::time::Duration;
use varstotas_i18n::tr;

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

/// The joints' part operations of rig `rig` (of master `master`) for `values` (normalized);
/// the script runs separately ([`script_pose`]).
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
                    let x = number(&v).ok_or_else(|| tr!("script-err-what-the-value-is-not-a-3", what = what().to_string()))?;
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
                    let on = flag(&v).ok_or_else(|| tr!("script-err-what-the-value-is-not-true", what = what().to_string()))?;
                    ops.push(PartOp::Visible(j.part.clone(), on));
                }
                JointKind::Material => {
                    let s = v.clone().into_string().map_err(|_| tr!("script-err-what-the-value-is-not-a-2", what = what().to_string()))?;
                    ops.push(PartOp::Paint(j.part.clone(), s));
                }
                JointKind::Param { key } => {
                    let x = to_value(&v).ok_or_else(|| tr!("script-err-what-the-value-is-not-a", what = what().to_string()))?;
                    ops.push(PartOp::Values(j.part.clone(), GenValues::from([(key.clone(), x)])));
                }
            }
        }
    }
    Ok(ops)
}

/// [`rig_ops`] with the master's own rig, unresolved (parameters of parts as
/// [`PartOp::Values`]).
fn raw_ops(model: &Model, master: DefId, values: &GenValues) -> Result<Vec<PartOp>, String> {
    let rig = model.defs.get(master).and_then(|d| d.rig.clone()).ok_or_else(|| tr!("script-err-not-a-component-with-parameters"))?;
    rig_ops(model, master, &rig, values)
}

/// [`rig_ops`] with the master's own rig, parameters of parts resolved into the parts'
/// definitions for them ([`PartOp::Swap`], built when new).
pub fn ops(model: &mut Model, master: DefId, values: &GenValues) -> Result<Vec<PartOp>, String> {
    let ops = raw_ops(model, master, values)?;
    resolve(model, master, ops)
}

/// What a part with parameters is: a generated component (generator name) or a variant of a
/// component with a rig (its master), with its values.
enum PartKind {
    Generated(String),
    Rigged(DefId),
}

fn part_kind(model: &Model, def: DefId) -> Option<(PartKind, GenValues)> {
    let d = model.defs.get(def)?;
    if let Some(g) = &d.generated {
        return Some((PartKind::Generated(g.generator.clone()), g.values.clone()));
    }
    let r = d.rigged.as_ref().filter(|r| r.paint.is_none())?;
    Some((PartKind::Rigged(r.master), r.values.clone()))
}

/// The parameters of part `part` of master `master` (None when it has none): for the editor
/// and the checks.
pub fn part_generator(model: &Model, master: DefId, part: &str) -> Option<Generator> {
    let (id, _) = model.rig_parts(master).into_iter().find(|(_, n)| n == part)?;
    let def = model.defs.get(master)?.mesh.instances.get(id)?.def;
    match part_kind(model, def)?.0 {
        PartKind::Generated(name) => model.generators.get(&name).cloned(),
        PartKind::Rigged(m) => generator(model, m),
    }
}

/// [`PartOp::Values`] → [`PartOp::Swap`] to the part's definition for its values in the master
/// with these over them (merged per part, in order).
fn resolve(model: &mut Model, master: DefId, ops: Vec<PartOp>) -> Result<Vec<PartOp>, String> {
    if !ops.iter().any(|o| matches!(o, PartOp::Values(..))) {
        return Ok(ops);
    }
    let mut merged: Vec<(String, GenValues)> = Vec::new();
    let mut out = Vec::new();
    for op in ops {
        match op {
            PartOp::Values(part, v) => match merged.iter_mut().find(|(p, _)| *p == part) {
                Some((_, m)) => m.extend(v),
                None => merged.push((part, v)),
            },
            op => out.push(op),
        }
    }
    let parts = model.rig_parts(master);
    let mut swaps = Vec::new();
    for (part, over) in merged {
        let (id, _) = parts.iter().find(|(_, n)| *n == part).ok_or_else(|| tr!("script-err-part-no-such-part", part = part.to_string()))?;
        let def = model.defs.get(master).and_then(|d| d.mesh.instances.get(*id)).map(|i| i.def).ok_or_else(|| tr!("script-err-no-such-component"))?;
        let (kind, mut values) = part_kind(model, def).ok_or_else(|| tr!("script-err-part-the-part-has-no-parameters-2", part = part.to_string()))?;
        values.extend(over);
        let d = match kind {
            PartKind::Generated(name) => crate::generator::def_for(model, &name, &values),
            PartKind::Rigged(m) => variant_for(model, m, &values),
        }
        .map_err(|e| format!("{part}: {e}"))?;
        swaps.push(PartOp::Swap(part, d));
    }
    // Swaps first: painting applies to the swapped definition.
    swaps.extend(out);
    Ok(swaps)
}

/// Hash of serialized `v` (telling what a script changed).
fn hash_of(v: &impl serde::Serialize) -> u64 {
    use std::hash::Hasher;
    struct W(std::collections::hash_map::DefaultHasher);
    impl std::io::Write for W {
        fn write(&mut self, b: &[u8]) -> std::io::Result<usize> {
            self.0.write(b);
            Ok(b.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    let mut w = W(Default::default());
    let _ = serde_json::to_writer(&mut w, v);
    w.0.finish()
}

/// The own geometry of a mesh (not its groups / components).
fn geometry_hash(m: &Mesh) -> u64 {
    hash_of(&(&m.vertices, &m.edges, &m.faces, &m.curves, &m.splines))
}

/// Run the rig's script on a copy of `model` whose top level is `content` (the variant posed by
/// the joints): the copy afterwards.
fn sandbox(model: &Model, content: &Mesh, script: &str, p: rhai::Map) -> Result<Model, String> {
    let mut sb = model.clone();
    let root = sb.root;
    sb.defs[root].mesh = content.clone();
    // `update(p, parts)` of older documents: the parts' bounds by name.
    let legacy = rhai::Engine::new().compile(script).map_err(|e| format!("script: {e}"))?.iter_functions().find(|f| f.name == "update").map(|f| f.params.len());
    let mut vars = vec![("p".to_string(), rhai::Dynamic::from_map(p))];
    let call = match legacy {
        Some(1) => "update(p);",
        Some(2) => {
            let mut parts = rhai::Map::new();
            for i in content.instances.values() {
                let name = if i.name.is_empty() { sb.defs.get(i.def).map(|d| d.name.clone()).unwrap_or_default() } else { i.name.clone() };
                if let Some(b) = sb.def_bounds(i.def).map(|b| b.transformed(&i.transform)) {
                    let mut x = rhai::Map::new();
                    x.insert("min".into(), rhai::Dynamic::from(b.min));
                    x.insert("max".into(), rhai::Dynamic::from(b.max));
                    x.insert("center".into(), rhai::Dynamic::from(b.center()));
                    x.insert("size".into(), rhai::Dynamic::from(b.size()));
                    parts.insert(name.as_str().into(), rhai::Dynamic::from_map(x));
                }
            }
            vars.push(("parts".to_string(), rhai::Dynamic::from_map(parts)));
            "update(p, parts);"
        }
        _ => return Err("script: needs `fn update(p) { ... }`".into()),
    };
    let ctx = ScriptCtx { path: Vec::new(), selection: Vec::new(), time_limit: Some(Duration::from_secs(10)) };
    let (run, _) = crate::run_with(&mut sb, &ctx, &format!("{script}\n{call}\n"), vars, |_| ());
    match run.error {
        Some(e) => Err(format!("script: {e}")),
        None => Ok(sb),
    }
}

/// Definitions used inside `d` of `model` (nested, `d` itself not).
fn nested(model: &Model, d: DefId, out: &mut HashSet<DefId>) {
    let Some(def) = model.defs.get(d) else { return };
    for i in def.mesh.instances.values() {
        if out.insert(i.def) {
            nested(model, i.def, out);
        }
    }
}

/// The script of `master`'s rig run on variant `target` (posed by the joints for `values`):
/// the variant takes the result. Definitions the script left alone stay shared, changed ones are
/// copied for the variant (posed copies, written back to their originals), new generated
/// components and components with parameters are the document's for their values.
fn script_pose(model: &mut Model, target: DefId, master: DefId, values: &GenValues) -> Result<(), String> {
    let Some(rig) = model.defs.get(master).and_then(|d| d.rig.clone()).filter(|r| !r.script.trim().is_empty()) else { return Ok(()) };
    let name = model.defs[master].name.clone();
    let p = derive(&generator_of(&name, &rig), values)?;
    let sb = sandbox(model, &model.defs[target].mesh, &rig.script, p)?;
    // Reachable from the result; unchanged = same content, nested ones unchanged too.
    let mut reach = HashSet::new();
    nested(&sb, sb.root, &mut reach);
    let orig: HashSet<DefId> = model.defs.keys().collect();
    fn same(sb: &Model, model: &Model, orig: &HashSet<DefId>, d: DefId, memo: &mut HashMap<DefId, bool>) -> bool {
        if let Some(x) = memo.get(&d) {
            return *x;
        }
        memo.insert(d, false);
        let ok = orig.contains(&d)
            && hash_of(&sb.defs[d]) == hash_of(&model.defs[d])
            && sb.defs[d].mesh.instances.values().all(|i| same(sb, model, orig, i.def, memo));
        memo.insert(d, ok);
        ok
    }
    let mut memo = HashMap::new();
    let mut pre: HashMap<DefId, DefId> = HashMap::new();
    let mut resolve: Vec<DefId> = Vec::new();
    for d in &reach {
        if same(&sb, model, &orig, *d, &mut memo) {
            pre.insert(*d, *d);
        } else {
            resolve.push(*d);
        }
    }
    for (n, g) in &sb.generators {
        if !model.generators.contains_key(n) {
            model.add_generator(g.clone());
        }
    }
    for d in resolve {
        let def = &sb.defs[d];
        let found = if let Some(g) = &def.generated {
            Some(crate::generator::def_for(model, &g.generator, &g.values)?)
        } else if let Some(r) = def.rigged.as_ref().filter(|r| r.paint.is_none() && orig.contains(&r.master)) {
            Some(variant_for(model, r.master, &r.values)?)
        } else {
            None
        };
        if let Some(x) = found {
            pre.insert(d, x);
        }
    }
    let mut old = HashSet::new();
    nested(model, target, &mut old);
    let (mesh, map) = model.graft_mesh_with(&sb, sb.root_mesh().clone(), pre.clone());
    for (src, dst) in map {
        if orig.contains(&src) && !pre.contains_key(&src) && let Some(def) = model.defs.get_mut(dst) {
            def.rig = None;
            def.rigged = Some(Rigged { master: src, values: GenValues::new(), paint: Some(String::new()) });
        }
    }
    model.defs[target].mesh = mesh;
    model.remove_unused(old);
    Ok(())
}

/// Pose `target` as `master` for `values` (normalized): the joints, then the script.
fn pose(model: &mut Model, target: DefId, master: DefId, values: &GenValues) -> Result<(), String> {
    let ops = ops(model, master, values)?;
    model.pose_rig(target, master, &ops);
    script_pose(model, target, master, values)
}

/// Undo the script's pose in the edited variant `v` (values `values`): parts the script made go,
/// its moves, visibility, materials and definitions are reverted. The master's own geometry
/// stays when the script changes it (edits of it in a scripted pose are not kept).
fn unscript(model: &mut Model, v: DefId, master: DefId, values: &GenValues) -> Result<(), String> {
    let Some(rig) = model.defs.get(master).and_then(|d| d.rig.clone()).filter(|r| !r.script.trim().is_empty()) else { return Ok(()) };
    let p = derive(&generator_of(&model.defs[master].name, &rig), values)?;
    // The joints' pose (J) and the script's (P), in a copy.
    let mut tmp = model.clone();
    let t = tmp.new_rig_variant(master, values.clone(), &[]).ok_or_else(|| tr!("script-err-no-such-component"))?;
    let o = ops(&mut tmp, master, values)?;
    tmp.pose_rig(t, master, &o);
    let j = tmp.defs[t].mesh.clone();
    let sb = sandbox(&tmp, &j, &rig.script, p)?;
    let pm = sb.root_mesh();
    let mut e = model.defs[v].mesh.clone();
    let made: Vec<InstanceId> = pm.instances.keys().filter(|k| !j.instances.contains_key(*k)).collect();
    for k in made {
        e.instances.remove(k);
    }
    for (k, ji) in &j.instances {
        let (Some(pi), Some(ei)) = (pm.instances.get(k), e.instances.get_mut(k)) else { continue };
        if pi.transform != ji.transform {
            ei.transform = ji.transform * pi.transform.inverse() * ei.transform;
        }
        if pi.hidden != ji.hidden {
            ei.hidden = ji.hidden;
        }
        if pi.material != ji.material {
            ei.material = ji.material;
        }
        if pi.def != ji.def
            && let Some(mi) = model.defs[master].mesh.instances.get(k)
        {
            ei.def = mi.def;
        }
    }
    if geometry_hash(pm) != geometry_hash(&j) {
        let m = &model.defs[master].mesh;
        (e.vertices, e.edges, e.faces, e.curves, e.splines) =
            (m.vertices.clone(), m.edges.clone(), m.faces.clone(), m.curves.clone(), m.splines.clone());
    }
    model.defs[v].mesh = e;
    Ok(())
}

/// Values over the rig's defaults, normalized.
pub fn normalize(model: &Model, master: DefId, values: &GenValues) -> Result<GenValues, String> {
    generator(model, master).ok_or_else(|| tr!("script-err-not-a-component-with-parameters"))?.normalize(values)
}

/// The variant of `master` for `values` (over the defaults): the shared one, else a new one.
pub fn variant_for(model: &mut Model, master: DefId, values: &GenValues) -> Result<DefId, String> {
    let values = normalize(model, master, values)?;
    if let Some(v) = model.rig_variant(master, &values) {
        return Ok(v);
    }
    let v = model.new_rig_variant(master, values.clone(), &[]).ok_or_else(|| tr!("script-err-no-such-component"))?;
    pose(model, v, master, &values)?;
    Ok(v)
}

/// Change the values of the rigged instance `inst` (over its current ones, or over the defaults
/// with `reset`): it moves to the variant with the new values — the shared one, its own posed
/// again when nothing else uses it, else a new one.
pub fn set_values(model: &mut Model, path: &[InstanceId], inst: InstanceId, values: &GenValues, reset: bool) -> Result<(), String> {
    let (old, cur) = model.instance_rigged(path, inst).map(|(d, r)| (d, r.clone())).ok_or_else(|| tr!("script-err-not-a-component-with-parameters"))?;
    let mut merged = if reset { GenValues::new() } else { cur.values.clone() };
    merged.extend(values.iter().map(|(k, v)| (k.clone(), v.clone())));
    let values = normalize(model, cur.master, &merged)?;
    if values == cur.values {
        return Ok(());
    }
    let target = match model.rig_variant(cur.master, &values) {
        Some(d) => d,
        None if model.instance_count(old) == 1 => {
            pose(model, old, cur.master, &values)?;
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
    let g = generator(model, master).ok_or_else(|| tr!("script-err-not-a-component-with-parameters"))?;
    let mut seen: Vec<(GenValues, DefId)> = Vec::new();
    let mut merged: HashMap<DefId, DefId> = HashMap::new();
    for (v, old) in model.rig_variants(master) {
        let kept: GenValues = old.into_iter().filter(|(k, x)| g.param(k).is_some_and(|p| p.normalize(x).is_ok())).collect();
        let values = g.normalize(&kept)?;
        if let Some((_, k)) = seen.iter().find(|s| s.0 == values) {
            merged.insert(v, *k);
            continue;
        }
        pose(model, v, master, &values)?;
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
            unscript(model, v, r.master, &r.values)?;
            // Swaps only need to be marked to be undone: no definitions built.
            let ops: Vec<PartOp> = raw_ops(model, r.master, &r.values)?
                .into_iter()
                .map(|o| match o {
                    PartOp::Values(p, _) => PartOp::Swap(p, DefId::default()),
                    o => o,
                })
                .collect();
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
    let def = model.defs.get(master).ok_or_else(|| tr!("script-err-no-such-component"))?;
    if def.is_group {
        return Err(tr!("script-err-groups-can-t-have-parameters-make").into());
    }
    if def.generated.is_some() || def.params.is_some() || def.rigged.is_some() {
        return Err(tr!("script-err-this-component-is-built-by-a").into());
    }
    let (had, name) = (def.rig.is_some(), def.name.clone());
    match rig {
        Some(rig) => {
            validate(&rig)?;
            if !had {
                model.name_rig_parts(master);
            }
            let g = generator_of(&name, &rig);
            let defaults = rig_ops(model, master, &rig, &g.defaults())?;
            check_part_params(model, master, &defaults)?;
            // The script must run for the defaults (tried on a copy: placing copies doesn't
            // report errors).
            if !rig.script.trim().is_empty() {
                let mut t = model.clone();
                t.defs[master].rig = Some(rig.clone());
                let values = g.defaults();
                let v = t.new_rig_variant(master, values.clone(), &[]).ok_or_else(|| tr!("script-err-no-such-component"))?;
                pose(&mut t, v, master, &values)?;
            }
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
            return Err(tr!("script-err-k-twice-parameters-and-formulas-share", k = k.to_string()));
        }
    }
    for j in &rig.joints {
        if j.part.trim().is_empty() {
            return Err(tr!("script-err-a-joint-without-a-part").into());
        }
        if j.expr.trim().is_empty() && !keys.contains(j.param.as_str()) {
            return Err(tr!("script-err-part-choose-a-parameter-or-write", part = j.part.to_string()));
        }
        if let JointKind::Rotate { axis, .. } = j.kind
            && axis.length() < 1e-9
        {
            return Err(tr!("script-err-part-the-axis-has-no-direction", part = j.part.to_string()));
        }
        if let JointKind::Slide { dir } = j.kind
            && dir.length() < 1e-9
        {
            return Err(tr!("script-err-part-the-direction-is-zero", part = j.part.to_string()));
        }
        if let JointKind::Param { key } = &j.kind
            && key.trim().is_empty()
        {
            return Err(tr!("script-err-part-choose-the-part-s-parameter", part = j.part.to_string()));
        }
        if matches!(j.kind, JointKind::Visible) && j.expr.is_empty() && rig.params.iter().find(|p| p.key == j.param).is_some_and(|p| matches!(p.kind, GenKind::Material | GenKind::Choice(_))) {
            return Err(tr!("script-err-part-show-hide-needs-a-yes", part = j.part.to_string()));
        }
    }
    Ok(())
}

/// Parameters of parts set by `ops` exist and their parts have parameters.
fn check_part_params(model: &Model, master: DefId, ops: &[PartOp]) -> Result<(), String> {
    for op in ops {
        if let PartOp::Values(part, v) = op {
            let g = part_generator(model, master, part).ok_or_else(|| tr!("script-err-part-the-part-has-no-parameters", part = part.to_string()))?;
            for k in v.keys() {
                if g.param(k).is_none() {
                    return Err(tr!("script-err-part-no-parameter-k-it-has", part = part.to_string(), k = k.to_string(), keys = g.params.iter().map(|p| p.key.as_str()).collect::<Vec<_>>().join(", ")));
                }
            }
        }
    }
    Ok(())
}

/// Problems that don't stop the rig (for the editor): joints naming parts that aren't there or
/// parameters the part doesn't have.
pub fn warnings(model: &Model, master: DefId, rig: &Rig) -> Vec<String> {
    let parts: HashSet<String> = model.rig_parts(master).into_iter().map(|(_, n)| n).collect();
    let mut seen = HashSet::new();
    let mut out: Vec<String> =
        rig.joints.iter().filter(|j| !parts.contains(&j.part) && seen.insert(j.part.clone())).map(|j| tr!("script-err-no-part-part-in-the-component", part = j.part.to_string())).collect();
    for j in &rig.joints {
        let JointKind::Param { key } = &j.kind else { continue };
        if !parts.contains(&j.part) {
            continue;
        }
        match part_generator(model, master, &j.part) {
            None => out.push(tr!("script-err-part-has-no-parameters", part = j.part.to_string())),
            Some(g) if g.param(key).is_none() => out.push(tr!("script-err-part-has-no-parameter-key", part = j.part.to_string(), key = key.to_string())),
            _ => {}
        }
    }
    out
}
