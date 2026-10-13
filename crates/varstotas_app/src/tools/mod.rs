//! Interactive tools (state machines driven by viewport input). Owner: `tools` agent.
//!
//! Scaffolding by the shell agent: the [`Tool`] trait, [`ToolContext`], [`ToolId`] metadata,
//! working Select / Orbit / Pan / Zoom tools, and stubs for everything else.
//!
//! Coordinates: `PointerInfo::pos` is in viewport-local logical points (origin top-left of
//! the 3D view), which is what `Camera::screen_ray` / `world_to_screen` expect.

// Public API for the Tools agent: not every item is used by the scaffolding yet.
#![allow(dead_code)]

mod nav_tools;
mod select;

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
    Line,
    Arc,
    Rectangle,
    Circle,
    Polygon,
    PushPull,
    Move,
    Rotate,
    Scale,
    Offset,
    FollowMe,
    Tape,
    Protractor,
    Orbit,
    Pan,
    Zoom,
}

impl ToolId {
    pub const ALL: [ToolId; 19] = [
        ToolId::Select,
        ToolId::Eraser,
        ToolId::Paint,
        ToolId::Line,
        ToolId::Arc,
        ToolId::Rectangle,
        ToolId::Circle,
        ToolId::Polygon,
        ToolId::PushPull,
        ToolId::Move,
        ToolId::Rotate,
        ToolId::Scale,
        ToolId::Offset,
        ToolId::FollowMe,
        ToolId::Tape,
        ToolId::Protractor,
        ToolId::Orbit,
        ToolId::Pan,
        ToolId::Zoom,
    ];

    pub fn name(self) -> &'static str {
        match self {
            ToolId::Select => "Select",
            ToolId::Eraser => "Eraser",
            ToolId::Paint => "Paint Bucket",
            ToolId::Line => "Line",
            ToolId::Arc => "Arc",
            ToolId::Rectangle => "Rectangle",
            ToolId::Circle => "Circle",
            ToolId::Polygon => "Polygon",
            ToolId::PushPull => "Push/Pull",
            ToolId::Move => "Move",
            ToolId::Rotate => "Rotate",
            ToolId::Scale => "Scale",
            ToolId::Offset => "Offset",
            ToolId::FollowMe => "Follow Me",
            ToolId::Tape => "Tape Measure",
            ToolId::Protractor => "Protractor",
            ToolId::Orbit => "Orbit",
            ToolId::Pan => "Pan",
            ToolId::Zoom => "Zoom",
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
            ToolId::Polygon | ToolId::FollowMe | ToolId::Protractor => return None,
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
            ToolId::Line => Icon::Line,
            ToolId::Arc => Icon::Arc,
            ToolId::Rectangle => Icon::Rectangle,
            ToolId::Circle => Icon::Circle,
            ToolId::Polygon => Icon::Polygon,
            ToolId::PushPull => Icon::PushPull,
            ToolId::Move => Icon::Move,
            ToolId::Rotate => Icon::Rotate,
            ToolId::Scale => Icon::Scale,
            ToolId::Offset => Icon::Offset,
            ToolId::FollowMe => Icon::FollowMe,
            ToolId::Tape => Icon::Tape,
            ToolId::Protractor => Icon::Protractor,
            ToolId::Orbit => Icon::Orbit,
            ToolId::Pan => Icon::Pan,
            ToolId::Zoom => Icon::Zoom,
        }
    }

    /// One-line description (tooltips).
    pub fn description(self) -> &'static str {
        match self {
            ToolId::Select => "Select entities to modify with other tools.",
            ToolId::Eraser => "Erase edges, guides and other entities.",
            ToolId::Paint => "Apply a material to faces and groups.",
            ToolId::Line => "Draw edges. Closed planar loops become faces.",
            ToolId::Arc => "Draw a 2-point arc.",
            ToolId::Rectangle => "Draw a rectangular face from two corners.",
            ToolId::Circle => "Draw a circle from center and radius.",
            ToolId::Polygon => "Draw a regular polygon.",
            ToolId::PushPull => "Push or pull a face to add or remove volume.",
            ToolId::Move => "Move, stretch or copy entities.",
            ToolId::Rotate => "Rotate, stretch or copy entities about an axis.",
            ToolId::Scale => "Resize and stretch entities.",
            ToolId::Offset => "Offset a face's edges inward or outward.",
            ToolId::FollowMe => "Sweep a face along a path.",
            ToolId::Tape => "Measure distances and create guides.",
            ToolId::Protractor => "Measure angles and create angled guides.",
            ToolId::Orbit => "Rotate the camera around the model.",
            ToolId::Pan => "Move the camera sideways.",
            ToolId::Zoom => "Zoom the camera in or out.",
        }
    }

    /// Longer help shown in the Instructor panel.
    pub fn instructor(self) -> &'static str {
        match self {
            ToolId::Select => {
                "Click an entity to select it.\n\
                 • Double-click a face: select it with its edges.\n\
                 • Triple-click: select everything connected.\n\
                 • Drag left→right: window selection (fully inside).\n\
                 • Drag right→left: crossing selection (touching).\n\
                 • Ctrl/Cmd = add, Shift = toggle, Ctrl/Cmd+Shift = remove.\n\
                 • Double-click a group to edit it; Esc to close it.\n\
                 • Delete erases the selection."
            }
            ToolId::Eraser => {
                "Click or drag over edges to erase them. Erasing an edge removes the faces it bounds."
            }
            ToolId::Paint => {
                "Pick a material in the Materials panel, then click faces.\n\
                 • Alt/Option: sample a material.\n\
                 • Ctrl: fill connected faces.\n\
                 • Shift: replace everywhere."
            }
            ToolId::Line => {
                "Click to start, click to end. Each end starts a new segment.\n\
                 Type a length and press Enter for precision.\n\
                 Shift locks the inference direction; arrow keys lock axes."
            }
            ToolId::Arc => "Click start, click end, then move to set the bulge and click.",
            ToolId::Rectangle => "Click the first corner, then the opposite corner. Type width,height to be exact.",
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
            ToolId::FollowMe => "Select a path, then click the profile face to sweep it.",
            ToolId::Tape => "Click two points to measure. Starting on an edge creates a parallel guide.",
            ToolId::Protractor => "Click the center, click the base direction, click the angle to create a guide.",
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
        }
    }

    /// Whether the tool is implemented (others are stubs).
    pub fn implemented(self) -> bool {
        matches!(self, ToolId::Select | ToolId::Orbit | ToolId::Pan | ToolId::Zoom)
    }
}

/// Create a fresh instance of a tool.
pub fn make_tool(id: ToolId) -> Box<dyn Tool> {
    match id {
        ToolId::Select => Box::new(SelectTool::default()),
        ToolId::Orbit => Box::new(OrbitTool::default()),
        ToolId::Pan => Box::new(PanTool::default()),
        ToolId::Zoom => Box::new(ZoomTool::default()),
        other => Box::new(StubTool(other)),
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
        crate::document::guarded(|| self.doc.model.raycast(&self.pointer.ray)).ok().flatten()
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
