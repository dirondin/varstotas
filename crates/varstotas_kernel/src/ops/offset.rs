//! Offset of a face boundary within its plane (mitered joins).

use crate::geom::{self, PointLoc, Projector};
use crate::mesh::Mesh;
use crate::topology::{Classify, Orient};
use crate::{FaceId, OpError, OpResult, TOL};
use glam::{DVec2, DVec3};

/// Offset a closed 2D loop: each edge moves by `d` along its right-hand normal
/// (outward for a CCW loop). Returns `None` if the loop collapses.
pub(crate) fn offset_loop(pts: &[DVec2], d: f64) -> Option<Vec<DVec2>> {
    let n = pts.len();
    if n < 3 {
        return None;
    }
    let right = |a: DVec2, b: DVec2| {
        let t = (b - a).normalize();
        DVec2::new(t.y, -t.x)
    };
    let mut out = Vec::with_capacity(n);
    for i in 0..n {
        let (p0, p1, p2) = (pts[(i + n - 1) % n], pts[i], pts[(i + 1) % n]);
        let (r1, r2) = (right(p0, p1), right(p1, p2));
        let q = match geom::line_line_2d(p0 + r1 * d, p1 - p0, p1 + r2 * d, p2 - p1) {
            Some(q) => q,
            None => p1 + r1 * d,
        };
        out.push(q);
    }
    // Every edge must keep its direction and a non-trivial length.
    for i in 0..n {
        let (a, b) = (pts[i], pts[(i + 1) % n]);
        let (c, e) = (out[i], out[(i + 1) % n]);
        if (e - c).length() < TOL || (e - c).dot(b - a) <= 0.0 {
            return None;
        }
    }
    Some(out)
}

fn loops_intersect(loops: &[Vec<DVec2>]) -> bool {
    let mut segs: Vec<(usize, usize, DVec2, DVec2)> = Vec::new();
    for (li, l) in loops.iter().enumerate() {
        for i in 0..l.len() {
            segs.push((li, i, l[i], l[(i + 1) % l.len()]));
        }
    }
    if segs.len() > 4000 {
        return false; // too expensive to check exhaustively
    }
    for i in 0..segs.len() {
        for j in i + 1..segs.len() {
            let (li, ii, a, b) = segs[i];
            let (lj, jj, c, d) = segs[j];
            if li == lj {
                let n = loops[li].len();
                if jj == (ii + 1) % n || ii == (jj + 1) % n {
                    continue;
                }
            }
            if geom::segs_intersect_2d(a, b, c, d, 1e-9) {
                return true;
            }
        }
    }
    false
}

impl Mesh {
    /// Offset face boundary inward (negative) or outward (positive) within the face plane,
    /// splitting the face. Holes are offset consistently (they shrink when the face grows).
    /// Returns the new inner face for an inward offset, or the new outer ring face for an
    /// outward offset. Errors with `Degenerate` if the offset collapses or self-intersects.
    pub fn offset_face(&mut self, f: FaceId, distance: f64) -> OpResult<FaceId> {
        self.ensure();
        if !self.faces.contains_key(f) {
            return Err(OpError::NotFound);
        }
        if distance.abs() < TOL || !distance.is_finite() {
            return Err(OpError::Degenerate);
        }
        let face = self.faces[f].clone();
        let pr = Projector::new(self.pos(face.outer[0]), face.normal);
        let to2 = |m: &Mesh, l: &Vec<crate::VertexId>| -> Vec<DVec2> { l.iter().map(|v| pr.to2(m.pos(*v))).collect() };
        let outer2 = to2(self, &face.outer);
        let holes2: Vec<Vec<DVec2>> = face.holes.iter().map(|h| to2(self, h)).collect();
        // Outer is CCW and holes CW in this projection; "right" normal points out of the face.
        let new_outer = offset_loop(&outer2, distance).ok_or(OpError::Degenerate)?;
        let mut new_holes = Vec::new();
        for h in &holes2 {
            if let Some(nh) = offset_loop(h, distance) {
                if geom::signed_area(&nh) < 0.0 {
                    new_holes.push(nh);
                }
                // a hole that vanished when growing the face simply disappears
            } else if distance < 0.0 {
                return Err(OpError::Degenerate);
            }
        }
        if geom::signed_area(&new_outer) <= 0.0 {
            return Err(OpError::Degenerate);
        }
        let mut all = vec![new_outer.clone()];
        all.extend(new_holes.iter().cloned());
        if loops_intersect(&all) {
            return Err(OpError::Degenerate);
        }
        // Inward offset must stay inside the original face (holes included).
        if distance < 0.0 {
            for l in &all {
                for p in l {
                    if geom::point_in_region(*p, &outer2, &holes2, 0.0) != PointLoc::Inside {
                        return Err(OpError::Degenerate);
                    }
                }
            }
        }
        let o3: Vec<DVec3> = new_outer.iter().map(|p| pr.to3(*p)).collect();
        let h3: Vec<Vec<DVec3>> = new_holes.iter().map(|h| h.iter().map(|p| pr.to3(*p)).collect()).collect();
        let mut seeds = Vec::new();
        for l in std::iter::once(&o3).chain(h3.iter()) {
            for i in 0..l.len() {
                let (pieces, _) = self.insert_segment(l[i], l[(i + 1) % l.len()]);
                for e in pieces {
                    seeds.extend(self.edges[e].v);
                }
            }
        }
        seeds.extend(face.outer.iter().copied());
        let plane = crate::Plane::new(pr.origin, face.normal);
        let cls = Classify::Region { outer: o3.clone(), holes: h3.clone() };
        let regs = self.refresh_plane(plane, &seeds, &[f], &cls, Orient::prefer(face.normal));
        let inside_new = |p: DVec2| geom::point_in_region(p, &new_outer, &new_holes, TOL) == PointLoc::Inside;
        let inside_old = |p: DVec2| geom::point_in_polygon(p, &outer2, TOL) == PointLoc::Inside;
        let pick = regs
            .iter()
            .filter(|r| self.faces.contains_key(r.face))
            .filter(|r| {
                let s = pr.to2(r.sample);
                if distance < 0.0 { inside_new(s) } else { inside_new(s) && !inside_old(s) }
            })
            .max_by(|a, b| a.area.total_cmp(&b.area))
            .map(|r| r.face);
        pick.ok_or(OpError::Degenerate)
    }
}
