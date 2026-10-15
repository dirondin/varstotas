//! 3D Text: closed outlines (glyph contours, flattened) → a group of faces, optionally extruded.
//!
//! The outlines come from the app (it owns the font). They lie in the local XY plane; nesting
//! decides what is filled (even-odd): a region inside an odd number of outlines is solid.

use crate::geom::{self, PointLoc};
use crate::mesh::Mesh;
use crate::model::{Definition, Model};
use crate::tessellate::triangulate_face;
use crate::{EntityRef, Instance, InstanceId, OpError, OpResult};
use glam::{DMat4, DVec2, DVec3};

/// Side edges folding by less than this become soft and smooth.
const SOFT_SIDE_DEG: f64 = 35.0;

/// How the outlines become geometry.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Text3dOptions {
    /// Faces between the outlines (otherwise only edges).
    pub filled: bool,
    /// Extrusion along +Z (local) when filled; `None` = flat.
    pub extrude: Option<f64>,
}

/// Build the geometry of `outlines` in a new mesh (local coordinates, z = 0).
pub fn text_mesh(outlines: &[Vec<DVec2>], opts: Text3dOptions) -> Mesh {
    let mut mesh = Mesh::new();
    let loops: Vec<Vec<DVec2>> = outlines.iter().filter(|l| l.len() >= 3).cloned().collect();
    for l in &loops {
        let mut pts: Vec<DVec3> = l.iter().map(|p| p.extend(0.0)).collect();
        if !opts.filled {
            mesh.add_polyline(&pts, true, false);
            continue;
        }
        // Counter-clockwise: the faces look up (+Z).
        if geom::signed_area(l) < 0.0 {
            pts.reverse();
        }
        mesh.add_face_exact(&pts);
    }
    if !opts.filled {
        // Closed polylines found faces on their own; outlines only.
        let faces: Vec<EntityRef> = mesh.faces.keys().map(EntityRef::Face).collect();
        mesh.erase(&faces);
        return mesh;
    }
    // Faces inside an even number of outlines are counters (holes): erase them.
    let inside_count = |p: DVec2| loops.iter().filter(|l| geom::point_in_polygon(p, l, 1e-9) == PointLoc::Inside).count();
    let holes: Vec<EntityRef> = mesh
        .faces
        .keys()
        .filter(|f| {
            let tris = triangulate_face(&mesh, *f);
            let Some(t) = tris.iter().max_by(|a, b| tri_area(a).total_cmp(&tri_area(b))) else { return false };
            let c = (t[0] + t[1] + t[2]) / 3.0;
            inside_count(c.truncate()) % 2 == 0
        })
        .map(EntityRef::Face)
        .collect();
    mesh.erase(&holes);
    if let Some(d) = opts.extrude.filter(|d| d.abs() > 1e-6) {
        let faces: Vec<_> = mesh.faces.keys().collect();
        for f in faces {
            // Faces look up; a free-standing face becomes the base of the prism.
            let _ = mesh.push_pull(f, d, false);
        }
        // Curved sides look smooth (softens them too); corners stay hard.
        let edges: Vec<_> = mesh.edges.keys().collect();
        mesh.soften_by_angle(&edges, SOFT_SIDE_DEG, true, false);
    }
    mesh
}

fn tri_area(t: &[DVec3; 3]) -> f64 {
    (t[1] - t[0]).cross(t[2] - t[0]).length()
}

impl Model {
    /// Add 3D text as a new group in the context `path`, placed by the world transform `xf`
    /// (local XY plane of the outlines → world). Returns the new instance.
    pub fn add_text3d_in(
        &mut self,
        path: &[InstanceId],
        outlines: &[Vec<DVec2>],
        opts: Text3dOptions,
        xf: &DMat4,
    ) -> OpResult<InstanceId> {
        let pd = self.def_at(path).ok_or(OpError::NotFound)?;
        let mesh = text_mesh(outlines, opts);
        if mesh.edges.is_empty() {
            return Err(OpError::Invalid("no outlines".into()));
        }
        let def = self.defs.insert(Definition { name: "Group".into(), mesh, is_group: true, guides: Default::default(), params: None });
        let inst = Instance {
            def,
            transform: self.transform_at(path).inverse() * *xf,
            name: String::new(),
            material: None,
            tag: None,
            hidden: false,
            locked: false,
        };
        Ok(self.defs[pd].mesh.instances.insert(inst))
    }
}
