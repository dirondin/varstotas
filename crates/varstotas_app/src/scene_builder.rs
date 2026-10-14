//! Model + selection + editing context → GPU-ready `SceneGeometry`.

use crate::document::guarded;
use varstotas_kernel::glam::DVec3;
use varstotas_kernel::{ContextPath, EntityRef, InstanceId, MaterialId, Model, Tessellation, tessellate};
use varstotas_render::{FaceStyle, LineVertex, SceneGeometry, TriVertex};
use std::collections::{HashMap, HashSet};

pub const FRONT_DEFAULT: [f32; 4] = [1.0, 1.0, 1.0, 1.0];
pub const BACK_DEFAULT: [f32; 4] = [0xA4 as f32 / 255.0, 0xB0 as f32 / 255.0, 0xC8 as f32 / 255.0, 1.0];
pub const SELECTED: [f32; 4] = [0.0, 0x50 as f32 / 255.0, 1.0, 1.0];
pub const EDGE: [f32; 4] = [0.0, 0.0, 0.0, 1.0];
const DIM_EDGE: [f32; 4] = [0.62, 0.62, 0.64, 1.0];
const DIM_TARGET: [f32; 4] = [0.86, 0.86, 0.87, 1.0];

fn mix(a: [f32; 4], b: [f32; 4], t: f32) -> [f32; 4] {
    [
        a[0] + (b[0] - a[0]) * t,
        a[1] + (b[1] - a[1]) * t,
        a[2] + (b[2] - a[2]) * t,
        a[3] + (b[3] - a[3]) * t,
    ]
}

fn starts_with(path: &[InstanceId], prefix: &[InstanceId]) -> bool {
    path.len() >= prefix.len() && path[..prefix.len()] == *prefix
}

pub struct BuildParams<'a> {
    pub selection: &'a [EntityRef],
    pub active_path: &'a ContextPath,
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

/// Selection / dimming state of a primitive owned by `owner_path`.
/// `selected_set` is `p.selection` as a set (selection can hold tens of thousands of refs).
fn classify(owner_path: &[InstanceId], own: EntityRef, p: &BuildParams, selected_set: &HashSet<EntityRef>) -> (bool, bool) {
    let active = p.active_path.as_slice();
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

/// Build render geometry. Never panics: if tessellation is unavailable an empty scene is returned.
pub fn build(model: &Model, params: &BuildParams) -> SceneGeometry {
    match guarded(|| tessellate(model)) {
        Ok(t) => build_from(model, &t, params),
        Err(e) => {
            log::debug!("tessellate unavailable: {e}");
            SceneGeometry { radius: 10_000.0, ..Default::default() }
        }
    }
}

fn build_from(model: &Model, tess: &Tessellation, p: &BuildParams) -> SceneGeometry {
    let mut geo = SceneGeometry::default();
    let sel: HashSet<EntityRef> = p.selection.iter().copied().collect();
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
    for f in &tess.faces {
        let own = f.face.map(EntityRef::Face).unwrap_or(EntityRef::Instance(InstanceId::default()));
        let (selected, dimmed) = classify(&f.owner_path, own, p, &sel);
        let mut front = mat_color(f.front.or(f.inherited)).unwrap_or(FRONT_DEFAULT);
        let mut back = mat_color(f.back.or(f.inherited)).unwrap_or(BACK_DEFAULT);
        if selected {
            front = mix(front, SELECTED, 0.45);
            back = mix(back, SELECTED, 0.45);
        } else if dimmed {
            front = mix(front, DIM_TARGET, 0.6);
            back = mix(back, DIM_TARGET, 0.6);
        }
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
    for e in &tess.edges {
        if e.soft {
            continue;
        }
        let (selected, dimmed) = classify(&e.owner_path, EntityRef::Edge(e.edge), p, &sel);
        let color = if selected {
            SELECTED
        } else if dimmed {
            DIM_EDGE
        } else {
            EDGE
        };
        let a = LineVertex { pos: rel(e.a), color };
        let b = LineVertex { pos: rel(e.b), color };
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
        let geo = build(&m, &BuildParams { selection: &[], active_path: &Vec::new(), face_style: FaceStyle::Shaded, prev_origin: None });
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
}
