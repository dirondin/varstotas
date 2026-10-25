//! Menu bar: File Edit View Camera Draw Tools Window Help.

use varstotas_i18n::tr;
use crate::app::{Action, FaceStylePref, VarstotasApp};
use crate::templates::Template;
use varstotas_kernel::EntityRef;
use crate::tools::ToolId;
use super::commands::Cmd;
use super::icons::{Icon, paint_icon};
use egui::{Button, Response, Ui};
use varstotas_render::StandardView;

/// Side of a menu row's icon (not taller than the text: the rows keep their height).
const MENU_ICON: f32 = 15.0;

/// A menu row with room for an icon on the left (empty when `icon` is None, so the labels of
/// a menu line up).
pub(crate) fn icon_row(ui: &mut Ui, icon: Option<Icon>, label: &str, shortcut: Option<&str>, enabled: bool, selected: bool) -> Response {
    let id = ui.id().with(("menu_icon", label));
    let mut b = Button::new((egui::Atom::custom(id, egui::vec2(MENU_ICON, MENU_ICON)), label)).selected(selected);
    if let Some(s) = shortcut {
        b = b.shortcut_text(s);
    }
    let r = ui.add_enabled_ui(enabled, |ui| b.atom_ui(ui)).inner;
    if let (Some(icon), Some(rect)) = (icon, r.rect(id)) {
        let mut painter = ui.painter().clone();
        if !enabled {
            painter.multiply_opacity(0.4);
        }
        paint_icon(&painter, rect, icon, false);
    }
    r.response
}

/// A checkbox with an icon between the box and the label.
fn icon_check(ui: &mut Ui, on: &mut bool, icon: Icon, label: &str) -> Response {
    let id = ui.id().with(("check_icon", label));
    let r = ui.add(egui::Checkbox::new(on, (egui::Atom::custom(id, egui::vec2(MENU_ICON, MENU_ICON)), label)));
    paint_after_box(ui, &r, icon);
    r
}

/// A radio button with an icon between the circle and the label.
fn icon_radio(ui: &mut Ui, on: bool, icon: Icon, label: &str) -> Response {
    let id = ui.id().with(("radio_icon", label));
    let r = ui.add(egui::RadioButton::new(on, (egui::Atom::custom(id, egui::vec2(MENU_ICON, MENU_ICON)), label)));
    paint_after_box(ui, &r, icon);
    r
}

/// The icon of a checkbox / radio row: right after the box.
fn paint_after_box(ui: &Ui, r: &Response, icon: Icon) {
    let x = r.rect.left() + ui.spacing().icon_width + ui.spacing().icon_spacing;
    let rect = egui::Rect::from_min_size(egui::pos2(x, r.rect.center().y - MENU_ICON / 2.0), egui::vec2(MENU_ICON, MENU_ICON));
    let mut painter = ui.painter().clone();
    if !ui.is_enabled() {
        painter.multiply_opacity(0.4);
    }
    paint_icon(&painter, rect, icon, false);
}

/// A submenu's row with an icon slot, like [`icon_row`].
fn icon_menu<R>(ui: &mut Ui, icon: Option<Icon>, label: &str, add_contents: impl FnOnce(&mut Ui) -> R) -> Response {
    let id = ui.id().with(("menu_icon", label));
    let r = ui.menu_button((egui::Atom::custom(id, egui::vec2(MENU_ICON, MENU_ICON)), label), add_contents).response;
    if let Some(icon) = icon {
        let x = r.rect.left() + ui.spacing().button_padding.x;
        let rect = egui::Rect::from_min_size(egui::pos2(x, r.rect.center().y - MENU_ICON / 2.0), egui::vec2(MENU_ICON, MENU_ICON));
        let mut painter = ui.painter().clone();
        if !ui.is_enabled() {
            painter.multiply_opacity(0.4);
        }
        paint_icon(&painter, rect, icon, false);
    }
    r
}

impl VarstotasApp {
    /// A command's row: its icon, name, shortcut and state.
    fn cmd_item(&mut self, ui: &mut Ui, c: Cmd) {
        self.cmd_item_as(ui, c, &c.menu_label());
    }

    pub(super) fn cmd_item_as(&mut self, ui: &mut Ui, c: Cmd, label: &str) {
        let response = icon_row(ui, Some(c.icon()), label, c.shortcut().as_deref(), c.enabled(self), false);
        super::tray_lib::spot(format!("cmd:{}", c.key()), response.rect.center());
        if response.clicked() {
            self.actions.push(c.action());
        }
    }

    fn item(&mut self, ui: &mut Ui, label: &str, shortcut: Option<&str>, enabled: bool, action: Action) {
        self.icon_item(ui, None, label, shortcut, enabled, action);
    }

    fn icon_item(&mut self, ui: &mut Ui, icon: Option<Icon>, label: &str, shortcut: Option<&str>, enabled: bool, action: Action) {
        if icon_row(ui, icon, label, shortcut, enabled, false).clicked() {
            self.actions.push(action);
        }
    }

    fn tool_item(&mut self, ui: &mut Ui, id: ToolId) {
        let sel = self.tool_id() == id;
        if icon_row(ui, Some(id.icon()), &id.name(), id.shortcut_label().as_deref(), true, sel).clicked() {
            self.actions.push(Action::SetTool(id));
        }
    }

    /// Intersect / Reverse / Orient / Flip entries (Edit menu and the viewport context menu).
    fn face_ops_items(&mut self, ui: &mut Ui) {
        ui.add_enabled_ui(Cmd::IntersectModel.enabled(self), |ui| {
            icon_menu(ui, Some(Icon::Intersect(0)), &tr!("menu-intersect-faces"), |ui| {
                for c in [Cmd::IntersectModel, Cmd::IntersectSelection, Cmd::IntersectContext] {
                    self.cmd_item(ui, c);
                }
            });
        });
        self.cmd_item(ui, Cmd::ReverseFaces);
        self.cmd_item(ui, Cmd::OrientFaces);
        ui.add_enabled_ui(Cmd::FlipRed.enabled(self), |ui| {
            icon_menu(ui, Some(Icon::Flip(0)), &tr!("menu-flip-along"), |ui| {
                for c in [Cmd::FlipRed, Cmd::FlipGreen, Cmd::FlipBlue] {
                    self.cmd_item(ui, c);
                }
            });
        });
    }

    fn clipboard_items(&mut self, ui: &mut Ui) {
        for c in [Cmd::Cut, Cmd::Copy, Cmd::Paste, Cmd::PasteInPlace] {
            self.cmd_item(ui, c);
        }
    }

    /// Right-click menu of the 3D view.
    pub(crate) fn viewport_context_menu(&mut self, ui: &mut Ui) {
        if self.ctx_on_axes {
            self.item(ui, &tr!("menu-axes-place"), None, true, Action::PlaceAxes);
            self.item(ui, &tr!("menu-axes-move"), None, true, Action::MoveAxes);
            self.item(ui, &tr!("menu-axes-reset"), None, self.doc.model.axes.is_some(), Action::ResetAxes);
            return;
        }
        self.clipboard_items(ui);
        self.cmd_item_as(ui, Cmd::Delete, &tr!("menu-erase"));
        ui.separator();
        for c in [Cmd::Hide, Cmd::Lock, Cmd::MakeGroup, Cmd::MakeComponent, Cmd::MakeParametric, Cmd::Explode] {
            self.cmd_item(ui, c);
            // on a component other copies share
            if c == Cmd::MakeComponent && self.doc.can_make_unique() {
                self.cmd_item(ui, Cmd::MakeUnique);
            }
        }
        ui.separator();
        if let Some(i) = self.doc.section_sel {
            let on = self.doc.model.sections.get(i).is_some_and(|s| s.active);
            let b = Button::new(tr!("menu-active-cut")).selected(on);
            if ui.add(b).clicked() {
                self.actions.push(Action::ActiveCut);
                ui.close();
            }
            self.item(ui, &tr!("menu-reverse"), None, true, Action::ReverseSection);
            self.item(ui, &tr!("menu-align-view"), None, true, Action::AlignSectionView);
            ui.separator();
        }
        self.face_ops_items(ui);
        let can_unfold = self.doc.selection.iter().any(|e| matches!(e, EntityRef::Face(_) | EntityRef::Instance(_)));
        if icon_row(ui, Some(Icon::Unfold), &tr!("menu-unfold"), None, can_unfold, false).clicked() {
            self.actions.push(Action::Unfold);
        }
        if self.selected_textured_face() {
            ui.menu_button(tr!("menu-texture"), |ui| {
                self.icon_item(ui, Some(Icon::TexturePosition), &tr!("menu-texture-position"), None, true, Action::SetTool(ToolId::TexturePosition));
                self.icon_item(ui, Some(Icon::UvEditor), &tr!("menu-uv-editor"), None, true, Action::UvEditor);
            });
        }
        // UV seams on the selected edges
        let edges: Vec<varstotas_kernel::EdgeId> = self.doc.selection.iter().filter_map(|e| if let EntityRef::Edge(e) = e { Some(*e) } else { None }).collect();
        if !edges.is_empty() {
            let mesh = self.doc.model.mesh_at(&self.doc.active_path);
            let any = |seam: bool| edges.iter().any(|e| mesh.and_then(|m| m.edges.get(*e)).is_some_and(|ed| ed.seam == seam));
            let (unmarked, marked) = (any(false), any(true));
            if unmarked && ui.button(tr!("menu-mark-seam")).clicked() {
                self.uv_run("seams_mark");
                ui.close();
            }
            if marked && ui.button(tr!("menu-clear-seam")).clicked() {
                self.uv_run("seams_clear");
                ui.close();
            }
        }
        if !self.doc.active_path.is_empty() {
            ui.separator();
            self.cmd_item(ui, Cmd::CloseGroup);
        }
    }

    /// A selected face of the active context has a textured material.
    fn selected_textured_face(&self) -> bool {
        let model = &self.doc.model;
        let Some(mesh) = model.mesh_at(&self.doc.active_path) else { return false };
        self.doc.selection.iter().any(|e| {
            let EntityRef::Face(fid) = e else { return false };
            mesh.faces.get(*fid).is_some_and(|f| {
                [f.front, f.back].into_iter().flatten().any(|id| model.materials.get(id).is_some_and(|m| m.image.is_some()))
            })
        })
    }

    pub(crate) fn menu_bar(&mut self, ui: &mut Ui) {
        // Collect QA targets for the whole frame, including menus before the toolbar.
        super::tray_lib::clear_spots();
        egui::MenuBar::new().ui(ui, |ui| {
            ui.menu_button(tr!("menu-file"), |ui| {
                self.cmd_item(ui, Cmd::New);
                icon_menu(ui, None, &tr!("menu-new-from-template"), |ui| {
                    for t in Template::ALL {
                        let b = Button::new(t.name()).selected(self.prefs.template == Some(t));
                        if ui.add(b).on_hover_text(t.description()).clicked() {
                            self.actions.push(Action::NewFromTemplate(t));
                        }
                    }
                    ui.separator();
                    ui.label(egui::RichText::new(tr!("template-files-heading")).weak());
                    let dir = crate::templates::directory(self.prefs.templates_dir.as_deref());
                    if let Some(dir) = &dir {
                        match crate::templates::scan(dir) {
                            Ok(files) if files.is_empty() => { ui.label(tr!("template-files-empty")); }
                            Ok(files) => {
                                egui::ScrollArea::vertical().max_height(300.0).show(ui, |ui| {
                                    for file in files {
                                        if ui.button(&file.name).on_hover_text(file.path.display().to_string()).clicked() {
                                            self.actions.push(Action::NewFromFileTemplate(file.path));
                                            ui.close();
                                        }
                                    }
                                });
                            }
                            Err(e) => { ui.label(tr!("template-folder-error", error = e.to_string())); }
                        }
                    }
                    ui.separator();
                    if ui.add_enabled(dir.is_some(), Button::new(tr!("template-open-folder"))).clicked() {
                        if let Some(dir) = dir { super::extensions::reveal(&dir); }
                        ui.close();
                    }
                });
                self.cmd_item(ui, Cmd::Open);
                let recent = self.prefs.recent.clone();
                ui.add_enabled_ui(!recent.is_empty(), |ui| {
                    icon_menu(ui, None, &tr!("menu-open-recent"), |ui| {
                        for p in recent {
                            let name = p.file_name().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
                            if ui.button(name).on_hover_text(p.display().to_string()).clicked() {
                                self.actions.push(Action::OpenPath(p));
                            }
                        }
                    });
                });
                let r = ui.button(tr!("start-menu"));
                super::tray_lib::spot("menu:StartScreen", r.rect.center());
                if r.clicked() { self.actions.push(Action::StartScreen); ui.close(); }
                ui.separator();
                self.cmd_item(ui, Cmd::Save);
                self.cmd_item(ui, Cmd::SaveAs);
                icon_menu(ui, Some(Icon::Picture(0)), &tr!("menu-preview-image"), |ui| {
                    self.cmd_item(ui, Cmd::PreviewShow);
                    ui.separator();
                    for c in [Cmd::PreviewFromFile, Cmd::PreviewFromView, Cmd::PreviewRemove] {
                        self.cmd_item(ui, c);
                    }
                })
                .on_hover_text(tr!("menu-preview-image-hint"));
                ui.separator();
                icon_menu(ui, Some(Icon::Import("")), &tr!("menu-import"), |ui| {
                    use Cmd::*;
                    for c in [ImportObj, ImportStl, ImportDxf, ImportGltf, Import3mf, ImportDae, ImportFbx, ImportPly, ImportCsv] {
                        self.cmd_item(ui, c);
                    }
                    self.plugin_file_items(ui, false);
                    self.model_import_items(ui);
                });
                icon_menu(ui, Some(Icon::Export("")), &tr!("menu-export"), |ui| {
                    use Cmd::*;
                    self.cmd_item(ui, ExportImage);
                    self.cmd_item(ui, ExportAnimation);
                    ui.separator();
                    for c in [ExportObj, ExportStl, ExportDxf, ExportGlb, Export3mf, ExportDae, ExportPly, ExportDrawing] {
                        self.cmd_item(ui, c);
                    }
                    self.cmd_item_as(ui, Unfold, &tr!("menu-unfold-export"));
                    self.cmd_item(ui, Sheets);
                    ui.separator();
                    for c in [ExportReport, ExportFacesJson, ExportAttributions] {
                        self.cmd_item(ui, c);
                    }
                    self.plugin_file_items(ui, true);
                });
                ui.separator();
                self.cmd_item(ui, Cmd::RunScript);
                ui.separator();
                self.cmd_item(ui, Cmd::Quit);
            });

            let edit_menu = ui.menu_button(tr!("menu-edit"), |ui| {
                let undo = self.doc.history.undo_label().map(|l| tr!("menu-undo-named", name = l)).unwrap_or_else(|| Cmd::Undo.menu_label());
                let redo = self.doc.history.redo_label().map(|l| tr!("menu-redo-named", name = l)).unwrap_or_else(|| Cmd::Redo.menu_label());
                self.cmd_item_as(ui, Cmd::Undo, &undo);
                self.cmd_item_as(ui, Cmd::Redo, &redo);
                ui.separator();
                self.clipboard_items(ui);
                for c in [Cmd::Delete, Cmd::PurgeUnused, Cmd::DeleteGuides] {
                    self.cmd_item(ui, c);
                }
                ui.separator();
                self.cmd_item(ui, Cmd::SelectAll);
                self.cmd_item(ui, Cmd::SelectNone);
                ui.separator();
                self.cmd_item(ui, Cmd::Hide);
                self.cmd_item(ui, Cmd::Lock);
                self.cmd_item(ui, Cmd::UnhideAll);
                self.cmd_item(ui, Cmd::UnlockAll);
                ui.separator();
                for c in [Cmd::MakeGroup, Cmd::MakeComponent, Cmd::MakeUnique, Cmd::MakeParametric, Cmd::Explode, Cmd::CloseGroup] {
                    self.cmd_item(ui, c);
                }
                ui.separator();
                self.face_ops_items(ui);
            });
            super::tray_lib::spot("menu:Edit", edit_menu.response.rect.center());

            ui.menu_button(tr!("menu-view"), |ui| {
                ui.label(egui::RichText::new(tr!("menu-face-style")).small().weak());
                for (st, icon, label) in [
                    (FaceStylePref::ShadedColor, Icon::StyleShaded, tr!("menu-shaded")),
                    (FaceStylePref::Shaded, Icon::StyleTextured, tr!("menu-shaded-textures")),
                    (FaceStylePref::Monochrome, Icon::StyleMonochrome, tr!("menu-monochrome")),
                    (FaceStylePref::HiddenLine, Icon::StyleHiddenLine, tr!("menu-hidden-line")),
                    (FaceStylePref::Wireframe, Icon::StyleWireframe, tr!("menu-wireframe")),
                    (FaceStylePref::Realistic, Icon::StyleRealistic, tr!("menu-realistic")),
                ] {
                    if icon_radio(ui, self.prefs.face_style == st, icon, &label).clicked() {
                        self.prefs.face_style = st;
                    }
                }
                icon_check(ui, &mut self.prefs.xray, Icon::StyleXRay, &tr!("menu-x-ray"));
                ui.separator();
                ui.label(egui::RichText::new(tr!("menu-edge-style")).small().weak());
                icon_check(ui, self.prefs.edges_mut(), Icon::StyleEdges, &tr!("menu-edges")).on_hover_text(tr!("menu-edges-hint"));
                let r = icon_check(ui, &mut self.prefs.show_triangles, Icon::StyleWireframe, &tr!("view-triangles"));
                super::tray_lib::spot("menu:Triangles", r.rect.center());
                r.on_hover_text(tr!("view-triangles-hint"));
                icon_check(ui, &mut self.prefs.profiles, Icon::StyleProfiles, &tr!("menu-profiles"));
                ui.separator();
                let mut guides = !self.doc.model.guides_hidden;
                if icon_check(ui, &mut guides, Icon::Guides, &tr!("menu-guides")).clicked() {
                    self.actions.push(Action::ToggleGuides);
                }
                icon_check(ui, &mut self.prefs.show_axes, Icon::Axes, &tr!("menu-axes"));
                icon_check(ui, &mut self.prefs.show_ground, Icon::Ground, &tr!("menu-ground"));
                icon_check(ui, &mut self.prefs.show_sky, Icon::Sky, &tr!("menu-sky")).on_hover_text(tr!("menu-sky-hint"));
                let mut grid = self.prefs.grid.show;
                if icon_check(ui, &mut grid, Icon::Grid, &tr!("menu-grid")).clicked() {
                    self.actions.push(Action::ToggleGrid);
                }
                ui.separator();
                let mut shadows = self.doc.model.shadows.enabled;
                if icon_check(ui, &mut shadows, Icon::Shadows, &tr!("menu-shadows")).clicked() {
                    self.actions.push(Action::ToggleShadows);
                }
                ui.separator();
                let d = self.doc.model.section_display;
                for (k, mut on, icon, label) in [
                    (0u8, d.planes, Icon::Section, tr!("menu-section-planes")),
                    (1, d.cuts, Icon::SectionCut, tr!("menu-section-cuts")),
                    (2, d.fill, Icon::SectionFill, tr!("menu-section-fill")),
                ] {
                    if icon_check(ui, &mut on, icon, &label).clicked() {
                        self.actions.push(Action::ToggleSectionDisplay(k));
                    }
                }
                ui.separator();
                let panes = [Icon::Panes1, Icon::Panes2, Icon::Panes2, Icon::Panes4, Icon::Panes4];
                icon_menu(ui, Some(panes[self.pane_count.min(4)]), &tr!("menu-viewports"), |ui| {
                    for (n, label) in [(1, tr!("menu-viewports-one")), (2, tr!("menu-viewports-two")), (4, tr!("menu-viewports-four"))] {
                        if icon_radio(ui, self.pane_count == n, panes[n], &label).clicked() {
                            self.actions.push(Action::SetPanes(n));
                            ui.close();
                        }
                    }
                    ui.separator();
                    ui.label(egui::RichText::new(tr!("menu-presets")).small().weak());
                    for (i, p) in crate::panes::PRESETS.iter().enumerate() {
                        if icon_row(ui, Some(if p.panes.len() == 4 { Icon::Panes4 } else { Icon::Panes2 }), &p.name(), None, true, false).clicked() {
                            self.actions.push(Action::ViewPreset(i));
                            ui.close();
                        }
                    }
                });
                ui.separator();
                icon_menu(ui, Some(Icon::Film), &tr!("menu-animation"), |ui| {
                    let has = !self.doc.model.scenes.is_empty();
                    let (play, icon) = if self.scene_player.is_some() { (tr!("menu-pause"), Icon::Pause) } else { (tr!("menu-play"), Icon::Play) };
                    self.icon_item(ui, Some(icon), &play, None, has, Action::ToggleAnimation);
                    ui.separator();
                    self.icon_item(ui, Some(Icon::NextScene), &tr!("menu-next-scene"), crate::keymap::label(crate::keymap::KeyCmd::NextScene).as_deref(), has, Action::NextScene);
                    self.icon_item(ui, Some(Icon::PrevScene), &tr!("menu-previous-scene"), crate::keymap::label(crate::keymap::KeyCmd::PrevScene).as_deref(), has, Action::PrevScene);
                    ui.separator();
                    self.icon_item(ui, Some(Icon::AddScene), &tr!("menu-add-scene"), None, true, Action::AddScene);
                });
            });

            ui.menu_button(tr!("menu-camera"), |ui| {
                ui.menu_button(tr!("menu-standard-views"), |ui| {
                    for (name, v) in [
                        (tr!("view-iso"), StandardView::Iso),
                        (tr!("view-top"), StandardView::Top),
                        (tr!("view-bottom"), StandardView::Bottom),
                        (tr!("view-front"), StandardView::Front),
                        (tr!("view-back"), StandardView::Back),
                        (tr!("view-left"), StandardView::Left),
                        (tr!("view-right"), StandardView::Right),
                    ] {
                        self.icon_item(ui, Some(super::view_hud::view_icon(v)), &name, None, true, Action::View(v));
                    }
                });
                ui.separator();
                let ortho = self.camera.ortho;
                if ui.radio(ortho, tr!("menu-parallel-projection")).clicked() {
                    self.actions.push(Action::SetParallel(true));
                }
                if ui.radio(!ortho, tr!("menu-perspective")).clicked() {
                    self.actions.push(Action::SetParallel(false));
                }
                ui.separator();
                self.tool_item(ui, ToolId::Orbit);
                self.tool_item(ui, ToolId::Pan);
                self.tool_item(ui, ToolId::Zoom);
                self.cmd_item(ui, Cmd::ZoomExtents);
                ui.separator();
                self.tool_item(ui, ToolId::PositionCamera);
                self.tool_item(ui, ToolId::LookAround);
                self.tool_item(ui, ToolId::Walk);
            });

            ui.menu_button(tr!("menu-draw"), |ui| {
                self.tool_item(ui, ToolId::Line);
                ui.separator();
                for id in [ToolId::ArcCenter, ToolId::Arc, ToolId::Arc3, ToolId::Pie, ToolId::Bezier, ToolId::SmoothCurve, ToolId::Freehand, ToolId::EditCurve] {
                    self.tool_item(ui, id);
                }
                ui.separator();
                for id in [ToolId::Rectangle, ToolId::RotatedRectangle, ToolId::Circle, ToolId::Polygon] {
                    self.tool_item(ui, id);
                }
                ui.separator();
                ui.label(egui::RichText::new(tr!("menu-shapes")).small().weak());
                for id in crate::tools::SHAPES {
                    self.tool_item(ui, id);
                }
                self.cmd_item(ui, Cmd::Text3d);
            });

            ui.menu_button(tr!("menu-tools"), |ui| {
                for id in [ToolId::Select, ToolId::Eraser, ToolId::Paint, ToolId::TexturePosition, ToolId::Section, ToolId::GuidePlane, ToolId::Slice] {
                    self.tool_item(ui, id);
                }
                ui.separator();
                for id in [
                    ToolId::Move,
                    ToolId::Rotate,
                    ToolId::Scale,
                    ToolId::PushPull,
                    ToolId::FollowMe,
                    ToolId::Offset,
                    ToolId::Chamfer,
                    ToolId::Fillet,
                ] {
                    self.tool_item(ui, id);
                }
                icon_menu(ui, Some(Icon::Solids(2)), &tr!("menu-solid-tools"), |ui| {
                    for op in varstotas_kernel::SolidOp::ALL {
                        self.cmd_item(ui, Cmd::Solid(op));
                    }
                });
                ui.separator();
                for id in [ToolId::Tape, ToolId::Dimension, ToolId::Protractor, ToolId::Axes, ToolId::Text] {
                    self.tool_item(ui, id);
                }
            });

            ui.menu_button(tr!("menu-snapping"), |ui| self.snapping_menu(ui));

            let extensions_menu = ui.menu_button(tr!("menu-extensions"), |ui| self.extensions_menu(ui));
            super::tray_lib::spot("menu:Extensions",extensions_menu.response.rect.center());

            // Stays open on clicks so several tray cards can be toggled; buttons close it.
            let keep = egui::containers::menu::MenuConfig::new().close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside);
            let (window_menu, _) = egui::containers::menu::MenuButton::new(tr!("menu-window")).config(keep).ui(ui, |ui| {
                let mut open = self.prefs.tray_open;
                if icon_check(ui, &mut open, Icon::SideBar, &tr!("menu-side-bar")).clicked() {
                    self.actions.push(Action::ToggleTray);
                }
                // The tray's cards: a checkbox each (turning one on opens the tray).
                ui.indent("tray_cards", |ui| {
                    for c in crate::ui::tray::TRAY_CARDS {
                        let mut shown = Cmd::Card(c).checked(self).unwrap_or(true);
                        if icon_check(ui, &mut shown, Cmd::Card(c).icon(), &crate::ui::tray::card_label(c)).clicked() {
                            self.actions.push(Action::ToggleCard(c));
                        }
                    }
                });
                ui.separator();
                use Cmd::*;
                // by feature: the model, the online libraries, its look, making things from it, scripting; Some(None): the
                // UV Editor (no command of its own); None: a separator
                for c in [
                    Some(Some(ModelInfo)),
                    Some(Some(ComponentParams)),
                    Some(Some(WorkPlanes)),
                    None,
                    Some(Some(Library3d)),
                    Some(Some(MaterialsOnline)),
                    None,
                    Some(None),
                    Some(Some(Render)),
                    None,
                    Some(Some(Physics)),
                    Some(Some(Unfold)),
                    Some(Some(Sheets)),
                    Some(Some(PrintCheck)),
                    None,
                    Some(Some(GeneratorEditor)),
                    Some(Some(Console)),
                    Some(Some(Log)),
                    None,
                    Some(Some(Preferences)),
                ] {
                    let Some(c) = c else {
                        ui.separator();
                        continue;
                    };
                    let Some(c) = c else {
                        let label = tr!("menu-uv-editor");
                        let r = icon_row(ui, Some(Icon::UvEditor), label.trim_end_matches('…'), None, true, self.uv.open);
                        super::tray_lib::spot("menu:UvEditor", r.rect.center());
                        if r.clicked() {
                            self.actions.push(Action::UvEditor);
                            ui.close();
                        }
                        continue;
                    };
                    let label = match c {
                        Sheets => tr!("menu-sheets"),
                        PrintCheck => tr!("print-title"),
                        _ => c.menu_label(),
                    };
                    let on = c.checked(self).unwrap_or(false);
                    // window names without the trailing "…" here
                    let r = icon_row(ui, Some(c.icon()), label.trim_end_matches('…'), c.shortcut().as_deref(), c.enabled(self), on).on_hover_text(c.description());
                    super::tray_lib::spot(format!("cmd:{}", c.key()), r.rect.center());
                    if r.clicked() {
                        self.actions.push(c.action());
                        ui.close();
                    }
                }
            });
            super::tray_lib::spot("menu:Window", window_menu.rect.center());

            let help_menu=ui.menu_button(tr!("menu-help"), |ui| {
                let r = ui.button(tr!("tutorial-title"));
                super::tray_lib::spot("menu:Tutorial", r.rect.center());
                if r.clicked() { self.actions.push(Action::Tutorial); ui.close(); }
                let guide=ui.add(egui::Button::new(tr!("menu-user-guide")).shortcut_text("F1").selected(self.app_help.open)).on_hover_text(tr!("menu-user-guide-hint"));
                super::tray_lib::spot("menu:UserGuide",guide.rect.center());
                if guide.clicked() {
                    self.open_app_help(crate::ui::app_help::Topic::Start);
                    ui.close();
                }
                let reference=ui.add(egui::Button::new(tr!("menu-script-reference")).selected(self.script_help.open)).on_hover_text(tr!("menu-script-reference-hint"));
                super::tray_lib::spot("menu:ScriptReference",reference.rect.center());
                if reference.clicked() {
                    self.open_script_help(crate::ui::script_help::Topic::Basics, None);
                    ui.close();
                }
                let examples = ui.add_enabled(!crate::EXAMPLES_URL.is_empty(), egui::Button::new(tr!("menu-examples"))).on_hover_text(tr!("menu-examples-hint"));
                super::tray_lib::spot("menu:Examples", examples.rect.center());
                if examples.clicked() {
                    super::online_mats::open_url(crate::EXAMPLES_URL);
                    ui.close();
                }
                ui.separator();
                let licences=ui.add(egui::Button::new(tr!("menu-licences")).selected(self.licences.open)).on_hover_text(tr!("menu-licences-hint"));
                super::tray_lib::spot("menu:Licences",licences.rect.center());
                if licences.clicked() {
                    self.open_licences(crate::ui::licences::Tab::Varstotas);
                    ui.close();
                }
                if ui.button(tr!("menu-about")).clicked() {
                    self.actions.push(Action::About);
                    ui.close();
                }
                ui.separator();
                let r = ui.button(tr!("report-menu"));
                super::tray_lib::spot("menu:ReportProblem", r.rect.center());
                if r.clicked() { self.actions.push(Action::ReportProblem); ui.close(); }
            });
            super::tray_lib::spot("menu:Help",help_menu.response.rect.center());
        });
    }

    /// View → Snapping and the magnet button's menu: the master switch and every inference.
    pub(crate) fn snapping_menu(&mut self, ui: &mut Ui) {
        use crate::inference::settings::GROUPS;
        let mut on = self.prefs.snapping.enabled;
        if icon_check(ui, &mut on, Icon::Magnet, &tr!("menu-snapping")).on_hover_text(crate::keymap::label(crate::keymap::KeyCmd::ToggleSnapping).unwrap_or_default()).clicked() {
            self.actions.push(Action::ToggleSnapping);
        }
        ui.horizontal(|ui| {
            if ui.small_button(tr!("menu-all-on")).clicked() {
                self.prefs.snapping.off.clear();
            }
            if ui.small_button(tr!("menu-all-off")).clicked() {
                for k in crate::inference::settings::all_kinds() {
                    self.prefs.snapping.set(k, false);
                }
            }
        });
        for (group, kinds) in GROUPS {
            ui.separator();
            ui.label(egui::RichText::new(crate::inference::settings::group_label(group)).small().weak());
            if group == "Grid" { // i18n-ignore
                let mut grid = self.prefs.grid.show;
                if icon_check(ui, &mut grid, Icon::Grid, &tr!("menu-show-grid")).clicked() {
                    self.actions.push(Action::ToggleGrid);
                }
            }
            ui.add_enabled_ui(self.prefs.snapping.enabled, |ui| {
                for &k in kinds {
                    let mut v = self.prefs.snapping.is_on(k);
                    let r = icon_check(ui, &mut v, Icon::Snap(k), &k.menu_label());
                    super::tray_lib::spot(format!("snap:{}", k.menu_label()), r.rect.center());
                    if r.changed() {
                        self.prefs.snapping.set(k, v);
                    }
                }
            });
        }        // Increments: round free moves, angles and scale factors to a step.
        ui.separator();
        ui.label(egui::RichText::new(tr!("menu-increments")).small().weak());
        let unit = self.doc.model.unit;
        ui.add_enabled_ui(self.prefs.snapping.enabled, |ui| {
            let inc = &mut self.prefs.snapping.increments;
            egui::Grid::new("snap_increments").num_columns(2).spacing([8.0, 2.0]).show(ui, |ui| {
                ui.checkbox(&mut inc.move_on, tr!("menu-increments-move"));
                ui.add(
                    egui::DragValue::new(&mut inc.move_step)
                        .range(0.001..=1.0e6)
                        .speed(1.0)
                        .custom_formatter(move |v, _| varstotas_kernel::units::format_length(v, unit))
                        .custom_parser(move |t| varstotas_kernel::units::parse_length(t, unit)),
                );
                ui.end_row();
                ui.checkbox(&mut inc.rotate_on, tr!("menu-increments-rotate"));
                ui.add(egui::DragValue::new(&mut inc.angle_step).range(0.01..=180.0).speed(0.5).suffix("°"));
                ui.end_row();
                ui.checkbox(&mut inc.scale_on, tr!("menu-increments-scale"));
                ui.add(egui::DragValue::new(&mut inc.scale_step).range(0.001..=10.0).speed(0.01).max_decimals(3));
                ui.end_row();
            });
        });
    }
}
