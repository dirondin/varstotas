//! Model → GPU-ready `SceneGeometry` (base colours + highlight ids), and selection / editing
//! context → per-primitive highlight states for `Renderer::update_highlight`.
//!
//! Geometry is rebuilt only when the model or face style changes; a selection or context
//! change only recomputes [`highlight`] from the cached `Tessellation`.

use crate::document::guarded;
use varstotas_kernel::glam::DVec3;
use varstotas_kernel::{ContextPath, EntityRef, InstanceId, MaterialId, Model, Tessellation, tessellate};
use varstotas_render::{FaceStyle, Highlight, LineVertex, SceneGeometry, TriVertex};
use std::collections::{HashMap, HashSet};

pub const FRONT_DEFAULT: [f32; 4] = [1.0, 1.0, 1.0, 1.0];
pub const BACK_DEFAULT: [f32; 4] = [0xA4 as f32 / 255.0, 0xB0 as f32 / 255.0, 0xC8 as f32 / 255.0, 1.0];
pub const EDGE: [f32; 4] = [0.0, 0.0, 0.0, 1.0];

fn starts_with(path: &[InstanceId], prefix: &[InstanceId]) -> bool {
    path.len() >= prefix.len() && path[..prefix.len()] == *prefix
}

/// Inputs of the geometry build (model-version dependent only).
pub struct BuildParams {
    pub face_style: FaceStyle,
    /// Origin used by the previous build; kept while it stays close to the model centre so
    /// the GPU-side origin is stable (see [`choose_origin`]).
    pub prev_origin: Option<DVec3>,
}

/// Render origin for geometry with bounds `min..max`: the bbox centre, unless `prev` is still
/// within a quarter of the model size (min 1 m) of it — then `prev` is kept to avoid needless
/// origin changes. Vertices are stored as f32 relative to it, so precision depends only on the
/// model's own size, not on its distance from the world origin.
pub fn choose_origin(min: DVec3, max: DVec3, prev: Option<DVec3>) -> DVec3 {
    if !(min.is_finite() && max.is_finite()) || min.cmpgt(max).any() {
        return prev.unwrap_or(DVec3::ZERO);
    }
    let centre = (min + max) * 0.5;
    let tol = ((max - min).length() * 0.25).max(1000.0);
    match prev {
        Some(p) if p.is_finite() && p.distance(centre) <= tol => p,
        _ => centre,
    }
}

/// Inputs of the highlight pass (selection / editing context).
pub struct HighlightParams<'a> {
    pub selection: &'a [EntityRef],
    pub active_path: &'a ContextPath,
}

/// Selection / dimming state of a primitive owned by `owner_path`.
/// `selected_set` is the selection as a set (it can hold tens of thousands of refs).
fn classify(owner_path: &[InstanceId], own: EntityRef, active: &[InstanceId], selected_set: &HashSet<EntityRef>) -> (bool, bool) {
    if !starts_with(owner_path, active) {
        return (false, !active.is_empty());
    }
    let selected = if owner_path.len() == active.len() {
        selected_set.contains(&own)
    } else {
        selected_set.contains(&EntityRef::Instance(owner_path[active.len()]))
    };
    (selected, false)
}

/// Tessellate the model; never panics (an empty tessellation is returned on failure).
pub fn tessellate_model(model: &Model) -> Tessellation {
    guarded(|| tessellate(model)).unwrap_or_else(|e| {
        log::debug!("tessellate unavailable: {e}");
        Tessellation::default()
    })
}

/// Build render geometry (tessellates). Used by tests; the viewport keeps the tessellation.
#[allow(dead_code)]
pub fn build(model: &Model, params: &BuildParams) -> SceneGeometry {
    build_from(model, &tessellate_model(model), params)
}

/// Per-primitive highlight states, indexed like the ids written by [`build_from`]
/// (face id = index in `tess.faces`, edge id = index in `tess.edges`).
pub fn highlight(tess: &Tessellation, p: &HighlightParams) -> (Vec<Highlight>, Vec<Highlight>) {
    let sel: HashSet<EntityRef> = p.selection.iter().copied().collect();
    let active = p.active_path.as_slice();
    let state = |(selected, dimmed): (bool, bool)| {
        if selected {
            Highlight::Selected
        } else if dimmed {
            Highlight::Dimmed
        } else {
            Highlight::None
        }
    };
    let faces = tess
        .faces
        .iter()
        .map(|f| {
            let own = f.face.map(EntityRef::Face).unwrap_or(EntityRef::Instance(InstanceId::default()));
            state(classify(&f.owner_path, own, active, &sel))
        })
        .collect();
    let edges = tess
        .edges
        .iter()
        .map(|e| state(classify(&e.owner_path, EntityRef::Edge(e.edge), active, &sel)))
        .collect();
    (faces, edges)
}

/// Geometry with base (material) colours and highlight ids; selection is applied separately.
pub fn build_from(model: &Model, tess: &Tessellation, p: &BuildParams) -> SceneGeometry {
    let mut geo = SceneGeometry::default();
    let mono = p.face_style != FaceStyle::Shaded;
    let mat_color = |id: Option<MaterialId>| -> Option<[f32; 4]> {
        if mono {
            return None;
        }
        id.and_then(|id| model.materials.get(id)).map(|m| m.color)
    };

    // Bounds (f64) → origin; positions are made relative to it in f64 before the f32 cast.
    let (mut lo, mut hi) = (DVec3::splat(f64::INFINITY), DVec3::splat(f64::NEG_INFINITY));
    for f in &tess.faces {
        for pos in &f.positions {
            lo = lo.min(*pos);
            hi = hi.max(*pos);
        }
    }
    for e in tess.edges.iter().filter(|e| !e.soft) {
        lo = lo.min(e.a).min(e.b);
        hi = hi.max(e.a).max(e.b);
    }
    let origin = choose_origin(lo, hi, p.prev_origin);
    geo.origin = origin.to_array();
    let rel = |q: DVec3| (q - origin).as_vec3().to_array();
    let mut max_r2: f64 = 0.0;
    for (fi, f) in tess.faces.iter().enumerate() {
        let front = mat_color(f.front.or(f.inherited)).unwrap_or(FRONT_DEFAULT);
        let back = mat_color(f.back.or(f.inherited)).unwrap_or(BACK_DEFAULT);
        geo.face_ids.extend(std::iter::repeat_n(fi as u32, f.positions.len() / 3));
        let n = f.normal.as_vec3().to_array();
        for pos in &f.positions {
            max_r2 = max_r2.max((*pos - origin).length_squared());
            geo.tris.push(TriVertex { pos: rel(*pos), normal: n, color: front });
            geo.back_colors.push(back);
        }
    }

    // Border edges (fewer than two faces) are drawn as profiles.
    let mut face_count: HashMap<(ContextPath, varstotas_kernel::EdgeId), usize> = HashMap::new();
    let mut mesh_cache: HashMap<ContextPath, bool> = HashMap::new();
    for (ei, e) in tess.edges.iter().enumerate() {
        if e.soft {
            continue;
        }
        let a = LineVertex { pos: rel(e.a), color: EDGE };
        let b = LineVertex { pos: rel(e.b), color: EDGE };
        geo.edge_ids.push(ei as u32);
        max_r2 = max_r2.max((e.a - origin).length_squared()).max((e.b - origin).length_squared());
        geo.edges.push(a);
        geo.edges.push(b);

        let has_mesh = *mesh_cache
            .entry(e.owner_path.clone())
            .or_insert_with(|| model.mesh_at(&e.owner_path).is_some());
        if has_mesh {
            let n = *face_count.entry((e.owner_path.clone(), e.edge)).or_insert_with(|| {
                guarded(|| model.mesh_at(&e.owner_path).map(|m| m.edge_faces(e.edge).len()).unwrap_or(2))
                    .unwrap_or(2)
            });
            if n < 2 {
                geo.profile_edges.push(a);
                geo.profile_edges.push(b);
                geo.profile_ids.push(ei as u32);
            }
        }
    }
    geo.radius = (max_r2.sqrt() as f32).max(10_000.0);
    geo
}

#[cfg(test)]
mod tests {
    use super::*;
    use varstotas_kernel::glam::DVec3;

    #[test]
    fn origin_is_bbox_centre_and_stable() {
        let (a, b) = (DVec3::new(5.0e6, 5.0e6, 0.0), DVec3::new(5.003e6, 5.002e6, 2000.0));
        let o = choose_origin(a, b, None);
        assert_eq!(o, (a + b) * 0.5);
        // Small growth keeps the previous origin; moving far away re-centres.
        assert_eq!(choose_origin(a, b + DVec3::X * 500.0, Some(o)), o);
        let far = DVec3::new(1.0e6, 0.0, 0.0);
        assert_eq!(choose_origin(far, far + DVec3::ONE * 1000.0, Some(o)), far + DVec3::ONE * 500.0);
        // Empty scene keeps the previous origin (or the world origin).
        let empty = (DVec3::splat(f64::INFINITY), DVec3::splat(f64::NEG_INFINITY));
        assert_eq!(choose_origin(empty.0, empty.1, Some(o)), o);
        assert_eq!(choose_origin(empty.0, empty.1, None), DVec3::ZERO);
    }

    #[test]
    fn vertices_are_relative_to_origin_far_from_world_origin() {
        let mut m = Model::new();
        let base = DVec3::new(5.0e6, 5.0e6, 0.0);
        let f = m.root_mesh_mut().add_face(&[
            base,
            base + DVec3::new(1000.0, 0.0, 0.0),
            base + DVec3::new(1000.0, 1000.0, 0.0),
            base + DVec3::new(0.0, 1000.0, 0.0),
        ]);
        let f = f.expect("face");
        let _ = m.root_mesh_mut().push_pull(f, 1000.0, false);
        let geo = build(&m, &BuildParams { face_style: FaceStyle::Shaded, prev_origin: None });
        let o = DVec3::from_array(geo.origin);
        let bb = m.bounds().expect("bounds");
        assert!((o - (bb.min + bb.max) * 0.5).length() < 1e-6, "{o}");
        assert!((bb.max - bb.min - DVec3::splat(1000.0)).length() < 1e-6);
        // Relative coordinates are small and exact to well below a millimetre.
        for v in &geo.tris {
            for c in v.pos {
                assert!(c.abs() <= 500.0 + 1e-3);
                assert!((c.abs() - 500.0).abs() < 1e-3, "{c}");
            }
        }
        assert!(!geo.edges.is_empty());
    }

    fn unit_box(m: &mut Model, x: f64) {
        let mesh = m.root_mesh_mut();
        let pts = [
            DVec3::new(x, 0.0, 0.0),
            DVec3::new(x + 1000.0, 0.0, 0.0),
            DVec3::new(x + 1000.0, 1000.0, 0.0),
            DVec3::new(x, 1000.0, 0.0),
        ];
        let f = mesh.add_face(&pts).expect("face");
        let _ = mesh.push_pull(f, -1000.0, false);
    }

    #[test]
    fn ids_and_highlight_states() {
        let mut m = Model::new();
        unit_box(&mut m, 0.0);
        let loose: Vec<EntityRef> = m.root_mesh().faces.keys().map(EntityRef::Face).collect();
        unit_box(&mut m, 3000.0);
        let all: Vec<EntityRef> = m
            .root_mesh()
            .faces
            .keys()
            .map(EntityRef::Face)
            .chain(m.root_mesh().edges.keys().map(EntityRef::Edge))
            .collect();
        let grouped: Vec<EntityRef> = all.iter().copied().filter(|e| !loose.contains(e)).collect();
        let grouped: Vec<EntityRef> = grouped
            .into_iter()
            .filter(|e| match e {
                EntityRef::Edge(id) => m.root_mesh().edge_faces(*id).iter().all(|f| !loose.contains(&EntityRef::Face(*f))),
                _ => true,
            })
            .collect();
        let g = m.make_group(&[], &grouped).expect("group");

        let tess = tessellate_model(&m);
        let geo = build_from(&m, &tess, &BuildParams { face_style: FaceStyle::Shaded, prev_origin: None });
        assert_eq!(geo.face_ids.len(), geo.tris.len() / 3);
        assert_eq!(geo.edge_ids.len(), geo.edges.len() / 2);
        assert!(geo.face_ids.iter().all(|&i| (i as usize) < tess.faces.len()));
        // Base colours only: no vertex is tinted.
        assert!(geo.tris.iter().all(|v| v.color == FRONT_DEFAULT));
        assert!(geo.edges.iter().all(|v| v.color == EDGE));

        // Root context, group selected: its faces are Selected, loose ones None.
        let root: ContextPath = Vec::new();
        let sel = [EntityRef::Instance(g)];
        let (f, e) = highlight(&tess, &HighlightParams { selection: &sel, active_path: &root });
        assert_eq!(f.len(), tess.faces.len());
        assert_eq!(e.len(), tess.edges.len());
        for (st, tf) in f.iter().zip(&tess.faces) {
            let want = if tf.owner_path.is_empty() { Highlight::None } else { Highlight::Selected };
            assert_eq!(*st, want);
        }
        // Inside the group with nothing selected: everything outside is dimmed.
        let inside = vec![g];
        let (f, e) = highlight(&tess, &HighlightParams { selection: &[], active_path: &inside });
        for (st, tf) in f.iter().zip(&tess.faces) {
            let want = if tf.owner_path.is_empty() { Highlight::Dimmed } else { Highlight::None };
            assert_eq!(*st, want);
        }
        assert!(e.contains(&Highlight::Dimmed) && e.contains(&Highlight::None));
    }
}
