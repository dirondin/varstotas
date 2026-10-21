// ---------------------------------------------------------------------------
// Path tracer (track R batch 3, pathtrace.rs): progressive, one sample per pixel per dispatch.
// Uses the scene module's environment (group 0), textures and material table (group 1) and
// custom surfaces; its own data is group 2 (scene BVH, triangles, lights, accumulation) and
// group 3 (one denoise step).
// ---------------------------------------------------------------------------

struct PtU {
    eye: vec4<f32>,    // xyz: camera position (local frame); w: 1 = ortho
    right: vec4<f32>,  // xyz: right; w: tan of half the horizontal angle | half width (ortho)
    up: vec4<f32>,     // xyz: up; w: tan of half the vertical angle | half height (ortho)
    fwd: vec4<f32>,    // xyz: forward; w: lens radius (mm, 0 = pinhole)
    size: vec4<f32>,   // x, y: image size; z: sample index (0 = restart); w: bounces
    lens: vec4<f32>,   // x: focus distance (mm); y: cos of the sun's angular radius; z: lights; w: firefly clamp
    sun: vec4<f32>,    // xyz: direction toward the sun; w: strength (0 = none)
    misc: vec4<f32>,   // x: z of the local frame in the world (custom shaders); y: time; z: 1 = ground
    ground: vec4<f32>, // rgb: ground colour (linear); w: z of the world ground plane (local frame)
    grade: vec4<f32>,  // x: exposure (linear); y: tone curve (0 ACES, 1 AgX, 2 clip); z: saturation; w: contrast
    grade2: vec4<f32>, // rgb: white balance; w: vignette
    extra: vec4<f32>,  // x: 1 = transparent background (the sky and the ground plane seen directly)
};

@group(2) @binding(0) var<uniform> pt: PtU;
@group(2) @binding(1) var<storage, read> bvh: array<vec4<f32>>;
// Per triangle 3 vec4: [v0, front material], [v1 - v0, back material], [v2 - v0, layers].
@group(2) @binding(2) var<storage, read> tri_geo: array<vec4<f32>>;
// Per triangle 5 vec4: [n0, front colour], [n1, back colour], [n2, 0], [uv0, uv1], [uv2, 0, 0].
@group(2) @binding(3) var<storage, read> tri_attr: array<vec4<f32>>;
// Per light 4 vec4 (pathtrace.rs `light_records`).
@group(2) @binding(4) var<storage, read> lights: array<vec4<f32>>;
// Per pixel: sum of samples (rgb) and their count.
@group(2) @binding(5) var<storage, read_write> accum: array<vec4<f32>>;
// Per pixel 2 vec4: albedo of the first hit, [normal, distance].
@group(2) @binding(6) var<storage, read_write> aov: array<vec4<f32>>;
@group(2) @binding(7) var<storage, read> dn_in: array<vec4<f32>>;
@group(2) @binding(8) var<storage, read_write> dn_out: array<vec4<f32>>;
@group(2) @binding(9) var pt_out: texture_storage_2d<rgba8unorm, write>;

// x: step (px); y: 1 = read the accumulation instead of `dn_in`; z: colour sigma;
// w: 1 = the resolve shows the denoised buffer.
@group(3) @binding(0) var<uniform> dn: vec4<f32>;

const NO_HIT: u32 = 0xffffffffu;

var<private> rng_state: u32;
// The first hit of the current path: albedo, [normal, distance] (summed over the samples in
// `aov`, the denoisers' guides).
var<private> first_albedo: vec3<f32>;
var<private> first_nd: vec4<f32>;
// 0 when the camera ray found the background (summed in `aov`: the alpha of a transparent one).
var<private> first_cover: f32;

fn pcg(v: u32) -> u32 {
    let s = v * 747796405u + 2891336453u;
    let w = ((s >> ((s >> 28u) + 4u)) ^ s) * 277803737u;
    return (w >> 22u) ^ w;
}

fn rnd() -> f32 {
    rng_state = pcg(rng_state);
    return f32(rng_state >> 8u) / 16777216.0;
}

fn lum(c: vec3<f32>) -> f32 {
    return dot(c, vec3<f32>(0.2126, 0.7152, 0.0722));
}

/// `c` scaled down to luminance `k` at most (keeps its hue, unlike clamping each channel).
fn clamp_lum(c: vec3<f32>, k: f32) -> vec3<f32> {
    let l = max(c.r, max(c.g, c.b));
    return select(c, c * (k / l), l > k);
}

struct Hit {
    t: f32,
    tri: u32,
    u: f32,
    v: f32,
};

/// A textured side with a transparent texel at the hit is cut out (like the raster path).
fn cut_out(tri: u32, u: f32, v: f32, front: bool) -> bool {
    let layers = bitcast<u32>(tri_geo[3u * tri + 2u].w);
    let layer = select(layers >> 16u, layers & 0xffffu, front);
    if layer == 0u || tex_info[layer - 1u].z < 0.5 {
        return false;
    }
    let a3 = tri_attr[5u * tri + 3u];
    let a4 = tri_attr[5u * tri + 4u];
    let uv = a3.xy * (1.0 - u - v) + a3.zw * u + a4.xy * v;
    let li = i32(layer) - 1;
    return textureSampleLevel(face_tex, face_tex_sampler, uv * tex_info[li].xy, li, 0.0).a < 0.5;
}

/// Material of the side of `tri` a ray along `d` hits. Glass is a volume: from inside a glass
/// object (the back of a face whose front is glass) the glass material applies too.
fn side_material(tri: u32, d: vec3<f32>) -> vec2<u32> {
    let g0 = tri_geo[3u * tri];
    let g1 = tri_geo[3u * tri + 1u];
    let front = dot(cross(g1.xyz, tri_geo[3u * tri + 2u].xyz), d) < 0.0;
    let mf = bitcast<u32>(g0.w);
    let mb = bitcast<u32>(g1.w);
    if front {
        return vec2<u32>(mf, 1u);
    }
    if is_glass(mf) && !is_glass(mb) {
        return vec2<u32>(mf, 1u);
    }
    return vec2<u32>(mb, 0u);
}

fn is_glass(mat: u32) -> bool {
    return mat > 0u && materials[(mat - 1u) * 8u].z > 0.5 && env.p3.x < 0.5;
}

// ---- bvh traversal (pathtrace_rq.wgsl replaces it with ray queries) ----

/// Entry distance of the ray into node `n`'s box, or a huge value when it misses.
fn node_enter(n: u32, o: vec3<f32>, inv: vec3<f32>, tmax: f32) -> f32 {
    let a = bvh[2u * n].xyz;
    let b = bvh[2u * n + 1u].xyz;
    let t0 = (a - o) * inv;
    let t1 = (b - o) * inv;
    let tn = max(max(min(t0.x, t1.x), min(t0.y, t1.y)), max(min(t0.z, t1.z), 0.0));
    let tf = min(min(max(t0.x, t1.x), max(t0.y, t1.y)), min(max(t0.z, t1.z), tmax));
    return select(1e30, tn, tn <= tf);
}

/// Closest triangle along `o + d·t`, t in (0, tmax).
fn intersect(o: vec3<f32>, d: vec3<f32>, tmax: f32) -> Hit {
    var hit: Hit;
    hit.t = tmax;
    hit.tri = NO_HIT;
    let inv = 1.0 / select(d, vec3<f32>(1e-20), abs(d) < vec3<f32>(1e-20));
    var stack: array<u32, 64>;
    var sp = 1;
    stack[0] = 0u;
    if node_enter(0u, o, inv, tmax) > 1e29 {
        return hit;
    }
    while sp > 0 {
        sp -= 1;
        let n = stack[sp];
        let first = bitcast<u32>(bvh[2u * n].w);
        let count = bitcast<u32>(bvh[2u * n + 1u].w);
        if count == 0u {
            let ta = node_enter(first, o, inv, hit.t);
            let tb = node_enter(first + 1u, o, inv, hit.t);
            // Push the far child first so the near one is visited next.
            if ta < tb {
                if tb < 1e29 && sp < 64 { stack[sp] = first + 1u; sp += 1; }
                if ta < 1e29 && sp < 64 { stack[sp] = first; sp += 1; }
            } else {
                if ta < 1e29 && sp < 64 { stack[sp] = first; sp += 1; }
                if tb < 1e29 && sp < 64 { stack[sp] = first + 1u; sp += 1; }
            }
            continue;
        }
        for (var i = first; i < first + count; i++) {
            let g0 = tri_geo[3u * i];
            let e1 = tri_geo[3u * i + 1u].xyz;
            let e2 = tri_geo[3u * i + 2u].xyz;
            let pv = cross(d, e2);
            let det = dot(e1, pv);
            if abs(det) < 1e-12 {
                continue;
            }
            let id = 1.0 / det;
            let tv = o - g0.xyz;
            let u = dot(tv, pv) * id;
            if u < 0.0 || u > 1.0 {
                continue;
            }
            let qv = cross(tv, e1);
            let v = dot(d, qv) * id;
            if v < 0.0 || u + v > 1.0 {
                continue;
            }
            let t = dot(e2, qv) * id;
            if t > 1e-4 && t < hit.t && !cut_out(i, u, v, det > 0.0) {
                hit.t = t;
                hit.tri = i;
                hit.u = u;
                hit.v = v;
            }
        }
    }
    return hit;
}

/// Light passing from `o` along `d` for `tmax`: 0 when blocked; each glass face lets most of
/// it through. One traversal in any order that stops at the first opaque face.
fn transmittance(o: vec3<f32>, d: vec3<f32>, tmax: f32) -> f32 {
    let inv = 1.0 / select(d, vec3<f32>(1e-20), abs(d) < vec3<f32>(1e-20));
    var tr = 1.0;
    var stack: array<u32, 64>;
    var sp = 0;
    if node_enter(0u, o, inv, tmax) < 1e29 {
        stack[0] = 0u;
        sp = 1;
    }
    while sp > 0 {
        sp -= 1;
        let n = stack[sp];
        let first = bitcast<u32>(bvh[2u * n].w);
        let count = bitcast<u32>(bvh[2u * n + 1u].w);
        if count == 0u {
            if node_enter(first, o, inv, tmax) < 1e29 && sp < 64 { stack[sp] = first; sp += 1; }
            if node_enter(first + 1u, o, inv, tmax) < 1e29 && sp < 64 { stack[sp] = first + 1u; sp += 1; }
            continue;
        }
        for (var i = first; i < first + count; i++) {
            let g0 = tri_geo[3u * i];
            let e1 = tri_geo[3u * i + 1u].xyz;
            let e2 = tri_geo[3u * i + 2u].xyz;
            let pv = cross(d, e2);
            let det = dot(e1, pv);
            if abs(det) < 1e-12 {
                continue;
            }
            let id = 1.0 / det;
            let tv = o - g0.xyz;
            let u = dot(tv, pv) * id;
            if u < 0.0 || u > 1.0 {
                continue;
            }
            let qv = cross(tv, e1);
            let v = dot(d, qv) * id;
            if v < 0.0 || u + v > 1.0 {
                continue;
            }
            let t = dot(e2, qv) * id;
            if t > 1e-4 && t < tmax && !cut_out(i, u, v, det > 0.0) {
                if !is_glass(side_material(i, d).x) {
                    return 0.0;
                }
                tr *= 0.85;
            }
        }
    }
    return tr;
}

// ---- end bvh traversal ----

/// A direction around `n` with density cos/π.
fn sample_cosine(n: vec3<f32>) -> vec3<f32> {
    let r1 = rnd();
    let r2 = rnd();
    let phi = 6.2831853 * r1;
    let r = sqrt(r2);
    let t = tangent_of(n);
    let b = cross(n, t);
    return normalize(t * (r * cos(phi)) + b * (r * sin(phi)) + n * sqrt(max(0.0, 1.0 - r2)));
}

/// A GGX microfacet normal seen from `v` (visible normals, Heitz 2018); `a` = roughness².
fn sample_vndf(n: vec3<f32>, v: vec3<f32>, a: f32) -> vec3<f32> {
    let t = tangent_of(n);
    let b = cross(n, t);
    let vl = vec3<f32>(dot(v, t), dot(v, b), dot(v, n));
    let vh = normalize(vec3<f32>(a * vl.x, a * vl.y, vl.z));
    let lensq = vh.x * vh.x + vh.y * vh.y;
    var t1 = vec3<f32>(1.0, 0.0, 0.0);
    if lensq > 0.0 {
        t1 = vec3<f32>(-vh.y, vh.x, 0.0) / sqrt(lensq);
    }
    let t2 = cross(vh, t1);
    let r = sqrt(rnd());
    let phi = 6.2831853 * rnd();
    let p1 = r * cos(phi);
    var p2 = r * sin(phi);
    let s = 0.5 * (1.0 + vh.z);
    p2 = (1.0 - s) * sqrt(max(0.0, 1.0 - p1 * p1)) + s * p2;
    let nh = p1 * t1 + p2 * t2 + sqrt(max(0.0, 1.0 - p1 * p1 - p2 * p2)) * vh;
    let m = normalize(vec3<f32>(a * nh.x, a * nh.y, max(0.0, nh.z)));
    return normalize(t * m.x + b * m.y + n * m.z);
}

/// Smith masking of direction `w` for an anisotropic GGX (`ax` along `t`, `ay` along `b`).
fn smith_g1_aniso(w: vec3<f32>, n: vec3<f32>, t: vec3<f32>, b: vec3<f32>, ax: f32, ay: f32) -> f32 {
    let wn = max(dot(w, n), 1e-4);
    let wt = dot(w, t) * ax;
    let wb = dot(w, b) * ay;
    return 2.0 * wn / (wn + sqrt(wn * wn + wt * wt + wb * wb));
}

/// `sample_vndf` with separate roughness² along the tangent `t` (`ax`) and across it (`ay`).
fn sample_vndf_aniso(n: vec3<f32>, t: vec3<f32>, v: vec3<f32>, ax: f32, ay: f32) -> vec3<f32> {
    let b = cross(n, t);
    let vl = vec3<f32>(dot(v, t), dot(v, b), dot(v, n));
    let vh = normalize(vec3<f32>(ax * vl.x, ay * vl.y, vl.z));
    let lensq = vh.x * vh.x + vh.y * vh.y;
    var t1 = vec3<f32>(1.0, 0.0, 0.0);
    if lensq > 0.0 {
        t1 = vec3<f32>(-vh.y, vh.x, 0.0) / sqrt(lensq);
    }
    let t2 = cross(vh, t1);
    let r = sqrt(rnd());
    let phi = 6.2831853 * rnd();
    let p1 = r * cos(phi);
    var p2 = r * sin(phi);
    let s = 0.5 * (1.0 + vh.z);
    p2 = (1.0 - s) * sqrt(max(0.0, 1.0 - p1 * p1)) + s * p2;
    let nh = p1 * t1 + p2 * t2 + sqrt(max(0.0, 1.0 - p1 * p1 - p2 * p2)) * vh;
    let m = normalize(vec3<f32>(ax * nh.x, ay * nh.y, max(0.0, nh.z)));
    return normalize(t * m.x + b * m.y + n * m.z);
}

fn smith_g1(nx: f32, a: f32) -> f32 {
    let a2 = a * a;
    return 2.0 * nx / (nx + sqrt(a2 + (1.0 - a2) * nx * nx));
}

fn fresnel(f0: vec3<f32>, c: f32) -> vec3<f32> {
    return f0 + (vec3<f32>(1.0) - f0) * pow(1.0 - clamp(c, 0.0, 1.0), 5.0);
}

/// Fresnel reflectance of a dielectric; `eta` = n_from / n_to.
fn fresnel_dielectric(cos_i: f32, eta: f32) -> f32 {
    let s2 = eta * eta * (1.0 - cos_i * cos_i);
    if s2 >= 1.0 {
        return 1.0;
    }
    let ct = sqrt(1.0 - s2);
    let rs = (eta * cos_i - ct) / (eta * cos_i + ct);
    let rp = (cos_i - eta * ct) / (cos_i + eta * ct);
    return 0.5 * (rs * rs + rp * rp);
}

/// What a surface point looks like (after maps and custom shaders).
struct Mat {
    albedo: vec3<f32>,
    alpha: f32,
    metallic: f32,
    rough: f32,
    transmission: f32,
    ior: f32,
    emission: vec3<f32>,
    n: vec3<f32>,
    ng: vec3<f32>,
    /// The ray comes from the side the face's front looks to (enters a glass volume).
    front: bool,
    /// Clear coat (0..1), its roughness and normal (the surface before its normal map).
    coat: f32,
    coat_rough: f32,
    nc: vec3<f32>,
    /// Fabric sheen colour (strength and tint applied).
    sheen: vec3<f32>,
    /// Anisotropy (−1..1) along the tangent `t` (texture u).
    aniso: f32,
    t: vec3<f32>,
};

/// `sample_tiled` for the path tracer (the base level).
fn sample_tiled_l(tex: texture_2d_array<f32>, li: i32, q: vec2<f32>, at: f32) -> vec4<f32> {
    if at <= 0.0 {
        return textureSampleLevel(tex, face_tex_sampler, q, li, 0.0);
    }
    let t = tile_mix(q, at);
    let a = textureSampleLevel(tex, face_tex_sampler, q + t.a, li, 0.0);
    let b = textureSampleLevel(tex, face_tex_sampler, q + t.b, li, 0.0);
    return mix(a, b, t.w);
}

/// A shading normal turned away from the viewer (a bevel at a silhouette, a strong normal
/// map) is tilted back until it faces `v` a little: otherwise that spot renders black.
fn facing(n: vec3<f32>, v: vec3<f32>) -> vec3<f32> {
    let nv = dot(n, v);
    if nv >= 0.05 {
        return n;
    }
    return normalize(n + v * (0.05 - nv));
}

/// Rounded edges (the "bevel" shader): near a convex edge within `r` of `p` the normal bends
/// toward the face beyond it, reaching the bisector at the edge, like a fillet of radius `r`.
/// Four probe rays (a random turn each sample) come back toward `p` from beside it, just
/// under the surface: they hit the outside of a face beyond an edge.
fn bevel_normal(p: vec3<f32>, ng: vec3<f32>, r: f32) -> vec3<f32> {
    let t = tangent_of(ng);
    let b = cross(ng, t);
    let phi0 = 6.2831853 * rnd();
    // The two strongest edges (a corner has two).
    var w1 = 0.0;
    var n1 = ng;
    var w2 = 0.0;
    var n2 = ng;
    for (var k = 0u; k < 4u; k++) {
        let phi = phi0 + f32(k) * 1.5707963;
        let u = t * cos(phi) + b * sin(phi);
        let h = intersect(p + u * r - ng * (0.3 * r), -u, r);
        if h.tri == NO_HIT {
            continue;
        }
        let nh = normalize(cross(tri_geo[3u * h.tri + 1u].xyz, tri_geo[3u * h.tri + 2u].xyz));
        // Only the outside of a face that looks away from `p` (not the inside of the solid).
        let across = nh - ng * dot(nh, ng);
        let la = length(across);
        if dot(nh, u) <= 0.05 || la < 0.2 {
            continue;
        }
        let e = (r - h.t) * dot(u, across / la);
        if e < 0.0 || e >= r {
            continue;
        }
        let f = 1.0 - e / r;
        let w = 0.5 * f * f;
        if dot(nh, n1) > 0.95 || w1 == 0.0 {
            if w > w1 {
                w1 = w;
                n1 = nh;
            }
        } else if (dot(nh, n2) > 0.95 || w2 == 0.0) && w > w2 {
            w2 = w;
            n2 = nh;
        }
    }
    return normalize(ng + (n1 - ng) * w1 + (n2 - ng) * w2);
}

/// The surface of triangle `h.tri` hit by a ray along `d` at `p`.
fn surface_at(h: Hit, d: vec3<f32>, p: vec3<f32>) -> Mat {
    let i = h.tri;
    let g0 = tri_geo[3u * i];
    let g1 = tri_geo[3u * i + 1u];
    let g2 = tri_geo[3u * i + 2u];
    let a0 = tri_attr[5u * i];
    let a1 = tri_attr[5u * i + 1u];
    let a2 = tri_attr[5u * i + 2u];
    let a3 = tri_attr[5u * i + 3u];
    let a4 = tri_attr[5u * i + 4u];
    let w = 1.0 - h.u - h.v;
    var ng = normalize(cross(g1.xyz, g2.xyz));
    let front = dot(ng, d) < 0.0;
    var n = a0.xyz * w + a1.xyz * h.u + a2.xyz * h.v;
    n = select(ng, normalize(n), dot(n, n) > 1e-12);
    var cpack = bitcast<u32>(a0.w);
    let side = side_material(i, d);
    let mat = side.x;
    let layers = bitcast<u32>(g2.w);
    var layer = layers & 0xffffu;
    if !front {
        ng = -ng;
        n = -n;
    }
    if side.y == 0u {
        cpack = bitcast<u32>(a1.w);
        layer = layers >> 16u;
    }
    if dot(n, ng) < 0.0 {
        n = ng;
    }
    let cs = unpack4x8unorm(cpack);
    var c = vec4<f32>(srgb_to_linear(cs.rgb), cs.a);
    let uv = a3.xy * w + a3.zw * h.u + a4.xy * h.v;
    let clay = env.p3.x > 0.5;
    let mb = (max(mat, 1u) - 1u) * 8u;
    let pbr = mat > 0u && !clay;
    var m6 = vec4<f32>(0.0, 0.05, 0.0, 0.5);
    var m7 = vec4<f32>(0.0);
    if pbr {
        m6 = materials[mb + 6u];
        m7 = materials[mb + 7u];
    }
    let at = m7.y;
    if layer > 0u {
        let li = i32(layer) - 1;
        let k = tex_info[li].xy;
        c = vec4<f32>(sample_tiled_l(face_tex, li, uv * k, at).rgb, c.a);
    }
    // Tangent of the triangle's planar coordinates (u direction).
    var tg = tangent_of(n);
    let du1 = a3.zw - a3.xy;
    let du2 = a4.xy - a3.xy;
    let det = du1.x * du2.y - du1.y * du2.x;
    var bt = cross(n, tg);
    if abs(det) > 1e-12 {
        let t0 = (g1.xyz * du2.y - g2.xyz * du1.y) / det;
        let b0 = (g2.xyz * du1.x - g1.xyz * du2.x) / det;
        let t1 = t0 - n * dot(n, t0);
        let b1 = b0 - n * dot(n, b0);
        if dot(t1, t1) > 1e-20 && dot(b1, b1) > 1e-20 {
            tg = normalize(t1);
            bt = normalize(b1);
        }
    }
    // Rounded edges bend the surface itself (the clear coat follows them).
    let bevel = m7.z;
    if bevel > 0.0 {
        let nb = bevel_normal(p, ng, bevel);
        n = normalize(n + nb - ng);
    }
    let n_coat = n;
    var m: Mat;
    m.albedo = c.rgb;
    m.alpha = c.a;
    m.metallic = 0.0;
    m.rough = 0.6;
    m.transmission = 0.0;
    m.ior = 1.5;
    m.emission = vec3<f32>(0.0);
    if pbr {
        let b = mb;
        let m0 = materials[b];
        let m1 = materials[b + 1u];
        let m2 = materials[b + 2u];
        m.metallic = m0.x;
        m.rough = m0.y;
        m.transmission = m0.z;
        m.ior = max(m0.w, 1.0);
        m.emission = m1.rgb;
        // Glass faces carry a lowered alpha for the raster path (app `realistic_alpha`).
        m.alpha = min(c.a / max(1.0 - 0.9 * m0.z, 0.1), 1.0);
        let orm_layer = u32(m2.y + 0.5);
        if orm_layer > 0u {
            let li = i32(orm_layer) - 1;
            let orm = sample_tiled_l(data_tex, li, uv * data_info[li].xy, at).rgb;
            let mask = u32(m2.w + 0.5);
            if (mask & 2u) != 0u {
                m.rough *= orm.g;
            }
            if (mask & 4u) != 0u {
                m.metallic *= orm.b;
            }
        }
        let em_layer = u32(m2.x + 0.5);
        if em_layer > 0u {
            let li = i32(em_layer) - 1;
            m.emission *= sample_tiled_l(face_tex, li, uv * tex_info[li].xy, at).rgb;
        }
        let nl = u32(m2.z + 0.5);
        if nl > 0u {
            let li = i32(nl) - 1;
            let tn = sample_tiled_l(data_tex, li, uv * data_info[li].xy, at).rgb * 2.0 - 1.0;
            let pn = normalize(tg * tn.x * m1.w + bt * tn.y * m1.w + n * max(tn.z, 1e-3));
            if all(pn == pn) {
                n = pn;
            }
        }
        let id = u32(materials[b + 3u].x + 0.5);
        if id > 0u {
            var s: Surface;
            s.base = m.albedo;
            s.alpha = m.alpha;
            s.metallic = m.metallic;
            s.roughness = m.rough;
            s.transmission = m.transmission;
            s.emission = m.emission;
            s.normal = n;
            s.pos = p + vec3<f32>(0.0, 0.0, pt.misc.x);
            s.uv = uv;
            s.view = -d;
            s.time = pt.misc.y;
            s = custom_surface(id, s, materials[b + 4u], materials[b + 5u]);
            m.albedo = max(s.base, vec3<f32>(0.0));
            m.alpha = clamp(s.alpha, 0.0, 1.0);
            m.metallic = s.metallic;
            m.rough = s.roughness;
            m.transmission = clamp(s.transmission, 0.0, 1.0);
            m.emission = max(s.emission, vec3<f32>(0.0));
            if dot(s.normal, s.normal) > 1e-8 {
                n = normalize(s.normal);
            }
        }
    }
    if clay {
        m.albedo = vec3<f32>(0.62, 0.6, 0.57);
        m.alpha = select(c.a, 1.0, c.a > 0.3);
        m.metallic = 0.0;
        m.rough = 0.75;
        m.transmission = 0.0;
        m.emission = vec3<f32>(0.0);
    }
    m.rough = clamp(m.rough, 0.02, 1.0);
    m.metallic = clamp(m.metallic, 0.0, 1.0);
    if dot(n, ng) < 0.05 {
        n = normalize(n + ng * (0.05 - dot(n, ng)));
    }
    n = facing(n, -d);
    m.n = n;
    m.ng = ng;
    m.front = front;
    m.coat = clamp(m6.x, 0.0, 1.0);
    m.coat_rough = clamp(m6.y, 0.02, 1.0);
    m.nc = facing(n_coat, -d);
    let tint = m.albedo / max(max(m.albedo.r, max(m.albedo.g, m.albedo.b)), 1e-3);
    m.sheen = clamp(m6.z, 0.0, 1.0) * mix(vec3<f32>(1.0), tint, clamp(m6.w, 0.0, 1.0)) * (1.0 - m.metallic);
    m.aniso = clamp(m7.x, -1.0, 1.0);
    m.t = normalize(tg - n * dot(n, tg));
    return m;
}

/// Diffuse + GGX reflection of an opaque surface for light from `l` seen from `v`, times cos.
fn eval_opaque(m: Mat, v: vec3<f32>, l: vec3<f32>) -> vec3<f32> {
    let nl = dot(m.n, l);
    let nv = max(dot(m.n, v), 1e-4);
    if dot(m.ng, l) <= 0.0 {
        return vec3<f32>(0.0);
    }
    let h = normalize(v + l);
    let vh = max(dot(v, h), 0.0);
    var out = vec3<f32>(0.0);
    // The clear coat reflects part of the light; the rest reaches the surface under it.
    var base_k = 1.0;
    if m.coat > 0.0 {
        let nvc = max(dot(m.nc, v), 1e-4);
        base_k = 1.0 - m.coat * (0.04 + 0.96 * pow(1.0 - nvc, 5.0));
        let nlc = dot(m.nc, l);
        if nlc > 0.0 {
            let ac = m.coat_rough * m.coat_rough;
            let ac2 = ac * ac;
            let hc = max(dot(m.nc, h), 0.0);
            let ddc = hc * hc * (ac2 - 1.0) + 1.0;
            let fc = 0.04 + 0.96 * pow(1.0 - vh, 5.0);
            let spec_c = ac2 / (PI * ddc * ddc) * smith_g1(nlc, ac) * smith_g1(nvc, ac) * fc / (4.0 * nvc * nlc);
            out += vec3<f32>(m.coat * spec_c * nlc);
        }
    }
    if nl <= 0.0 {
        return out;
    }
    let f0 = mix(vec3<f32>(0.04), m.albedo, m.metallic);
    let a = m.rough * m.rough;
    var spec_dg = 0.0;
    if m.aniso != 0.0 {
        let t = m.t;
        let b = cross(m.n, t);
        let ax = max(a * (1.0 + m.aniso), 1e-3);
        let ay = max(a * (1.0 - m.aniso), 1e-3);
        let x = dot(t, h) / ax;
        let y = dot(b, h) / ay;
        let z = dot(m.n, h);
        let dd = x * x + y * y + z * z;
        let dist = 1.0 / (PI * ax * ay * dd * dd);
        spec_dg = dist * smith_g1_aniso(l, m.n, t, b, ax, ay) * smith_g1_aniso(v, m.n, t, b, ax, ay);
    } else {
        let nh = max(dot(m.n, h), 0.0);
        let a2 = a * a;
        let dd = nh * nh * (a2 - 1.0) + 1.0;
        spec_dg = a2 / (PI * dd * dd) * smith_g1(nl, a) * smith_g1(nv, a);
    }
    let f = fresnel(f0, vh);
    let spec = spec_dg * f / (4.0 * nv * nl);
    let diff = m.albedo * (1.0 - m.metallic) * (1.0 - m.transmission) / PI * (vec3<f32>(1.0) - f);
    // Sheen (Burley): a rim at grazing angles, on top of the diffuse.
    let sheen = m.sheen * pow(1.0 - vh, 5.0);
    return out + ((diff + spec) * base_k + sheen) * nl;
}

/// Light arriving straight from the sun and the light objects at `p`, reflected toward `v`.
fn direct_light(m: Mat, p: vec3<f32>, v: vec3<f32>) -> vec3<f32> {
    var col = vec3<f32>(0.0);
    let o = p + m.ng * 0.5;
    if pt.sun.w > 0.0 {
        // A direction in the sun's disc.
        let ct = 1.0 - rnd() * (1.0 - pt.lens.y);
        let st = sqrt(max(0.0, 1.0 - ct * ct));
        let phi = 6.2831853 * rnd();
        let t = tangent_of(pt.sun.xyz);
        let b = cross(pt.sun.xyz, t);
        let l = normalize(pt.sun.xyz * ct + (t * cos(phi) + b * sin(phi)) * st);
        let f = eval_opaque(m, v, l);
        if any(f > vec3<f32>(0.0)) {
            col += f * (PI * pt.sun.w) * transmittance(o, l, 1e9);
        }
    }
    // One light, picked by a guess of its share of the light here (intensity / distance², facing).
    let count = u32(pt.lens.z);
    if count == 0u {
        return col;
    }
    var total = 0.0;
    for (var j = 0u; j < count; j++) {
        total += light_weight(j, o, m.n);
    }
    var pick = rnd() * total;
    var i = count - 1u;
    var w = 0.0;
    for (var j = 0u; j < count; j++) {
        w = light_weight(j, o, m.n);
        pick -= w;
        if pick < 0.0 {
            i = j;
            break;
        }
    }
    if i == count - 1u {
        w = light_weight(i, o, m.n);
    }
    let prob = w / max(total, 1e-30);
    if prob <= 0.0 {
        return col;
    }
    {
        let l0 = lights[4u * i];
        let l1 = lights[4u * i + 1u];
        let l2 = lights[4u * i + 2u];
        let l3 = lights[4u * i + 3u];
        var q = l0.xyz;
        var k = 1.0;
        if l0.w > 1.5 {
            // Area: a point on the rectangle; it shines toward its direction only.
            let vv = normalize(cross(l1.xyz, l3.xyz)) * l1.w;
            q += l3.xyz * (2.0 * rnd() - 1.0) + vv * (2.0 * rnd() - 1.0);
        } else if l1.w > 0.0 {
            // A point in the bulb's sphere (soft shadows).
            let z = 2.0 * rnd() - 1.0;
            let a = 6.2831853 * rnd();
            let r = sqrt(max(0.0, 1.0 - z * z));
            q += vec3<f32>(r * cos(a), r * sin(a), z) * l1.w * pow(rnd(), 1.0 / 3.0);
        }
        let to = q - o;
        let d2 = max(dot(to, to), 1.0);
        let dist = sqrt(d2);
        let l = to / dist;
        if l0.w > 1.5 {
            k = max(dot(l1.xyz, -l), 0.0);
        } else if l0.w > 0.5 {
            k = smoothstep(l2.w, l3.w, dot(l1.xyz, -l));
        }
        let f = eval_opaque(m, v, l);
        if k > 0.0 && any(f > vec3<f32>(0.0)) {
            col += f * l2.rgb * (k / (d2 * prob)) * transmittance(o, l, dist - 1.0);
        }
    }
    return col;
}

/// How much light `i` may give a point `o` with normal `n` (relative; never 0 for a light that
/// can reach it, so picking by it stays unbiased).
fn light_weight(i: u32, o: vec3<f32>, n: vec3<f32>) -> f32 {
    let l0 = lights[4u * i];
    let l1 = lights[4u * i + 1u];
    let l2 = lights[4u * i + 2u];
    let l3 = lights[4u * i + 3u];
    let to = l0.xyz - o;
    var size = l1.w;
    if l0.w > 1.5 {
        size = max(l1.w, length(l3.xyz));
    }
    let d2 = max(dot(to, to), max(size * size, 1.0));
    let l = to * inverseSqrt(max(dot(to, to), 1e-6));
    var k = 0.05 + max(dot(n, l), 0.0);
    if l0.w > 1.5 {
        k *= 0.05 + max(dot(l1.xyz, -l), 0.0);
    } else if l0.w > 0.5 {
        k *= 0.02 + smoothstep(l2.w, l3.w, dot(l1.xyz, -l));
    }
    return lum(l2.rgb) * k / d2;
}

/// One path through pixel `px`.
fn trace_path(px: vec2<f32>) -> vec3<f32> {
    // Camera ray through a jittered point of the pixel.
    let jitter = vec2<f32>(rnd(), rnd());
    let ndc = vec2<f32>((px.x + jitter.x) / pt.size.x * 2.0 - 1.0, 1.0 - (px.y + jitter.y) / pt.size.y * 2.0);
    var o = pt.eye.xyz;
    var d: vec3<f32>;
    if pt.eye.w > 0.5 {
        o += pt.right.xyz * (ndc.x * pt.right.w) + pt.up.xyz * (ndc.y * pt.up.w);
        d = pt.fwd.xyz;
    } else {
        d = normalize(pt.fwd.xyz + pt.right.xyz * (ndc.x * pt.right.w) + pt.up.xyz * (ndc.y * pt.up.w));
        if pt.fwd.w > 0.0 {
            // Thin lens: start on the aperture, aim at the point in focus.
            let focus = o + d * (pt.lens.x / max(dot(d, pt.fwd.xyz), 1e-4));
            let r = sqrt(rnd()) * pt.fwd.w;
            let a = 6.2831853 * rnd();
            o += pt.right.xyz * (r * cos(a)) + pt.up.xyz * (r * sin(a));
            d = normalize(focus - o);
        }
    }
    var col = vec3<f32>(0.0);
    var thr = vec3<f32>(1.0);
    let bounces = u32(pt.size.w);
    var b = 0u;
    var passes = 0u;
    // After a diffuse bounce glossy surfaces are rougher (path regularization): the caustics
    // that a few samples would only show as fireflies become soft.
    var blur = 0.0;
    loop {
        let h = intersect(o, d, 1e9);
        // The ground plane under the model (its world z = 0), when shown.
        var tg = 1e30;
        if pt.misc.z > 0.5 && d.z < -1e-6 && o.z > pt.ground.w {
            tg = (pt.ground.w - o.z) / d.z;
        }
        if h.tri == NO_HIT && tg > 1e29 {
            if b == 0u {
                first_albedo = vec3<f32>(1.0);
                first_nd = vec4<f32>(-d, 1e9);
                first_cover = 0.0;
                if pt.extra.x > 0.5 {
                    break;
                }
            }
            col += thr * env_radiance(d, 0.0);
            break;
        }
        // A transparent background: the ground plane seen directly is background too.
        if b == 0u && tg < h.t && pt.extra.x > 0.5 {
            first_albedo = vec3<f32>(1.0);
            first_nd = vec4<f32>(-d, 1e9);
            first_cover = 0.0;
            break;
        }
        var m: Mat;
        var t = h.t;
        if tg < h.t {
            t = tg;
            m.albedo = pt.ground.rgb;
            m.alpha = 1.0;
            m.metallic = 0.0;
            m.rough = 0.9;
            m.transmission = 0.0;
            m.ior = 1.5;
            m.emission = vec3<f32>(0.0);
            m.n = vec3<f32>(0.0, 0.0, 1.0);
            m.ng = m.n;
            m.front = true;
            m.coat = 0.0;
            m.coat_rough = 1.0;
            m.nc = m.n;
            m.sheen = vec3<f32>(0.0);
            m.aniso = 0.0;
            m.t = vec3<f32>(1.0, 0.0, 0.0);
        }
        let p = o + d * t;
        if tg >= h.t {
            m = surface_at(h, d, p);
        }
        m.rough = max(m.rough, blur);
        let v = -d;
        // Partly transparent (not glass): passes straight through with probability 1 - alpha.
        if m.alpha < 0.999 && m.transmission < 0.01 && rnd() > m.alpha {
            o = p + d * 0.5;
            passes += 1u;
            if passes > 16u {
                break;
            }
            continue;
        }
        if b == 0u {
            first_albedo = max(m.albedo, vec3<f32>(0.02));
            first_nd = vec4<f32>(m.n, t);
        }
        var e = thr * m.emission;
        if b > 0u {
            // Emitters seen after a diffuse bounce are mostly lamp bulbs, whose light the lamp's
            // light object already brings (with shadow rays): at most a bright surface here,
            // or the few paths that find a bulb become fireflies.
            e = clamp_lum(e, select(pt.lens.w, 1.0, blur > 0.0));
        }
        col += e;
        if b >= bounces {
            break;
        }
        if rnd() < m.transmission {
            // Glass: reflect or refract on a microfacet.
            let entering = m.front;
            var nn = m.n;
            let eta = select(m.ior, 1.0 / m.ior, entering);
            var mn = nn;
            if m.rough > 0.03 {
                mn = sample_vndf(nn, v, m.rough * m.rough);
            }
            let ci = max(dot(v, mn), 1e-4);
            let fr = fresnel_dielectric(ci, eta);
            if rnd() < fr {
                d = reflect(d, mn);
                o = p + m.ng * 0.5;
            } else {
                d = refract(d, mn, eta);
                if dot(d, d) < 0.5 {
                    d = reflect(-v, mn);
                    o = p + m.ng * 0.5;
                } else {
                    o = p - m.ng * 0.5;
                    thr *= m.albedo;
                }
            }
        } else {
            let direct = thr * direct_light(m, p, v);
            col += select(clamp_lum(direct, pt.lens.w), direct, b == 0u);
            // Next direction: the clear coat, the specular or the diffuse lobe.
            let nv = max(dot(m.n, v), 1e-4);
            let f0 = mix(vec3<f32>(0.04), m.albedo, m.metallic);
            let fv = fresnel(f0, nv);
            let diff = m.albedo * (1.0 - m.metallic) * (1.0 - m.transmission);
            let ps = select(clamp(lum(fv) / (lum(fv) + lum(diff + m.sheen) * (1.0 - lum(fv)) + 1e-4), 0.1, 0.9), 1.0, lum(diff + m.sheen) < 1e-4);
            var fc = 0.0;
            var pc = 0.0;
            if m.coat > 0.0 {
                fc = m.coat * (0.04 + 0.96 * pow(1.0 - max(dot(m.nc, v), 1e-4), 5.0));
                pc = clamp(fc * 3.0 + 0.05, 0.05, 0.5);
            }
            if rnd() < pc {
                let ac = m.coat_rough * m.coat_rough;
                let mn = sample_vndf(m.nc, v, ac);
                let l = reflect(d, mn);
                let nl = dot(m.nc, l);
                if nl <= 0.0 || dot(m.ng, l) <= 0.0 {
                    break;
                }
                thr *= m.coat * (0.04 + 0.96 * pow(1.0 - clamp(dot(v, mn), 0.0, 1.0), 5.0)) * smith_g1(nl, ac) / pc;
                d = l;
            } else {
                thr *= (1.0 - fc) / (1.0 - pc);
                if rnd() < ps {
                    let a = m.rough * m.rough;
                    var mn: vec3<f32>;
                    var ax = a;
                    var ay = a;
                    let bt = cross(m.n, m.t);
                    if m.aniso != 0.0 {
                        ax = max(a * (1.0 + m.aniso), 1e-3);
                        ay = max(a * (1.0 - m.aniso), 1e-3);
                        mn = sample_vndf_aniso(m.n, m.t, v, ax, ay);
                    } else {
                        mn = sample_vndf(m.n, v, a);
                    }
                    let l = reflect(d, mn);
                    let nl = dot(m.n, l);
                    if nl <= 0.0 || dot(m.ng, l) <= 0.0 {
                        break;
                    }
                    var g1 = smith_g1(nl, a);
                    if m.aniso != 0.0 {
                        g1 = smith_g1_aniso(l, m.n, m.t, bt, ax, ay);
                    }
                    thr *= fresnel(f0, dot(v, mn)) * g1 / ps;
                    d = l;
                } else {
                    let l = sample_cosine(m.n);
                    if dot(m.ng, l) <= 0.0 {
                        break;
                    }
                    let lh = max(dot(l, normalize(v + l)), 0.0);
                    thr *= (diff * (vec3<f32>(1.0) - fv) + m.sheen * pow(1.0 - lh, 5.0) * PI) / (1.0 - ps);
                    d = l;
                    blur = 0.35;
                }
            }
            o = p + m.ng * 0.5;
        }
        b += 1u;
        // Russian roulette after a few bounces.
        if b > 3u {
            let q = clamp(max(thr.r, max(thr.g, thr.b)), 0.05, 1.0);
            if rnd() > q {
                break;
            }
            thr /= q;
        }
    }
    return col;
}

@compute @workgroup_size(8, 8)
fn pt_trace(@builtin(global_invocation_id) gid: vec3<u32>) {
    let w = u32(pt.size.x);
    let h = u32(pt.size.y);
    if gid.x >= w || gid.y >= h {
        return;
    }
    let pix = gid.y * w + gid.x;
    let frame = u32(pt.size.z);
    // misc.w samples per dispatch.
    let n = max(u32(pt.misc.w), 1u);
    var sum = vec3<f32>(0.0);
    var sa = vec3<f32>(0.0);
    var sn = vec4<f32>(0.0);
    var cover = 0.0;
    for (var k = 0u; k < n; k++) {
        rng_state = pcg(pix * 9781u + pcg((frame + k) * 6271u + 17u));
        first_albedo = vec3<f32>(1.0);
        first_nd = vec4<f32>(0.0, 0.0, 1.0, 1e9);
        first_cover = 1.0;
        let c = trace_path(vec2<f32>(gid.xy));
        if all(c == c) {
            sum += c;
        }
        sa += first_albedo;
        sn += first_nd;
        cover += first_cover;
    }
    if frame == 0u {
        accum[pix] = vec4<f32>(sum, f32(n));
        aov[2u * pix] = vec4<f32>(sa, cover);
        aov[2u * pix + 1u] = sn;
    } else {
        accum[pix] += vec4<f32>(sum, f32(n));
        aov[2u * pix] += vec4<f32>(sa, cover);
        aov[2u * pix + 1u] += sn;
    }
}

/// Mean first-hit albedo of a pixel.
fn aov_albedo(pix: u32) -> vec3<f32> {
    return max(aov[2u * pix].rgb / max(accum[pix].w, 1.0), vec3<f32>(0.02));
}

/// Mean first-hit normal and distance of a pixel.
fn aov_nd(pix: u32) -> vec4<f32> {
    let a = aov[2u * pix + 1u] / max(accum[pix].w, 1.0);
    let l = length(a.xyz);
    return vec4<f32>(select(vec3<f32>(0.0, 0.0, 1.0), a.xyz / l, l > 1e-6), a.w);
}

/// Mean of the samples so far, divided by the albedo (the denoiser smooths light, not texture).
fn demodulated(pix: u32) -> vec3<f32> {
    let a = accum[pix];
    return a.rgb / max(a.w, 1.0) / aov_albedo(pix);
}

/// One à-trous step (edge-avoiding wavelet filter, guided by normals, distance and albedo).
@compute @workgroup_size(8, 8)
fn pt_denoise(@builtin(global_invocation_id) gid: vec3<u32>) {
    let w = i32(pt.size.x);
    let h = i32(pt.size.y);
    if i32(gid.x) >= w || i32(gid.y) >= h {
        return;
    }
    let step = i32(dn.x);
    let pix = gid.y * u32(w) + gid.x;
    let from_accum = dn.y > 0.5;
    var c0 = select(dn_in[pix].rgb, demodulated(pix), from_accum);
    let n0 = aov_nd(pix);
    let a0 = aov_albedo(pix);
    let kern = array<f32, 3>(0.375, 0.25, 0.0625);
    var sum = vec3<f32>(0.0);
    var wsum = 0.0;
    for (var j = -2; j <= 2; j++) {
        for (var i = -2; i <= 2; i++) {
            let q = vec2<i32>(i32(gid.x) + i * step, i32(gid.y) + j * step);
            if q.x < 0 || q.y < 0 || q.x >= w || q.y >= h {
                continue;
            }
            let qi = u32(q.y * w + q.x);
            let c = select(dn_in[qi].rgb, demodulated(qi), from_accum);
            let nq = aov_nd(qi);
            let aq = aov_albedo(qi);
            let dc = c - c0;
            let wc = exp(-dot(dc, dc) / (dn.z * dn.z * (lum(c0) + lum(c) + 0.05)));
            let wn = pow(max(dot(n0.xyz, nq.xyz), 0.0), 64.0);
            let wz = exp(-abs(n0.w - nq.w) / (0.02 * n0.w + 1.0));
            let da = aq - a0;
            let wa = exp(-dot(da, da) * 40.0);
            let k = kern[abs(i)] * kern[abs(j)] * wc * wn * wz * wa;
            sum += c * k;
            wsum += k;
        }
    }
    dn_out[pix] = vec4<f32>(sum / max(wsum, 1e-6), 1.0);
}

/// The image to show / save: exposure, tone map, sRGB.
@compute @workgroup_size(8, 8)
fn pt_resolve(@builtin(global_invocation_id) gid: vec3<u32>) {
    let w = u32(pt.size.x);
    if gid.x >= w || gid.y >= u32(pt.size.y) {
        return;
    }
    let pix = gid.y * w + gid.x;
    let px = pt_pixel(pix);
    var c = max(px.rgb, vec3<f32>(0.0)) * pt.grade2.rgb;
    var mapped = pt_tone(c);
    let luma = dot(mapped, vec3<f32>(0.2126, 0.7152, 0.0722));
    mapped = max(mix(vec3<f32>(luma), mapped, pt.grade.z), vec3<f32>(0.0));
    var s = linear_to_srgb(clamp(mapped, vec3<f32>(0.0), vec3<f32>(1.0)));
    s = (s - vec3<f32>(0.5)) * pt.grade.w + vec3<f32>(0.5);
    // Vignette: darker towards the corners (by the distance over the half diagonal).
    let uv = (vec2<f32>(gid.xy) + 0.5) / pt.size.xy * 2.0 - 1.0;
    let r2 = dot(uv, uv) * 0.5;
    s *= 1.0 - pt.grade2.w * r2 * (0.6 + 0.4 * r2);
    textureStore(pt_out, vec2<i32>(gid.xy), vec4<f32>(clamp(s, vec3<f32>(0.0), vec3<f32>(1.0)), px.a));
}

/// Mean light of a pixel (denoised or not) with the exposure, and its alpha (1 unless the sky
/// is transparent; the light is then divided by it).
fn pt_pixel(pix: u32) -> vec4<f32> {
    var c: vec3<f32>;
    if dn.w > 0.5 {
        c = dn_in[pix].rgb * aov_albedo(pix);
    } else {
        let a = accum[pix];
        c = a.rgb / max(a.w, 1.0);
    }
    var alpha = 1.0;
    if pt.extra.x > 0.5 {
        alpha = clamp(aov[2u * pix].w / max(accum[pix].w, 1.0), 0.0, 1.0);
        c = c / max(alpha, 1e-3);
    }
    return vec4<f32>(c * pt.grade.x, alpha);
}

fn pt_tone(x: vec3<f32>) -> vec3<f32> {
    if pt.grade.y < 0.5 {
        return aces(x);
    }
    if pt.grade.y < 1.5 {
        return agx(x);
    }
    return clamp(x, vec3<f32>(0.0), vec3<f32>(1.0));
}

/// Linear light of a pixel for saving (EXR): the denoised or the accumulated mean.
@compute @workgroup_size(8, 8)
fn pt_linear(@builtin(global_invocation_id) gid: vec3<u32>) {
    let w = u32(pt.size.x);
    if gid.x >= w || gid.y >= u32(pt.size.y) {
        return;
    }
    let pix = gid.y * w + gid.x;
    let px = pt_pixel(pix);
    dn_out[pix] = vec4<f32>(px.rgb * pt.grade2.rgb * px.a, px.a);
}
