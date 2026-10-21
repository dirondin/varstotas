//! Interactive tools (state machines driven by viewport input). Owner: `tools` agent.
//!
//! Scaffolding by the shell agent: the [`Tool`] trait, [`ToolContext`], [`ToolId`] metadata,
//! working Select / Orbit / Pan / Zoom tools, and stubs for everything else.
//!
//! Coordinates: `PointerInfo::pos` is in viewport-local logical points (origin top-left of
//! the 3D view), which is what `Camera::screen_ray` / `world_to_screen` expect.

// Public API for the Tools agent: not every item is used by the scaffolding yet.
#![allow(dead_code)]

pub mod common;
mod arcs;
pub mod bezier;
mod camera_tools;
pub(crate) mod construct;
mod dims;
mod curve_edit;
mod draw;
mod edit;
mod nav_tools;
pub(crate) mod primitives;
pub mod plugin_pick;
pub mod plugin_select;
pub(crate) mod light;
pub(crate) mod section;
mod slice;
pub(crate) mod workplane;
mod chamfer;
pub(crate) mod solid;
mod text;
pub(crate) mod text3d;
mod texture;
pub(crate) mod select;
pub(crate) mod transform;

use varstotas_i18n::tr;
use crate::document::Document;
use crate::overlay::Marker2d;
use crate::ui::icons::Icon;
use egui::{CursorIcon, Key, Modifiers, PointerButton, Pos2, Vec2};
use varstotas_kernel::glam::{DVec2, DVec3};
use varstotas_kernel::{Model, ModelHit, Ray};
use varstotas_render::{Camera, Overlay3d};

pub use nav_tools::{OrbitTool, PanTool, ZoomTool};
pub use select::SelectTool;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum ToolId {
    Select,
    Eraser,
    Paint,
    TexturePosition,
    Line,
    Arc,
    ArcCenter,
    Arc3,
    Pie,
    Bezier,
    SmoothCurve,
    Freehand,
    EditCurve,
    Rectangle,
    RotatedRectangle,
    Circle,
    Polygon,
    Box,
    Cylinder,
    Cone,
    Pyramid,
    Sphere,
    Torus,
    PushPull,
    Move,
    Rotate,
    Scale,
    Offset,
    Chamfer,
    Fillet,
    SolidTools,
    FollowMe,
    Tape,
    Protractor,
    Axes,
    Dimension,
    Text,
    Section,
    GuidePlane,
    Slice,
    Orbit,
    Pan,
    Zoom,
    PositionCamera,
    LookAround,
    Walk,
}

impl ToolId {
    pub const ALL: [ToolId; 46] = [
        ToolId::Select,
        ToolId::Eraser,
        ToolId::Paint,
        ToolId::TexturePosition,
        ToolId::Line,
        ToolId::Arc,
        ToolId::ArcCenter,
        ToolId::Arc3,
        ToolId::Pie,
        ToolId::Bezier,
        ToolId::SmoothCurve,
        ToolId::Freehand,
        ToolId::EditCurve,
        ToolId::Rectangle,
        ToolId::RotatedRectangle,
        ToolId::Circle,
        ToolId::Polygon,
        ToolId::Box,
        ToolId::Cylinder,
        ToolId::Cone,
        ToolId::Pyramid,
        ToolId::Sphere,
        ToolId::Torus,
        ToolId::PushPull,
        ToolId::Move,
        ToolId::Rotate,
        ToolId::Scale,
        ToolId::Offset,
        ToolId::Chamfer,
        ToolId::Fillet,
        ToolId::SolidTools,
        ToolId::FollowMe,
        ToolId::Tape,
        ToolId::Protractor,
        ToolId::Axes,
        ToolId::Dimension,
        ToolId::Text,
        ToolId::Section,
        ToolId::GuidePlane,
        ToolId::Slice,
        ToolId::Orbit,
        ToolId::Pan,
        ToolId::Zoom,
        ToolId::PositionCamera,
        ToolId::LookAround,
        ToolId::Walk,
    ];

    pub fn name(self) -> String {
        varstotas_i18n::tr(match self {
            ToolId::Select => "tool-select",
            ToolId::Eraser => "tool-eraser",
            ToolId::Paint => "tool-paint",
            ToolId::TexturePosition => "tool-texture-position",
            ToolId::Line => "tool-line",
            ToolId::Arc => "tool-arc",
            ToolId::ArcCenter => "tool-arc-center",
            ToolId::Arc3 => "tool-arc3",
            ToolId::Pie => "tool-pie",
            ToolId::Bezier => "tool-bezier",
            ToolId::SmoothCurve => "tool-smooth-curve",
            ToolId::Freehand => "tool-freehand",
            ToolId::EditCurve => "tool-edit-curve",
            ToolId::Rectangle => "tool-rectangle",
            ToolId::RotatedRectangle => "tool-rotated-rectangle",
            ToolId::Circle => "tool-circle",
            ToolId::Polygon => "tool-polygon",
            ToolId::Box => "tool-box",
            ToolId::Cylinder => "tool-cylinder",
            ToolId::Cone => "tool-cone",
            ToolId::Pyramid => "tool-pyramid",
            ToolId::Sphere => "tool-sphere",
            ToolId::Torus => "tool-torus",
            ToolId::PushPull => "tool-push-pull",
            ToolId::Move => "tool-move",
            ToolId::Rotate => "tool-rotate",
            ToolId::Scale => "tool-scale",
            ToolId::Offset => "tool-offset",
            ToolId::Chamfer => "tool-chamfer",
            ToolId::Fillet => "tool-fillet",
            ToolId::SolidTools => "tool-solid-tools",
            ToolId::FollowMe => "tool-follow-me",
            ToolId::Tape => "tool-tape",
            ToolId::Protractor => "tool-protractor",
            ToolId::Axes => "tool-axes",
            ToolId::Dimension => "tool-dimension",
            ToolId::Text => "tool-text",
            ToolId::Section => "tool-section",
            ToolId::GuidePlane => "tool-guide-plane",
            ToolId::Slice => "tool-slice",
            ToolId::Orbit => "tool-orbit",
            ToolId::Pan => "tool-pan",
            ToolId::Zoom => "tool-zoom",
            ToolId::PositionCamera => "tool-position-camera",
            ToolId::LookAround => "tool-look-around",
            ToolId::Walk => "tool-walk",
        })
    }

    /// Single-key shortcut (no modifiers), per docs/05-UX-SPEC.md.
    pub fn shortcut(self) -> Option<Key> {
        Some(match self {
            ToolId::Select => Key::Space,
            ToolId::Eraser => Key::E,
            ToolId::Paint => Key::B,
            ToolId::Line => Key::L,
            ToolId::Arc => Key::A,
            ToolId::Rectangle => Key::R,
            ToolId::Circle => Key::C,
            ToolId::PushPull => Key::P,
            ToolId::Move => Key::M,
            ToolId::Rotate => Key::Q,
            ToolId::Scale => Key::S,
            ToolId::Offset => Key::F,
            ToolId::Tape => Key::T,
            ToolId::Orbit => Key::O,
            ToolId::Pan => Key::H,
            ToolId::Zoom => Key::Z,
            ToolId::Polygon
            | ToolId::Box
            | ToolId::Cylinder
            | ToolId::Cone
            | ToolId::Pyramid
            | ToolId::Sphere
            | ToolId::Torus
            | ToolId::FollowMe
            | ToolId::Chamfer
            | ToolId::Fillet
            | ToolId::SolidTools
            | ToolId::Protractor
            | ToolId::ArcCenter
            | ToolId::Arc3
            | ToolId::Pie
            | ToolId::Bezier
            | ToolId::SmoothCurve
            | ToolId::Freehand
            | ToolId::EditCurve
            | ToolId::RotatedRectangle
            | ToolId::TexturePosition
            | ToolId::Section
            | ToolId::GuidePlane
            | ToolId::Slice
            | ToolId::Text
            | ToolId::PositionCamera
            | ToolId::LookAround
            | ToolId::Walk
            | ToolId::Axes
            | ToolId::Dimension => return None,
        })
    }

    /// Human-readable shortcut label for tooltips/menus.
    pub fn shortcut_label(self) -> Option<String> {
        crate::keymap::label(crate::keymap::KeyCmd::Tool(self))
    }

    pub fn icon(self) -> Icon {
        match self {
            ToolId::Select => Icon::Select,
            ToolId::Eraser => Icon::Eraser,
            ToolId::Paint => Icon::Paint,
            ToolId::TexturePosition => Icon::Paint,
            ToolId::Line => Icon::Line,
            ToolId::Arc => Icon::Arc,
            ToolId::ArcCenter => Icon::ArcCenter,
            ToolId::Arc3 => Icon::Arc3,
            ToolId::Pie => Icon::Pie,
            ToolId::Bezier => Icon::Bezier,
            ToolId::SmoothCurve => Icon::SmoothCurve,
            ToolId::Freehand => Icon::Freehand,
            ToolId::EditCurve => Icon::EditCurve,
            ToolId::Rectangle => Icon::Rectangle,
            ToolId::RotatedRectangle => Icon::RotatedRectangle,
            ToolId::Circle => Icon::Circle,
            ToolId::Polygon => Icon::Polygon,
            ToolId::Box => Icon::Box3d,
            ToolId::Cylinder => Icon::Cylinder,
            ToolId::Cone => Icon::Cone,
            ToolId::Pyramid => Icon::Pyramid,
            ToolId::Sphere => Icon::Sphere,
            ToolId::Torus => Icon::Torus,
            ToolId::PushPull => Icon::PushPull,
            ToolId::Move => Icon::Move,
            ToolId::Rotate => Icon::Rotate,
            ToolId::Scale => Icon::Scale,
            ToolId::Offset => Icon::Offset,
            ToolId::Chamfer => Icon::Chamfer,
            ToolId::Fillet => Icon::Fillet,
            ToolId::SolidTools => Icon::Solid,
            ToolId::FollowMe => Icon::FollowMe,
            ToolId::Tape => Icon::Tape,
            ToolId::Protractor => Icon::Protractor,
            ToolId::Axes => Icon::Axes,
            ToolId::Dimension => Icon::Dimension,
            ToolId::Text => Icon::Text,
            ToolId::Section => Icon::Section,
            ToolId::GuidePlane => Icon::GuidePlane,
            ToolId::Slice => Icon::Slice,
            ToolId::Orbit => Icon::Orbit,
            ToolId::Pan => Icon::Pan,
            ToolId::Zoom => Icon::Zoom,
            ToolId::PositionCamera => Icon::PositionCamera,
            ToolId::LookAround => Icon::LookAround,
            ToolId::Walk => Icon::Walk,
        }
    }

    /// One-line description (tooltips).
    pub fn description(self) -> String {
        varstotas_i18n::tr(match self {
            ToolId::Select => "tool-select-desc",
            ToolId::Eraser => "tool-eraser-desc",
            ToolId::Paint => "tool-paint-desc",
            ToolId::TexturePosition => "tool-texture-position-desc",
            ToolId::Line => "tool-line-desc",
            ToolId::Arc => "tool-arc-desc",
            ToolId::ArcCenter => "tool-arc-center-desc",
            ToolId::Arc3 => "tool-arc3-desc",
            ToolId::Pie => "tool-pie-desc",
            ToolId::Bezier => "tool-bezier-desc",
            ToolId::SmoothCurve => "tool-smooth-curve-desc",
            ToolId::Freehand => "tool-freehand-desc",
            ToolId::EditCurve => "tool-edit-curve-desc",
            ToolId::Rectangle => "tool-rectangle-desc",
            ToolId::RotatedRectangle => "tool-rotated-rectangle-desc",
            ToolId::Circle => "tool-circle-desc",
            ToolId::Polygon => "tool-polygon-desc",
            ToolId::Box => "tool-box-desc",
            ToolId::Cylinder => "tool-cylinder-desc",
            ToolId::Cone => "tool-cone-desc",
            ToolId::Pyramid => "tool-pyramid-desc",
            ToolId::Sphere => "tool-sphere-desc",
            ToolId::Torus => "tool-torus-desc",
            ToolId::PushPull => "tool-push-pull-desc",
            ToolId::Move => "tool-move-desc",
            ToolId::Rotate => "tool-rotate-desc",
            ToolId::Scale => "tool-scale-desc",
            ToolId::Offset => "tool-offset-desc",
            ToolId::Chamfer => "tool-chamfer-desc",
            ToolId::Fillet => "tool-fillet-desc",
            ToolId::SolidTools => "tool-solid-tools-desc",
            ToolId::FollowMe => "tool-follow-me-desc",
            ToolId::Tape => "tool-tape-desc",
            ToolId::Protractor => "tool-protractor-desc",
            ToolId::Axes => "tool-axes-desc",
            ToolId::Dimension => "tool-dimension-desc",
            ToolId::Text => "tool-text-desc",
            ToolId::Section => "tool-section-desc",
            ToolId::GuidePlane => "tool-guide-plane-desc",
            ToolId::Slice => "tool-slice-desc",
            ToolId::Orbit => "tool-orbit-desc",
            ToolId::Pan => "tool-pan-desc",
            ToolId::Zoom => "tool-zoom-desc",
            ToolId::PositionCamera => "tool-position-camera-desc",
            ToolId::LookAround => "tool-look-around-desc",
            ToolId::Walk => "tool-walk-desc",
        })
    }

    /// Longer help shown in the Instructor panel.
    pub fn instructor(self) -> String {
        self.instructor_text()
    }

    fn instructor_text(self) -> String {
        tr!(match self {
            ToolId::Select => "tool-select-instructor",
            ToolId::Eraser => "tool-eraser-instructor",
            ToolId::Paint => "tool-paint-instructor",
            ToolId::TexturePosition => "tool-texture-position-instructor",
            ToolId::Line => "tool-line-instructor",
            ToolId::Arc => "tool-arc-instructor",
            ToolId::ArcCenter => "tool-arc-center-instructor",
            ToolId::Arc3 => "tool-arc3-instructor",
            ToolId::Bezier => "tool-bezier-instructor",
            ToolId::EditCurve => "tool-edit-curve-instructor",
            ToolId::SmoothCurve => "tool-smooth-curve-instructor",
            ToolId::Freehand => "tool-freehand-instructor",
            ToolId::Pie => "tool-pie-instructor",
            ToolId::Rectangle => "tool-rectangle-instructor",
            ToolId::RotatedRectangle => "tool-rotated-rectangle-instructor",
            ToolId::Circle => "tool-circle-instructor",
            ToolId::Polygon => "tool-polygon-instructor",
            ToolId::Box | ToolId::Pyramid => "tool-box-instructor",
            ToolId::Cylinder | ToolId::Cone => "tool-cylinder-instructor",
            ToolId::Sphere => "tool-sphere-instructor",
            ToolId::Torus => "tool-torus-instructor",
            ToolId::PushPull => "tool-push-pull-instructor",
            ToolId::Move => "tool-move-instructor",
            ToolId::Rotate => "tool-rotate-instructor",
            ToolId::Scale => "tool-scale-instructor",
            ToolId::Offset => "tool-offset-instructor",
            ToolId::Chamfer => "tool-chamfer-instructor",
            ToolId::Fillet => "tool-fillet-instructor",
            ToolId::SolidTools => "tool-solid-tools-instructor",
            ToolId::FollowMe => "tool-follow-me-instructor",
            ToolId::Tape => "tool-tape-instructor",
            ToolId::Protractor => "tool-protractor-instructor",
            ToolId::Axes => "tool-axes-instructor",
            ToolId::Dimension => "tool-dimension-instructor",
            ToolId::Text => "tool-text-instructor",
            ToolId::Section => "tool-section-instructor",
            ToolId::GuidePlane => "tool-guide-plane-instructor",
            ToolId::Slice => "tool-slice-instructor",
            ToolId::Orbit => "tool-orbit-instructor",
            ToolId::Pan => "tool-pan-instructor",
            ToolId::Zoom => "tool-zoom-instructor",
            ToolId::PositionCamera => "tool-position-camera-instructor",
            ToolId::LookAround => "tool-look-around-instructor",
            ToolId::Walk => "tool-walk-instructor",
        })
    }

    /// Whether the tool is implemented (others are stubs).
    pub fn implemented(self) -> bool {
        true
    }
}

/// Draw → Shapes: the solid primitives (`primitives.rs`).
pub const SHAPES: [ToolId; 6] = [ToolId::Box, ToolId::Cylinder, ToolId::Cone, ToolId::Pyramid, ToolId::Sphere, ToolId::Torus];

/// Create a fresh instance of a tool.
pub fn make_tool(id: ToolId) -> Box<dyn Tool> {
    match id {
        ToolId::Select => Box::new(SelectTool::default()),
        ToolId::Orbit => Box::new(OrbitTool::default()),
        ToolId::Pan => Box::new(PanTool::default()),
        ToolId::Zoom => Box::new(ZoomTool::default()),
        ToolId::PositionCamera => Box::new(camera_tools::PositionCameraTool::default()),
        ToolId::LookAround => Box::new(camera_tools::LookAroundTool::default()),
        ToolId::Walk => Box::new(camera_tools::WalkTool::default()),
        ToolId::Line => Box::new(draw::LineTool::default()),
        ToolId::Rectangle => Box::new(draw::RectangleTool::default()),
        ToolId::Circle => Box::new(draw::CircleTool::new(false)),
        ToolId::Polygon => Box::new(draw::CircleTool::new(true)),
        ToolId::Box | ToolId::Cylinder | ToolId::Cone | ToolId::Pyramid | ToolId::Sphere | ToolId::Torus => {
            Box::new(primitives::PrimitiveTool::new(id))
        }
        ToolId::Arc => Box::new(draw::ArcTool::default()),
        ToolId::ArcCenter => Box::new(arcs::ArcCenterTool::new(false)),
        ToolId::Pie => Box::new(arcs::ArcCenterTool::new(true)),
        ToolId::Bezier => Box::new(bezier::BezierTool::new(bezier::Method::Handles)),
        ToolId::SmoothCurve => Box::new(bezier::BezierTool::new(bezier::Method::Smooth)),
        ToolId::Freehand => Box::new(bezier::BezierTool::new(bezier::Method::Freehand)),
        ToolId::EditCurve => Box::new(curve_edit::EditCurveTool::default()),
        ToolId::Arc3 => Box::new(arcs::Arc3Tool::default()),
        ToolId::RotatedRectangle => Box::new(arcs::RotatedRectTool::default()),
        ToolId::Axes => Box::new(construct::AxesTool::default()),
        ToolId::Dimension => Box::new(dims::DimensionTool::default()),
        ToolId::Section => Box::new(section::SectionTool::default()),
        ToolId::GuidePlane => Box::new(workplane::GuidePlaneTool::default()),
        ToolId::Slice => Box::new(slice::SliceTool::default()),
        ToolId::Text => Box::new(text::TextTool::default()),
        ToolId::Eraser => Box::new(edit::EraserTool::default()),
        ToolId::Paint => Box::new(edit::PaintTool::default()),
        ToolId::TexturePosition => Box::new(texture::TexturePositionTool::default()),
        ToolId::PushPull => Box::new(edit::PushPullTool::default()),
        ToolId::Offset => Box::new(edit::OffsetTool::default()),
        ToolId::Chamfer => Box::new(chamfer::ChamferTool::new(false)),
        ToolId::Fillet => Box::new(chamfer::ChamferTool::new(true)),
        ToolId::SolidTools => Box::new(solid::SolidTool::new(varstotas_kernel::SolidOp::OuterShell)),
        ToolId::FollowMe => Box::new(edit::FollowMeTool::default()),
        ToolId::Move => Box::new(transform::MoveTool::default()),
        ToolId::Rotate => Box::new(transform::RotateTool::default()),
        ToolId::Scale => Box::new(transform::ScaleTool::default()),
        ToolId::Tape => Box::new(construct::TapeTool::default()),
        ToolId::Protractor => Box::new(construct::ProtractorTool::default()),
    }
}

/// Pointer state passed to tools.
#[derive(Clone, Debug)]
pub struct PointerInfo {
    /// Viewport-local position in logical points.
    pub pos: Pos2,
    /// Movement since the last event.
    pub delta: Vec2,
    /// World ray through `pos`.
    pub ray: Ray,
    pub modifiers: Modifiers,
    /// Button for down/up/click events.
    pub button: Option<PointerButton>,
    /// Whether the primary button is currently held.
    pub primary_down: bool,
    /// Where the current press started (viewport-local).
    pub press_origin: Option<Pos2>,
    /// 1 = single, 2 = double, 3 = triple click.
    pub click_count: u32,
}

/// Requests a tool can make to the application.
#[derive(Clone, Debug, PartialEq)]
pub enum ToolRequest {
    SwitchTool(ToolId),
    ZoomExtents,
    /// Tape Measure: ask "resize the model / group?" (uniform factor) in a dialog.
    ConfirmResize(f64),
    /// Select: edit the text of a dimension of the active context.
    EditDimensionText(varstotas_kernel::DimId),
    /// A plugin's `@pick` points were clicked: run it.
    PluginPoints(Vec<varstotas_kernel::glam::DVec3>),
    /// `@select` finished (Enter): the selection goes to the script as `objects`.
    PluginObjects,
}

/// Everything a tool can touch while handling an event.
pub struct ToolContext<'a> {
    pub doc: &'a mut Document,
    pub camera: &'a mut Camera,
    /// Viewport size in logical points.
    pub viewport: Vec2,
    pub pointer: PointerInfo,
    /// Seconds (egui clock), for hover delays.
    pub time: f64,
    pub requests: &'a mut Vec<ToolRequest>,
}

impl ToolContext<'_> {
    /// Run `f` as one undoable operation: snapshots the model for undo, bumps
    /// `model_version`, marks dirty. Returns None (and restores) if the op panicked.
    pub fn commit<R>(&mut self, label: &str, f: impl FnOnce(&mut Model) -> R) -> Option<R> {
        self.doc.commit(label, f)
    }

    pub fn viewport_d(&self) -> DVec2 {
        DVec2::new(self.viewport.x as f64, self.viewport.y as f64)
    }

    pub fn ray_at(&self, pos: Pos2) -> Ray {
        crate::nav::ray_at(self.camera, pos, self.viewport)
    }

    /// Viewport-local screen position of a world point.
    pub fn to_screen(&self, p: DVec3) -> Option<Pos2> {
        self.camera
            .world_to_screen(p, self.viewport_d())
            .map(|s| Pos2::new(s.x as f32, s.y as f32))
    }

    /// Closest face hit under the pointer (whole model).
    pub fn pick(&self) -> Option<ModelHit> {
        crate::document::cached_raycast(&self.doc.model, self.doc.model_version, &self.pointer.ray)
    }

    /// World point under the pointer (model, else ground plane, else target depth).
    pub fn point_under_pointer(&self) -> DVec3 {
        crate::nav::point_under(&self.doc.model, self.camera, self.pointer.pos, self.viewport)
    }
}

/// Read-only view state for drawing overlays.
pub struct OverlayView<'a> {
    pub doc: &'a Document,
    pub camera: &'a Camera,
    pub viewport: Vec2,
    pub pointer: Option<Pos2>,
}

pub trait Tool {
    fn id(&self) -> ToolId;
    fn activate(&mut self, _ctx: &mut ToolContext) {}
    fn deactivate(&mut self, _ctx: &mut ToolContext) {}
    fn on_pointer_move(&mut self, _ctx: &mut ToolContext) {}
    fn on_pointer_down(&mut self, _ctx: &mut ToolContext) {}
    fn on_pointer_up(&mut self, _ctx: &mut ToolContext) {}
    /// Called for double (click_count = 2) and triple (3) clicks, after the up event.
    fn on_double_click(&mut self, _ctx: &mut ToolContext) {}
    /// Called every frame while [`Tool::animating`]; `dt` in seconds.
    fn on_frame(&mut self, _ctx: &mut ToolContext, _dt: f64) {}
    /// The tool moves something on its own (e.g. Walk while the button is held): the app
    /// calls [`Tool::on_frame`] and keeps repainting.
    fn animating(&self) -> bool {
        false
    }
    /// Key press/release not consumed by the VCB or global hotkeys (Esc, arrows, modifiers…).
    /// Return true if handled.
    fn on_key(&mut self, _ctx: &mut ToolContext, _key: Key, _pressed: bool) -> bool {
        false
    }
    /// Measurements box input on Enter. Return false if the value was rejected.
    fn on_vcb(&mut self, _ctx: &mut ToolContext, _input: &str) -> bool {
        false
    }
    fn vcb_label(&self) -> String {
        tr!("tool-vcb-measurements")
    }
    fn vcb_value(&self) -> String {
        String::new()
    }
    fn status_text(&self) -> String;
    fn overlay(&self, _view: &OverlayView, _out3d: &mut Overlay3d, _out2d: &mut Vec<Marker2d>) {}
    fn cursor(&self) -> CursorIcon {
        CursorIcon::Default
    }
    /// Tool cursor glyph drawn at the pointer (modeling tools); the tool then returns
    /// `CursorIcon::None` from [`Tool::cursor`] to hide the OS cursor.
    fn cursor_glyph(&self) -> Option<Marker2d> {
        match self.id() {
            ToolId::Select | ToolId::Orbit | ToolId::Pan | ToolId::Zoom => None,
            id if !id.implemented() => None,
            id => Some(Marker2d::ToolCursor { icon: id.icon(), plus: self.cursor_plus() }),
        }
    }
    /// Show a "+" badge on the tool cursor (copy / keep original / guides on).
    fn cursor_plus(&self) -> bool {
        false
    }
    /// True while in the middle of an operation (Esc cancels it rather than leaving context).
    fn is_busy(&self) -> bool {
        false
    }
}

/// Placeholder for tools the Tools agent has not implemented yet.
pub struct StubTool(pub ToolId);

impl Tool for StubTool {
    fn id(&self) -> ToolId {
        self.0
    }
    fn status_text(&self) -> String {
        tr!("tool-not-implemented", name = self.0.name())
    }
    fn cursor(&self) -> CursorIcon {
        CursorIcon::NotAllowed
    }
}
