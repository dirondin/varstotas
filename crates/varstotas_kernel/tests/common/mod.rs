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
