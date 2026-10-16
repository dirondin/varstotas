//! Which inferences are on (View → Snapping and the magnet button on the toolbar).
//!
//! The preferences hold the master switch and the kinds turned off; the app publishes them
//! here once per frame (like the drawing axes), and the engine and the tools ask [`allowed`]
//! before offering a snap. Explicit constraints (arrow keys, a tool's own line) still work
//! with snapping off.

use super::InferenceKind;
use serde::{Deserialize, Serialize};
use std::cell::Cell;

/// Menu groups and their kinds, in menu order.
pub const GROUPS: [(&str, &[InferenceKind]); 5] = {
    use InferenceKind::*;
    [
        ("Points", &[Endpoint, Midpoint, Center, Intersection, Origin]),
        ("On Objects", &[OnEdge, OnFace, OnGuide, GuidePoint]),
        ("Directions", &[OnRedAxis, OnGreenAxis, OnBlueAxis, ParallelToEdge, PerpendicularToEdge, FromPoint, Tangent]),
        ("Shapes", &[Square, GoldenSection, HalfCircle]),
        ("Grid", &[GridPoint, OnGridLine]),
    ]
};

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
    pub fn menu_label(self) -> &'static str {
        match self {
            Self::Endpoint => "Endpoints",
            Self::Midpoint => "Midpoints",
            Self::Center => "Centers",
            Self::Intersection => "Intersections",
            Self::Origin => "Origin",
            Self::OnEdge => "On Edge",
            Self::OnFace => "On Face",
            Self::OnGuide => "On Guide Lines",
            Self::GuidePoint => "Guide Points",
            Self::OnRedAxis => "Red Axis",
            Self::OnGreenAxis => "Green Axis",
            Self::OnBlueAxis => "Blue Axis",
            Self::ParallelToEdge => "Parallel to Edge",
            Self::PerpendicularToEdge => "Perpendicular to Edge",
            Self::FromPoint => "From Point",
            Self::Tangent => "Tangent at Vertex",
            Self::Square => "Square",
            Self::GoldenSection => "Golden Section",
            Self::HalfCircle => "Half Circle",
            Self::GridPoint => "Grid Points",
            Self::OnGridLine => "Grid Lines",
            Self::Free => "",
        }
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
}

impl Default for SnapPrefs {
    fn default() -> Self {
        Self { enabled: true, off: Vec::new() }
    }
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
        Self { show: false, step: 10.0, major: 10 }
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
}

/// Publish the preferences (called every frame and after they change).
pub fn publish(p: &SnapPrefs) {
    CURRENT.with(|c| c.set((p.enabled, p.mask())));
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
}
