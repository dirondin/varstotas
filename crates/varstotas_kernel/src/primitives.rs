//! 3D primitives (Draw → Shapes): box, cylinder, cone, pyramid, sphere and torus as closed
//! solids. Each shape is a list of planar polygons in its own axes (outward normals, CCW seen
//! from outside); [`Primitive::mesh`] welds them into a [`Mesh`] with the edges inside the
//! curved surfaces soft and smooth.
//!
//! Own axes: the box and the pyramid start at the origin (`0..w`, `0..d`), the cylinder and
//! the cone stand on a base centred at the origin, the sphere and the torus are centred at it.

use crate::Mesh;
use glam::DVec3;
use std::collections::HashSet;
use std::f64::consts::TAU;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Primitive {
    Box { w: f64, d: f64, h: f64 },
    Cylinder { r: f64, h: f64, segs: u32 },
    Cone { r: f64, h: f64, segs: u32 },
    Pyramid { w: f64, d: f64, h: f64 },
    Sphere { r: f64, segs: u32 },
    /// `r`: from the centre to the middle of the tube; `tube`: the tube's radius.
    Torus { r: f64, tube: f64, segs: u32, sides: u32 },
}

/// A polygon of a primitive and whether it belongs to a curved surface.
pub type Poly = (Vec<DVec3>, bool);

fn ring(r: f64, z: f64, segs: u32) -> Vec<DVec3> {
    (0..segs).map(|i| {
        let a = TAU * i as f64 / segs as f64;
        DVec3::new(r * a.cos(), r * a.sin(), z)
    })
    .collect()
}

impl Primitive {
    /// Sizes are positive and finite, segment counts in range.
    pub fn is_valid(&self) -> bool {
        let pos = |v: &[f64]| v.iter().all(|x| x.is_finite() && *x > 1e-6);
        let segs = |s: u32| (3..=1000).contains(&s);
        match *self {
            Primitive::Box { w, d, h } | Primitive::Pyramid { w, d, h } => pos(&[w, d, h]),
            Primitive::Cylinder { r, h, segs: s } | Primitive::Cone { r, h, segs: s } => pos(&[r, h]) && segs(s),
            Primitive::Sphere { r, segs: s } => pos(&[r]) && segs(s) && s >= 4,
            Primitive::Torus { r, tube, segs: s, sides } => pos(&[r, tube]) && tube < r && segs(s) && segs(sides),
        }
    }

    /// The polygons of the shape (empty when not [`Self::is_valid`]).
    pub fn polygons(&self) -> Vec<Poly> {
        if !self.is_valid() {
            return Vec::new();
        }
        let mut out: Vec<Poly> = Vec::new();
        match *self {
            Primitive::Box { w, d, h } => {
                let c = |x: f64, y: f64, z: f64| DVec3::new(x * w, y * d, z * h);
                let quads = [
                    [c(0., 0., 0.), c(0., 1., 0.), c(1., 1., 0.), c(1., 0., 0.)],
                    [c(0., 0., 1.), c(1., 0., 1.), c(1., 1., 1.), c(0., 1., 1.)],
                    [c(0., 0., 0.), c(1., 0., 0.), c(1., 0., 1.), c(0., 0., 1.)],
                    [c(1., 0., 0.), c(1., 1., 0.), c(1., 1., 1.), c(1., 0., 1.)],
                    [c(1., 1., 0.), c(0., 1., 0.), c(0., 1., 1.), c(1., 1., 1.)],
                    [c(0., 1., 0.), c(0., 0., 0.), c(0., 0., 1.), c(0., 1., 1.)],
                ];
                out.extend(quads.iter().map(|q| (q.to_vec(), false)));
            }
            Primitive::Cylinder { r, h, segs } => {
                let (lo, hi) = (ring(r, 0.0, segs), ring(r, h, segs));
                out.push((lo.iter().rev().copied().collect(), false));
                out.push((hi.clone(), false));
                for i in 0..segs as usize {
                    let j = (i + 1) % segs as usize;
                    out.push((vec![lo[i], lo[j], hi[j], hi[i]], true));
                }
            }
            Primitive::Cone { r, h, segs } => {
                let lo = ring(r, 0.0, segs);
                let apex = DVec3::Z * h;
                out.push((lo.iter().rev().copied().collect(), false));
                for i in 0..segs as usize {
                    let j = (i + 1) % segs as usize;
                    out.push((vec![lo[i], lo[j], apex], true));
                }
            }
            Primitive::Pyramid { w, d, h } => {
                let b = [DVec3::ZERO, DVec3::new(w, 0.0, 0.0), DVec3::new(w, d, 0.0), DVec3::new(0.0, d, 0.0)];
                let apex = DVec3::new(w / 2.0, d / 2.0, h);
                out.push((b.iter().rev().copied().collect(), false));
                for i in 0..4 {
                    out.push((vec![b[i], b[(i + 1) % 4], apex], false));
                }
            }
            Primitive::Sphere { r, segs } => {
                let rings = (segs / 2).max(2);
                // Latitude k = 0 (south pole) ..= rings (north pole).
                let at = |k: u32, i: u32| {
                    let phi = std::f64::consts::PI * k as f64 / rings as f64 - std::f64::consts::FRAC_PI_2;
                    let a = TAU * (i % segs) as f64 / segs as f64;
                    DVec3::new(r * phi.cos() * a.cos(), r * phi.cos() * a.sin(), r * phi.sin())
                };
                for k in 0..rings {
                    for i in 0..segs {
                        let poly = if k == 0 {
                            vec![at(0, 0), at(1, i + 1), at(1, i)]
                        } else if k == rings - 1 {
                            vec![at(k, i), at(k, i + 1), at(rings, 0)]
                        } else {
                            vec![at(k, i), at(k, i + 1), at(k + 1, i + 1), at(k + 1, i)]
                        };
                        out.push((poly, true));
                    }
                }
                // South pole triangles were listed clockwise seen from outside: reverse them.
                for p in out.iter_mut().take(segs as usize) {
                    p.0.reverse();
                }
            }
            Primitive::Torus { r, tube, segs, sides } => {
                let at = |i: u32, k: u32| {
                    let a = TAU * (i % segs) as f64 / segs as f64;
                    let b = TAU * (k % sides) as f64 / sides as f64;
                    let rr = r + tube * b.cos();
                    DVec3::new(rr * a.cos(), rr * a.sin(), tube * b.sin())
                };
                for i in 0..segs {
                    for k in 0..sides {
                        out.push((vec![at(i, k), at(i + 1, k), at(i + 1, k + 1), at(i, k + 1)], true));
                    }
                }
            }
        }
        out
    }

    /// The shape as a mesh: one closed solid, curved surfaces soft and smooth. None when not
    /// [`Self::is_valid`].
    pub fn mesh(&self) -> Option<Mesh> {
        let polys = self.polygons();
        if polys.is_empty() {
            return None;
        }
        let mut m = Mesh::new();
        let (mut curved, mut flat) = (Vec::new(), Vec::new());
        for (p, c) in &polys {
            let f = m.add_face_exact(p)?;
            if *c { curved.push(f) } else { flat.push(f) }
        }
        let hard: HashSet<_> = flat.iter().flat_map(|f| m.face_edges(*f)).collect();
        let mut edges: Vec<_> = curved.iter().flat_map(|f| m.face_edges(*f)).filter(|e| !hard.contains(e)).collect();
        edges.sort();
        edges.dedup();
        m.soften_by_angle(&edges, 179.0, true, false);
        Some(m)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn closed(m: &Mesh) -> bool {
        m.edges.keys().all(|e| m.edge_faces(e).len() == 2)
    }

    #[test]
    fn primitives_are_closed_solids() {
        let shapes = [
            (Primitive::Box { w: 100.0, d: 200.0, h: 300.0 }, 6, 6_000_000.0),
            (Primitive::Cylinder { r: 50.0, h: 100.0, segs: 24 }, 26, 0.0),
            (Primitive::Cone { r: 50.0, h: 100.0, segs: 24 }, 25, 0.0),
            (Primitive::Pyramid { w: 100.0, d: 100.0, h: 300.0 }, 5, 1_000_000.0),
            (Primitive::Sphere { r: 50.0, segs: 24 }, 24 * 12, 0.0),
            (Primitive::Torus { r: 100.0, tube: 20.0, segs: 24, sides: 12 }, 24 * 12, 0.0),
        ];
        for (p, faces, volume) in shapes {
            let m = p.mesh().unwrap_or_else(|| panic!("{p:?}"));
            assert_eq!(m.faces.len(), faces, "{p:?}");
            assert!(closed(&m), "{p:?} is not closed");
            if volume > 0.0 {
                let v = m.volume();
                assert!((v - volume).abs() < 1e-3 * volume, "{p:?}: {v}");
            } else {
                assert!(m.volume() > 0.0, "{p:?}: inside out");
            }
        }
        // The curved surface is soft, the caps' rims are hard.
        let m = Primitive::Cylinder { r: 50.0, h: 100.0, segs: 12 }.mesh().unwrap();
        assert_eq!(m.edges.values().filter(|e| e.soft && e.smooth).count(), 12);
        assert_eq!(m.edges.values().filter(|e| !e.soft).count(), 24);
        assert!(Primitive::Torus { r: 10.0, tube: 20.0, segs: 8, sides: 8 }.mesh().is_none());
    }
}
