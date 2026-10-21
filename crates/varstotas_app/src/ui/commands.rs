//! Commands of the File, Edit, Tools, Snapping and Window menus that are not tools: their names,
//! icons, shortcuts and when they apply. The menus draw their rows from here, and any of them
//! can be put on the toolbar (Preferences → Toolbar).

use varstotas_i18n::tr;
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
    ExportDae,
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
            ExportDxf, ExportGlb, Export3mf, ExportDae, ExportPly, ExportDrawing, Sheets, ExportReport, ExportFacesJson, ExportAttributions,
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
            "File" // i18n-ignore
        } else if i < at(Cmd::Solid(SolidOp::ALL[0])) {
            "Edit" // i18n-ignore
        } else if i < at(Cmd::Snapping) {
            "Tools" // i18n-ignore
        } else if i < at(Cmd::SideBar) {
            "Snapping" // i18n-ignore
        } else {
            "Window" // i18n-ignore
        }
    }

    /// The id kept in the preferences (the toolbar's layout).
    pub fn key(self) -> String {
        match self {
            Cmd::Snap(k) => format!("Snap:{}", k.key()), // i18n-ignore
            Cmd::Solid(op) => format!("Solid:{op:?}"), // i18n-ignore
            // "Snapping" is the toolbar's magnet (the menu)
            Cmd::Snapping => "SnappingOnOff".into(), // i18n-ignore
            Cmd::Card(n) => format!("Card:{n}"), // i18n-ignore
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
                tr!("cmd-name-import", name = self.menu_label().trim_end_matches('…'))
            }
            Cmd::ExportImage
            | Cmd::ExportAnimation
            | Cmd::ExportObj
            | Cmd::ExportStl
            | Cmd::ExportDxf
            | Cmd::ExportGlb
            | Cmd::Export3mf
            | Cmd::ExportPly
            | Cmd::ExportDae
            | Cmd::ExportDrawing
            | Cmd::ExportReport
            | Cmd::ExportFacesJson
            | Cmd::ExportAttributions => tr!("cmd-name-export", name = self.menu_label().trim_end_matches('…')),
            Cmd::PreviewShow => tr!("cmd-name-preview-show"),
            Cmd::PreviewFromFile => tr!("cmd-name-preview-from-file"),
            Cmd::PreviewFromView => tr!("cmd-name-preview-from-view"),
            Cmd::PreviewRemove => tr!("cmd-name-preview-remove"),
            Cmd::IntersectModel => tr!("cmd-name-intersect-model"),
            Cmd::IntersectSelection => tr!("cmd-name-intersect-selection"),
            Cmd::IntersectContext => tr!("cmd-name-intersect-context"),
            Cmd::FlipRed => tr!("cmd-name-flip-red"),
            Cmd::FlipGreen => tr!("cmd-name-flip-green"),
            Cmd::FlipBlue => tr!("cmd-name-flip-blue"),
            Cmd::Solid(op) => tr!("cmd-name-solid", name = op.name()),
            Cmd::Snap(k) => tr!("cmd-name-snap", name = k.menu_label()),
            Cmd::Card(n) => tr!("cmd-name-card", card = crate::ui::tray::card_label(n)),
            _ => self.menu_label().trim_end_matches('…').into(),
        }
    }

    /// The row in its menu (a submenu's rows leave out what the submenu says).
    pub fn menu_label(self) -> String {
        varstotas_i18n::tr(match self {
            Cmd::New => "cmd-new",
            Cmd::Open => "cmd-open",
            Cmd::Save => "cmd-save",
            Cmd::SaveAs => "cmd-save-as",
            Cmd::PreviewShow => "cmd-preview-show",
            Cmd::PreviewFromFile => "cmd-preview-from-file",
            Cmd::PreviewFromView => "cmd-preview-from-view",
            Cmd::PreviewRemove => "cmd-preview-remove",
            Cmd::ImportObj | Cmd::ExportObj => "cmd-import-obj",
            Cmd::ImportStl | Cmd::ExportStl => "cmd-import-stl",
            Cmd::ImportDxf | Cmd::ExportDxf => "cmd-import-dxf",
            Cmd::ImportGltf => "cmd-import-gltf",
            Cmd::Import3mf => "cmd-mf",
            Cmd::ImportDae | Cmd::ExportDae => "cmd-import-dae",
            Cmd::ImportFbx => "cmd-import-fbx",
            Cmd::ImportPly | Cmd::ExportPly => "cmd-import-ply",
            Cmd::ImportCsv => "cmd-import-csv",
            Cmd::ExportImage => "cmd-export-image",
            Cmd::ExportAnimation => "cmd-export-animation",
            Cmd::ExportGlb => "cmd-export-glb",
            Cmd::Export3mf => "cmd-mf",
            Cmd::ExportDrawing => "cmd-export-drawing",
            Cmd::Sheets => "cmd-sheets",
            Cmd::ExportReport => "cmd-export-report",
            Cmd::ExportFacesJson => "cmd-export-faces-json",
            Cmd::ExportAttributions => "cmd-export-attributions",
            Cmd::RunScript => "cmd-run-script",
            Cmd::Quit => "cmd-quit",
            Cmd::Undo => "cmd-undo",
            Cmd::Redo => "cmd-redo",
            Cmd::Cut => "cmd-cut",
            Cmd::Copy => "cmd-copy",
            Cmd::Paste => "cmd-paste",
            Cmd::PasteInPlace => "cmd-paste-in-place",
            Cmd::Delete => "cmd-delete",
            Cmd::PurgeUnused => "cmd-purge-unused",
            Cmd::DeleteGuides => "cmd-delete-guides",
            Cmd::SelectAll => "cmd-select-all",
            Cmd::SelectNone => "cmd-select-none",
            Cmd::Hide => "cmd-hide",
            Cmd::UnhideAll => "cmd-unhide-all",
            Cmd::MakeGroup => "cmd-make-group",
            Cmd::MakeComponent => "cmd-make-component",
            Cmd::MakeUnique => "cmd-make-unique",
            Cmd::MakeParametric => "cmd-make-parametric",
            Cmd::Explode => "cmd-explode",
            Cmd::CloseGroup => "cmd-close-group",
            Cmd::IntersectModel => "cmd-intersect-model",
            Cmd::IntersectSelection => "cmd-intersect-selection",
            Cmd::IntersectContext => "cmd-intersect-context",
            Cmd::ReverseFaces => "cmd-reverse-faces",
            Cmd::OrientFaces => "cmd-orient-faces",
            Cmd::FlipRed => "cmd-flip-red",
            Cmd::FlipGreen => "cmd-flip-green",
            Cmd::FlipBlue => "cmd-flip-blue",
            Cmd::Solid(op) => return op.name(),
            Cmd::Unfold => "cmd-unfold",
            Cmd::Text3d => "cmd-d",
            Cmd::ZoomExtents => "cmd-zoom-extents",
            Cmd::Snapping => "cmd-snapping",
            Cmd::ShowGrid => "cmd-show-grid",
            Cmd::Snap(k) => return k.menu_label(),
            Cmd::SideBar => "cmd-side-bar",
            Cmd::Card(n) => return crate::ui::tray::card_label(n),
            Cmd::Render => "cmd-render",
            Cmd::WorkPlanes => "cmd-work-planes",
            Cmd::ModelInfo => "cmd-model-info",
            Cmd::ComponentParams => "cmd-component-params",
            Cmd::Library3d => "cmd-d",
            Cmd::GeneratorEditor => "cmd-generator-editor",
            Cmd::Console => "cmd-console",
            Cmd::Log => "cmd-log",
            Cmd::Preferences => "cmd-preferences",
        })
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
            Cmd::ExportDae => Icon::Export("DAE"),
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
                "Entity Info" => Icon::Info, // i18n-ignore
                "Materials" => Icon::Paint, // i18n-ignore
                "Components" => Icon::Component, // i18n-ignore
                "Tags" => Icon::Tag, // i18n-ignore
                "Outliner" => Icon::Outliner, // i18n-ignore
                "Scenes" => Icon::AddScene, // i18n-ignore
                "Shadows" => Icon::Shadows, // i18n-ignore
                "Environment" => Icon::StyleRealistic, // i18n-ignore
                "Soften Edges" => Icon::Arc, // i18n-ignore
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
            Cmd::Delete => return Some("Delete".into()), // i18n-ignore
            Cmd::CloseGroup => return Some("Esc".into()), // i18n-ignore
            _ => return None,
        };
        label(key)
    }

    /// One line for the toolbar's tooltip.
    pub fn description(self) -> String {
        match self {
            Cmd::New => tr!("cmd-desc-new"),
            Cmd::Open => tr!("cmd-desc-open"),
            Cmd::Save => tr!("cmd-desc-save"),
            Cmd::SaveAs => tr!("cmd-desc-save-as"),
            Cmd::PreviewShow => tr!("cmd-desc-preview-show"),
            Cmd::PreviewFromFile => tr!("cmd-desc-preview-from-file"),
            Cmd::PreviewFromView => tr!("cmd-desc-preview-from-view"),
            Cmd::PreviewRemove => tr!("cmd-desc-preview-remove"),
            Cmd::RunScript => tr!("cmd-desc-run-script"),
            Cmd::Quit => tr!("cmd-desc-quit"),
            Cmd::Undo | Cmd::Redo => tr!("cmd-desc-undo-redo", name = self.menu_label()),
            Cmd::Cut => tr!("cmd-desc-cut"),
            Cmd::Copy => tr!("cmd-desc-copy"),
            Cmd::Paste => tr!("cmd-desc-paste"),
            Cmd::PasteInPlace => tr!("cmd-desc-paste-in-place"),
            Cmd::Delete => tr!("cmd-desc-delete"),
            Cmd::PurgeUnused => tr!("cmd-desc-purge-unused"),
            Cmd::DeleteGuides => tr!("cmd-desc-delete-guides"),
            Cmd::SelectAll => tr!("cmd-desc-select-all"),
            Cmd::SelectNone => tr!("cmd-desc-select-none"),
            Cmd::Hide => tr!("cmd-desc-hide"),
            Cmd::UnhideAll => tr!("cmd-desc-unhide-all"),
            Cmd::MakeGroup => tr!("cmd-desc-make-group"),
            Cmd::MakeComponent => tr!("cmd-desc-make-component"),
            Cmd::MakeUnique => tr!("cmd-desc-make-unique"),
            Cmd::MakeParametric => tr!("cmd-desc-make-parametric"),
            Cmd::Explode => tr!("cmd-desc-explode"),
            Cmd::CloseGroup => tr!("cmd-desc-close-group"),
            Cmd::IntersectModel => tr!("cmd-desc-intersect-model"),
            Cmd::IntersectSelection => tr!("cmd-desc-intersect-selection"),
            Cmd::IntersectContext => tr!("cmd-desc-intersect-context"),
            Cmd::ReverseFaces => tr!("cmd-desc-reverse-faces"),
            Cmd::OrientFaces => tr!("cmd-desc-orient-faces"),
            Cmd::FlipRed | Cmd::FlipGreen | Cmd::FlipBlue => tr!("cmd-desc-flip"),
            Cmd::Solid(op) => tr!(match op {
                SolidOp::OuterShell => "cmd-desc-solid-outer-shell",
                SolidOp::Intersect => "cmd-desc-solid-intersect",
                SolidOp::Union => "cmd-desc-solid-union",
                SolidOp::Subtract => "cmd-desc-solid-subtract",
                SolidOp::Trim => "cmd-desc-solid-trim",
                SolidOp::Split => "cmd-desc-solid-split",
            }),
            Cmd::Unfold => tr!("cmd-desc-unfold"),
            Cmd::Text3d => tr!("cmd-desc-text-3d"),
            Cmd::ZoomExtents => tr!("cmd-desc-zoom-extents"),
            Cmd::Snapping => tr!("cmd-desc-snapping"),
            Cmd::ShowGrid => tr!("cmd-desc-show-grid"),
            Cmd::Snap(_) => tr!("cmd-desc-snap"),
            Cmd::SideBar => tr!("cmd-desc-side-bar"),
            Cmd::Card(n) => tr!("cmd-desc-card", card = crate::ui::tray::card_label(n)),
            Cmd::Render => tr!("cmd-desc-render"),
            Cmd::WorkPlanes => tr!("cmd-desc-work-planes"),
            Cmd::ModelInfo => tr!("cmd-desc-model-info"),
            Cmd::ComponentParams => tr!("cmd-desc-component-params"),
            Cmd::Library3d => tr!("cmd-desc-library-3d"),
            Cmd::GeneratorEditor => tr!("cmd-desc-generator-editor"),
            Cmd::Console => tr!("cmd-desc-console"),
            Cmd::Log => tr!("cmd-desc-log"),
            Cmd::Preferences => tr!("cmd-desc-preferences"),
            _ => tr!("cmd-desc-default", name = self.name()),
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
            Cmd::ExportDae => Action::ExportDae,
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
        (false, false) => format!("Ctrl+{key}"), // i18n-ignore
        (false, true) => format!("Ctrl+Shift+{key}"), // i18n-ignore
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
