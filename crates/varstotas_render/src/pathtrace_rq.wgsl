// Ray queries in place of the BVH traversal of pathtrace.wgsl (pathtrace.rs `rq_source`), when
// the device has hardware / driver ray tracing (wgpu EXPERIMENTAL_RAY_QUERY: Metal, Vulkan,
// DX12). Two instances: the opaque triangles and the ones the shader has to look at (glass,
// cut-out textures); the `bvh` buffer maps instance offset + primitive to the triangle record
// (4 per vec4).

@group(2) @binding(10) var pt_accel: acceleration_structure;

fn rq_triangle(c: RayIntersection) -> u32 {
    let k = c.instance_custom_data + c.primitive_index;
    return bitcast<u32>(bvh[k / 4u][k % 4u]);
}

/// The ray along `d` meets the front of triangle `tri`.
fn rq_front(tri: u32, d: vec3<f32>) -> bool {
    return dot(cross(tri_geo[3u * tri + 1u].xyz, tri_geo[3u * tri + 2u].xyz), d) < 0.0;
}

/// Closest triangle along `o + d·t`, t in (0, tmax).
fn intersect(o: vec3<f32>, d: vec3<f32>, tmax: f32) -> Hit {
    var hit: Hit;
    hit.t = tmax;
    hit.tri = NO_HIT;
    var rq: ray_query;
    rayQueryInitialize(&rq, pt_accel, RayDesc(RAY_FLAG_NONE, 0xffu, 1e-4, tmax, o, d));
    while rayQueryProceed(&rq) {
        let c = rayQueryGetCandidateIntersection(&rq);
        let tri = rq_triangle(c);
        if !cut_out(tri, c.barycentrics.x, c.barycentrics.y, rq_front(tri, d)) {
            rayQueryConfirmIntersection(&rq);
        }
    }
    let h = rayQueryGetCommittedIntersection(&rq);
    if h.kind != RAY_QUERY_INTERSECTION_NONE {
        hit.t = h.t;
        hit.tri = rq_triangle(h);
        hit.u = h.barycentrics.x;
        hit.v = h.barycentrics.y;
    }
    return hit;
}

/// Light passing from `o` along `d` for `tmax`: 0 when blocked; each glass face lets most of
/// it through.
fn transmittance(o: vec3<f32>, d: vec3<f32>, tmax: f32) -> f32 {
    var tr = 1.0;
    var rq: ray_query;
    rayQueryInitialize(&rq, pt_accel, RayDesc(RAY_FLAG_TERMINATE_ON_FIRST_HIT, 0xffu, 1e-4, tmax, o, d));
    while rayQueryProceed(&rq) {
        let c = rayQueryGetCandidateIntersection(&rq);
        let tri = rq_triangle(c);
        if cut_out(tri, c.barycentrics.x, c.barycentrics.y, rq_front(tri, d)) {
            continue;
        }
        if is_glass(side_material(tri, d).x) {
            tr *= 0.85;
            continue;
        }
        rayQueryConfirmIntersection(&rq);
    }
    if rayQueryGetCommittedIntersection(&rq).kind != RAY_QUERY_INTERSECTION_NONE {
        return 0.0;
    }
    return tr;
}

