//! Which inferences are on (View → Snapping and the magnet button on the toolbar).
//!
//! The preferences hold the master switch and the kinds turned off; the app publishes them
//! here once per frame (like the drawing axes), and the engine and the tools ask [`allowed`]
//! before offering a snap. Explicit constraints (arrow keys, a tool's own line) still work
//! with snapping off.

use varstotas_i18n::tr;
use super::InferenceKind;
use serde::{Deserialize, Serialize};
use std::cell::Cell;

/// Menu groups and their kinds, in menu order.
pub const GROUPS: [(&str, &[InferenceKind]); 5] = {
    use InferenceKind::*;
    [
        ("Points", &[Endpoint, Midpoint, Center, Intersection, Origin]), // i18n-ignore
        ("On Objects", &[OnEdge, OnFace, OnGuide, GuidePoint]), // i18n-ignore
        ("Directions", &[OnRedAxis, OnGreenAxis, OnBlueAxis, ParallelToEdge, PerpendicularToEdge, FromPoint, Tangent]), // i18n-ignore
        ("Shapes", &[Square, GoldenSection, HalfCircle]), // i18n-ignore
        ("Grid", &[GridPoint, OnGridLine]), // i18n-ignore
    ]
};

/// The menu title of a group of [`GROUPS`] (the group name is its id).
pub fn group_label(group: &str) -> String {
    tr!(match group {
        "Points" => "snap-group-points", // i18n-ignore
        "On Objects" => "snap-group-on-objects", // i18n-ignore
        "Directions" => "snap-group-directions", // i18n-ignore
        "Shapes" => "snap-group-shapes", // i18n-ignore
        _ => "snap-group-grid",
    })
}

/// Every kind that can be turned off.
pub fn all_kinds() -> impl Iterator<Item = InferenceKind> {
    GROUPS.iter().flat_map(|(_, k)| k.iter().copied())
}

impl InferenceKind {
    /// Stable name for the preferences and the harness.
    pub fn key(self) -> &'static str {
        match self {
            Self::Endpoint => "endpoint",
            Self::Midpoint => "midpoint",
            Self::Center => "center",
            Self::Intersection => "intersection",
            Self::Origin => "origin",
            Self::OnEdge => "edge",
            Self::OnFace => "face",
            Self::OnGuide => "guide",
            Self::GuidePoint => "guide_point",
            Self::OnRedAxis => "red",
            Self::OnGreenAxis => "green",
            Self::OnBlueAxis => "blue",
            Self::ParallelToEdge => "parallel",
            Self::PerpendicularToEdge => "perpendicular",
            Self::FromPoint => "from_point",
            Self::Tangent => "tangent",
            Self::Square => "square",
            Self::GoldenSection => "golden",
            Self::HalfCircle => "half_circle",
            Self::GridPoint => "grid_point",
            Self::OnGridLine => "grid_line",
            Self::Free => "free",
        }
    }

    pub fn from_key(key: &str) -> Option<Self> {
        all_kinds().find(|k| k.key() == key)
    }

    /// Name in the Snapping menu.
    pub fn menu_label(self) -> String {
        varstotas_i18n::tr(match self {
            Self::Endpoint => "snap-endpoint",
            Self::Midpoint => "snap-midpoint",
            Self::Center => "snap-center",
            Self::Intersection => "snap-intersection",
            Self::Origin => "snap-origin",
            Self::OnEdge => "snap-on-edge",
            Self::OnFace => "snap-on-face",
            Self::OnGuide => "snap-on-guide",
            Self::GuidePoint => "snap-guide-point",
            Self::OnRedAxis => "snap-on-red-axis",
            Self::OnGreenAxis => "snap-on-green-axis",
            Self::OnBlueAxis => "snap-on-blue-axis",
            Self::ParallelToEdge => "snap-parallel-to-edge",
            Self::PerpendicularToEdge => "snap-perpendicular-to-edge",
            Self::FromPoint => "snap-from-point",
            Self::Tangent => "snap-tangent",
            Self::Square => "snap-square",
            Self::GoldenSection => "snap-golden-section",
            Self::HalfCircle => "snap-half-circle",
            Self::GridPoint => "snap-grid-point",
            Self::OnGridLine => "snap-on-grid-line",
            Self::Free => return String::new(),
        })
    }
}

/// Snapping preferences (persisted with the UI preferences).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct SnapPrefs {
    /// Master switch.
    pub enabled: bool,
    /// Keys of the kinds turned off.
    pub off: Vec<String>,
    /// Move / Rotate / Scale by increments.
    pub increments: Increments,
}

impl Default for SnapPrefs {
    fn default() -> Self {
        Self { enabled: true, off: Vec::new(), increments: Increments::default() }
    }
}

/// Increments (magnet menu): Move / Copy distances, Rotate and Protractor angles and Scale
/// factors are rounded to a step unless the point snaps to model geometry or a value is typed.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Increments {
    pub move_on: bool,
    /// Distance step (mm).
    pub move_step: f64,
    pub rotate_on: bool,
    /// Angle step (degrees).
    pub angle_step: f64,
    pub scale_on: bool,
    pub scale_step: f64,
}

impl Default for Increments {
    fn default() -> Self {
        Self { move_on: false, move_step: 10.0, rotate_on: false, angle_step: 15.0, scale_on: false, scale_step: 0.1 }
    }
}

/// `v` rounded to a multiple of `step` (unchanged for a bad step).
pub fn round_to(v: f64, step: f64) -> f64 {
    if step > 1e-9 && step.is_finite() { (v / step).round() * step } else { v }
}

impl SnapPrefs {
    pub fn is_on(&self, kind: InferenceKind) -> bool {
        !self.off.iter().any(|k| k == kind.key())
    }

    pub fn set(&mut self, kind: InferenceKind, on: bool) {
        self.off.retain(|k| k != kind.key());
        if !on {
            self.off.push(kind.key().to_string());
            self.off.sort();
        }
    }

    /// Some kinds are off (the magnet shows it).
    pub fn partial(&self) -> bool {
        all_kinds().any(|k| !self.is_on(k))
    }

    fn mask(&self) -> u32 {
        all_kinds().filter(|k| !self.is_on(*k)).fold(0, |m, k| m | bit(k))
    }
}

/// View → Grid: shown, finest step (mm) and the major line ratio (persisted with the prefs).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct GridPrefs {
    pub show: bool,
    pub step: f64,
    pub major: u32,
}

impl Default for GridPrefs {
    fn default() -> Self {
        Self { show: true, step: 10.0, major: 10 }
    }
}

impl GridPrefs {
    /// The grid on the drawing axes' red-green plane (work planes come later).
    pub fn style(&self) -> Option<varstotas_render::GridStyle> {
        if !self.show || !(self.step > 0.0 && self.step.is_finite()) {
            return None;
        }
        let ax = crate::axes::current();
        Some(varstotas_render::GridStyle {
            origin: ax.origin.to_array(),
            u: ax.x.to_array(),
            v: ax.y.to_array(),
            step: self.step,
            major: self.major.max(2),
            color: [0.22, 0.25, 0.30, 0.55],
        })
    }
}

thread_local! {
    static GRID: Cell<Option<(f64, u32)>> = const { Cell::new(None) };
}

/// Publish the grid (with [`publish`]); the engine snaps to it when it is shown.
pub fn publish_grid(g: &GridPrefs) {
    GRID.with(|c| c.set(g.show.then_some((g.step, g.major))));
}

/// The shown grid, for snapping.
pub fn grid() -> Option<varstotas_render::GridStyle> {
    let (step, major) = GRID.with(|c| c.get())?;
    GridPrefs { show: true, step, major }.style()
}

fn bit(kind: InferenceKind) -> u32 {
    1 << (kind as u32)
}

thread_local! {
    /// (master switch, mask of the kinds turned off).
    static CURRENT: Cell<(bool, u32)> = const { Cell::new((true, 0)) };
    static INCREMENTS: Cell<Option<Increments>> = const { Cell::new(None) };
}

/// Publish the preferences (called every frame and after they change).
pub fn publish(p: &SnapPrefs) {
    CURRENT.with(|c| c.set((p.enabled, p.mask())));
    INCREMENTS.with(|c| c.set(p.enabled.then_some(p.increments)));
}

fn step(on: fn(&Increments) -> bool, step: fn(&Increments) -> f64) -> Option<f64> {
    let i = INCREMENTS.with(|c| c.get())?;
    let s = step(&i);
    (on(&i) && s > 1e-9 && s.is_finite()).then_some(s)
}

/// Move / Copy distance step (mm), when on.
pub fn move_step() -> Option<f64> {
    step(|i| i.move_on, |i| i.move_step)
}

/// Rotate / Protractor angle step (degrees), when on.
pub fn angle_step() -> Option<f64> {
    step(|i| i.rotate_on, |i| i.angle_step)
}

/// Scale factor step, when on.
pub fn scale_step() -> Option<f64> {
    step(|i| i.scale_on, |i| i.scale_step)
}

/// A free Move / Copy destination rounded by the distance step: along the inferred direction,
/// else per drawing-axes component. Snaps to model geometry are kept as they are.
pub fn round_move(from: varstotas_kernel::glam::DVec3, r: &mut super::InferenceResult) {
    let Some(s) = move_step() else { return };
    if r.is_geometry_snap() {
        return;
    }
    let d = r.point - from;
    r.point = match r.constraint_dir {
        Some(dir) => from + dir * round_to(d.dot(dir), s),
        None => {
            let ax = crate::axes::current();
            let l = ax.local_vec(d);
            from + ax.world_vec(varstotas_kernel::glam::DVec3::new(round_to(l.x, s), round_to(l.y, s), round_to(l.z, s)))
        }
    };
}

/// Whether the engine and the tools may offer this kind of snap.
pub fn allowed(kind: InferenceKind) -> bool {
    if kind == InferenceKind::Free {
        return true;
    }
    let (on, mask) = CURRENT.with(|c| c.get());
    on && mask & bit(kind) == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keys_round_trip_and_mask() {
        for k in all_kinds() {
            assert_eq!(InferenceKind::from_key(k.key()), Some(k));
        }
        let mut p = SnapPrefs::default();
        assert!(!p.partial());
        p.set(InferenceKind::Midpoint, false);
        p.set(InferenceKind::Midpoint, false);
        assert_eq!(p.off, vec!["midpoint".to_string()]);
        assert!(p.partial());
        publish(&p);
        assert!(!allowed(InferenceKind::Midpoint) && allowed(InferenceKind::Endpoint));
        p.enabled = false;
        publish(&p);
        assert!(!allowed(InferenceKind::Endpoint) && allowed(InferenceKind::Free));
        publish(&SnapPrefs::default());
    }

    #[test]
    fn increments_round_free_moves_only() {
        use varstotas_kernel::glam::DVec3;
        let mut p = SnapPrefs::default();
        publish(&p);
        assert_eq!(move_step(), None);
        p.increments.move_on = true;
        p.increments.rotate_on = true;
        publish(&p);
        assert_eq!((move_step(), angle_step(), scale_step()), (Some(10.0), Some(15.0), None));
        let mut r = super::super::InferenceResult::free(DVec3::new(123.4, -56.0, 4.9));
        round_move(DVec3::new(1.0, 0.0, 0.0), &mut r);
        assert!(r.point.distance(DVec3::new(121.0, -60.0, 0.0)) < 1e-9);
        // Along a direction: the length is rounded.
        let mut r = super::super::InferenceResult::with_kind(DVec3::new(30.0, 40.0, 0.0) * 0.987, InferenceKind::ParallelToEdge);
        r.constraint_dir = Some(DVec3::new(0.6, 0.8, 0.0));
        round_move(DVec3::ZERO, &mut r);
        assert!(r.point.distance(DVec3::new(30.0, 40.0, 0.0)) < 1e-9);
        // Model geometry wins.
        let mut r = super::super::InferenceResult::with_kind(DVec3::new(3.0, 0.0, 0.0), InferenceKind::Endpoint);
        round_move(DVec3::ZERO, &mut r);
        assert_eq!(r.point, DVec3::new(3.0, 0.0, 0.0));
        p.enabled = false;
        publish(&p);
        assert_eq!(move_step(), None);
        publish(&SnapPrefs::default());
    }
}
