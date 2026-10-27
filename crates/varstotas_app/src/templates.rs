//! Built-in unit presets and user document templates from `.vrst` files.

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

    pub fn name(self) -> String {
        varstotas_i18n::tr(match self {
            Template::ArchitecturalMm => "template-architectural-mm",
            Template::ArchitecturalM => "template-architectural-m",
            Template::WoodworkingMm => "template-woodworking-mm",
            Template::Imperial => "template-imperial",
        })
    }

    pub fn description(self) -> String {
        varstotas_i18n::tr(match self {
            Template::ArchitecturalMm => "template-architectural-mm-desc",
            Template::ArchitecturalM => "template-architectural-m-desc",
            Template::WoodworkingMm => "template-woodworking-mm-desc",
            Template::Imperial => "template-imperial-desc",
        })
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

/// The default user template directory (no example files are installed).
pub fn default_dir() -> Option<std::path::PathBuf> {
    crate::paths::data_dir().map(|p| p.join("templates"))
}

pub fn directory(custom: Option<&std::path::Path>) -> Option<std::path::PathBuf> {
    custom.map(std::path::Path::to_path_buf).or_else(default_dir)
}

#[derive(Debug)]
pub struct FileTemplate {
    pub name: String,
    pub path: std::path::PathBuf,
}

/// List the directory afresh so external file additions appear on the next menu opening.
pub fn scan(dir: &std::path::Path) -> std::io::Result<Vec<FileTemplate>> {
    let mut files = Vec::new();
    for entry in std::fs::read_dir(dir)? {
        let path = entry?.path();
        if path.is_file() && path.extension().is_some_and(|e| e.eq_ignore_ascii_case("vrst")) {
            let name = path.file_stem().unwrap_or_default().to_string_lossy().into_owned();
            files.push(FileTemplate { name, path });
        }
    }
    files.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()).then_with(|| a.path.cmp(&b.path)));
    Ok(files)
}

/// A template is a fresh unsaved document; it never becomes the Save target.
pub fn load_document(path: &std::path::Path) -> Result<crate::document::Document, varstotas_kernel::io::IoError> {
    let model = varstotas_kernel::io::load(path)?;
    let mut doc = crate::document::Document::from_model(model, None);
    doc.dirty = true;
    Ok(doc)
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
    fn temp_dir() -> std::path::PathBuf {
        static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let dir = std::env::temp_dir().join(format!("varstotas_templates_{}_{}", std::process::id(), NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn file_templates_scan_only_models_and_refresh() {
        let dir = temp_dir();
        std::fs::write(dir.join("zeta.vrst"), "").unwrap();
        std::fs::write(dir.join("Alpha.VRST"), "").unwrap();
        std::fs::write(dir.join("ignored.txt"), "").unwrap();
        std::fs::create_dir(dir.join("directory.vrst")).unwrap();
        assert_eq!(scan(&dir).unwrap().iter().map(|f| f.name.as_str()).collect::<Vec<_>>(), ["Alpha", "zeta"]);
        std::fs::write(dir.join("new.vrst"), "").unwrap();
        assert_eq!(scan(&dir).unwrap().len(), 3);
        assert_eq!(directory(Some(&dir)), Some(dir.clone()));
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn file_template_is_an_unsaved_copy_preserving_model_settings() {
        let dir = temp_dir();
        let path = dir.join("Room.vrst");
        let mut model = varstotas_kernel::Model::new();
        model.unit = LengthUnit::Meter;
        model.add_material("Template material", [0.2, 0.3, 0.4, 1.0]);
        model.render.samples = 64;
        varstotas_kernel::io::save(&model, &path).unwrap();
        let original = std::fs::read(&path).unwrap();
        let mut doc = load_document(&path).unwrap();
        assert!(doc.path.is_none());
        assert!(doc.dirty);
        assert_eq!(doc.history.undo_steps(), 0);
        assert_eq!(doc.model.unit, LengthUnit::Meter);
        assert_eq!(doc.model.render.samples, 64);
        assert!(doc.model.materials.values().any(|m| m.name == "Template material"));
        doc.model.unit = LengthUnit::Inch;
        varstotas_kernel::io::save(&doc.model, &dir.join("New document.vrst")).unwrap();
        assert_eq!(std::fs::read(&path).unwrap(), original);
        std::fs::write(dir.join("Broken.vrst"), "invalid JSON").unwrap();
        assert!(load_document(&dir.join("Broken.vrst")).is_err());
        assert!(load_document(&dir.join("Missing.vrst")).is_err());
        std::fs::remove_dir_all(dir).unwrap();
    }

}
