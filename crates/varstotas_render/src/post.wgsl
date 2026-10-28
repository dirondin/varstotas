// ---------------------------------------------------------------------------
// Effects path of the Realistic style (post.rs). The scene pass leaves linear light (exposure
// applied, see HDR_OUT) and a G-buffer; these full-screen passes add ambient occlusion,
// screen-space reflections, fog, depth of field and bloom, then tone map, vignette and the
// stylized looks, drawn into the multisampled target before the overlays.
// Appended to shader.wgsl (same module: Globals, EnvU, tone_curve, pixel_ray, ...).
// ---------------------------------------------------------------------------

struct PostU {
    inv_vp: mat4x4<f32>,  // clip of this target -> position relative to the render origin
    size: vec4<f32>,      // xy: target size (px); zw: 1 / size
    ao: vec4<f32>,        // x: radius (mm); y: strength; z: samples; w: 1 = on
    ssr: vec4<f32>,       // x: strength; y: steps; z: max distance (mm); w: 1 = on
    bloom: vec4<f32>,     // x: strength; y: threshold; z: -; w: 1 = on
    dof: vec4<f32>,       // x: focus distance (mm, 0 = centre); y: max blur radius (px); z: taps; w: 1 = on
    fog: vec4<f32>,       // x: density (1/mm); y: height falloff (1/mm); z: far distance (mm); w: 1 = on
    fog_color: vec4<f32>, // linear, exposure compensated
    look: vec4<f32>,      // x: vignette; y: stylize (0 none, 1 pencil, 2 watercolour, 3 technical);
                          // z: 1 = the background is the style gradient (not tone mapped); w: ppp
};

@group(1) @binding(0) var p_src: texture_2d<f32>;
@group(1) @binding(1) var p_lin: texture_2d<f32>;
@group(1) @binding(2) var p_gbuf: texture_2d<f32>;
@group(1) @binding(3) var p_ao: texture_2d<f32>;
@group(1) @binding(4) var p_bloom: texture_2d<f32>;
// the depth buffer as unfilterable float: OpenGL (GLSL) cannot textureLoad a depth texture
@group(1) @binding(5) var p_depth: texture_multisampled_2d<f32>;
@group(1) @binding(6) var p_samp: sampler;
@group(1) @binding(7) var<uniform> pu: PostU;

const FAR: f32 = 1e30;

struct PostV {
    @builtin(position) pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_post(@builtin(vertex_index) vi: u32) -> PostV {
    let x = f32((vi << 1u) & 2u) * 2.0 - 1.0;
    let y = f32(vi & 2u) * 2.0 - 1.0;
    var o: PostV;
    o.pos = vec4<f32>(x, y, 0.0, 1.0);
    o.uv = vec2<f32>(x * 0.5 + 0.5, 0.5 - y * 0.5);
    return o;
}

/// Tile NDC of a target uv -> NDC of the whole image (pixel_ray works on the whole image).
fn full_ndc(uv: vec2<f32>) -> vec2<f32> {
    let t = vec2<f32>(uv.x * 2.0 - 1.0, 1.0 - uv.y * 2.0);
    return (t - g.tile.zw) / g.tile.xy;
}

/// Position (relative to the render origin) seen at target uv at view depth `z`.
fn view_pos(uv: vec2<f32>, z: f32) -> vec3<f32> {
    let r = pixel_ray(full_ndc(uv));
    if g.eye_rel.w > 0.5 {
        return r.o + r.d * z;
    }
    return r.o + r.d * (z / max(dot(r.d, g.fwd.xyz), 1e-4));
}

fn view_depth(p: vec3<f32>) -> f32 {
    return dot(p - g.eye_rel.xyz, g.fwd.xyz);
}

/// Target uv of a relative position, and its view depth (z < 0 = behind the camera).
fn project_uv(p: vec3<f32>) -> vec3<f32> {
    let c = g.view_proj * vec4<f32>(p, 1.0);
    let n = c.xy / c.w;
    return vec3<f32>(n.x * 0.5 + 0.5, 0.5 - n.y * 0.5, view_depth(p));
}

fn lin_at(px: vec2<i32>) -> f32 {
    let d = vec2<i32>(textureDimensions(p_lin)) - 1;
    return textureLoad(p_lin, clamp(px, vec2<i32>(0), d), 0).r;
}

fn gbuf_normal(t: vec4<f32>) -> vec3<f32> {
    let e = t.xy * 2.0 - 1.0;
    var n = vec3<f32>(e, 1.0 - abs(e.x) - abs(e.y));
    if n.z < 0.0 {
        let o = (1.0 - abs(n.yx)) * select(vec2<f32>(-1.0), vec2<f32>(1.0), n.xy >= vec2<f32>(0.0));
        n = vec3<f32>(o, n.z);
    }
    return normalize(n);
}

// --- linear depth -----------------------------------------------------------

@fragment
fn fs_post_depth(in: PostV) -> @location(0) vec4<f32> {
    let d = textureLoad(p_depth, vec2<i32>(in.pos.xy), 0).x;
    if d <= 0.0 {
        return vec4<f32>(FAR);
    }
    let ndc = vec2<f32>(in.uv.x * 2.0 - 1.0, 1.0 - in.uv.y * 2.0);
    let h = pu.inv_vp * vec4<f32>(ndc, d, 1.0);
    return vec4<f32>(view_depth(h.xyz / h.w));
}

// --- ambient occlusion --------------------------------------------------------

@fragment
fn fs_post_ao(in: PostV) -> @location(0) vec4<f32> {
    let px = vec2<i32>(in.pos.xy);
    let z = lin_at(px);
    let gb = textureLoad(p_gbuf, px, 0);
    if z >= FAR * 0.5 || gb.b < 0.01 {
        return vec4<f32>(1.0);
    }
    let p = view_pos(in.uv, z);
    let n = gbuf_normal(gb);
    let radius = pu.ao.x;
    let count = i32(pu.ao.z);
    let a = ign(in.pos.xy) * 6.2831853;
    // Tangent frame turned by a per-pixel angle.
    var t = normalize(cross(n, select(vec3<f32>(1.0, 0.0, 0.0), vec3<f32>(0.0, 1.0, 0.0), abs(n.x) > 0.8)));
    let b0 = cross(n, t);
    let tt = t * cos(a) + b0 * sin(a);
    let bb = cross(n, tt);
    var occ = 0.0;
    for (var i = 0; i < count; i++) {
        let fi = f32(i);
        // Points in the hemisphere, denser near the centre.
        let r = (fi + 0.5) / f32(count);
        let ang = fi * 2.3999632;
        let lift = 0.15 + 0.85 * fract(fi * 0.618034 + 0.3);
        let dir = normalize(tt * cos(ang) * (1.0 - lift * 0.5) + bb * sin(ang) * (1.0 - lift * 0.5) + n * lift);
        let s = p + n * radius * 0.02 + dir * radius * (0.1 + 0.9 * r * r);
        let q = project_uv(s);
        if q.z <= 0.0 || any(q.xy < vec2<f32>(0.0)) || any(q.xy > vec2<f32>(1.0)) {
            continue;
        }
        let sz = lin_at(vec2<i32>(q.xy * pu.size.xy));
        let range = smoothstep(0.0, 1.0, radius / max(abs(z - sz), 1e-3));
        if sz < q.z - radius * 0.03 {
            occ += range;
        }
    }
    return vec4<f32>(1.0 - occ / f32(max(count, 1)));
}

@fragment
fn fs_post_ao_blur(in: PostV) -> @location(0) vec4<f32> {
    let px = vec2<i32>(in.pos.xy);
    let z = lin_at(px);
    let d = vec2<i32>(textureDimensions(p_src)) - 1;
    var sum = 0.0;
    var w = 0.0;
    for (var j = -2; j <= 2; j++) {
        for (var i = -2; i <= 2; i++) {
            let q = clamp(px + vec2<i32>(i, j), vec2<i32>(0), d);
            let zq = lin_at(q);
            let k = 1.0 / (1.0 + 40.0 * abs(zq - z) / max(z, 1.0));
            sum += textureLoad(p_src, q, 0).r * k;
            w += k;
        }
    }
    return vec4<f32>(sum / max(w, 1e-4));
}

// --- composite: AO, reflections, fog ------------------------------------------

/// Screen-space reflection along `r` from `p` (relative) seen at pixel `px`; rgb = colour,
/// a = confidence.
fn ssr(p: vec3<f32>, r: vec3<f32>, px: vec2<f32>) -> vec4<f32> {
    let steps = i32(pu.ssr.y);
    let maxd = pu.ssr.z;
    var prev = 0.0;
    var t = maxd / f32(steps) * (0.3 + 0.7 * ign(px));
    for (var i = 0; i < steps; i++) {
        let s = p + r * t;
        let q = project_uv(s);
        if q.z <= 0.0 || any(q.xy < vec2<f32>(0.0)) || any(q.xy > vec2<f32>(1.0)) {
            return vec4<f32>(0.0);
        }
        let sz = lin_at(vec2<i32>(q.xy * pu.size.xy));
        let thick = max(q.z * 0.02, maxd / f32(steps));
        if sz < q.z && q.z - sz < thick {
            // Refine between the last two steps.
            var lo = prev;
            var hi = t;
            for (var k = 0; k < 5; k++) {
                let mid = 0.5 * (lo + hi);
                let qm = project_uv(p + r * mid);
                if lin_at(vec2<i32>(qm.xy * pu.size.xy)) < qm.z {
                    hi = mid;
                } else {
                    lo = mid;
                }
            }
            let qh = project_uv(p + r * hi);
            let dz = qh.z - lin_at(vec2<i32>(qh.xy * pu.size.xy));
            if dz < -thick * 0.25 || dz > thick {
                // The refined point is not on a surface (a depth edge): keep marching.
                prev = t;
                t += maxd / f32(steps) * (1.0 + f32(i) * 0.08);
                continue;
            }
            // The hit surface must face the ray (else it is the back of something or a grazing miss).
            let nh = gbuf_normal(textureLoad(p_gbuf, vec2<i32>(qh.xy * pu.size.xy), 0));
            let facing = smoothstep(0.0, 0.15, -dot(nh, r));
            let edge = min(min(qh.x, 1.0 - qh.x), min(qh.y, 1.0 - qh.y));
            let fade = smoothstep(0.0, 0.08, edge) * (1.0 - smoothstep(0.6, 1.0, hi / maxd));
            let c = textureSampleLevel(p_src, p_samp, qh.xy, 0.0).rgb;
            return vec4<f32>(c, fade * facing);
        }
        prev = t;
        t += maxd / f32(steps) * (1.0 + f32(i) * 0.08);
    }
    return vec4<f32>(0.0);
}

/// Fog transmittance along a ray from relative point `a` to `b`.
fn fog_trans(a: vec3<f32>, b: vec3<f32>) -> f32 {
    let dist = length(b - a);
    var amount = pu.fog.x * dist;
    let h = pu.fog.y;
    if h > 0.0 {
        let base = g.origin.z + g.origin.w;
        let za = max(a.z + base, 0.0);
        let zb = max(b.z + base, 0.0);
        let dz = zb - za;
        var f = exp(-h * za);
        if abs(dz) > 1e-3 {
            f = (exp(-h * za) - exp(-h * zb)) / (h * dz);
        }
        amount *= f;
    }
    return exp(-amount);
}

@fragment
fn fs_post_composite(in: PostV) -> @location(0) vec4<f32> {
    let px = vec2<i32>(in.pos.xy);
    var c = textureLoad(p_src, px, 0);
    let gb = textureLoad(p_gbuf, px, 0);
    let z = lin_at(px);
    let surface = gb.b >= 0.01 && z < FAR * 0.5;
    let eye = g.eye_rel.xyz;
    if surface {
        let p = view_pos(in.uv, z);
        let n = gbuf_normal(gb);
        if pu.ao.w > 0.5 {
            let ao = textureLoad(p_ao, px, 0).r;
            c = vec4<f32>(c.rgb * pow(clamp(ao, 0.0, 1.0), pu.ao.y), c.a);
        }
        let rough = gb.b;
        if pu.ssr.w > 0.5 && rough < 0.55 {
            var v = g.fwd.xyz;
            if g.eye_rel.w < 0.5 {
                v = normalize(p - eye);
            }
            let r = reflect(v, n);
            let nv = max(dot(n, -v), 0.0);
            // Rays heading back toward the eye leave the depth buffer's view: they are unreliable.
            let away = smoothstep(-0.05, 0.35, dot(r, g.fwd.xyz));
            var hit = vec4<f32>(0.0);
            if away > 0.0 {
                hit = ssr(p + n * max(z * 0.002, 1.0), r, in.pos.xy);
                hit.a *= away;
            }
            if hit.a > 0.0 {
                let metal = gb.a;
                let gloss = 1.0 - smoothstep(0.1, 0.55, rough);
                let w = hit.a * gloss * pu.ssr.x;
                let fres = 0.04 + 0.96 * pow(1.0 - nv, 5.0);
                // Metals: the reflection replaces the environment's; dielectrics add a Fresnel part.
                let mc = mix(c.rgb, hit.rgb * 0.9, clamp(w * metal, 0.0, 1.0));
                c = vec4<f32>(mc + hit.rgb * fres * w * (1.0 - metal), c.a);
            }
        }
        if pu.fog.w > 0.5 {
            let tr = fog_trans(eye, p);
            c = vec4<f32>(c.rgb * tr + pu.fog_color.rgb * (1.0 - tr), c.a);
        }
    } else if pu.fog.w > 0.5 {
        let r = pixel_ray(full_ndc(in.uv));
        let tr = fog_trans(r.o, r.o + r.d * pu.fog.z);
        c = vec4<f32>(c.rgb * tr + pu.fog_color.rgb * (1.0 - tr), c.a);
    }
    return c;
}

// --- depth of field -----------------------------------------------------------

fn focus_depth() -> f32 {
    if pu.dof.x > 0.0 {
        return pu.dof.x;
    }
    // What is in the middle of the whole image (tile centre when exporting in tiles).
    let c = vec2<i32>(pu.size.xy * 0.5);
    var z = FAR;
    for (var j = -2; j <= 2; j++) {
        for (var i = -2; i <= 2; i++) {
            z = min(z, lin_at(c + vec2<i32>(i, j) * 4));
        }
    }
    return select(z, 1e6, z >= FAR * 0.5);
}

fn coc(z: f32, focus: f32) -> f32 {
    let zz = min(z, focus * 1000.0);
    return clamp(abs(zz - focus) / max(zz, 1.0) * pu.dof.y * 2.0, 0.0, pu.dof.y);
}

@fragment
fn fs_post_dof(in: PostV) -> @location(0) vec4<f32> {
    let px = vec2<i32>(in.pos.xy);
    let focus = focus_depth();
    let c0 = textureLoad(p_src, px, 0);
    let r0 = coc(lin_at(px), focus);
    let taps = i32(pu.dof.z);
    var sum = c0.rgb;
    var w = 1.0;
    let rmax = pu.dof.y;
    let a = ign(in.pos.xy) * 6.2831853;
    for (var i = 0; i < taps; i++) {
        let fi = f32(i) + 0.5;
        let rr = sqrt(fi / f32(taps)) * rmax;
        let ang = fi * 2.3999632 + a;
        let o = vec2<f32>(cos(ang), sin(ang)) * rr;
        let q = px + vec2<i32>(o);
        let zq = lin_at(q);
        let rq = coc(zq, focus);
        // A sample spreads over this pixel if its own blur reaches it; sharp foreground does not
        // take blur from behind.
        let reach = smoothstep(rr - 1.0, rr + 1.0, rq);
        let behind = select(1.0, smoothstep(rr - 1.0, rr + 1.0, r0), zq > lin_at(px));
        let k = reach * behind;
        sum += textureSampleLevel(p_src, p_samp, (vec2<f32>(q) + 0.5) * pu.size.zw, 0.0).rgb * k;
        w += k;
    }
    return vec4<f32>(sum / w, c0.a);
}

// --- bloom ----------------------------------------------------------------------

fn box4(uv: vec2<f32>) -> vec3<f32> {
    let t = 1.0 / vec2<f32>(textureDimensions(p_src));
    let a = textureSampleLevel(p_src, p_samp, uv + t * vec2<f32>(-1.0, -1.0), 0.0).rgb;
    let b = textureSampleLevel(p_src, p_samp, uv + t * vec2<f32>(1.0, -1.0), 0.0).rgb;
    let c = textureSampleLevel(p_src, p_samp, uv + t * vec2<f32>(-1.0, 1.0), 0.0).rgb;
    let d = textureSampleLevel(p_src, p_samp, uv + t * vec2<f32>(1.0, 1.0), 0.0).rgb;
    return (a + b + c + d) * 0.25;
}

@fragment
fn fs_bloom_prefilter(in: PostV) -> @location(0) vec4<f32> {
    let c = min(box4(in.uv), vec3<f32>(1000.0));
    let b = max(c.r, max(c.g, c.b));
    let knee = pu.bloom.y * 0.5;
    var soft = clamp(b - pu.bloom.y + knee, 0.0, 2.0 * knee);
    soft = soft * soft / (4.0 * knee + 1e-4);
    let k = max(soft, b - pu.bloom.y) / max(b, 1e-4);
    return vec4<f32>(c * k, 1.0);
}

@fragment
fn fs_bloom_down(in: PostV) -> @location(0) vec4<f32> {
    return vec4<f32>(box4(in.uv), 1.0);
}

@fragment
fn fs_bloom_up(in: PostV) -> @location(0) vec4<f32> {
    // 3×3 tent from the smaller level, added onto this one (additive blend).
    let t = 1.0 / vec2<f32>(textureDimensions(p_src));
    var s = vec3<f32>(0.0);
    for (var j = -1; j <= 1; j++) {
        for (var i = -1; i <= 1; i++) {
            let w = (2.0 - abs(f32(i))) * (2.0 - abs(f32(j)));
            s += textureSampleLevel(p_src, p_samp, in.uv + t * vec2<f32>(f32(i), f32(j)), 0.0).rgb * w;
        }
    }
    return vec4<f32>(s / 16.0, 1.0);
}

// --- final: bloom, tone map, vignette, stylized looks -----------------------------

fn luma(c: vec3<f32>) -> f32 {
    return dot(c, vec3<f32>(0.2126, 0.7152, 0.0722));
}

/// Linear light of this target at pixel `q` -> display linear (tone mapped unless it is the
/// style's background gradient).
fn mapped_at(q: vec2<i32>) -> vec3<f32> {
    let d = vec2<i32>(textureDimensions(p_src)) - 1;
    let qq = clamp(q, vec2<i32>(0), d);
    var c = textureLoad(p_src, qq, 0).rgb;
    if pu.bloom.w > 0.5 {
        let uv = (vec2<f32>(qq) + 0.5) * pu.size.zw;
        c += textureSampleLevel(p_bloom, p_samp, uv, 0.0).rgb * pu.bloom.x;
    }
    let bg = textureLoad(p_gbuf, qq, 0).b < 0.01;
    if bg && pu.look.z > 0.5 {
        return clamp(c, vec3<f32>(0.0), vec3<f32>(1.0));
    }
    return tone_curve(c);
}

/// Outline strength (0..1) at `q`: depth jumps (silhouettes) and normal creases; `w` = width (px).
fn outline(q: vec2<i32>, w: f32) -> vec2<f32> {
    let z = lin_at(q);
    let n0 = gbuf_normal(textureLoad(p_gbuf, q, 0));
    let s = i32(max(round(w), 1.0));
    var sil = 0.0;
    var crease = 0.0;
    let dims = vec2<i32>(textureDimensions(p_gbuf)) - 1;
    var dirs = array<vec2<i32>, 4>(vec2<i32>(1, 0), vec2<i32>(-1, 0), vec2<i32>(0, 1), vec2<i32>(0, -1));
    for (var k = 0; k < 4; k++) {
        let o = dirs[k] * s;
        let qq = clamp(q + o, vec2<i32>(0), dims);
        let zq = lin_at(qq);
        let near = min(z, zq);
        if abs(zq - z) > near * 0.02 + 2.0 {
            sil = 1.0;
        }
        let gq = textureLoad(p_gbuf, qq, 0);
        if gq.b >= 0.01 && dot(n0, gbuf_normal(gq)) < 0.85 {
            crease = 1.0;
        }
    }
    return vec2<f32>(sil, crease);
}

fn hash2(p: vec2<f32>) -> f32 {
    return hash3(vec3<f32>(p, 7.31));
}

fn pencil(q: vec2<i32>, c: vec3<f32>, ppp: f32) -> vec3<f32> {
    let p = vec2<f32>(q) / ppp;
    let paper = vec3<f32>(0.965, 0.955, 0.93) * (0.97 + 0.03 * noise(vec3<f32>(p * 0.9, 0.0)));
    let l = luma(c);
    let wob = (noise(vec3<f32>(p * 0.05, 1.0)) - 0.5) * 6.0;
    // Hatching layers: diagonal, cross, then dense strokes as the tone darkens.
    let h1 = abs(fract((p.x + p.y + wob) / 6.0) - 0.5);
    let h2 = abs(fract((p.x - p.y + wob) / 6.0) - 0.5);
    let h3 = abs(fract((p.x + p.y * 0.3 + wob) / 3.5) - 0.5);
    var ink = 0.0;
    ink = max(ink, (1.0 - smoothstep(0.08, 0.16, h1)) * smoothstep(0.78, 0.6, l));
    ink = max(ink, (1.0 - smoothstep(0.08, 0.16, h2)) * smoothstep(0.5, 0.32, l));
    ink = max(ink, (1.0 - smoothstep(0.1, 0.2, h3)) * smoothstep(0.26, 0.1, l));
    ink *= 0.55 + 0.45 * noise(vec3<f32>(p * 0.35, 2.0));
    let jq = q + vec2<i32>(vec2<f32>(noise(vec3<f32>(p * 0.08, 3.0)) - 0.5, noise(vec3<f32>(p * 0.08, 4.0)) - 0.5) * 3.0 * ppp);
    let e = outline(jq, ppp);
    ink = max(ink, max(e.x, e.y * 0.7) * (0.7 + 0.3 * noise(vec3<f32>(p * 0.2, 5.0))));
    let graphite = vec3<f32>(0.2, 0.2, 0.23);
    return mix(paper, graphite, clamp(ink, 0.0, 1.0) * 0.85);
}

fn watercolor(q: vec2<i32>, ppp: f32) -> vec3<f32> {
    let p = vec2<f32>(q) / ppp;
    // Soft wash: a blurred, lightened colour.
    var wash = vec3<f32>(0.0);
    let r = i32(2.0 * ppp);
    for (var j = -1; j <= 1; j++) {
        for (var i = -1; i <= 1; i++) {
            wash += mapped_at(q + vec2<i32>(i, j) * r);
        }
    }
    wash /= 9.0;
    let c = mapped_at(q);
    // Pigment pools at colour edges (darker rims).
    let gx = luma(mapped_at(q + vec2<i32>(r, 0))) - luma(mapped_at(q - vec2<i32>(r, 0)));
    let gy = luma(mapped_at(q + vec2<i32>(0, r))) - luma(mapped_at(q - vec2<i32>(0, r)));
    let rim = clamp(length(vec2<f32>(gx, gy)) * 2.5, 0.0, 1.0);
    let grain = noise(vec3<f32>(p * 0.6, 0.0)) * 0.6 + noise(vec3<f32>(p * 0.15, 1.0)) * 0.4;
    var col = mix(wash, c, 0.35);
    col = mix(vec3<f32>(1.0), col, 0.78 + 0.22 * grain);  // washed out on the paper
    col *= 1.0 - rim * 0.35;
    let e = outline(q, ppp);
    col *= 1.0 - 0.25 * max(e.x, e.y * 0.5);
    let paper = vec3<f32>(0.985, 0.975, 0.95) * (0.96 + 0.04 * grain);
    return col * paper;
}

fn technical(q: vec2<i32>, c: vec3<f32>, ppp: f32) -> vec3<f32> {
    let bg = textureLoad(p_gbuf, q, 0).b < 0.01;
    let l = luma(c);
    // Flat light fills in three tones.
    let tone = select(select(0.97, 0.9, l < 0.45), 0.82, l < 0.2);
    var col = vec3<f32>(tone);
    if bg {
        col = vec3<f32>(1.0);
    }
    let thick = outline(q, 1.5 * ppp);
    let thin = outline(q, ppp);
    let ink = max(thick.x, thin.y * 0.8);
    return mix(col, vec3<f32>(0.08, 0.09, 0.12), ink);
}

@fragment
fn fs_post_final(in: PostV) -> @location(0) vec4<f32> {
    let q = vec2<i32>(in.pos.xy);
    let a = textureLoad(p_src, q, 0).a;
    var c = mapped_at(q);
    let ppp = max(pu.look.w, 1.0);
    let style = i32(pu.look.y + 0.5);
    if style == 1 {
        c = pencil(q, c, ppp);
    } else if style == 2 {
        c = watercolor(q, ppp);
    } else if style == 3 {
        c = technical(q, c, ppp);
    }
    if pu.look.x > 0.0 {
        let n = full_ndc(in.uv);
        let aspect = g.tile.x / g.tile.y * pu.size.x / pu.size.y;
        let d = length(n * vec2<f32>(aspect, 1.0)) / length(vec2<f32>(aspect, 1.0));
        c *= 1.0 - pu.look.x * smoothstep(0.35, 1.0, d);
    }
    if g.params.w > 0.5 {
        return vec4<f32>(linear_to_srgb(clamp(c, vec3<f32>(0.0), vec3<f32>(1.0))), a);
    }
    return vec4<f32>(c, a);
}
