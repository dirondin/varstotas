//! Basic print-surface inspection and conservative, undo-friendly mesh repair.
//! Coordinates and tolerances are in world millimetres. This does not certify wall thickness
//! or detect self-intersections; callers must not label a clean basic report print-ready.
use crate::glam::{DMat4, DVec3};
pub mod analysis;
pub mod preparation;

use crate::{Aabb, ContextPath, EntityRef, FaceId, InstanceId, Model, OpResult, VertexId};
use std::collections::{HashMap, HashSet};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PrintSelection {
    pub path: ContextPath,
    pub entities: Vec<EntityRef>,
}
impl PrintSelection {
    fn includes(&self, path: &[InstanceId], entity: EntityRef) -> bool {
        path.strip_prefix(self.path.as_slice()).is_some_and(|tail| {
            if tail.is_empty() {
                self.entities.contains(&entity)
            } else {
                self.entities.contains(&EntityRef::Instance(tail[0]))
            }
        })
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PrintIssueKind {
    OpenEdges,
    NonManifoldEdges,
    InconsistentWinding,
    DegenerateTriangles,
    DuplicateTriangles,
    InvertedShells,
    LooseEdges,
}
impl PrintIssueKind {
    pub const ALL: [Self; 7] = [
        Self::OpenEdges,
        Self::NonManifoldEdges,
        Self::InconsistentWinding,
        Self::DegenerateTriangles,
        Self::DuplicateTriangles,
        Self::InvertedShells,
        Self::LooseEdges,
    ];
}
#[derive(Clone, Debug)]
pub struct PrintIssue {
    pub kind: PrintIssueKind,
    pub count: usize,
    pub segments: Vec<[DVec3; 2]>,
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct PrintFace {
    pub path: ContextPath,
    pub face: FaceId,
}
#[derive(Clone, Debug)]
pub struct PrintShell {
    pub faces: Vec<PrintFace>,
    pub closed: bool,
    pub consistent: bool,
    pub volume_mm3: f64,
    pub inverted: bool,
}
#[derive(Clone, Debug)]
pub struct PrintReport {
    pub faces: Vec<PrintFace>,
    pub triangles: usize,
    pub bounds: Option<Aabb>,
    pub area_mm2: f64,
    pub shells: Vec<PrintShell>,
    pub issues: Vec<PrintIssue>,
    pub tolerance_mm: f64,
}
impl PrintReport {
    pub fn count(&self, kind: PrintIssueKind) -> usize {
        self.issues
            .iter()
            .find(|i| i.kind == kind)
            .map_or(0, |i| i.count)
    }
    pub fn has_surface_errors(&self) -> bool {
        self.issues
            .iter()
            .any(|i| i.kind != PrintIssueKind::LooseEdges && i.count > 0)
    }
    fn issue(
        &mut self,
        kind: PrintIssueKind,
        count: usize,
        segments: impl IntoIterator<Item = [DVec3; 2]>,
    ) {
        let i = self.issues.iter_mut().find(|i| i.kind == kind).unwrap();
        i.count += count;
        i.segments.extend(segments);
    }
}
/// Spatially match seams without the bucket-rounding errors of quantized position keys.
#[derive(Default)]
struct Points {
    points: Vec<DVec3>,
    cells: HashMap<(usize, [i64; 3]), Vec<usize>>,
}
impl Points {
    fn add(&mut self, context: usize, p: DVec3, tol: f64) -> usize {
        let cell = p.to_array().map(|x| (x / tol).floor() as i64);
        for x in -1i64..=1 {
            for y in -1i64..=1 {
                for z in -1i64..=1 {
                    let key = [
                        cell[0].saturating_add(x),
                        cell[1].saturating_add(y),
                        cell[2].saturating_add(z),
                    ];
                    for &i in self.cells.get(&(context, key)).into_iter().flatten() {
                        if self.points[i].distance_squared(p) <= tol * tol {
                            return i;
                        }
                    }
                }
            }
        }
        let i = self.points.len();
        self.points.push(p);
        self.cells.entry((context, cell)).or_default().push(i);
        i
    }
}
fn tolerance(t: f64) -> f64 {
    if t.is_finite() {
        t.clamp(1e-9, 1.0)
    } else {
        1e-5
    }
}
fn edge(a: usize, b: usize) -> (usize, usize) {
    if a < b { (a, b) } else { (b, a) }
}
fn find(parent: &mut [usize], i: usize) -> usize {
    if parent[i] != i {
        parent[i] = find(parent, parent[i]);
    }
    parent[i]
}
fn join(parent: &mut [usize], a: usize, b: usize) {
    let (a, b) = (find(parent, a), find(parent, b));
    if a != b {
        parent[b] = a;
    }
}
#[derive(Clone)]
struct Triangle {
    v: [usize; 3],
    owner: PrintFace,
}

impl Model {
    /// Inspect the visible scene, or the selected entities and their visible descendants.
    /// Read-only: no cache, topology, selection or document state is changed.
    pub fn print_check(&self, selection: Option<&PrintSelection>, tol: f64) -> PrintReport {
        let tol = tolerance(tol);
        let mut report = PrintReport {
            faces: Vec::new(),
            triangles: 0,
            bounds: None,
            area_mm2: 0.,
            shells: Vec::new(),
            issues: PrintIssueKind::ALL
                .into_iter()
                .map(|kind| PrintIssue {
                    kind,
                    count: 0,
                    segments: Vec::new(),
                })
                .collect(),
            tolerance_mm: tol,
        };
        let tess = crate::tessellate(self);
        let mut points = Points::default();
        let mut contexts = HashMap::new();
        let mut tris = Vec::new();
        let mut duplicates = HashMap::new();
        for face in tess.faces {
            let Some(fid) = face.face else { continue };
            if selection.is_some_and(|s| !s.includes(&face.owner_path, EntityRef::Face(fid))) {
                continue;
            }
            let owner = PrintFace {
                path: face.owner_path.clone(),
                face: fid,
            };
            report.faces.push(owner.clone());
            let n = contexts.len();
            let context = *contexts.entry(face.owner_path).or_insert(n);
            if face.positions.is_empty() {
                report.issue(PrintIssueKind::DegenerateTriangles, 1, []);
                continue;
            }
            for p in face.positions.as_chunks::<3>().0 {
                report.triangles += 1;
                let ids = [
                    points.add(context, p[0], tol),
                    points.add(context, p[1], tol),
                    points.add(context, p[2], tol),
                ];
                let area = (p[1] - p[0]).cross(p[2] - p[0]).length() * 0.5;
                report.area_mm2 += area;
                let segs = [[p[0], p[1]], [p[1], p[2]], [p[2], p[0]]];
                if ids[0] == ids[1] || ids[1] == ids[2] || ids[0] == ids[2] || area <= tol * tol {
                    report.issue(PrintIssueKind::DegenerateTriangles, 1, segs);
                    continue;
                }
                // Duplicate surfaces are defects even when they come from separate instances.
                let mut key = p
                    .iter()
                    .map(|p| p.to_array().map(|x| (x / tol).round() as i64))
                    .collect::<Vec<_>>();
                key.sort();
                if duplicates.insert(key, ()).is_some() {
                    report.issue(PrintIssueKind::DuplicateTriangles, 1, segs);
                }
                tris.push(Triangle {
                    v: ids,
                    owner: owner.clone(),
                });
            }
        }
        report.bounds = Aabb::from_points(points.points.iter().copied());
        for e in tess.edges {
            if selection.is_some_and(|s| !s.includes(&e.owner_path, EntityRef::Edge(e.edge))) {
                continue;
            }
            if self
                .mesh_at(&e.owner_path)
                .is_some_and(|m| m.edge_faces(e.edge).is_empty())
            {
                report.issue(PrintIssueKind::LooseEdges, 1, [[e.a, e.b]]);
            }
        }
        let mut edges: HashMap<_, Vec<(usize, bool)>> = HashMap::new();
        for (i, t) in tris.iter().enumerate() {
            for k in 0..3 {
                let (a, b) = (t.v[k], t.v[(k + 1) % 3]);
                edges.entry(edge(a, b)).or_default().push((i, a < b));
            }
        }
        let mut parent: Vec<_> = (0..tris.len()).collect();
        for (&(a, b), uses) in &edges {
            for &(i, _) in &uses[1..] {
                join(&mut parent, uses[0].0, i);
            }
            let kind = match uses.len() {
                1 => Some(PrintIssueKind::OpenEdges),
                2 if uses[0].1 == uses[1].1 => Some(PrintIssueKind::InconsistentWinding),
                n if n > 2 => Some(PrintIssueKind::NonManifoldEdges),
                _ => None,
            };
            if let Some(kind) = kind {
                report.issue(kind, 1, [[points.points[a], points.points[b]]]);
            }
        }
        let mut groups: HashMap<usize, Vec<usize>> = HashMap::new();
        for i in 0..tris.len() {
            groups.entry(find(&mut parent, i)).or_default().push(i);
        }
        let mut groups: Vec<_> = groups.into_values().collect();
        groups.sort_by_key(|g| g[0]);
        let mut tri_shell = vec![0; tris.len()];
        for (s, g) in groups.iter().enumerate() {
            for &i in g {
                tri_shell[i] = s;
            }
            let origin = points.points[tris[g[0]].v[0]];
            let closed = g.iter().all(|i| {
                (0..3).all(|k| {
                    let v = tris[*i].v;
                    edges[&edge(v[k], v[(k + 1) % 3])].len() == 2
                })
            });
            let consistent = g.iter().all(|i| {
                (0..3).all(|k| {
                    let v = tris[*i].v;
                    let u = &edges[&edge(v[k], v[(k + 1) % 3])];
                    u.len() != 2 || u[0].1 != u[1].1
                })
            });
            let volume_mm3 = g
                .iter()
                .map(|i| {
                    let p = tris[*i].v.map(|v| points.points[v] - origin);
                    p[0].dot(p[1].cross(p[2])) / 6.
                })
                .sum();
            let mut faces: Vec<_> = g
                .iter()
                .map(|i| tris[*i].owner.clone())
                .collect::<HashSet<_>>()
                .into_iter()
                .collect();
            faces.sort_by(|a, b| a.path.cmp(&b.path).then(a.face.cmp(&b.face)));
            report.shells.push(PrintShell {
                faces,
                closed,
                consistent,
                volume_mm3,
                inverted: false,
            });
        }
        // A cavity shell must face inward. Classify nesting against all other closed shells,
        // including shells stored in separate groups, rather than flipping every negative volume.
        let positions: Vec<_> = tris.iter().map(|t| t.v.map(|v| points.points[v])).collect();
        let bvh = Bvh::new(&positions);
        let shell_bounds: Vec<_> = groups
            .iter()
            .map(|g| Aabb::from_points(g.iter().flat_map(|i| positions[*i])).unwrap())
            .collect();
        for (s, g) in groups.iter().enumerate() {
            if !report.shells[s].closed || !report.shells[s].consistent {
                continue;
            }
            let p = positions[g[0]].iter().sum::<DVec3>() / 3.;
            let mut hits = bvh.hits(&positions, p, DVec3::new(0.927, 0.371, 0.239).normalize());
            hits.retain(|(_, t)| {
                let other = tri_shell[*t];
                let (a, b) = (shell_bounds[s], shell_bounds[other]);
                other != s
                    && report.shells[other].closed
                    && b.min.cmplt(a.min - DVec3::splat(tol)).all()
                    && b.max.cmpgt(a.max + DVec3::splat(tol)).all()
            });
            hits.sort_by(|a, b| {
                tri_shell[a.1]
                    .cmp(&tri_shell[b.1])
                    .then(a.0.total_cmp(&b.0))
            });
            hits.dedup_by(|a, b| tri_shell[a.1] == tri_shell[b.1] && (a.0 - b.0).abs() <= tol);
            let cavity = hits.len() % 2 == 1;
            let shell = &mut report.shells[s];
            shell.inverted = (shell.volume_mm3 < 0.) != cavity;
        }
        let inverted = report.shells.iter().filter(|s| s.inverted).count();
        if inverted > 0 {
            let segs = edges
                .iter()
                .filter(|(_, uses)| report.shells[tri_shell[uses[0].0]].inverted)
                .map(|(&(a, b), _)| [points.points[a], points.points[b]])
                .collect::<Vec<_>>();
            report.issue(PrintIssueKind::InvertedShells, inverted, segs);
        }
        report
    }
}

struct Node {
    bounds: Aabb,
    range: std::ops::Range<usize>,
    children: Option<(usize, usize)>,
}
struct Bvh {
    nodes: Vec<Node>,
    order: Vec<usize>,
}
impl Bvh {
    fn new(tris: &[[DVec3; 3]]) -> Self {
        let mut b = Self {
            nodes: Vec::new(),
            order: (0..tris.len()).collect(),
        };
        if !tris.is_empty() {
            b.build(tris, 0..tris.len());
        }
        b
    }
    fn build(&mut self, tris: &[[DVec3; 3]], range: std::ops::Range<usize>) -> usize {
        let bounds =
            Aabb::from_points(self.order[range.clone()].iter().flat_map(|i| tris[*i])).unwrap();
        let id = self.nodes.len();
        self.nodes.push(Node {
            bounds,
            range: range.clone(),
            children: None,
        });
        if range.len() > 8 {
            let size = bounds.max - bounds.min;
            let axis = if size.x >= size.y && size.x >= size.z {
                0
            } else if size.y >= size.z {
                1
            } else {
                2
            };
            let middle = range.start + range.len() / 2;
            self.order[range.clone()].select_nth_unstable_by(range.len() / 2, |a, b| {
                let center = |i: usize| tris[i].iter().map(|p| p[axis]).sum::<f64>();
                center(*a).total_cmp(&center(*b))
            });
            let left = self.build(tris, range.start..middle);
            let right = self.build(tris, middle..range.end);
            self.nodes[id].children = Some((left, right));
        }
        id
    }
    fn hits(&self, tris: &[[DVec3; 3]], p: DVec3, d: DVec3) -> Vec<(f64, usize)> {
        let mut hits = Vec::new();
        if self.nodes.is_empty() {
            return hits;
        }
        let mut stack = vec![0];
        let inv = d.recip();
        while let Some(i) = stack.pop() {
            let node = &self.nodes[i];
            let mut near = 0.0f64;
            let mut far = f64::INFINITY;
            let mut miss = false;
            for axis in 0..3 {
                if d[axis].abs() < 1e-15 {
                    if p[axis] < node.bounds.min[axis] || p[axis] > node.bounds.max[axis] {
                        miss = true;
                        break;
                    }
                } else {
                    let a = (node.bounds.min[axis] - p[axis]) * inv[axis];
                    let b = (node.bounds.max[axis] - p[axis]) * inv[axis];
                    near = near.max(a.min(b));
                    far = far.min(a.max(b));
                }
            }
            if miss || near > far {
                continue;
            }
            if let Some((a, b)) = node.children {
                stack.extend([a, b]);
                continue;
            }
            for &j in &self.order[node.range.clone()] {
                if let Some(t) = ray_triangle(p, d, tris[j]) {
                    hits.push((t, j));
                }
            }
        }
        hits
    }
}
fn ray_triangle(p: DVec3, d: DVec3, t: [DVec3; 3]) -> Option<f64> {
    let (a, b) = (t[1] - t[0], t[2] - t[0]);
    let q = d.cross(b);
    let det = a.dot(q);
    if det.abs() <= 1e-12 * a.length() * b.length() {
        return None;
    }
    let s = p - t[0];
    let u = s.dot(q) / det;
    if !(0.0..=1.0).contains(&u) {
        return None;
    }
    let q = s.cross(a);
    let v = d.dot(q) / det;
    if v < 0. || u + v > 1. {
        return None;
    }
    let t = b.dot(q) / det;
    (t > 1e-8).then_some(t)
}

#[derive(Clone, Debug)]
pub struct PrintRepairOptions {
    pub tolerance_mm: f64,
    pub weld_vertices: bool,
    pub remove_degenerate: bool,
    pub remove_duplicates: bool,
    pub orient_faces: bool,
    pub orient_shells: bool,
    pub fill_holes: bool,
    pub max_hole_mm: f64,
}
impl Default for PrintRepairOptions {
    fn default() -> Self {
        Self {
            tolerance_mm: 1e-5,
            weld_vertices: true,
            remove_degenerate: true,
            remove_duplicates: true,
            orient_faces: true,
            orient_shells: true,
            fill_holes: false,
            max_hole_mm: 10.,
        }
    }
}
#[derive(Clone, Debug, Default)]
pub struct PrintRepairSummary {
    pub welded_vertices: usize,
    pub removed_faces: usize,
    pub reversed_faces: usize,
    pub filled_holes: usize,
    pub skipped_objects: usize,
}
impl PrintRepairSummary {
    pub fn changed(&self) -> bool {
        self.welded_vertices + self.removed_faces + self.reversed_faces + self.filled_holes > 0
    }
    fn add(&mut self, r: &Self) {
        self.welded_vertices += r.welded_vertices;
        self.removed_faces += r.removed_faces;
        self.reversed_faces += r.reversed_faces;
        self.filled_holes += r.filled_holes;
        self.skipped_objects += r.skipped_objects;
    }
}
impl Model {
    /// Repair whole visible objects transactionally. Selected instances become unique only
    /// when their mesh changes; other copies, locks, hidden geometry and procedural objects
    /// are preserved. Arbitrary face subsets can be inspected but are not repair targets.
    pub fn repair_for_printing(
        &mut self,
        selection: Option<&PrintSelection>,
        options: &PrintRepairOptions,
    ) -> OpResult<PrintRepairSummary> {
        if selection.is_some_and(|s| {
            s.entities
                .iter()
                .any(|e| !matches!(e, EntityRef::Instance(_)))
        }) {
            return Err(crate::OpError::Invalid(varstotas_i18n::tr!(
                "print-repair-whole-objects"
            )));
        }
        let mut draft = self.clone();
        let summary = draft.repair_print_impl(selection, options)?;
        crate::io::validate_model(&draft).map_err(|e| crate::OpError::Invalid(e.to_string()))?;
        self.validate_locked_changes(&draft)
            .map_err(crate::OpError::Invalid)?;
        if summary.changed() {
            *self = draft;
        }
        Ok(summary)
    }
    fn print_context_editable(&self, path: &[InstanceId]) -> bool {
        !self.context_locked(path)
            && (0..=path.len()).all(|n| {
                self.def_at(&path[..n]).is_some_and(|d| {
                    let def = &self.defs[d];
                    def.params.is_none()
                        && def.generated.is_none()
                        && def.rig.is_none()
                        && def.rigged.is_none()
                })
            })
    }
    fn repair_print_impl(
        &mut self,
        selection: Option<&PrintSelection>,
        options: &PrintRepairOptions,
    ) -> OpResult<PrintRepairSummary> {
        let initial = self.print_check(selection, options.tolerance_mm);
        let original_normals: Vec<_> = initial
            .faces
            .iter()
            .filter_map(|f| {
                self.mesh_at(&f.path)?
                    .faces
                    .get(f.face)
                    .map(|face| (f.clone(), face.normal))
            })
            .collect();
        let mut targets: HashMap<ContextPath, HashSet<FaceId>> = HashMap::new();
        for f in initial.faces {
            targets.entry(f.path).or_default().insert(f.face);
        }
        let mut targets: Vec<_> = targets.into_iter().collect();
        targets.sort_by(|a, b| b.0.len().cmp(&a.0.len()).then(a.0.cmp(&b.0)));
        let mut total = PrintRepairSummary::default();
        for (path, mut faces) in targets {
            let Some(d) = self.def_at(&path) else {
                continue;
            };
            let definition = &self.defs[d];
            if !self.print_context_editable(&path) {
                total.skipped_objects += 1;
                continue;
            }
            let original_count = faces.len();
            faces.retain(|f| !self.entity_locked_in(d, EntityRef::Face(*f)));
            if faces.len() != original_count {
                total.skipped_objects += 1;
            }
            if faces.is_empty() {
                continue;
            }

            // Never move a vertex attached to protected geometry, including explicit locks.
            let protected: HashSet<_> = definition
                .mesh
                .edges
                .iter()
                .filter(|(e, _)| self.entity_locked_in(d, EntityRef::Edge(*e)))
                .flat_map(|(_, e)| e.v)
                .collect();
            let mut candidate = definition.mesh.clone();
            let transform = self.transform_at(&path);
            if !transform.is_finite() || transform.determinant().abs() < 1e-15 {
                total.skipped_objects += 1;
                continue;
            }
            let summary = repair_mesh(&mut candidate, &faces, &protected, &transform, options);
            if summary.changed() {
                // Fork every shared ancestor before installing a repaired descendant.
                let mut parent = Vec::new();
                for &i in &path {
                    self.make_unique(&parent, i)?;
                    parent.push(i);
                }
                let d = self.def_at(&path).ok_or(crate::OpError::NotFound)?;
                // Candidate geometry carries the latest nested-instance references (targets
                // are processed deepest first), so installing it preserves already-forked parts.
                self.defs[d].mesh = candidate;
                total.add(&summary);
            }
        }
        if options.orient_shells {
            let report = self.print_check(selection, options.tolerance_mm);
            for shell in report.shells.iter().filter(|s| s.inverted) {
                let paths: HashMap<_, _> = shell
                    .faces
                    .iter()
                    .map(|f| f.path.clone())
                    .collect::<HashSet<_>>()
                    .into_iter()
                    .map(|path| {
                        let editable = self.print_context_editable(&path);
                        (path, editable)
                    })
                    .collect();
                let editable = shell.faces.iter().all(|f| {
                    paths[&f.path]
                        && self
                            .def_at(&f.path)
                            .is_some_and(|d| !self.entity_locked_in(d, EntityRef::Face(f.face)))
                });
                if !editable {
                    total.skipped_objects += 1;
                    continue;
                }
                let mut paths: Vec<_> = shell
                    .faces
                    .iter()
                    .map(|f| f.path.clone())
                    .collect::<HashSet<_>>()
                    .into_iter()
                    .collect();
                paths.sort_by_key(Vec::len);
                for path in paths {
                    let mut parent = Vec::new();
                    for i in path {
                        self.make_unique(&parent, i)?;
                        parent.push(i);
                    }
                }
                for f in &shell.faces {
                    if let Some(mesh) = self.mesh_at_mut(&f.path) {
                        mesh.reverse_face(f.face);
                        total.reversed_faces += 1;
                    }
                }
            }
        }
        // Count the final changed faces, not intermediate BFS reversals that may be undone
        // when the complete shell is oriented (one bad cube face must not report 11 fixes).
        total.reversed_faces = original_normals
            .iter()
            .filter(|(f, n)| {
                self.mesh_at(&f.path)
                    .and_then(|m| m.faces.get(f.face))
                    .is_some_and(|face| face.normal.dot(*n) < 0.)
            })
            .count();
        Ok(total)
    }
}
// Booth's minimum cyclic rotation, linear even for large CAD polygon boundaries.
fn minimum_rotation(p: &[[i64; 3]]) -> Vec<[i64; 3]> {
    let n = p.len();
    if n == 0 {
        return Vec::new();
    }
    let (mut i, mut j, mut k) = (0, 1, 0);
    while i < n && j < n && k < n {
        match p[(i + k) % n].cmp(&p[(j + k) % n]) {
            std::cmp::Ordering::Equal => {
                k += 1;
                continue;
            }
            std::cmp::Ordering::Greater => {
                i += k + 1;
                if i == j {
                    i += 1;
                }
            }
            std::cmp::Ordering::Less => {
                j += k + 1;
                if i == j {
                    j += 1;
                }
            }
        }
        k = 0;
    }
    let start = i.min(j);
    (0..n).map(|k| p[(start + k) % n]).collect()
}
fn face_key(mesh: &crate::Mesh, f: FaceId, xf: &DMat4, tol: f64) -> Vec<Vec<[i64; 3]>> {
    let mut loops: Vec<_> = mesh.faces[f]
        .loops()
        .map(|lp| {
            let p: Vec<_> = lp
                .iter()
                .map(|v| {
                    xf.transform_point3(mesh.vertices[*v].pos)
                        .to_array()
                        .map(|x| (x / tol).round() as i64)
                })
                .collect();
            let forward = minimum_rotation(&p);
            let reversed: Vec<_> = p.iter().rev().copied().collect();
            forward.min(minimum_rotation(&reversed))
        })
        .collect();
    if loops.len() > 1 {
        loops[1..].sort();
    }
    loops
}
fn repair_mesh(
    mesh: &mut crate::Mesh,
    selected: &HashSet<FaceId>,
    locked_vertices: &HashSet<VertexId>,
    xf: &DMat4,
    o: &PrintRepairOptions,
) -> PrintRepairSummary {
    mesh.ensure();
    let source_vertices: HashSet<_> = selected
        .iter()
        .filter_map(|f| mesh.faces.get(*f))
        .flat_map(|f| f.loops().flatten().copied())
        .collect();
    let tol = tolerance(o.tolerance_mm);
    let mut out = PrintRepairSummary::default();
    let mut faces = selected.clone();
    if o.weld_vertices {
        let mut protected = locked_vertices.clone();
        for (f, face) in &mesh.faces {
            if !faces.contains(&f) {
                protected.extend(face.loops().flatten().copied());
            }
        }
        let mut vertices: Vec<_> = faces
            .iter()
            .filter_map(|f| mesh.faces.get(*f))
            .flat_map(|f| f.loops().flatten().copied())
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();
        vertices.sort();
        let mut pool = Points::default();
        let mut kept = Vec::new();
        for v in vertices {
            if protected.contains(&v) {
                continue;
            }
            let n = pool.points.len();
            let i = pool.add(0, xf.transform_point3(mesh.vertices[v].pos), tol);
            if i == n {
                kept.push(v);
                continue;
            }
            let keep = kept[i];
            let shading: Vec<_> = mesh
                .faces_of_vertex(v)
                .into_iter()
                .map(|f| {
                    (
                        f,
                        mesh.faces[f].normals.clone(),
                        mesh.faces[f].loops().map(Vec::len).collect::<Vec<_>>(),
                    )
                })
                .collect();
            let before = mesh.faces.len();
            mesh.merge_vertices(keep, v);
            out.removed_faces += before - mesh.faces.len();
            out.welded_vertices += 1;
            for (f, normals, loops) in shading {
                if let Some(face) = mesh.faces.get_mut(f)
                    && face.loops().map(Vec::len).collect::<Vec<_>>() == loops
                {
                    face.normals = normals;
                }
            }
        }
        faces.retain(|f| mesh.faces.contains_key(*f));
    }
    let mut seen = HashSet::new();
    if o.remove_duplicates {
        for f in mesh.faces.keys().filter(|f| !faces.contains(f)) {
            seen.insert(face_key(mesh, f, xf, tol));
        }
    }
    let mut sorted: Vec<_> = faces.iter().copied().collect();
    sorted.sort();
    let mut orphan_edges = HashSet::new();
    for f in sorted {
        let tris = crate::tessellate::triangulate_face(mesh, f);
        let area: f64 = tris
            .iter()
            .map(|p| {
                let p = p.map(|p| xf.transform_point3(p));
                (p[1] - p[0]).cross(p[2] - p[0]).length() * 0.5
            })
            .sum();
        let bad = o.remove_degenerate && area <= tol * tol;
        let duplicate = o.remove_duplicates && !seen.insert(face_key(mesh, f, xf, tol));
        if bad || duplicate {
            orphan_edges.extend(mesh.face_edges(f));
            mesh.f_remove(f);
            faces.remove(&f);
            out.removed_faces += 1;
        }
    }
    for e in orphan_edges {
        if mesh.edges.contains_key(e) && mesh.edge_faces(e).is_empty() && !mesh.edges[e].locked {
            mesh.e_remove(e);
        }
    }
    if o.orient_faces {
        let mut visited = HashSet::new();
        let mut seeds: Vec<_> = faces.iter().copied().collect();
        seeds.sort();
        for seed in seeds {
            if !visited.insert(seed) {
                continue;
            }
            let mut queue = vec![seed];
            while let Some(f) = queue.pop() {
                let loops: Vec<_> = mesh.faces[f].loops().cloned().collect();
                for lp in loops {
                    for i in 0..lp.len() {
                        let (a, b) = (lp[i], lp[(i + 1) % lp.len()]);
                        let Some(e) = mesh.edge_between(a, b) else {
                            continue;
                        };
                        let adjacent = mesh.edge_faces(e);
                        if adjacent.len() != 2 {
                            continue;
                        }
                        for g in adjacent {
                            if !faces.contains(&g) || !visited.insert(g) {
                                continue;
                            }
                            let same = mesh.faces[g].loops().any(|lp| {
                                (0..lp.len()).any(|k| lp[k] == a && lp[(k + 1) % lp.len()] == b)
                            });
                            if same {
                                mesh.reverse_face(g);
                                out.reversed_faces += 1;
                            }
                            queue.push(g);
                        }
                    }
                }
            }
        }
    }
    if o.fill_holes && o.max_hole_mm.is_finite() && o.max_hole_mm > 0. {
        out.filled_holes = fill_planar_holes(mesh, &faces, locked_vertices, xf, tol, o.max_hole_mm);
    }
    // Remove only newly orphaned vertices; unrelated wire geometry remains untouched.
    let orphan: Vec<_> = source_vertices
        .into_iter()
        .filter(|v| mesh.vertices.contains_key(*v))
        .filter(|v| mesh.vertex_edges(*v).is_empty() && !locked_vertices.contains(v))
        .collect();
    for v in orphan {
        mesh.v_remove(v);
    }
    mesh.rebuild_caches();
    out
}
fn fill_planar_holes(
    mesh: &mut crate::Mesh,
    faces: &HashSet<FaceId>,
    protected: &HashSet<VertexId>,
    xf: &DMat4,
    tol: f64,
    max_size: f64,
) -> usize {
    let mut next: HashMap<VertexId, Vec<VertexId>> = HashMap::new();
    let mut sources = HashMap::new();
    for &f in faces {
        for lp in mesh.faces[f].loops() {
            for i in 0..lp.len() {
                let (a, b) = (lp[i], lp[(i + 1) % lp.len()]);
                let Some(e) = mesh.edge_between(a, b) else {
                    continue;
                };
                if mesh.edge_faces(e).len() == 1
                    && !mesh.edges[e].locked
                    && !protected.contains(&a)
                    && !protected.contains(&b)
                {
                    next.entry(b).or_default().push(a);
                    sources.insert((b, a), f);
                }
            }
        }
    }
    let mut visited = HashSet::new();
    let mut loops = Vec::new();
    let mut starts: Vec<_> = next.keys().copied().collect();
    starts.sort();
    for start in starts {
        if visited.contains(&start) {
            continue;
        }
        let mut lp = Vec::new();
        let mut v = start;
        let mut closed = false;
        while lp.len() < 10_000 && visited.insert(v) {
            lp.push(v);
            let Some(neighbors) = next.get(&v).filter(|n| n.len() == 1) else {
                break;
            };
            v = neighbors[0];
            if v == start {
                closed = true;
                break;
            }
        }
        if !closed || lp.len() < 3 {
            continue;
        }
        let points: Vec<_> = lp
            .iter()
            .map(|v| xf.transform_point3(mesh.vertices[*v].pos))
            .collect();
        let relative: Vec<_> = points.iter().map(|p| *p - points[0]).collect();
        let normal = crate::geom::newell(&relative).normalize_or_zero();
        if normal == DVec3::ZERO || relative.iter().any(|p| p.dot(normal).abs() > tol) {
            continue;
        }
        let pr = crate::geom::Projector::new(points[0], normal);
        let polygon: Vec<_> = points.iter().map(|p| pr.to2(*p)).collect();
        if crate::triangulate::triangulate(&polygon, &[]).len() != lp.len() - 2 {
            continue;
        }
        let intersects = (0..lp.len()).any(|i| {
            ((i + 1)..lp.len()).any(|j| {
                j != i + 1
                    && !(i == 0 && j == lp.len() - 1)
                    && crate::geom::segs_intersect_2d(
                        polygon[i],
                        polygon[(i + 1) % lp.len()],
                        polygon[j],
                        polygon[(j + 1) % lp.len()],
                        tol,
                    )
            })
        });
        if intersects {
            continue;
        }
        loops.push((lp, points, normal, pr, polygon));
    }
    let mut nested = HashSet::new();
    for i in 0..loops.len() {
        for j in i + 1..loops.len() {
            let (a, b) = (&loops[i], &loops[j]);
            if a.2.dot(b.2).abs() > 1. - 1e-10 && (a.1[0] - b.1[0]).dot(b.2).abs() <= tol {
                let inside = crate::geom::point_in_polygon(a.3.to2(b.1[0]), &a.4, tol)
                    != crate::geom::PointLoc::Outside
                    || crate::geom::point_in_polygon(b.3.to2(a.1[0]), &b.4, tol)
                        != crate::geom::PointLoc::Outside;
                if inside {
                    nested.insert(i);
                    nested.insert(j);
                }
            }
        }
    }
    let mut filled = 0;
    for (i, (lp, points, normal, _, _)) in loops.into_iter().enumerate() {
        if nested.contains(&i)
            || Aabb::from_points(points.iter().copied())
                .is_none_or(|b| (b.max - b.min).length() > max_size)
        {
            continue;
        }
        let source = sources[&(lp[0], lp[1])];
        // A flat sheet's outer boundary is not a hole in a solid. Require its connected
        // surface to leave the cap plane before considering any closure.
        let mut seen = HashSet::new();
        let mut pending = vec![source];
        let mut body = false;
        while let Some(f) = pending.pop() {
            if !seen.insert(f) {
                continue;
            }
            if mesh.faces[f].loops().flatten().any(|v| {
                (xf.transform_point3(mesh.vertices[*v].pos) - points[0])
                    .dot(normal)
                    .abs()
                    > tol
            }) {
                body = true;
                break;
            }
            for e in mesh.face_edges(f) {
                pending.extend(mesh.edge_faces(e).into_iter().filter(|f| faces.contains(f)));
            }
        }
        if !body {
            continue;
        }
        let mut template = crate::Face::template();
        let neighbor = &mesh.faces[source];
        template.front = neighbor.front;
        template.back = neighbor.back;
        template.tag = neighbor.tag;
        mesh.f_new(lp, Vec::new(), &template);
        filled += 1;
    }
    filled
}
