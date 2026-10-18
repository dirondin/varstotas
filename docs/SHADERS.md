# Custom surface shaders

In the Realistic style a material can run a small WGSL program that changes its surface pixel
by pixel. Use it for procedural patterns (wood, tiles), animated water, ribbed glass or glowing
panels. Open Materials → Physical (Realistic) → **Shader**. Pick a bundled preset (Wood, Water,
Reeded Glass, Glow) or press **Edit…** to write your own. The shader is saved with the material:
in the model file, in its components and in scenes.

## The function

A shader defines one function:

```wgsl
fn surface(s: Surface, p: Params) -> Surface {
    var o = s;
    // change o.base, o.roughness, o.normal, …
    return o;
}
```

It gets the surface as the material made it, with its maps and colour or texture applied. It
returns the changed surface, which then goes through the normal lighting: environment, sun,
shadows and effects.

```wgsl
struct Surface {
    base: vec3<f32>,     // linear base colour
    alpha: f32,          // opacity
    metallic: f32,
    roughness: f32,
    transmission: f32,   // glass: 1 = light passes through
    emission: vec3<f32>, // linear, strength included
    normal: vec3<f32>,   // world, unit, facing the viewer
    pos: vec3<f32>,      // model position, mm (z = up)
    uv: vec2<f32>,       // planar texture coordinates, mm
    view: vec3<f32>,     // unit direction toward the eye
    time: f32,           // seconds (the view keeps redrawing while a shader uses it)
};
```

Your shader can also declare its own helper functions, `const`s and `struct`s. Varstotas renames
them per shader, so names never clash between materials.

## Parameters

Comment lines at the top become sliders in the material editor and fields of `p`:

```wgsl
// @param spacing = 18 (3..120) "Ring spacing (mm)"
// @param rings = 0.55 "Ring contrast"
```

The format is `name = default (min..max) "Label"`. The range and the label are optional. Without
a range the slider runs 0..1 when the default is inside it, otherwise it spans around the default.
A shader can have at most 8 parameters. When you edit the code, parameters keep their values by name.

## Helpers

| Function | Meaning |
|---|---|
| `hash3(p: vec3) -> f32` | pseudo-random value 0..1 per point |
| `noise(p: vec3) -> f32` | smooth value noise 0..1 |
| `fbm(p: vec3, octaves: i32) -> f32` | layered noise (clouds, marble, warping) |
| `srgb(c: vec3) -> vec3` | colour picked in sRGB (0..1) → linear |
| `hsv(h, s, v) -> vec3` | hue / saturation / value → linear colour |
| `tangent_of(n) -> vec3` | a unit vector across `n` |
| `bump(n, dh: vec2) -> vec3` | `n` tilted by the slope of a height field |

You can also use any WGSL built-in (`fract`, `smoothstep`, `mix`, `sin`, …).

## The editor

The shader window highlights WGSL and completes as you type: after `s.` or `o.` it lists the
`Surface` fields, after `p.` your parameters, after `s.pos.` the swizzles, and on a `// @` line
the parameter tag. ↑ / ↓ pick, Tab or Enter inserts, Esc closes the list. Inside a helper call
(`fbm(`, `noise(`, …) a hint shows the signature with the current argument. **Start from…**
replaces the code with a preset, and **Reference** shows the fields and functions beside the code.

## Errors

The editor applies the code half a second after you stop typing, or when you press **Apply**, but
only when it compiles. A shader that sets `transmission` makes the material see-through (its faces are drawn blended, like glass). Otherwise the error shows under the code with its line number, and the
material keeps its last working shader. If a saved shader stops compiling (for example after a
Varstotas update), the material editor shows the error and the surface renders without the shader.

## Examples

Here is a checker pattern; **Edit…** on a material without a shader starts with it:

```wgsl
// @param size = 200 (10..2000) "Square size (mm)"
// @param contrast = 0.5 (0..1) "Contrast"
fn surface(s: Surface, p: Params) -> Surface {
    var o = s;
    let c = floor(s.pos / p.size);
    let odd = fract((c.x + c.y + c.z) * 0.5) * 2.0;
    o.base = s.base * (1.0 - p.contrast * odd);
    return o;
}
```

To see what the presets do, pick one and press **Edit…**. Their source lives in
`crates/varstotas_app/src/shaders.rs`.

The path tracer (Window → Render…) runs the same `surface` function at every hit, so a
custom material looks the same in a render; `s.view` is the direction back along the ray.
