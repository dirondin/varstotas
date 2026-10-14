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
    params: vec4<f32>,        // x: face mode (0 shaded, 1 mono, 2 hidden-line); y: alpha mul;
                              // z: line bias relative; w: 1 = encode gamma manually
    params2: vec4<f32>,       // x: line bias absolute (ndc); y: ground enabled
};

@group(0) @binding(0) var<uniform> g: Globals;

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
    if g.params.w > 0.5 {
        return vec4<f32>(linear_to_srgb(clamp(c.rgb, vec3<f32>(0.0), vec3<f32>(1.0))), c.a);
    }
    return c;
}

fn project(p_world: vec3<f32>) -> vec4<f32> {
    return g.view_proj * vec4<f32>(p_world - g.origin.xyz, 1.0);
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
    o.ndc = vec2<f32>(x, y);
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
    var t: f32;
    if g.eye_rel.w > 0.5 {
        t = clamp(in.ndc.y * 0.5 + 0.5, 0.0, 1.0);
    } else {
        let r = pixel_ray(in.ndc);
        t = clamp(r.d.z / 0.6, 0.0, 1.0);
        t = sqrt(t);
    }
    let c = mix(g.sky_horizon.rgb, g.sky_top.rgb, t);
    return out_color(vec4<f32>(c, 1.0));
}

struct GroundOut {
    @location(0) color: vec4<f32>,
    @builtin(frag_depth) depth: f32,
};

@fragment
fn fs_ground(in: FsOut) -> GroundOut {
    let r = pixel_ray(in.ndc);
    let ortho = g.eye_rel.w > 0.5;
    // origin.w = world z of the geometry's local frame (SceneGeometry::origin.z).
    let plane_z = -(g.origin.z + g.origin.w); // world z = 0 in relative coords
    let eye_world_z = g.eye_rel.z + g.origin.z + g.origin.w;
    // Only visible from above.
    var valid = r.d.z < -1e-6 && (ortho || eye_world_z > 0.0);
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
    var o: GroundOut;
    o.color = out_color(vec4<f32>(g.ground.rgb, alpha));
    o.depth = depth;
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
};

struct FaceOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) normal: vec3<f32>,
    @location(1) color: vec4<f32>,
    @location(2) back: vec4<f32>,
};

@vertex
fn vs_face(v: FaceIn) -> FaceOut {
    var o: FaceOut;
    o.pos = project(v.pos);
    o.normal = v.normal;
    o.color = lin(v.color);
    o.back = lin(v.back);
    return o;
}

@fragment
fn fs_face(in: FaceOut, @builtin(front_facing) ff: bool) -> @location(0) vec4<f32> {
    let mode = g.params.x;
    let alpha_mul = g.params.y;
    if mode > 1.5 {
        return out_color(vec4<f32>(g.hidden_face.rgb, alpha_mul));
    }
    var n = in.normal;
    if dot(n, n) < 1e-12 {
        n = -g.fwd.xyz;
    } else {
        n = normalize(n);
    }
    var c = in.color;
    if !ff {
        n = -n;
        c = in.back;
    }
    if mode > 0.5 {
        c = select(g.mono_back, g.mono_front, ff);
    }
    // Fixed world-space sun (Lambert) + ambient, plus a small camera fill so faces
    // turned away from the sun stay readable. Shade is defined perceptually (sRGB-ish)
    // and converted to linear so differences between planes stay visible.
    let ambient = g.light.w;
    let sun = min(max(dot(n, g.light.xyz), 0.0) * 1.25, 1.0);
    let fill = 0.1 * max(dot(n, -g.fwd.xyz), 0.0);
    let shade = clamp(ambient + (1.0 - ambient) * sun + fill * (1.0 - sun), 0.0, 1.0);
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
};

fn edge_col(c: vec4<f32>) -> vec4<f32> {
    // Pure black edges follow the style's edge color.
    if all(c.rgb == vec3<f32>(0.0)) {
        return vec4<f32>(g.edge_color.rgb, g.edge_color.a * c.a);
    }
    return lin(c);
}

@vertex
fn vs_edge(v: LineIn) -> LineOut {
    var o: LineOut;
    o.pos = bias_clip(project(v.pos));
    o.color = edge_col(v.color);
    return o;
}

@fragment
fn fs_flat(in: LineOut) -> @location(0) vec4<f32> {
    return out_color(in.color);
}

// Overlay polygons (translucent, per-vertex color, slight bias so they show on faces)
@vertex
fn vs_poly(v: LineIn) -> LineOut {
    var o: LineOut;
    o.pos = bias_clip(project(v.pos));
    o.color = lin(v.color);
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
};

fn thick_color(c: vec4<f32>, width: f32) -> vec4<f32> {
    if width == 0.0 {
        return edge_col(c);
    }
    return lin(c);
}

@vertex
fn vs_thick(@builtin(vertex_index) vi: u32, v: ThickIn) -> ThickOut {
    return thick_vertex(vi, v);
}

// Same segments read from a storage buffer: one non-instanced triangle list, 6 vertices per
// segment (avoids tiny instances, which are slow on tile-based GPUs).
struct Seg {
    a: vec3<f32>,
    width: f32,
    b: vec3<f32>,
    dash: f32,
    color: vec4<f32>,
};

@group(1) @binding(0) var<storage, read> segs: array<Seg>;

@vertex
fn vs_thick_seg(@builtin(vertex_index) vi: u32) -> ThickOut {
    let s = segs[vi / 6u];
    var v: ThickIn;
    v.a = s.a;
    v.width = s.width;
    v.b = s.b;
    v.dash = s.dash;
    v.color = s.color;
    return thick_vertex(vi % 6u, v);
}

fn thick_vertex(vi: u32, v: ThickIn) -> ThickOut {
    // corners: (end, side)
    var ends = array<f32, 6>(0.0, 0.0, 1.0, 0.0, 1.0, 1.0);
    var sides = array<f32, 6>(-1.0, 1.0, 1.0, -1.0, 1.0, -1.0);
    let end = ends[vi];
    let side = sides[vi];

    var ca = project(v.a);
    var cb = project(v.b);
    // Clip the segment against a plane slightly in front of the eye (perspective only).
    let wmin = 1e-5;
    var o: ThickOut;
    if ca.w < wmin && cb.w < wmin {
        o.pos = vec4<f32>(0.0, 0.0, -2.0, 1.0);
        return o;
    }
    if ca.w < wmin {
        ca = mix(ca, cb, (wmin - ca.w) / (cb.w - ca.w));
    } else if cb.w < wmin {
        cb = mix(cb, ca, (wmin - cb.w) / (ca.w - cb.w));
    }
    let half_vp = g.viewport.xy * 0.5;
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
    o.color = thick_color(v.color, v.width);
    o.along = (end * len + ext) / g.viewport.z; // in points
    o.dash = v.dash;
    return o;
}

@fragment
fn fs_thick(in: ThickOut) -> @location(0) vec4<f32> {
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
