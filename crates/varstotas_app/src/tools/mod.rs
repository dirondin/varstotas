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
mod camera_tools;
pub(crate) mod construct;
mod dims;
mod draw;
mod edit;
mod nav_tools;
pub(crate) mod section;
mod chamfer;
mod text;
pub(crate) mod text3d;
mod texture;
pub(crate) mod select;
pub(crate) mod transform;

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
    Rectangle,
    RotatedRectangle,
    Circle,
    Polygon,
    PushPull,
    Move,
    Rotate,
    Scale,
    Offset,
    Chamfer,
    Fillet,
    FollowMe,
    Tape,
    Protractor,
    Axes,
    Dimension,
    Text,
    Section,
    Orbit,
    Pan,
    Zoom,
    PositionCamera,
    LookAround,
    Walk,
}

impl ToolId {
    pub const ALL: [ToolId; 33] = [
        ToolId::Select,
        ToolId::Eraser,
        ToolId::Paint,
        ToolId::TexturePosition,
        ToolId::Line,
        ToolId::Arc,
        ToolId::ArcCenter,
        ToolId::Arc3,
        ToolId::Pie,
        ToolId::Rectangle,
        ToolId::RotatedRectangle,
        ToolId::Circle,
        ToolId::Polygon,
        ToolId::PushPull,
        ToolId::Move,
        ToolId::Rotate,
        ToolId::Scale,
        ToolId::Offset,
        ToolId::Chamfer,
        ToolId::Fillet,
        ToolId::FollowMe,
        ToolId::Tape,
        ToolId::Protractor,
        ToolId::Axes,
        ToolId::Dimension,
        ToolId::Text,
        ToolId::Section,
        ToolId::Orbit,
        ToolId::Pan,
        ToolId::Zoom,
        ToolId::PositionCamera,
        ToolId::LookAround,
        ToolId::Walk,
    ];

    pub fn name(self) -> &'static str {
        match self {
            ToolId::Select => "Select",
            ToolId::Eraser => "Eraser",
            ToolId::Paint => "Paint Bucket",
            ToolId::TexturePosition => "Texture Position",
            ToolId::Line => "Line",
            ToolId::Arc => "2 Point Arc",
            ToolId::ArcCenter => "Arc",
            ToolId::Arc3 => "3 Point Arc",
            ToolId::Pie => "Pie",
            ToolId::Rectangle => "Rectangle",
            ToolId::RotatedRectangle => "Rotated Rectangle",
            ToolId::Circle => "Circle",
            ToolId::Polygon => "Polygon",
            ToolId::PushPull => "Push/Pull",
            ToolId::Move => "Move",
            ToolId::Rotate => "Rotate",
            ToolId::Scale => "Scale",
            ToolId::Offset => "Offset",
            ToolId::Chamfer => "Chamfer",
            ToolId::Fillet => "Fillet",
            ToolId::FollowMe => "Follow Me",
            ToolId::Tape => "Tape Measure",
            ToolId::Protractor => "Protractor",
            ToolId::Axes => "Axes",
            ToolId::Dimension => "Dimensions",
            ToolId::Text => "Text",
            ToolId::Section => "Section Plane",
            ToolId::Orbit => "Orbit",
            ToolId::Pan => "Pan",
            ToolId::Zoom => "Zoom",
            ToolId::PositionCamera => "Position Camera",
            ToolId::LookAround => "Look Around",
            ToolId::Walk => "Walk",
        }
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
            | ToolId::FollowMe
            | ToolId::Chamfer
            | ToolId::Fillet
            | ToolId::Protractor
            | ToolId::ArcCenter
            | ToolId::Arc3
            | ToolId::Pie
            | ToolId::RotatedRectangle
            | ToolId::TexturePosition
            | ToolId::Section
            | ToolId::Text
            | ToolId::PositionCamera
            | ToolId::LookAround
            | ToolId::Walk
            | ToolId::Axes
            | ToolId::Dimension => return None,
        })
    }

    /// Human-readable shortcut label for tooltips/menus.
    pub fn shortcut_label(self) -> Option<&'static str> {
        self.shortcut().map(|k| if k == Key::Space { "Space" } else { k.name() })
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
            ToolId::Rectangle => Icon::Rectangle,
            ToolId::RotatedRectangle => Icon::RotatedRectangle,
            ToolId::Circle => Icon::Circle,
            ToolId::Polygon => Icon::Polygon,
            ToolId::PushPull => Icon::PushPull,
            ToolId::Move => Icon::Move,
            ToolId::Rotate => Icon::Rotate,
            ToolId::Scale => Icon::Scale,
            ToolId::Offset => Icon::Offset,
            ToolId::Chamfer => Icon::Chamfer,
            ToolId::Fillet => Icon::Fillet,
            ToolId::FollowMe => Icon::FollowMe,
            ToolId::Tape => Icon::Tape,
            ToolId::Protractor => Icon::Protractor,
            ToolId::Axes => Icon::Axes,
            ToolId::Dimension => Icon::Dimension,
            ToolId::Text => Icon::Text,
            ToolId::Section => Icon::Section,
            ToolId::Orbit => Icon::Orbit,
            ToolId::Pan => Icon::Pan,
            ToolId::Zoom => Icon::Zoom,
            ToolId::PositionCamera => Icon::PositionCamera,
            ToolId::LookAround => Icon::LookAround,
            ToolId::Walk => Icon::Walk,
        }
    }

    /// One-line description (tooltips).
    pub fn description(self) -> &'static str {
        match self {
            ToolId::Select => "Select entities to modify with other tools.",
            ToolId::Eraser => "Erase edges, guides and other entities.",
            ToolId::Paint => "Apply a material to faces and groups.",
            ToolId::TexturePosition => "Move, rotate and scale a face's texture with pins.",
            ToolId::Line => "Draw edges. Closed planar loops become faces.",
            ToolId::Arc => "Draw an arc from two end points and a bulge.",
            ToolId::ArcCenter => "Draw an arc from its center and two points.",
            ToolId::Arc3 => "Draw an arc through three points.",
            ToolId::Pie => "Draw a closed pie-shaped face from a center and two points.",
            ToolId::Rectangle => "Draw a rectangular face from two corners.",
            ToolId::RotatedRectangle => "Draw a rectangle at any angle from an edge and a width.",
            ToolId::Circle => "Draw a circle from center and radius.",
            ToolId::Polygon => "Draw a regular polygon.",
            ToolId::PushPull => "Push or pull a face to add or remove volume.",
            ToolId::Move => "Move, stretch or copy entities.",
            ToolId::Rotate => "Rotate, stretch or copy entities about an axis.",
            ToolId::Scale => "Resize and stretch entities.",
            ToolId::Offset => "Offset a face's edges inward or outward.",
            ToolId::Chamfer => "Cut an edge between two faces with a flat bevel.",
            ToolId::Fillet => "Round an edge between two faces.",
            ToolId::FollowMe => "Sweep a face along a path.",
            ToolId::Tape => "Measure distances and create guides.",
            ToolId::Protractor => "Measure angles and create angled guides.",
            ToolId::Axes => "Move or reorient the drawing axes.",
            ToolId::Dimension => "Place linear dimensions between points or on edges.",
            ToolId::Text => "Add a text with a leader pointing at the model.",
            ToolId::Section => "Cut the model with a section plane to see inside.",
            ToolId::Orbit => "Rotate the camera around the model.",
            ToolId::Pan => "Move the camera sideways.",
            ToolId::Zoom => "Zoom the camera in or out.",
            ToolId::PositionCamera => "Stand at a point at eye height to look at the model.",
            ToolId::LookAround => "Turn your head: pivot the view around the eye.",
            ToolId::Walk => "Walk through the model at eye height.",
        }
    }

    /// Longer help shown in the Instructor panel.
    pub fn instructor(self) -> &'static str {
        match self {
            ToolId::Select if cfg!(target_os = "macos") => {
                "Click an entity to select it.\n\
                 • Double-click a face: select it with its edges.\n\
                 • Triple-click: select everything connected.\n\
                 • Drag left→right: window selection (fully inside).\n\
                 • Drag right→left: crossing selection (touching).\n\
                 • Option (or Cmd) = add, Shift = toggle, Shift+Option = remove.\n\
                 • Double-click a group to edit it; Esc to close it.\n\
                 • Delete erases the selection."
            }
            ToolId::Select => {
                "Click an entity to select it.\n\
                 • Double-click a face: select it with its edges.\n\
                 • Triple-click: select everything connected.\n\
                 • Drag left→right: window selection (fully inside).\n\
                 • Drag right→left: crossing selection (touching).\n\
                 • Ctrl = add, Shift = toggle, Ctrl+Shift = remove.\n\
                 • Double-click a group to edit it; Esc to close it.\n\
                 • Delete erases the selection."
            }
            ToolId::Eraser => {
                "Click or drag over edges to erase them. Erasing an edge removes the faces it bounds."
            }
            ToolId::Paint if cfg!(target_os = "macos") => {
                "Pick a material in the Materials panel, then click faces.\n\
                 • Cmd: sample a material.\n\
                 • Option: fill connected faces with the same material.\n\
                 • Shift: replace the material on all matching faces.\n\
                 • Option+Shift: replace it on matching connected faces."
            }
            ToolId::Paint => {
                "Pick a material in the Materials panel, then click faces.\n\
                 • Alt: sample a material.\n\
                 • Ctrl: fill connected faces with the same material.\n\
                 • Shift: replace the material on all matching faces.\n\
                 • Ctrl+Shift: replace it on matching connected faces."
            }
            ToolId::TexturePosition => {
                "Position the texture of a textured face (select it, or click it).\n\
                 • Drag the face or the red pin: move the texture.\n\
                 • Green pin: rotate and scale about the red pin.\n\
                 • Blue pin: stretch the texture height.\n\
                 • Esc cancels a drag; Enter or a click outside finishes."
            }
            ToolId::Line => {
                "Click to start, click to end. Each end starts a new segment.\n\
                 Type a length and press Enter for precision.\n\
                 Shift locks the inference direction; arrow keys lock axes."
            }
            ToolId::Arc => {
                "Click the start point, click the end point, then move to set the bulge and click.\n\
                 • Type a length for the chord, then a bulge (or `500r` for a radius); `12s` sets segments.\n\
                 • Starting at the end of an edge, a cyan arc is tangent to it.\n\
                 • Double-click to draw an arc tangent to the previous one."
            }
            ToolId::ArcCenter => {
                "Click the center, click the start point (radius), then move around and click the end.\n\
                 Type a radius after the first click, then an angle. `12s` sets segments."
            }
            ToolId::Arc3 => {
                "Click the start point, click a point the arc passes through, then click the end.\n\
                 Type a length after the first click, an angle after the second."
            }
            ToolId::Pie => {
                "Click the center, click the start point (radius), then click the end to make a closed pie face.\n\
                 Type a radius, then an angle."
            }
            ToolId::Rectangle => "Click the first corner, then the opposite corner. Type width,height to be exact.",
            ToolId::RotatedRectangle => {
                "Click the first corner, click the second (the first edge), then move to set the width and click.\n\
                 Type a length (or `l,w`) for the edge, then the width (or `w,angle`)."
            }
            ToolId::Circle => "Click the center, then move and click to set the radius. Type `24s` before clicking to set segments.",
            ToolId::Polygon => "Click the center, then the radius. Type `6s` to set the number of sides.",
            ToolId::PushPull => {
                "Hover a face, click, move to set the distance, click again.\n\
                 Ctrl/Option keeps the original face. Double-click repeats the last distance."
            }
            ToolId::Move => "Click a point to pick up the selection, click the destination. Ctrl/Option makes a copy; type `5x` for an array.",
            ToolId::Rotate => "Click the center, click the zero direction, click the angle. Type an angle in degrees.",
            ToolId::Scale => "Select entities, then drag a grip. Type a scale factor.",
            ToolId::Offset => "Click a face, move inward or outward, click. Type a distance.",
            ToolId::Chamfer => "Click an edge between two faces (or one of the selected edges: all of them). \
                 Type the distance; typing right after a chamfer redoes it with the new size.",
            ToolId::Fillet => "Click an edge between two faces (or one of the selected edges: all of them). \
                 Type the radius, `12s` for the number of segments, or both: `20,12s`; typing right \
                 after a fillet redoes it.",
            ToolId::FollowMe => "Select a path, then click the profile face to sweep it.",
            ToolId::Tape => "Click two points to measure. Starting on an edge creates a parallel guide.",
            ToolId::Protractor => "Click the center, click the base direction, click the angle to create a guide.",
            ToolId::Axes => {
                "Click the new origin, click a point along the red axis, then a point toward the green axis.\n\
                 Type `x,y,z` for the origin. Right-click the axes for Place / Move / Reset."
            }
            ToolId::Dimension => {
                "Click a start point and an end point (or click an edge), then move to offset the dimension and click.\n\
                 Double-click a dimension's text with Select to edit it; empty text restores the measured length."
            }
            ToolId::Text => {
                "Click a point, edge or face, move the text away and click again.\n\
                 • Default text: a point's coordinates, an edge's length, a face's area.\n\
                 • Click empty space for a text without a leader.\n\
                 • Edit the text in Entity Info; move it with Move or by moving the geometry."
            }
            ToolId::Section => {
                "Click a face to place a section plane on it; it becomes the active cut.\n\
                 • The arrows point to the part that stays visible.\n\
                 • Select a plane by its border; move it with Move, delete it with Delete.\n\
                 • Right-click a plane: Active Cut, Reverse, Align View.\n\
                 • View → Section Planes / Section Cuts / Section Fill."
            }
            ToolId::Orbit => {
                "Drag to orbit around the point under the cursor.\n\
                 Middle mouse button orbits in any tool; Shift+middle pans.\n\
                 Trackpad: two-finger scroll orbits, Shift+scroll pans, pinch zooms."
            }
            ToolId::Pan => "Drag to pan the view. Shift + middle mouse button pans in any tool.",
            ToolId::Zoom => {
                "Drag up to zoom in, down to zoom out. The mouse wheel zooms toward the cursor in any tool.\n\
                 Type a field of view (e.g. 35) and press Enter. Shift+Z zooms to the whole model."
            }
            ToolId::PositionCamera => {
                "Click to stand at a point, looking the way the camera looks now.\n\
                 • Drag from where to stand toward where to look.\n\
                 • Type the eye height (default 1676 mm) before clicking.\n\
                 • Switches to Look Around afterwards."
            }
            ToolId::LookAround => "Drag to turn the view around the eye. Type an eye height and press Enter.",
            ToolId::Walk => {
                "Drag up/down to walk forward/back, left/right to turn.\n\
                 • Shift: move up/down and sideways. Ctrl: run.\n\
                 • Arrow keys: step and turn. Type an eye height and press Enter."
            }
        }
    }

    /// Whether the tool is implemented (others are stubs).
    pub fn implemented(self) -> bool {
        true
    }
}

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
        ToolId::Arc => Box::new(draw::ArcTool::default()),
        ToolId::ArcCenter => Box::new(arcs::ArcCenterTool::new(false)),
        ToolId::Pie => Box::new(arcs::ArcCenterTool::new(true)),
        ToolId::Arc3 => Box::new(arcs::Arc3Tool::default()),
        ToolId::RotatedRectangle => Box::new(arcs::RotatedRectTool::default()),
        ToolId::Axes => Box::new(construct::AxesTool::default()),
        ToolId::Dimension => Box::new(dims::DimensionTool::default()),
        ToolId::Section => Box::new(section::SectionTool::default()),
        ToolId::Text => Box::new(text::TextTool::default()),
        ToolId::Eraser => Box::new(edit::EraserTool::default()),
        ToolId::Paint => Box::new(edit::PaintTool::default()),
        ToolId::TexturePosition => Box::new(texture::TexturePositionTool::default()),
        ToolId::PushPull => Box::new(edit::PushPullTool::default()),
        ToolId::Offset => Box::new(edit::OffsetTool::default()),
        ToolId::Chamfer => Box::new(chamfer::ChamferTool::new(false)),
        ToolId::Fillet => Box::new(chamfer::ChamferTool::new(true)),
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
    fn vcb_label(&self) -> &str {
        "Measurements"
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
        format!("{} tool is not implemented yet.", self.0.name())
    }
    fn cursor(&self) -> CursorIcon {
        CursorIcon::NotAllowed
    }
}
