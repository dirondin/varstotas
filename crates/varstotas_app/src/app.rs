//! `VarstotasApp`: window layout, actions, input routing, file handling.

use crate::document::{Document, guarded};
use crate::nav::{self, CameraAnimation};
use crate::overlay::Marker2d;
use crate::tools::{self, OverlayView, PointerInfo, Tool, ToolContext, ToolId, ToolRequest};
use crate::ui::tray::TrayState;
use crate::viewport::Viewport;
use egui::{Key, KeyboardShortcut, Modifiers, PointerButton, Pos2, Response, ViewportCommand};
use varstotas_kernel::glam::DVec3;
use crate::templates::Template;
use varstotas_kernel::{ContextPath, EntityRef, FaceId, InstanceId, IntersectScope, Model, Scene, SceneStyle, io};
use std::rc::Rc;
use varstotas_render::{Camera, FaceStyle, Overlay3d, RenderStyle, StandardView};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

const PREFS_KEY: &str = "varstotas_prefs";
const MAX_RECENT: usize = 8;
/// How long transient status messages stay visible.
const FLASH_SECS: f64 = 3.0;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum FaceStylePref {
    /// Shaded with Textures.
    #[default]
    Shaded,
    /// Shaded: material colours, textures off.
    ShadedColor,
    Monochrome,
    HiddenLine,
    Wireframe,
    /// Physically based materials lit by the environment (track Р).
    Realistic,
}

impl FaceStylePref {
    /// Name stored in [`varstotas_kernel::SceneStyle::face_style`].
    pub fn key(self) -> &'static str {
        match self {
            Self::Shaded => "shaded",
            Self::ShadedColor => "shadedcolor",
            Self::Monochrome => "monochrome",
            Self::HiddenLine => "hiddenline",
            Self::Wireframe => "wireframe",
            Self::Realistic => "realistic",
        }
    }

    pub fn from_key(k: &str) -> Option<Self> {
        [Self::Shaded, Self::ShadedColor, Self::Monochrome, Self::HiddenLine, Self::Wireframe, Self::Realistic]
            .into_iter()
            .find(|s| s.key() == k)
    }

    pub fn to_render(self) -> FaceStyle {
        match self {
            Self::Shaded | Self::ShadedColor => FaceStyle::Shaded,
            Self::Monochrome => FaceStyle::Monochrome,
            Self::HiddenLine => FaceStyle::HiddenLine,
            Self::Wireframe => FaceStyle::Wireframe,
            Self::Realistic => FaceStyle::Realistic,
        }
    }
}

/// UI preferences persisted between runs.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Prefs {
    pub tray_open: bool,
    pub face_style: FaceStylePref,
    pub xray: bool,
    /// Edges in every style but Realistic. Stored under a new key: the old `show_edges` was
    /// shared with Realistic, and its scenes (demo house, gallery) left it off everywhere.
    #[serde(rename = "edges_shown")]
    pub show_edges: bool,
    /// Edges in the Realistic style (off by default: a photo has no lines).
    pub realistic_edges: bool,
    pub profiles: bool,
    pub show_axes: bool,
    pub show_ground: bool,
    /// The sky gradient behind the model; off = a plain light background.
    pub show_sky: bool,
    pub recent: Vec<PathBuf>,
    /// Autosave every `autosave_minutes` while the document has unsaved changes.
    pub autosave: bool,
    pub autosave_minutes: u32,
    /// Template for File → New (None until chosen on first start).
    pub template: Option<Template>,
    /// Script Console editor text.
    pub console_code: String,
    /// Components library folder (None = ~/.varstotas/components).
    pub library_dir: Option<PathBuf>,
    /// Components list: the bundled example generators hidden.
    pub hide_examples: bool,
    /// Components list: the document's own generators (the Document folder) hidden.
    #[serde(default)]
    pub hide_document: bool,
    /// Components list: collapsed folders (the Materials card's sections: "mat:" + folder).
    pub library_collapsed: Vec<String>,
    /// Tile size of the Components and Materials lists.
    pub lib_tiles: crate::ui::tray_lib::TileSize,
    /// Materials library folder (None = ~/.varstotas/materials).
    pub mat_library_dir: Option<PathBuf>,
    /// Materials list: the bundled example materials hidden.
    pub hide_mat_examples: bool,
    /// Favourites of the Components list ("c:" + library file, "g:" + bundled generator).
    pub lib_favourites: Vec<String>,
    /// Components placed last from the library (same keys), most recent first.
    pub lib_recent: Vec<String>,
    /// Favourites of the Materials list ("m:" + library file, "x:" + example name).
    pub mat_favourites: Vec<String>,
    /// Library materials used last (same keys), most recent first.
    pub mat_recent: Vec<String>,
    /// 3D Library → Other Sites: offer new model files of the Downloads folder for import.
    pub watch_downloads: bool,
    /// 3D Library → Other Sites: listen for the Send to Varstotas bookmarklet (127.0.0.1).
    pub send_to_varstotas: bool,
    /// Side bar cards turned off in the Window menu (by title, see `ui::tray::TRAY_CARDS`).
    pub tray_hidden: Vec<String>,
    /// The Snapping menu: master switch and the inferences turned off.
    pub snapping: crate::inference::settings::SnapPrefs,
    /// View → Grid.
    pub grid: crate::inference::settings::GridPrefs,
    /// The folder Watch Downloads watches (None = the system's Downloads folder).
    pub downloads_watch_dir: Option<PathBuf>,
    /// How a new document looks (Preferences → View).
    pub new_view: crate::ui::preferences::ViewDefaults,
}

impl Prefs {
    /// Edges are drawn in the current face style (View → Edges edits this style's switch).
    pub fn edges_on(&self) -> bool {
        if self.face_style == FaceStylePref::Realistic { self.realistic_edges } else { self.show_edges }
    }

    pub fn edges_mut(&mut self) -> &mut bool {
        if self.face_style == FaceStylePref::Realistic { &mut self.realistic_edges } else { &mut self.show_edges }
    }
}

impl Default for Prefs {
    fn default() -> Self {
        Self {
            tray_open: true,
            face_style: FaceStylePref::Shaded,
            xray: false,
            show_edges: true,
            realistic_edges: false,
            profiles: true,
            show_axes: true,
            show_ground: true,
            show_sky: true,
            recent: Vec::new(),
            autosave: true,
            autosave_minutes: crate::autosave::DEFAULT_MINUTES,
            template: None,
            console_code: String::new(),
            library_dir: None,
            hide_examples: false,
            hide_document: false,
            library_collapsed: Vec::new(),
            lib_tiles: Default::default(),
            mat_library_dir: None,
            hide_mat_examples: false,
            lib_favourites: Vec::new(),
            lib_recent: Vec::new(),
            mat_favourites: Vec::new(),
            mat_recent: Vec::new(),
            watch_downloads: false,
            send_to_varstotas: false,
            tray_hidden: Vec::new(),
            snapping: Default::default(),
            grid: Default::default(),
            downloads_watch_dir: None,
            new_view: Default::default(),
        }
    }
}

/// Everything the menus, toolbar, tray and hotkeys can ask the app to do.
#[derive(Clone, Debug, PartialEq)]
pub enum Action {
    New,
    Open,
    OpenPath(PathBuf),
    Save,
    SaveAs,
    ImportObj,
    ImportStl,
    ImportDxf,
    ImportGltf,
    /// File → Import of another native format: 3mf, dae, fbx, ply, csv (points).
    ImportAs(&'static str),
    ExportObj,
    ExportStl,
    ExportDxf,
    ExportGlb,
    Export3mf,
    ExportPly,
    /// Every face with its outer loop, normal, area and material (JSON).
    ExportFacesJson,
    ExportReport,
    /// Credits of the downloaded components in the model (text).
    ExportAttributions,
    /// File → Import with a `@model_import` plugin (its index).
    ImportModelPlugin(usize),
    /// Window → 3D Library.
    ModelLibrary,
    /// Window → Model Info.
    ModelInfo,
    /// Window → Component Parameters (for the selected component).
    RigEditor,
    /// Edit → Purge Unused: components, groups and materials nothing in the scene uses.
    PurgeUnused,
    /// File → Preview Image: from an image file, from the current view, remove.
    PreviewShow,
    PreviewFromFile,
    PreviewFromView,
    PreviewRemove,
    /// File → Export → 2D Drawing (SVG / PDF)…: the options dialog.
    ExportDrawing,
    /// Tools → Unfold…: the flat pattern window.
    Unfold,
    /// Window → Sheets…: layout sheets.
    Sheets,
    WorkPlanes,
    InsertDemo,
    /// File → Open Demo: Country House (a new, unsaved document).
    OpenDemoHouse,
    OpenDemoGallery,
    Quit,
    Undo,
    Redo,
    SelectAll,
    SelectNone,
    Delete,
    MakeGroup,
    MakeComponent,
    Explode,
    HideSelected,
    UnhideAll,
    ExitContext,
    SetTool(ToolId),
    View(StandardView),
    SetParallel(bool),
    ZoomExtents,
    ToggleTray,
    SelectInstance(ContextPath, InstanceId),
    ToggleInstanceHidden(ContextPath, InstanceId),
    /// Outliner: select the instance and frame it in the view.
    FocusInstance(ContextPath, InstanceId),
    /// Outliner: open the group / component for editing.
    EditInstance(ContextPath, InstanceId),
    /// Outliner: delete the instance (one undo step).
    DeleteInstance(ContextPath, InstanceId),
    /// Outliner: move the instance into another context, keeping it in place (one undo step).
    MoveInstance(ContextPath, InstanceId, ContextPath),
    AddScene,
    UpdateScene(usize),
    DeleteScene(usize),
    ActivateScene(usize),
    /// Move scene `i` one place left (`false`) or right (`true`).
    MoveScene(usize, bool),
    /// View → Animation → Play / Pause.
    ToggleAnimation,
    /// Materials → + → New from Texture…: new material from an image file.
    ImportTexture,
    NextScene,
    PrevScene,
    About,
    Preferences,
    Cut,
    Copy,
    /// Paste attached to the cursor (click places it).
    Paste,
    /// Paste at the original position.
    PasteInPlace,
    DeleteGuides,
    /// View → Guides (show / hide).
    ToggleGuides,
    /// Snapping → Snapping on/off (Shift+S).
    ToggleSnapping,
    /// View → Viewports: 1, 2 or 4 panes.
    SetPanes(usize),
    /// A viewport preset (`panes::PRESETS`).
    ViewPreset(usize),
    /// View → Grid (show / hide).
    ToggleGrid,
    /// File → Export → 2D Graphic… (opens the export dialog).
    ExportImage,
    ExportAnimation,
    NewFromTemplate(Template),
    IntersectFaces(IntersectScope),
    ReverseFaces,
    OrientFaces,
    /// Flip Along the context's red (0), green (1) or blue (2) axis.
    FlipAlong(usize),
    /// Axes context menu: Place (Axes tool), Move… (dialog), Reset (world axes).
    PlaceAxes,
    MoveAxes,
    ResetAxes,
    /// View → Section Planes (0) / Section Cuts (1) / Section Fill (2).
    ToggleSectionDisplay(u8),
    /// Selected section plane: Active Cut on/off, Reverse, Align View.
    ActiveCut,
    ReverseSection,
    AlignSectionView,
    /// Soften/Smooth Edges panel: apply to the selected edges (and edges of selected faces).
    SoftenEdges(crate::ui::tray::SoftenSettings),
    /// View → Shadows.
    ToggleShadows,
    /// Shadows panel: new settings (no undo step, like the classic panel).
    SetShadows(varstotas_kernel::ShadowSettings),
    /// Environment card: new settings of the Realistic style (no undo step, like Shadows).
    SetEnvironment(varstotas_kernel::Environment),
    /// Environment card: pick an HDRI file (`.hdr`) for the document.
    PickHdri,
    /// Material editor: pick an image for map `.1` (`normal`, `roughness`, …) of material `.0`.
    PickPbrMap(varstotas_kernel::MaterialId, &'static str),
    /// Tools → 3D Text…: open the dialog.
    Text3d,
    /// Place 3D text with these values (the text follows the cursor).
    PlaceText3d(crate::text3d::Text3dParams),
    /// Tools → Solid Tools → an operation: on the selected solids, or pick them with the tool.
    SolidTool(varstotas_kernel::SolidOp),
    /// Edit → Make Parametric: the selected group / component becomes a parametric group.
    MakeParametric,
    /// Set the attributes of the selected parametric group (Entity Info).
    SetParams(varstotas_kernel::Parametric),
    /// Window → Script Console.
    ToggleConsole,
    /// File → Run Script…
    RunScript,
    /// Extensions menu: run a plugin (with a file for importers / exporters).
    Plugin(usize, Option<PathBuf>),
    PluginImport(usize),
    PluginExport(usize),
    ReloadPlugins,
    /// Components library: place item i / save the selection under a name / show the card.
    PlaceComponent(usize),
    SaveToLibrary(String),
    ShowComponents,
    /// Components card → Generators: place a component of this generator source.
    /// Place generator source with values (over its defaults).
    PlaceGenerator(String, varstotas_kernel::GenValues),
    /// Window → Generator Editor (a new generator when nothing is open).
    GeneratorEditor,
    GenEditorSave,
    GenEditorApply,
    /// Entity Info: generator values for the selected generated components (true = reset to
    /// the defaults first).
    SetGenValues(varstotas_kernel::GenValues, bool),
    /// Values of the selected components with parameters (over the current ones, or reset).
    SetRigValues(varstotas_kernel::GenValues, bool),
    /// Give the selected component its own definition.
    MakeUnique,
}

/// What a [`TextPrompt`] edits.
#[derive(Clone, Debug, PartialEq)]
pub enum PromptKind {
    DimText(ContextPath, varstotas_kernel::DimId),
    /// `dx, dy, dz[, angle]`: move the axes along themselves, rotate about blue (degrees).
    MoveAxes,
}

/// One-line text input dialog.
#[derive(Clone, Debug, PartialEq)]
pub struct TextPrompt {
    pub kind: PromptKind,
    pub title: String,
    pub hint: String,
    pub value: String,
    pub focused: bool,
}

/// File → Export → 2D Graphic… dialog state.
#[derive(Clone, Debug, PartialEq)]
pub struct ExportDialog {
    pub width: u32,
    pub height: u32,
    /// Keep width / height at the viewport's aspect ratio.
    pub lock_aspect: bool,
    pub content: ExportContent,
}

/// What goes into an exported image besides the model.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ExportContent {
    pub transparent: bool,
    pub axes: bool,
    pub guides: bool,
    pub dims: bool,
}

impl Default for ExportContent {
    fn default() -> Self {
        Self { transparent: false, axes: true, guides: true, dims: true }
    }
}

/// A PNG export waiting for the next paint (the renderer lives in the paint callback).
#[derive(Clone, Debug)]
pub struct ExportJob {
    pub path: PathBuf,
    /// Guides only (no tool previews, no selection boxes).
    pub overlay: Overlay3d,
    pub camera: Camera,
    pub size: [u32; 2],
    pub transparent: bool,
    pub axes: bool,
    /// Line widths scale with the image (image pixels per viewport point).
    pub line_scale: f32,
}

/// Something waiting for "save changes?" confirmation.
#[derive(Clone, Debug, PartialEq)]
pub enum Pending {
    New,
    NewFrom(Template),
    Open,
    OpenPath(PathBuf),
    DemoHouse,
    DemoGallery,
    Quit,
}

/// World-space corners of an oriented box (index bits: 1 = x, 2 = y, 4 = z).
type BoxCorners = [[f32; 3]; 8];
/// ((model version, selection version), selected instances' boxes).
type SelBoxes = ((u64, u64), Vec<BoxCorners>);
type SectionCut = ((u64, [u64; 4], bool), Vec<[varstotas_kernel::glam::DVec3; 2]>, Vec<[varstotas_kernel::glam::DVec3; 3]>);

/// View → Animation → Play: steps through the scenes in a loop.
pub(crate) struct ScenePlayer {
    /// Seconds left on the current scene before moving on (counted after the transition).
    pub wait: f64,
}

pub struct VarstotasApp {
    pub(crate) doc: Document,
    pub(crate) camera: Camera,
    pub(crate) cam_anim: Option<CameraAnimation>,
    /// Values of components with parameters moving to a scene's.
    pub(crate) rig_anim: Option<crate::ui::rig_editor::RigAnimation>,
    pub(crate) scene_player: Option<ScenePlayer>,
    pub(crate) viewport: Viewport,
    pub(crate) tool: Box<dyn Tool>,
    pub(crate) vcb: String,
    pub(crate) vcb_error: bool,
    pub(crate) prefs: Prefs,
    pub(crate) base_style: RenderStyle,
    pub(crate) actions: Vec<Action>,
    pub(crate) confirm: Option<Pending>,
    pub(crate) show_about: bool,
    pub(crate) console: crate::ui::console::ConsoleState,
    pub(crate) script_help: crate::ui::script_help::ScriptHelp,
    pub(crate) app_help: crate::ui::app_help::AppHelp,
    pub(crate) log_window: crate::ui::log_window::LogWindow,
    /// Local server for Python scripts and AI agents.
    pub(crate) bridge: crate::bridge_ops::BridgeState,
    pub(crate) plugins: crate::ui::extensions::PluginState,
    pub(crate) library: crate::ui::library::LibraryState,
    pub(crate) mat_library: crate::ui::mat_library::MatLibraryState,
    /// A Components / Materials tile being dragged into the viewport.
    pub(crate) tray_drag: Option<crate::ui::tray_lib::TrayDrag>,
    /// Transient status message (text, expiry time).
    /// Transient message (text, expiry time, is_error), shown next to the tool hint.
    pub(crate) flash: Option<(String, f64, bool)>,
    pub(crate) tray: TrayState,
    pub(crate) debug: crate::debug_capture::DebugCapture,
    last_title: String,
    allow_close: bool,
    pub(crate) tool_pressed: bool,
    /// Primary button state after the last processed pointer event (see
    /// `handle_viewport_input`).
    primary_held: bool,
    pub(crate) last_pointer: Option<Pos2>,
    /// Where the primary button went down in the viewport (click vs drag detection).
    press_local: Option<Pos2>,
    pub(crate) last_modifiers: Modifiers,
    /// A text field had keyboard focus last frame: Escape / Enter that just released it must
    /// not also reach the tools (e.g. clear the selection).
    text_edit_was_focused: bool,
    /// Selected instances' bounding boxes ((model, selection) version → world corners).
    sel_boxes: std::cell::RefCell<Option<SelBoxes>>,
    /// Active section cut lines and fill, keyed by (model version, plane).
    pub(crate) section_cut: std::cell::RefCell<Option<SectionCut>>,
    now: f64,
    pub(crate) prefs_win: crate::ui::preferences::PrefsWindow,
    pub(crate) autosave: AutosaveState,
    /// Copied entities (Edit → Copy / Cut).
    pub(crate) clipboard: Option<Rc<crate::clipboard::Clipboard>>,
    /// Tape Measure resize waiting for confirmation (uniform factor).
    pub(crate) resize_confirm: Option<f64>,
    pub(crate) export_dialog: Option<ExportDialog>,
    /// File → Export → 2D Drawing: options of the dialog (`Some` while open) and the last used.
    pub(crate) vector_dialog: Option<crate::vector_export::VectorOptions>,
    pub(crate) vector_opts: crate::vector_export::VectorOptions,
    /// Lines and texts of the last 2D drawing export (`dump`).
    pub(crate) last_vector: Option<(usize, usize)>,
    /// Tools → 3D Text… dialog (its values); the last placed values are kept for the next one.
    pub(crate) text3d_dialog: Option<crate::text3d::Text3dParams>,
    /// Tools → Unfold… window.
    pub(crate) unfold: crate::ui::unfold::UnfoldState,
    /// Window → Sheets… window.
    pub(crate) sheets: crate::ui::sheets::SheetsState,
    /// Window → Work Planes… window.
    pub(crate) planes_window: crate::ui::workplanes::WorkPlanesWindow,
    pub(crate) render_window: crate::ui::render::RenderWindow,
    pub(crate) anim_window: crate::ui::animation::AnimWindow,
    pub(crate) online: crate::ui::online_mats::OnlineWindow,
    pub(crate) models: crate::ui::model_library::ModelWindow,
    /// Window → Model Info.
    pub(crate) model_info: crate::ui::model_info::ModelInfoWindow,
    /// Rendered thumbnails of the Components card.
    pub(crate) thumbs: crate::thumbs::Thumbs,
    pub(crate) rig_editor: crate::ui::rig_editor::RigEditor,
    /// File → Preview Image → Use Current View: the PNG being rendered for it.
    pub(crate) preview_shot: Option<(PathBuf, crate::ui::library::PreviewTarget)>,
    /// Where the viewport's view / style controls are (`ui/view_hud.rs`): clicks there are theirs.
    pub(crate) hud_rect: egui::Rect,
    /// Split view (`panes.rs`): 1, 2 or 4 panes shown, the active one (its settings are the
    /// app's own), the others' settings (the active slot holds nothing of use).
    pub(crate) pane_count: usize,
    /// The HUD drop-down open this frame: (the active pane then, which) — for the harness.
    pub(crate) hud_open: Option<(usize, &'static str)>,
    pub(crate) active_pane: usize,
    pub(crate) panes: Vec<crate::panes::PaneView>,
    pub(crate) gen_editor: crate::ui::gen_editor::GenEditor,
    pub(crate) text3d_last: crate::text3d::Text3dParams,
    /// One-line text prompt (dimension text, Move Axes).
    pub(crate) text_prompt: Option<TextPrompt>,
    /// The last right-click hit the drawing axes (context menu shows the axes items).
    pub(crate) ctx_on_axes: bool,
    /// Template chooser shown (first start / File → New from Template…).
    pub(crate) show_templates: bool,
}

/// Autosave bookkeeping (see [`crate::autosave`]).
pub(crate) struct AutosaveState {
    worker: crate::autosave::Autosaver,
    /// Directory for untitled autosaves.
    dir: Option<PathBuf>,
    /// False for scripted runs without `VARSTOTAS_AUTOSAVE_DIR` (keeps test runs hermetic).
    allowed: bool,
    /// Time of the last autosave (or of the last moment the document was clean).
    last: f64,
    /// `model_version` written by the last autosave.
    version: u64,
    /// Autosave file owned by the current document (deleted on clean save / close).
    written: Option<PathBuf>,
    /// Autosaves waiting for the user's recover/discard decision.
    pub(crate) recoveries: Vec<crate::autosave::Recovery>,
}

/// The background with the sky off (View → Sky): a neutral off-white.
const NO_SKY: [f32; 4] = [0xF4 as f32 / 255.0, 0xF4 as f32 / 255.0, 0xF2 as f32 / 255.0, 1.0];

fn fallback_style() -> RenderStyle {
    RenderStyle {
        face_style: FaceStyle::Shaded,
        xray: false,
        show_edges: true,
        profiles: true,
        profile_width_px: 2.5,
        show_axes: true,
        show_ground: true,
        sky_top: [0xA8 as f32 / 255.0, 0xC8 as f32 / 255.0, 0xE8 as f32 / 255.0, 1.0],
        sky_horizon: [0xE8 as f32 / 255.0, 0xF0 as f32 / 255.0, 0xF8 as f32 / 255.0, 1.0],
        ground: [0xC8 as f32 / 255.0, 0xC0 as f32 / 255.0, 0xB0 as f32 / 255.0, 1.0],
        edge_color: [0.0, 0.0, 0.0, 1.0],
        light_dir: varstotas_render::DEFAULT_LIGHT_DIR,
        ambient: varstotas_render::DEFAULT_AMBIENT,
        shadows: None,
        grid: None,
        env: Default::default(),
    }
}

impl VarstotasApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        crate::ui::theme::apply(&cc.egui_ctx);
        let prefs: Prefs = cc
            .storage
            .and_then(|s| eframe::get_value(s, PREFS_KEY))
            .unwrap_or_default();
        let base_style = guarded(RenderStyle::default).unwrap_or_else(|_| fallback_style());
        let mut actions = Vec::new();
        if crate::debug_capture::DebugCapture::demo_requested() {
            actions.push(Action::InsertDemo);
        }
        let debug = crate::debug_capture::DebugCapture::from_env();
        let autosave_dir = crate::autosave::data_dir();
        let allowed = !debug.script_active() || std::env::var_os("VARSTOTAS_AUTOSAVE_DIR").is_some_and(|v| !v.is_empty());
        let recoveries = if allowed {
            crate::autosave::startup_recoveries(&prefs.recent, autosave_dir.as_deref())
        } else {
            Vec::new()
        };
        // Scripted (test) runs always start in millimeters, whatever the user's default is.
        let template = if debug.script_active() {
            Template::ArchitecturalMm
        } else {
            prefs.template.unwrap_or(Template::ArchitecturalMm)
        };
        let show_templates = prefs.template.is_none() && !debug.script_active();
        let mut app = Self {
            doc: Document::from_model(template.new_model(), None),
            camera: template.camera(nav::startup_camera()),
            cam_anim: None,
            rig_anim: None,
            scene_player: None,
            viewport: Viewport::default(),
            tool: tools::make_tool(ToolId::Select),
            vcb: String::new(),
            vcb_error: false,
            prefs,
            base_style,
            actions,
            confirm: None,
            show_about: false,
            console: Default::default(),
            script_help: Default::default(),
            app_help: Default::default(),
            log_window: Default::default(),
            bridge: Default::default(),
            plugins: Default::default(),
            library: Default::default(),
            mat_library: Default::default(),
            tray_drag: None,
            flash: None,
            tray: TrayState::default(),
            debug,
            last_title: String::new(),
            allow_close: false,
            tool_pressed: false,
            primary_held: false,
            last_pointer: None,
            press_local: None,
            last_modifiers: Modifiers::NONE,
            text_edit_was_focused: false,
            sel_boxes: Default::default(),
            section_cut: Default::default(),
            now: 0.0,
            prefs_win: Default::default(),
            clipboard: None,
            resize_confirm: None,
            text_prompt: None,
            ctx_on_axes: false,
            export_dialog: None,
            vector_dialog: None,
            vector_opts: Default::default(),
            last_vector: None,
            text3d_dialog: None,
            unfold: Default::default(),
            sheets: Default::default(),
            planes_window: Default::default(),
            render_window: Default::default(),
            anim_window: Default::default(),
            online: Default::default(),
            models: Default::default(),
            preview_shot: None,
            hud_rect: egui::Rect::NOTHING,
            pane_count: 1,
            hud_open: None,
            active_pane: 0,
            panes: (0..4).map(|_| Default::default()).collect(),
            model_info: Default::default(),
            thumbs: Default::default(),
            rig_editor: Default::default(),
            gen_editor: Default::default(),
            text3d_last: Default::default(),
            show_templates: false,
            autosave: AutosaveState {
                worker: crate::autosave::Autosaver::new(),
                dir: autosave_dir,
                allowed,
                last: 0.0,
                version: 0,
                written: None,
                recoveries,
            },
        };
        app.show_templates = show_templates;
        app.start_bridge(&cc.egui_ctx);
        app.load_plugins();
        app
    }

    pub fn render_style(&self) -> RenderStyle {
        let mut s = self.base_style.clone();
        s.face_style = self.prefs.face_style.to_render();
        s.xray = self.prefs.xray;
        s.show_edges = self.prefs.edges_on();
        s.profiles = self.prefs.profiles;
        s.show_axes = self.prefs.show_axes;
        s.show_ground = self.prefs.show_ground;
        if !self.prefs.show_sky {
            s.sky_top = NO_SKY;
            s.sky_horizon = NO_SKY;
        }
        s.grid = self.prefs.grid.style();
        let sh = &self.doc.model.shadows;
        s.shadows = sh.enabled.then(|| varstotas_render::ShadowStyle {
            sun_dir: sh.sun_direction().to_array(),
            light: sh.light,
            dark: sh.dark,
            on_faces: sh.on_faces,
            on_ground: sh.on_ground,
        });
        s.env = crate::pbr::env_style(&self.doc.model.environment, sh.sun_direction().as_vec3().to_array());
        s.env.effects.time = (self.now % 3600.0) as f32;
        s
    }

    pub fn tool_id(&self) -> ToolId {
        self.tool.id()
    }

    pub(crate) fn flash(&mut self, msg: impl Into<String>) {
        self.flash = Some((msg.into(), self.now + FLASH_SECS, false));
    }

    /// An error in the status bar, kept in Window → Log too.
    pub(crate) fn flash_error(&mut self, msg: impl Into<String>) {
        let msg = msg.into();
        // (the same error still on screen — shown again by another path — is logged once)
        if !self.flash.as_ref().is_some_and(|f| f.2 && f.0 == msg && f.1 > self.now) {
            crate::applog::error("", &msg);
        }
        self.flash = Some((msg, self.now + FLASH_SECS, true));
    }

    /// An error in the status bar that is already in Window → Log (the console logged it).
    pub(crate) fn flash_error_quiet(&mut self, msg: impl Into<String>) {
        self.flash = Some((msg.into(), self.now + FLASH_SECS, true));
    }

    pub(crate) fn aspect(&self) -> f64 {
        let s = self.viewport.size();
        (s.x / s.y.max(1.0)) as f64
    }

    // ---- tools -----------------------------------------------------------

    pub(crate) fn pointer_info(&self, pos: Pos2, delta: egui::Vec2, button: Option<PointerButton>, primary_down: bool) -> PointerInfo {
        let vp = self.viewport.size();
        let ray = guarded(|| nav::ray_at(&self.camera, pos, vp)).unwrap_or(varstotas_kernel::Ray {
            origin: self.camera.eye,
            dir: (self.camera.target - self.camera.eye).normalize_or(DVec3::NEG_Z),
        });
        PointerInfo {
            pos,
            delta,
            ray,
            modifiers: self.last_modifiers,
            button,
            primary_down,
            press_origin: None,
            click_count: 1,
        }
    }

    /// Run `f` with the active tool and a fresh `ToolContext`.
    pub(crate) fn with_tool<R>(&mut self, pointer: PointerInfo, f: impl FnOnce(&mut dyn Tool, &mut ToolContext) -> R) -> R {
        let mut requests = Vec::new();
        let viewport = self.viewport.size();
        let r = {
            let mut ctx = ToolContext {
                doc: &mut self.doc,
                camera: &mut self.camera,
                viewport,
                pointer,
                time: self.now,
                requests: &mut requests,
            };
            f(self.tool.as_mut(), &mut ctx)
        };
        for req in requests {
            match req {
                ToolRequest::SwitchTool(id) => self.actions.push(Action::SetTool(id)),
                ToolRequest::ZoomExtents => self.actions.push(Action::ZoomExtents),
                ToolRequest::ConfirmResize(f) => self.resize_confirm = Some(f),
                ToolRequest::PluginPoints(pts) => self.plugin_points(pts),
                ToolRequest::PluginObjects => self.plugin_objects(),
                ToolRequest::EditDimensionText(id) => {
                    let path = self.doc.active_path.clone();
                    if let Some(g) = self.doc.model.dimension_in(&path, id) {
                        self.text_prompt = Some(TextPrompt {
                            kind: PromptKind::DimText(path, id),
                            title: "Edit Dimension Text".into(),
                            hint: "Empty restores the measured length; `<>` stands for it.".into(),
                            value: g.text.unwrap_or_else(|| "<>".into()),
                            focused: false,
                        });
                    }
                }
            }
        }
        r
    }

    fn current_pointer(&self) -> PointerInfo {
        let pos = self.last_pointer.unwrap_or((self.viewport.size() * 0.5).to_pos2());
        self.pointer_info(pos, egui::Vec2::ZERO, None, false)
    }

    pub(crate) fn set_tool(&mut self, id: ToolId) {
        let p = self.current_pointer();
        self.with_tool(p.clone(), |t, ctx| t.deactivate(ctx));
        self.tool = tools::make_tool(id);
        self.with_tool(p, |t, ctx| t.activate(ctx));
        self.vcb.clear();
        self.vcb_error = false;
        self.tool_pressed = false;
    }

    /// The Measurements box sets the grid step while the grid is shown and Select is idle.
    pub(crate) fn grid_vcb(&self) -> bool {
        self.prefs.grid.show && self.tool_id() == ToolId::Select
    }

    pub(crate) fn submit_vcb(&mut self) {
        if self.grid_vcb() {
            let input = std::mem::take(&mut self.vcb);
            match varstotas_kernel::units::parse_length(&input, self.doc.model.unit).filter(|v| *v > 1e-6 && v.is_finite()) {
                Some(v) => {
                    self.prefs.grid.step = v;
                    crate::inference::settings::publish_grid(&self.prefs.grid);
                    self.vcb_error = false;
                    let msg = format!("Grid step {}", crate::tools::common::fmt_len(&self.doc.model, v));
                    self.flash(msg);
                }
                None => {
                    self.vcb_error = true;
                    self.flash_error(format!("Invalid grid step: {input}"));
                }
            }
            return;
        }
        let input = std::mem::take(&mut self.vcb);
        let p = self.current_pointer();
        let ok = self.with_tool(p.clone(), |t, ctx| t.on_vcb(ctx, &input));
        if !ok {
            self.vcb_error = true;
            self.flash_error(format!("Invalid value: {input}"));
        } else {
            self.vcb_error = false;
            self.flash = None;
            // Refresh hover/inference so no stale pre-commit overlay lingers.
            self.with_tool(p, |t, ctx| t.on_pointer_move(ctx));
        }
    }

    // ---- keyboard ---------------------------------------------------------

    fn handle_keyboard(&mut self, ctx: &egui::Context) {
        // Scripted runs inject keys, VCB text and modifiers themselves (`key`, `vcb`, `mods`);
        // real OS keyboard events (including modifier state carried by key events) must not
        // leak in and change tool behaviour.
        if self.debug.synthetic_input() {
            return;
        }
        let focused = ctx.text_edit_focused();
        let was_focused = std::mem::replace(&mut self.text_edit_was_focused, focused);
        if focused || was_focused || self.modal_open() {
            return;
        }
        let cmd = Modifiers::COMMAND;
        let cmd_shift = Modifiers::COMMAND | Modifiers::SHIFT;
        let shortcuts: [(Modifiers, Key, Action); 10] = [
            (cmd_shift, Key::Z, Action::Redo),
            (cmd, Key::Z, Action::Undo),
            (cmd, Key::Y, Action::Redo),
            (cmd_shift, Key::S, Action::SaveAs),
            (cmd, Key::S, Action::Save),
            (cmd, Key::O, Action::Open),
            (cmd, Key::N, Action::New),
            (cmd, Key::A, Action::SelectAll),
            (cmd, Key::T, Action::SelectNone),
            (cmd, Key::Q, Action::Quit),
        ];
        for (m, k, a) in shortcuts {
            if ctx.input_mut(|i| i.consume_shortcut(&KeyboardShortcut::new(m, k))) {
                self.actions.push(a);
            }
        }

        let events = ctx.input(|i| i.events.clone());
        for ev in events {
            match ev {
                egui::Event::Key { key, pressed, modifiers, .. } => {
                    self.last_modifiers = modifiers;
                    if modifiers.command || modifiers.ctrl || modifiers.alt {
                        if !pressed {
                            let p = self.current_pointer();
                            self.with_tool(p, |t, c| t.on_key(c, key, false));
                        }
                        continue;
                    }
                    if !pressed {
                        let p = self.current_pointer();
                        self.with_tool(p, |t, c| t.on_key(c, key, false));
                        continue;
                    }
                    let typing = !self.vcb.is_empty();
                    match key {
                        Key::Backspace if typing => {
                            self.vcb.pop();
                        }
                        Key::Backspace | Key::Delete => self.actions.push(Action::Delete),
                        Key::Enter if typing => self.submit_vcb(),
                        Key::Escape if typing => {
                            self.vcb.clear();
                            self.vcb_error = false;
                        }
                        Key::Z if modifiers.shift && !typing => self.actions.push(Action::ZoomExtents),
                        Key::S if modifiers.shift && !typing => self.actions.push(Action::ToggleSnapping),
                        Key::G if !typing && !modifiers.shift => self.actions.push(Action::MakeGroup),
                        Key::PageDown if !typing => self.actions.push(Action::NextScene),
                        Key::PageUp if !typing => self.actions.push(Action::PrevScene),
                        Key::F1 if !typing => self.app_help_for_tool(),
                        Key::Escape if self.scene_player.is_some() => self.scene_player = None,
                        _ => {
                            // Letters while typing in the VCB go to the VCB (via Text events).
                            let is_letter = key.name().len() == 1 && key.name().chars().all(|c| c.is_ascii_alphabetic());
                            if typing && (is_letter || key == Key::Space) {
                                continue;
                            }
                            if !modifiers.shift
                                && let Some(id) = ToolId::ALL.iter().copied().find(|t| t.shortcut() == Some(key)) {
                                    self.actions.push(Action::SetTool(id));
                                    continue;
                                }
                            let p = self.current_pointer();
                            self.with_tool(p, |t, c| t.on_key(c, key, true));
                        }
                    }
                }
                egui::Event::Text(text) => {
                    for ch in text.chars() {
                        let starts = ch.is_ascii_digit() || "-.,;'\"/<~".contains(ch);
                        let cont = !self.vcb.is_empty() && (ch.is_alphabetic() || ch == ' ' || ch == '*');
                        if starts || cont {
                            if self.vcb_error {
                                self.vcb.clear();
                                self.vcb_error = false;
                            }
                            self.vcb.push(ch);
                        }
                    }
                }
                egui::Event::ModifiersChanged(m) => {
                    if m != self.last_modifiers {
                        self.last_modifiers = m;
                        // Modifier toggles (Move copy, Tape guides, …) react without a mouse move.
                        if self.last_pointer.is_some() {
                            let p = self.current_pointer();
                            self.with_tool(p, |t, c| t.on_pointer_move(c));
                        }
                    }
                }
                egui::Event::Copy => self.actions.push(Action::Copy),
                egui::Event::Cut => self.actions.push(Action::Cut),
                // Cmd+V: egui reads the OS clipboard text; Varstotas pastes its own clipboard
                // (the OS clipboard only carries a marker). Cmd+Shift+V = Paste in Place.
                egui::Event::Paste(_) => {
                    let shift = ctx.input(|i| i.modifiers.shift) || self.last_modifiers.shift;
                    self.actions.push(if shift { Action::PasteInPlace } else { Action::Paste });
                }
                _ => {}
            }
        }
        // A key that opened a dialog (e.g. Enter submitting a Tape resize) must not also
        // answer it in the same frame.
        if self.modal_open() {
            ctx.input_mut(|i| {
                i.events.retain(|e| !matches!(e, egui::Event::Key { key: Key::Enter | Key::Escape, .. }))
            });
        }
    }

    /// A modal dialog (or the Preferences window) is open: model / tool shortcuts and
    /// viewport input are blocked; Enter / Esc belong to the dialog.
    pub(crate) fn modal_open(&self) -> bool {
        self.confirm.is_some()
            || !self.autosave.recoveries.is_empty()
            || self.resize_confirm.is_some()
            || self.text_prompt.is_some()
            || self.export_dialog.is_some()
            || self.vector_dialog.is_some()
            || self.text3d_dialog.is_some()
            || self.show_templates
            || self.show_about
    }

    // ---- viewport input ---------------------------------------------------

    fn handle_viewport_input(&mut self, ctx: &egui::Context, response: &Response) {
        let rect = self.viewport.rect;
        if self.modal_open() {
            // Nothing behind a dialog reacts (navigation, tools, clicks).
            self.primary_held = ctx.input(|i| i.pointer.primary_down());
            self.tool_pressed = false;
            self.press_local = None;
            return;
        }
        let user_nav = self.viewport.nav.handle(response, rect, &self.doc.model, &mut self.camera);
        if user_nav {
            self.cam_anim = None;
            self.scene_player = None;
        }
        if self.scene_player.is_some() && response.contains_pointer() && ctx.input(|i| i.pointer.any_pressed()) {
            self.scene_player = None;
        }
        // Not over the view / style controls drawn on top of the viewport.
        let hud = self.hud_rect.expand(2.0);
        let on_hud = move |p: egui::Pos2| hud.contains(p);
        let over = response.contains_pointer() && !ctx.input(|i| i.pointer.hover_pos()).is_some_and(on_hud);
        let (events, modifiers) = ctx.input(|i| (i.events.clone(), i.modifiers));
        // Primary-button state as of each event, NOT `i.pointer.primary_down()` (the state at
        // the end of the frame): one frame often holds the last moves of a drag *and* the
        // release (high-rate trackpads/mice), and those moves happened with the button down.
        let mut primary_down = self.primary_held;
        // Scripted runs set modifiers with the `mods` command; real OS state must not reset them.
        if !self.debug.synthetic_input() {
            self.last_modifiers = modifiers;
        }
        // Scripted runs drive the tools themselves; a real mouse resting over the window must
        // not inject pointer events (e.g. a move with the button up cancels a scripted drag).
        let events = if self.debug.synthetic_input() { Vec::new() } else { events };
        for ev in events {
            match ev {
                egui::Event::PointerMoved(p) => {
                    if over || self.tool_pressed {
                        let local = (p - rect.min).to_pos2();
                        let delta = self.last_pointer.map(|l| local - l).unwrap_or_default();
                        self.last_pointer = Some(local);
                        let pi = self.pointer_info(local, delta, None, primary_down);
                        self.with_tool(pi, |t, c| t.on_pointer_move(c));
                    }
                }
                egui::Event::PointerButton { pos, button, pressed, modifiers } => {
                    if button == PointerButton::Primary {
                        primary_down = pressed;
                    }
                    if button == PointerButton::Middle {
                        continue;
                    }
                    self.last_modifiers = modifiers;
                    let local = (pos - rect.min).to_pos2();
                    if pressed && over && rect.contains(pos) && !on_hud(pos) {
                        // A new click supersedes any stale error message.
                        if self.flash.as_ref().is_some_and(|f| f.2) {
                            self.flash = None;
                        }
                        if button == PointerButton::Secondary {
                            self.context_select(local);
                        }
                        self.tool_pressed = true;
                        self.last_pointer = Some(local);
                        self.press_local = (button == PointerButton::Primary).then_some(local);
                        let pi = self.pointer_info(local, egui::Vec2::ZERO, Some(button), button == PointerButton::Primary);
                        self.with_tool(pi, |t, c| t.on_pointer_down(c));
                    } else if !pressed && self.tool_pressed {
                        self.tool_pressed = false;
                        let pi = self.pointer_info(local, egui::Vec2::ZERO, Some(button), false);
                        self.with_tool(pi, |t, c| t.on_pointer_up(c));
                        if button == PointerButton::Primary
                            && let Some(p0) = self.press_local.take()
                            && p0.distance(local) < 4.0
                        {
                            self.after_primary_click(local, modifiers);
                        }
                    }
                }
                _ => {}
            }
        }
        self.primary_held = primary_down;
        let clicks = if response.triple_clicked() {
            3
        } else if response.double_clicked() {
            2
        } else {
            0
        };
        if clicks > 0
            && !self.debug.synthetic_input()
            && let Some(local) = self.last_pointer {
                let mut pi = self.pointer_info(local, egui::Vec2::ZERO, Some(PointerButton::Primary), false);
                pi.click_count = clicks;
                self.with_tool(pi, |t, c| t.on_double_click(c));
            }
        if over {
            let icon = if self.viewport.nav.is_dragging() { egui::CursorIcon::Grabbing } else { self.tool.cursor() };
            ctx.set_cursor_icon(icon);
        } else if !self.tool_pressed && !self.debug.synthetic_input() {
            self.last_pointer = None;
        }
    }

    /// Right-click: an entity (or guide) under the cursor that isn't selected becomes the
    /// selection, so the context menu acts on it; a right-click inside the
    /// selection or on empty space keeps the selection.
    fn context_select(&mut self, local: Pos2) {
        use crate::tools::common::{pick_edge_at, pick_face, pick_guide};
        let pi = self.pointer_info(local, egui::Vec2::ZERO, None, false);
        let (ent, guide, dim) = self.with_tool(pi, |_, c| {
            let dim = crate::tools::common::pick_dim(c).map(|d| d.0);
            if let Some(e) = pick_edge_at(c, c.pointer.pos, crate::inference::EDGE_PICK_PX as f32) {
                return (Some(e.entity()), None, None);
            }
            if dim.is_some() {
                return (None, None, dim);
            }
            if let Some(g) = pick_guide(c) {
                return (None, Some(g), None);
            }
            (pick_face(c).map(|h| h.entity()), None, None)
        });
        // Nothing under the cursor but the drawing axes: the menu offers Place / Move / Reset.
        self.ctx_on_axes = ent.is_none() && guide.is_none() && dim.is_none() && self.axes_near(local);
        if let Some(d) = dim {
            if !self.doc.dim_sel.contains(&d) {
                self.doc.set_selection_full(Vec::new(), Vec::new(), vec![d]);
            }
        } else if let Some(e) = ent
            && !self.doc.selection.contains(&e)
        {
            self.doc.set_selection(vec![e]);
        } else if let Some(g) = guide
            && !self.doc.guide_sel.contains(&g)
        {
            self.doc.set_selection_with_guides(Vec::new(), vec![g]);
        }
    }

    /// Apply the text prompt's value. Returns false (dialog stays open) when it is invalid.
    pub(crate) fn apply_prompt(&mut self, p: &TextPrompt) -> bool {
        match &p.kind {
            PromptKind::DimText(path, id) => {
                let (path, id) = (path.clone(), *id);
                let text = Some(p.value.trim().to_string()).filter(|t| !t.is_empty() && t != "<>");
                self.doc.commit("Edit Dimension Text", |m| m.set_dimension_text_in(&path, id, text));
                true
            }
            PromptKind::MoveAxes => {
                let parts: Vec<&str> = p.value.split([',', ';']).map(str::trim).collect();
                if !(3..=4).contains(&parts.len()) {
                    return false;
                }
                let unit = self.doc.model.unit;
                let lens: Option<Vec<f64>> =
                    parts[..3].iter().map(|s| varstotas_kernel::units::parse_length(s, unit)).collect();
                let Some(l) = lens else { return false };
                let ang = match parts.get(3) {
                    Some(s) => match crate::tools::common::parse_angle(s) {
                        Some(a) => a,
                        None => return false,
                    },
                    None => 0.0,
                };
                let a = crate::axes::current();
                let origin = a.origin + a.world_vec(DVec3::new(l[0], l[1], l[2]));
                let r = ang.to_radians();
                let x = a.x * r.cos() + a.y * r.sin();
                let y = a.y * r.cos() - a.x * r.sin();
                if self.doc.try_commit("Move Axes", |m| m.set_drawing_frame(origin, x, y)).is_some() {
                    crate::axes::sync(&self.doc.model);
                }
                true
            }
        }
    }

    /// Viewport point `local` lies on one of the drawing axes (positive half, on screen).
    fn axes_near(&self, local: Pos2) -> bool {
        let a = crate::axes::current();
        let vp = varstotas_kernel::glam::DVec2::new(self.viewport.size().x as f64, self.viewport.size().y as f64);
        let c = varstotas_kernel::glam::DVec2::new(local.x as f64, local.y as f64);
        let len = (self.camera.eye - self.camera.target).length() * 4.0;
        a.dirs().iter().any(|d| {
            crate::inference::geo::project_segment(&self.camera, vp, a.origin, a.origin + *d * len)
                .is_some_and(|(p, q)| crate::inference::geo::dist_point_seg2(c, p, q).0 <= 6.0)
        })
    }

    // ---- actions ------------------------------------------------------------

    fn run_actions(&mut self, ctx: &egui::Context) {
        let actions = std::mem::take(&mut self.actions);
        for a in actions {
            self.run_action(ctx, a);
        }
    }

    fn run_action(&mut self, ctx: &egui::Context, action: Action) {
        match action {
            Action::New => self.guard_dirty(Pending::New),
            Action::Open => self.guard_dirty(Pending::Open),
            Action::OpenPath(p) => self.guard_dirty(Pending::OpenPath(p)),
            Action::Quit => {
                ctx.send_viewport_cmd(ViewportCommand::Close);
            }
            Action::Save => {
                self.save();
            }
            Action::SaveAs => {
                self.save_as();
            }
            Action::ImportTexture => {
                if let Some(path) = rfd::FileDialog::new().add_filter("Image", &["png", "jpg", "jpeg"]).pick_file() {
                    self.add_texture_material(&path, 1000.0);
                }
            }
            Action::ImportObj => self.import(&["obj"], "OBJ"),
            Action::ImportStl => self.import(&["stl"], "STL"),
            Action::ExportObj => self.export("obj"),
            Action::ImportDxf => self.import(&["dxf"], "DXF"),
            Action::ImportGltf => self.import(&["glb", "gltf"], "glTF"),
            Action::ImportAs(ext) => {
                let label = match ext {
                    "dae" => "COLLADA",
                    "csv" => "Points",
                    e => &e.to_ascii_uppercase(),
                };
                self.import(&[ext], label)
            }
            Action::ExportStl => self.export("stl"),
            Action::ExportDxf => self.export("dxf"),
            Action::ExportGlb => self.export("glb"),
            Action::Export3mf => self.export("3mf"),
            Action::ExportPly => self.export("ply"),
            Action::ExportFacesJson => self.export("json"),
            Action::ExportReport => self.export("csv"),
            Action::ExportAttributions => {
                if self.doc.model.attributions().is_empty() {
                    self.flash("No downloaded components with credits in this model".to_string());
                } else {
                    self.export("txt");
                }
            }
            Action::ImportModelPlugin(i) => {
                if let Some(p) = self.plugins.list.get(i).cloned() {
                    let exts: Vec<&str> = p.model_import.iter().map(String::as_str).collect();
                    if let Some(path) = rfd::FileDialog::new().add_filter(p.name.clone(), &exts).pick_file() {
                        self.import_model_file(&path, Some(&p));
                    }
                }
            }
            Action::ModelLibrary => self.models.open = !self.models.open,
            Action::PreviewFromFile => {
                if let Some(f) = rfd::FileDialog::new().add_filter("Image", &["png", "jpg", "jpeg", "tga", "bmp"]).pick_file() {
                    self.preview_from_file(&f);
                }
            }
            Action::PreviewFromView => self.preview_from_view(crate::ui::library::PreviewTarget::Document),
            Action::PreviewShow => self.model_info.preview_open = !self.model_info.preview_open,
            Action::ModelInfo => self.model_info.open = !self.model_info.open,
            Action::RigEditor => self.open_rig_editor(),
            Action::PurgeUnused => self.purge_unused(true, true),
            Action::PreviewRemove => self.set_doc_preview(None),
            Action::ExportDrawing => self.vector_dialog = Some(self.vector_opts),
            Action::Unfold => self.unfold.open = true,
            Action::Sheets => self.sheets.open = true,
            Action::WorkPlanes => self.planes_window.open = true,
            Action::InsertDemo => self.insert_demo(),
            Action::OpenDemoHouse => self.guard_dirty(Pending::DemoHouse),
            Action::OpenDemoGallery => self.guard_dirty(Pending::DemoGallery),
            Action::Undo => match self.doc.undo() {
                Some(l) => self.flash(format!("Undo {l}")),
                None => self.flash("Nothing to undo"),
            },
            Action::Redo => match self.doc.redo() {
                Some(l) => self.flash(format!("Redo {l}")),
                None => self.flash("Nothing to redo"),
            },
            Action::SelectAll => {
                self.doc.select_all();
            }
            Action::SelectNone => self.doc.clear_selection(),
            Action::Delete => self.doc.erase_selection(),
            Action::MakeGroup | Action::MakeComponent => {
                if !self.doc.has_selection() {
                    self.flash("Select something to group first");
                    return;
                }
                let sel = self.doc.selection.clone();
                let guides = self.doc.guide_sel.clone();
                let dims = self.doc.dim_sel.clone();
                let path = self.doc.active_path.clone();
                let r = if action == Action::MakeGroup {
                    self.doc.try_commit("Make Group", |m| m.make_group_with(&path, &sel, &guides, &dims))
                } else {
                    let n = self.doc.model.defs.len();
                    self.doc.try_commit("Make Component", |m| {
                        m.make_component_with(&path, &sel, &guides, &dims, &format!("Component#{n}"))
                    })
                };
                if let Some(id) = r {
                    self.doc.set_selection(vec![EntityRef::Instance(id)]);
                }
            }
            Action::Explode => {
                let path = self.doc.active_path.clone();
                let insts: Vec<InstanceId> = self
                    .doc
                    .selection
                    .iter()
                    .filter_map(|e| if let EntityRef::Instance(i) = e { Some(*i) } else { None })
                    .collect();
                if insts.is_empty() {
                    self.flash("Select a group or component to explode");
                    return;
                }
                let r = self.doc.try_commit("Explode", |m| {
                    let mut out = Vec::new();
                    for i in insts {
                        out.extend(m.explode(&path, i)?);
                    }
                    Ok::<_, varstotas_kernel::OpError>(out)
                });
                if let Some(ents) = r {
                    self.doc.set_selection(ents);
                }
            }
            Action::HideSelected => {
                let sel = self.doc.selection.clone();
                let path = self.doc.active_path.clone();
                if sel.is_empty() {
                    return;
                }
                self.doc.commit("Hide", |m| {
                    if let Some(mesh) = m.mesh_at_mut(&path) {
                        for e in sel {
                            match e {
                                EntityRef::Face(id) => {
                                    if let Some(f) = mesh.faces.get_mut(id) {
                                        f.hidden = true;
                                    }
                                }
                                EntityRef::Edge(id) => {
                                    if let Some(x) = mesh.edges.get_mut(id) {
                                        x.hidden = true;
                                    }
                                }
                                EntityRef::Instance(id) => {
                                    if let Some(x) = mesh.instances.get_mut(id) {
                                        x.hidden = true;
                                    }
                                }
                                EntityRef::Vertex(_) => {}
                            }
                        }
                    }
                });
                self.doc.clear_selection();
            }
            Action::UnhideAll => {
                self.doc.commit("Unhide All", |m| {
                    for (_, d) in m.defs.iter_mut() {
                        d.mesh.faces.values_mut().for_each(|f| f.hidden = false);
                        d.mesh.edges.values_mut().for_each(|e| e.hidden = false);
                        d.mesh.instances.values_mut().for_each(|i| i.hidden = false);
                    }
                });
            }
            Action::ExitContext => {
                self.doc.exit_context();
            }
            Action::SetTool(id) => {
                self.set_tool(id);
                if !id.implemented() {
                    self.flash(format!("{} is not implemented yet", id.name()));
                }
            }
            Action::View(v) => {
                let to = nav::standard_view_camera(&self.doc.model, &self.camera, v, self.aspect());
                self.animate_to(to);
            }
            Action::SetParallel(p) => {
                self.cam_anim = None;
                self.camera.set_ortho(p);
            }
            Action::ZoomExtents => {
                let to = nav::zoom_extents_camera(&self.doc.model, &self.camera, self.aspect());
                self.animate_to(to);
            }
            Action::ToggleTray => self.prefs.tray_open = !self.prefs.tray_open,
            Action::SelectInstance(path, id) => {
                self.doc.set_active_path(path);
                self.doc.set_selection(vec![EntityRef::Instance(id)]);
            }
            Action::FocusInstance(path, id) => {
                self.doc.set_active_path(path.clone());
                self.doc.set_selection(vec![EntityRef::Instance(id)]);
                if let Some(b) = self.doc.model.instance_world_bounds(&path, id) {
                    let mut to = self.camera.clone();
                    to.zoom_extents(b.min, b.max, self.aspect());
                    self.animate_to(to);
                }
            }
            Action::EditInstance(path, id) => {
                let mut p = path;
                p.push(id);
                if self.doc.model.def_at(&p).is_some() {
                    self.doc.set_active_path(p);
                }
            }
            Action::DeleteInstance(path, id) => {
                let mut inner = path.clone();
                inner.push(id);
                if self.doc.active_path.starts_with(&inner) {
                    self.doc.set_active_path(path.clone());
                }
                self.doc.selection.retain(|e| *e != EntityRef::Instance(id));
                self.doc.commit("Delete", |m| m.remove_instance(&path, id));
                self.doc.selection_version += 1;
            }
            Action::MoveInstance(from, id, to) => {
                if !self.doc.model.can_move_instance(&from, id, &to) {
                    self.flash("Can't move a group or component into itself");
                    return;
                }
                let mut inner = from.clone();
                inner.push(id);
                if self.doc.active_path.starts_with(&inner) {
                    self.doc.set_active_path(Vec::new());
                }
                let label = if to.is_empty() { "Move Out of Group" } else { "Move into Group" };
                if let Some(Some(nid)) = self.doc.commit(label, |m| m.move_instance(&from, id, &to)) {
                    self.doc.set_active_path(to);
                    self.doc.set_selection(vec![EntityRef::Instance(nid)]);
                }
            }
            Action::ToggleInstanceHidden(path, id) => {
                self.doc.commit("Hide", |m| {
                    if let Some(i) = m.mesh_at_mut(&path).and_then(|mesh| mesh.instances.get_mut(id)) {
                        i.hidden = !i.hidden;
                    }
                });
            }
            Action::AddScene => {
                let n = self.doc.model.scenes.len() + 1;
                let scene = self.scene_from_camera(format!("Scene {n}"));
                self.doc.commit("Add Scene", |m| m.scenes.push(scene));
                self.tray.selected_scene = Some(self.doc.model.scenes.len().saturating_sub(1));
            }
            Action::UpdateScene(i) => {
                let name = self.doc.model.scenes.get(i).map(|s| s.name.clone()).unwrap_or_default();
                let scene = self.scene_from_camera(name);
                self.doc.commit("Update Scene", |m| {
                    if let Some(s) = m.scenes.get_mut(i) {
                        *s = scene;
                    }
                });
            }
            Action::DeleteScene(i) => {
                self.doc.commit("Delete Scene", |m| {
                    if i < m.scenes.len() {
                        m.scenes.remove(i);
                    }
                });
                self.tray.selected_scene = None;
            }
            Action::ActivateScene(i) => {
                self.scene_player = None;
                self.go_to_scene(i);
            }
            Action::MoveScene(i, right) => {
                let j = if right { i + 1 } else { i.wrapping_sub(1) };
                if i < self.doc.model.scenes.len() && j < self.doc.model.scenes.len() {
                    self.doc.commit("Move Scene", |m| m.scenes.swap(i, j));
                    self.tray.selected_scene = Some(j);
                }
            }
            Action::ToggleAnimation => {
                if self.scene_player.take().is_none() {
                    self.play_animation();
                }
            }
            Action::NextScene | Action::PrevScene => {
                self.scene_player = None;
                if let Some(i) = self.step_scene(action == Action::NextScene, false) {
                    self.go_to_scene(i);
                }
            }
            Action::About => self.show_about = true,
            Action::Preferences => self.prefs_win.open = true,
            Action::Copy => {
                self.copy(ctx);
            }
            Action::Cut => {
                if self.copy(ctx) {
                    self.doc.erase_selection_labeled("Cut");
                }
            }
            Action::Paste | Action::PasteInPlace => self.paste(action == Action::PasteInPlace),
            Action::DeleteGuides => {
                if !self.doc.delete_all_guides() {
                    self.flash("There are no guides");
                }
            }
            Action::SetPanes(n) => self.set_pane_count(n),
            Action::ViewPreset(i) => self.apply_preset(i),
            Action::ToggleSnapping => {
                self.prefs.snapping.enabled = !self.prefs.snapping.enabled;
                crate::inference::settings::publish(&self.prefs.snapping);
        crate::inference::settings::publish_grid(&self.prefs.grid);
                self.flash(if self.prefs.snapping.enabled { "Snapping on" } else { "Snapping off" });
            }
            Action::ToggleGrid => {
                self.prefs.grid.show = !self.prefs.grid.show;
                crate::inference::settings::publish_grid(&self.prefs.grid);
            }
            Action::ToggleGuides => {
                let hidden = !self.doc.model.guides_hidden;
                self.doc.mutate_untracked(|m| m.guides_hidden = hidden);
                self.doc.prune_selection();
            }
            Action::Text3d => self.text3d_dialog = Some(self.text3d_last.clone()),
            Action::PlaceText3d(p) => self.place_text3d(p),
            Action::SolidTool(op) => self.solid_tool(op),
            Action::ToggleConsole => self.console.open = !self.console.open,
            Action::Plugin(i, path) => self.start_plugin(i, path),
            Action::PlaceComponent(i) => self.place_component(i),
            Action::PlaceGenerator(src, values) => self.place_generator(&src, &values),
            Action::GeneratorEditor => {
                if self.gen_editor.source.is_empty() {
                    self.gen_editor.new_generator();
                }
                self.gen_editor.open = true;
            }
            Action::GenEditorSave => self.gen_editor_save(),
            Action::GenEditorApply => self.gen_editor_apply(),
            Action::SetGenValues(v, reset) => self.set_gen_values(v, reset),
            Action::SetRigValues(v, reset) => self.set_rig_values(v, reset),
            Action::MakeUnique => {
                let path = self.doc.active_path.clone();
                let [EntityRef::Instance(id)] = self.doc.selection.as_slice() else {
                    return self.flash("Select one component");
                };
                let id = *id;
                if self.doc.try_commit("Make Unique", |m| m.make_unique(&path, id)).is_none()
                    && let Some(e) = self.doc.last_error.clone()
                {
                    self.flash_error(e);
                }
            }
            Action::SaveToLibrary(name) => self.save_to_library(&name),
            Action::ShowComponents => {
                self.prefs.tray_open = true;
                self.prefs.tray_hidden.retain(|c| c != "Components");
                self.library.open_card = true;
            }
            Action::PluginImport(i) => self.plugin_file(i, false),
            Action::PluginExport(i) => self.plugin_file(i, true),
            Action::ReloadPlugins => {
                self.load_plugins();
                self.flash(format!("{} plugins loaded", self.plugins.list.len()));
            }
            Action::RunScript => {
                if let Some(p) = rfd::FileDialog::new().add_filter("Script", &["rhai", "py"]).pick_file() {
                    self.run_script_file(&p);
                }
            }
            Action::MakeParametric | Action::SetParams(_) => {
                let path = self.doc.active_path.clone();
                let [EntityRef::Instance(id)] = self.doc.selection.as_slice() else {
                    self.flash("Select one group or component");
                    return;
                };
                let id = *id;
                let r = match action {
                    Action::SetParams(p) => self.doc.try_commit("Component Parameters", |m| m.set_instance_params(&path, id, p)),
                    _ => self.doc.try_commit("Make Parametric", |m| m.make_parametric(&path, id).map(|_| ())),
                };
                if r.is_none()
                    && let Some(e) = self.doc.last_error.clone()
                {
                    self.flash_error(e);
                }
            }
            Action::ToggleShadows => self.doc.mutate_untracked(|m| m.shadows.enabled = !m.shadows.enabled),
            Action::SetShadows(sh) => {
                if self.doc.model.shadows != sh {
                    self.doc.mutate_untracked(|m| m.shadows = sh);
                }
            }
            Action::SetEnvironment(e) => {
                let e = e.sanitized();
                if self.doc.model.environment != e {
                    self.doc.mutate_untracked(|m| m.environment = e);
                }
            }
            Action::PickHdri => {
                if let Some(path) = rfd::FileDialog::new().add_filter("HDRI", &["hdr", "png", "jpg", "jpeg"]).pick_file() {
                    self.load_hdri(&path);
                }
            }
            Action::PickPbrMap(mid, map) => {
                if let Some(path) = rfd::FileDialog::new().add_filter("Image", &["png", "jpg", "jpeg"]).pick_file() {
                    self.load_pbr_map(mid, map, &path);
                }
            }
            Action::ToggleSectionDisplay(k) => {
                self.doc.mutate_untracked(|m| {
                    let d = &mut m.section_display;
                    match k {
                        0 => d.planes = !d.planes,
                        1 => d.cuts = !d.cuts,
                        _ => d.fill = !d.fill,
                    }
                });
                self.doc.prune_selection();
            }
            Action::ActiveCut => {
                if let Some(i) = self.doc.section_sel {
                    let on = self.doc.model.sections.get(i).is_some_and(|s| s.active);
                    self.doc.commit("Active Cut", |m| {
                        m.set_active_section((!on).then_some(i));
                        m.section_display.cuts = true;
                    });
                }
            }
            Action::ReverseSection => {
                if let Some(i) = self.doc.section_sel {
                    self.doc.commit("Reverse Section", |m| {
                        if let Some(s) = m.sections.get_mut(i) {
                            *s = s.reversed();
                        }
                    });
                }
            }
            Action::SoftenEdges(st) => {
                let edges = self.soften_targets();
                if !edges.is_empty() {
                    let p = self.doc.active_path.clone();
                    self.doc.commit("Soften Edges", |m| {
                        m.mesh_at_mut(&p).map(|mesh| mesh.soften_by_angle(&edges, st.angle, st.smooth, st.coplanar))
                    });
                }
            }
            Action::AlignSectionView => {
                if let Some(s) = self.doc.section_sel.and_then(|i| self.doc.model.sections.get(i)) {
                    let n = s.normal;
                    let mut cam = self.camera.clone();
                    let d = cam.distance().max(1e-3);
                    cam.up = if n.z.abs() > 0.999 { varstotas_kernel::glam::DVec3::Y * n.z.signum() } else { varstotas_kernel::glam::DVec3::Z };
                    cam.eye = cam.target - n * d;
                    let to = nav::zoom_extents_camera(&self.doc.model, &cam, self.aspect());
                    self.animate_to(to);
                }
            }
            Action::ExportAnimation => self.anim_window.open = true,
            Action::ExportImage => {
                // Physical pixels (what the viewport shows on a Retina screen).
                let s = self.viewport.size() * ctx.pixels_per_point();
                self.export_dialog = Some(ExportDialog {
                    width: s.x.round().max(1.0) as u32,
                    height: s.y.round().max(1.0) as u32,
                    lock_aspect: true,
                    content: ExportContent {
                        transparent: false,
                        axes: self.prefs.show_axes,
                        guides: !self.doc.model.guides_hidden,
                        dims: true,
                    },
                });
            }
            Action::NewFromTemplate(t) => {
                self.prefs.template.get_or_insert(t);
                self.guard_dirty(Pending::NewFrom(t));
            }
            Action::IntersectFaces(scope) => self.intersect(scope),
            Action::ReverseFaces => {
                let faces = self.selected_faces();
                if faces.is_empty() {
                    self.flash("Select faces to reverse");
                    return;
                }
                let path = self.doc.active_path.clone();
                self.doc.commit("Reverse Faces", |m| m.mesh_at_mut(&path).map(|mesh| mesh.reverse_faces(&faces)));
            }
            Action::OrientFaces => {
                let Some(f) = self.selected_faces().first().copied() else {
                    self.flash("Select a face to orient its neighbours like it");
                    return;
                };
                let path = self.doc.active_path.clone();
                let n = self
                    .doc
                    .commit("Orient Faces", |m| m.mesh_at_mut(&path).map(|mesh| mesh.orient_faces_like(f)).unwrap_or(0));
                if let Some(n) = n {
                    self.flash(match n {
                        0 => "All connected faces already agree".to_string(),
                        1 => "1 face reversed".to_string(),
                        n => format!("{n} faces reversed"),
                    });
                }
            }
            Action::PlaceAxes => self.set_tool(ToolId::Axes),
            Action::MoveAxes => {
                self.text_prompt = Some(TextPrompt {
                    kind: PromptKind::MoveAxes,
                    title: "Move Axes".into(),
                    hint: "Offset along red, green, blue and an optional rotation about blue: `dx, dy, dz[, angle]`.".into(),
                    value: "0, 0, 0".into(),
                    focused: false,
                });
            }
            Action::ResetAxes => {
                if self.doc.model.axes.is_some() {
                    self.doc.commit("Reset Axes", |m| m.reset_axes());
                    crate::axes::sync(&self.doc.model);
                }
            }
            Action::FlipAlong(axis) => {
                let sel: Vec<EntityRef> = self.doc.selection.clone();
                if sel.is_empty() {
                    self.flash("Select something to flip");
                    return;
                }
                let path = self.doc.active_path.clone();
                // Drawing axes at the top level; inside a group its own (local) axes.
                let a = if path.is_empty() { crate::axes::current().dir(axis) } else { [DVec3::X, DVec3::Y, DVec3::Z][axis.min(2)] };
                let label = ["Flip Along Red", "Flip Along Green", "Flip Along Blue"][axis.min(2)];
                self.doc.try_commit(label, |m| m.flip_along(&path, &sel, a));
                self.doc.prune_selection();
            }
        }
    }

    fn selected_faces(&self) -> Vec<FaceId> {
        self.doc.selection.iter().filter_map(|e| if let EntityRef::Face(f) = e { Some(*f) } else { None }).collect()
    }

    fn intersect(&mut self, scope: IntersectScope) {
        let sel = self.doc.selection.clone();
        let usable = sel.iter().any(|e| matches!(e, EntityRef::Face(_) | EntityRef::Instance(_)));
        if !usable {
            self.flash("Select faces, groups or components to intersect");
            return;
        }
        let path = self.doc.active_path.clone();
        let label = match scope {
            IntersectScope::Model => "Intersect With Model",
            IntersectScope::Selection => "Intersect With Selection",
            IntersectScope::Context => "Intersect With Context",
        };
        // Nothing created → no (empty) undo step.
        let r = self.doc.try_commit_if(label, |m| m.intersect_faces(&path, &sel, scope), |e| !e.is_empty());
        if let Some(edges) = r {
            self.doc.prune_selection();
            self.flash(match edges.len() {
                0 => "No intersections found".to_string(),
                1 => "1 intersection edge created".to_string(),
                n => format!("{n} intersection edges created"),
            });
        }
    }

    /// Edit → Copy: returns true when something was copied.
    fn copy(&mut self, ctx: &egui::Context) -> bool {
        if !self.doc.has_selection() {
            self.flash("Nothing selected to copy");
            return false;
        }
        match guarded(|| crate::clipboard::Clipboard::from_selection(&self.doc)) {
            Ok(Ok(c)) => {
                let n = c.count;
                self.clipboard = Some(Rc::new(c));
                // The OS clipboard gets a text marker (other apps then don't paste stale text
                // instead; Cmd+V only produces a paste event when the clipboard has text).
                if !self.debug.script_active() {
                    ctx.copy_text(format!("Varstotas: {n} entities"));
                }
                true
            }
            Ok(Err(e)) => {
                self.flash_error(format!("Copy failed: {e}"));
                false
            }
            Err(e) => {
                self.flash_error(format!("Copy failed: {e}"));
                false
            }
        }
    }

    /// Start placing 3D text: it follows the cursor until a click (shown as the Move tool).
    pub(crate) fn place_text3d(&mut self, p: crate::text3d::Text3dParams) {
        if crate::text3d::outlines(&p).is_empty() {
            self.flash_error("The text has no visible characters");
            return;
        }
        self.text3d_last = p.clone();
        let ptr = self.current_pointer();
        self.with_tool(ptr.clone(), |t, ctx| t.deactivate(ctx));
        self.tool = Box::new(crate::tools::text3d::Text3dPlaceTool::new(p));
        self.with_tool(ptr, |t, ctx| t.activate(ctx));
        self.vcb.clear();
        self.vcb_error = false;
        self.tool_pressed = false;
    }

    /// Solid Tools: run on the selection, or start the tool to click the solids.
    pub(crate) fn solid_tool(&mut self, op: varstotas_kernel::SolidOp) {
        if crate::tools::solid::apply_to_selection(&mut self.doc, op).is_some() {
            return;
        }
        let ptr = self.current_pointer();
        self.with_tool(ptr.clone(), |t, ctx| t.deactivate(ctx));
        self.tool = Box::new(crate::tools::solid::SolidTool::new(op));
        self.with_tool(ptr, |t, ctx| t.activate(ctx));
        self.vcb.clear();
        self.vcb_error = false;
        self.tool_pressed = false;
    }

    fn paste(&mut self, in_place: bool) {
        let Some(clip) = self.clipboard.clone() else {
            self.flash("The clipboard is empty");
            return;
        };
        if in_place {
            crate::tools::transform::paste_clipboard(&mut self.doc, &clip, &varstotas_kernel::glam::DMat4::IDENTITY);
            return;
        }
        self.install_tool(Box::new(crate::tools::transform::PasteTool::new(clip)));
    }

    /// Switch to a tool that is not in the toolbar (paste, plugin point picking, …).
    pub(crate) fn install_tool(&mut self, tool: Box<dyn Tool>) {
        let p = self.current_pointer();
        self.with_tool(p.clone(), |t, ctx| t.deactivate(ctx));
        self.tool = tool;
        self.with_tool(p, |t, ctx| t.activate(ctx));
        self.vcb.clear();
        self.vcb_error = false;
        self.tool_pressed = false;
    }

    /// Apply a confirmed Tape Measure resize.
    pub(crate) fn apply_resize(&mut self, factor: f64) {
        if crate::tools::construct::resize_active(&mut self.doc, factor) {
            // The Tape's measurement is stale after resizing.
            self.set_tool(self.tool_id());
            self.flash(format!("Resized ×{factor:.4}"));
        }
    }

    /// Run a pending PNG export with the viewport's renderer.
    pub(crate) fn export_png_to(&mut self, path: PathBuf, size: [u32; 2], content: ExportContent) {
        let vp = self.viewport.size();
        let k = (size[0] as f32 / vp.x.max(1.0)).max(size[1] as f32 / vp.y.max(1.0));
        // Pixels per logical point of the image (lines keep their on-screen look).
        let line_scale = k.max(0.25);
        // Guide point crosses keep their on-screen size relative to the image.
        let pts = egui::vec2(size[0] as f32, size[1] as f32) / line_scale;
        let mut overlay = Overlay3d::default();
        // Exported guides are never drawn as selected.
        if content.guides {
            let sel = std::mem::take(&mut self.doc.guide_sel);
            {
                let view = OverlayView { doc: &self.doc, camera: &self.camera, viewport: pts, pointer: None };
                tools::common::draw_guides(&view, &mut overlay);
            }
            self.doc.guide_sel = sel;
        }
        // Dimensions too (never selected in the image).
        if content.dims {
            let dsel = std::mem::take(&mut self.doc.dim_sel);
            {
                let view = OverlayView { doc: &self.doc, camera: &self.camera, viewport: pts, pointer: None };
                tools::common::draw_dims(&view, &mut overlay);
            }
            self.doc.dim_sel = dsel;
        }
        // Section planes (never selected) and the active cut's lines and fill.
        let ssel = self.doc.section_sel.take();
        {
            let view = OverlayView { doc: &self.doc, camera: &self.camera, viewport: pts, pointer: None };
            tools::section::draw_sections(&view, &mut overlay);
        }
        self.doc.section_sel = ssel;
        self.section_cut_overlay(&mut overlay);
        let camera = self.camera.clone();
        let (transparent, axes) = (content.transparent, content.axes);
        self.viewport.export = Some(ExportJob { path, overlay, camera, size, transparent, axes, line_scale });
    }

    /// Scene tabs above the viewport: click goes to the scene, right-click for the menu.
    fn scene_tabs(&mut self, ui: &mut egui::Ui) {
        let names: Vec<String> = self.doc.model.scenes.iter().map(|s| s.name.clone()).collect();
        let n = names.len();
        ui.horizontal_wrapped(|ui| {
            for (i, name) in names.into_iter().enumerate() {
                let r = ui.add(egui::Button::selectable(self.tray.selected_scene == Some(i), name));
                if r.clicked() {
                    self.actions.push(Action::ActivateScene(i));
                }
                r.context_menu(|ui| {
                    let mut item = |ui: &mut egui::Ui, label: &str, enabled: bool, a: Action| {
                        if ui.add_enabled(enabled, egui::Button::new(label)).clicked() {
                            self.actions.push(a);
                        }
                    };
                    item(ui, "Update", true, Action::UpdateScene(i));
                    item(ui, "Add", true, Action::AddScene);
                    item(ui, "Delete", true, Action::DeleteScene(i));
                    ui.separator();
                    item(ui, "Move Left", i > 0, Action::MoveScene(i, false));
                    item(ui, "Move Right", i + 1 < n, Action::MoveScene(i, true));
                    ui.separator();
                    let play = if self.scene_player.is_some() { "Pause Animation" } else { "Play Animation" };
                    item(ui, play, true, Action::ToggleAnimation);
                });
            }
        });
    }

    /// Add a material textured with the image at `path` (one tile `width_mm` wide) and make it
    /// the paint material.
    pub(crate) fn add_texture_material(&mut self, path: &Path, width_mm: f64) {
        let (tex, decoded) = match crate::textures::load_file(path, width_mm) {
            Ok(t) => t,
            Err(e) => {
                self.flash_error(format!("Can't use {}: {e}", path.display()));
                return;
            }
        };
        let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("Texture").to_string();
        let mut name = stem.clone();
        let mut k = 2;
        while self.doc.model.materials.values().any(|m| m.name == name) {
            name = format!("{stem} {k}");
            k += 1;
        }
        let pm = crate::document::PaletteMaterial {
            name,
            color: crate::textures::average_color(&decoded),
            image: Some(tex),
            pbr: None,
        };
        let p2 = pm.clone();
        self.doc.commit("Add Material", |m| {
            p2.ensure_in(m);
        });
        self.doc.current_material = Some(pm);
        self.set_tool(ToolId::Paint);
    }

    /// A scene of the current view: camera, display style and hidden tags.
    pub(crate) fn scene_from_camera(&self, name: String) -> Scene {
        let c = &self.camera;
        let mut s = Scene::camera(name, c.eye, c.target, c.up, c.fov_y_deg, c.ortho);
        s.style = Some(SceneStyle {
            face_style: self.prefs.face_style.key().into(),
            xray: self.prefs.xray,
            edges: self.prefs.edges_on(),
            profiles: self.prefs.profiles,
        });
        s.hidden_tags = Some(self.doc.model.tags.iter().filter(|(_, t)| !t.visible).map(|(id, _)| id).collect());
        s.untagged_hidden = Some(self.doc.model.untagged_hidden);
        s.sections = Some(self.doc.model.scene_sections());
        s.shadows = Some(self.doc.model.shadows);
        s.environment = Some(self.doc.model.environment);
        let rigs = self.doc.model.rig_instances();
        s.rigs = (!rigs.is_empty()).then_some(rigs);
        s
    }

    /// Camera stored in scene `s` (projection size taken from the eye distance).
    pub(crate) fn scene_camera(&self, s: &Scene) -> Camera {
        let mut to = self.camera.clone();
        to.eye = s.eye;
        to.target = s.target;
        to.up = s.up;
        to.fov_y_deg = s.fov_y_deg;
        to.ortho = s.ortho;
        if s.ortho {
            let d = (s.eye - s.target).length();
            to.ortho_height = 2.0 * d * (s.fov_y_deg.to_radians() * 0.5).tan();
        }
        to
    }

    /// Make scene `i` current: style and tags switch at once, the camera moves with the
    /// model's transition settings (a cut when transitions are off).
    pub(crate) fn go_to_scene(&mut self, i: usize) {
        let Some(s) = self.doc.model.scenes.get(i).cloned() else { return };
        self.tray.selected_scene = Some(i);
        if let Some(st) = &s.style {
            if let Some(fs) = FaceStylePref::from_key(&st.face_style) {
                self.prefs.face_style = fs;
            }
            self.prefs.xray = st.xray;
            *self.prefs.edges_mut() = st.edges;
            self.prefs.profiles = st.profiles;
        }
        if let Some(hidden) = &s.hidden_tags {
            let changes = self.doc.model.tags.iter().any(|(id, t)| t.visible == hidden.contains(&id));
            if changes {
                self.doc.mutate_untracked(|m| {
                    for (id, t) in m.tags.iter_mut() {
                        t.visible = !hidden.contains(&id);
                    }
                });
            }
        }
        if let Some(u) = s.untagged_hidden
            && u != self.doc.model.untagged_hidden
        {
            self.doc.mutate_untracked(|m| m.untagged_hidden = u);
        }
        if let Some(sec) = s.sections
            && sec != self.doc.model.scene_sections()
        {
            self.doc.mutate_untracked(|m| m.apply_scene_sections(&sec));
            self.doc.prune_selection();
        }
        if let Some(sh) = s.shadows
            && sh != self.doc.model.shadows
        {
            self.doc.mutate_untracked(|m| m.shadows = sh);
        }
        if let Some(e) = s.environment
            && e != self.doc.model.environment
        {
            self.doc.mutate_untracked(|m| m.environment = e);
        }
        let to = self.scene_camera(&s);
        let set = self.doc.model.scene_settings;
        self.scene_rigs(&s, if set.transitions { set.transition_secs.max(0.0) } else { 0.0 });
        if set.transitions && set.transition_secs > 0.0 {
            self.cam_anim = Some(CameraAnimation::new(self.camera.clone(), to, set.transition_secs as f32));
        } else {
            self.cam_anim = None;
            self.camera = to;
        }
    }

    /// Next (or previous) scene after the current one, wrapping around; with `animated`
    /// only scenes included in the animation count.
    fn step_scene(&self, forward: bool, animated: bool) -> Option<usize> {
        let scenes = &self.doc.model.scenes;
        let n = scenes.len();
        let cur = self.tray.selected_scene.filter(|i| *i < n);
        (1..=n)
            .map(|k| match cur {
                Some(c) if forward => (c + k) % n,
                Some(c) => (c + n * 2 - k) % n,
                None if forward => k - 1,
                None => n - k,
            })
            .find(|i| !animated || scenes[*i].in_animation)
    }

    /// View → Animation → Play: start at the next scene and loop until stopped.
    fn play_animation(&mut self) {
        match self.step_scene(true, true) {
            Some(i) => {
                self.go_to_scene(i);
                self.scene_player = Some(ScenePlayer { wait: self.doc.model.scene_settings.delay_secs });
            }
            None => self.flash("No scenes to play. Add scenes in the Scenes panel."),
        }
    }

    /// Advance the scene animation by `dt` seconds (after the camera transition finishes).
    fn step_player(&mut self, dt: f64) {
        let Some(p) = self.scene_player.as_mut() else { return };
        if self.cam_anim.is_some() || self.rig_anim.is_some() {
            return;
        }
        p.wait -= dt;
        if p.wait <= 0.0 {
            self.scene_player = None;
            self.play_animation();
        }
    }

    fn animate_to(&mut self, to: Camera) {
        self.cam_anim = Some(CameraAnimation::new(self.camera.clone(), to, 0.55));
    }

    // ---- files ----------------------------------------------------------------

    fn guard_dirty(&mut self, p: Pending) {
        if self.doc.dirty {
            self.confirm = Some(p);
        } else {
            self.proceed(p);
        }
    }

    /// Continue a pending operation after the user confirmed (or nothing was dirty).
    pub(crate) fn proceed(&mut self, p: Pending) {
        match p {
            Pending::New => {
                let t = self.prefs.template.unwrap_or(Template::ArchitecturalMm);
                self.new_from_template(t);
            }
            Pending::NewFrom(t) => self.new_from_template(t),
            Pending::Open => {
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("Varstotas model", &["vrst"])
                    .pick_file()
                {
                    self.open_path(&path);
                }
            }
            Pending::OpenPath(path) => self.open_path(&path),
            Pending::DemoHouse => {
                self.replace_document(Document::from_model(crate::demo::country_house(), None));
                self.go_to_scene(0);
                self.flash("Demo: Country House — see the Scenes card and Window → Render…");
            }
            Pending::DemoGallery => {
                self.replace_document(Document::from_model(crate::demo::material_gallery(), None));
                self.go_to_scene(0);
                self.flash("Demo: Material Gallery — see the Scenes card and Window → Render…");
            }
            Pending::Quit => {
                self.allow_close = true;
                self.tray.close_requested = true;
            }
        }
    }

    pub(crate) fn new_from_template(&mut self, t: Template) {
        self.replace_document(Document::from_model(t.new_model(), None));
        self.camera = t.camera(nav::startup_camera());
        self.apply_view_defaults();
    }

    fn replace_document(&mut self, doc: Document) {
        // The old document was saved or its changes discarded: its autosave is obsolete.
        if let Some(p) = self.autosave.written.take() {
            self.autosave.worker.delete(p);
        }
        self.autosave.version = doc.model_version;
        self.autosave.last = self.now;
        self.doc = doc;
        self.viewport.invalidate();
        self.tool = tools::make_tool(ToolId::Select);
        self.vcb.clear();
        self.tray = TrayState::default();
        let mut cam = nav::startup_camera();
        if !self.doc.model.scenes.is_empty() {
            let s = &self.doc.model.scenes[0];
            cam.eye = s.eye;
            cam.target = s.target;
            cam.up = s.up;
            cam.fov_y_deg = s.fov_y_deg;
            cam.ortho = s.ortho;
            self.camera = cam;
        } else {
            self.camera = nav::zoom_extents_camera(&self.doc.model, &cam, self.aspect());
        }
        self.cam_anim = None;
        self.scene_player = None;
        self.tray.selected_scene = None;
    }

    pub(crate) fn open_path(&mut self, path: &Path) {
        match guarded(|| io::load(path)) {
            Ok(Ok(model)) => {
                self.replace_document(Document::from_model(model, Some(path.to_path_buf())));
                self.add_recent(path);
                self.flash(format!("Opened {}", path.display()));
                if self.autosave.allowed
                    && let Some(r) = crate::autosave::find_recovery(Some(path), None)
                    && !self.autosave.recoveries.iter().any(|x| x.autosave == r.autosave)
                {
                    self.autosave.recoveries.insert(0, r);
                }
            }
            Ok(Err(e)) => self.flash(format!("Could not open {}: {e}", path.display())),
            Err(e) => self.flash(format!("Could not open {}: {e}", path.display())),
        }
    }

    fn add_recent(&mut self, path: &Path) {
        self.prefs.recent.retain(|p| p != path);
        self.prefs.recent.insert(0, path.to_path_buf());
        self.prefs.recent.truncate(MAX_RECENT);
    }

    /// Returns true if saved.
    pub(crate) fn save(&mut self) -> bool {
        match self.doc.path.clone() {
            Some(p) => self.save_to(&p),
            None => self.save_as(),
        }
    }

    fn save_as(&mut self) -> bool {
        let Some(mut path) = rfd::FileDialog::new()
            .add_filter("Varstotas model", &["vrst"])
            .set_file_name(format!("{}.vrst", self.doc.display_name()))
            .save_file()
        else {
            return false;
        };
        if path.extension().is_none() {
            path.set_extension("vrst");
        }
        self.save_to(&path)
    }

    pub(crate) fn save_to(&mut self, path: &Path) -> bool {
        match guarded(|| io::save(&self.doc.model, path)) {
            Ok(Ok(())) => {
                self.doc.path = Some(path.to_path_buf());
                self.doc.dirty = false;
                // Clean save: drop the autosave we wrote (untitled or old name) and any stale
                // autosave next to the new file.
                if let Some(p) = self.autosave.written.take() {
                    self.autosave.worker.delete(p);
                }
                if let Some(p) = crate::autosave::path_for(Some(path), None) {
                    self.autosave.worker.delete(p);
                }
                self.autosave.recoveries.retain(|r| r.original.as_deref() != Some(path));
                self.autosave.version = self.doc.model_version;
                self.autosave.last = self.now;
                self.add_recent(path);
                self.flash(format!("Saved {}", path.display()));
                true
            }
            Ok(Err(e)) => {
                self.flash(format!("Save failed: {e}"));
                false
            }
            Err(e) => {
                self.flash(format!("Save failed: {e}"));
                false
            }
        }
    }

    fn export(&mut self, ext: &str) {
        let Some(mut path) = rfd::FileDialog::new()
            .add_filter(ext.to_uppercase(), &[ext])
            .set_file_name(format!("{}.{ext}", self.doc.display_name()))
            .save_file()
        else {
            return;
        };
        if path.extension().is_none() {
            path.set_extension(ext);
        }
        self.export_to(&path);
    }

    /// Export the model to `path`; the format follows the extension.
    pub(crate) fn export_to(&mut self, path: &std::path::Path) {
        let model = &self.doc.model;
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or_default().to_ascii_lowercase();
        let r = guarded(|| match ext.as_str() {
            "obj" => io::export_obj(model, path),
            "dxf" => io::export_dxf(model, path),
            "glb" => io::export_glb(model, path),
            "3mf" => io::export_3mf(model, path),
            "ply" => io::export_ply(model, path),
            "json" => io::export_faces_json(model, path),
            "csv" => std::fs::write(path, varstotas_kernel::report::report_csv(model)).map_err(Into::into),
            "txt" => {
                let lines: Vec<String> = model.attributions().iter().map(|a| a.line()).collect();
                std::fs::write(path, format!("Credits of the 3D models in {}\n\n{}\n", self.doc.display_name(), lines.join("\n"))).map_err(Into::into)
            }
            "svg" | "pdf" => {
                let vp = self.viewport.size();
                let d = crate::vector_export::build(model, &self.camera, varstotas_kernel::glam::DVec2::new(vp.x as f64, vp.y as f64), &self.vector_opts);
                self.last_vector = Some((d.lines.len(), d.texts.len()));
                let bytes = if ext == "svg" { crate::vector_export::to_svg(&d).into_bytes() } else { crate::vector_export::to_pdf(&d) };
                std::fs::write(path, bytes).map_err(Into::into)
            }
            _ => io::export_stl(model, path),
        });
        match r {
            Ok(Ok(())) => match crate::downloads::licence_warning(&self.doc.model.attributions()) {
                Some(w) if ext != "txt" => self.flash_error(format!("Exported {}. {w}", path.display())),
                _ => self.flash(format!("Exported {}", path.display())),
            },
            Ok(Err(e)) => self.flash(format!("Export failed: {e}")),
            Err(e) => self.flash(format!("Export failed: {e}")),
        }
    }

    fn import(&mut self, exts: &[&str], label: &str) {
        let Some(path) = rfd::FileDialog::new().add_filter(label, exts).pick_file() else {
            return;
        };
        self.import_from(&path, label);
    }

    /// Import `path` (format by extension) as one undo step "Import <label>".
    pub(crate) fn import_from(&mut self, path: &std::path::Path, label: &str) {
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or_default().to_ascii_lowercase();
        let r = self.doc.try_commit(&format!("Import {label}"), |m| match ext.as_str() {
            "obj" => io::import_obj(m, path),
            "dxf" => io::import_dxf(m, path),
            "glb" | "gltf" => io::import_gltf(m, path),
            "3mf" => io::import_3mf(m, path),
            "dae" => io::import_dae(m, path),
            "fbx" => io::import_fbx(m, path),
            "ply" => io::import_ply(m, path),
            "csv" => io::import_points_csv(m, path).map(|_| ()),
            _ => io::import_stl(m, path),
        });
        if r.is_some() {
            self.flash(format!("Imported {}", path.display()));
            self.actions.push(Action::ZoomExtents);
        } else if let Some(e) = self.doc.last_error.clone() {
            self.flash_error(e);
        }
    }

    fn insert_demo(&mut self) {
        fn make_box(mesh: &mut varstotas_kernel::Mesh, x: f64, y: f64, w: f64, d: f64, h: f64) {
            let pts = [
                DVec3::new(x, y, 0.0),
                DVec3::new(x + w, y, 0.0),
                DVec3::new(x + w, y + d, 0.0),
                DVec3::new(x, y + d, 0.0),
            ];
            if let Some(f) = mesh.add_face(&pts) {
                let up = mesh.faces.get(f).map(|f| f.normal.z >= 0.0).unwrap_or(true);
                let _ = mesh.push_pull(f, if up { h } else { -h }, false);
            }
        }
        let r = self.doc.commit("Insert Demo Geometry", |m: &mut Model| {
            let brick = m.add_material("Brick", [0.70, 0.33, 0.24, 1.0]);
            let wood = m.add_material("Wood", [0.78, 0.60, 0.40, 1.0]);
            let mesh = m.root_mesh_mut();
            make_box(mesh, 0.0, 0.0, 3000.0, 2000.0, 2400.0);
            make_box(mesh, 4000.0, 0.0, 1000.0, 1000.0, 3200.0);
            // A painted group.
            let before: std::collections::HashSet<EntityRef> = all_refs(m.root_mesh());
            make_box(m.root_mesh_mut(), 0.0, 3000.0, 1800.0, 900.0, 750.0);
            let new: Vec<EntityRef> = all_refs(m.root_mesh()).difference(&before).copied().collect();
            if let Ok(g) = m.make_group(&[], &new)
                && let Some(i) = m.root_mesh_mut().instances.get_mut(g) {
                    i.name = "Table".into();
                    i.material = Some(wood);
                }
            // Paint the tall box's faces.
            let mesh = m.root_mesh_mut();
            let ids: Vec<_> = mesh.faces.iter().filter(|(_, f)| {
                f.outer.iter().all(|v| mesh.vertices[*v].pos.x >= 4000.0 - 1e-6)
            }).map(|(id, _)| id).collect();
            for id in ids {
                if let Some(f) = mesh.faces.get_mut(id) {
                    f.front = Some(brick);
                }
            }
        });
        if r.is_some() {
            self.actions.push(Action::ZoomExtents);
        } else if let Some(e) = self.doc.last_error.clone() {
            self.flash_error(e);
        }
    }

    // ---- autosave ---------------------------------------------------------------

    /// Write an autosave now (if the document has changes since the last one).
    pub(crate) fn autosave_now(&mut self) {
        if !self.autosave.allowed {
            return;
        }
        let Some(path) = crate::autosave::path_for(self.doc.path.as_deref(), self.autosave.dir.as_deref()) else {
            return;
        };
        if let Some(old) = self.autosave.written.take()
            && old != path
        {
            self.autosave.worker.delete(old);
        }
        self.autosave.worker.write(path.clone(), self.doc.model.clone());
        self.autosave.written = Some(path);
        self.autosave.version = self.doc.model_version;
        self.autosave.last = self.now;
    }

    /// Per-frame autosave timer.
    fn autosave_tick(&mut self, ctx: &egui::Context) {
        for r in self.autosave.worker.poll() {
            if let Err(e) = r.result {
                self.flash_error(format!("Autosave to {} failed: {e}", r.path.display()));
            }
        }
        if !self.autosave.allowed || !self.prefs.autosave {
            return;
        }
        let minutes = self.prefs.autosave_minutes.max(1);
        if !self.doc.dirty || self.doc.model_version == self.autosave.version {
            // Nothing new to protect: the interval starts at the first unsaved change.
            self.autosave.last = self.now;
            return;
        }
        // Don't hitch an in-progress drag; try again on the next frame.
        if !self.tool_pressed
            && crate::autosave::is_due(
                minutes,
                self.now,
                self.autosave.last,
                self.doc.dirty,
                self.doc.model_version,
                self.autosave.version,
            )
        {
            self.autosave_now();
        }
        let wait = crate::autosave::seconds_until_due(minutes, self.now, self.autosave.last);
        ctx.request_repaint_after(std::time::Duration::from_secs_f64(wait.max(0.5)));
    }

    /// Apply the user's choice for the first pending recovery.
    pub(crate) fn resolve_recovery(&mut self, recover: Option<bool>) {
        if self.autosave.recoveries.is_empty() {
            return;
        }
        let r = self.autosave.recoveries.remove(0);
        match recover {
            Some(true) => match guarded(|| io::load(&r.autosave)) {
                Ok(Ok(model)) => {
                    let mut doc = Document::from_model(model, r.original.clone());
                    doc.dirty = true;
                    doc.model_version = crate::document::next_model_version();
                    self.replace_document(doc);
                    // Keep the autosave until the recovered model is saved (or discarded).
                    self.autosave.written = Some(r.autosave.clone());
                    self.autosave.version = 0;
                    self.flash(format!("Recovered {} from autosave", r.display_name()));
                }
                Ok(Err(e)) => self.flash_error(format!("Could not recover {}: {e}", r.display_name())),
                Err(e) => self.flash_error(format!("Could not recover {}: {e}", r.display_name())),
            },
            Some(false) => self.autosave.worker.delete(r.autosave),
            None => {}
        }
    }

    fn update_title(&mut self, ctx: &egui::Context) {
        let title = format!("{}{} — Varstotas", self.doc.display_name(), if self.doc.dirty { "*" } else { "" });
        if title != self.last_title {
            ctx.send_viewport_cmd(ViewportCommand::Title(title.clone()));
            self.last_title = title;
        }
    }

    fn handle_close_request(&mut self, ctx: &egui::Context) {
        if self.tray.close_requested {
            self.tray.close_requested = false;
            ctx.send_viewport_cmd(ViewportCommand::Close);
        }
        if ctx.input(|i| i.viewport().close_requested()) && self.doc.dirty && !self.allow_close && !self.debug.force_close {
            ctx.send_viewport_cmd(ViewportCommand::CancelClose);
            self.confirm = Some(Pending::Quit);
        }
    }

    /// Collect overlays from the active tool.
    /// classic behaviour: a plain Select-tool click outside the open group's bounding box
    /// closes it (repeatedly, for nested groups whose boxes the click also misses).
    pub(crate) fn after_primary_click(&mut self, pos: Pos2, modifiers: Modifiers) {
        if self.tool_id() != ToolId::Select
            || self.doc.active_path.is_empty()
            || crate::tools::select::sel_op(&modifiers) != crate::tools::select::SelOp::Replace
        {
            return;
        }
        let ray = self.pointer_info(pos, egui::Vec2::ZERO, None, false).ray;
        let model = &self.doc.model;
        let mut path = self.doc.active_path.clone();
        while !path.is_empty() {
            let inside = guarded(|| {
                let Some(b) = model.def_at(&path).and_then(|d| model.def_bounds(d)) else {
                    return false;
                };
                let inv = model.transform_at(&path).inverse();
                let o = inv.transform_point3(ray.origin);
                let d = inv.transform_vector3(ray.dir);
                let eps = ((b.max - b.min).length() * 1e-3).max(1e-3);
                b.inflate(eps).ray_hit(o, d).is_some()
            })
            .unwrap_or(false);
            if inside {
                break;
            }
            path.pop();
        }
        if path != self.doc.active_path {
            self.doc.set_active_path(path);
        }
    }

    /// World-space edges of the open group's (oriented) bounding box.
    fn context_box_lines(&self, out: &mut Overlay3d) {
        let path = &self.doc.active_path;
        if path.is_empty() {
            return;
        }
        let model = &self.doc.model;
        let Ok(Some((b, xf))) =
            guarded(|| model.def_at(path).and_then(|d| model.def_bounds(d)).map(|b| (b, model.transform_at(path))))
        else {
            return;
        };
        let c = b.corners().map(|p| xf.transform_point3(p).as_vec3().to_array());
        // Corner index bits: 1 = x, 2 = y, 4 = z.
        for i in 0..8usize {
            for bit in [1usize, 2, 4] {
                if i & bit == 0 {
                    out.lines.push(varstotas_render::OverlayLine {
                        a: c[i],
                        b: c[i | bit],
                        color: [0.25, 0.25, 0.25, 0.9],
                        width_px: 1.0,
                        dashed: true,
                        on_top: true,
                    });
                }
            }
        }
    }

    /// Cut lines (and fill) of the active section plane, cached by (model version, plane).
    pub(crate) fn section_cut_overlay(&self, out: &mut Overlay3d) {
        let model = &self.doc.model;
        let Some(s) = model.active_section() else { return };
        let key = (self.doc.model_version, s.plane().map(f64::to_bits), model.section_display.fill);
        let mut cache = self.section_cut.borrow_mut();
        if cache.as_ref().is_none_or(|c| c.0 != key) {
            let (lines, fill) = guarded(|| {
                let lines = model.section_cut(s);
                let fill = if key.2 { varstotas_kernel::section::section_fill(s, &lines) } else { Vec::new() };
                (lines, fill)
            })
            .unwrap_or_default();
            *cache = Some((key, lines, fill));
        }
        if let Some((_, lines, fill)) = cache.as_ref() {
            tools::section::draw_cut(lines, fill, out);
        }
    }

    /// Oriented bounding boxes of the selected groups / component instances (their contents
    /// are not tinted). Cached by (model, selection) version.
    fn selection_boxes(&self, out: &mut Overlay3d) {
        let key = (self.doc.model_version, self.doc.selection_version);
        let mut cache = self.sel_boxes.borrow_mut();
        if cache.as_ref().is_none_or(|(k, _)| *k != key) {
            let model = &self.doc.model;
            let mut boxes = Vec::new();
            for e in &self.doc.selection {
                let EntityRef::Instance(i) = e else { continue };
                let mut path = self.doc.active_path.clone();
                path.push(*i);
                if let Ok(Some((b, xf))) = guarded(|| {
                    model.def_at(&path).and_then(|d| model.def_bounds(d)).map(|b| (b, model.transform_at(&path)))
                }) {
                    boxes.push(b.corners().map(|p| xf.transform_point3(p).as_vec3().to_array()));
                }
            }
            *cache = Some((key, boxes));
        }
        if let Some((_, boxes)) = cache.as_ref() {
            for c in boxes {
                out.add_box(*c, varstotas_render::HIGHLIGHT_SELECTED, 1.0);
            }
        }
    }

    fn tool_overlays(&self) -> (Overlay3d, Vec<Marker2d>) {
        let mut o3 = Overlay3d::default();
        let mut o2 = Vec::new();
        let view = OverlayView {
            doc: &self.doc,
            camera: &self.camera,
            viewport: self.viewport.size(),
            pointer: self.last_pointer,
        };
        tools::common::draw_guides(&view, &mut o3);
        tools::common::draw_dims(&view, &mut o3);
        self.context_box_lines(&mut o3);
        self.selection_boxes(&mut o3);
        tools::section::draw_sections(&view, &mut o3);
        tools::workplane::draw_work_planes(&view, &mut o3);
        self.section_cut_overlay(&mut o3);
        tools::light::draw_lights(&view, &mut o3);
        self.tool.overlay(&view, &mut o3, &mut o2);
        o2.extend(self.tool.cursor_glyph());
        (o3, o2)
    }

    /// Status-bar text for the left side.
    /// Tool hint (always shown) and an optional unexpired transient message (text, is_error).
    pub(crate) fn status_text(&self) -> (String, Option<(String, bool)>) {
        let flash = self
            .flash
            .as_ref()
            .filter(|f| f.1 > self.now)
            .map(|f| (f.0.clone(), f.2));
        (self.tool.status_text(), flash)
    }

}

fn all_refs(mesh: &varstotas_kernel::Mesh) -> std::collections::HashSet<EntityRef> {
    let mut s = std::collections::HashSet::new();
    s.extend(mesh.faces.keys().map(EntityRef::Face));
    s.extend(mesh.edges.keys().map(EntityRef::Edge));
    s
}

impl eframe::App for VarstotasApp {
    /// Scripted (test) runs start from the same UI state every time (tray scroll, open cards).
    fn persist_egui_memory(&self) -> bool {
        !self.debug.script_active()
    }

    fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        if self.debug.script_active() {
            pace_script_frame();
        }
        let frame_start = std::time::Instant::now();
        let ctx = ui.ctx().clone();
        self.now = ctx.input(|i| i.time);
        let dt = ctx.input(|i| i.stable_dt).min(0.1);

        crate::axes::sync(&self.doc.model);
        crate::inference::settings::publish(&self.prefs.snapping);
        crate::inference::settings::publish_grid(&self.prefs.grid);
        self.handle_close_request(&ctx);
        self.handle_keyboard(&ctx);
        crate::axes::sync(&self.doc.model);

        self.anim_tick(&ctx);
        egui::Panel::top("menu_bar")
            .frame(crate::ui::theme::bar_frame(&ctx))
            .show(ui, |ui| self.menu_bar(ui));
        egui::Panel::top("toolbar")
            .frame(crate::ui::theme::toolbar_frame(&ctx))
            .show(ui, |ui| self.toolbar(ui));
        // The Snapping menus may have changed the switches before the viewport runs the tool.
        crate::inference::settings::publish(&self.prefs.snapping);
        crate::inference::settings::publish_grid(&self.prefs.grid);
        egui::Panel::bottom("status_bar")
            .frame(crate::ui::theme::bar_frame(&ctx))
            .show(ui, |ui| self.status_bar(ui));
        let mut tray_open = self.prefs.tray_open;
        egui::Panel::right("default_tray")
            .resizable(true)
            .default_size(290.0)
            .size_range(220.0..=480.0)
            .frame(crate::ui::theme::tray_frame(&ctx))
            .show_collapsible(ui, &mut tray_open, |ui| self.tray_ui(ui));
        self.prefs.tray_open = tray_open;
        if !self.doc.model.scenes.is_empty() {
            egui::Panel::top("scene_tabs")
                .frame(crate::ui::theme::bar_frame(&ctx))
                .show(ui, |ui| self.scene_tabs(ui));
        }

        egui::CentralPanel::no_frame().show(ui, |ui| {
            self.viewport.view_rect = ui.available_rect_before_wrap();
            let rects = crate::panes::pane_rects(self.viewport.view_rect, self.pane_count);
            self.hud_open = None;
            self.init_panes(&rects);
            self.click_activate(ui, &rects);
            let active = self.active_pane;
            let response = self.viewport.allocate(ui, rects[active]);
            self.handle_viewport_input(&ctx, &response);
            self.tray_drag_frame(&ctx, &response);
            if self.tool.animating() {
                // Scripts get a fixed step so runs are repeatable.
                let step = if self.debug.script_active() { 1.0 / 60.0 } else { dt as f64 };
                let p = self.current_pointer();
                self.with_tool(p, |t, c| t.on_frame(c, step));
                ctx.request_repaint();
            }
            response.context_menu(|ui| self.viewport_context_menu(ui));
            if let Some(anim) = self.cam_anim.as_mut() {
                if !anim.step(dt, &mut self.camera) {
                    self.cam_anim = None;
                }
                ctx.request_repaint();
            }
            if self.rig_anim.is_some() {
                self.step_rig_anim(dt as f64);
                ctx.request_repaint();
            }
            if self.scene_player.is_some() {
                self.step_player(dt as f64);
                ctx.request_repaint();
            }
            let (o3, o2) = self.tool_overlays();
            let cursor = response.hover_pos().or_else(|| {
                let p = self.last_pointer.filter(|_| self.debug.script_active())?;
                Some(self.viewport.rect.min + p.to_vec2())
            });
            for (k, &rect) in rects.iter().enumerate() {
                let is_active = k == active;
                if !is_active {
                    // its own settings in, for the frame
                    self.exchange(k);
                    if let Some(anim) = self.cam_anim.as_mut() {
                        if !anim.step(dt, &mut self.camera) {
                            self.cam_anim = None;
                        }
                        ctx.request_repaint();
                    }
                }
                let style = self.render_style();
                if style.face_style == varstotas_render::FaceStyle::Realistic && crate::pbr::animated(&self.doc.model) {
                    ctx.request_repaint();
                }
                self.viewport.show_textures = matches!(self.prefs.face_style, FaceStylePref::Shaded | FaceStylePref::Realistic);
                // the tool's markers and the cursor belong to the active pane; its 3D preview shows everywhere
                let (markers, cur) = if is_active { (&o2[..], cursor) } else { (&[][..], None) };
                self.viewport.paint(ui, frame, k, rect, is_active, &self.doc, &self.camera, &style, &o3, markers, cur);
                ui.push_id(("pane", k), |ui| self.view_hud(ui, rect));
                if !is_active {
                    self.exchange(k);
                }
            }
            if rects.len() > 1 {
                // the active pane: the one the settings and the tools apply to
                // the soft selection colour, a little stronger
                use crate::ui::theme::{ACCENT, ACCENT_SOFT};
                let frame = ACCENT_SOFT.lerp_to_gamma(ACCENT, 0.4);
                ui.painter().rect_stroke(rects[active], 0.0, egui::Stroke::new(2.5, frame), egui::StrokeKind::Inside);
            }
        });
        // Animation frames collect their own results (anim_tick).
        let export_result = if self.anim_window.run.is_some() { None } else { self.viewport.export_result.take() };
        match export_result {
            Some(r) if self.bridge_shot_done(&r) => {}
            Some(r) if self.preview_shot_done(&r) => {}
            Some(Ok(p)) => {
                log::info!("exported {}", p.display());
                match crate::downloads::licence_warning(&self.doc.model.attributions()) {
                    Some(w) => self.flash_error(format!("Exported {}. {w}", p.display())),
                    None => self.flash(format!("Exported {}", p.display())),
                }
            }
            Some(Err(e)) => {
                log::warn!("PNG export failed: {e}");
                self.flash_error(format!("Export failed: {e}"));
            }
            None => {}
        }

        self.dialogs(&ctx);
        self.console_window(&ctx);
        self.app_help_window(&ctx);
        self.script_help_window(&ctx);
        self.log_window(&ctx);
        self.gen_editor_window(&ctx);
        self.model_info_window(&ctx);
        self.preferences_window(&ctx);
        self.doc_preview_window(&ctx);
        self.rig_editor_window(&ctx);
        self.shader_editor_window(&ctx);
        self.plugin_dialog(&ctx);
        self.unfold_window(&ctx);
        self.sheets_window(&ctx);
        self.work_planes_window(&ctx);
        self.render_window(&ctx);
        self.animation_window(&ctx);
        self.online_window(&ctx);
        self.models_window(&ctx);
        self.downloads_tick(&ctx);
        // Not while a harness screenshot is pending: rendering a thumbnail then loses the capture.
        if !self.debug.screenshot_pending() {
            self.thumbs.process(&ctx, frame);
        }
        self.bridge_tick(&ctx);
        self.run_actions(&ctx);
        self.autosave_tick(&ctx);
        self.update_title(&ctx);
        if self.debug.update(&ctx) {
            self.actions.push(Action::SelectAll);
        }
        self.script_step(&ctx);
        if let Some(e) = self.doc.last_error.take() {
            self.flash_error(e);
        }
        if self.flash.as_ref().is_some_and(|f| f.1 > self.now) {
            ctx.request_repaint_after(std::time::Duration::from_millis(500));
        } else if self.vcb_error {
            // The red Measurements border expires together with the error message.
            self.vcb_error = false;
        }
        if let Some(w) = self.debug.perf.as_mut()
            && let Some(line) = w.record(frame_start.elapsed().as_secs_f64() * 1e3, self.viewport.rebuild_count)
        {
            log::info!("{line}");
            self.debug.perf = None;
        }
    }

    fn raw_input_hook(&mut self, _ctx: &egui::Context, raw_input: &mut egui::RawInput) {
        self.debug.raw_input_hook(raw_input);
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        // Scripted (test) runs must not leak into the user's preferences / recent files.
        if self.debug.script_active() {
            return;
        }
        eframe::set_value(storage, PREFS_KEY, &self.prefs);
    }

    fn on_exit(&mut self) {
        self.stop_bridge();
        // A normal close of a saved/discarded document removes its autosave; a dirty document
        // (forced close) keeps it for recovery.
        if !self.doc.dirty
            && let Some(p) = self.autosave.written.take()
        {
            self.autosave.worker.delete(p);
        }
        self.autosave.worker.flush();
    }
}

/// Scripted runs draw without vsync (see `main`), so they keep the frame rate a display
/// would give: double clicks, hover times and animations then see the usual timing.
fn pace_script_frame() {
    use std::cell::Cell;
    use std::time::{Duration, Instant};
    thread_local! {
        static LAST: Cell<Option<Instant>> = const { Cell::new(None) };
    }
    const FRAME: Duration = Duration::from_micros(16_667);
    LAST.with(|l| {
        if let Some(t) = l.get() {
            let spent = t.elapsed();
            if spent < FRAME {
                std::thread::sleep(FRAME - spent);
            }
        }
        l.set(Some(Instant::now()));
    });
}
