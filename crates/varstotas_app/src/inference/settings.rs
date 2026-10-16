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
pub const GROUPS: [(&str, &[InferenceKind]); 4] = {
    use InferenceKind::*;
    [
        ("Points", &[Endpoint, Midpoint, Center, Intersection, Origin]),
        ("On Objects", &[OnEdge, OnFace, OnGuide, GuidePoint]),
        ("Directions", &[OnRedAxis, OnGreenAxis, OnBlueAxis, ParallelToEdge, PerpendicularToEdge, FromPoint, Tangent]),
        ("Shapes", &[Square, GoldenSection, HalfCircle]),
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
