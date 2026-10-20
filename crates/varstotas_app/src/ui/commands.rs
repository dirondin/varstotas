//! Commands of the File, Edit, Tools, Snapping and Window menus that are not tools: their names,
//! icons, shortcuts and when they apply. The menus draw their rows from here, and any of them
//! can be put on the toolbar (Preferences → Toolbar).

use super::icons::Icon;
use crate::app::{Action, VarstotasApp};
use crate::inference::InferenceKind;
use crate::keymap::{KeyCmd, label};
use varstotas_kernel::{EntityRef, IntersectScope, SolidOp};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cmd {
    New,
    Open,
    Save,
    SaveAs,
    PreviewShow,
    PreviewFromFile,
    PreviewFromView,
    PreviewRemove,
    ImportObj,
    ImportStl,
    ImportDxf,
    ImportGltf,
    Import3mf,
    ImportDae,
    ImportFbx,
    ImportPly,
    ImportCsv,
    ExportImage,
    ExportAnimation,
    ExportObj,
    ExportStl,
    ExportDxf,
    ExportGlb,
    Export3mf,
    ExportPly,
    ExportDrawing,
    Sheets,
    ExportReport,
    ExportFacesJson,
    ExportAttributions,
    RunScript,
    Quit,
    Undo,
    Redo,
    Cut,
    Copy,
    Paste,
    PasteInPlace,
    Delete,
    PurgeUnused,
    DeleteGuides,
    SelectAll,
    SelectNone,
    Hide,
    UnhideAll,
    MakeGroup,
    MakeComponent,
    MakeUnique,
    MakeParametric,
    Explode,
    CloseGroup,
    IntersectModel,
    IntersectSelection,
    IntersectContext,
    ReverseFaces,
    OrientFaces,
    FlipRed,
    FlipGreen,
    FlipBlue,
    Solid(SolidOp),
    Unfold,
    Text3d,
    ZoomExtents,
    /// The master switch of snapping.
    Snapping,
    ShowGrid,
    /// One inference on / off.
    Snap(InferenceKind),
    /// The side bar (tray) shown / hidden.
    SideBar,
    /// A card of the side bar shown / hidden (`tray::TRAY_CARDS`).
    Card(&'static str),
    Render,
    WorkPlanes,
    ModelInfo,
    ComponentParams,
    Library3d,
    GeneratorEditor,
    Console,
    Log,
    Preferences,
}

impl Cmd {
    /// Every command, in the menus' order.
    pub fn all() -> Vec<Cmd> {
        use Cmd::*;
        let mut v = vec![
            New, Open, Save, SaveAs, PreviewShow, PreviewFromFile, PreviewFromView, PreviewRemove, ImportObj, ImportStl, ImportDxf,
            ImportGltf, Import3mf, ImportDae, ImportFbx, ImportPly, ImportCsv, ExportImage, ExportAnimation, ExportObj, ExportStl,
            ExportDxf, ExportGlb, Export3mf, ExportPly, ExportDrawing, Sheets, ExportReport, ExportFacesJson, ExportAttributions,
            RunScript, Quit, Undo, Redo, Cut, Copy, Paste, PasteInPlace, Delete, PurgeUnused, DeleteGuides, SelectAll, SelectNone,
            Hide, UnhideAll, MakeGroup, MakeComponent, MakeUnique, MakeParametric, Explode, CloseGroup, IntersectModel, IntersectSelection,
            IntersectContext, ReverseFaces, OrientFaces, FlipRed, FlipGreen, FlipBlue,
        ];
        v.extend(SolidOp::ALL.map(Solid));
        v.extend([Unfold, Text3d, ZoomExtents, Snapping, ShowGrid]);
        v.extend(crate::inference::settings::all_kinds().map(Snap));
        v.push(SideBar);
        v.extend(crate::ui::tray::TRAY_CARDS.map(Card));
        v.extend([Render, WorkPlanes, ModelInfo, ComponentParams, Library3d, GeneratorEditor, Console, Log, Preferences]);
        v
    }

    /// Its menu (the headings of Preferences → Toolbar's reserve).
    pub fn group(self) -> &'static str {
        let all = Cmd::all();
        let at = |c: Cmd| all.iter().position(|x| *x == c).unwrap_or(0);
        let i = at(self);
        if i < at(Cmd::Undo) {
            "File"
        } else if i < at(Cmd::Solid(SolidOp::ALL[0])) {
            "Edit"
        } else if i < at(Cmd::Snapping) {
            "Tools"
        } else if i < at(Cmd::SideBar) {
            "Snapping"
        } else {
            "Window"
        }
    }

    /// The id kept in the preferences (the toolbar's layout).
    pub fn key(self) -> String {
        match self {
            Cmd::Snap(k) => format!("Snap:{}", k.key()),
            Cmd::Solid(op) => format!("Solid:{op:?}"),
            // "Snapping" is the toolbar's magnet (the menu)
            Cmd::Snapping => "SnappingOnOff".into(),
            Cmd::Card(n) => format!("Card:{n}"),
            _ => format!("{self:?}"),
        }
    }

    pub fn from_key(k: &str) -> Option<Cmd> {
        Cmd::all().into_iter().find(|c| c.key() == k)
    }

    /// The name on its own (toolbar tooltips, Preferences → Toolbar).
    pub fn name(self) -> String {
        match self {
            Cmd::ImportObj | Cmd::ImportStl | Cmd::ImportDxf | Cmd::ImportGltf | Cmd::Import3mf | Cmd::ImportDae | Cmd::ImportFbx | Cmd::ImportPly | Cmd::ImportCsv => {
                format!("Import {}", self.menu_label().trim_end_matches('…'))
            }
            Cmd::ExportImage
            | Cmd::ExportAnimation
            | Cmd::ExportObj
            | Cmd::ExportStl
            | Cmd::ExportDxf
            | Cmd::ExportGlb
            | Cmd::Export3mf
            | Cmd::ExportPly
            | Cmd::ExportDrawing
            | Cmd::ExportReport
            | Cmd::ExportFacesJson
            | Cmd::ExportAttributions => format!("Export {}", self.menu_label().trim_end_matches('…')),
            Cmd::PreviewShow => "Preview Image: Show".into(),
            Cmd::PreviewFromFile => "Preview Image from File".into(),
            Cmd::PreviewFromView => "Preview Image from View".into(),
            Cmd::PreviewRemove => "Remove Preview Image".into(),
            Cmd::IntersectModel => "Intersect with Model".into(),
            Cmd::IntersectSelection => "Intersect with Selection".into(),
            Cmd::IntersectContext => "Intersect with Context".into(),
            Cmd::FlipRed => "Flip Along Red".into(),
            Cmd::FlipGreen => "Flip Along Green".into(),
            Cmd::FlipBlue => "Flip Along Blue".into(),
            Cmd::Solid(op) => format!("Solid {}", op.name()),
            Cmd::Snap(k) => format!("Snap: {}", k.menu_label()),
            Cmd::Card(n) => format!("Side Bar: {n}"),
            _ => self.menu_label().trim_end_matches('…').into(),
        }
    }

    /// The row in its menu (a submenu's rows leave out what the submenu says).
    pub fn menu_label(self) -> &'static str {
        match self {
            Cmd::New => "New",
            Cmd::Open => "Open…",
            Cmd::Save => "Save",
            Cmd::SaveAs => "Save As…",
            Cmd::PreviewShow => "Show…",
            Cmd::PreviewFromFile => "From Image File…",
            Cmd::PreviewFromView => "From Current View",
            Cmd::PreviewRemove => "Remove",
            Cmd::ImportObj | Cmd::ExportObj => "OBJ…",
            Cmd::ImportStl | Cmd::ExportStl => "STL…",
            Cmd::ImportDxf | Cmd::ExportDxf => "DXF…",
            Cmd::ImportGltf => "glTF / GLB…",
            Cmd::Import3mf => "3MF…",
            Cmd::ImportDae => "COLLADA (DAE)…",
            Cmd::ImportFbx => "FBX…",
            Cmd::ImportPly | Cmd::ExportPly => "PLY…",
            Cmd::ImportCsv => "Points (CSV)…",
            Cmd::ExportImage => "2D Graphic (PNG)…",
            Cmd::ExportAnimation => "Animation (Video / PNG)…",
            Cmd::ExportGlb => "glTF (GLB)…",
            Cmd::Export3mf => "3MF (3D printing)…",
            Cmd::ExportDrawing => "2D Drawing (SVG / PDF)…",
            Cmd::Sheets => "Sheets (PDF / SVG)…",
            Cmd::ExportReport => "Report: Parts and Areas (CSV)…",
            Cmd::ExportFacesJson => "Faces (JSON)…",
            Cmd::ExportAttributions => "Attributions (credits of downloaded models)…",
            Cmd::RunScript => "Run Script…",
            Cmd::Quit => "Quit",
            Cmd::Undo => "Undo",
            Cmd::Redo => "Redo",
            Cmd::Cut => "Cut",
            Cmd::Copy => "Copy",
            Cmd::Paste => "Paste",
            Cmd::PasteInPlace => "Paste in Place",
            Cmd::Delete => "Delete",
            Cmd::PurgeUnused => "Purge Unused",
            Cmd::DeleteGuides => "Delete Guides",
            Cmd::SelectAll => "Select All",
            Cmd::SelectNone => "Select None",
            Cmd::Hide => "Hide",
            Cmd::UnhideAll => "Unhide All",
            Cmd::MakeGroup => "Make Group",
            Cmd::MakeComponent => "Make Component…",
            Cmd::MakeUnique => "Make Unique",
            Cmd::MakeParametric => "Make Parametric",
            Cmd::Explode => "Explode",
            Cmd::CloseGroup => "Close Group",
            Cmd::IntersectModel => "With Model",
            Cmd::IntersectSelection => "With Selection",
            Cmd::IntersectContext => "With Context",
            Cmd::ReverseFaces => "Reverse Faces",
            Cmd::OrientFaces => "Orient Faces",
            Cmd::FlipRed => "Red Direction",
            Cmd::FlipGreen => "Green Direction",
            Cmd::FlipBlue => "Blue Direction",
            Cmd::Solid(op) => op.name(),
            Cmd::Unfold => "Unfold…",
            Cmd::Text3d => "3D Text…",
            Cmd::ZoomExtents => "Zoom Extents",
            Cmd::Snapping => "Snapping",
            Cmd::ShowGrid => "Show Grid",
            Cmd::Snap(k) => k.menu_label(),
            Cmd::SideBar => "Side Bar",
            Cmd::Card(n) => n,
            Cmd::Render => "Render…",
            Cmd::WorkPlanes => "Work Planes…",
            Cmd::ModelInfo => "Model Info…",
            Cmd::ComponentParams => "Component Parameters…",
            Cmd::Library3d => "3D Library…",
            Cmd::GeneratorEditor => "Generator Editor…",
            Cmd::Console => "Script Console",
            Cmd::Log => "Log",
            Cmd::Preferences => "Preferences…",
        }
    }

    pub fn icon(self) -> Icon {
        match self {
            Cmd::New => Icon::FileNew,
            Cmd::Open => Icon::FileOpen,
            Cmd::Save => Icon::FileSave,
            Cmd::SaveAs => Icon::FileSaveAs,
            Cmd::PreviewShow => Icon::Picture(0),
            Cmd::PreviewFromFile => Icon::Picture(1),
            Cmd::PreviewFromView => Icon::Picture(2),
            Cmd::PreviewRemove => Icon::Picture(3),
            Cmd::ImportObj => Icon::Import("OBJ"),
            Cmd::ImportStl => Icon::Import("STL"),
            Cmd::ImportDxf => Icon::Import("DXF"),
            Cmd::ImportGltf => Icon::Import("GLB"),
            Cmd::Import3mf => Icon::Import("3MF"),
            Cmd::ImportDae => Icon::Import("DAE"),
            Cmd::ImportFbx => Icon::Import("FBX"),
            Cmd::ImportPly => Icon::Import("PLY"),
            Cmd::ImportCsv => Icon::Import("CSV"),
            Cmd::ExportImage => Icon::Export("PNG"),
            Cmd::ExportAnimation => Icon::Film,
            Cmd::ExportObj => Icon::Export("OBJ"),
            Cmd::ExportStl => Icon::Export("STL"),
            Cmd::ExportDxf => Icon::Export("DXF"),
            Cmd::ExportGlb => Icon::Export("GLB"),
            Cmd::Export3mf => Icon::Export("3MF"),
            Cmd::ExportPly => Icon::Export("PLY"),
            Cmd::ExportDrawing => Icon::Drawing,
            Cmd::Sheets => Icon::Sheets,
            Cmd::ExportReport => Icon::Report,
            Cmd::ExportFacesJson => Icon::Json,
            Cmd::ExportAttributions => Icon::Credits,
            Cmd::RunScript => Icon::Script,
            Cmd::Quit => Icon::Quit,
            Cmd::Undo => Icon::Undo,
            Cmd::Redo => Icon::Redo,
            Cmd::Cut => Icon::Cut,
            Cmd::Copy => Icon::Copy,
            Cmd::Paste => Icon::Paste,
            Cmd::PasteInPlace => Icon::PasteInPlace,
            Cmd::Delete => Icon::Trash,
            Cmd::PurgeUnused => Icon::Purge,
            Cmd::DeleteGuides => Icon::DeleteGuides,
            Cmd::SelectAll => Icon::SelectAll,
            Cmd::SelectNone => Icon::SelectNone,
            Cmd::Hide => Icon::EyeOff,
            Cmd::UnhideAll => Icon::Eye,
            Cmd::MakeGroup => Icon::Group,
            Cmd::MakeComponent | Cmd::MakeUnique => Icon::Component,
            Cmd::MakeParametric => Icon::Parametric,
            Cmd::Explode => Icon::Explode,
            Cmd::CloseGroup => Icon::CloseGroup,
            Cmd::IntersectModel => Icon::Intersect(0),
            Cmd::IntersectSelection => Icon::Intersect(1),
            Cmd::IntersectContext => Icon::Intersect(2),
            Cmd::ReverseFaces => Icon::ReverseFaces,
            Cmd::OrientFaces => Icon::OrientFaces,
            Cmd::FlipRed => Icon::Flip(0),
            Cmd::FlipGreen => Icon::Flip(1),
            Cmd::FlipBlue => Icon::Flip(2),
            Cmd::Solid(op) => Icon::Solids(SolidOp::ALL.iter().position(|o| *o == op).unwrap_or(0) as u8),
            Cmd::Unfold => Icon::Unfold,
            Cmd::Text3d => Icon::Text3d,
            Cmd::ZoomExtents => Icon::ZoomExtents,
            Cmd::Snapping => Icon::Magnet,
            Cmd::ShowGrid => Icon::Grid,
            Cmd::Snap(k) => Icon::Snap(k),
            Cmd::SideBar => Icon::SideBar,
            Cmd::Card(n) => match n {
                "Entity Info" => Icon::Info,
                "Materials" => Icon::Paint,
                "Components" => Icon::Component,
                "Tags" => Icon::Tag,
                "Outliner" => Icon::Outliner,
                "Scenes" => Icon::AddScene,
                "Shadows" => Icon::Shadows,
                "Environment" => Icon::StyleRealistic,
                "Soften Edges" => Icon::Arc,
                _ => Icon::Select,
            },
            Cmd::Render => Icon::Render,
            Cmd::WorkPlanes => Icon::GuidePlane,
            Cmd::ModelInfo => Icon::ModelInfo,
            Cmd::ComponentParams => Icon::Parametric,
            Cmd::Library3d => Icon::Cloud,
            Cmd::GeneratorEditor => Icon::Generator,
            Cmd::Console => Icon::Console,
            Cmd::Log => Icon::Log,
            Cmd::Preferences => Icon::Gear,
        }
    }

    pub fn shortcut(self) -> Option<String> {
        let key = match self {
            Cmd::New => KeyCmd::New,
            Cmd::Open => KeyCmd::Open,
            Cmd::Save => KeyCmd::Save,
            Cmd::SaveAs => KeyCmd::SaveAs,
            Cmd::Quit => KeyCmd::Quit,
            Cmd::Undo => KeyCmd::Undo,
            Cmd::Redo => KeyCmd::Redo,
            Cmd::SelectAll => KeyCmd::SelectAll,
            Cmd::SelectNone => KeyCmd::SelectNone,
            Cmd::MakeGroup => KeyCmd::MakeGroup,
            Cmd::ZoomExtents => KeyCmd::ZoomExtents,
            Cmd::Snapping => KeyCmd::ToggleSnapping,
            Cmd::Cut => return Some(cmd_key("X", false)),
            Cmd::Copy => return Some(cmd_key("C", false)),
            Cmd::Paste => return Some(cmd_key("V", false)),
            Cmd::PasteInPlace => return Some(cmd_key("V", true)),
            Cmd::Delete => return Some("Delete".into()),
            Cmd::CloseGroup => return Some("Esc".into()),
            _ => return None,
        };
        label(key)
    }

    /// One line for the toolbar's tooltip.
    pub fn description(self) -> String {
        match self {
            Cmd::New => "Start a new document.".into(),
            Cmd::Open => "Open a Varstotas file.".into(),
            Cmd::Save => "Save the document.".into(),
            Cmd::SaveAs => "Save the document under a new name.".into(),
            Cmd::PreviewShow => "The picture the Components library shows for this file.".into(),
            Cmd::PreviewFromFile => "Use an image file as the document's preview.".into(),
            Cmd::PreviewFromView => "Use the current view as the document's preview.".into(),
            Cmd::PreviewRemove => "Drop the preview (the library draws the model).".into(),
            Cmd::RunScript => "Run a Rhai script file on the model.".into(),
            Cmd::Quit => "Close Varstotas.".into(),
            Cmd::Undo | Cmd::Redo => format!("{} the last change.", self.menu_label()),
            Cmd::Cut => "Copy the selection and erase it.".into(),
            Cmd::Copy => "Copy the selection.".into(),
            Cmd::Paste => "Place the copied geometry with the mouse.".into(),
            Cmd::PasteInPlace => "Paste where the geometry was.".into(),
            Cmd::Delete => "Erase the selection.".into(),
            Cmd::PurgeUnused => "Drop materials and components the model no longer uses.".into(),
            Cmd::DeleteGuides => "Erase every guide line and guide point.".into(),
            Cmd::SelectAll => "Select everything in the active context.".into(),
            Cmd::SelectNone => "Clear the selection.".into(),
            Cmd::Hide => "Hide the selection.".into(),
            Cmd::UnhideAll => "Show everything hidden.".into(),
            Cmd::MakeGroup => "Wrap the selection in a group.".into(),
            Cmd::MakeComponent => "Turn the selection into a component.".into(),
            Cmd::MakeUnique => "Give the selected component its own definition: editing it no longer changes the other copies.".into(),
            Cmd::MakeParametric => "Give the selected group parameters (Component Parameters).".into(),
            Cmd::Explode => "Unpack the selected groups and components.".into(),
            Cmd::CloseGroup => "Leave the group being edited.".into(),
            Cmd::IntersectModel | Cmd::IntersectSelection | Cmd::IntersectContext => {
                format!("Add edges where the selected faces cut {}.", match self {
                    Cmd::IntersectModel => "the model",
                    Cmd::IntersectSelection => "each other",
                    _ => "the group being edited",
                })
            }
            Cmd::ReverseFaces => "Swap the front and back of the selected faces.".into(),
            Cmd::OrientFaces => "Turn the faces next to the selected one the same way.".into(),
            Cmd::FlipRed | Cmd::FlipGreen | Cmd::FlipBlue => "Mirror the selection along this axis.".into(),
            Cmd::Solid(op) => format!("{} two selected solids.", match op {
                SolidOp::OuterShell => "Keep the outer shell of",
                SolidOp::Intersect => "Keep the shared volume of",
                SolidOp::Union => "Join",
                SolidOp::Subtract => "Cut the first out of the second of",
                SolidOp::Trim => "Trim the second by the first of",
                SolidOp::Split => "Split into pieces",
            }),
            Cmd::Unfold => "Lay the selected faces flat (SVG / PDF / DXF).".into(),
            Cmd::Text3d => "Add text as 3D geometry (a group).".into(),
            Cmd::ZoomExtents => "Fit the whole model in the view.".into(),
            Cmd::Snapping => "Snapping on / off.".into(),
            Cmd::ShowGrid => "The grid on the ground on / off.".into(),
            Cmd::Snap(_) => "This snap on / off.".into(),
            Cmd::SideBar => "The side bar with the cards (Entity Info, Materials …) shown or hidden.".into(),
            Cmd::Card(n) => format!("The {n} card of the side bar shown or hidden."),
            Cmd::Render => "Path-traced pictures of the model (Window → Render).".into(),
            Cmd::WorkPlanes => "Saved drawing planes.".into(),
            Cmd::ModelInfo => "Statistics of the document, Purge Unused.".into(),
            Cmd::ComponentParams => "Parameters of the selected component: values that turn, slide, show or paint its parts.".into(),
            Cmd::Library3d => "Free models from online libraries.".into(),
            Cmd::GeneratorEditor => "Write and run generator scripts.".into(),
            Cmd::Console => "Run Rhai commands on the model.".into(),
            Cmd::Log => "Errors and warnings from everywhere: scripts, import / export, libraries, downloads …".into(),
            Cmd::Preferences => "Settings of Varstotas.".into(),
            _ => format!("{}.", self.name()),
        }
    }

    pub fn action(self) -> Action {
        match self {
            Cmd::New => Action::New,
            Cmd::Open => Action::Open,
            Cmd::Save => Action::Save,
            Cmd::SaveAs => Action::SaveAs,
            Cmd::PreviewShow => Action::PreviewShow,
            Cmd::PreviewFromFile => Action::PreviewFromFile,
            Cmd::PreviewFromView => Action::PreviewFromView,
            Cmd::PreviewRemove => Action::PreviewRemove,
            Cmd::ImportObj => Action::ImportObj,
            Cmd::ImportStl => Action::ImportStl,
            Cmd::ImportDxf => Action::ImportDxf,
            Cmd::ImportGltf => Action::ImportGltf,
            Cmd::Import3mf => Action::ImportAs("3mf"),
            Cmd::ImportDae => Action::ImportAs("dae"),
            Cmd::ImportFbx => Action::ImportAs("fbx"),
            Cmd::ImportPly => Action::ImportAs("ply"),
            Cmd::ImportCsv => Action::ImportAs("csv"),
            Cmd::ExportImage => Action::ExportImage,
            Cmd::ExportAnimation => Action::ExportAnimation,
            Cmd::ExportObj => Action::ExportObj,
            Cmd::ExportStl => Action::ExportStl,
            Cmd::ExportDxf => Action::ExportDxf,
            Cmd::ExportGlb => Action::ExportGlb,
            Cmd::Export3mf => Action::Export3mf,
            Cmd::ExportPly => Action::ExportPly,
            Cmd::ExportDrawing => Action::ExportDrawing,
            Cmd::Sheets => Action::Sheets,
            Cmd::ExportReport => Action::ExportReport,
            Cmd::ExportFacesJson => Action::ExportFacesJson,
            Cmd::ExportAttributions => Action::ExportAttributions,
            Cmd::RunScript => Action::RunScript,
            Cmd::Quit => Action::Quit,
            Cmd::Undo => Action::Undo,
            Cmd::Redo => Action::Redo,
            Cmd::Cut => Action::Cut,
            Cmd::Copy => Action::Copy,
            Cmd::Paste => Action::Paste,
            Cmd::PasteInPlace => Action::PasteInPlace,
            Cmd::Delete => Action::Delete,
            Cmd::PurgeUnused => Action::PurgeUnused,
            Cmd::DeleteGuides => Action::DeleteGuides,
            Cmd::SelectAll => Action::SelectAll,
            Cmd::SelectNone => Action::SelectNone,
            Cmd::Hide => Action::HideSelected,
            Cmd::UnhideAll => Action::UnhideAll,
            Cmd::MakeGroup => Action::MakeGroup,
            Cmd::MakeComponent => Action::MakeComponent,
            Cmd::MakeUnique => Action::MakeUnique,
            Cmd::MakeParametric => Action::MakeParametric,
            Cmd::Explode => Action::Explode,
            Cmd::CloseGroup => Action::ExitContext,
            Cmd::IntersectModel => Action::IntersectFaces(IntersectScope::Model),
            Cmd::IntersectSelection => Action::IntersectFaces(IntersectScope::Selection),
            Cmd::IntersectContext => Action::IntersectFaces(IntersectScope::Context),
            Cmd::ReverseFaces => Action::ReverseFaces,
            Cmd::OrientFaces => Action::OrientFaces,
            Cmd::FlipRed => Action::FlipAlong(0),
            Cmd::FlipGreen => Action::FlipAlong(1),
            Cmd::FlipBlue => Action::FlipAlong(2),
            Cmd::Solid(op) => Action::SolidTool(op),
            Cmd::Unfold => Action::Unfold,
            Cmd::Text3d => Action::Text3d,
            Cmd::ZoomExtents => Action::ZoomExtents,
            Cmd::Snapping => Action::ToggleSnapping,
            Cmd::ShowGrid => Action::ToggleGrid,
            Cmd::Snap(k) => Action::ToggleSnap(k),
            Cmd::SideBar => Action::ToggleTray,
            Cmd::Card(n) => Action::ToggleCard(n),
            Cmd::Render => Action::ToggleRender,
            Cmd::WorkPlanes => Action::WorkPlanes,
            Cmd::ModelInfo => Action::ModelInfo,
            Cmd::ComponentParams => Action::RigEditor,
            Cmd::Library3d => Action::ModelLibrary,
            Cmd::GeneratorEditor => Action::GeneratorEditor,
            Cmd::Console => Action::ToggleConsole,
            Cmd::Log => Action::ToggleLog,
            Cmd::Preferences => Action::Preferences,
        }
    }

    /// On / off for the switches (their toolbar button is shown pressed when on).
    pub fn checked(self, app: &VarstotasApp) -> Option<bool> {
        match self {
            Cmd::Snapping => Some(app.prefs.snapping.enabled),
            Cmd::ShowGrid => Some(app.prefs.grid.show),
            Cmd::Snap(k) => Some(app.prefs.snapping.is_on(k)),
            Cmd::SideBar => Some(app.prefs.tray_open),
            Cmd::Card(n) => Some(!app.prefs.tray_hidden.iter().any(|h| h == n)),
            Cmd::Sheets => Some(app.sheets.open),
            Cmd::Render => Some(app.render_window.open),
            Cmd::WorkPlanes => Some(app.planes_window.open),
            Cmd::ModelInfo => Some(app.model_info.open),
            Cmd::ComponentParams => Some(app.rig_editor.open),
            Cmd::Library3d => Some(app.models.open),
            Cmd::GeneratorEditor => Some(app.gen_editor.open),
            Cmd::Console => Some(app.console.open),
            Cmd::Log => Some(app.log_window.open),
            Cmd::Preferences => Some(app.prefs_win.open),
            _ => None,
        }
    }

    /// Whether the command applies now (else its row and button are greyed out).
    pub fn enabled(self, app: &VarstotasApp) -> bool {
        let doc = &app.doc;
        let sel = &doc.selection;
        match self {
            Cmd::PreviewRemove => doc.model.preview.is_some(),
            Cmd::Undo => doc.history.undo_label().is_some(),
            Cmd::Redo => doc.history.redo_label().is_some(),
            Cmd::Cut | Cmd::Copy | Cmd::Delete | Cmd::SelectNone => doc.has_selection(),
            Cmd::Paste | Cmd::PasteInPlace => app.clipboard.is_some(),
            Cmd::DeleteGuides => doc.model.has_any_guides(),
            Cmd::Hide | Cmd::MakeGroup | Cmd::MakeComponent | Cmd::FlipRed | Cmd::FlipGreen | Cmd::FlipBlue => !sel.is_empty(),
            Cmd::MakeUnique => doc.can_make_unique(),
            Cmd::MakeParametric => {
                matches!(sel.as_slice(), [EntityRef::Instance(i)] if doc.model.instance_params(&doc.active_path, *i).is_none())
            }
            Cmd::Explode => sel.iter().any(|e| matches!(e, EntityRef::Instance(_))),
            Cmd::CloseGroup => !doc.active_path.is_empty(),
            Cmd::IntersectModel | Cmd::IntersectSelection | Cmd::IntersectContext => {
                sel.iter().any(|e| matches!(e, EntityRef::Face(_) | EntityRef::Instance(_)))
            }
            Cmd::ReverseFaces | Cmd::OrientFaces => sel.iter().any(|e| matches!(e, EntityRef::Face(_))),
            Cmd::Snap(_) => app.prefs.snapping.enabled,
            _ => true,
        }
    }
}

fn cmd_key(key: &str, shift: bool) -> String {
    match (cfg!(target_os = "macos"), shift) {
        (true, false) => format!("⌘{key}"),
        (true, true) => format!("⇧⌘{key}"),
        (false, false) => format!("Ctrl+{key}"),
        (false, true) => format!("Ctrl+Shift+{key}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keys_are_unique_and_round_trip() {
        let all = Cmd::all();
        for c in &all {
            assert_eq!(Cmd::from_key(&c.key()), Some(*c));
            // no clash with the tools' ids on the toolbar
            assert!(!crate::tools::ToolId::ALL.iter().any(|t| format!("{t:?}") == c.key()), "{c:?}");
            assert!(!["Snapping", "|"].contains(&c.key().as_str()), "{c:?}");
        }
        let mut keys: Vec<String> = all.iter().map(|c| c.key()).collect();
        keys.sort();
        keys.dedup();
        assert_eq!(keys.len(), all.len());
        // the toolbar's older ids still mean the same
        assert_eq!(Cmd::from_key("Text3d"), Some(Cmd::Text3d));
        assert_eq!(Cmd::from_key("ZoomExtents"), Some(Cmd::ZoomExtents));
    }
}
