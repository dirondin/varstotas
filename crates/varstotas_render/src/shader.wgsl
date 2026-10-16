// Varstotas scene shaders. Reverse-Z (depth 1 = near, 0 = far), compare GreaterEqual.
// All shading happens in linear space; vertex colors arrive sRGB-encoded.

struct Globals {
    view_proj: mat4x4<f32>,   // world-relative-to-`origin` -> clip
    origin: vec4<f32>,        // xyz: render origin (≈ eye) in the geometry's local frame; w: local frame world z
    eye_rel: vec4<f32>,       // xyz: eye - origin; w: 1 = ortho
    right: vec4<f32>,         // xyz: camera right; w: tan_half_x (persp) | half width (ortho)
    up: vec4<f32>,            // xyz: camera up;    w: tan_half_y (persp) | half height (ortho)
    fwd: vec4<f32>,           // xyz: camera forward
    light: vec4<f32>,         // xyz: unit direction toward the sun (world); w: ambient
    viewport: vec4<f32>,      // x,y: size in px; z: pixels_per_point; w: profile width (px)
    sky_top: vec4<f32>,       // linear
    sky_horizon: vec4<f32>,
    ground: vec4<f32>,
    edge_color: vec4<f32>,
    mono_front: vec4<f32>,
    mono_back: vec4<f32>,
    hidden_face: vec4<f32>,
    params: vec4<f32>,        // x: face mode (0 shaded, 1 mono, 2 hidden-line, 3 wireframe tint,
                              //    4 realistic); y: alpha mul;
                              // z: line bias relative; w: 1 = encode gamma manually
    params2: vec4<f32>,       // x: line bias absolute (ndc); y: ground enabled;
                              // z: highlight hide mask (bit 0 Selected, bit 1 Dimmed)
    hl_selected: vec4<f32>,   // rgb: selection color (sRGB); w: face mix
    hl_dim_face: vec4<f32>,   // rgb: dimmed-face target (sRGB); w: face mix
    hl_dim_edge: vec4<f32>,   // dimmed edge color (linear)
    tile: vec4<f32>,          // tile NDC = full NDC * xy + zw (export tiling; (1,1,0,0) = whole image)
    section: vec4<f32>,       // active section cut, local frame: keep dot(xyz, p) >= w; off = (0,0,0,-1)
    shadow_vp: mat4x4<f32>,   // local frame -> shadow map clip (xy -1..1, z 0..1)
    shadow: vec4<f32>,        // x: faces receive; y: ground receives; z: light shade; w: dark shade
    shadow2: vec4<f32>,       // x: receiver normal offset; y: 1 / map size; z: 1 = sun up; w: 1 = sun shading
    grid_o: vec4<f32>,        // xyz: grid origin relative to the render origin; w: finest step
    grid_u: vec4<f32>,        // xyz: u axis; w: major ratio
    grid_v: vec4<f32>,        // xyz: v axis; w: 1 = shown
    grid_color: vec4<f32>,    // linear rgb, a = strength of major lines
};

// Highlight states (Rust `Highlight`): 0 none, 1 selected, 2 dimmed (outside the editing context).
const HL_SELECTED: u32 = 1u;
const HL_DIMMED: u32 = 2u;

/// Highlight state after the export filter (`params2.z`): image export can hide the selection
/// (and optionally the editing-context dimming) without touching the state buffers.
fn hl_state(s: u32) -> u32 {
    let mask = u32(g.params2.z + 0.5); // bit 0: hide Selected, bit 1: hide Dimmed
    if (s == HL_SELECTED && (mask & 1u) != 0u) || (s == HL_DIMMED && (mask & 2u) != 0u) {
        return 0u;
    }
    return s;
}

/// Face color (linear) with the highlight mix done in sRGB, like the app used to bake it.
fn highlight_face(c: vec4<f32>, state: u32) -> vec4<f32> {
    if state == 0u {
        return c;
    }
    var t = g.hl_selected;
    if state != HL_SELECTED {
        t = g.hl_dim_face;
    }
    let m = mix(linear_to_srgb(clamp(c.rgb, vec3<f32>(0.0), vec3<f32>(1.0))), t.rgb, t.w);
    return vec4<f32>(srgb_to_linear(m), mix(c.a, 1.0, t.w));
}

/// Edge / profile color (linear) for a highlight state.
fn highlight_edge(c: vec4<f32>, state: u32) -> vec4<f32> {
    if state == HL_SELECTED {
        return vec4<f32>(srgb_to_linear(g.hl_selected.rgb), 1.0);
    }
    if state == HL_DIMMED {
        return g.hl_dim_edge;
    }
    return c;
}

@group(0) @binding(0) var<uniform> g: Globals;
@group(0) @binding(1) var shadow_map: texture_depth_2d;
@group(0) @binding(2) var shadow_sampler: sampler_comparison;

// Environment of the Realistic style (see env.rs): equirect panorama with mips + SH irradiance.
struct EnvU {
    sh: array<vec4<f32>, 9>, // irradiance per SH basis function (rgb)
    p: vec4<f32>,            // x: rotation (rad); y: intensity; z: last mip; w: 1 = background
    p2: vec4<f32>,           // x: exposure multiplier; y: tone map (0 ACES, 1 AgX, 2 clamp); z: sun;
                             // w: time (s, custom shaders)
    p3: vec4<f32>,           // x: 1 = clay; y: soft shadow penumbra per shadow-map depth (0 = off);
                             // z: 1 = high quality
};

// Scene pass of the effects path (post.rs): light stays linear (exposure applied, no tone
// curve, no manual gamma); the final post pass maps it.
override HDR_OUT: bool = false;
@group(0) @binding(3) var env_map: texture_2d<f32>;
@group(0) @binding(4) var env_sampler: sampler;
@group(0) @binding(5) var<uniform> env: EnvU;

const PI: f32 = 3.14159265;

fn realistic() -> bool {
    return g.params.x > 3.5;
}

/// World direction -> environment direction (the panorama turned by `env.p.x`).
fn env_dir(d: vec3<f32>) -> vec3<f32> {
    let c = cos(env.p.x);
    let s = sin(env.p.x);
    return vec3<f32>(c * d.x + s * d.y, -s * d.x + c * d.y, d.z);
}

/// Radiance of the environment toward -d (d = direction looking out), mip `lod`.
fn env_radiance(d: vec3<f32>, lod: f32) -> vec3<f32> {
    let e = env_dir(normalize(d));
    let u = (atan2(e.y, e.x) + PI) / (2.0 * PI);
    let v = acos(clamp(e.z, -1.0, 1.0)) / PI;
    return textureSampleLevel(env_map, env_sampler, vec2<f32>(u, v), lod).rgb * env.p.y;
}

/// Diffuse irradiance arriving at a surface with normal `n`.
fn env_irradiance(n: vec3<f32>) -> vec3<f32> {
    let d = env_dir(n);
    let x = d.x;
    let y = d.y;
    let z = d.z;
    var e = env.sh[0].rgb * 0.282095;
    e += env.sh[1].rgb * (0.488603 * y) + env.sh[2].rgb * (0.488603 * z) + env.sh[3].rgb * (0.488603 * x);
    e += env.sh[4].rgb * (1.092548 * x * y) + env.sh[5].rgb * (1.092548 * y * z);
    e += env.sh[6].rgb * (0.315392 * (3.0 * z * z - 1.0)) + env.sh[7].rgb * (1.092548 * x * z);
    e += env.sh[8].rgb * (0.546274 * (x * x - y * y));
    return max(e, vec3<f32>(0.0)) * env.p.y;
}

fn aces(x: vec3<f32>) -> vec3<f32> {
    // Narkowicz 2015 fit
    let a = 2.51;
    let b = 0.03;
    let c = 2.43;
    let d = 0.59;
    let e = 0.14;
    return clamp((x * (a * x + b)) / (x * (c * x + d) + e), vec3<f32>(0.0), vec3<f32>(1.0));
}

fn agx(x: vec3<f32>) -> vec3<f32> {
    // Minimal AgX (Benjamin Wrensch): inset, log2 encoding, polynomial contrast, outset; linear out.
    let inset = mat3x3<f32>(
        vec3<f32>(0.842479062253094, 0.0423282422610123, 0.0423756549057051),
        vec3<f32>(0.0784335999999992, 0.878468636469772, 0.0784336),
        vec3<f32>(0.0792237451477643, 0.0791661274605434, 0.879142973793104));
    let outset = mat3x3<f32>(
        vec3<f32>(1.19687900512017, -0.0528968517574562, -0.0529716355144438),
        vec3<f32>(-0.0980208811401368, 1.15190312990417, -0.0980434501171241),
        vec3<f32>(-0.0990297440797205, -0.0989611768448433, 1.15107367264116));
    let min_ev = -12.47393;
    let max_ev = 4.026069;
    var v = inset * max(x, vec3<f32>(1e-10));
    v = clamp(log2(v), vec3<f32>(min_ev), vec3<f32>(max_ev));
    v = (v - min_ev) / (max_ev - min_ev);
    let v2 = v * v;
    let v4 = v2 * v2;
    v = 15.5 * v4 * v2 - 40.14 * v4 * v + 31.96 * v4 - 6.868 * v2 * v + 0.4298 * v2 + 0.1191 * v - 0.00232;
    v = outset * v;
    return pow(clamp(v, vec3<f32>(0.0), vec3<f32>(1.0)), vec3<f32>(2.2));
}

/// Scene light (linear) -> display (linear 0..1): exposure, then the tone map (only the
/// exposure in the effects path, see `HDR_OUT`).
fn tonemap(c: vec3<f32>) -> vec3<f32> {
    let x = max(c, vec3<f32>(0.0)) * env.p2.x;
    if HDR_OUT {
        return min(x, vec3<f32>(60000.0));
    }
    return tone_curve(x);
}

/// The tone map alone (exposure already applied).
fn tone_curve(x: vec3<f32>) -> vec3<f32> {
    if env.p2.y < 0.5 {
        return aces(x);
    }
    if env.p2.y < 1.5 {
        return agx(x);
    }
    return clamp(x, vec3<f32>(0.0), vec3<f32>(1.0));
}

/// Fraction of sunlight reaching local-frame point `p` with normal `n` (1 = lit), 3×3 PCF.
/// Points outside the map are lit; points beyond its far plane compare as far.
fn shadow_vis(p: vec3<f32>, n: vec3<f32>) -> f32 {
    let c = g.shadow_vp * vec4<f32>(p + n * g.shadow2.x, 1.0);
    let uv = vec2<f32>(c.x * 0.5 + 0.5, 0.5 - c.y * 0.5);
    if c.z < 0.0 || any(uv < vec2<f32>(0.0)) || any(uv > vec2<f32>(1.0)) {
        return 1.0;
    }
    let z = min(c.z, 1.0);
    if realistic() && env.p3.y > 0.0 {
        return shadow_pcss(uv, z, p);
    }
    var sum = 0.0;
    for (var i = -1; i <= 1; i++) {
        for (var j = -1; j <= 1; j++) {
            let o = vec2<f32>(f32(i), f32(j)) * g.shadow2.y;
            sum += textureSampleCompareLevel(shadow_map, shadow_sampler, uv + o, z);
        }
    }
    return sum / 9.0;
}

const POISSON: array<vec2<f32>, 16> = array<vec2<f32>, 16>(
    vec2<f32>(-0.94201624, -0.39906216), vec2<f32>(0.94558609, -0.76890725),
    vec2<f32>(-0.09418410, -0.92938870), vec2<f32>(0.34495938, 0.29387760),
    vec2<f32>(-0.91588581, 0.45771432), vec2<f32>(-0.81544232, -0.87912464),
    vec2<f32>(-0.38277543, 0.27676845), vec2<f32>(0.97484398, 0.75648379),
    vec2<f32>(0.44323325, -0.97511554), vec2<f32>(0.53742981, -0.47373420),
    vec2<f32>(-0.26496911, -0.41893023), vec2<f32>(0.79197514, 0.19090188),
    vec2<f32>(-0.24188840, 0.99706507), vec2<f32>(-0.81409955, 0.91437590),
    vec2<f32>(0.19984126, 0.78641367), vec2<f32>(0.14383161, -0.14100790));

/// Interleaved gradient noise of a position (0..1), stable per surface point.
fn ign(p: vec2<f32>) -> f32 {
    return fract(52.9829189 * fract(dot(p, vec2<f32>(0.06711056, 0.00583715))));
}

/// Percentage-closer soft shadows: the average blocker depth sets the filter width, so the
/// penumbra grows with the distance between the occluder and the receiver.
/// The fragment's pixel (set by the fragment shaders that sample soft shadows): the per-pixel
/// rotation of the PCSS kernel.
var<private> frag_px: vec2<f32>;

fn shadow_pcss(uv: vec2<f32>, z: f32, p: vec3<f32>) -> f32 {
    let texel = g.shadow2.y;
    let dims = vec2<f32>(textureDimensions(shadow_map));
    let k = env.p3.y;
    let a = ign(frag_px) * 6.2831853;
    let rot = mat2x2<f32>(cos(a), sin(a), -sin(a), cos(a));
    let search = clamp(k * z, 2.0 * texel, 48.0 * texel);
    var blockers = 0.0;
    var sum = 0.0;
    for (var i = 0; i < 16; i++) {
        let t = clamp(uv + rot * POISSON[i] * search, vec2<f32>(0.0), vec2<f32>(0.9999));
        let d = textureLoad(shadow_map, vec2<i32>(t * dims), 0);
        if d < z - 1.5 * texel {
            blockers += 1.0;
            sum += d;
        }
    }
    if blockers < 0.5 {
        return 1.0;
    }
    let width = clamp((z - sum / blockers) * k, 1.0 * texel, 48.0 * texel);
    var lit = 0.0;
    for (var i = 0; i < 16; i++) {
        lit += textureSampleCompareLevel(shadow_map, shadow_sampler, uv + rot * POISSON[i] * width, z);
    }
    return lit / 16.0;
}

// Shadow pass (group 0 unused, so the map can be bound elsewhere in the frame).
struct ShadowU {
    vp: mat4x4<f32>,
    section: vec4<f32>,
};
@group(1) @binding(0) var<uniform> shadow_u: ShadowU;

struct ShadowOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) clip: f32,
};

@vertex
fn vs_shadow(@location(0) pos: vec3<f32>) -> ShadowOut {
    var o: ShadowOut;
    o.pos = shadow_u.vp * vec4<f32>(pos, 1.0);
    o.clip = dot(shadow_u.section.xyz, pos) - shadow_u.section.w;
    return o;
}

@fragment
fn fs_shadow(in: ShadowOut) {
    if in.clip < 0.0 {
        discard;
    }
}

fn srgb_to_linear(c: vec3<f32>) -> vec3<f32> {
    let lo = c / 12.92;
    let hi = pow((c + 0.055) / 1.055, vec3<f32>(2.4));
    return select(hi, lo, c <= vec3<f32>(0.04045));
}

fn linear_to_srgb(c: vec3<f32>) -> vec3<f32> {
    let lo = c * 12.92;
    let hi = 1.055 * pow(c, vec3<f32>(1.0 / 2.4)) - 0.055;
    return select(hi, lo, c <= vec3<f32>(0.0031308));
}

fn lin(c: vec4<f32>) -> vec4<f32> {
    return vec4<f32>(srgb_to_linear(clamp(c.rgb, vec3<f32>(0.0), vec3<f32>(1.0))), c.a);
}

fn out_color(c: vec4<f32>) -> vec4<f32> {
    if HDR_OUT {
        return c;
    }
    if g.params.w > 0.5 {
        return vec4<f32>(linear_to_srgb(clamp(c.rgb, vec3<f32>(0.0), vec3<f32>(1.0))), c.a);
    }
    return c;
}

fn project(p_world: vec3<f32>) -> vec4<f32> {
    return g.view_proj * vec4<f32>(p_world - g.origin.xyz, 1.0);
}

/// Signed distance of a local-frame point to the section plane (< 0 = cut away).
fn sec_dist(p: vec3<f32>) -> f32 {
    return dot(g.section.xyz, p) - g.section.w;
}

/// Pull toward the camera (reverse-Z: larger depth = nearer).
fn bias_clip(c: vec4<f32>) -> vec4<f32> {
    var r = c;
    r.z = c.z * (1.0 + g.params.z) + g.params2.x * c.w;
    return r;
}

// ---------------------------------------------------------------------------
// Full-screen: sky + ground
// ---------------------------------------------------------------------------

struct FsOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) ndc: vec2<f32>,
};

@vertex
fn vs_fullscreen(@builtin(vertex_index) vi: u32) -> FsOut {
    let x = f32((vi << 1u) & 2u) * 2.0 - 1.0;
    let y = f32(vi & 2u) * 2.0 - 1.0;
    var o: FsOut;
    o.pos = vec4<f32>(x, y, 0.0, 1.0);
    // Tile NDC -> NDC of the whole image (sky / ground rays are defined on the whole image).
    o.ndc = (vec2<f32>(x, y) - g.tile.zw) / g.tile.xy;
    return o;
}

struct Ray {
    o: vec3<f32>, // relative to origin
    d: vec3<f32>,
};

fn pixel_ray(ndc: vec2<f32>) -> Ray {
    var r: Ray;
    if g.eye_rel.w > 0.5 {
        r.o = g.eye_rel.xyz + g.right.xyz * (ndc.x * g.right.w) + g.up.xyz * (ndc.y * g.up.w);
        r.d = g.fwd.xyz;
    } else {
        r.o = g.eye_rel.xyz;
        r.d = normalize(g.fwd.xyz + g.right.xyz * (ndc.x * g.right.w) + g.up.xyz * (ndc.y * g.up.w));
    }
    return r;
}

@fragment
fn fs_sky(in: FsOut) -> @location(0) vec4<f32> {
    if realistic() && env.p.w > 0.5 {
        let r = pixel_ray(in.ndc);
        return out_color(vec4<f32>(tonemap(env_radiance(r.d, 0.0)), 1.0));
    }
    var t: f32;
    if g.eye_rel.w > 0.5 {
        t = clamp(in.ndc.y * 0.5 + 0.5, 0.0, 1.0);
        // Parallel side views look exactly along the ground plane, so the ground pass sees no
        // intersection: paint everything below world z = 0 with the ground colour (QA2-008).
        let r = pixel_ray(in.ndc);
        let world_z = r.o.z + g.origin.z + g.origin.w;
        if g.params2.y > 0.5 && r.d.z >= -1e-6 && world_z < 0.0 {
            let sky = mix(g.sky_horizon.rgb, g.sky_top.rgb, t);
            return out_color(vec4<f32>(mix(sky, g.ground.rgb, g.ground.a), 1.0));
        }
    } else {
        let r = pixel_ray(in.ndc);
        t = clamp(r.d.z / 0.6, 0.0, 1.0);
        t = sqrt(t);
        // Eye exactly on the ground plane (standard side views): the ground pass cannot
        // intersect, so paint the lower half with the ground colour as background (QA2-008).
        let eye_z = g.eye_rel.z + g.origin.z + g.origin.w;
        if g.params2.y > 0.5 && r.d.z < -1e-6 && eye_z <= 1e-3 && eye_z > -1.0 {
            let sky = mix(g.sky_horizon.rgb, g.sky_top.rgb, t);
            let a = g.ground.a * smoothstep(0.0, 0.035, -r.d.z);
            return out_color(vec4<f32>(mix(sky, g.ground.rgb, a), 1.0));
        }
    }
    let c = mix(g.sky_horizon.rgb, g.sky_top.rgb, t);
    return out_color(vec4<f32>(c, 1.0));
}

struct GroundOut {
    @location(0) color: vec4<f32>,
    /// Effects path only (post.rs): surface normal, roughness, metalness.
    @location(1) gbuf: vec4<f32>,
    @builtin(frag_depth) depth: f32,
};

/// G-buffer texel of the effects path (Rgba8Unorm): octahedral world normal, roughness
/// (never 0: 0 marks the background) and metalness.
fn gbuf_pack(n: vec3<f32>, rough: f32, metal: f32) -> vec4<f32> {
    var o = n.xy / (abs(n.x) + abs(n.y) + abs(n.z));
    if n.z < 0.0 {
        o = (1.0 - abs(o.yx)) * select(vec2<f32>(-1.0), vec2<f32>(1.0), o >= vec2<f32>(0.0));
    }
    return vec4<f32>(o * 0.5 + 0.5, clamp(rough, 0.02, 1.0), clamp(metal, 0.0, 1.0));
}

@fragment
fn fs_ground(in: FsOut) -> GroundOut {
    frag_px = in.pos.xy;
    let r = pixel_ray(in.ndc);
    let ortho = g.eye_rel.w > 0.5;
    // origin.w = world z of the geometry's local frame (SceneGeometry::origin.z).
    let plane_z = -(g.origin.z + g.origin.w); // world z = 0 in relative coords
    let eye_world_z = g.eye_rel.z + g.origin.z + g.origin.w;
    // Only visible from above.
    var valid = r.d.z < -1e-6 && (ortho || eye_world_z > 1e-3);
    var t = 0.0;
    if valid {
        t = (plane_z - r.o.z) / r.d.z;
    }
    valid = valid && (ortho || t > 0.0);
    let hit = r.o + r.d * t;
    let clip = g.view_proj * vec4<f32>(hit, 1.0);
    let depth_raw = clip.z / clip.w;
    // Push the ground behind anything coplanar with z = 0 (faces there must look exactly as
    // above ground; QA-004). Covers the faces' own slope bias via the screen-space depth
    // derivative, plus a relative (perspective) / absolute (ortho) margin.
    // Derivatives are taken before any discard (uniform control flow).
    let slope = fwidth(depth_raw);
    if !valid || depth_raw > 1.0 {
        discard;
    }
    let margin = depth_raw * max(5.0 * g.params.z, 1e-6) + 6.0 * g.params2.x + 3.0 * slope;
    let depth = clamp(depth_raw - margin, 0.0, 1.0);
    var alpha = g.ground.a;
    if !ortho {
        // fade toward the horizon
        alpha *= smoothstep(0.0, 0.035, -r.d.z);
    }
    var col = g.ground.rgb;
    if realistic() {
        // A diffuse plane lit by the environment and the sun.
        var vis = 1.0;
        if g.shadow.y > 0.5 && g.shadow2.z > 0.5 {
            vis = shadow_vis(hit + g.origin.xyz, vec3<f32>(0.0, 0.0, 1.0));
        }
        let sun = env.p2.z * PI * max(g.light.z, 0.0) * vis;
        col = tonemap(col / PI * (env_irradiance(vec3<f32>(0.0, 0.0, 1.0)) + sun));
        var o: GroundOut;
        o.color = out_color(vec4<f32>(col, alpha));
        o.gbuf = gbuf_pack(vec3<f32>(0.0, 0.0, 1.0), 1.0, 0.0);
        o.depth = depth;
        return o;
    }
    if g.shadow.y > 0.5 && g.shadow2.z > 0.5 {
        // Ground shadows: darker and more opaque where the sun is blocked.
        let shade = 1.0 - shadow_vis(hit + g.origin.xyz, vec3<f32>(0.0, 0.0, 1.0));
        let k = mix(1.0, pow(max(g.shadow.w / max(g.shadow.z, 0.01), 0.2), 2.2), shade);
        col *= k;
        alpha = mix(alpha, max(alpha, 0.95 * smoothstep(0.0, 0.035, -r.d.z)), shade);
    }
    var o: GroundOut;
    o.color = out_color(vec4<f32>(col, alpha));
    o.gbuf = gbuf_pack(vec3<f32>(0.0, 0.0, 1.0), 1.0, 0.0);
    o.depth = depth;
    return o;
}

// ---------------------------------------------------------------------------
// Grid: lines on the plane (grid_o, grid_u, grid_v); levels step * ratio^k chosen per pixel
// (crate::grid::GridStyle::level), the finer level fades out as the footprint grows.
// ---------------------------------------------------------------------------

/// Line coverage (0..1) of lines every `step` along both plane coordinates.
fn grid_lines(c: vec2<f32>, w: vec2<f32>, step: f32) -> f32 {
    let q = c / step;
    let fw = max(w / step, vec2<f32>(1e-6));
    let d = abs(fract(q - 0.5) - 0.5) / fw;
    // ~1 px wide, antialiased.
    return 1.0 - clamp(min(d.x, d.y) - 0.25, 0.0, 1.0);
}

@fragment
fn fs_grid(in: FsOut) -> GroundOut {
    let r = pixel_ray(in.ndc);
    let n = normalize(cross(g.grid_u.xyz, g.grid_v.xyz));
    let dn = dot(r.d, n);
    let ortho = g.eye_rel.w > 0.5;
    var t = 0.0;
    if abs(dn) > 1e-7 {
        t = dot(g.grid_o.xyz - r.o, n) / dn;
    }
    let valid = abs(dn) > 1e-7 && (ortho || t > 0.0);
    let hit = r.o + r.d * t;
    let rel = hit - g.grid_o.xyz;
    let c = vec2<f32>(dot(rel, g.grid_u.xyz), dot(rel, g.grid_v.xyz));
    let w = fwidth(c);
    let clip = g.view_proj * vec4<f32>(hit, 1.0);
    let depth_raw = clip.z / clip.w;
    let slope = fwidth(depth_raw);
    if !valid || depth_raw > 1.0 || depth_raw < 0.0 {
        discard;
    }
    let ratio = max(g.grid_u.w, 2.0);
    let step = max(g.grid_o.w, 1e-6);
    let pixel = max(max(w.x, w.y), 1e-9);
    let l = log(max(pixel * 40.0 / step, 1e-12)) / log(ratio); // GRID_MIN_PX
    var k = 0.0;
    var f = 0.0;
    if l > 0.0 {
        k = floor(l);
        f = l - k;
    }
    let s0 = step * pow(ratio, k);
    let a0 = grid_lines(c, w, s0) * 0.35 * (1.0 - f);
    let a1 = grid_lines(c, w, s0 * ratio) * mix(0.8, 0.35, f);
    let a2 = grid_lines(c, w, s0 * ratio * ratio) * 0.8;
    var alpha = max(max(a0, a1), a2) * g.grid_color.a;
    // Fade toward the horizon / when the plane turns edge-on.
    alpha *= smoothstep(0.02, 0.12, abs(dn));
    if alpha < 0.003 {
        discard;
    }
    let margin = depth_raw * max(5.0 * g.params.z, 1e-6) + 6.0 * g.params2.x + 3.0 * slope;
    var o: GroundOut;
    o.color = out_color(vec4<f32>(g.grid_color.rgb, alpha));
    o.depth = clamp(depth_raw - margin, 0.0, 1.0);
    return o;
}

// ---------------------------------------------------------------------------
// Faces
// ---------------------------------------------------------------------------

struct FaceIn {
    @location(0) pos: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) color: vec4<f32>,
    @location(3) back: vec4<f32>,
    @location(4) state: u32,
    @location(5) uv: vec2<f32>,
    @location(6) layers: vec2<u32>,
    @location(7) mats: vec2<u32>,
};

struct FaceOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) normal: vec3<f32>,
    @location(1) color: vec4<f32>,
    @location(2) back: vec4<f32>,
    @location(3) @interpolate(flat) state: u32,
    @location(4) uv: vec2<f32>,
    @location(5) @interpolate(flat) layers: vec2<u32>,
    @location(6) clip: f32,
    @location(7) lpos: vec3<f32>,
    @location(8) @interpolate(flat) mats: vec2<u32>,
};

// Material textures (see textures.rs): layer i + 1 in `layers`; `tex_info[i].xy` = 1 / tile size (mm).
@group(1) @binding(0) var face_tex: texture_2d_array<f32>;
@group(1) @binding(1) var face_tex_sampler: sampler;
@group(1) @binding(2) var<uniform> tex_info: array<vec4<f32>, 256>;
// Data textures (linear: packed occlusion / roughness / metalness, normal maps), same scheme.
@group(1) @binding(3) var data_tex: texture_2d_array<f32>;
@group(1) @binding(4) var<uniform> data_info: array<vec4<f32>, 256>;
// PBR materials (textures.rs `set_materials`), 6 vec4 each; material i + 1 in `mats`:
// [metallic, roughness, transmission, ior], [emission rgb, normal scale],
// [emission layer, orm layer, normal layer, orm mask], [custom shader + 1, 0, 0, 0],
// custom shader parameters 0..3, 4..7.
@group(1) @binding(5) var<uniform> materials: array<vec4<f32>, 1536>;

@vertex
fn vs_face(v: FaceIn) -> FaceOut {
    var o: FaceOut;
    o.pos = project(v.pos);
    o.normal = v.normal;
    o.color = lin(v.color);
    o.back = lin(v.back);
    o.state = hl_state(v.state);
    o.uv = v.uv;
    o.layers = v.layers;
    o.clip = sec_dist(v.pos);
    o.lpos = v.pos;
    o.mats = v.mats;
    if g.params.x > 2.5 && g.params.x < 3.5 && o.state != HL_SELECTED {
        o.pos = vec4<f32>(0.0, 0.0, -2.0, 1.0); // wireframe tint pass: drop unselected triangles
    }
    return o;
}

struct FaceFrag {
    @location(0) color: vec4<f32>,
    /// Effects path only (post.rs, Realistic): see `gbuf_pack`.
    @location(1) gbuf: vec4<f32>,
};

@fragment
fn fs_face(in: FaceOut, @builtin(front_facing) ff: bool) -> FaceFrag {
    if in.clip < 0.0 {
        discard;
    }
    frag_px = in.pos.xy;
    let mode = g.params.x;
    if mode > 3.5 {
        return fs_face_pbr(in, ff);
    }
    var o: FaceFrag;
    o.color = face_classic(in, ff);
    o.gbuf = vec4<f32>(0.5, 0.5, 1.0, 0.0);
    return o;
}

fn face_classic(in: FaceOut, ff: bool) -> vec4<f32> {
    let mode = g.params.x;
    let alpha_mul = g.params.y;
    if mode > 2.5 {
        return out_color(vec4<f32>(srgb_to_linear(g.hl_selected.rgb), 0.3));
    }
    if mode > 1.5 {
        let h = highlight_face(vec4<f32>(g.hidden_face.rgb, 1.0), in.state);
        return out_color(vec4<f32>(h.rgb, alpha_mul));
    }
    var n = in.normal;
    if dot(n, n) < 1e-12 {
        n = -g.fwd.xyz;
    } else {
        n = normalize(n);
    }
    var c = in.color;
    var layer = in.layers.x;
    if !ff {
        n = -n;
        c = in.back;
        layer = in.layers.y;
    }
    // Sampled for every fragment (implicit derivatives need uniform control flow).
    let li = max(i32(layer) - 1, 0);
    let texel = textureSample(face_tex, face_tex_sampler, in.uv * tex_info[li].xy, li);
    if layer > 0u && mode < 0.5 {
        if texel.a < 0.5 {
            discard;
        }
        c = vec4<f32>(texel.rgb, c.a);
    }
    if mode > 0.5 {
        c = select(g.mono_back, g.mono_front, ff);
    }
    c = highlight_face(c, in.state);
    // Fixed world-space sun (Lambert) + ambient, plus a small camera fill so faces
    // turned away from the sun stay readable. Shade is defined perceptually (sRGB-ish)
    // and converted to linear so differences between planes stay visible.
    var ambient = g.light.w;
    var top = 1.0;
    var sun = min(max(dot(n, g.light.xyz), 0.0) * 1.25, 1.0);
    if g.shadow2.w > 0.5 {
        // Sun shading (Shadows panel): light / dark shades, shadow map, nothing below the horizon.
        ambient = g.shadow.w;
        top = max(min(g.shadow.z * 1.25, 1.0), ambient);
        sun *= g.shadow2.z;
        if g.shadow.x > 0.5 && sun > 0.0 {
            sun *= shadow_vis(in.lpos, n);
        }
    }
    let fill = 0.1 * max(dot(n, -g.fwd.xyz), 0.0);
    let shade = clamp(ambient + (top - ambient) * sun + fill * (1.0 - sun), 0.0, 1.0);
    let shade_lin = pow(shade, 2.2);
    return out_color(vec4<f32>(c.rgb * shade_lin, c.a * alpha_mul));
}

// ---------------------------------------------------------------------------
// Thin lines (1 px, line list) — model edges
// ---------------------------------------------------------------------------

struct LineIn {
    @location(0) pos: vec3<f32>,
    @location(1) color: vec4<f32>,
};

struct LineOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) clip: f32,
};

fn edge_col(c: vec4<f32>) -> vec4<f32> {
    // Pure black edges follow the style's edge color.
    if all(c.rgb == vec3<f32>(0.0)) {
        return vec4<f32>(g.edge_color.rgb, g.edge_color.a * c.a);
    }
    return lin(c);
}

struct EdgeIn {
    @location(0) pos: vec3<f32>,
    @location(1) color: vec4<f32>,
    @location(2) state: u32,
};

@vertex
fn vs_edge(v: EdgeIn) -> LineOut {
    var o: LineOut;
    o.pos = bias_clip(project(v.pos));
    o.color = highlight_edge(edge_col(v.color), hl_state(v.state));
    o.clip = sec_dist(v.pos);
    return o;
}

@fragment
fn fs_flat(in: LineOut) -> @location(0) vec4<f32> {
    if in.clip < 0.0 {
        discard;
    }
    return out_color(in.color);
}

// Overlay polygons (translucent, per-vertex color, slight bias so they show on faces)
@vertex
fn vs_poly(v: LineIn) -> LineOut {
    var o: LineOut;
    o.pos = bias_clip(project(v.pos));
    o.color = lin(v.color);
    o.clip = 1.0;
    return o;
}

// ---------------------------------------------------------------------------
// Thick lines: one instance per segment, expanded to a screen-space quad.
// ---------------------------------------------------------------------------

struct ThickIn {
    @location(0) a: vec3<f32>,
    @location(1) width: f32,   // logical points; < 0 => profile width uniform; 0 => model edge (1 pt)
    @location(2) b: vec3<f32>,
    @location(3) dash: f32,    // dash period in points (0 = solid); < 0 => dotted pattern
    @location(4) color: vec4<f32>,
};

struct ThickOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) @interpolate(linear) along: f32,
    @location(2) @interpolate(flat) dash: f32,
    @location(3) clip: f32,
};

fn thick_color(c: vec4<f32>, width: f32) -> vec4<f32> {
    if width == 0.0 {
        return edge_col(c);
    }
    return lin(c);
}

// Overlay lines / polylines / axes: `ThickIn` + dash phase at `a` (points, continuous polylines).
@vertex
fn vs_line(@builtin(vertex_index) vi: u32, v: ThickIn, @location(5) phase: f32) -> ThickOut {
    return thick_vertex(vi, v, 0u, phase, false);
}

// Static model edges / profiles, instanced fallback with a per-segment highlight stream.
@vertex
fn vs_thick_state(@builtin(vertex_index) vi: u32, v: ThickIn, @location(5) state: u32) -> ThickOut {
    return thick_vertex(vi, v, state, 0.0, true);
}

// Static edges / profiles read straight from the raw `LineVertex` buffer (storage, 7 floats per
// vertex: pos.xyz, color.rgba; 2 vertices per segment) as one non-instanced triangle list,
// 6 vertices per segment (tiny instances are slow on tile-based GPUs).
@group(1) @binding(0) var<storage, read> seg_lines: array<f32>;
@group(1) @binding(1) var<storage, read> seg_states: array<u32>;

fn seg_vertex(vi: u32, width: f32) -> ThickOut {
    let i = vi / 6u;
    let o = i * 14u;
    var v: ThickIn;
    v.a = vec3<f32>(seg_lines[o], seg_lines[o + 1u], seg_lines[o + 2u]);
    v.color = vec4<f32>(seg_lines[o + 3u], seg_lines[o + 4u], seg_lines[o + 5u], seg_lines[o + 6u]);
    v.b = vec3<f32>(seg_lines[o + 7u], seg_lines[o + 8u], seg_lines[o + 9u]);
    v.width = width;
    v.dash = 0.0;
    return thick_vertex(vi % 6u, v, seg_states[i], 0.0, true);
}

@vertex
fn vs_edge_seg(@builtin(vertex_index) vi: u32) -> ThickOut {
    return seg_vertex(vi, 0.0); // model edge: 1 logical point, style edge colour
}

@vertex
fn vs_profile_seg(@builtin(vertex_index) vi: u32) -> ThickOut {
    return seg_vertex(vi, -1.0); // profile width uniform
}

/// Tile clip coordinates -> clip coordinates of the whole image (inverse of the tile transform).
fn full_clip(c: vec4<f32>) -> vec4<f32> {
    return vec4<f32>((c.xy - g.tile.zw * c.w) / g.tile.xy, c.z, c.w);
}

fn thick_vertex(vi: u32, v: ThickIn, state: u32, phase0: f32, clipped: bool) -> ThickOut {
    // corners: (end, side)
    var ends = array<f32, 6>(0.0, 0.0, 1.0, 0.0, 1.0, 1.0);
    var sides = array<f32, 6>(-1.0, 1.0, 1.0, -1.0, 1.0, -1.0);
    let end = ends[vi];
    let side = sides[vi];

    var ca = project(v.a);
    var cb = project(v.b);
    var o: ThickOut;
    // Clip the segment (homogeneous, Liang-Barsky) against a plane slightly in front of the eye
    // and against the side planes of the WHOLE image widened by a margin. Very long lines (guides,
    // axes) then never produce huge screen coordinates, which keeps rasterisation and the dash
    // phase precise. Planes are in whole-image clip space (tile-independent), so export tiles agree.
    let wmin = 1e-5;
    let k = 1.5;
    let fa = full_clip(ca);
    let fb = full_clip(cb);
    var da = array<f32, 5>(fa.w - wmin, k * fa.w - fa.x, k * fa.w + fa.x, k * fa.w - fa.y, k * fa.w + fa.y);
    var db = array<f32, 5>(fb.w - wmin, k * fb.w - fb.x, k * fb.w + fb.x, k * fb.w - fb.y, k * fb.w + fb.y);
    var t0 = 0.0;
    var t1 = 1.0;
    for (var p = 0; p < 5; p++) {
        let a = da[p];
        let b = db[p];
        if a < 0.0 && b < 0.0 {
            o.pos = vec4<f32>(0.0, 0.0, -2.0, 1.0);
            return o;
        }
        if a < 0.0 {
            t0 = max(t0, a / (a - b));
        } else if b < 0.0 {
            t1 = min(t1, a / (a - b));
        }
    }
    if t0 > t1 {
        o.pos = vec4<f32>(0.0, 0.0, -2.0, 1.0);
        return o;
    }
    let half_vp = g.viewport.xy * 0.5;
    // Dash phase: keep the pattern anchored at `a` when `a` is in front of the eye.
    var phase = 0.0;
    let ca0 = ca;
    let c0 = mix(ca0, cb, t0);
    let c1 = mix(ca0, cb, t1);
    ca = c0;
    cb = c1;
    if t0 > 0.0 && ca0.w > wmin {
        phase = length((ca.xy / ca.w - ca0.xy / ca0.w) * half_vp);
    }
    let sa = ca.xy / ca.w * half_vp;
    let sb = cb.xy / cb.w * half_vp;
    var d = sb - sa;
    let len = length(d);
    if len < 1e-6 {
        d = vec2<f32>(1.0, 0.0);
    } else {
        d = d / len;
    }
    let n = vec2<f32>(-d.y, d.x);
    var w = v.width;
    if w < 0.0 {
        w = g.viewport.w;
    } else if w == 0.0 {
        w = g.viewport.z;
    } else {
        w = w * g.viewport.z;
    }
    let hw = max(w, 1.0) * 0.5;
    var base = ca;
    var ext = -hw;
    if end > 0.5 {
        base = cb;
        ext = hw;
    }
    let off = (n * side * hw + d * ext) / half_vp * base.w;
    o.pos = bias_clip(vec4<f32>(base.xy + off, base.z, base.w));
    o.color = highlight_edge(thick_color(v.color, v.width), hl_state(state));
    o.along = (phase + end * len + ext) / g.viewport.z + phase0; // in points
    o.dash = v.dash;
    o.clip = 1.0;
    if clipped {
        o.clip = mix(sec_dist(v.a), sec_dist(v.b), mix(t0, t1, end));
    }
    return o;
}

@fragment
fn fs_thick(in: ThickOut) -> @location(0) vec4<f32> {
    if in.clip < 0.0 {
        discard;
    }
    if in.dash > 0.0 {
        // dashed: 60% on
        if fract(in.along / in.dash) > 0.6 {
            discard;
        }
    } else if in.dash < 0.0 {
        // dotted: 40% on
        if fract(in.along / -in.dash) > 0.4 {
            discard;
        }
    }
    return out_color(in.color);
}

// ---------------------------------------------------------------------------
// Sprites: text glyphs, text boxes, arrowheads and markers. Each vertex is an offset in
// physical pixels from a 3D anchor; unrotated text snaps the anchor to the pixel grid so
// glyphs map 1:1 to atlas texels. Offsets are in whole-image pixels (tile-independent).
// ---------------------------------------------------------------------------

@group(1) @binding(0) var glyph_atlas: texture_2d<f32>;
@group(1) @binding(1) var glyph_sampler: sampler;

struct SpriteIn {
    @location(0) anchor: vec3<f32>,
    @location(1) mode: f32,        // bit 0: glyph (atlas coverage); bit 1: no pixel snapping
    @location(2) offset: vec2<f32>, // physical pixels, x right, y down
    @location(3) uv: vec2<f32>,     // atlas texels
    @location(4) color: vec4<f32>,  // sRGB
};

struct SpriteOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) @interpolate(flat) glyph: u32,
};

@vertex
fn vs_sprite(v: SpriteIn) -> SpriteOut {
    var o: SpriteOut;
    let c = project(v.anchor);
    if c.w <= 1e-5 {
        o.pos = vec4<f32>(0.0, 0.0, -2.0, 1.0); // anchor behind the eye
        return o;
    }
    let mode = u32(v.mode + 0.5);
    let half_vp = g.viewport.xy * 0.5;
    // Pixels from the target's bottom-left corner (tile offsets are whole pixels, so snapping
    // agrees between export tiles).
    var px = (c.xy / c.w) * half_vp + half_vp;
    if (mode & 2u) == 0u {
        px = floor(px + 0.5);
    }
    let q = px + vec2<f32>(v.offset.x, -v.offset.y) - half_vp;
    o.pos = bias_clip(vec4<f32>(q / half_vp * c.w, c.z, c.w));
    o.color = lin(v.color);
    o.uv = v.uv;
    o.glyph = mode & 1u;
    return o;
}

@fragment
fn fs_sprite(in: SpriteOut) -> @location(0) vec4<f32> {
    var c = in.color;
    // Sample unconditionally (uniform control flow); solid sprites ignore it.
    let dims = vec2<f32>(textureDimensions(glyph_atlas));
    let cov = textureSampleLevel(glyph_atlas, glyph_sampler, in.uv / dims, 0.0).a;
    if in.glyph == 1u {
        // The rasteriser's coverage is tuned for gamma-space blending; blending happens in
        // linear space here, which makes dark text on light backgrounds look thin. Boost the
        // coverage of dark text to match egui's look (light text keeps raw coverage).
        let lum = dot(c.rgb, vec3<f32>(0.2126, 0.7152, 0.0722));
        let k = mix(0.55, 1.0, clamp(lum * 2.0, 0.0, 1.0));
        c.a *= pow(cov, k);
    }
    return out_color(c);
}

// ---------------------------------------------------------------------------
// Realistic faces: metallic-roughness PBR (GGX / Smith / Schlick), image-based light from the
// environment (SH diffuse, mip-blurred reflections) and the sun, tone mapped.
// ---------------------------------------------------------------------------

/// Split-sum environment BRDF approximation (Karis, mobile).
fn env_brdf(f0: vec3<f32>, rough: f32, nv: f32) -> vec3<f32> {
    let c0 = vec4<f32>(-1.0, -0.0275, -0.572, 0.022);
    let c1 = vec4<f32>(1.0, 0.0425, 1.04, -0.04);
    let r = rough * c0 + c1;
    let a004 = min(r.x * r.x, exp2(-9.28 * nv)) * r.x + r.y;
    let ab = vec2<f32>(-1.04, 1.04) * a004 + r.zw;
    return f0 * ab.x + ab.y;
}

fn fs_face_pbr(in: FaceOut, ff: bool) -> FaceFrag {
    // Derivatives first (uniform control flow); maps are sampled with explicit gradients.
    let dp_dx = dpdx(in.lpos);
    let dp_dy = dpdy(in.lpos);
    let duv_dx = dpdx(in.uv);
    let duv_dy = dpdy(in.uv);
    var n = in.normal;
    if dot(n, n) < 1e-12 {
        n = -g.fwd.xyz;
    } else {
        n = normalize(n);
    }
    var c = in.color;
    var layer = in.layers.x;
    var mat = in.mats.x;
    if !ff {
        n = -n;
        c = in.back;
        layer = in.layers.y;
        mat = in.mats.y;
    }
    if layer > 0u {
        let li = i32(layer) - 1;
        let k = tex_info[li].xy;
        let texel = textureSampleGrad(face_tex, face_tex_sampler, in.uv * k, li, duv_dx * k, duv_dy * k);
        if texel.a < 0.5 {
            discard;
        }
        c = vec4<f32>(texel.rgb, c.a);
    }
    var metallic = 0.0;
    var rough = 0.6;
    var transmission = 0.0;
    var emission = vec3<f32>(0.0);
    var ao = 1.0;
    let clay = env.p3.x > 0.5;
    if mat > 0u && !clay {
        let m = (mat - 1u) * 6u;
        let m0 = materials[m];
        let m1 = materials[m + 1u];
        let m2 = materials[m + 2u];
        metallic = m0.x;
        rough = m0.y;
        transmission = m0.z;
        emission = m1.rgb;
        let orm_layer = u32(m2.y + 0.5);
        if orm_layer > 0u {
            let li = i32(orm_layer) - 1;
            let k = data_info[li].xy;
            let orm = textureSampleGrad(data_tex, face_tex_sampler, in.uv * k, li, duv_dx * k, duv_dy * k).rgb;
            let mask = u32(m2.w + 0.5);
            if (mask & 1u) != 0u {
                ao = orm.r;
            }
            if (mask & 2u) != 0u {
                rough *= orm.g;
            }
            if (mask & 4u) != 0u {
                metallic *= orm.b;
            }
        }
        let em_layer = u32(m2.x + 0.5);
        if em_layer > 0u {
            let li = i32(em_layer) - 1;
            let k = tex_info[li].xy;
            emission *= textureSampleGrad(face_tex, face_tex_sampler, in.uv * k, li, duv_dx * k, duv_dy * k).rgb;
        }
        let nl = u32(m2.z + 0.5);
        if nl > 0u {
            // Tangent frame from the planar coordinates (cotangent frame, no stored tangents).
            let li = i32(nl) - 1;
            let k = data_info[li].xy;
            let tn = textureSampleGrad(data_tex, face_tex_sampler, in.uv * k, li, duv_dx * k, duv_dy * k).rgb * 2.0 - 1.0;
            let dp2perp = cross(dp_dy, n);
            let dp1perp = cross(n, dp_dx);
            var t = dp2perp * duv_dx.x + dp1perp * duv_dy.x;
            var b = dp2perp * duv_dx.y + dp1perp * duv_dy.y;
            let inv = inverseSqrt(max(max(dot(t, t), dot(b, b)), 1e-20));
            t *= inv;
            b *= inv;
            let pn = normalize(t * tn.x * m1.w + b * tn.y * m1.w + n * max(tn.z, 1e-3));
            if all(pn == pn) {
                n = pn;
            }
        }
    }
    // View direction (toward the eye).
    var v = -g.fwd.xyz;
    if g.eye_rel.w < 0.5 {
        v = normalize(g.eye_rel.xyz - (in.lpos - g.origin.xyz));
    }
    if mat > 0u && !clay {
        let m = (mat - 1u) * 6u;
        let id = u32(materials[m + 3u].x + 0.5);
        if id > 0u {
            var s: Surface;
            s.base = c.rgb;
            s.alpha = c.a;
            s.metallic = metallic;
            s.roughness = rough;
            s.transmission = transmission;
            s.emission = emission;
            s.normal = n;
            s.pos = in.lpos + vec3<f32>(0.0, 0.0, g.origin.w);
            s.uv = in.uv;
            s.view = v;
            s.time = env.p2.w;
            s = custom_surface(id, s, materials[m + 4u], materials[m + 5u]);
            c = vec4<f32>(max(s.base, vec3<f32>(0.0)), clamp(s.alpha, 0.0, 1.0));
            metallic = s.metallic;
            rough = s.roughness;
            transmission = clamp(s.transmission, 0.0, 1.0);
            emission = max(s.emission, vec3<f32>(0.0));
            if dot(s.normal, s.normal) > 1e-8 {
                n = normalize(s.normal);
            }
        }
    }
    if clay {
        c = vec4<f32>(0.62, 0.6, 0.57, select(c.a, 1.0, c.a > 0.3));
        metallic = 0.0;
        rough = 0.75;
        transmission = 0.0;
        emission = vec3<f32>(0.0);
    }
    rough = clamp(rough, 0.03, 1.0);
    metallic = clamp(metallic, 0.0, 1.0);
    let albedo = c.rgb;
    let nv = max(dot(n, v), 1e-4);
    let f0 = mix(vec3<f32>(0.04), albedo, metallic);
    let diffuse_col = albedo * (1.0 - metallic) * (1.0 - transmission);
    // Image-based light.
    let r = reflect(-v, n);
    let lod = env.p.z * (1.0 - pow(1.0 - rough, 2.0)) * 0.85;
    let spec_ibl = env_radiance(r, lod) * env_brdf(f0, rough, nv);
    var col = (diffuse_col * env_irradiance(n) / PI) * ao + spec_ibl * ao;
    // The sun.
    let l = g.light.xyz;
    let nl_ = dot(n, l);
    var sun = env.p2.z;
    if g.shadow2.w > 0.5 {
        sun *= g.shadow2.z;
        if g.shadow.x > 0.5 && sun > 0.0 && nl_ > 0.0 {
            sun *= shadow_vis(in.lpos, n);
        }
    }
    if nl_ > 0.0 && sun > 0.0 {
        let h = normalize(v + l);
        let nh = max(dot(n, h), 0.0);
        let vh = max(dot(v, h), 0.0);
        let a = rough * rough;
        let a2 = a * a;
        let dd = nh * nh * (a2 - 1.0) + 1.0;
        let d = a2 / (PI * dd * dd);
        let k = (rough + 1.0) * (rough + 1.0) / 8.0;
        let vis = 1.0 / ((nv * (1.0 - k) + k) * (nl_ * (1.0 - k) + k));
        let f = f0 + (vec3<f32>(1.0) - f0) * pow(1.0 - vh, 5.0);
        let spec = d * vis * 0.25 * f;
        // `sun` = irradiance in units of a white sky's (E = π·sun).
        col += (diffuse_col / PI * (vec3<f32>(1.0) - f) + spec) * sun * PI * nl_;
    }
    col += emission;
    // Glass: reflections must survive alpha blending, so the opacity rises with the Fresnel
    // reflectance and the colour is divided back.
    var a = c.a * g.params.y;
    if a < 0.999 {
        let fr = env_brdf(f0, rough, nv).g + 0.04;
        let a2 = clamp(mix(a, 1.0, fr), a, 1.0);
        col = (col - spec_ibl * ao) * (a / a2) + spec_ibl * ao / a2;
        a = a2;
    }
    let mapped = highlight_face(vec4<f32>(tonemap(col), 1.0), in.state);
    var o: FaceFrag;
    o.color = out_color(vec4<f32>(mapped.rgb, a));
    o.gbuf = gbuf_pack(n, rough, metallic);
    return o;
}

// ---------------------------------------------------------------------------
// Custom WGSL surfaces (docs/SHADERS.md): `custom_surface` is generated by custom.rs, which
// appends each material's `fn surface(s: Surface, p: Params) -> Surface` (names suffixed).
// ---------------------------------------------------------------------------

struct Surface {
    base: vec3<f32>,     // linear base colour
    alpha: f32,          // opacity (the material's; faces are blended only if it is below 1)
    metallic: f32,
    roughness: f32,
    transmission: f32,   // glass: 1 = light passes through
    emission: vec3<f32>, // linear, strength included
    normal: vec3<f32>,   // world, unit, facing the viewer
    pos: vec3<f32>,      // model position (mm, relative to the render frame; z = world z)
    uv: vec2<f32>,       // planar texture coordinates (mm)
    view: vec3<f32>,     // unit direction toward the eye
    time: f32,           // seconds
};

fn hash3(p: vec3<f32>) -> f32 {
    let q = fract(p * vec3<f32>(0.1031, 0.1030, 0.0973));
    let r = q + dot(q, q.yzx + 33.33);
    return fract((r.x + r.y) * r.z);
}

/// Value noise, -1..1 is never reached: 0..1, smooth.
fn noise(p: vec3<f32>) -> f32 {
    let i = floor(p);
    let f = fract(p);
    let u = f * f * (3.0 - 2.0 * f);
    let a = mix(mix(hash3(i), hash3(i + vec3<f32>(1.0, 0.0, 0.0)), u.x),
                mix(hash3(i + vec3<f32>(0.0, 1.0, 0.0)), hash3(i + vec3<f32>(1.0, 1.0, 0.0)), u.x), u.y);
    let b = mix(mix(hash3(i + vec3<f32>(0.0, 0.0, 1.0)), hash3(i + vec3<f32>(1.0, 0.0, 1.0)), u.x),
                mix(hash3(i + vec3<f32>(0.0, 1.0, 1.0)), hash3(i + vec3<f32>(1.0, 1.0, 1.0)), u.x), u.y);
    return mix(a, b, u.z);
}

/// Fractal noise: `octaves` layers of `noise`, 0..1.
fn fbm(p: vec3<f32>, octaves: i32) -> f32 {
    var sum = 0.0;
    var amp = 0.5;
    var q = p;
    var norm = 0.0;
    for (var i = 0; i < octaves; i++) {
        sum += amp * noise(q);
        norm += amp;
        q = q * 2.03 + vec3<f32>(17.1, 3.7, 9.2);
        amp *= 0.5;
    }
    return sum / max(norm, 1e-6);
}

/// sRGB colour (0..1) -> linear.
fn srgb(c: vec3<f32>) -> vec3<f32> {
    return srgb_to_linear(c);
}

/// Hue, saturation, value (0..1) -> linear colour.
fn hsv(h: f32, sat: f32, v: f32) -> vec3<f32> {
    let k = vec3<f32>(1.0, 2.0 / 3.0, 1.0 / 3.0);
    let q = abs(fract(vec3<f32>(h) + k) * 6.0 - 3.0);
    return srgb_to_linear(v * mix(vec3<f32>(1.0), clamp(q - 1.0, vec3<f32>(0.0), vec3<f32>(1.0)), sat));
}

/// A unit vector across `n` (with `cross(n, t)` it makes a tangent frame).
fn tangent_of(n: vec3<f32>) -> vec3<f32> {
    return normalize(cross(n, select(vec3<f32>(0.0, 0.0, 1.0), vec3<f32>(1.0, 0.0, 0.0), abs(n.z) > 0.9)));
}

/// `n` tilted by the slope of a height field: `dh` = change of height along the tangent
/// frame's two directions (per unit of their length).
fn bump(n: vec3<f32>, dh: vec2<f32>) -> vec3<f32> {
    let t = tangent_of(n);
    let b = cross(n, t);
    return normalize(n - t * dh.x - b * dh.y);
}
