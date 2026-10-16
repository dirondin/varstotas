//! Bundled custom surface shaders (docs/SHADERS.md): procedural wood, water, reeded glass and
//! glowing panels for the material editor's Shader menu.

/// (name, WGSL source).
pub const PRESETS: [(&str, &str); 4] = [("Wood", WOOD), ("Water", WATER), ("Reeded Glass", GLASS), ("Glow", GLOW)];

/// The preset whose source is `src`.
pub fn preset_name(src: &str) -> Option<&'static str> {
    PRESETS.iter().find(|(_, s)| *s == src).map(|(n, _)| *n)
}

pub const WOOD: &str = r#"// Wood: growth rings around the red (X) axis and a fine grain along it.
// @param spacing = 18 (3..120) "Ring spacing (mm)"
// @param rings = 0.55 (0..1) "Ring contrast"
// @param grain = 0.5 (0..1) "Grain"
// @param tone = 0.35 (0..1) "Light / dark wood"
fn surface(s: Surface, p: Params) -> Surface {
    var o = s;
    let q = s.pos;
    let warp = fbm(vec3<f32>(q.x * 0.004, q.y * 0.02, q.z * 0.02), 3) * 3.0;
    let r = length(q.yz) / p.spacing + warp;
    let ring = smoothstep(0.0, 0.35, fract(r)) * (1.0 - smoothstep(0.75, 1.0, fract(r)));
    let fine = noise(vec3<f32>(q.x * 0.01, q.y * 1.5, q.z * 1.5));
    let light = mix(vec3<f32>(0.80, 0.62, 0.40), vec3<f32>(0.52, 0.33, 0.18), p.tone);
    let dark = light * vec3<f32>(0.62, 0.55, 0.5);
    var c = mix(light, dark, ring * p.rings);
    c *= 1.0 - p.grain * 0.25 * fine;
    o.base = srgb(c);
    o.roughness = 0.5 + 0.2 * ring;
    o.metallic = 0.0;
    return o;
}
"#;

pub const WATER: &str = r#"// Water: moving waves (the normal follows the slope of a noise height field).
// @param size = 400 (20..5000) "Wave size (mm)"
// @param height = 0.4 (0..2) "Wave height"
// @param speed = 0.6 (0..5) "Speed"
// @param depth = 0.5 (0..1) "Deep colour"
fn wave(x: vec2<f32>, t: f32) -> f32 {
    return fbm(vec3<f32>(x + vec2<f32>(t * 0.3, t * 0.2), t * 0.25), 4);
}

fn surface(s: Surface, p: Params) -> Surface {
    var o = s;
    let x = s.pos.xy / p.size;
    let t = s.time * p.speed;
    let e = 0.02;
    let h0 = wave(x, t);
    let dh = vec2<f32>(wave(x + vec2<f32>(e, 0.0), t) - h0, wave(x + vec2<f32>(0.0, e), t) - h0) / e;
    o.normal = bump(s.normal, dh * p.height * 0.25);
    o.base = mix(srgb(vec3<f32>(0.25, 0.55, 0.6)), srgb(vec3<f32>(0.02, 0.12, 0.2)), p.depth);
    o.roughness = 0.04;
    o.metallic = 0.0;
    return o;
}
"#;

pub const GLASS: &str = r#"// Reeded glass: vertical ribs that bend the reflections, a light tint.
// @param pitch = 12 (2..100) "Rib width (mm)"
// @param depth = 0.6 (0..2) "Rib depth"
// @param frost = 0.08 (0..1) "Frost"
// @param tint = 0.15 (0..1) "Green tint"
fn surface(s: Surface, p: Params) -> Surface {
    var o = s;
    let u = (s.pos.x + s.pos.y) / p.pitch;
    let slope = cos(fract(u) * 6.2831853) * p.depth;
    o.normal = bump(s.normal, vec2<f32>(slope, 0.0));
    o.base = mix(vec3<f32>(0.95), srgb(vec3<f32>(0.75, 0.9, 0.82)), p.tint);
    o.roughness = 0.02 + p.frost * 0.5;
    o.transmission = 1.0;
    o.metallic = 0.0;
    return o;
}
"#;

pub const GLOW: &str = r#"// Glow: a light panel that pulses slowly; optional grid lines.
// @param hue = 0.08 (0..1) "Hue"
// @param strength = 4 (0..30) "Brightness"
// @param pulse = 0.3 (0..1) "Pulse"
// @param lines = 0 (0..1) "Grid lines"
fn surface(s: Surface, p: Params) -> Surface {
    var o = s;
    let wave = 1.0 - p.pulse * (0.5 + 0.5 * sin(s.time * 2.0));
    let g = fract(s.uv / 250.0);
    let line = max(step(g.x, 0.04), step(g.y, 0.04));
    let k = mix(1.0, 1.0 - line * 0.85, p.lines);
    o.emission = hsv(p.hue, 0.75, 1.0) * p.strength * wave * k;
    o.base = vec3<f32>(0.05);
    o.roughness = 0.4;
    return o;
}
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn presets_compile() {
        for (name, src) in PRESETS {
            let params = varstotas_kernel::shader_params(src).into_iter().map(|p| p.name).collect();
            let sh = varstotas_render::CustomSurface { source: src.to_string(), params };
            assert_eq!(varstotas_render::shaders::check(&sh), None, "{name}");
        }
        assert_eq!(preset_name(WATER), Some("Water"));
    }
}
