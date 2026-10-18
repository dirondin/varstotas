//! Procedural, tileable textures of the demo house: colour, normal and roughness maps drawn
//! in code (value noise, cells, boards, tiles) and encoded as PNG.

use varstotas_kernel::TextureImage;

/// A texture being drawn: sRGB colour, height (for the normal map) and roughness per pixel.
pub struct Tex {
    pub w: usize,
    pub h: usize,
    pub color: Vec<[f32; 3]>,
    pub height: Vec<f32>,
    pub rough: Vec<f32>,
}

/// The maps of one material, `width_mm` wide.
pub struct Maps {
    pub color: TextureImage,
    pub normal: Option<TextureImage>,
    pub rough: Option<TextureImage>,
    /// Average colour (the material colour in the non-textured styles).
    pub average: [f32; 3],
}

impl Tex {
    fn new(w: usize, h: usize, f: impl Fn(f32, f32) -> ([f32; 3], f32, f32)) -> Self {
        let n = w * h;
        let (mut color, mut height, mut rough) = (Vec::with_capacity(n), Vec::with_capacity(n), Vec::with_capacity(n));
        for y in 0..h {
            for x in 0..w {
                // Image rows go down; texture v goes up.
                let (c, ht, r) = f(x as f32 / w as f32, 1.0 - (y as f32 + 0.5) / h as f32);
                color.push(c);
                height.push(ht);
                rough.push(r);
            }
        }
        Self { w, h, color, height, rough }
    }

    /// Encode the maps; `bump` is the height of 1.0 in pixels (0 = no normal map), `rough`
    /// whether to write a roughness map.
    pub fn finish(&self, name: &str, width_mm: f64, bump: f32, rough: bool) -> Maps {
        let n = (self.w * self.h) as f32;
        let mut avg = [0.0f32; 3];
        for c in &self.color {
            for i in 0..3 {
                avg[i] += c[i] / n;
            }
        }
        let color = png(self.w, self.h, |i| self.color[i].map(byte));
        let normal = (bump > 0.0).then(|| {
            let (w, h) = (self.w as isize, self.h as isize);
            let at = |x: isize, y: isize| self.height[(y.rem_euclid(h) * w + x.rem_euclid(w)) as usize];
            png(self.w, self.h, |i| {
                let (x, y) = ((i % self.w) as isize, (i / self.w) as isize);
                let dx = (at(x + 1, y) - at(x - 1, y)) * 0.5 * bump;
                // Image y goes down, the normal map's green points up.
                let dy = (at(x, y - 1) - at(x, y + 1)) * 0.5 * bump;
                let len = (dx * dx + dy * dy + 1.0).sqrt();
                [byte(0.5 - 0.5 * dx / len), byte(0.5 - 0.5 * dy / len), byte(0.5 + 0.5 / len)]
            })
        });
        let rough_png = rough.then(|| png(self.w, self.h, |i| [byte(self.rough[i]); 3]));
        let image = |bytes: Vec<u8>, suffix: &str| TextureImage::new(bytes, format!("{name}{suffix}.png"), width_mm);
        Maps {
            color: image(color, ""),
            normal: normal.map(|b| image(b, "_normal")),
            rough: rough_png.map(|b| image(b, "_rough")),
            average: avg,
        }
    }
}

fn byte(v: f32) -> u8 {
    (v.clamp(0.0, 1.0) * 255.0 + 0.5) as u8
}

fn png(w: usize, h: usize, px: impl Fn(usize) -> [u8; 3]) -> Vec<u8> {
    let img = image::RgbImage::from_fn(w as u32, h as u32, |x, y| image::Rgb(px(y as usize * w + x as usize)));
    let mut out = Vec::new();
    img.write_to(&mut std::io::Cursor::new(&mut out), image::ImageFormat::Png).expect("PNG encoding");
    out
}

// ---- noise -------------------------------------------------------------------------------

fn hash(x: i32, y: i32, seed: u32) -> f32 {
    let mut h = (x as u32).wrapping_mul(0x8da6_b343) ^ (y as u32).wrapping_mul(0xd816_3841) ^ seed.wrapping_mul(0xcb1a_b31f);
    h ^= h >> 13;
    h = h.wrapping_mul(0x5bd1_e995);
    h ^= h >> 15;
    (h & 0xff_ffff) as f32 / 0x100_0000 as f32
}

fn smooth(t: f32) -> f32 {
    t * t * (3.0 - 2.0 * t)
}

/// Value noise 0..1 at `(x, y)` in lattice units, repeating every `px` × `py` cells.
fn vnoise(x: f32, y: f32, px: i32, py: i32, seed: u32) -> f32 {
    let (xi, yi) = (x.floor() as i32, y.floor() as i32);
    let (fx, fy) = (smooth(x - xi as f32), smooth(y - yi as f32));
    let h = |i: i32, j: i32| hash(i.rem_euclid(px), j.rem_euclid(py), seed);
    let a = h(xi, yi) + (h(xi + 1, yi) - h(xi, yi)) * fx;
    let b = h(xi, yi + 1) + (h(xi + 1, yi + 1) - h(xi, yi + 1)) * fx;
    a + (b - a) * fy
}

/// Tileable fractal noise 0..1 of `u, v` in 0..1 with `fx` × `fy` cells at the first octave.
fn fbm(u: f32, v: f32, fx: i32, fy: i32, octaves: u32, seed: u32) -> f32 {
    let (mut sum, mut amp, mut norm, mut k) = (0.0, 1.0, 0.0, 1);
    for o in 0..octaves {
        sum += amp * vnoise(u * (fx * k) as f32, v * (fy * k) as f32, fx * k, fy * k, seed + o * 101);
        norm += amp;
        amp *= 0.5;
        k *= 2;
    }
    sum / norm
}

/// Tileable cells (`n` × `n` jittered points): (distance to the nearest, to the second
/// nearest, id of the nearest), distances in cell units.
fn cells(u: f32, v: f32, n: i32, seed: u32) -> (f32, f32, u32) {
    let (x, y) = (u * n as f32, v * n as f32);
    let (xi, yi) = (x.floor() as i32, y.floor() as i32);
    let (mut d1, mut d2, mut id) = (9.0f32, 9.0f32, 0u32);
    for j in -1..=1 {
        for i in -1..=1 {
            let (cx, cy) = (xi + i, yi + j);
            let (wx, wy) = (cx.rem_euclid(n), cy.rem_euclid(n));
            let px = cx as f32 + 0.15 + 0.7 * hash(wx, wy, seed);
            let py = cy as f32 + 0.15 + 0.7 * hash(wx, wy, seed + 7);
            let d = ((px - x).powi(2) + (py - y).powi(2)).sqrt();
            if d < d1 {
                d2 = d1;
                d1 = d;
                id = (wy * n + wx) as u32;
            } else if d < d2 {
                d2 = d;
            }
        }
    }
    (d1, d2, id)
}

fn mix(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t, a[2] + (b[2] - a[2]) * t]
}

fn scale(c: [f32; 3], k: f32) -> [f32; 3] {
    c.map(|x| x * k)
}

fn smoothstep(a: f32, b: f32, x: f32) -> f32 {
    smooth(((x - a) / (b - a)).clamp(0.0, 1.0))
}

/// Wood grain along `u`: stretched noise and growth rings, 0..1 (dark lines high).
fn grain(u: f32, v: f32, rows: i32, seed: u32) -> f32 {
    let warp = fbm(u, v, 2, rows * 2, 3, seed);
    let rings = ((v * rows as f32 * 6.0 + warp * 5.0) * std::f32::consts::TAU).sin() * 0.5 + 0.5;
    let fine = fbm(u, v, 4, rows * 24, 2, seed + 50);
    (rings.powf(3.0) * 0.6 + fine * 0.4).clamp(0.0, 1.0)
}

// ---- the textures ------------------------------------------------------------------------

/// Painted horizontal clapboard siding: 6 boards per 1080 mm.
pub fn siding(color: [f32; 3]) -> Maps {
    let t = Tex::new(512, 512, |u, v| {
        let boards = 6.0;
        let b = v * boards;
        let t = b.fract();
        let row = b.floor() as i32;
        let g = grain(u, v, 6, 3 + row as u32);
        let lip = smoothstep(0.0, 0.05, t);
        let paint = 0.93 + 0.05 * fbm(u, v, 8, 8, 3, 11) - 0.05 * g;
        let c = scale(color, paint * (0.55 + 0.45 * lip));
        (c, t * 0.25 + 0.06 * g, 0.62 + 0.1 * g)
    });
    t.finish("siding", 1080.0, 6.0, true)
}

/// Rough stone in mortar.
pub fn stone() -> Maps {
    let t = Tex::new(512, 512, |u, v| {
        let (d1, d2, id) = cells(u, v, 6, 21);
        let edge = d2 - d1;
        let mortar = 1.0 - smoothstep(0.03, 0.09, edge);
        let r = hash(id as i32, 0, 5);
        let base = mix([0.55, 0.52, 0.47], [0.42, 0.4, 0.38], r);
        let base = mix(base, [0.6, 0.55, 0.46], hash(id as i32, 3, 9) * 0.5);
        let n = fbm(u, v, 16, 16, 4, 31);
        let c = mix(scale(base, 0.8 + 0.4 * n), [0.62, 0.6, 0.56], mortar);
        let h = (1.0 - mortar) * (0.6 + 0.4 * smoothstep(0.0, 0.25, edge)) + 0.25 * n;
        (c, h, 0.8 + 0.15 * mortar)
    });
    t.finish("stone", 1200.0, 10.0, false)
}

/// Graphite standing-seam metal roofing, a seam every 500 mm.
pub fn roof_metal() -> Maps {
    let t = Tex::new(512, 512, |u, v| {
        let s = (u * 2.0).fract();
        let seam = 1.0 - smoothstep(0.0, 0.03, s.min(1.0 - s));
        let n = fbm(u, v, 4, 32, 3, 41);
        let c = scale([0.2, 0.21, 0.22], 0.9 + 0.2 * n + 0.15 * seam);
        (c, seam + 0.02 * n, 0.38 + 0.15 * n)
    });
    t.finish("roof_metal", 1000.0, 8.0, true)
}

/// Oak floor boards, 180 mm wide, staggered joints.
pub fn oak_floor() -> Maps {
    let t = Tex::new(1024, 1024, |u, v| {
        let rows = 8;
        let b = v * rows as f32;
        let (row, t) = (b.floor() as i32, b.fract());
        let off = hash(row, 0, 61);
        let s = u + off;
        let plank = (s * 2.0).floor() as i32;
        let j = (s * 2.0).fract();
        let id = row * 7 + plank.rem_euclid(2);
        let tint = hash(id, 1, 62);
        let g = grain(s.fract(), v, rows, 70 + id as u32);
        let base = mix([0.74, 0.55, 0.35], [0.62, 0.43, 0.26], tint);
        let gap = (1.0 - smoothstep(0.0, 0.02, t.min(1.0 - t))).max(1.0 - smoothstep(0.0, 0.004, j.min(1.0 - j)));
        let c = scale(mix(base, [0.45, 0.3, 0.17], g * 0.55), 1.0 - 0.6 * gap);
        (c, 1.0 - gap + 0.05 * g, 0.42 + 0.15 * g + 0.2 * gap)
    });
    t.finish("oak_floor", 1440.0, 6.0, true)
}

/// Larch decking: 145 mm boards with gaps.
pub fn decking() -> Maps {
    let t = Tex::new(512, 512, |u, v| {
        let rows = 10;
        let b = v * rows as f32;
        let (row, t) = (b.floor() as i32, b.fract());
        let gap = 1.0 - smoothstep(0.02, 0.05, t.min(1.0 - t));
        let g = grain(u, v, rows, 80 + row as u32);
        let tint = hash(row, 2, 81);
        let base = mix([0.62, 0.47, 0.34], [0.55, 0.47, 0.4], tint);
        let c = scale(mix(base, [0.4, 0.3, 0.22], g * 0.5), 1.0 - 0.75 * gap);
        (c, 1.0 - gap + 0.08 * g, 0.7 + 0.1 * g)
    });
    t.finish("decking", 1500.0, 8.0, false)
}

/// Square tiles (`n` per tile) in grout.
pub fn tiles(name: &str, n: f32, width_mm: f64, color: [f32; 3], grout: [f32; 3]) -> Maps {
    let t = Tex::new(512, 512, |u, v| {
        let (x, y) = ((u * n).fract(), (v * n).fract());
        let id = (u * n).floor() as i32 * 31 + (v * n).floor() as i32;
        let e = x.min(1.0 - x).min(y.min(1.0 - y));
        let g = 1.0 - smoothstep(0.008, 0.016, e);
        let bevel = smoothstep(0.0, 0.05, e);
        let c = mix(scale(color, 0.97 + 0.05 * hash(id, 0, 91) + 0.02 * fbm(u, v, 8, 8, 2, 92)), grout, g);
        (c, (1.0 - g) * (0.6 + 0.4 * bevel), 0.12 + 0.7 * g)
    });
    t.finish(name, width_mm, 5.0, true)
}

/// Glazed subway tiles 75 × 150 mm, staggered.
pub fn subway() -> Maps {
    let t = Tex::new(512, 512, |u, v| {
        let rows = 8.0;
        let b = v * rows;
        let row = b.floor() as i32;
        let y = b.fract();
        let s = u * 4.0 + if row % 2 == 0 { 0.0 } else { 0.5 };
        let x = s.fract();
        let ex = x.min(1.0 - x) * 0.5;
        let e = ex.min(y.min(1.0 - y));
        let g = 1.0 - smoothstep(0.01, 0.02, e);
        let bevel = smoothstep(0.0, 0.12, e);
        let id = row * 13 + s.floor() as i32;
        let c = mix(scale([0.93, 0.94, 0.91], 0.97 + 0.04 * hash(id, 3, 93)), [0.75, 0.74, 0.7], g);
        (c, (1.0 - g) * (0.5 + 0.5 * bevel), 0.08 + 0.7 * g)
    });
    t.finish("subway_tile", 600.0, 6.0, true)
}

/// White marble with grey veins.
pub fn marble() -> Maps {
    let t = Tex::new(1024, 1024, |u, v| {
        let w = fbm(u, v, 3, 3, 5, 101);
        let vein = ((u + v * 0.6 + w * 1.8) * 3.0 * std::f32::consts::TAU).sin().abs();
        let fine = ((u * 0.4 - v + fbm(u, v, 6, 6, 4, 103) * 1.2) * 7.0 * std::f32::consts::TAU).sin().abs();
        let d = (1.0 - smoothstep(0.0, 0.06, vein)) * 0.7 + (1.0 - smoothstep(0.0, 0.03, fine)) * 0.3;
        let c = mix(scale([0.93, 0.92, 0.9], 0.97 + 0.04 * w), [0.45, 0.46, 0.48], d);
        (c, 0.0, 0.08 + 0.05 * d)
    });
    t.finish("marble", 1600.0, 0.0, true)
}

/// Furniture wood along `u` (no boards).
pub fn wood(name: &str, light: [f32; 3], dark: [f32; 3]) -> Maps {
    let t = Tex::new(512, 512, |u, v| {
        let g = grain(u, v, 3, 111);
        let c = mix(light, dark, g * 0.7 + 0.15 * fbm(u, v, 2, 6, 2, 112));
        (c, 0.1 * g, 0.5)
    });
    t.finish(name, 1000.0, 2.0, false)
}

/// Pine ceiling boards running up the slope (vertical in the image), 150 mm.
pub fn pine_boards() -> Maps {
    let t = Tex::new(512, 512, |u, v| {
        let n = 8.0;
        let b = u * n;
        let (col, t) = (b.floor() as i32, b.fract());
        let gap = 1.0 - smoothstep(0.0, 0.03, t.min(1.0 - t));
        let g = grain(v, u, 8, 120 + col as u32);
        let base = mix([0.86, 0.68, 0.46], [0.8, 0.6, 0.4], hash(col, 4, 121));
        let c = scale(mix(base, [0.64, 0.44, 0.26], g * 0.5), 1.0 - 0.5 * gap);
        (c, 1.0 - gap, 0.6)
    });
    t.finish("pine_boards", 1200.0, 5.0, false)
}

/// Woven fabric.
pub fn fabric(name: &str, color: [f32; 3]) -> Maps {
    let t = Tex::new(256, 256, |u, v| {
        let n = 64.0;
        let (x, y) = ((u * n).fract(), (v * n).fract());
        let over = ((u * n).floor() as i32 + (v * n).floor() as i32) % 2 == 0;
        let thread = if over { (x * std::f32::consts::PI).sin() } else { (y * std::f32::consts::PI).sin() };
        let slub = fbm(u, v, 8, 32, 2, 131);
        let c = scale(color, 0.82 + 0.14 * thread + 0.08 * slub);
        (c, thread * 0.5 + 0.2 * slub, 0.9)
    });
    t.finish(name, 200.0, 0.8, false)
}

/// A rug 2400 × 1700 mm: border stripes and a diamond field.
pub fn rug() -> Maps {
    let t = Tex::new(512, 363, |u, v| {
        let (cream, rust, char_) = ([0.86, 0.8, 0.68], [0.62, 0.3, 0.2], [0.2, 0.19, 0.18]);
        let (x, y) = (u.min(1.0 - u) * 2400.0, v.min(1.0 - v) * 1700.0);
        let b = x.min(y);
        let c = if b < 60.0 {
            char_
        } else if b < 90.0 {
            cream
        } else if b < 130.0 {
            rust
        } else if b < 170.0 {
            cream
        } else {
            let (dx, dy) = ((u * 8.0).fract() - 0.5, (v * 6.0).fract() - 0.5);
            let d = dx.abs() + dy.abs();
            if (0.32..0.38).contains(&d) { char_ } else if d < 0.12 { rust } else { cream }
        };
        let f = fbm(u, v, 96, 72, 2, 141);
        (scale(c, 0.85 + 0.2 * f), f, 0.95)
    });
    t.finish("rug", 2400.0, 2.0, false)
}

/// Lawn.
pub fn grass() -> Maps {
    let t = Tex::new(1024, 1024, |u, v| {
        let large = fbm(u, v, 5, 5, 4, 151);
        let blades = fbm(u, v, 320, 320, 2, 152);
        let base = mix([0.22, 0.36, 0.12], [0.36, 0.45, 0.17], large);
        let c = scale(mix(base, [0.42, 0.5, 0.22], blades * 0.4), 0.8 + 0.35 * blades);
        (c, blades, 0.9)
    });
    t.finish("grass", 5000.0, 4.0, false)
}

/// Gravel path.
pub fn gravel() -> Maps {
    let t = Tex::new(512, 512, |u, v| {
        let (d1, d2, id) = cells(u, v, 40, 161);
        let edge = d2 - d1;
        let r = hash(id as i32, 0, 162);
        let c = mix([0.62, 0.58, 0.52], [0.45, 0.43, 0.4], r);
        let c = mix(c, [0.72, 0.66, 0.56], hash(id as i32, 1, 163) * 0.6);
        let shade = smoothstep(0.0, 0.2, edge);
        (scale(c, 0.55 + 0.45 * shade), shade, 0.9)
    });
    t.finish("gravel", 800.0, 6.0, false)
}

/// Red brick 250 × 65 mm with 10 mm joints (4 bricks × 13 courses).
pub fn brick() -> Maps {
    let t = Tex::new(512, 480, |u, v| {
        let rows = 13.0;
        let b = v * rows;
        let row = b.floor() as i32;
        let y = b.fract();
        let s = u * 4.0 + if row % 2 == 0 { 0.0 } else { 0.5 };
        let x = s.fract();
        let joint = (1.0 - smoothstep(0.03, 0.05, x.min(1.0 - x))).max(1.0 - smoothstep(0.06, 0.12, y.min(1.0 - y)));
        let id = row * 17 + s.floor() as i32;
        let n = fbm(u, v, 24, 24, 3, 171);
        let base = mix([0.6, 0.28, 0.2], [0.45, 0.22, 0.17], hash(id, 0, 172));
        let c = mix(scale(base, 0.85 + 0.3 * n), [0.7, 0.67, 0.6], joint);
        (c, (1.0 - joint) * 0.8 + 0.2 * n, 0.85)
    });
    t.finish("brick", 1040.0, 6.0, false)
}

/// Interior lime plaster, slightly mottled.
pub fn plaster() -> Maps {
    let t = Tex::new(256, 256, |u, v| {
        let n = fbm(u, v, 6, 6, 4, 181);
        (scale([0.92, 0.9, 0.86], 0.97 + 0.05 * n), n, 0.9)
    });
    t.finish("plaster", 2000.0, 1.5, false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_are_png_with_their_sizes() {
        let m = siding([0.5, 0.6, 0.55]);
        assert_eq!(m.color.pixels, [512, 512]);
        assert_eq!(m.color.size, [1080.0, 1080.0]);
        assert!(m.normal.is_some() && m.rough.is_some());
        let b = brick();
        assert_eq!(b.color.pixels, [512, 480]);
        assert!((b.color.size[1] - 975.0).abs() < 1e-9);
        assert!(m.average.iter().all(|c| (0.2..0.8).contains(c)), "{:?}", m.average);
    }

    #[test]
    fn noise_tiles() {
        for (u, v) in [(0.0, 0.3), (0.5, 0.0)] {
            let a = fbm(u, v, 3, 5, 4, 1);
            let b = fbm(u + 1.0, v + 1.0, 3, 5, 4, 1);
            assert!((a - b).abs() < 1e-5);
        }
        let (d, _, id) = cells(0.001, 0.5, 6, 2);
        let (d2, _, id2) = cells(1.001, 0.5, 6, 2);
        assert!((d - d2).abs() < 1e-4 && id == id2);
    }
}
