//! The drawing axes (the classic Axes tool): origin plus red / green / blue directions.
//!
//! The model stores them; the app publishes the current document's axes here once per frame
//! (and before tools run), so inference, axis locks and drawing planes use them without
//! threading the model through every helper.

use varstotas_kernel::glam::DVec3;
use std::cell::Cell;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Axes {
    pub origin: DVec3,
    /// Red, green, blue unit directions (right-handed, orthonormal).
    pub x: DVec3,
    pub y: DVec3,
    pub z: DVec3,
}

impl Default for Axes {
    fn default() -> Self {
        Self::WORLD
    }
}

impl Axes {
    pub const WORLD: Axes = Axes { origin: DVec3::ZERO, x: DVec3::X, y: DVec3::Y, z: DVec3::Z };

    /// Axes from an origin, a red direction and a point/direction roughly along green:
    /// green is made perpendicular to red, blue = red × green. None when degenerate.
    pub fn from_dirs(origin: DVec3, red: DVec3, green_hint: DVec3) -> Option<Axes> {
        let x = red.normalize_or_zero();
        if x == DVec3::ZERO {
            return None;
        }
        let y = (green_hint - x * green_hint.dot(x)).normalize_or_zero();
        if y == DVec3::ZERO {
            return None;
        }
        let z = x.cross(y).normalize();
        Some(Axes { origin, x, y, z })
    }

    /// Direction of axis `i` (0 = red, 1 = green, 2 = blue).
    pub fn dir(&self, i: usize) -> DVec3 {
        [self.x, self.y, self.z][i.min(2)]
    }

    pub fn dirs(&self) -> [DVec3; 3] {
        [self.x, self.y, self.z]
    }

    /// World vector expressed in axes coordinates.
    pub fn local_vec(&self, v: DVec3) -> DVec3 {
        DVec3::new(v.dot(self.x), v.dot(self.y), v.dot(self.z))
    }

    /// Axes-coordinate vector → world vector.
    pub fn world_vec(&self, v: DVec3) -> DVec3 {
        self.x * v.x + self.y * v.y + self.z * v.z
    }

    /// The model's drawing axes: the active work plane's frame, else the drawing axes.
    pub fn of_model(m: &varstotas_kernel::Model) -> Axes {
        let f = m.drawing_frame();
        Axes { origin: f.translation, x: f.matrix3.x_axis, y: f.matrix3.y_axis, z: f.matrix3.z_axis }
    }

    pub fn is_valid(&self) -> bool {
        let ok = |v: DVec3| v.is_finite() && (v.length() - 1.0).abs() < 1e-6;
        ok(self.x)
            && ok(self.y)
            && ok(self.z)
            && self.origin.is_finite()
            && self.x.dot(self.y).abs() < 1e-6
            && self.x.cross(self.y).dot(self.z) > 0.0
    }
}

thread_local! {
    static CURRENT: Cell<Axes> = const { Cell::new(Axes::WORLD) };
}

/// Axes of the current document (world axes by default).
pub fn current() -> Axes {
    CURRENT.with(|c| c.get())
}

/// Publish the model's axes (called every frame and after actions that may change them).
pub fn sync(m: &varstotas_kernel::Model) {
    set_current(Axes::of_model(m));
}

/// Publish the current document's axes (invalid axes fall back to the world axes).
pub fn set_current(a: Axes) {
    CURRENT.with(|c| c.set(if a.is_valid() { a } else { Axes::WORLD }));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_dirs_orthonormalizes() {
        let a = Axes::from_dirs(DVec3::new(1.0, 2.0, 3.0), DVec3::new(2.0, 0.0, 0.0), DVec3::new(1.0, 1.0, 0.0)).unwrap();
        assert!((a.x - DVec3::X).length() < 1e-12 && (a.y - DVec3::Y).length() < 1e-12 && (a.z - DVec3::Z).length() < 1e-12);
        assert!(a.is_valid());
        assert!(Axes::from_dirs(DVec3::ZERO, DVec3::X, DVec3::X).is_none());
        let r = Axes::from_dirs(DVec3::ZERO, DVec3::new(1.0, 1.0, 0.0), DVec3::new(-1.0, 1.0, 0.0)).unwrap();
        assert!((r.local_vec(DVec3::new(1.0, 1.0, 0.0)) - DVec3::new(2f64.sqrt(), 0.0, 0.0)).length() < 1e-12);
        assert!((r.world_vec(DVec3::X) - r.x).length() < 1e-12);
    }
}
