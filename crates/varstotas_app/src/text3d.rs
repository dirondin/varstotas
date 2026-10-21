//! 3D Text: glyph outlines of the fonts bundled with egui, flattened to closed polygons.
//! The kernel turns them into faces ([`varstotas_kernel::text3d`]).

use varstotas_kernel::glam::DVec2;
use skrifa::instance::{LocationRef, Size};
use skrifa::outline::{DrawSettings, OutlinePen};
use skrifa::{FontRef, MetadataProvider};

/// Segments per quadratic / cubic curve of a glyph.
const QUAD_STEPS: usize = 6;
const CUBIC_STEPS: usize = 8;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Text3dFont {
    Sans,
    Mono,
}

impl Text3dFont {
    pub const ALL: [Text3dFont; 2] = [Text3dFont::Sans, Text3dFont::Mono];

    pub fn name(self) -> &'static str {
        match self {
            Text3dFont::Sans => "Ubuntu Light", // i18n-ignore
            Text3dFont::Mono => "Hack", // i18n-ignore
        }
    }

    fn bytes(self) -> &'static [u8] {
        match self {
            Text3dFont::Sans => epaint_default_fonts::UBUNTU_LIGHT,
            Text3dFont::Mono => epaint_default_fonts::HACK_REGULAR,
        }
    }
}

/// The 3D Text dialog's values.
#[derive(Clone, Debug, PartialEq)]
pub struct Text3dParams {
    pub text: String,
    pub font: Text3dFont,
    /// Height of capital letters, mm.
    pub height: f64,
    /// Faces (otherwise only edges).
    pub filled: bool,
    pub extruded: bool,
    /// Extrusion, mm.
    pub depth: f64,
}

impl Default for Text3dParams {
    /// 10" high, extruded 2" (here 250 / 50 mm).
    fn default() -> Self {
        Self { text: "Enter text".into() /* i18n-ignore */, font: Text3dFont::Sans, height: 250.0, filled: true, extruded: true, depth: 50.0 }
    }
}

impl Text3dParams {
    pub fn options(&self) -> varstotas_kernel::Text3dOptions {
        varstotas_kernel::Text3dOptions {
            filled: self.filled,
            extrude: (self.filled && self.extruded && self.depth.abs() > 1e-6).then_some(self.depth),
        }
    }
}

#[derive(Default)]
struct Pen {
    loops: Vec<Vec<DVec2>>,
    cur: Vec<DVec2>,
    /// Glyph origin and scale (font units → mm).
    origin: DVec2,
    scale: f64,
}

impl Pen {
    fn pt(&self, x: f32, y: f32) -> DVec2 {
        self.origin + DVec2::new(x as f64, y as f64) * self.scale
    }
    fn last(&self) -> DVec2 {
        self.cur.last().copied().unwrap_or(self.origin)
    }
    fn finish(&mut self) {
        let mut l = std::mem::take(&mut self.cur);
        if l.len() > 1 && l[0].distance(l[l.len() - 1]) < 1e-9 {
            l.pop();
        }
        l.dedup_by(|a, b| a.distance(*b) < 1e-9);
        if l.len() >= 3 {
            self.loops.push(l);
        }
    }
}

impl OutlinePen for Pen {
    fn move_to(&mut self, x: f32, y: f32) {
        self.finish();
        let p = self.pt(x, y);
        self.cur.push(p);
    }
    fn line_to(&mut self, x: f32, y: f32) {
        let p = self.pt(x, y);
        self.cur.push(p);
    }
    fn quad_to(&mut self, cx0: f32, cy0: f32, x: f32, y: f32) {
        let (p0, c, p1) = (self.last(), self.pt(cx0, cy0), self.pt(x, y));
        for i in 1..=QUAD_STEPS {
            let t = i as f64 / QUAD_STEPS as f64;
            let u = 1.0 - t;
            self.cur.push(p0 * u * u + c * 2.0 * u * t + p1 * t * t);
        }
    }
    fn curve_to(&mut self, cx0: f32, cy0: f32, cx1: f32, cy1: f32, x: f32, y: f32) {
        let (p0, c0, c1, p1) = (self.last(), self.pt(cx0, cy0), self.pt(cx1, cy1), self.pt(x, y));
        for i in 1..=CUBIC_STEPS {
            let t = i as f64 / CUBIC_STEPS as f64;
            let u = 1.0 - t;
            self.cur.push(p0 * u * u * u + c0 * 3.0 * u * u * t + c1 * 3.0 * u * t * t + p1 * t * t * t);
        }
    }
    fn close(&mut self) {
        self.finish();
    }
}

/// Closed outlines of `p.text` in mm: the baseline of the first line is y = 0, lines go down,
/// x starts at 0. Unknown characters are skipped.
pub fn outlines(p: &Text3dParams) -> Vec<Vec<DVec2>> {
    let Ok(font) = FontRef::new(p.font.bytes()) else { return Vec::new() };
    let m = font.metrics(Size::unscaled(), LocationRef::default());
    let cap = m.cap_height.filter(|c| *c > 0.0).unwrap_or(m.units_per_em as f32 * 0.7) as f64;
    let scale = p.height.max(1e-3) / cap;
    let line_h = (m.ascent - m.descent + m.leading) as f64 * scale;
    let charmap = font.charmap();
    let glyphs = font.outline_glyphs();
    let gm = font.glyph_metrics(Size::unscaled(), LocationRef::default());
    let settings = || DrawSettings::unhinted(Size::unscaled(), LocationRef::default());
    let mut pen = Pen { scale, ..Default::default() };
    for (row, line) in p.text.lines().enumerate() {
        let mut x = 0.0;
        for ch in line.chars() {
            let Some(gid) = charmap.map(ch) else { continue };
            pen.origin = DVec2::new(x, -(row as f64) * line_h);
            if let Some(g) = glyphs.get(gid) {
                let _ = g.draw(settings(), &mut pen);
                pen.finish();
            }
            x += gm.advance_width(gid).unwrap_or(0.0) as f64 * scale;
        }
    }
    pen.loops
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn glyph_outlines() {
        let p = Text3dParams { text: "O".into(), ..Default::default() };
        let l = outlines(&p);
        assert_eq!(l.len(), 2, "outer contour and counter");
        let top = l.iter().flatten().map(|v| v.y).fold(f64::MIN, f64::max);
        assert!((top - 250.0).abs() < 15.0, "cap height ≈ 250 mm, got {top}");
        let p = Text3dParams { text: "i i\nB".into(), ..Default::default() };
        // i: stem + dot, twice; B: outer + two counters.
        assert_eq!(outlines(&p).len(), 2 + 2 + 3);
    }
}
