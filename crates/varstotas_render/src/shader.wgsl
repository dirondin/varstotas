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
    params2: vec4<f32>,       // x: line bias absolute (ndc); y: ground enabled;
                              // z: highlight hide mask (bit 0 Selected, bit 1 Dimmed)
    hl_selected: vec4<f32>,   // rgb: selection color (sRGB); w: face mix
    hl_dim_face: vec4<f32>,   // rgb: dimmed-face target (sRGB); w: face mix
    hl_dim_edge: vec4<f32>,   // dimmed edge color (linear)
    tile: vec4<f32>,          // tile NDC = full NDC * xy + zw (export tiling; (1,1,0,0) = whole image)
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
    @location(4) state: u32,
    @location(5) uv: vec2<f32>,
    @location(6) layers: vec2<u32>,
};

struct FaceOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) normal: vec3<f32>,
    @location(1) color: vec4<f32>,
    @location(2) back: vec4<f32>,
    @location(3) @interpolate(flat) state: u32,
    @location(4) uv: vec2<f32>,
    @location(5) @interpolate(flat) layers: vec2<u32>,
};

// Material textures (see textures.rs): layer i + 1 in `layers`; `tex_info[i].xy` = 1 / tile size (mm).
@group(1) @binding(0) var face_tex: texture_2d_array<f32>;
@group(1) @binding(1) var face_tex_sampler: sampler;
@group(1) @binding(2) var<uniform> tex_info: array<vec4<f32>, 256>;

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
    if g.params.x > 2.5 && o.state != HL_SELECTED {
        o.pos = vec4<f32>(0.0, 0.0, -2.0, 1.0); // wireframe tint pass: drop unselected triangles
    }
    return o;
}

@fragment
fn fs_face(in: FaceOut, @builtin(front_facing) ff: bool) -> @location(0) vec4<f32> {
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

// Overlay lines / polylines / axes: `ThickIn` + dash phase at `a` (points, continuous polylines).
@vertex
fn vs_line(@builtin(vertex_index) vi: u32, v: ThickIn, @location(5) phase: f32) -> ThickOut {
    return thick_vertex(vi, v, 0u, phase);
}

// Static model edges / profiles, instanced fallback with a per-segment highlight stream.
@vertex
fn vs_thick_state(@builtin(vertex_index) vi: u32, v: ThickIn, @location(5) state: u32) -> ThickOut {
    return thick_vertex(vi, v, state, 0.0);
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
    return thick_vertex(vi % 6u, v, seg_states[i], 0.0);
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

fn thick_vertex(vi: u32, v: ThickIn, state: u32, phase0: f32) -> ThickOut {
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
