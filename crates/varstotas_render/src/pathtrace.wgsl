// ---------------------------------------------------------------------------
// Path tracer (track Р batch 3, pathtrace.rs): progressive, one sample per pixel per dispatch.
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
    return mat > 0u && materials[(mat - 1u) * 6u].z > 0.5 && env.p3.x < 0.5;
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
};

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
    if layer > 0u {
        let li = i32(layer) - 1;
        let k = tex_info[li].xy;
        c = vec4<f32>(textureSampleLevel(face_tex, face_tex_sampler, uv * k, li, 0.0).rgb, c.a);
    }
    var m: Mat;
    m.albedo = c.rgb;
    m.alpha = c.a;
    m.metallic = 0.0;
    m.rough = 0.6;
    m.transmission = 0.0;
    m.ior = 1.5;
    m.emission = vec3<f32>(0.0);
    let clay = env.p3.x > 0.5;
    if mat > 0u && !clay {
        let b = (mat - 1u) * 6u;
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
            let orm = textureSampleLevel(data_tex, face_tex_sampler, uv * data_info[li].xy, li, 0.0).rgb;
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
            m.emission *= textureSampleLevel(face_tex, face_tex_sampler, uv * tex_info[li].xy, li, 0.0).rgb;
        }
        let nl = u32(m2.z + 0.5);
        if nl > 0u {
            // Tangent frame of the triangle's planar coordinates.
            let li = i32(nl) - 1;
            let tn = textureSampleLevel(data_tex, face_tex_sampler, uv * data_info[li].xy, li, 0.0).rgb * 2.0 - 1.0;
            let du1 = a3.zw - a3.xy;
            let du2 = a4.xy - a3.xy;
            let det = du1.x * du2.y - du1.y * du2.x;
            if abs(det) > 1e-12 {
                var t = (g1.xyz * du2.y - g2.xyz * du1.y) / det;
                var bt = (g2.xyz * du1.x - g1.xyz * du2.x) / det;
                t = normalize(t - n * dot(n, t));
                bt = normalize(bt - n * dot(n, bt));
                let pn = normalize(t * tn.x * m1.w + bt * tn.y * m1.w + n * max(tn.z, 1e-3));
                if all(pn == pn) {
                    n = pn;
                }
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
    m.n = n;
    m.ng = ng;
    m.front = front;
    return m;
}

/// Diffuse + GGX reflection of an opaque surface for light from `l` seen from `v`, times cos.
fn eval_opaque(m: Mat, v: vec3<f32>, l: vec3<f32>) -> vec3<f32> {
    let nl = dot(m.n, l);
    let nv = max(dot(m.n, v), 1e-4);
    if nl <= 0.0 || dot(m.ng, l) <= 0.0 {
        return vec3<f32>(0.0);
    }
    let f0 = mix(vec3<f32>(0.04), m.albedo, m.metallic);
    let h = normalize(v + l);
    let nh = max(dot(m.n, h), 0.0);
    let a = m.rough * m.rough;
    let a2 = a * a;
    let dd = nh * nh * (a2 - 1.0) + 1.0;
    let dist = a2 / (PI * dd * dd);
    let f = fresnel(f0, dot(v, h));
    let spec = dist * smith_g1(nl, a) * smith_g1(nv, a) * f / (4.0 * nv * nl);
    let diff = m.albedo * (1.0 - m.metallic) * (1.0 - m.transmission) / PI * (vec3<f32>(1.0) - f);
    return (diff + spec) * nl;
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
            col += thr * env_radiance(d, 0.0);
            if b == 0u {
                first_albedo = vec3<f32>(1.0);
                first_nd = vec4<f32>(-d, 1e9);
            }
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
            // Next direction: the specular or the diffuse lobe.
            let nv = max(dot(m.n, v), 1e-4);
            let f0 = mix(vec3<f32>(0.04), m.albedo, m.metallic);
            let fv = fresnel(f0, nv);
            let diff = m.albedo * (1.0 - m.metallic) * (1.0 - m.transmission);
            let ps = select(clamp(lum(fv) / (lum(fv) + lum(diff) * (1.0 - lum(fv)) + 1e-4), 0.1, 0.9), 1.0, lum(diff) < 1e-4);
            if rnd() < ps {
                let a = m.rough * m.rough;
                let mn = sample_vndf(m.n, v, a);
                let l = reflect(d, mn);
                let nl = dot(m.n, l);
                if nl <= 0.0 || dot(m.ng, l) <= 0.0 {
                    break;
                }
                thr *= fresnel(f0, dot(v, mn)) * smith_g1(nl, a) / ps;
                d = l;
            } else {
                let l = sample_cosine(m.n);
                if dot(m.ng, l) <= 0.0 {
                    break;
                }
                thr *= diff * (vec3<f32>(1.0) - fv) / (1.0 - ps);
                d = l;
                blur = 0.35;
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
    for (var k = 0u; k < n; k++) {
        rng_state = pcg(pix * 9781u + pcg((frame + k) * 6271u + 17u));
        first_albedo = vec3<f32>(1.0);
        first_nd = vec4<f32>(0.0, 0.0, 1.0, 1e9);
        let c = trace_path(vec2<f32>(gid.xy));
        if all(c == c) {
            sum += c;
        }
        sa += first_albedo;
        sn += first_nd;
    }
    if frame == 0u {
        accum[pix] = vec4<f32>(sum, f32(n));
        aov[2u * pix] = vec4<f32>(sa, 0.0);
        aov[2u * pix + 1u] = sn;
    } else {
        accum[pix] += vec4<f32>(sum, f32(n));
        aov[2u * pix] += vec4<f32>(sa, 0.0);
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
    var c: vec3<f32>;
    if dn.w > 0.5 {
        c = dn_in[pix].rgb * aov_albedo(pix);
    } else {
        let a = accum[pix];
        c = a.rgb / max(a.w, 1.0);
    }
    let mapped = tone_curve(max(c, vec3<f32>(0.0)) * env.p2.x);
    textureStore(pt_out, vec2<i32>(gid.xy), vec4<f32>(linear_to_srgb(clamp(mapped, vec3<f32>(0.0), vec3<f32>(1.0))), 1.0));
}

/// Linear light of a pixel for saving (EXR): the denoised or the accumulated mean.
@compute @workgroup_size(8, 8)
fn pt_linear(@builtin(global_invocation_id) gid: vec3<u32>) {
    let w = u32(pt.size.x);
    if gid.x >= w || gid.y >= u32(pt.size.y) {
        return;
    }
    let pix = gid.y * w + gid.x;
    if dn.w > 0.5 {
        dn_out[pix] = vec4<f32>(dn_in[pix].rgb * aov_albedo(pix) * env.p2.x, 1.0);
    } else {
        let a = accum[pix];
        dn_out[pix] = vec4<f32>(a.rgb / max(a.w, 1.0) * env.p2.x, 1.0);
    }
}
