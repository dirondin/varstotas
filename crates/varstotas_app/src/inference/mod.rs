//! Inference (snapping) engine. Owner: `tools` agent.
//!
//! Placeholder: only the result types are defined here; the engine that collects candidates
//! (endpoints, midpoints, edges, faces, axes, parallels, guides) in screen space and ranks
//! them per docs/05-UX-SPEC.md will be implemented by the Tools agent.

#![allow(dead_code)]

use egui::Color32;
use varstotas_kernel::glam::DVec3;
use varstotas_kernel::{ContextPath, EntityRef};

/// Kind of snap, in priority order (see the inference table in docs/05-UX-SPEC.md).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum InferenceKind {
    Endpoint,
    Midpoint,
    Center,
    Intersection,
    OnEdge,
    OnRedAxis,
    OnGreenAxis,
    OnBlueAxis,
    ParallelToEdge,
    PerpendicularToEdge,
    FromPoint,
    OnFace,
    GuidePoint,
    OnGuide,
    /// No snap: point on the ground plane / at a default depth.
    Free,
}

impl InferenceKind {
    /// Tooltip text shown near the cursor.
    pub fn label(self) -> &'static str {
        match self {
            Self::Endpoint => "Endpoint",
            Self::Midpoint => "Midpoint",
            Self::Center => "Center",
            Self::Intersection => "Intersection",
            Self::OnEdge => "On Edge",
            Self::OnRedAxis => "On Red Axis",
            Self::OnGreenAxis => "On Green Axis",
            Self::OnBlueAxis => "On Blue Axis",
            Self::ParallelToEdge => "Parallel to Edge",
            Self::PerpendicularToEdge => "Perpendicular to Edge",
            Self::FromPoint => "From Point",
            Self::OnFace => "On Face",
            Self::GuidePoint => "Guide Point",
            Self::OnGuide => "On Guide",
            Self::Free => "",
        }
    }

    /// Marker colour per the UX spec.
    pub fn color(self) -> Color32 {
        match self {
            Self::Endpoint | Self::Center => Color32::from_rgb(0x00, 0xC0, 0x00),
            Self::Midpoint => Color32::from_rgb(0x00, 0xC0, 0xFF),
            Self::OnEdge | Self::Intersection | Self::OnRedAxis => Color32::from_rgb(0xFF, 0x00, 0x00),
            Self::OnGreenAxis => Color32::from_rgb(0x00, 0xB0, 0x00),
            Self::OnBlueAxis | Self::OnFace => Color32::from_rgb(0x00, 0x00, 0xFF),
            Self::ParallelToEdge | Self::PerpendicularToEdge => Color32::from_rgb(0xFF, 0x00, 0xFF),
            Self::FromPoint | Self::GuidePoint | Self::OnGuide => Color32::from_rgb(0x30, 0x30, 0x30),
            Self::Free => Color32::BLACK,
        }
    }
}

/// Entity the inference snapped to (in the context reached by `path`).
#[derive(Clone, Debug, PartialEq)]
pub struct InferenceRef {
    pub path: ContextPath,
    pub entity: EntityRef,
}

#[derive(Clone, Debug, PartialEq)]
pub struct InferenceResult {
    /// Snapped world-space point.
    pub point: DVec3,
    pub kind: InferenceKind,
    /// Direction constraint (axis / parallel / perpendicular), unit vector.
    pub constraint_dir: Option<DVec3>,
    pub ref_entity: Option<InferenceRef>,
}

impl InferenceResult {
    pub fn free(point: DVec3) -> Self {
        Self { point, kind: InferenceKind::Free, constraint_dir: None, ref_entity: None }
    }
}
