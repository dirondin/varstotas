//! Bounding volume hierarchy over triangles for the path tracer: binned SAH build, flattened
//! into two `vec4` per node for the GPU (`pathtrace.wgsl`).
//!
//! Node layout: `[min.xyz, a]`, `[max.xyz, b]` with `a`, `b` bit-cast `u32`. Inner node:
//! `b = 0`, children at `a` and `a + 1`. Leaf: `b` = triangle count, triangles `a .. a + b` of
//! the reordered triangle list (`Bvh::order`).

/// Triangles per leaf at most (smaller leaves stop splitting earlier when SAH says so).
const MAX_LEAF: usize = 4;
const BINS: usize = 12;

#[derive(Clone, Copy, Debug)]
struct Aabb {
    min: [f32; 3],
    max: [f32; 3],
}

impl Aabb {
    const EMPTY: Self = Self { min: [f32::INFINITY; 3], max: [f32::NEG_INFINITY; 3] };

    fn grow(&mut self, p: [f32; 3]) {
        self.join(&Aabb { min: p, max: p });
    }

    #[allow(clippy::needless_range_loop)]
    fn join(&mut self, o: &Aabb) {
        for k in 0..3 {
            self.min[k] = self.min[k].min(o.min[k]);
            self.max[k] = self.max[k].max(o.max[k]);
        }
    }

    fn area(&self) -> f32 {
        let d = [self.max[0] - self.min[0], self.max[1] - self.min[1], self.max[2] - self.min[2]];
        if d[0] < 0.0 {
            return 0.0;
        }
        2.0 * (d[0] * d[1] + d[1] * d[2] + d[2] * d[0])
    }
}

/// The flattened tree: `nodes` (2 per node) and the triangle order its leaves refer to.
pub struct Bvh {
    pub nodes: Vec<[f32; 4]>,
    /// Leaf slot `i` holds original triangle `order[i]`.
    pub order: Vec<u32>,
}

struct Node {
    bounds: Aabb,
    first: usize,
    count: usize,
    left: usize,
}

/// Build over triangles given as three corners each.
#[allow(clippy::needless_range_loop)]
pub fn build(tris: &[[[f32; 3]; 3]]) -> Bvh {
    let n = tris.len();
    let mut boxes = Vec::with_capacity(n);
    let mut centers = Vec::with_capacity(n);
    for t in tris {
        let mut b = Aabb::EMPTY;
        for p in t {
            b.grow(*p);
        }
        centers.push([(b.min[0] + b.max[0]) * 0.5, (b.min[1] + b.max[1]) * 0.5, (b.min[2] + b.max[2]) * 0.5]);
        boxes.push(b);
    }
    let mut order: Vec<u32> = (0..n as u32).collect();
    let mut nodes: Vec<Node> = Vec::with_capacity(2 * n.max(1));
    let mut root = Node { bounds: Aabb::EMPTY, first: 0, count: n, left: 0 };
    for b in &boxes {
        root.bounds.join(b);
    }
    nodes.push(root);
    let mut stack = vec![0usize];
    while let Some(i) = stack.pop() {
        let (first, count) = (nodes[i].first, nodes[i].count);
        if count <= 2 {
            continue;
        }
        // Bins over the centroid bounds.
        let mut cb = Aabb::EMPTY;
        for &t in &order[first..first + count] {
            cb.grow(centers[t as usize]);
        }
        let mut best = (f32::INFINITY, 0usize, 0usize);
        for axis in 0..3 {
            let lo = cb.min[axis];
            let ext = cb.max[axis] - lo;
            if ext <= 1e-9 {
                continue;
            }
            let mut bins = [(Aabb::EMPTY, 0usize); BINS];
            for &t in &order[first..first + count] {
                let b = (((centers[t as usize][axis] - lo) / ext * BINS as f32) as usize).min(BINS - 1);
                bins[b].0.join(&boxes[t as usize]);
                bins[b].1 += 1;
            }
            let mut right_area = [0f32; BINS];
            let mut right_count = [0usize; BINS];
            let (mut rb, mut rc) = (Aabb::EMPTY, 0);
            for k in (1..BINS).rev() {
                rb.join(&bins[k].0);
                rc += bins[k].1;
                right_area[k] = rb.area();
                right_count[k] = rc;
            }
            let (mut lb, mut lc) = (Aabb::EMPTY, 0);
            for k in 0..BINS - 1 {
                lb.join(&bins[k].0);
                lc += bins[k].1;
                let cost = lb.area() * lc as f32 + right_area[k + 1] * right_count[k + 1] as f32;
                if lc > 0 && right_count[k + 1] > 0 && cost < best.0 {
                    best = (cost, axis, k + 1);
                }
            }
        }
        if !best.0.is_finite() {
            // All centroids equal: halve big leaves anyway.
            if count > MAX_LEAF {
                split_at(&mut nodes, &mut stack, &boxes, &order, i, count / 2);
            }
            continue;
        }
        if best.0 >= nodes[i].bounds.area() * count as f32 && count <= MAX_LEAF {
            continue;
        }
        let (axis, bin) = (best.1, best.2);
        let lo = cb.min[axis];
        let ext = cb.max[axis] - lo;
        let slice = &mut order[first..first + count];
        let mut l = 0;
        for k in 0..slice.len() {
            let t = slice[k] as usize;
            let b = (((centers[t][axis] - lo) / ext * BINS as f32) as usize).min(BINS - 1);
            if b < bin {
                slice.swap(k, l);
                l += 1;
            }
        }
        split_at(&mut nodes, &mut stack, &boxes, &order, i, l);
    }
    let mut flat = Vec::with_capacity(nodes.len() * 2);
    for nd in &nodes {
        let (a, b) = if nd.count > 0 { (nd.first as u32, nd.count as u32) } else { (nd.left as u32, 0) };
        let (mn, mx) = if nd.bounds.min[0] <= nd.bounds.max[0] { (nd.bounds.min, nd.bounds.max) } else { ([0.0; 3], [-1.0; 3]) };
        flat.push([mn[0], mn[1], mn[2], f32::from_bits(a)]);
        flat.push([mx[0], mx[1], mx[2], f32::from_bits(b)]);
    }
    Bvh { nodes: flat, order }
}

/// Turn node `i` into an inner node with children over `order[first .. first + l]` and the rest.
fn split_at(nodes: &mut Vec<Node>, stack: &mut Vec<usize>, boxes: &[Aabb], order: &[u32], i: usize, l: usize) {
    let (first, count) = (nodes[i].first, nodes[i].count);
    let bounds = |s: &[u32]| {
        let mut b = Aabb::EMPTY;
        for &t in s {
            b.join(&boxes[t as usize]);
        }
        b
    };
    let left = Node { bounds: bounds(&order[first..first + l]), first, count: l, left: 0 };
    let right = Node { bounds: bounds(&order[first + l..first + count]), first: first + l, count: count - l, left: 0 };
    let li = nodes.len();
    nodes.push(left);
    nodes.push(right);
    nodes[i].count = 0;
    nodes[i].left = li;
    stack.push(li);
    stack.push(li + 1);
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Closest hit by walking the flattened tree (the GPU's traversal in Rust).
    fn trace(bvh: &Bvh, tris: &[[[f32; 3]; 3]], o: [f32; 3], d: [f32; 3]) -> Option<(u32, f32)> {
        let mut best: Option<(u32, f32)> = None;
        let mut stack = vec![0u32];
        while let Some(n) = stack.pop() {
            let a = bvh.nodes[2 * n as usize];
            let b = bvh.nodes[2 * n as usize + 1];
            let (mut t0, mut t1) = (0f32, f32::INFINITY);
            for k in 0..3 {
                let inv = 1.0 / d[k];
                let (mut ta, mut tb) = ((a[k] - o[k]) * inv, (b[k] - o[k]) * inv);
                if ta > tb {
                    std::mem::swap(&mut ta, &mut tb);
                }
                t0 = t0.max(ta);
                t1 = t1.min(tb);
            }
            if t0 > t1 || best.is_some_and(|(_, t)| t0 > t) {
                continue;
            }
            let (first, count) = (a[3].to_bits(), b[3].to_bits());
            if count == 0 {
                stack.push(first);
                stack.push(first + 1);
                continue;
            }
            for slot in first..first + count {
                let ti = bvh.order[slot as usize];
                let [p0, p1, p2] = tris[ti as usize];
                let sub = |a: [f32; 3], b: [f32; 3]| [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
                let cross = |a: [f32; 3], b: [f32; 3]| [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]];
                let dot = |a: [f32; 3], b: [f32; 3]| a[0] * b[0] + a[1] * b[1] + a[2] * b[2];
                let (e1, e2) = (sub(p1, p0), sub(p2, p0));
                let pv = cross(d, e2);
                let det = dot(e1, pv);
                if det.abs() < 1e-12 {
                    continue;
                }
                let tv = sub(o, p0);
                let u = dot(tv, pv) / det;
                let qv = cross(tv, e1);
                let v = dot(d, qv) / det;
                let t = dot(e2, qv) / det;
                if u >= 0.0 && v >= 0.0 && u + v <= 1.0 && t > 0.0 && best.is_none_or(|(_, bt)| t < bt) {
                    best = Some((ti, t));
                }
            }
        }
        best
    }

    #[test]
    fn bvh_finds_the_nearest_triangle() {
        // A 20×20 grid of small quads at z = 0 and a big one at z = 5 over part of it.
        let mut tris = Vec::new();
        for i in 0..20 {
            for j in 0..20 {
                let (x, y) = (i as f32, j as f32);
                tris.push([[x, y, 0.0], [x + 1.0, y, 0.0], [x + 1.0, y + 1.0, 0.0]]);
                tris.push([[x, y, 0.0], [x + 1.0, y + 1.0, 0.0], [x, y + 1.0, 0.0]]);
            }
        }
        let roof = tris.len() as u32;
        tris.push([[0.0, 0.0, 5.0], [10.0, 0.0, 5.0], [0.0, 10.0, 5.0]]);
        let bvh = build(&tris);
        assert_eq!(bvh.order.len(), tris.len());
        let mut seen = bvh.order.clone();
        seen.sort();
        assert!(seen.iter().enumerate().all(|(i, &t)| t == i as u32));
        // Under the roof: the roof; elsewhere: the grid triangle under the point.
        assert_eq!(trace(&bvh, &tris, [2.2, 2.3, 10.0], [0.0, 0.0, -1.0]).map(|h| h.0), Some(roof));
        let (t, d) = trace(&bvh, &tris, [15.7, 12.2, 10.0], [0.0, 0.0, -1.0]).unwrap();
        assert_eq!((t, d), (2 * (15 * 20 + 12), 10.0));
        assert_eq!(trace(&bvh, &tris, [25.0, 2.0, 10.0], [0.0, 0.0, -1.0]), None);
        // Degenerate input: identical triangles still make a valid tree.
        let same = vec![[[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]]; 50];
        let b = build(&same);
        assert!(trace(&b, &same, [0.2, 0.2, 1.0], [0.0, 0.0, -1.0]).is_some());
        assert_eq!(build(&[]).nodes.len(), 2);
    }
}
