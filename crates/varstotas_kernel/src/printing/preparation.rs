//! Print-part snapshots, rigid bed placement, capped triangle clipping and dowel sockets.
use super::{Bvh, PrintSelection};
use crate::{
    Aabb, Definition, Edge, EntityRef, Face, FaceId, Instance, InstanceId, Mesh, Model, OpError,
    OpResult, Plane, TexPlacement, Vertex, VertexId,
    glam::{DMat4, DQuat, DVec2, DVec3},
};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct BedProfile {
    pub name: String,
    pub width_mm: f64,
    pub depth_mm: f64,
    pub height_mm: f64,
    pub margin_mm: f64,
    pub gap_mm: f64,
    pub nozzle_mm: f64,
    pub rotate_parts: bool,
}
impl Default for BedProfile {
    fn default() -> Self {
        Self {
            name: "220 × 220 × 250".into(),
            width_mm: 220.,
            depth_mm: 220.,
            height_mm: 250.,
            margin_mm: 5.,
            gap_mm: 5.,
            nozzle_mm: 0.4,
            rotate_parts: true,
        }
    }
}
impl BedProfile {
    pub fn valid(&self) -> bool {
        [self.width_mm, self.depth_mm, self.height_mm, self.nozzle_mm]
            .iter()
            .all(|x| x.is_finite() && *x > 0.)
            && self.margin_mm.is_finite()
            && self.margin_mm >= 0.
            && self.gap_mm.is_finite()
            && self.gap_mm >= 0.
            && self.margin_mm * 2. < self.width_mm.min(self.depth_mm)
    }
    pub fn usable(&self) -> DVec3 {
        DVec3::new(
            self.width_mm - 2. * self.margin_mm,
            self.depth_mm - 2. * self.margin_mm,
            self.height_mm,
        )
    }
    pub fn fits(&self, b: Aabb) -> bool {
        let e = 1e-6;
        b.min.x >= self.margin_mm - e
            && b.min.y >= self.margin_mm - e
            && b.min.z >= -e
            && b.max.x <= self.width_mm - self.margin_mm + e
            && b.max.y <= self.depth_mm - self.margin_mm + e
            && b.max.z <= self.height_mm + e
    }
}
#[derive(Clone, Debug)]
pub struct PrintPartBounds {
    pub name: String,
    pub bounds: Aabb,
    pub fits: bool,
    pub on_bed: bool,
}
#[derive(Clone, Debug, Default)]
pub struct BedReport {
    pub parts: Vec<PrintPartBounds>,
    pub bounds_overlaps: usize,
}
#[derive(Clone, Copy, Debug)]
pub enum Placement {
    Drop,
    Center,
    Arrange,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct ConnectorOptions {
    pub enabled: bool,
    pub count: usize,
    pub diameter_mm: f64,
    pub length_mm: f64,
    pub clearance_mm: f64,
    pub wall_mm: f64,
    pub segments: u32,
}
impl Default for ConnectorOptions {
    fn default() -> Self {
        Self {
            enabled: false,
            count: 2,
            diameter_mm: 4.,
            length_mm: 12.,
            clearance_mm: 0.2,
            wall_mm: 1.,
            segments: 24,
        }
    }
}
#[derive(Clone, Debug)]
pub struct PlacementResult {
    pub path: Vec<InstanceId>,
    pub entities: Vec<EntityRef>,
    pub changed: bool,
}
#[derive(Clone, Debug, Default)]
pub struct SplitResult {
    pub parts: Vec<InstanceId>,
    pub dowels: usize,
}
fn bad(s: &str) -> OpError {
    OpError::Invalid(s.into())
}
fn definition(name: String, mesh: Mesh) -> Definition {
    Definition {
        name,
        mesh,
        is_group: true,
        guides: Default::default(),
        params: None,
        generated: None,
        source: None,
        rig: None,
        rigged: None,
        preview: None,
    }
}
fn insert_part(
    model: &mut Model,
    parent: &[InstanceId],
    name: String,
    mesh: Mesh,
    world: DMat4,
) -> OpResult<InstanceId> {
    let xf = model.transform_at(parent).inverse() * world;
    if !xf.is_finite() {
        return Err(bad(&varstotas_i18n::tr!("print-error-1")));
    }
    let d = model.defs.insert(definition(name.clone(), mesh));
    let m = model.mesh_at_mut(parent).ok_or(OpError::NotFound)?;
    Ok(m.instances.insert(Instance {
        def: d,
        transform: xf,
        name,
        material: None,
        tag: None,
        hidden: false,
        locked: false,
    }))
}
#[derive(Default)]
struct Surface {
    mesh: Mesh,
    points: super::Points,
    vertices: Vec<VertexId>,
    edges: HashSet<[VertexId; 2]>,
}
impl Surface {
    fn vertex(&mut self, p: DVec3) -> VertexId {
        let n = self.points.points.len();
        let i = self.points.add(0, p, 1e-7);
        if i == n {
            self.vertices
                .push(self.mesh.vertices.insert(Vertex { pos: p }));
        }
        self.vertices[i]
    }
    fn triangle(
        &mut self,
        p: [DVec3; 3],
        normals: [DVec3; 3],
        uv: [DVec2; 3],
        front: Option<crate::MaterialId>,
        back: Option<crate::MaterialId>,
    ) {
        let normal = (p[1] - p[0]).cross(p[2] - p[0]).normalize_or_zero();
        if normal == DVec3::ZERO {
            return;
        }
        let v = p.map(|p| self.vertex(p));
        if v[0] == v[1] || v[1] == v[2] || v[0] == v[2] {
            return;
        }
        for i in 0..3 {
            let (a, b) = (v[i], v[(i + 1) % 3]);
            let key = if a < b { [a, b] } else { [b, a] };
            if self.edges.insert(key) {
                self.mesh.edges.insert(Edge::new(a, b));
            }
        }
        let mut f = Face::template();
        f.outer = v.to_vec();
        f.normal = normal;
        f.front = front;
        f.back = back;
        f.normals = normals.iter().map(|n| n.normalize_or(normal)).collect();
        f.uv = TexPlacement::fit(&p, &uv, normal, [1., 1.]);
        self.mesh.faces.insert(f);
    }
    fn finish(mut self) -> Mesh {
        self.mesh.rebuild_caches();
        let soft: Vec<_> = self
            .mesh
            .edges
            .iter()
            .filter_map(|(e, edge)| {
                let fs = self.mesh.edge_faces(e);
                if fs.len() != 2 {
                    return None;
                }
                edge.v
                    .iter()
                    .all(|v| {
                        let at = |f: FaceId| {
                            let f = &self.mesh.faces[f];
                            f.outer
                                .iter()
                                .position(|x| x == v)
                                .and_then(|i| f.normals.get(i))
                                .copied()
                                .unwrap_or(f.normal)
                        };
                        at(fs[0]).dot(at(fs[1])) > 1. - 1e-4
                    })
                    .then_some(e)
            })
            .collect();
        for e in soft {
            self.mesh.edges[e].soft = true;
            self.mesh.edges[e].smooth = true;
        }
        self.mesh
    }
}
impl Model {
    /// Flatten only the checked scope into separate world-space parts. Preserve placement,
    /// inherited materials, corner normals and UVs; never normalize the bounding-box origin.
    pub fn print_export_model(&self, selection: Option<&PrintSelection>) -> OpResult<Model> {
        let mut out = Model::new();
        out.unit = crate::LengthUnit::Millimeter;
        out.metadata = self.metadata.clone();
        out.materials = self.materials.clone();
        let mut groups: HashMap<Vec<InstanceId>, Vec<crate::TessFace>> = HashMap::new();
        for f in crate::tessellate(self).faces {
            if f.face.is_some_and(|id| {
                selection.is_none_or(|s| s.includes(&f.owner_path, EntityRef::Face(id)))
            }) {
                groups.entry(f.owner_path.clone()).or_default().push(f);
            }
        }
        let mut groups: Vec<_> = groups.into_iter().collect();
        groups.sort_by(|a, b| a.0.cmp(&b.0));
        for (path, faces) in groups {
            let mut surface = Surface::default();
            for f in faces {
                for (i, p) in f.positions.as_chunks::<3>().0.iter().enumerate() {
                    let normal = |k| f.normals.get(i * 3 + k).copied().unwrap_or(f.normal);
                    let uv = |k| f.uv.get(i * 3 + k).copied().unwrap_or_default();
                    surface.triangle(
                        *p,
                        [normal(0), normal(1), normal(2)],
                        [uv(0), uv(1), uv(2)],
                        f.front.or(f.inherited),
                        f.back.or(f.inherited),
                    );
                }
            }
            if surface.mesh.faces.is_empty() {
                continue;
            }
            let name = path
                .split_last()
                .and_then(|(i, p)| self.mesh_at(p)?.instances.get(*i))
                .map(|i| {
                    if i.name.is_empty() {
                        self.defs[i.def].name.clone()
                    } else {
                        i.name.clone()
                    }
                })
                .unwrap_or_else(|| self.defs[self.root].name.clone());
            let i = insert_part(&mut out, &[], name, surface.finish(), DMat4::IDENTITY)?;
            let d = out.root_mesh().instances[i].def;
            out.defs[d].source = (0..=path.len())
                .rev()
                .find_map(|n| self.defs.get(self.def_at(&path[..n])?)?.source.clone());
        }
        if out.root_mesh().instances.is_empty() {
            return Err(bad(&varstotas_i18n::tr!("print-error-2")));
        }
        out.rebuild_caches();
        crate::io::validate_model(&out).map_err(|e| bad(&e.to_string()))?;
        Ok(out)
    }
    pub fn print_bed_report(
        &self,
        selection: Option<&PrintSelection>,
        bed: &BedProfile,
    ) -> OpResult<BedReport> {
        if !bed.valid() {
            return Err(bad(&varstotas_i18n::tr!("print-error-3")));
        }
        let snapshot = self.print_export_model(selection)?;
        let mut report = BedReport::default();
        for inst in snapshot.root_mesh().instances.values() {
            if let Some(b) = snapshot.def_bounds(inst.def) {
                report.parts.push(PrintPartBounds {
                    name: inst.name.clone(),
                    bounds: b,
                    fits: bed.fits(b),
                    on_bed: b.min.z.abs() < 1e-5,
                });
            }
        }
        for i in 0..report.parts.len() {
            for j in i + 1..report.parts.len() {
                let (a, b) = (report.parts[i].bounds, report.parts[j].bounds);
                if a.min.cmplt(b.max - DVec3::splat(1e-6)).all()
                    && b.min.cmplt(a.max - DVec3::splat(1e-6)).all()
                {
                    report.bounds_overlaps += 1;
                }
            }
        }
        Ok(report)
    }
    fn editable_print_tree(&self, path: &[InstanceId], visible: bool) -> OpResult<()> {
        if !self.print_context_editable(path) {
            return Err(bad(&varstotas_i18n::tr!("print-error-4")));
        }
        let d = self.def_at(path).ok_or(OpError::NotFound)?;
        let m = &self.defs[d].mesh;
        for (f, face) in &m.faces {
            if self.entity_locked_in(d, EntityRef::Face(f)) {
                return Err(bad(&varstotas_i18n::tr!("print-error-5")));
            }
            if visible && (face.hidden || !self.tag_visible(d, face.tag)) {
                return Err(bad(&varstotas_i18n::tr!("print-error-6")));
            }
        }
        for (e, _) in &m.edges {
            if self.entity_locked_in(d, EntityRef::Edge(e)) {
                return Err(bad(&varstotas_i18n::tr!("print-error-7")));
            }
        }
        for (i, inst) in &m.instances {
            if self.entity_locked_in(d, EntityRef::Instance(i)) {
                return Err(bad(&varstotas_i18n::tr!("print-error-8")));
            }
            if visible && (inst.hidden || !self.tag_visible(d, inst.tag)) {
                return Err(bad(&varstotas_i18n::tr!("print-error-6")));
            }
            let mut p = path.to_vec();
            p.push(i);
            self.editable_print_tree(&p, visible)?;
        }
        Ok(())
    }
    fn print_targets(
        &mut self,
        selection: Option<&PrintSelection>,
    ) -> OpResult<(Vec<InstanceId>, Vec<InstanceId>)> {
        let path = selection.map_or(Vec::new(), |s| s.path.clone());
        let d = self.def_at(&path).ok_or(OpError::NotFound)?;
        let mesh = &self.defs[d].mesh;
        let mut instances = Vec::new();
        let mut loose = Vec::new();
        if let Some(s) = selection {
            if s.entities.is_empty() {
                return Err(bad(&varstotas_i18n::tr!("print-error-9")));
            }
            for e in &s.entities {
                match e {
                    EntityRef::Instance(i) if mesh.instances.contains_key(*i) => instances.push(*i),
                    _ => {
                        return Err(bad(&varstotas_i18n::tr!("print-error-10")));
                    }
                }
            }
        } else {
            instances.extend(
                mesh.instances
                    .iter()
                    .filter(|(_, i)| !i.hidden && self.tag_visible(d, i.tag))
                    .map(|(i, _)| i),
            );
            loose.extend(
                mesh.faces
                    .iter()
                    .filter(|(_, f)| !f.hidden && self.tag_visible(d, f.tag))
                    .map(|(f, _)| EntityRef::Face(f)),
            );
        }
        for i in &instances {
            let mut p = path.clone();
            p.push(*i);
            self.editable_print_tree(&p, false)?;
        }
        if !loose.is_empty() {
            if mesh
                .faces
                .values()
                .any(|f| f.hidden || !self.tag_visible(d, f.tag))
            {
                return Err(bad(&varstotas_i18n::tr!("print-error-11")));
            }
            self.editable_print_tree(&path, false)?;
            let i = self.make_group(&path, &loose)?;
            instances.push(i);
        }
        instances.sort();
        instances.dedup();
        if instances.is_empty() {
            return Err(bad(&varstotas_i18n::tr!("print-error-12")));
        }
        Ok((path, instances))
    }
    fn world_part_bounds(&self, path: &[InstanceId], i: InstanceId) -> OpResult<Aabb> {
        let s = PrintSelection {
            path: path.to_vec(),
            entities: vec![EntityRef::Instance(i)],
        };
        self.print_check(Some(&s), 1e-7)
            .bounds
            .ok_or(bad(&varstotas_i18n::tr!("print-error-13")))
    }
    fn move_part(&mut self, path: &[InstanceId], i: InstanceId, delta: DMat4) -> OpResult<()> {
        let parent = self.transform_at(path);
        let m = parent.inverse() * delta * parent;
        if !m.is_finite() {
            return Err(bad(&varstotas_i18n::tr!("print-error-14")));
        }
        let mesh = self.mesh_at_mut(path).ok_or(OpError::NotFound)?;
        let inst = mesh.instances.get_mut(i).ok_or(OpError::NotFound)?;
        inst.transform = m * inst.transform;
        Ok(())
    }
    pub fn print_place(
        &mut self,
        selection: Option<&PrintSelection>,
        bed: &BedProfile,
        mode: Placement,
    ) -> OpResult<PlacementResult> {
        if !bed.valid() {
            return Err(bad(&varstotas_i18n::tr!("print-error-15")));
        }
        let mut draft = self.clone();
        let (path, parts) = draft.print_targets(selection)?;
        let mut entries: Vec<_> = parts
            .iter()
            .map(|i| Ok((*i, draft.world_part_bounds(&path, *i)?)))
            .collect::<OpResult<_>>()?;
        match mode {
            Placement::Center => {
                let b = Aabb::from_points(entries.iter().flat_map(|(_, b)| b.corners()))
                    .ok_or(bad(&varstotas_i18n::tr!("print-error-16")))?;
                let delta = DVec3::new(
                    bed.width_mm / 2. - b.center().x,
                    bed.depth_mm / 2. - b.center().y,
                    0.,
                );
                for (i, _) in entries {
                    draft.move_part(&path, i, DMat4::from_translation(delta))?;
                }
            }
            Placement::Drop => {
                for (i, b) in entries {
                    draft.move_part(
                        &path,
                        i,
                        DMat4::from_translation(DVec3::new(0., 0., -b.min.z)),
                    )?;
                }
            }
            Placement::Arrange => {
                entries.sort_by(|a, b| {
                    let s = |b: Aabb| (b.max.x - b.min.x) * (b.max.y - b.min.y);
                    s(b.1).total_cmp(&s(a.1)).then(a.0.cmp(&b.0))
                });
                let (mut x, mut y, mut row) = (bed.margin_mm, bed.margin_mm, 0.);
                for (i, original) in entries {
                    let mut candidates = Vec::new();
                    for rotate in [false, true] {
                        if rotate && !bed.rotate_parts {
                            continue;
                        }
                        let r = if rotate {
                            DMat4::from_rotation_z(std::f64::consts::FRAC_PI_2)
                        } else {
                            DMat4::IDENTITY
                        };
                        let b = original.transformed(&r);
                        let size = b.max - b.min;
                        if size.z > bed.height_mm + 1e-6 {
                            continue;
                        }
                        for new_row in [false, true] {
                            let (tx, ty) = if new_row {
                                (bed.margin_mm, y + row + bed.gap_mm)
                            } else {
                                (x, y)
                            };
                            if tx + size.x <= bed.width_mm - bed.margin_mm + 1e-6
                                && ty + size.y <= bed.depth_mm - bed.margin_mm + 1e-6
                            {
                                candidates.push((ty + size.y, tx, ty, size, rotate, new_row, r, b));
                            }
                        }
                    }
                    candidates.sort_by(|a, b| {
                        a.0.total_cmp(&b.0)
                            .then(a.1.total_cmp(&b.1))
                            .then(a.4.cmp(&b.4))
                    });
                    let Some((_, tx, ty, size, _, new_row, r, b)) = candidates.into_iter().next()
                    else {
                        return Err(bad(&varstotas_i18n::tr!("print-error-17")));
                    };
                    draft.move_part(
                        &path,
                        i,
                        DMat4::from_translation(DVec3::new(tx, ty, 0.) - b.min) * r,
                    )?;
                    if new_row {
                        row = 0.;
                    }
                    x = tx + size.x + bed.gap_mm;
                    y = ty;
                    row = row.max(size.y);
                }
            }
        }
        self.validate_locked_changes(&draft)
            .map_err(OpError::Invalid)?;
        crate::io::validate_model(&draft).map_err(|e| bad(&e.to_string()))?;
        let changed = self.defs.len() != draft.defs.len()
            || parts.iter().any(|i| {
                self.mesh_at(&path)
                    .and_then(|m| m.instances.get(*i))
                    .is_none_or(|old| {
                        !old.transform.abs_diff_eq(
                            draft.mesh_at(&path).unwrap().instances[*i].transform,
                            1e-10,
                        )
                    })
            });
        if changed {
            *self = draft;
        }
        Ok(PlacementResult {
            path,
            entities: parts.into_iter().map(EntityRef::Instance).collect(),
            changed,
        })
    }
    /// A picked face's entire enclosing part is rigidly rotated so the outward face points down.
    pub fn print_lay_face(
        &mut self,
        path: &[InstanceId],
        face: FaceId,
        bed: &BedProfile,
    ) -> OpResult<PlacementResult> {
        if !bed.valid() {
            return Err(bad(&varstotas_i18n::tr!("print-error-15")));
        }
        let mut draft = self.clone();
        let mesh = draft.mesh_at(path).ok_or(OpError::NotFound)?;
        let f = mesh.faces.get(face).ok_or(OpError::NotFound)?;
        let xf = draft.transform_at(path);
        let normal = xf
            .inverse()
            .transpose()
            .transform_vector3(f.normal)
            .normalize_or_zero();
        if normal == DVec3::ZERO {
            return Err(bad(&varstotas_i18n::tr!("print-error-18")));
        }
        let center = xf.transform_point3(mesh.vertices[f.outer[0]].pos);
        let (parent, i) = if let Some((&i, parent)) = path.split_last() {
            (parent.to_vec(), i)
        } else {
            let mut seen = HashSet::new();
            let mut pending = vec![face];
            while let Some(f) = pending.pop() {
                if !seen.insert(f) {
                    continue;
                }
                for e in draft.root_mesh().face_edges(f) {
                    pending.extend(draft.root_mesh().edge_faces(e));
                }
            }
            let es: Vec<_> = seen.into_iter().map(EntityRef::Face).collect();
            let i = draft.make_group(&[], &es)?;
            (Vec::new(), i)
        };
        let mut p = parent.clone();
        p.push(i);
        draft.editable_print_tree(&p, false)?;
        let rotation = DMat4::from_quat(DQuat::from_rotation_arc(normal, -DVec3::Z));
        draft.move_part(
            &parent,
            i,
            DMat4::from_translation(center) * rotation * DMat4::from_translation(-center),
        )?;
        let b = draft.world_part_bounds(&parent, i)?;
        if center.z - b.min.z > 1e-4 {
            return Err(bad(&varstotas_i18n::tr!("print-error-19")));
        }
        draft.move_part(
            &parent,
            i,
            DMat4::from_translation(DVec3::new(
                bed.width_mm / 2. - b.center().x,
                bed.depth_mm / 2. - b.center().y,
                -b.min.z,
            )),
        )?;
        self.validate_locked_changes(&draft)
            .map_err(OpError::Invalid)?;
        let changed = self.defs.len() != draft.defs.len()
            || self
                .mesh_at(&parent)
                .and_then(|m| m.instances.get(i))
                .is_none_or(|old| {
                    !old.transform.abs_diff_eq(
                        draft.mesh_at(&parent).unwrap().instances[i].transform,
                        1e-10,
                    )
                });
        if changed {
            *self = draft;
        }
        Ok(PlacementResult {
            path: parent,
            entities: vec![EntityRef::Instance(i)],
            changed,
        })
    }
}

#[derive(Clone, Copy)]
struct Corner {
    p: DVec3,
    n: DVec3,
    uv: DVec2,
}
fn clip_polygon(p: &[Corner], plane: Plane, positive: bool) -> Vec<Corner> {
    let mut out = Vec::new();
    let sign = if positive { 1. } else { -1. };
    for i in 0..p.len() {
        let (a, b) = (p[i], p[(i + 1) % p.len()]);
        let (da, db) = (plane.distance(a.p) * sign, plane.distance(b.p) * sign);
        if da >= -1e-8 {
            out.push(a);
        }
        if (da > 0. && db < 0.) || (da < 0. && db > 0.) {
            let t = da / (da - db);
            out.push(Corner {
                p: a.p.lerp(b.p, t),
                n: a.n.lerp(b.n, t).normalize_or(a.n),
                uv: a.uv.lerp(b.uv, t),
            });
        }
    }
    out
}
fn cap(
    mesh: &mut Mesh,
    plane: Plane,
    normal: DVec3,
    material: Option<crate::MaterialId>,
) -> OpResult<()> {
    mesh.rebuild_caches();
    let mut next = HashMap::new();
    for (f, face) in &mesh.faces {
        for lp in face.loops() {
            for k in 0..lp.len() {
                let (a, b) = (lp[k], lp[(k + 1) % lp.len()]);
                let Some(e) = mesh.edge_between(a, b) else {
                    continue;
                };
                if mesh.edge_faces(e) == vec![f]
                    && plane.distance(mesh.vertices[a].pos).abs() < 1e-5
                    && plane.distance(mesh.vertices[b].pos).abs() < 1e-5
                    && next.insert(b, a).is_some()
                {
                    return Err(bad(&varstotas_i18n::tr!("print-error-20")));
                }
            }
        }
    }
    let pr = crate::geom::Projector::new(plane.origin, normal);
    let mut loops = Vec::new();
    while let Some(&start) = next.keys().min() {
        let mut lp = vec![start];
        let mut v = start;
        loop {
            let w = next
                .remove(&v)
                .ok_or(bad(&varstotas_i18n::tr!("print-error-21")))?;
            if w == start {
                break;
            }
            lp.push(w);
            v = w;
            if lp.len() > mesh.vertices.len() {
                return Err(bad(&varstotas_i18n::tr!("print-error-22")));
            }
        }
        let poly: Vec<_> = lp.iter().map(|v| pr.to2(mesh.vertices[*v].pos)).collect();
        let area = crate::geom::signed_area(&poly);
        if area.abs() > 1e-12 {
            loops.push((lp, poly, area));
        }
    }
    let outers: Vec<_> = loops
        .iter()
        .enumerate()
        .filter(|(_, l)| l.2 > 0.)
        .map(|(i, _)| i)
        .collect();
    let mut holes: HashMap<usize, Vec<Vec<VertexId>>> = HashMap::new();
    for (lp, p, area) in &loops {
        if *area >= 0. {
            continue;
        }
        let outer = outers
            .iter()
            .filter(|i| {
                crate::geom::point_in_polygon(p[0], &loops[**i].1, 1e-7)
                    != crate::geom::PointLoc::Outside
            })
            .min_by(|a, b| loops[**a].2.total_cmp(&loops[**b].2))
            .ok_or(bad(&varstotas_i18n::tr!("print-error-23")))?;
        holes.entry(*outer).or_default().push(lp.clone());
    }
    for i in outers {
        let mut template = Face::template();
        template.front = material;
        template.back = material;
        mesh.f_new(
            loops[i].0.clone(),
            holes.remove(&i).unwrap_or_default(),
            &template,
        );
    }
    mesh.rebuild_caches();
    if !mesh.is_solid() {
        return Err(bad(&varstotas_i18n::tr!("print-error-24")));
    }
    Ok(())
}
fn clip_model(source: &Model, plane: Plane) -> OpResult<(Mesh, Mesh)> {
    let mut positive = Surface::default();
    let mut negative = Surface::default();
    let mut material = None;
    for f in crate::tessellate(source).faces {
        material = material.or(f.front.or(f.inherited));
        for (i, p) in f.positions.as_chunks::<3>().0.iter().enumerate() {
            let corners = [0, 1, 2].map(|k| Corner {
                p: p[k],
                n: f.normals.get(i * 3 + k).copied().unwrap_or(f.normal),
                uv: f.uv.get(i * 3 + k).copied().unwrap_or_default(),
            });
            let coplanar = p.iter().all(|p| plane.distance(*p).abs() < 1e-8);
            for (is_positive, surface) in [(true, &mut positive), (false, &mut negative)] {
                if coplanar && (f.normal.dot(plane.normal) < 0.) != is_positive {
                    continue;
                }
                let polygon = clip_polygon(&corners, plane, is_positive);
                for k in 1..polygon.len().saturating_sub(1) {
                    let c = [polygon[0], polygon[k], polygon[k + 1]];
                    surface.triangle(
                        c.map(|v| v.p),
                        c.map(|v| v.n),
                        c.map(|v| v.uv),
                        f.front.or(f.inherited),
                        f.back.or(f.inherited),
                    );
                }
            }
        }
    }
    let (mut a, mut b) = (positive.finish(), negative.finish());
    if a.faces.is_empty() || b.faces.is_empty() {
        return Err(bad(&varstotas_i18n::tr!("print-error-25")));
    }
    cap(&mut a, plane, -plane.normal, material)?;
    cap(&mut b, plane, plane.normal, material)?;
    Ok((a, b))
}
impl Model {
    pub fn print_split(
        &mut self,
        selection: &PrintSelection,
        plane: Plane,
        connectors: &ConnectorOptions,
    ) -> OpResult<SplitResult> {
        if !plane.origin.is_finite()
            || !plane.normal.is_finite()
            || plane.normal.length_squared() < 1e-12
        {
            return Err(bad(&varstotas_i18n::tr!("print-error-26")));
        }
        let mut draft = self.clone();
        let (path, targets) = draft.print_targets(Some(selection))?;
        if targets.len() != 1 {
            return Err(bad(&varstotas_i18n::tr!("print-error-27")));
        }
        let i = targets[0];
        let mut full = path.clone();
        full.push(i);
        draft.editable_print_tree(&full, true)?;
        let s = PrintSelection {
            path: path.clone(),
            entities: vec![EntityRef::Instance(i)],
        };
        let source = draft.print_export_model(Some(&s))?;
        let report = source.print_check(None, 1e-7);
        if report.has_surface_errors() || report.shells.iter().any(|s| !s.closed) {
            return Err(bad(&varstotas_i18n::tr!("print-error-28")));
        }
        let analysis = source.print_analyze(
            None,
            1e-7,
            &super::analysis::AnalysisOptions {
                max_samples: 1,
                ..Default::default()
            },
        );
        if !analysis.intersections_complete || analysis.intersection_pairs > 0 {
            return Err(bad(&varstotas_i18n::tr!("print-error-29")));
        }
        let plane = Plane::new(plane.origin, plane.normal);
        let (a, b) = clip_model(&source, plane)?;
        let mut work = Model::new();
        work.materials = source.materials.clone();
        let a = insert_part(
            &mut work,
            &[],
            format!("{} +", varstotas_i18n::tr!("print-part-name")),
            a,
            DMat4::IDENTITY,
        )?;
        let b = insert_part(
            &mut work,
            &[],
            format!("{} −", varstotas_i18n::tr!("print-part-name")),
            b,
            DMat4::IDENTITY,
        )?;
        let mut generated = SplitResult {
            parts: vec![a, b],
            dowels: 0,
        };
        if connectors.enabled {
            work.add_print_dowels(&[], &mut generated, plane, connectors)?;
        }
        let credit = source.attributions().into_iter().next();
        let mut result = SplitResult {
            parts: Vec::new(),
            dowels: generated.dowels,
        };
        let body_count = generated.parts.len() - generated.dowels;
        for (index, i) in generated.parts.into_iter().enumerate() {
            let inst = &work.root_mesh().instances[i];
            let mesh = work.defs[inst.def].mesh.clone();
            let out = insert_part(&mut draft, &path, inst.name.clone(), mesh, inst.transform)?;
            if index < body_count {
                let d = draft.mesh_at(&path).unwrap().instances[out].def;
                draft.defs[d].source = credit.clone();
            }
            result.parts.push(out);
        }
        draft.mesh_at_mut(&path).unwrap().instances.remove(i);
        self.validate_locked_changes(&draft)
            .map_err(OpError::Invalid)?;
        crate::io::validate_model(&draft).map_err(|e| bad(&e.to_string()))?;
        *self = draft;
        Ok(result)
    }
    pub fn print_split_to_bed(
        &mut self,
        selection: &PrintSelection,
        bed: &BedProfile,
    ) -> OpResult<SplitResult> {
        if !bed.valid() {
            return Err(bad(&varstotas_i18n::tr!("print-error-15")));
        }
        let mut draft = self.clone();
        let mut parts = match selection.entities.as_slice() {
            [EntityRef::Instance(i)] => vec![*i],
            _ => return Err(bad(&varstotas_i18n::tr!("print-error-30"))),
        };
        let path = selection.path.clone();
        let size = bed.usable();
        let bounds = draft.world_part_bounds(&path, parts[0])?;
        let extent = bounds.max - bounds.min;
        let needed = (0..3)
            .map(|axis| (extent[axis] / size[axis]).ceil().max(1.))
            .product::<f64>();
        if needed > 256. {
            return Err(bad(&varstotas_i18n::tr!("print-error-31")));
        }
        let mut cuts = 0;
        for axis in 0..3 {
            let mut next = Vec::new();
            for part in parts {
                let mut pending = vec![part];
                while let Some(i) = pending.pop() {
                    let b = draft.world_part_bounds(&path, i)?;
                    if b.max[axis] - b.min[axis] <= size[axis] + 1e-6 {
                        next.push(i);
                        continue;
                    }
                    if cuts >= 255 {
                        return Err(bad(&varstotas_i18n::tr!("print-error-31")));
                    }
                    let mut origin = b.center();
                    origin[axis] = b.min[axis] + size[axis];
                    let normal = match axis {
                        0 => DVec3::X,
                        1 => DVec3::Y,
                        _ => DVec3::Z,
                    };
                    let s = PrintSelection {
                        path: path.clone(),
                        entities: vec![EntityRef::Instance(i)],
                    };
                    let split =
                        draft.print_split(&s, Plane::new(origin, normal), &Default::default())?;
                    pending.extend(split.parts);
                    cuts += 1;
                }
            }
            parts = next;
        }
        self.validate_locked_changes(&draft)
            .map_err(OpError::Invalid)?;
        *self = draft;
        Ok(SplitResult { parts, dowels: 0 })
    }
    fn add_print_dowels(
        &mut self,
        path: &[InstanceId],
        result: &mut SplitResult,
        plane: Plane,
        o: &ConnectorOptions,
    ) -> OpResult<()> {
        if !(1..=16).contains(&o.count)
            || ![o.diameter_mm, o.length_mm, o.clearance_mm, o.wall_mm]
                .iter()
                .all(|v| v.is_finite())
            || o.diameter_mm <= 0.
            || o.length_mm <= 0.
            || o.clearance_mm < 0.
            || o.wall_mm < 0.
            || !(8..=64).contains(&o.segments)
        {
            return Err(bad(&varstotas_i18n::tr!("print-error-32")));
        }
        let pr = crate::geom::Projector::new(plane.origin, plane.normal);
        let mut points = Vec::new();
        for &i in &result.parts {
            let s = PrintSelection {
                path: path.to_vec(),
                entities: vec![EntityRef::Instance(i)],
            };
            for f in crate::tessellate(self).faces {
                if f.face
                    .is_some_and(|id| s.includes(&f.owner_path, EntityRef::Face(id)))
                {
                    points.extend(
                        f.positions
                            .into_iter()
                            .filter(|p| plane.distance(*p).abs() < 1e-5)
                            .map(|p| pr.to2(p)),
                    );
                }
            }
        }
        let lo = points
            .iter()
            .copied()
            .fold(DVec2::splat(f64::INFINITY), DVec2::min);
        let hi = points
            .iter()
            .copied()
            .fold(DVec2::splat(f64::NEG_INFINITY), DVec2::max);
        if !lo.is_finite() {
            return Err(bad(&varstotas_i18n::tr!("print-error-33")));
        }
        let axis = usize::from(hi.x - lo.x < hi.y - lo.y);
        let mut centers = Vec::new();
        let radius = o.diameter_mm / 2. + o.clearance_mm;
        let depth = o.length_mm / 2. + o.clearance_mm;
        for k in 0..o.count {
            let mut p = (lo + hi) * 0.5;
            p[axis] = lo[axis] + (hi[axis] - lo[axis]) * (k + 1) as f64 / (o.count + 1) as f64;
            let center = pr.to3(p);
            if centers
                .iter()
                .any(|p: &DVec3| p.distance(center) < 2. * (radius + o.wall_mm))
            {
                return Err(bad(&varstotas_i18n::tr!("print-error-34")));
            }
            centers.push(center);
        }
        // Validate clearance and depth on both halves before invoking booleans. A failed operation
        // remains in the draft: no partial sockets or removed source objects can leak into the scene.
        for &i in &result.parts {
            let s = PrintSelection {
                path: path.to_vec(),
                entities: vec![EntityRef::Instance(i)],
            };
            let snapshot = self.print_export_model(Some(&s))?;
            let triangles: Vec<_> = crate::tessellate(&snapshot)
                .faces
                .into_iter()
                .flat_map(|f| f.positions.as_chunks::<3>().0.to_vec())
                .collect();
            let bvh = Bvh::new(&triangles);
            let sign = if i == result.parts[0] { 1. } else { -1. };
            for &center in &centers {
                for z in [1e-4, depth] {
                    for k in 0..16 {
                        let angle = k as f64 * std::f64::consts::TAU / 16.;
                        let p = center
                            + plane.normal * (sign * z)
                            + pr.u * (angle.cos() * (radius + o.wall_mm))
                            + pr.v * (angle.sin() * (radius + o.wall_mm));
                        let mut hits =
                            bvh.hits(&triangles, p, DVec3::new(0.927, 0.371, 0.239).normalize());
                        hits.sort_by(|a, b| a.0.total_cmp(&b.0));
                        hits.dedup_by(|a, b| (a.0 - b.0).abs() < 1e-6);
                        if hits.len().is_multiple_of(2) {
                            return Err(bad(&varstotas_i18n::tr!("print-error-35")));
                        }
                    }
                }
            }
        }
        for center in centers {
            let rotation = DMat4::from_quat(DQuat::from_rotation_arc(DVec3::Z, plane.normal));
            let transform = DMat4::from_translation(center - plane.normal * depth) * rotation;
            let cutter = crate::primitives::Primitive::Cylinder {
                r: radius,
                h: depth * 2.,
                segs: o.segments,
            }
            .mesh()
            .ok_or(bad(&varstotas_i18n::tr!("print-error-36")))?;
            for side in 0..2 {
                let c = insert_part(
                    self,
                    path,
                    varstotas_i18n::tr!("print-socket-name"),
                    cutter.clone(),
                    transform,
                )?;
                let parts =
                    self.solid_op_in(path, crate::SolidOp::Subtract, c, result.parts[side])?;
                if parts.len() != 1 {
                    return Err(bad(&varstotas_i18n::tr!("print-error-37")));
                }
                result.parts[side] = parts[0];
            }
            let pin = crate::primitives::Primitive::Cylinder {
                r: o.diameter_mm / 2.,
                h: o.length_mm,
                segs: o.segments,
            }
            .mesh()
            .ok_or(bad(&varstotas_i18n::tr!("print-error-38")))?;
            let t = DMat4::from_translation(center - plane.normal * (o.length_mm / 2.)) * rotation;
            let i = insert_part(self, path, varstotas_i18n::tr!("print-dowel-name"), pin, t)?;
            result.parts.push(i);
            result.dowels += 1;
        }
        Ok(())
    }
}
