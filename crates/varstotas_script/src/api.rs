//! The functions a script sees. Every function works on the thread-local [`State`].

use crate::{Entity, STATE, State};
use varstotas_kernel::glam::{DMat3, DMat4, DVec3};
use varstotas_kernel::model::Definition;
use varstotas_kernel::primitives::Primitive;
use varstotas_kernel::tessellate::triangulate_face;
use varstotas_kernel::{
    Aabb, ContextPath, CurveInfo, EdgeId, EntityRef, FaceId, Instance, InstanceId, Mesh, OpError, SolidOp, units,
};
use rhai::{Array, Dynamic, Engine, EvalAltResult, ImmutableString, Map, Scope};
use std::time::{Duration, Instant};

type R<T> = Result<T, Box<EvalAltResult>>;

/// API function names, for the console.
pub const NAMES: &[&str] = &[
    "vec3", "line", "polyline", "face", "rect", "circle", "polygon", "arc", "box", "cylinder", "pushpull", "offset",
    "follow_me", "erase", "move", "copy", "rotate", "scale", "group", "component", "explode", "make_unique", "union",
    "subtract", "intersect", "trim", "split", "outer_shell", "reverse", "soften", "material", "paint", "tag",
    "make_parametric", "set_params", "params", "dimension", "note", "enter", "exit", "faces", "edges", "groups", "all",
    "entities", "selection", "select", "bounds", "model_bounds", "length", "units", "materials", "all_faces", "read_text",
    "write_text", "generators", "add_generator", "generate", "reset_params", "file_exists", "list_files", "list_dirs",
    "parse_json", "fetch_text", "fetch_file", "child", "children", "show", "hide",
];

/// Files under `dir` (subfolders too, at most 5000) ending with `.ext` (any file for ""),
/// sorted.
pub fn list_files(dir: &std::path::Path, ext: &str) -> Vec<String> {
    fn walk(d: &std::path::Path, ext: &str, depth: usize, out: &mut Vec<String>) {
        let Ok(rd) = std::fs::read_dir(d) else { return };
        for e in rd.flatten() {
            if out.len() >= 5000 {
                return;
            }
            let p = e.path();
            if p.is_dir() {
                if depth < 8 {
                    walk(&p, ext, depth + 1, out);
                }
            } else if ext.is_empty() || p.extension().is_some_and(|x| x.to_string_lossy().eq_ignore_ascii_case(ext)) {
                out.push(p.to_string_lossy().to_string());
            }
        }
    }
    let mut out = Vec::new();
    walk(dir, ext, 0, &mut out);
    out.sort();
    out
}

/// GET `url` with the system curl (into the file `to`, else the bytes back), with extra
/// request headers (`#{ "Authorization": "Token …" }`).
fn fetch(url: &str, to: Option<&str>, headers: &Map) -> Result<Vec<u8>, String> {
    let mut cmd = std::process::Command::new(if cfg!(windows) { "curl.exe" } else { "curl" });
    cmd.args(["-sSfL", "--max-time", "300", "--retry", "2", "--user-agent", "Varstotas"]);
    for (k, v) in headers {
        cmd.arg("-H").arg(format!("{k}: {v}"));
    }
    if let Some(p) = to {
        cmd.arg("-o").arg(p);
    }
    let out = cmd.arg(url).output().map_err(|e| format!("curl: {e}"))?;
    if !out.status.success() {
        return Err(format!("{url}: {}", String::from_utf8_lossy(&out.stderr).trim()));
    }
    Ok(out.stdout)
}

fn err<T>(msg: impl Into<String>) -> R<T> {
    Err(msg.into().into())
}

fn op<T>(r: Result<T, OpError>) -> R<T> {
    r.map_err(|e| e.to_string().into())
}

/// Run `f` with the script state.
fn with<T>(f: impl FnOnce(&mut State) -> R<T>) -> R<T> {
    STATE.with(|s| {
        let mut b = s.borrow_mut();
        match b.as_mut() {
            Some(st) => f(st),
            None => err("no model"),
        }
    })
}

/// Like [`with`], marking the model changed.
fn edit<T>(f: impl FnOnce(&mut State) -> R<T>) -> R<T> {
    with(|st| {
        st.dirty = true;
        f(st)
    })
}

// ---------------------------------------------------------------------------------------
// Values

pub(crate) fn num(d: &Dynamic) -> R<f64> {
    if let Ok(f) = d.as_float() {
        return Ok(f);
    }
    if let Ok(i) = d.as_int() {
        return Ok(i as f64);
    }
    if let Some(s) = d.clone().try_cast::<ImmutableString>() {
        return with(|st| match units::parse_length(&s, st.model.unit) {
            Some(v) => Ok(v),
            None => err(format!("not a length: {s}")),
        });
    }
    err(format!("expected a number, got {}", d.type_name()))
}

pub(crate) fn pt(d: &Dynamic) -> R<DVec3> {
    if let Some(v) = d.clone().try_cast::<DVec3>() {
        return Ok(v);
    }
    if let Some(a) = d.clone().try_cast::<Array>()
        && (2..=3).contains(&a.len())
    {
        let z = if a.len() == 3 { num(&a[2])? } else { 0.0 };
        return Ok(DVec3::new(num(&a[0])?, num(&a[1])?, z));
    }
    err(format!("expected a point (vec3 or [x, y, z]), got {}", d.type_name()))
}

fn pts(d: &Dynamic) -> R<Vec<DVec3>> {
    match d.clone().try_cast::<Array>() {
        Some(a) => a.iter().map(pt).collect(),
        None => err("expected an array of points"),
    }
}

/// One entity or an array (nested arrays are flattened). A text names a group / component in
/// the current context ([`find_child`]).
fn ents(d: &Dynamic) -> R<Vec<Entity>> {
    if let Some(e) = d.clone().try_cast::<Entity>() {
        return Ok(vec![e]);
    }
    if let Some(name) = d.clone().try_cast::<ImmutableString>() {
        return with(|st| {
            let path = st.path.clone();
            find_child(st, &path, &name, false).map(|e| vec![e]).ok_or_else(|| format!("no group / component named `{name}` here").into())
        });
    }
    if let Some(a) = d.clone().try_cast::<Array>() {
        let mut out = Vec::new();
        for x in &a {
            out.extend(ents(x)?);
        }
        return Ok(out);
    }
    if d.is_unit() {
        return err("expected an entity, got () (did the operation fail?)");
    }
    err(format!("expected an entity or an array of entities, got {}", d.type_name()))
}

/// The name of instance `i` as found by [`find_child`]: its own, else its definition's.
fn inst_name(st: &State, i: &varstotas_kernel::Instance) -> String {
    if i.name.is_empty() { st.model.defs.get(i.def).map(|d| d.name.clone()).unwrap_or_default() } else { i.name.clone() }
}

/// The first group / component named `name` (its own name, else its definition's) in the
/// context `path`; with `deep` also inside the groups and components there, nearer ones first.
fn find_child(st: &State, path: &ContextPath, name: &str, deep: bool) -> Option<Entity> {
    let mut level = vec![path.clone()];
    for _ in 0..32 {
        let mut next = Vec::new();
        for p in &level {
            let Ok(m) = mesh(st, p) else { continue };
            let mut ids: Vec<InstanceId> = m.instances.keys().collect();
            ids.sort();
            if let Some(id) = ids.iter().find(|id| inst_name(st, &m.instances[**id]) == name) {
                return Some(Entity { path: p.clone(), r: EntityRef::Instance(*id) });
            }
            next.extend(ids.into_iter().map(|id| {
                let mut q = p.clone();
                q.push(id);
                q
            }));
        }
        if !deep || next.is_empty() {
            return None;
        }
        level = next;
    }
    None
}

/// The context inside group / component `x` (the current one for `()`).
fn inside(x: &Dynamic) -> R<ContextPath> {
    if x.is_unit() {
        return with(|st| Ok(st.path.clone()));
    }
    let g = one(x)?;
    with(|st| {
        let i = inst_id(st, &g)?;
        let mut p = g.path.clone();
        p.push(i);
        Ok(p)
    })
}

/// The groups / components of the context `path` (with `deep` also those inside them).
fn children_of(st: &State, path: &ContextPath, deep: bool, out: &mut Array, depth: usize) -> R<()> {
    let m = mesh(st, path)?;
    let mut ids: Vec<InstanceId> = m.instances.keys().collect();
    ids.sort();
    for id in ids {
        out.push(dyn_ent(path, EntityRef::Instance(id)));
        if deep && depth < 32 {
            let mut q = path.clone();
            q.push(id);
            children_of(st, &q, true, out, depth + 1)?;
        }
    }
    Ok(())
}

fn one(d: &Dynamic) -> R<Entity> {
    let mut v = ents(d)?;
    if v.len() != 1 {
        return err(format!("expected one entity, got {}", v.len()));
    }
    Ok(v.remove(0))
}

fn dyn_ent(path: &ContextPath, r: EntityRef) -> Dynamic {
    Dynamic::from(Entity { path: path.clone(), r })
}

fn dyn_ents(path: &ContextPath, rs: impl IntoIterator<Item = EntityRef>) -> Dynamic {
    Dynamic::from_array(rs.into_iter().map(|r| dyn_ent(path, r)).collect())
}

fn key_str(k: impl std::fmt::Debug) -> String {
    let s = format!("{k:?}");
    match (s.find('('), s.rfind(')')) {
        (Some(a), Some(b)) if b > a => s[a + 1..b].to_string(),
        _ => s,
    }
}

/// Text of a value for the console.
pub fn show(v: &Dynamic) -> String {
    if let Some(p) = v.clone().try_cast::<DVec3>() {
        return show_vec(p);
    }
    if let Some(e) = v.clone().try_cast::<Entity>() {
        return show_ent(&e);
    }
    if let Some(a) = v.clone().try_cast::<Array>() {
        return format!("[{}]", a.iter().map(show).collect::<Vec<_>>().join(", "));
    }
    if let Some(m) = v.clone().try_cast::<Map>() {
        return format!("#{{{}}}", m.iter().map(|(k, v)| format!("{k}: {}", show(v))).collect::<Vec<_>>().join(", "));
    }
    v.to_string()
}

fn trim(f: f64) -> String {
    let s = format!("{f:.4}");
    let s = s.trim_end_matches('0').trim_end_matches('.');
    if s == "-0" { "0".into() } else { s.into() }
}

fn show_vec(p: DVec3) -> String {
    format!("vec3({}, {}, {})", trim(p.x), trim(p.y), trim(p.z))
}

/// The entity's kind (`"?"` outside a run).
pub(crate) fn kind_of(e: &Entity) -> String {
    with(|st| Ok(kind(st, e))).unwrap_or_else(|_| "?".into())
}

fn show_ent(e: &Entity) -> String {
    let kind = kind_of(e);
    let id = match e.r {
        EntityRef::Vertex(v) => key_str(v),
        EntityRef::Edge(x) => key_str(x),
        EntityRef::Face(x) => key_str(x),
        EntityRef::Instance(x) => key_str(x),
    };
    format!("{kind}({id})")
}

// ---------------------------------------------------------------------------------------
// State helpers

fn mesh<'a>(st: &'a State, path: &ContextPath) -> R<&'a Mesh> {
    st.model.mesh_at(path).ok_or_else(|| "context no longer exists".into())
}

fn mesh_mut<'a>(st: &'a mut State, path: &ContextPath) -> R<&'a mut Mesh> {
    st.model.mesh_at_mut(path).ok_or_else(|| "context no longer exists".into())
}

fn world(st: &State, path: &ContextPath) -> DMat4 {
    st.model.transform_at(path)
}

/// World transform `m` expressed in the local frame of `path`.
fn local_xf(st: &State, path: &ContextPath, m: DMat4) -> DMat4 {
    let w = world(st, path);
    w.inverse() * m * w
}

fn alive(st: &State, e: &Entity) -> bool {
    st.model.mesh_at(&e.path).is_some_and(|m| match e.r {
        EntityRef::Vertex(v) => m.vertices.contains_key(v),
        EntityRef::Edge(x) => m.edges.contains_key(x),
        EntityRef::Face(x) => m.faces.contains_key(x),
        EntityRef::Instance(x) => m.instances.contains_key(x),
    })
}

fn check(st: &State, e: &Entity) -> R<()> {
    if alive(st, e) { Ok(()) } else { err("entity no longer exists (merged, split or erased)") }
}

fn kind(st: &State, e: &Entity) -> String {
    match e.r {
        EntityRef::Vertex(_) => "vertex".into(),
        EntityRef::Edge(_) => "edge".into(),
        EntityRef::Face(_) => "face".into(),
        EntityRef::Instance(i) => {
            let d = st.model.mesh_at(&e.path).and_then(|m| m.instances.get(i)).and_then(|i| st.model.defs.get(i.def));
            match d {
                Some(d) if d.params.is_some() => "parametric".into(),
                Some(d) if d.is_group => "group".into(),
                Some(_) => "component".into(),
                None => "instance".into(),
            }
        }
    }
}

fn face_id(st: &State, e: &Entity) -> R<FaceId> {
    check(st, e)?;
    match e.r {
        EntityRef::Face(f) => Ok(f),
        _ => err(format!("expected a face, got {}", kind(st, e))),
    }
}

fn inst_id(st: &State, e: &Entity) -> R<InstanceId> {
    check(st, e)?;
    match e.r {
        EntityRef::Instance(i) => Ok(i),
        _ => err(format!("expected a group or component, got {}", kind(st, e))),
    }
}

fn edge_ids(st: &State, es: &[Entity]) -> R<Vec<EdgeId>> {
    es.iter()
        .map(|e| {
            check(st, e)?;
            match e.r {
                EntityRef::Edge(x) => Ok(x),
                _ => err(format!("expected edges, got {}", kind(st, e))),
            }
        })
        .collect()
}

/// Entities grouped by context.
fn by_path(es: Vec<Entity>) -> Vec<(ContextPath, Vec<EntityRef>)> {
    let mut out: Vec<(ContextPath, Vec<EntityRef>)> = Vec::new();
    for e in es {
        match out.iter_mut().find(|(p, _)| *p == e.path) {
            Some((_, v)) => v.push(e.r),
            None => out.push((e.path, vec![e.r])),
        }
    }
    out
}

/// World points of an entity's vertices.
fn ent_points(st: &State, e: &Entity) -> R<Vec<DVec3>> {
    check(st, e)?;
    let m = mesh(st, &e.path)?;
    let w = world(st, &e.path);
    let p = |v| w.transform_point3(m.vertices[v].pos);
    Ok(match e.r {
        EntityRef::Vertex(v) => vec![p(v)],
        EntityRef::Edge(x) => m.edges[x].v.iter().map(|v| p(*v)).collect(),
        EntityRef::Face(f) => m.faces[f].outer.iter().map(|v| p(*v)).collect(),
        EntityRef::Instance(i) => {
            let inst = &m.instances[i];
            match st.model.def_bounds(inst.def) {
                Some(b) => {
                    let b = b.transformed(&(w * inst.transform));
                    vec![b.min, b.max]
                }
                None => vec![],
            }
        }
    })
}

fn ent_bounds(st: &State, es: &[Entity]) -> R<Option<Aabb>> {
    let mut b: Option<Aabb> = None;
    for e in es {
        for p in ent_points(st, e)? {
            let x = Aabb { min: p, max: p };
            b = Some(b.map_or(x, |b| b.union(&x)));
        }
    }
    Ok(b)
}

fn bounds_map(b: Option<Aabb>) -> Dynamic {
    match b {
        Some(b) => {
            let mut m = Map::new();
            m.insert("min".into(), Dynamic::from(b.min));
            m.insert("max".into(), Dynamic::from(b.max));
            m.insert("size".into(), Dynamic::from(b.max - b.min));
            m.insert("center".into(), Dynamic::from((b.min + b.max) * 0.5));
            Dynamic::from_map(m)
        }
        None => Dynamic::UNIT,
    }
}

/// The face bounded by a closed loop just added (the smallest face on its first edge).
fn loop_face(m: &Mesh, edges: &[EdgeId]) -> Option<FaceId> {
    let e = *edges.first()?;
    m.edge_faces(e).into_iter().min_by(|a, b| m.face_area(*a).total_cmp(&m.face_area(*b)))
}

/// A new free-standing face lying down is turned to face up (world +Z, `up` in the context),
/// so `pushpull(face, +d)` grows upwards (would leave a ground face facing down).
fn face_up(m: &mut Mesh, f: FaceId, up: DVec3) {
    if m.faces[f].normal.dot(up.normalize_or_zero()) < -0.5 && m.face_neighbors(f).is_empty() {
        m.reverse_face(f);
    }
}

fn curve_result(path: &ContextPath, m: &mut Mesh, edges: Vec<EdgeId>, up: DVec3) -> Dynamic {
    match loop_face(m, &edges) {
        Some(f) => {
            face_up(m, f, up);
            dyn_ent(path, EntityRef::Face(f))
        }
        None => dyn_ents(path, edges.into_iter().map(EntityRef::Edge)),
    }
}

/// Add a closed / open curve in the current context (world coordinates).
fn add_curve(info: impl FnOnce(&State, &DMat4) -> Option<CurveInfo>, closed: bool) -> R<Dynamic> {
    edit(|st| {
        let path = st.path.clone();
        let inv = world(st, &path).inverse();
        let Some(c) = info(st, &inv) else { return err("degenerate curve") };
        let up = inv.transform_vector3(DVec3::Z);
        let m = mesh_mut(st, &path)?;
        let edges = m.add_curve(&c);
        if edges.is_empty() {
            return err("curve not added");
        }
        Ok(if closed { curve_result(&path, m, edges, up) } else { dyn_ents(&path, edges.into_iter().map(EntityRef::Edge)) })
    })
}

/// Insert `mesh` as a new group at the world transform `xf` in the current context.
fn insert_group(st: &mut State, mesh: Mesh, xf: DMat4) -> R<Dynamic> {
    let path = st.path.clone();
    let local = world(st, &path).inverse() * xf;
    let def = st.model.defs.insert(Definition {
        name: "Group".into(),
        mesh,
        is_group: true,
        guides: Default::default(),
        params: None,
        generated: None,
        source: None,
        rig: None,
        rigged: None,
        preview: None,
    });
    let m = mesh_mut(st, &path)?;
    let id = m.instances.insert(Instance {
        def,
        transform: local,
        name: String::new(),
        material: None,
        tag: None,
        hidden: false,
        locked: false,
    });
    Ok(dyn_ent(&path, EntityRef::Instance(id)))
}

/// Extrude the face `f` of `m` by `h` along +Z (whatever way its normal points).
fn extrude_up(m: &mut Mesh, f: FaceId, h: f64) -> R<()> {
    let n = m.faces[f].normal;
    op(m.push_pull(f, if n.z > 0.0 { h } else { -h }, false)).map(|_| ())
}

fn transform_ents(st: &mut State, es: Vec<Entity>, world_m: DMat4, copy: bool) -> R<Vec<Entity>> {
    let mut out = Vec::new();
    for (path, refs) in by_path(es) {
        for r in &refs {
            check(st, &Entity { path: path.clone(), r: *r })?;
        }
        let m = local_xf(st, &path, world_m);
        let mesh = mesh_mut(st, &path)?;
        if copy {
            out.extend(mesh.duplicate(&refs, &m).into_iter().map(|r| Entity { path: path.clone(), r }));
            continue;
        }
        let vs = mesh.vertices_of(&refs);
        if !vs.is_empty() {
            mesh.transform_vertices(&vs, &m);
        }
        for r in &refs {
            if let EntityRef::Instance(i) = r
                && let Some(inst) = mesh.instances.get_mut(*i)
            {
                inst.transform = m * inst.transform;
            }
        }
        out.extend(refs.into_iter().map(|r| Entity { path: path.clone(), r }).filter(|e| alive(st, e)));
    }
    Ok(out)
}

fn color(d: &Dynamic) -> R<[f32; 4]> {
    if let Some(s) = d.clone().try_cast::<ImmutableString>() {
        let h = s.trim_start_matches('#');
        let v = u32::from_str_radix(h, 16).map_err(|_| format!("bad colour {s}"))?;
        return match h.len() {
            6 => Ok([(v >> 16) as f32 / 255.0, ((v >> 8) & 255) as f32 / 255.0, (v & 255) as f32 / 255.0, 1.0]),
            8 => Ok([(v >> 24) as f32 / 255.0, ((v >> 16) & 255) as f32 / 255.0, ((v >> 8) & 255) as f32 / 255.0, (v & 255) as f32 / 255.0]),
            _ => err(format!("bad colour {s} (use #RRGGBB)")),
        };
    }
    if let Some(a) = d.clone().try_cast::<Array>()
        && (3..=4).contains(&a.len())
    {
        let v: Vec<f64> = a.iter().map(num).collect::<R<_>>()?;
        let k = if v.iter().take(3).any(|c| *c > 1.0) { 255.0 } else { 1.0 };
        return Ok([(v[0] / k) as f32, (v[1] / k) as f32, (v[2] / k) as f32, v.get(3).map_or(1.0, |a| (*a / if *a > 1.0 { 255.0 } else { 1.0 }) as f32)]);
    }
    err("expected a colour: \"#RRGGBB\" or [r, g, b]")
}

fn solid(a: &Dynamic, b: &Dynamic, op_: SolidOp, swap: bool) -> R<Dynamic> {
    let (a, b) = (one(a)?, one(b)?);
    edit(|st| {
        let (ia, ib) = (inst_id(st, &a)?, inst_id(st, &b)?);
        if a.path != b.path {
            return err("both solids must be in the same context");
        }
        let (first, second) = if swap { (ib, ia) } else { (ia, ib) };
        let out = op(st.model.solid_op_in(&a.path, op_, first, second))?;
        Ok(dyn_ents(&a.path, out.into_iter().map(EntityRef::Instance)))
    })
}

fn params_map(p: &varstotas_kernel::Parametric) -> Map {
    let mut m = Map::new();
    for (k, n) in ["x", "y", "z"].into_iter().enumerate() {
        m.insert(n.into(), p.size[k].map_or(Dynamic::UNIT, Dynamic::from_float));
    }
    m.insert("count".into(), Dynamic::from_int(p.count as i64));
    m.insert("axis".into(), Dynamic::from(["x", "y", "z"][p.axis as usize].to_string()));
    m.insert("gap".into(), Dynamic::from_float(p.gap));
    m.insert("stretch".into(), Dynamic::from_bool(p.stretch));
    m
}

// ---------------------------------------------------------------------------------------

pub fn scope() -> Scope<'static> {
    let mut s = Scope::new();
    s.push_constant("ORIGIN", DVec3::ZERO);
    s.push_constant("X", DVec3::X);
    s.push_constant("Y", DVec3::Y);
    s.push_constant("Z", DVec3::Z);
    s
}

pub fn engine(limit: Duration) -> Engine {
    let mut e = Engine::new();
    e.set_max_expr_depths(256, 256);
    e.set_max_call_levels(128);
    let start = Instant::now();
    e.on_progress(move |_| (start.elapsed() > limit).then(|| Dynamic::from(format!("time limit of {}s exceeded", limit.as_secs()))));
    e.on_print(|s| {
        let _ = with(|st| {
            st.out.push_str(s);
            st.out.push('\n');
            Ok(())
        });
    });
    e.on_debug(|s, _, _| {
        let _ = with(|st| {
            st.out.push_str(s);
            st.out.push('\n');
            Ok(())
        });
    });
    register_vec(&mut e);
    register_entity(&mut e);
    register_geometry(&mut e);
    register_ops(&mut e);
    register_context(&mut e);
    register_generators(&mut e);
    crate::generator::register_math(&mut e);
    e
}

pub(crate) fn register_vec(e: &mut Engine) {
    e.register_type_with_name::<DVec3>("Vec3");
    e.register_fn("vec3", |x: Dynamic, y: Dynamic, z: Dynamic| -> R<DVec3> { Ok(DVec3::new(num(&x)?, num(&y)?, num(&z)?)) });
    e.register_fn("vec3", |x: Dynamic, y: Dynamic| -> R<DVec3> { Ok(DVec3::new(num(&x)?, num(&y)?, 0.0)) });
    e.register_get_set("x", |v: &mut DVec3| v.x, |v: &mut DVec3, x: f64| v.x = x);
    e.register_get_set("y", |v: &mut DVec3| v.y, |v: &mut DVec3, x: f64| v.y = x);
    e.register_get_set("z", |v: &mut DVec3| v.z, |v: &mut DVec3, x: f64| v.z = x);
    e.register_fn("length", |v: &mut DVec3| v.length());
    e.register_fn("normalize", |v: &mut DVec3| v.normalize_or_zero());
    e.register_fn("dot", |a: &mut DVec3, b: DVec3| a.dot(b));
    e.register_fn("cross", |a: &mut DVec3, b: DVec3| a.cross(b));
    e.register_fn("distance", |a: &mut DVec3, b: DVec3| a.distance(b));
    e.register_fn("+", |a: DVec3, b: DVec3| a + b);
    e.register_fn("-", |a: DVec3, b: DVec3| a - b);
    e.register_fn("-", |a: DVec3| -a);
    e.register_fn("*", |a: DVec3, k: f64| a * k);
    e.register_fn("*", |a: DVec3, k: i64| a * k as f64);
    e.register_fn("*", |k: f64, a: DVec3| a * k);
    e.register_fn("*", |k: i64, a: DVec3| a * k as f64);
    e.register_fn("/", |a: DVec3, k: f64| a / k);
    e.register_fn("/", |a: DVec3, k: i64| a / k as f64);
    e.register_fn("==", |a: DVec3, b: DVec3| a.distance(b) < 1e-9);
    e.register_fn("!=", |a: DVec3, b: DVec3| a.distance(b) >= 1e-9);
    e.register_fn("to_string", |v: &mut DVec3| show_vec(*v));
    e.register_fn("to_debug", |v: &mut DVec3| show_vec(*v));
}

fn register_entity(e: &mut Engine) {
    e.register_type_with_name::<Entity>("Entity");
    e.register_fn("to_string", |x: &mut Entity| show_ent(x));
    e.register_fn("to_debug", |x: &mut Entity| show_ent(x));
    e.register_fn("==", |a: Entity, b: Entity| a == b);
    e.register_fn("!=", |a: Entity, b: Entity| a != b);
    e.register_get("kind", |x: &mut Entity| with(|st| Ok(kind(st, x))));
    e.register_get("valid", |x: &mut Entity| with(|st| Ok(alive(st, x))));
    e.register_get("area", |x: &mut Entity| -> R<f64> {
        with(|st| {
            let f = face_id(st, x)?;
            let w = world(st, &x.path);
            let m = mesh(st, &x.path)?;
            Ok(triangulate_face(m, f)
                .iter()
                .map(|t| {
                    let p = t.map(|p| w.transform_point3(p));
                    (p[1] - p[0]).cross(p[2] - p[0]).length() * 0.5
                })
                .sum())
        })
    });
    e.register_get("length", |x: &mut Entity| -> R<f64> {
        with(|st| match x.r {
            EntityRef::Edge(_) => {
                let p = ent_points(st, x)?;
                Ok(p[0].distance(p[1]))
            }
            _ => err("length: not an edge"),
        })
    });
    e.register_get("normal", |x: &mut Entity| -> R<DVec3> {
        with(|st| {
            let f = face_id(st, x)?;
            let n = mesh(st, &x.path)?.faces[f].normal;
            let w = DMat3::from_mat4(world(st, &x.path));
            Ok((w.inverse().transpose() * n).normalize_or_zero())
        })
    });
    e.register_get("center", |x: &mut Entity| -> R<DVec3> {
        with(|st| Ok(ent_bounds(st, std::slice::from_ref(x))?.map_or(DVec3::ZERO, |b| (b.min + b.max) * 0.5)))
    });
    e.register_get("bounds", |x: &mut Entity| -> R<Dynamic> { with(|st| Ok(bounds_map(ent_bounds(st, std::slice::from_ref(x))?))) });
    e.register_get("size", |x: &mut Entity| -> R<DVec3> {
        with(|st| Ok(ent_bounds(st, std::slice::from_ref(x))?.map_or(DVec3::ZERO, |b| b.max - b.min)))
    });
    e.register_get("min", |x: &mut Entity| -> R<DVec3> { with(|st| Ok(ent_bounds(st, std::slice::from_ref(x))?.map_or(DVec3::ZERO, |b| b.min))) });
    e.register_get("max", |x: &mut Entity| -> R<DVec3> { with(|st| Ok(ent_bounds(st, std::slice::from_ref(x))?.map_or(DVec3::ZERO, |b| b.max))) });
    e.register_get_set(
        "hidden",
        |x: &mut Entity| -> R<bool> {
            with(|st| {
                let i = inst_id(st, x)?;
                Ok(mesh(st, &x.path)?.instances[i].hidden)
            })
        },
        |x: &mut Entity, h: bool| -> R<()> {
            edit(|st| {
                let i = inst_id(st, x)?;
                mesh_mut(st, &x.path)?.instances[i].hidden = h;
                Ok(())
            })
        },
    );
    e.register_get("vertices", |x: &mut Entity| -> R<Array> {
        with(|st| Ok(ent_points(st, x)?.into_iter().map(Dynamic::from).collect()))
    });
    e.register_get("edges", |x: &mut Entity| -> R<Array> {
        with(|st| {
            let f = face_id(st, x)?;
            Ok(mesh(st, &x.path)?.face_edges(f).into_iter().map(|e| dyn_ent(&x.path, EntityRef::Edge(e))).collect())
        })
    });
    e.register_get("faces", |x: &mut Entity| -> R<Array> {
        with(|st| {
            check(st, x)?;
            let m = mesh(st, &x.path)?;
            let fs = match x.r {
                EntityRef::Edge(e) => m.edge_faces(e),
                EntityRef::Face(f) => m.face_neighbors(f),
                _ => return err("faces: not an edge or face"),
            };
            Ok(fs.into_iter().map(|f| dyn_ent(&x.path, EntityRef::Face(f))).collect())
        })
    });
    e.register_get_set(
        "name",
        |x: &mut Entity| -> R<String> {
            with(|st| {
                let i = inst_id(st, x)?;
                Ok(mesh(st, &x.path)?.instances[i].name.clone())
            })
        },
        |x: &mut Entity, n: String| -> R<()> {
            edit(|st| {
                let i = inst_id(st, x)?;
                mesh_mut(st, &x.path)?.instances[i].name = n;
                Ok(())
            })
        },
    );
    e.register_get("definition", |x: &mut Entity| -> R<String> {
        with(|st| {
            let i = inst_id(st, x)?;
            let d = mesh(st, &x.path)?.instances[i].def;
            Ok(st.model.defs.get(d).map(|d| d.name.clone()).unwrap_or_default())
        })
    });
    e.register_get("material", |x: &mut Entity| -> R<Dynamic> {
        with(|st| {
            check(st, x)?;
            let m = mesh(st, &x.path)?;
            let id = match x.r {
                EntityRef::Face(f) => m.faces[f].front,
                EntityRef::Instance(i) => m.instances[i].material,
                _ => None,
            };
            Ok(id.and_then(|id| st.model.materials.get(id)).map_or(Dynamic::UNIT, |m| Dynamic::from(m.name.clone())))
        })
    });
    e.register_get("tag", |x: &mut Entity| -> R<Dynamic> {
        with(|st| {
            check(st, x)?;
            let m = mesh(st, &x.path)?;
            let id = match x.r {
                EntityRef::Face(f) => m.faces[f].tag,
                EntityRef::Edge(e) => m.edges[e].tag,
                EntityRef::Instance(i) => m.instances[i].tag,
                EntityRef::Vertex(_) => None,
            };
            Ok(id.and_then(|id| st.model.tags.get(id)).map_or(Dynamic::UNIT, |t| Dynamic::from(t.name.clone())))
        })
    });
    e.register_get("soft", |x: &mut Entity| -> R<bool> {
        with(|st| {
            check(st, x)?;
            match x.r {
                EntityRef::Edge(e) => Ok(mesh(st, &x.path)?.edges[e].soft),
                _ => err("soft: not an edge"),
            }
        })
    });
}

fn register_geometry(e: &mut Engine) {
    e.register_fn("line", |a: Dynamic, b: Dynamic| -> R<Dynamic> {
        let (a, b) = (pt(&a)?, pt(&b)?);
        edit(|st| {
            let path = st.path.clone();
            let inv = world(st, &path).inverse();
            let es = mesh_mut(st, &path)?.add_line(inv.transform_point3(a), inv.transform_point3(b));
            Ok(dyn_ents(&path, es.into_iter().map(EntityRef::Edge)))
        })
    });
    fn polyline(p: &Dynamic, closed: bool) -> R<Dynamic> {
        let p = pts(p)?;
        edit(|st| {
            let path = st.path.clone();
            let inv = world(st, &path).inverse();
            let p: Vec<DVec3> = p.iter().map(|p| inv.transform_point3(*p)).collect();
            let up = inv.transform_vector3(DVec3::Z);
            let m = mesh_mut(st, &path)?;
            let es = m.add_polyline(&p, closed, false);
            if closed && let Some(f) = loop_face(m, &es) {
                face_up(m, f, up);
            }
            Ok(dyn_ents(&path, es.into_iter().map(EntityRef::Edge)))
        })
    }
    e.register_fn("polyline", |p: Dynamic| polyline(&p, false));
    e.register_fn("polyline", |p: Dynamic, closed: bool| polyline(&p, closed));
    fn face(p: Vec<DVec3>) -> R<Dynamic> {
        edit(|st| {
            let path = st.path.clone();
            let inv = world(st, &path).inverse();
            let p: Vec<DVec3> = p.iter().map(|p| inv.transform_point3(*p)).collect();
            let up = world(st, &path).inverse().transform_vector3(DVec3::Z);
            let m = mesh_mut(st, &path)?;
            match m.add_face(&p) {
                Some(f) => {
                    face_up(m, f, up);
                    Ok(dyn_ent(&path, EntityRef::Face(f)))
                }
                None => err("face: points are not a valid planar polygon"),
            }
        })
    }
    e.register_fn("face", |p: Dynamic| face(pts(&p)?));
    e.register_fn("rect", |o: Dynamic, w: Dynamic, d: Dynamic| -> R<Dynamic> {
        let (o, w, d) = (pt(&o)?, num(&w)?, num(&d)?);
        face(vec![o, o + DVec3::X * w, o + DVec3::new(w, d, 0.0), o + DVec3::Y * d])
    });
    fn circle(c: &Dynamic, r: &Dynamic, segs: i64, n: DVec3, polygon: bool) -> R<Dynamic> {
        let (c, r) = (pt(c)?, num(r)?);
        if r <= 0.0 || !(3..=1000).contains(&segs) {
            return err("circle: radius must be positive, segments 3..1000");
        }
        add_curve(
            |_, inv| {
                let (c, n) = (inv.transform_point3(c), inv.transform_vector3(n));
                let start = n.any_orthonormal_vector();
                let start = if n.cross(DVec3::X).length() > 1e-6 { DVec3::X } else { start };
                if polygon {
                    CurveInfo::polygon(c, n, start, r, segs as u32)
                } else {
                    CurveInfo::circle(c, n, start, r, segs as u32)
                }
            },
            true,
        )
    }
    e.register_fn("circle", |c: Dynamic, r: Dynamic| circle(&c, &r, 24, DVec3::Z, false));
    e.register_fn("circle", |c: Dynamic, r: Dynamic, s: i64| circle(&c, &r, s, DVec3::Z, false));
    e.register_fn("circle", |c: Dynamic, r: Dynamic, s: i64, n: Dynamic| circle(&c, &r, s, pt(&n)?, false));
    e.register_fn("polygon", |c: Dynamic, r: Dynamic, s: i64| circle(&c, &r, s, DVec3::Z, true));
    e.register_fn("polygon", |c: Dynamic, r: Dynamic, s: i64, n: Dynamic| circle(&c, &r, s, pt(&n)?, true));
    fn arc(c: &Dynamic, s: &Dynamic, deg: &Dynamic, segs: i64, n: DVec3) -> R<Dynamic> {
        let (c, s, deg) = (pt(c)?, pt(s)?, num(deg)?);
        if !(1..=1000).contains(&segs) {
            return err("arc: segments 1..1000");
        }
        add_curve(
            |_, inv| {
                CurveInfo::arc_center(
                    inv.transform_point3(c),
                    inv.transform_vector3(n),
                    inv.transform_point3(s),
                    deg.to_radians(),
                    segs as u32,
                )
            },
            false,
        )
    }
    e.register_fn("arc", |c: Dynamic, s: Dynamic, deg: Dynamic| arc(&c, &s, &deg, 12, DVec3::Z));
    e.register_fn("arc", |c: Dynamic, s: Dynamic, deg: Dynamic, n: i64| arc(&c, &s, &deg, n, DVec3::Z));
    e.register_fn("arc", |c: Dynamic, s: Dynamic, deg: Dynamic, n: i64, nn: Dynamic| arc(&c, &s, &deg, n, pt(&nn)?));
    e.register_fn("box", |o: Dynamic, sx: Dynamic, sy: Dynamic, sz: Dynamic| -> R<Dynamic> {
        let (o, s) = (pt(&o)?, DVec3::new(num(&sx)?, num(&sy)?, num(&sz)?));
        if s.abs().min_element() <= 1e-6 {
            return err("box: sizes must be non-zero");
        }
        let lo = o + s.min(DVec3::ZERO);
        let s = s.abs();
        edit(|st| {
            let mut m = Mesh::new();
            let f = m
                .add_face(&[DVec3::ZERO, DVec3::X * s.x, DVec3::new(s.x, s.y, 0.0), DVec3::Y * s.y])
                .ok_or("box: degenerate")?;
            extrude_up(&mut m, f, s.z)?;
            insert_group(st, m, DMat4::from_translation(lo))
        })
    });
    fn cylinder(c: &Dynamic, r: &Dynamic, h: &Dynamic, segs: i64) -> R<Dynamic> {
        let (c, r, h) = (pt(c)?, num(r)?, num(h)?);
        if r <= 0.0 || h.abs() <= 1e-6 || !(3..=1000).contains(&segs) {
            return err("cylinder: radius and height must be non-zero, segments 3..1000");
        }
        edit(|st| {
            let mut m = Mesh::new();
            let info = CurveInfo::circle(DVec3::ZERO, DVec3::Z, DVec3::X, r, segs as u32).ok_or("cylinder: degenerate")?;
            let es = m.add_curve(&info);
            let f = loop_face(&m, &es).ok_or("cylinder: no face")?;
            extrude_up(&mut m, f, h.abs())?;
            let base = if h < 0.0 { c + DVec3::Z * h } else { c };
            insert_group(st, m, DMat4::from_translation(base))
        })
    }
    e.register_fn("cylinder", |c: Dynamic, r: Dynamic, h: Dynamic| cylinder(&c, &r, &h, 24));
    e.register_fn("cylinder", |c: Dynamic, r: Dynamic, h: Dynamic, s: i64| cylinder(&c, &r, &h, s));
    /// A [`Primitive`] (in its own axes) as a group moved to `at`.
    fn primitive(p: Primitive, at: DVec3, what: &str) -> R<Dynamic> {
        let m = p.mesh().ok_or_else(|| format!("{what}: sizes must be positive (a torus tube thinner than its radius), segments 3..1000"))?;
        edit(|st| insert_group(st, m, DMat4::from_translation(at)))
    }
    fn segs(s: i64) -> u32 {
        s.clamp(0, 100_000) as u32
    }
    e.register_fn("cone", |c: Dynamic, r: Dynamic, h: Dynamic| -> R<Dynamic> {
        primitive(Primitive::Cone { r: num(&r)?, h: num(&h)?, segs: 24 }, pt(&c)?, "cone")
    });
    e.register_fn("cone", |c: Dynamic, r: Dynamic, h: Dynamic, s: i64| -> R<Dynamic> {
        primitive(Primitive::Cone { r: num(&r)?, h: num(&h)?, segs: segs(s) }, pt(&c)?, "cone")
    });
    e.register_fn("pyramid", |o: Dynamic, w: Dynamic, d: Dynamic, h: Dynamic| -> R<Dynamic> {
        primitive(Primitive::Pyramid { w: num(&w)?, d: num(&d)?, h: num(&h)? }, pt(&o)?, "pyramid")
    });
    e.register_fn("sphere", |c: Dynamic, r: Dynamic| -> R<Dynamic> { primitive(Primitive::Sphere { r: num(&r)?, segs: 24 }, pt(&c)?, "sphere") });
    e.register_fn("sphere", |c: Dynamic, r: Dynamic, s: i64| -> R<Dynamic> {
        primitive(Primitive::Sphere { r: num(&r)?, segs: segs(s) }, pt(&c)?, "sphere")
    });
    e.register_fn("torus", |c: Dynamic, r: Dynamic, t: Dynamic| -> R<Dynamic> {
        primitive(Primitive::Torus { r: num(&r)?, tube: num(&t)?, segs: 24, sides: 12 }, pt(&c)?, "torus")
    });
    e.register_fn("torus", |c: Dynamic, r: Dynamic, t: Dynamic, s: i64, sides: i64| -> R<Dynamic> {
        primitive(Primitive::Torus { r: num(&r)?, tube: num(&t)?, segs: segs(s), sides: segs(sides) }, pt(&c)?, "torus")
    });
    e.register_fn("dimension", |a: Dynamic, b: Dynamic, off: Dynamic| -> R<()> {
        let (a, b, off) = (pt(&a)?, pt(&b)?, pt(&off)?);
        edit(|st| {
            let path = st.path.clone();
            let inv = world(st, &path).inverse();
            st.model
                .add_dimension_in(&path, inv.transform_point3(a), inv.transform_point3(b), inv.transform_vector3(off))
                .map(|_| ())
                .ok_or_else(|| "dimension: not added".into())
        })
    });
    e.register_fn("note", |at: Dynamic, off: Dynamic, text: ImmutableString| -> R<()> {
        let (at, off) = (pt(&at)?, pt(&off)?);
        edit(|st| {
            let path = st.path.clone();
            let inv = world(st, &path).inverse();
            st.model
                .add_text_note_in(&path, inv.transform_point3(at), inv.transform_vector3(off), text.to_string())
                .map(|_| ())
                .ok_or_else(|| "note: not added".into())
        })
    });
}

fn register_ops(e: &mut Engine) {
    e.register_fn("pushpull", |f: Dynamic, d: Dynamic| -> R<Dynamic> {
        let (f, d) = (one(&f)?, num(&d)?);
        edit(|st| {
            let id = face_id(st, &f)?;
            let nf = op(mesh_mut(st, &f.path)?.push_pull(id, d, false))?;
            Ok(dyn_ent(&f.path, EntityRef::Face(nf)))
        })
    });
    e.register_fn("offset", |f: Dynamic, d: Dynamic| -> R<Dynamic> {
        let (f, d) = (one(&f)?, num(&d)?);
        edit(|st| {
            let id = face_id(st, &f)?;
            let nf = op(mesh_mut(st, &f.path)?.offset_face(id, d))?;
            Ok(dyn_ent(&f.path, EntityRef::Face(nf)))
        })
    });
    e.register_fn("follow_me", |f: Dynamic, path: Dynamic| -> R<()> {
        let (f, p) = (one(&f)?, ents(&path)?);
        edit(|st| {
            let id = face_id(st, &f)?;
            if p.iter().any(|e| e.path != f.path) {
                return err("follow_me: the path must be in the face's context");
            }
            let edges = edge_ids(st, &p)?;
            op(mesh_mut(st, &f.path)?.follow_me(id, &edges))
        })
    });
    e.register_fn("erase", |x: Dynamic| -> R<()> {
        let es = ents(&x)?;
        edit(|st| {
            for (path, refs) in by_path(es) {
                mesh_mut(st, &path)?.erase(&refs);
            }
            Ok(())
        })
    });
    e.register_fn("move", |x: Dynamic, v: Dynamic| -> R<Array> {
        let (es, v) = (ents(&x)?, pt(&v)?);
        edit(|st| Ok(transform_ents(st, es, DMat4::from_translation(v), false)?.into_iter().map(Dynamic::from).collect()))
    });
    e.register_fn("copy", |x: Dynamic, v: Dynamic| -> R<Array> {
        let (es, v) = (ents(&x)?, pt(&v)?);
        edit(|st| Ok(transform_ents(st, es, DMat4::from_translation(v), true)?.into_iter().map(Dynamic::from).collect()))
    });
    e.register_fn("rotate", |x: Dynamic, c: Dynamic, axis: Dynamic, deg: Dynamic| -> R<Array> {
        let (es, c, a, deg) = (ents(&x)?, pt(&c)?, pt(&axis)?, num(&deg)?);
        if a.length() < 1e-9 {
            return err("rotate: zero axis");
        }
        let m = DMat4::from_translation(c) * DMat4::from_axis_angle(a.normalize(), deg.to_radians()) * DMat4::from_translation(-c);
        edit(|st| Ok(transform_ents(st, es, m, false)?.into_iter().map(Dynamic::from).collect()))
    });
    e.register_fn("scale", |x: Dynamic, c: Dynamic, k: Dynamic| -> R<Array> {
        let (es, c) = (ents(&x)?, pt(&c)?);
        let k = if k.is::<DVec3>() || k.is_array() { pt(&k)? } else { DVec3::splat(num(&k)?) };
        if k.abs().min_element() < 1e-9 {
            return err("scale: zero factor");
        }
        let m = DMat4::from_translation(c) * DMat4::from_scale(k) * DMat4::from_translation(-c);
        edit(|st| Ok(transform_ents(st, es, m, false)?.into_iter().map(Dynamic::from).collect()))
    });
    fn make(x: &Dynamic, name: Option<&str>) -> R<Dynamic> {
        let es = ents(x)?;
        edit(|st| {
            let groups = by_path(es);
            let [(path, refs)] = groups.as_slice() else { return err("group: entities must be in one context") };
            let id = match name {
                Some(n) => op(st.model.make_component(path, refs, n))?,
                None => op(st.model.make_group(path, refs))?,
            };
            Ok(dyn_ent(path, EntityRef::Instance(id)))
        })
    }
    e.register_fn("group", |x: Dynamic| make(&x, None));
    e.register_fn("component", |x: Dynamic, n: ImmutableString| make(&x, Some(n.as_str())));
    e.register_fn("explode", |x: Dynamic| -> R<Dynamic> {
        let g = one(&x)?;
        edit(|st| {
            let i = inst_id(st, &g)?;
            let out = op(st.model.explode(&g.path, i))?;
            Ok(dyn_ents(&g.path, out))
        })
    });
    e.register_fn("make_unique", |x: Dynamic| -> R<()> {
        let g = one(&x)?;
        edit(|st| {
            let i = inst_id(st, &g)?;
            op(st.model.make_unique(&g.path, i))
        })
    });
    e.register_fn("union", |a: Dynamic, b: Dynamic| solid(&a, &b, SolidOp::Union, false));
    e.register_fn("intersect", |a: Dynamic, b: Dynamic| solid(&a, &b, SolidOp::Intersect, false));
    // a minus b: the kernel's Subtract / Trim keep "second minus first".
    e.register_fn("subtract", |a: Dynamic, b: Dynamic| solid(&a, &b, SolidOp::Subtract, true));
    e.register_fn("trim", |a: Dynamic, b: Dynamic| solid(&a, &b, SolidOp::Trim, true));
    e.register_fn("split", |a: Dynamic, b: Dynamic| solid(&a, &b, SolidOp::Split, false));
    e.register_fn("outer_shell", |x: Dynamic| -> R<Dynamic> {
        let es = ents(&x)?;
        edit(|st| {
            let groups = by_path(es);
            let [(path, refs)] = groups.as_slice() else { return err("outer_shell: solids must be in one context") };
            let ids: Vec<InstanceId> = refs.iter().filter_map(|r| if let EntityRef::Instance(i) = r { Some(*i) } else { None }).collect();
            let id = op(st.model.solid_union_all_in(path, &ids))?;
            Ok(dyn_ent(path, EntityRef::Instance(id)))
        })
    });
    e.register_fn("reverse", |x: Dynamic| -> R<()> {
        let es = ents(&x)?;
        edit(|st| {
            for (path, refs) in by_path(es) {
                let fs: Vec<FaceId> = refs.iter().filter_map(|r| if let EntityRef::Face(f) = r { Some(*f) } else { None }).collect();
                mesh_mut(st, &path)?.reverse_faces(&fs);
            }
            Ok(())
        })
    });
    e.register_fn("soften", |x: Dynamic, deg: Dynamic| -> R<i64> {
        let (es, deg) = (ents(&x)?, num(&deg)?);
        edit(|st| {
            let mut n = 0;
            for (path, refs) in by_path(es) {
                let m = mesh_mut(st, &path)?;
                let mut edges: Vec<EdgeId> = Vec::new();
                for r in refs {
                    match r {
                        EntityRef::Edge(e) => edges.push(e),
                        EntityRef::Face(f) if m.faces.contains_key(f) => edges.extend(m.face_edges(f)),
                        _ => {}
                    }
                }
                edges.sort();
                edges.dedup();
                n += m.soften_by_angle(&edges, deg, true, false) as i64;
            }
            Ok(n)
        })
    });
    e.register_fn("material", |name: ImmutableString, c: Dynamic| -> R<ImmutableString> {
        let c = color(&c)?;
        edit(|st| {
            match st.model.materials.values_mut().find(|m| m.name == name.as_str()) {
                Some(m) => m.color = c,
                None => {
                    st.model.add_material(&name, c);
                }
            }
            Ok(name.clone())
        })
    });
    fn set_hidden(x: &Dynamic, hidden: bool) -> R<()> {
        let es = ents(x)?;
        edit(|st| {
            for (path, refs) in by_path(es) {
                let m = mesh_mut(st, &path)?;
                for r in refs {
                    match r {
                        EntityRef::Instance(i) => {
                            if let Some(i) = m.instances.get_mut(i) {
                                i.hidden = hidden;
                            }
                        }
                        _ => return err("show / hide: groups and components only"),
                    }
                }
            }
            Ok(())
        })
    }
    e.register_fn("show", |x: Dynamic| set_hidden(&x, false));
    e.register_fn("show", |x: Dynamic, on: bool| set_hidden(&x, !on));
    e.register_fn("hide", |x: Dynamic| set_hidden(&x, true));
    e.register_fn("paint", |x: Dynamic, name: Dynamic| -> R<()> {
        let es = ents(&x)?;
        let name = if name.is_unit() { None } else { Some(name.into_immutable_string().map_err(|t| format!("paint: expected a material name, got {t}"))?) };
        edit(|st| {
            let mat = match &name {
                Some(n) => Some(
                    st.model
                        .materials
                        .iter()
                        .find(|(_, m)| m.name == n.as_str())
                        .map(|(id, _)| id)
                        .ok_or_else(|| format!("paint: no material {n} (create it with material(name, colour))"))?,
                ),
                None => None,
            };
            for (path, refs) in by_path(es) {
                let m = mesh_mut(st, &path)?;
                for r in refs {
                    match r {
                        EntityRef::Face(f) => {
                            if let Some(f) = m.faces.get_mut(f) {
                                f.front = mat;
                            }
                        }
                        EntityRef::Instance(i) => {
                            if let Some(i) = m.instances.get_mut(i) {
                                i.material = mat;
                            }
                        }
                        _ => {}
                    }
                }
            }
            Ok(())
        })
    });
    e.register_fn("tag", |x: Dynamic, name: ImmutableString| -> R<()> {
        let es = ents(&x)?;
        edit(|st| {
            let t = match st.model.tags.iter().find(|(_, t)| t.name == name.as_str()) {
                Some((id, _)) => id,
                None => st.model.add_tag(&name),
            };
            for (path, refs) in by_path(es) {
                let m = mesh_mut(st, &path)?;
                for r in refs {
                    match r {
                        EntityRef::Face(f) => m.faces.get_mut(f).into_iter().for_each(|f| f.tag = Some(t)),
                        EntityRef::Edge(e) => m.edges.get_mut(e).into_iter().for_each(|e| e.tag = Some(t)),
                        EntityRef::Instance(i) => m.instances.get_mut(i).into_iter().for_each(|i| i.tag = Some(t)),
                        EntityRef::Vertex(_) => {}
                    }
                }
            }
            Ok(())
        })
    });
    e.register_fn("make_parametric", |x: Dynamic| -> R<Dynamic> {
        let g = one(&x)?;
        edit(|st| {
            let i = inst_id(st, &g)?;
            op(st.model.make_parametric(&g.path, i))?;
            Ok(Dynamic::from(g.clone()))
        })
    });
    e.register_fn("params", |x: Dynamic| -> R<Dynamic> {
        let g = one(&x)?;
        with(|st| {
            let i = inst_id(st, &g)?;
            if let Some(gd) = st.model.instance_generated(&g.path, i) {
                return Ok(match st.model.generators.get(&gd.generator) {
                    Some(x) => Dynamic::from_map(crate::generator::values_map(x, &gd.values)),
                    None => Dynamic::UNIT,
                });
            }
            if let Some((_, rg)) = st.model.instance_rigged(&g.path, i) {
                return Ok(match crate::rig::generator(&st.model, rg.master) {
                    Some(x) => Dynamic::from_map(crate::generator::values_map(&x, &rg.values)),
                    None => Dynamic::UNIT,
                });
            }
            Ok(st.model.instance_params(&g.path, i).map_or(Dynamic::UNIT, |p| Dynamic::from_map(params_map(p))))
        })
    });
    e.register_fn("set_params", |x: Dynamic, m: Map| -> R<()> {
        let g = one(&x)?;
        let (generated, rigged) = with(|st| {
            let i = inst_id(st, &g)?;
            Ok((st.model.instance_generated(&g.path, i).is_some().then_some(i), st.model.instance_rigged(&g.path, i).is_some().then_some(i)))
        })?;
        if let Some(i) = generated {
            let values = gen_values(&m)?;
            return with_model(|m| crate::generator::set_values(m, &g.path, i, &values, false));
        }
        if let Some(i) = rigged {
            let values = gen_values(&m)?;
            return with_model(|m| crate::rig::set_values(m, &g.path, i, &values, false));
        }
        edit(|st| {
            let i = inst_id(st, &g)?;
            let Some(mut p) = st.model.instance_params(&g.path, i).cloned() else {
                return err("set_params: not parametric (call make_parametric first)");
            };
            for (k, v) in &m {
                match k.as_str() {
                    "x" | "y" | "z" => {
                        let a = (k.as_bytes()[0] - b'x') as usize;
                        p.size[a] = if v.is_unit() { None } else { Some(num(v)?) };
                    }
                    "count" => p.count = num(v)?.round().max(1.0) as u32,
                    "axis" => {
                        p.axis = match v.to_string().as_str() {
                            "x" | "0" => 0,
                            "y" | "1" => 1,
                            "z" | "2" => 2,
                            a => return err(format!("set_params: axis {a} (use \"x\", \"y\" or \"z\")")),
                        }
                    }
                    "gap" => p.gap = num(v)?,
                    "stretch" => p.stretch = v.as_bool().map_err(|_| "set_params: stretch is true / false")?,
                    k => return err(format!("set_params: unknown key {k} (x, y, z, count, axis, gap, stretch)")),
                }
            }
            op(st.model.set_instance_params(&g.path, i, p))
        })
    });
}

/// A script map as generator values.
fn gen_values(m: &Map) -> R<varstotas_kernel::GenValues> {
    use varstotas_kernel::GenValue;
    m.iter()
        .map(|(k, v)| {
            let x = if let Ok(b) = v.as_bool() {
                GenValue::Bool(b)
            } else if let Ok(f) = v.as_float() {
                GenValue::Num(f)
            } else if let Ok(i) = v.as_int() {
                GenValue::Num(i as f64)
            } else if let Some(s) = v.clone().try_cast::<ImmutableString>() {
                GenValue::Text(s.to_string())
            } else {
                return err(format!("{k}: expected a number, bool or string, got {}", v.type_name()));
            };
            Ok((k.to_string(), x))
        })
        .collect()
}

/// Run `f` on the model taken out of the script state (generators run scripts of their own).
fn with_model<T>(f: impl FnOnce(&mut varstotas_kernel::Model) -> Result<T, String>) -> R<T> {
    let mut m = edit(|st| Ok(std::mem::take(&mut st.model)))?;
    let r = f(&mut m);
    with(|st| {
        st.model = m;
        Ok(())
    })?;
    r.map_err(|e| e.into())
}

fn register_generators(e: &mut Engine) {
    e.register_fn("generators", || -> R<Array> {
        with(|st| Ok(st.model.generators.keys().map(|k| Dynamic::from(k.clone())).collect()))
    });
    e.register_fn("add_generator", |src: &str| -> R<String> { with_model(|m| crate::generator::add(m, src)) });
    fn generate(name: &str, at: Dynamic, values: Map) -> R<Dynamic> {
        let at = pt(&at)?;
        let values = gen_values(&values)?;
        let (path, local) = with(|st| {
            let path = st.path.clone();
            Ok((path.clone(), world(st, &path).inverse() * DMat4::from_translation(at)))
        })?;
        let id = with_model(|m| crate::generator::place(m, &path, name, &values, local))?;
        Ok(dyn_ent(&path, EntityRef::Instance(id)))
    }
    e.register_fn("generate", |name: &str, at: Dynamic| generate(name, at, Map::new()));
    e.register_fn("generate", generate);
    e.register_fn("reset_params", |x: Dynamic| -> R<()> {
        let g = one(&x)?;
        let i = with(|st| inst_id(st, &g))?;
        with_model(|m| crate::generator::set_values(m, &g.path, i, &Default::default(), true))
    });
}

fn register_context(e: &mut Engine) {
    e.register_fn("enter", |x: Dynamic| -> R<()> {
        let g = one(&x)?;
        with(|st| {
            let i = inst_id(st, &g)?;
            st.path = g.path.clone();
            st.path.push(i);
            Ok(())
        })
    });
    e.register_fn("exit", || -> R<()> {
        with(|st| {
            st.path.pop();
            Ok(())
        })
    });
    fn list(which: u8, path: Option<ContextPath>) -> R<Array> {
        with(|st| {
            let path = path.unwrap_or_else(|| st.path.clone());
            let m = mesh(st, &path)?;
            let mut out = Array::new();
            if which == 0 || which == 3 {
                out.extend(m.faces.keys().map(|f| dyn_ent(&path, EntityRef::Face(f))));
            }
            if which == 1 || which == 3 {
                out.extend(m.edges.keys().map(|x| dyn_ent(&path, EntityRef::Edge(x))));
            }
            if which == 2 || which == 3 {
                out.extend(m.instances.keys().map(|x| dyn_ent(&path, EntityRef::Instance(x))));
            }
            Ok(out)
        })
    }
    e.register_fn("faces", || list(0, None));
    e.register_fn("edges", || list(1, None));
    e.register_fn("groups", || list(2, None));
    e.register_fn("all", || list(3, None));
    e.register_fn("entities", |x: Dynamic| -> R<Array> {
        let g = one(&x)?;
        let i = with(|st| inst_id(st, &g))?;
        let mut p = g.path.clone();
        p.push(i);
        list(3, Some(p))
    });
    // Groups / components by name: in the current context or inside `parent`; `deep` searches
    // nested ones too (nearer first). () when there is none.
    fn child(parent: &Dynamic, name: &str, deep: bool) -> R<Dynamic> {
        let path = inside(parent)?;
        with(|st| Ok(find_child(st, &path, name, deep).map_or(Dynamic::UNIT, Dynamic::from)))
    }
    e.register_fn("child", |name: ImmutableString| child(&Dynamic::UNIT, &name, false));
    e.register_fn("child", |name: ImmutableString, deep: bool| child(&Dynamic::UNIT, &name, deep));
    e.register_fn("child", |parent: Dynamic, name: ImmutableString| child(&parent, &name, false));
    e.register_fn("child", |parent: Dynamic, name: ImmutableString, deep: bool| child(&parent, &name, deep));
    fn children(parent: &Dynamic, deep: bool) -> R<Array> {
        let path = inside(parent)?;
        with(|st| {
            let mut out = Array::new();
            children_of(st, &path, deep, &mut out, 0)?;
            Ok(out)
        })
    }
    e.register_fn("children", || children(&Dynamic::UNIT, false));
    e.register_fn("children", |deep: bool| children(&Dynamic::UNIT, deep));
    e.register_fn("children", |parent: Dynamic| children(&parent, false));
    e.register_fn("children", |parent: Dynamic, deep: bool| children(&parent, deep));
    e.register_fn("selection", || -> R<Array> {
        with(|st| Ok(st.selection.iter().filter(|x| alive(st, x)).cloned().map(Dynamic::from).collect()))
    });
    e.register_fn("select", |x: Dynamic| -> R<()> {
        let es = if x.is_unit() { vec![] } else { ents(&x)? };
        with(|st| {
            st.selection = es;
            st.selection_set = true;
            Ok(())
        })
    });
    e.register_fn("bounds", |x: Dynamic| -> R<Dynamic> {
        let es = ents(&x)?;
        with(|st| Ok(bounds_map(ent_bounds(st, &es)?)))
    });
    e.register_fn("model_bounds", || -> R<Dynamic> { with(|st| Ok(bounds_map(st.model.bounds()))) });
    e.register_fn("length", |s: ImmutableString| -> R<f64> { num(&Dynamic::from(s)) });
    e.register_fn("units", || -> R<String> { with(|st| Ok(format!("{:?}", st.model.unit).to_lowercase())) });
    e.register_fn("all_faces", || -> R<Array> {
        fn walk(st: &State, path: &ContextPath, out: &mut Array, depth: usize) -> R<()> {
            let m = mesh(st, path)?;
            out.extend(m.faces.keys().map(|f| dyn_ent(path, EntityRef::Face(f))));
            if depth < 64 {
                for (k, i) in m.instances.iter() {
                    if !i.hidden {
                        let mut p = path.clone();
                        p.push(k);
                        walk(st, &p, out, depth + 1)?;
                    }
                }
            }
            Ok(())
        }
        with(|st| {
            let mut out = Array::new();
            walk(st, &Vec::new(), &mut out, 0)?;
            Ok(out)
        })
    });
    e.register_fn("lang", || varstotas_i18n::language());
    e.register_fn("platform", || varstotas_i18n::platform().to_string());
    e.register_fn("read_text", |path: ImmutableString| -> R<String> {
        std::fs::read_to_string(path.as_str()).or_else(|e| err(format!("{path}: {e}")))
    });
    e.register_fn("write_text", |path: ImmutableString, text: ImmutableString| -> R<()> {
        std::fs::write(path.as_str(), text.as_str()).or_else(|e| err(format!("{path}: {e}")))
    });
    e.register_fn("file_exists", |path: ImmutableString| std::path::Path::new(path.as_str()).exists());
    e.register_fn("list_files", |dir: ImmutableString, ext: ImmutableString| -> Array {
        list_files(std::path::Path::new(dir.as_str()), ext.trim_start_matches('.')).into_iter().map(Dynamic::from).collect()
    });
    e.register_fn("list_dirs", |dir: ImmutableString| -> Array {
        let mut out: Vec<String> = std::fs::read_dir(dir.as_str())
            .map(|rd| rd.flatten().map(|e| e.path()).filter(|p| p.is_dir()).map(|p| p.to_string_lossy().to_string()).collect())
            .unwrap_or_default();
        out.sort();
        out.into_iter().map(Dynamic::from).collect()
    });
    e.register_fn("parse_json", |text: ImmutableString| -> R<Dynamic> {
        let v: serde_json::Value = serde_json::from_str(text.as_str()).or_else(|e| err(format!("JSON: {e}")))?;
        crate::json::from_json(&v).or_else(err)
    });
    fn fetch_text(url: &str, headers: &Map) -> R<String> {
        let bytes = fetch(url, None, headers).or_else(err)?;
        Ok(String::from_utf8_lossy(&bytes).to_string())
    }
    fn fetch_file(url: &str, path: &str, headers: &Map) -> R<()> {
        if let Some(d) = std::path::Path::new(path).parent() {
            let _ = std::fs::create_dir_all(d);
        }
        fetch(url, Some(path), headers).map(|_| ()).or_else(err)
    }
    e.register_fn("fetch_text", |url: ImmutableString| fetch_text(&url, &Map::new()));
    e.register_fn("fetch_text", |url: ImmutableString, headers: Map| fetch_text(&url, &headers));
    e.register_fn("fetch_file", |url: ImmutableString, path: ImmutableString| fetch_file(&url, &path, &Map::new()));
    e.register_fn("fetch_file", |url: ImmutableString, path: ImmutableString, headers: Map| fetch_file(&url, &path, &headers));
    e.register_fn("url_encode", |s: ImmutableString| -> String {
        s.bytes().map(|b| if b.is_ascii_alphanumeric() || b"-_.~".contains(&b) { (b as char).to_string() } else { format!("%{b:02X}") }).collect()
    });
    e.register_fn("materials", || -> R<Array> {
        with(|st| Ok(st.model.materials.values().map(|m| Dynamic::from(m.name.clone())).collect()))
    });
}
