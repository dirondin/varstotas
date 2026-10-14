//! Model templates: the starting point of File → New (units + initial camera scale).

use varstotas_kernel::LengthUnit;
use varstotas_render::Camera;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Template {
    ArchitecturalMm,
    ArchitecturalM,
    WoodworkingMm,
    Imperial,
}

impl Template {
    pub const ALL: [Template; 4] =
        [Template::ArchitecturalMm, Template::ArchitecturalM, Template::WoodworkingMm, Template::Imperial];

    pub fn name(self) -> &'static str {
        match self {
            Template::ArchitecturalMm => "Architectural — Millimeters",
            Template::ArchitecturalM => "Architectural — Meters",
            Template::WoodworkingMm => "Woodworking — Millimeters",
            Template::Imperial => "Imperial — Inches",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Template::ArchitecturalMm => "Buildings and interiors, lengths in millimeters.",
            Template::ArchitecturalM => "Buildings and sites, lengths in meters.",
            Template::WoodworkingMm => "Furniture and small objects: a close-up view, lengths in millimeters.",
            Template::Imperial => "Lengths in inches.",
        }
    }

    pub fn unit(self) -> LengthUnit {
        match self {
            Template::ArchitecturalMm | Template::WoodworkingMm => LengthUnit::Millimeter,
            Template::ArchitecturalM => LengthUnit::Meter,
            Template::Imperial => LengthUnit::Inch,
        }
    }

    /// The initial camera for a new model of this template (woodworking starts close up).
    pub fn camera(self, base: Camera) -> Camera {
        let k = match self {
            Template::WoodworkingMm => 0.2,
            _ => 1.0,
        };
        let mut cam = base;
        cam.eye = cam.target * k + (cam.eye - cam.target) * k;
        cam.target *= k;
        cam.ortho_height *= k;
        cam
    }

    /// A new, empty model set up for this template.
    pub fn new_model(self) -> varstotas_kernel::Model {
        let mut m = varstotas_kernel::Model::new();
        m.unit = self.unit();
        m
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn templates_set_units_and_camera_scale() {
        assert_eq!(Template::ArchitecturalM.new_model().unit, LengthUnit::Meter);
        assert_eq!(Template::Imperial.new_model().unit, LengthUnit::Inch);
        let base = crate::nav::startup_camera();
        let wood = Template::WoodworkingMm.camera(base.clone());
        let d0 = (base.eye - base.target).length();
        let d1 = (wood.eye - wood.target).length();
        assert!((d1 - d0 * 0.2).abs() < 1e-6);
        let arch = Template::ArchitecturalMm.camera(base.clone());
        assert_eq!(arch.eye, base.eye);
    }
}
