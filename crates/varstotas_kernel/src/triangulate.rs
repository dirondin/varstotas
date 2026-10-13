//! Polygon-with-holes triangulation (ear clipping with Eberly hole bridging).

use crate::geom::signed_area;
use glam::DVec2;

/// Triangulate a simple polygon with holes. Returns CCW triangles as indices into the
/// concatenated point list `[outer..., hole0..., hole1..., ...]`. Any input orientation is
/// accepted. Degenerate input yields as many triangles as could be recovered.
pub fn triangulate(outer: &[DVec2], holes: &[&[DVec2]]) -> Vec<[usize; 3]> {
    if outer.len() < 3 {
        return Vec::new();
    }
    let mut pts: Vec<DVec2> = outer.to_vec();
    let mut ring: Vec<usize> = (0..outer.len()).collect();
    if signed_area(outer) < 0.0 {
        ring.reverse();
    }
    let mut hole_rings: Vec<Vec<usize>> = Vec::new();
    for h in holes {
        let off = pts.len();
        pts.extend_from_slice(h);
        if h.len() < 3 {
            continue;
        }
        let mut r: Vec<usize> = (off..off + h.len()).collect();
        if signed_area(h) > 0.0 {
            r.reverse();
        }
        hole_rings.push(r);
    }
    let (mut lo, mut hi) = (pts[0], pts[0]);
    for p in &pts {
        lo = lo.min(*p);
        hi = hi.max(*p);
    }
    let scale = (hi - lo).max_element().max(1e-12);
    let eps = scale * scale * 1e-14;

    if hole_rings.is_empty() && is_convex(&pts, &ring, eps) {
        let mut out = Vec::with_capacity(ring.len() - 2);
        for i in 1..ring.len() - 1 {
            if (pts[ring[i]] - pts[ring[0]]).perp_dot(pts[ring[i + 1]] - pts[ring[0]]) > eps {
                out.push([ring[0], ring[i], ring[i + 1]]);
            }
        }
        return out;
    }
    hole_rings.sort_by(|a, b| {
        let ma = a.iter().map(|i| pts[*i].x).fold(f64::MIN, f64::max);
        let mb = b.iter().map(|i| pts[*i].x).fold(f64::MIN, f64::max);
        mb.total_cmp(&ma)
    });
    for h in hole_rings {
        ring = bridge(&pts, ring, &h, eps);
    }
    ear_clip(&pts, ring, eps)
}

fn is_convex(pts: &[DVec2], ring: &[usize], eps: f64) -> bool {
    let n = ring.len();
    (0..n).all(|i| {
        let (a, b, c) = (pts[ring[(i + n - 1) % n]], pts[ring[i]], pts[ring[(i + 1) % n]]);
        (b - a).perp_dot(c - b) >= -eps
    })
}

fn cross(o: DVec2, a: DVec2, b: DVec2) -> f64 {
    (a - o).perp_dot(b - o)
}

/// Is direction P→M inside the interior wedge at ring position `k` (CCW ring)?
fn locally_inside(pts: &[DVec2], ring: &[usize], k: usize, m: DVec2) -> bool {
    let n = ring.len();
    let (a, p, b) = (pts[ring[(k + n - 1) % n]], pts[ring[k]], pts[ring[(k + 1) % n]]);
    if cross(a, p, b) >= 0.0 {
        // convex corner
        cross(p, b, m) >= 0.0 && cross(p, m, a) >= 0.0
    } else {
        !(cross(p, a, m) > 0.0 && cross(p, m, b) > 0.0)
    }
}

fn bridge(pts: &[DVec2], ring: Vec<usize>, hole: &[usize], eps: f64) -> Vec<usize> {
    let n = ring.len();
    let mi = (0..hole.len()).max_by(|a, b| pts[hole[*a]].x.total_cmp(&pts[hole[*b]].x)).unwrap();
    let m = pts[hole[mi]];
    // Ray from M towards +x: nearest edge crossing.
    let mut best: Option<(f64, usize)> = None; // (x, ring position of candidate P)
    for i in 0..n {
        let (ia, ib) = (ring[i], ring[(i + 1) % n]);
        let (a, b) = (pts[ia], pts[ib]);
        if (a.y > m.y && b.y > m.y) || (a.y < m.y && b.y < m.y) {
            continue;
        }
        let x = if (b.y - a.y).abs() < 1e-300 {
            if a.x < m.x && b.x < m.x {
                continue;
            }
            a.x.min(b.x).max(m.x)
        } else {
            a.x + (m.y - a.y) * (b.x - a.x) / (b.y - a.y)
        };
        if x < m.x - eps.sqrt() {
            continue;
        }
        if best.is_none_or(|(bx, _)| x < bx) {
            let k = if (x - a.x).abs() < 1e-12 && (m.y - a.y).abs() < 1e-12 {
                i
            } else if (x - b.x).abs() < 1e-12 && (m.y - b.y).abs() < 1e-12 {
                (i + 1) % n
            } else if a.x > b.x {
                i
            } else {
                (i + 1) % n
            };
            best = Some((x, k));
        }
    }
    let Some((ix, mut k)) = best else {
        // Fallback: nearest ring vertex.
        let k = (0..n).min_by(|a, b| (pts[ring[*a]] - m).length().total_cmp(&(pts[ring[*b]] - m).length())).unwrap();
        return splice(ring, hole, mi, k);
    };
    let i_pt = DVec2::new(ix, m.y);
    let p = pts[ring[k]];
    // Vertices inside triangle (M, I, P) may block visibility: pick the one with the smallest
    // angle to the ray.
    let tri_has = |q: DVec2| {
        let (d1, d2, d3) = (cross(m, i_pt, q), cross(i_pt, p, q), cross(p, m, q));
        let neg = d1 < 0.0 || d2 < 0.0 || d3 < 0.0;
        let pos = d1 > 0.0 || d2 > 0.0 || d3 > 0.0;
        !(neg && pos)
    };
    if (p - i_pt).length() > 1e-12 {
        let mut best_ang = f64::INFINITY;
        let mut best_d = f64::INFINITY;
        for j in 0..n {
            let q = pts[ring[j]];
            if q == p || q.x < m.x {
                continue;
            }
            if tri_has(q) {
                let d = q - m;
                let ang = d.y.abs().atan2(d.x);
                let dist = d.length();
                if (ang < best_ang - 1e-12 || ((ang - best_ang).abs() <= 1e-12 && dist < best_d))
                    && locally_inside(pts, &ring, j, m)
                {
                    best_ang = ang;
                    best_d = dist;
                    k = j;
                }
            }
        }
    }
    // Among duplicate occurrences of the chosen vertex, take the one whose wedge contains M.
    let target = ring[k];
    let target_pos = pts[target];
    for j in 0..n {
        if pts[ring[j]] == target_pos && locally_inside(pts, &ring, j, m) {
            k = j;
            break;
        }
    }
    splice(ring, hole, mi, k)
}

fn splice(ring: Vec<usize>, hole: &[usize], mi: usize, k: usize) -> Vec<usize> {
    let mut out = Vec::with_capacity(ring.len() + hole.len() + 2);
    out.extend_from_slice(&ring[..=k]);
    out.extend_from_slice(&hole[mi..]);
    out.extend_from_slice(&hole[..=mi]);
    out.push(ring[k]);
    out.extend_from_slice(&ring[k + 1..]);
    out
}

fn ear_clip(pts: &[DVec2], mut v: Vec<usize>, eps: f64) -> Vec<[usize; 3]> {
    let mut tris = Vec::with_capacity(v.len().saturating_sub(2));
    let mut i = 0usize;
    let mut fails = 0usize;
    while v.len() > 3 {
        let n = v.len();
        i %= n;
        let (ia, ib, ic) = (v[(i + n - 1) % n], v[i], v[(i + 1) % n]);
        if is_ear(pts, &v, ia, ib, ic, eps) {
            tris.push([ia, ib, ic]);
            v.remove(i);
            fails = 0;
            i = (i + n - 2) % (n - 1);
            continue;
        }
        i += 1;
        fails += 1;
        if fails >= n {
            // Stuck (degenerate input): drop a collinear vertex, else force the most convex ear.
            let cr = |k: usize| {
                let (a, b, c) = (pts[v[(k + n - 1) % n]], pts[v[k]], pts[v[(k + 1) % n]]);
                (b - a).perp_dot(c - b)
            };
            if let Some(k) = (0..n).find(|k| cr(*k).abs() <= eps) {
                v.remove(k);
            } else {
                let k = (0..n).max_by(|a, b| cr(*a).total_cmp(&cr(*b))).unwrap();
                if cr(k) > 0.0 {
                    tris.push([v[(k + n - 1) % n], v[k], v[(k + 1) % n]]);
                }
                v.remove(k);
            }
            fails = 0;
            i = 0;
        }
    }
    if v.len() == 3 && cross(pts[v[0]], pts[v[1]], pts[v[2]]) > eps {
        tris.push([v[0], v[1], v[2]]);
    }
    tris
}

fn is_ear(pts: &[DVec2], v: &[usize], ia: usize, ib: usize, ic: usize, eps: f64) -> bool {
    let (a, b, c) = (pts[ia], pts[ib], pts[ic]);
    if cross(a, b, c) <= eps {
        return false;
    }
    for &j in v {
        let p = pts[j];
        if p == a || p == b || p == c {
            continue;
        }
        // Inside or on the boundary of the triangle?
        if cross(a, b, p) >= -eps && cross(b, c, p) >= -eps && cross(c, a, p) >= -eps {
            return false;
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    fn area(pts: &[DVec2], tris: &[[usize; 3]]) -> f64 {
        tris.iter().map(|t| cross(pts[t[0]], pts[t[1]], pts[t[2]]) * 0.5).sum()
    }

    #[test]
    fn square_with_hole() {
        let outer = [DVec2::new(0., 0.), DVec2::new(10., 0.), DVec2::new(10., 10.), DVec2::new(0., 10.)];
        let hole = [DVec2::new(3., 3.), DVec2::new(3., 7.), DVec2::new(7., 7.), DVec2::new(7., 3.)];
        let tris = triangulate(&outer, &[&hole]);
        let all: Vec<DVec2> = outer.iter().chain(hole.iter()).copied().collect();
        assert!((area(&all, &tris) - 84.0).abs() < 1e-9, "{}", area(&all, &tris));
    }

    #[test]
    fn concave_l() {
        let pts = [
            DVec2::new(0., 0.),
            DVec2::new(2., 0.),
            DVec2::new(2., 1.),
            DVec2::new(1., 1.),
            DVec2::new(1., 2.),
            DVec2::new(0., 2.),
        ];
        let tris = triangulate(&pts, &[]);
        assert_eq!(tris.len(), 4);
        assert!((area(&pts, &tris) - 3.0).abs() < 1e-9);
    }

    #[test]
    fn two_holes_clockwise_outer() {
        let outer = [DVec2::new(0., 0.), DVec2::new(0., 10.), DVec2::new(20., 10.), DVec2::new(20., 0.)];
        let h1 = [DVec2::new(2., 2.), DVec2::new(4., 2.), DVec2::new(4., 4.), DVec2::new(2., 4.)];
        let h2 = [DVec2::new(12., 2.), DVec2::new(14., 2.), DVec2::new(14., 4.), DVec2::new(12., 4.)];
        let tris = triangulate(&outer, &[&h1, &h2]);
        let all: Vec<DVec2> = outer.iter().chain(h1.iter()).chain(h2.iter()).copied().collect();
        assert!((area(&all, &tris) - 192.0).abs() < 1e-9);
    }

    #[test]
    fn circle_many_points() {
        let pts: Vec<DVec2> = (0..500)
            .map(|i| {
                let a = i as f64 / 500.0 * std::f64::consts::TAU;
                let r = if i % 2 == 0 { 10.0 } else { 8.0 };
                DVec2::new(a.cos() * r, a.sin() * r)
            })
            .collect();
        let tris = triangulate(&pts, &[]);
        assert_eq!(tris.len(), 498);
        assert!((area(&pts, &tris) - signed_area(&pts)).abs() < 1e-6);
    }
}
