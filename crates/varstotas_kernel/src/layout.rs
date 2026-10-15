//! Layout sheets (a light page layout): paper sheets stored in the document with viewports of
//! the model (a scene, or a view captured when the viewport was made), texts and a title
//! block. Drawing them on paper is the app's job (`varstotas_app::layout`); this module only
//! keeps the data and the scene state a viewport shows.

use crate::model::{Model, Scene};
use glam::DVec2;
use serde::{Deserialize, Serialize};

/// One sheet of paper. Coordinates are millimetres from the top-left corner, y down.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LayoutSheet {
    pub name: String,
    /// Paper size, mm.
    pub size: DVec2,
    #[serde(default)]
    pub viewports: Vec<SheetViewport>,
    #[serde(default)]
    pub texts: Vec<SheetText>,
    /// Title block in the bottom-right corner (`None`: no block).
    #[serde(default)]
    pub title: Option<TitleBlock>,
}

impl LayoutSheet {
    pub fn new(name: impl Into<String>, size: DVec2) -> Self {
        Self { name: name.into(), size, viewports: Vec::new(), texts: Vec::new(), title: Some(TitleBlock::default()) }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SheetViewport {
    /// Top-left corner and size on the paper, mm.
    pub min: DVec2,
    pub size: DVec2,
    /// What it shows when `scene` is unset or missing: camera, hidden tags, section state.
    pub view: Scene,
    /// Name of a model scene to follow (its later changes show on the sheet).
    #[serde(default)]
    pub scene: Option<String>,
    /// Scale 1:N (parallel projection only); `None`: fit the model into the viewport.
    #[serde(default)]
    pub scale: Option<f64>,
    #[serde(default = "yes")]
    pub profiles: bool,
    #[serde(default)]
    pub hidden: bool,
    #[serde(default = "yes")]
    pub dims: bool,
    #[serde(default = "yes")]
    pub sections: bool,
    /// Thin frame around the viewport.
    #[serde(default)]
    pub frame: bool,
    /// Caption under the viewport: the scene name and the scale.
    #[serde(default = "yes")]
    pub caption: bool,
}

fn yes() -> bool {
    true
}

impl SheetViewport {
    pub fn new(min: DVec2, size: DVec2, view: Scene) -> Self {
        Self { min, size, view, scene: None, scale: None, profiles: true, hidden: false, dims: true, sections: true, frame: false, caption: true }
    }

    pub fn contains(&self, p: DVec2) -> bool {
        p.x >= self.min.x && p.y >= self.min.y && p.x <= self.min.x + self.size.x && p.y <= self.min.y + self.size.y
    }
}

/// A text on the sheet (one or more lines; `at` is the start of the first baseline).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SheetText {
    pub at: DVec2,
    /// Text height, mm.
    pub size: f64,
    pub text: String,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct TitleBlock {
    pub title: String,
    pub author: String,
    pub date: String,
}

/// Paper sizes, portrait, mm.
pub const PAPERS: [(&str, DVec2); 4] = [
    ("A4", DVec2::new(210.0, 297.0)),
    ("A3", DVec2::new(297.0, 420.0)),
    ("A2", DVec2::new(420.0, 594.0)),
    ("A1", DVec2::new(594.0, 841.0)),
];

impl Model {
    /// The scene a viewport shows: the model scene it follows, else its own view.
    pub fn viewport_scene<'a>(&'a self, vp: &'a SheetViewport) -> &'a Scene {
        vp.scene.as_ref().and_then(|n| self.scenes.iter().find(|s| &s.name == n)).unwrap_or(&vp.view)
    }

    /// A copy of the model with the scene's tag visibility and section state applied.
    pub fn with_scene_state(&self, s: &Scene) -> Model {
        let mut m = self.clone();
        if let Some(hidden) = &s.hidden_tags {
            for (id, t) in m.tags.iter_mut() {
                t.visible = !hidden.contains(&id);
            }
        }
        if let Some(sec) = &s.sections {
            m.apply_scene_sections(sec);
        }
        m
    }
}
