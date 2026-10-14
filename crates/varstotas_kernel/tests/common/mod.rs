#![allow(dead_code)]
use varstotas_kernel::glam::DVec3;
use varstotas_kernel::*;

pub fn v(x: f64, y: f64, z: f64) -> DVec3 {
    DVec3::new(x, y, z)
}

/// Rectangle on z = `z` drawn with four `add_line` calls.
pub fn rect_lines(m: &mut Mesh, x0: f64, y0: f64, x1: f64, y1: f64, z: f64) {
    m.add_line(v(x0, y0, z), v(x1, y0, z));
    m.add_line(v(x1, y0, z), v(x1, y1, z));
    m.add_line(v(x1, y1, z), v(x0, y1, z));
    m.add_line(v(x0, y1, z), v(x0, y0, z));
}

pub fn ok(m: &Mesh) {
    if let Err(e) = m.validate() {
        panic!("invalid mesh: {e}");
    }
}

/// Every edge used by exactly two faces that traverse it in opposite directions.
pub fn assert_closed_consistent(m: &Mesh) {
    for (e, ed) in &m.edges {
        let fs = m.edge_faces(e);
        assert_eq!(fs.len(), 2, "edge {:?} has {} faces", ed.v, fs.len());
        let dir = |f: FaceId| {
            let face = &m.faces[f];
            for l in face.loops() {
                for i in 0..l.len() {
                    if l[i] == ed.v[0] && l[(i + 1) % l.len()] == ed.v[1] {
                        return 1;
                    }
                    if l[i] == ed.v[1] && l[(i + 1) % l.len()] == ed.v[0] {
                        return -1;
                    }
                }
            }
            0
        };
        assert_eq!(dir(fs[0]) + dir(fs[1]), 0, "inconsistent orientation at edge");
    }
}

pub fn centroid(m: &Mesh) -> DVec3 {
    let n = m.vertices.len() as f64;
    m.vertices.values().map(|v| v.pos).sum::<DVec3>() / n
}

pub fn face_centroid(m: &Mesh, f: FaceId) -> DVec3 {
    let o = &m.faces[f].outer;
    o.iter().map(|v| m.vertices[*v].pos).sum::<DVec3>() / o.len() as f64
}

/// For convex solids: all normals point away from the centroid.
pub fn assert_outward(m: &Mesh) {
    let c = centroid(m);
    for (f, face) in &m.faces {
        let d = face_centroid(m, f) - c;
        assert!(face.normal.dot(d) > 0.0, "face normal {:?} points inward", face.normal);
    }
}

pub fn total_area(m: &Mesh) -> f64 {
    m.faces.keys().map(|f| m.face_area(f)).sum()
}

pub fn face_with_normal(m: &Mesh, n: DVec3) -> Vec<FaceId> {
    m.faces.iter().filter(|(_, f)| f.normal.dot(n) > 0.999).map(|(id, _)| id).collect()
}

/// Ground rectangle face (faces down) built with add_face.
pub fn ground_rect(m: &mut Mesh, w: f64, d: f64) -> FaceId {
    m.add_face(&[v(0., 0., 0.), v(w, 0., 0.), v(w, d, 0.), v(0., d, 0.)]).unwrap()
}

/// Closed box w x d x h on the ground.
pub fn make_box(m: &mut Mesh, w: f64, d: f64, h: f64) -> FaceId {
    let f = ground_rect(m, w, d);
    m.push_pull(f, -h, false).unwrap()
}

pub fn volume(m: &Mesh) -> f64 {
    // Divergence theorem over triangulated faces.
    let mut vol = 0.0;
    for f in m.faces.keys() {
        for t in tessellate::triangulate_face(m, f) {
            vol += t[0].dot(t[1].cross(t[2])) / 6.0;
        }
    }
    vol
}

/// Two edges touching other than at a shared endpoint (crossing, T-junction, overlap)?
pub fn edges_touch(m: &Mesh) -> Option<(DVec3, DVec3, DVec3, DVec3)> {
    let segs: Vec<(DVec3, DVec3)> = m.edges.values().map(|e| (m.vertices[e.v[0]].pos, m.vertices[e.v[1]].pos)).collect();
    let tol = 1e-4;
    let interior = |p: DVec3, a: DVec3, b: DVec3| geom::point_seg_dist(p, a, b).0 < tol && p.distance(a) > tol && p.distance(b) > tol;
    for (i, &(a, b)) in segs.iter().enumerate() {
        for &(c, d) in &segs[i + 1..] {
            if interior(c, a, b) || interior(d, a, b) || interior(a, c, d) || interior(b, c, d) {
                return Some((a, b, c, d));
            }
            if let Some((_, _, p, q)) = geom::seg_seg_closest(a, b, c, d)
                && p.distance(q) < tol && interior(p, a, b) && interior(q, c, d) {
                    return Some((a, b, c, d));
                }
        }
    }
    None
}

/// Coplanar faces sharing interior area?
pub fn coplanar_overlap(m: &Mesh) -> Option<(FaceId, FaceId)> {
    use geom::PointLoc;
    for (f, face) in &m.faces {
        let sample = tessellate::triangulate_face(m, f)
            .into_iter()
            .max_by(|a, b| (a[1] - a[0]).cross(a[2] - a[0]).length().total_cmp(&(b[1] - b[0]).cross(b[2] - b[0]).length()))
            .map(|t| (t[0] + t[1] + t[2]) / 3.0)?;
        for (g, gf) in &m.faces {
            if g == f || gf.normal.dot(face.normal).abs() < 1.0 - 1e-6 || m.face_plane(g).distance(sample).abs() > TOL {
                continue;
            }
            if m.face_point_loc(g, sample) == PointLoc::Inside {
                return Some((f, g));
            }
        }
    }
    None
}

/// Valid topology, no edges touching without a vertex, no overlapping coplanar faces.
pub fn sane(m: &Mesh) {
    ok(m);
    if let Some(t) = edges_touch(m) {
        panic!("edges touch without a shared vertex: {t:?}");
    }
    if let Some(o) = coplanar_overlap(m) {
        panic!("overlapping coplanar faces: {o:?}");
    }
}

/// Closed box [lo, hi] built with sticky ops in `m` (outward normals).
pub fn box_at(m: &mut Mesh, lo: DVec3, hi: DVec3) {
    let f = m.add_face_exact(&[v(lo.x, lo.y, lo.z), v(lo.x, hi.y, lo.z), v(hi.x, hi.y, lo.z), v(hi.x, lo.y, lo.z)]).unwrap();
    m.push_pull(f, -(hi.z - lo.z), false).unwrap();
}
