//! Model + selection + editing context → GPU-ready `SceneGeometry`.

use crate::document::guarded;
use varstotas_kernel::{ContextPath, EntityRef, InstanceId, MaterialId, Model, Tessellation, tessellate};
use varstotas_render::{FaceStyle, LineVertex, SceneGeometry, TriVertex};
use std::collections::HashMap;

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
}

/// Selection / dimming state of a primitive owned by `owner_path`.
fn classify(owner_path: &[InstanceId], own: EntityRef, p: &BuildParams) -> (bool, bool) {
    let active = p.active_path.as_slice();
    if !starts_with(owner_path, active) {
        return (false, !active.is_empty());
    }
    let selected = if owner_path.len() == active.len() {
        p.selection.contains(&own)
    } else {
        p.selection.contains(&EntityRef::Instance(owner_path[active.len()]))
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
    let mono = p.face_style != FaceStyle::Shaded;
    let mat_color = |id: Option<MaterialId>| -> Option<[f32; 4]> {
        if mono {
            return None;
        }
        id.and_then(|id| model.materials.get(id)).map(|m| m.color)
    };

    let mut max_r2: f64 = 0.0;
    for f in &tess.faces {
        let own = f.face.map(EntityRef::Face).unwrap_or(EntityRef::Instance(InstanceId::default()));
        let (selected, dimmed) = classify(&f.owner_path, own, p);
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
            max_r2 = max_r2.max(pos.length_squared());
            geo.tris.push(TriVertex { pos: pos.as_vec3().to_array(), normal: n, color: front });
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
        let (selected, dimmed) = classify(&e.owner_path, EntityRef::Edge(e.edge), p);
        let color = if selected {
            SELECTED
        } else if dimmed {
            DIM_EDGE
        } else {
            EDGE
        };
        let a = LineVertex { pos: e.a.as_vec3().to_array(), color };
        let b = LineVertex { pos: e.b.as_vec3().to_array(), color };
        max_r2 = max_r2.max(e.a.length_squared()).max(e.b.length_squared());
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
