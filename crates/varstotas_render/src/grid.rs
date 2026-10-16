//! Drawing grid on a plane (View → Grid): drawn by the `fs_grid` pass, levels shared with the
//! app's grid snapping so the cursor snaps to the lines that are visible.

use glam::DVec3;

/// Grid for a frame: a plane through `origin` spanned by the unit axes `u` and `v`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GridStyle {
    pub origin: [f64; 3],
    pub u: [f64; 3],
    pub v: [f64; 3],
    /// Distance between the finest lines (world units).
    pub step: f64,
    /// Every `major`-th line is a major line; zooming out multiplies the step by it.
    pub major: u32,
    /// Line colour (sRGB, alpha = strength of the major lines).
    pub color: [f32; 4],
}

/// Spacing (px) below which the finest drawn level starts to fade out (gone at
/// `GRID_MIN_PX / major`).
pub const GRID_MIN_PX: f64 = 40.0;

impl GridStyle {
    pub fn normal(&self) -> DVec3 {
        DVec3::from_array(self.u).cross(DVec3::from_array(self.v)).normalize_or(DVec3::Z)
    }

    fn ratio(&self) -> f64 {
        self.major.max(2) as f64
    }

    /// Level of the grid for a world footprint of one pixel: `(k, f)` — lines every
    /// `step * major^k` fade out as `f` goes 0 → 1 (the same formula as the shader).
    pub fn level(&self, pixel: f64) -> (i32, f64) {
        let step = self.step.max(1e-9);
        let l = ((pixel * GRID_MIN_PX / step).max(1e-12)).ln() / self.ratio().ln();
        if l <= 0.0 { (0, 0.0) } else { (l.floor() as i32, l.fract()) }
    }

    /// The step of the clearly visible lines for a pixel footprint (what the cursor snaps to).
    pub fn snap_step(&self, pixel: f64) -> f64 {
        let (k, f) = self.level(pixel);
        let k = if f > 0.5 { k + 1 } else { k };
        self.step * self.ratio().powi(k)
    }

    /// The grid origin moved by whole coarse cells close to `near`, so the shader's `f32`
    /// plane coordinates stay small around the camera.
    pub fn origin_near(&self, near: DVec3, extent: f64) -> DVec3 {
        let o = DVec3::from_array(self.origin);
        let (u, v) = (DVec3::from_array(self.u), DVec3::from_array(self.v));
        let mut cell = self.step.max(1e-9);
        while cell < extent && cell < 1e12 {
            cell *= self.ratio();
        }
        let d = near - o;
        o + u * ((d.dot(u) / cell).round() * cell) + v * ((d.dot(v) / cell).round() * cell)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn grid() -> GridStyle {
        GridStyle { origin: [0.0; 3], u: [1.0, 0.0, 0.0], v: [0.0, 1.0, 0.0], step: 10.0, major: 10, color: [0.0; 4] }
    }

    #[test]
    fn levels_and_snap_step() {
        let g = grid();
        assert_eq!(g.level(0.2), (0, 0.0));
        assert_eq!(g.snap_step(0.2), 10.0);
        // 1 px = 0.5 mm: 10 mm lines are 20 px apart → level 0 half faded (f ≈ 0.3), still 10 mm.
        let (k, f) = g.level(0.5);
        assert_eq!(k, 0);
        assert!(f > 0.25 && f < 0.35);
        assert_eq!(g.snap_step(0.5), 10.0);
        // 1 px = 2 mm: 10 mm lines 5 px apart (nearly gone) → 100 mm.
        assert_eq!(g.snap_step(2.0), 100.0);
        // 1 px = 50 mm: 1 m lines 20 px apart → 1 m.
        assert_eq!(g.snap_step(50.0), 1000.0);
    }

    #[test]
    fn origin_near_keeps_alignment() {
        let g = grid();
        let o = g.origin_near(DVec3::new(123_456.0, -98_765.0, 0.0), 500.0);
        assert_eq!(o, DVec3::new(123_000.0, -99_000.0, 0.0));
        assert_eq!(o.x % 10.0, 0.0);
    }
}
