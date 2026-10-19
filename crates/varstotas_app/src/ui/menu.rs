//! Menu bar: File Edit View Camera Draw Tools Window Help.

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
        self.cmd_item_as(ui, c, c.menu_label());
    }

    fn cmd_item_as(&mut self, ui: &mut Ui, c: Cmd, label: &str) {
        if icon_row(ui, Some(c.icon()), label, c.shortcut().as_deref(), c.enabled(self), false).clicked() {
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
        if icon_row(ui, Some(id.icon()), id.name(), id.shortcut_label().as_deref(), true, sel).clicked() {
            self.actions.push(Action::SetTool(id));
        }
    }

    /// Intersect / Reverse / Orient / Flip entries (Edit menu and the viewport context menu).
    fn face_ops_items(&mut self, ui: &mut Ui) {
        ui.add_enabled_ui(Cmd::IntersectModel.enabled(self), |ui| {
            icon_menu(ui, Some(Icon::Intersect(0)), "Intersect Faces", |ui| {
                for c in [Cmd::IntersectModel, Cmd::IntersectSelection, Cmd::IntersectContext] {
                    self.cmd_item(ui, c);
                }
            });
        });
        self.cmd_item(ui, Cmd::ReverseFaces);
        self.cmd_item(ui, Cmd::OrientFaces);
        ui.add_enabled_ui(Cmd::FlipRed.enabled(self), |ui| {
            icon_menu(ui, Some(Icon::Flip(0)), "Flip Along", |ui| {
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
            self.item(ui, "Place", None, true, Action::PlaceAxes);
            self.item(ui, "Move…", None, true, Action::MoveAxes);
            self.item(ui, "Reset", None, self.doc.model.axes.is_some(), Action::ResetAxes);
            return;
        }
        self.clipboard_items(ui);
        self.cmd_item_as(ui, Cmd::Delete, "Erase");
        ui.separator();
        for c in [Cmd::Hide, Cmd::MakeGroup, Cmd::MakeComponent, Cmd::MakeParametric, Cmd::Explode] {
            self.cmd_item(ui, c);
        }
        ui.separator();
        if let Some(i) = self.doc.section_sel {
            let on = self.doc.model.sections.get(i).is_some_and(|s| s.active);
            let b = Button::new("Active Cut").selected(on);
            if ui.add(b).clicked() {
                self.actions.push(Action::ActiveCut);
                ui.close();
            }
            self.item(ui, "Reverse", None, true, Action::ReverseSection);
            self.item(ui, "Align View", None, true, Action::AlignSectionView);
            ui.separator();
        }
        self.face_ops_items(ui);
        let can_unfold = self.doc.selection.iter().any(|e| matches!(e, EntityRef::Face(_) | EntityRef::Instance(_)));
        if icon_row(ui, Some(Icon::Unfold), "Unfold…", None, can_unfold, false).clicked() {
            self.actions.push(Action::Unfold);
        }
        if self.selected_textured_face() {
            ui.menu_button("Texture", |ui| {
                self.item(ui, "Position", None, true, Action::SetTool(ToolId::TexturePosition));
            });
        }
        if !self.doc.active_path.is_empty() {
            ui.separator();
            self.cmd_item(ui, Cmd::CloseGroup);
        }
    }

    /// Exactly one face of the active context is selected and it has a textured material.
    fn selected_textured_face(&self) -> bool {
        let [EntityRef::Face(fid)] = self.doc.selection.as_slice() else { return false };
        let model = &self.doc.model;
        model.mesh_at(&self.doc.active_path).and_then(|m| m.faces.get(*fid)).is_some_and(|f| {
            [f.front, f.back].into_iter().flatten().any(|id| model.materials.get(id).is_some_and(|m| m.image.is_some()))
        })
    }

    pub(crate) fn menu_bar(&mut self, ui: &mut Ui) {
        egui::MenuBar::new().ui(ui, |ui| {
            ui.menu_button("File", |ui| {
                self.cmd_item(ui, Cmd::New);
                icon_menu(ui, None, "New from Template", |ui| {
                    for t in Template::ALL {
                        let b = Button::new(t.name()).selected(self.prefs.template == Some(t));
                        if ui.add(b).on_hover_text(t.description()).clicked() {
                            self.actions.push(Action::NewFromTemplate(t));
                        }
                    }
                });
                self.cmd_item(ui, Cmd::Open);
                let recent = self.prefs.recent.clone();
                ui.add_enabled_ui(!recent.is_empty(), |ui| {
                    icon_menu(ui, None, "Open Recent", |ui| {
                        for p in recent {
                            let name = p.file_name().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
                            if ui.button(name).on_hover_text(p.display().to_string()).clicked() {
                                self.actions.push(Action::OpenPath(p));
                            }
                        }
                    });
                });
                ui.separator();
                self.cmd_item(ui, Cmd::Save);
                self.cmd_item(ui, Cmd::SaveAs);
                icon_menu(ui, Some(Icon::Picture(0)), "Preview Image", |ui| {
                    self.cmd_item(ui, Cmd::PreviewShow);
                    ui.separator();
                    for c in [Cmd::PreviewFromFile, Cmd::PreviewFromView, Cmd::PreviewRemove] {
                        self.cmd_item(ui, c);
                    }
                })
                .on_hover_text("The picture the Components library shows for this file (else it draws the model)");
                ui.separator();
                icon_menu(ui, Some(Icon::Import("")), "Import", |ui| {
                    use Cmd::*;
                    for c in [ImportObj, ImportStl, ImportDxf, ImportGltf, Import3mf, ImportDae, ImportFbx, ImportPly, ImportCsv] {
                        self.cmd_item(ui, c);
                    }
                    self.plugin_file_items(ui, false);
                    self.model_import_items(ui);
                });
                icon_menu(ui, Some(Icon::Export("")), "Export", |ui| {
                    use Cmd::*;
                    self.cmd_item(ui, ExportImage);
                    self.cmd_item(ui, ExportAnimation);
                    ui.separator();
                    for c in [ExportObj, ExportStl, ExportDxf, ExportGlb, Export3mf, ExportPly, ExportDrawing] {
                        self.cmd_item(ui, c);
                    }
                    self.cmd_item_as(ui, Unfold, "Unfold (SVG / PDF / DXF)…");
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

            ui.menu_button("Edit", |ui| {
                let undo = self.doc.history.undo_label().map(|l| format!("Undo {l}"));
                let redo = self.doc.history.redo_label().map(|l| format!("Redo {l}"));
                self.cmd_item_as(ui, Cmd::Undo, undo.as_deref().unwrap_or("Undo"));
                self.cmd_item_as(ui, Cmd::Redo, redo.as_deref().unwrap_or("Redo"));
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
                self.cmd_item(ui, Cmd::UnhideAll);
                ui.separator();
                for c in [Cmd::MakeGroup, Cmd::MakeComponent, Cmd::MakeParametric, Cmd::Explode, Cmd::CloseGroup] {
                    self.cmd_item(ui, c);
                }
                ui.separator();
                self.face_ops_items(ui);
            });

            ui.menu_button("View", |ui| {
                ui.label(egui::RichText::new("Face Style").small().weak());
                for (st, icon, label) in [
                    (FaceStylePref::ShadedColor, Icon::StyleShaded, "Shaded"),
                    (FaceStylePref::Shaded, Icon::StyleTextured, "Shaded with Textures"),
                    (FaceStylePref::Monochrome, Icon::StyleMonochrome, "Monochrome"),
                    (FaceStylePref::HiddenLine, Icon::StyleHiddenLine, "Hidden Line"),
                    (FaceStylePref::Wireframe, Icon::StyleWireframe, "Wireframe"),
                    (FaceStylePref::Realistic, Icon::StyleRealistic, "Realistic"),
                ] {
                    if icon_radio(ui, self.prefs.face_style == st, icon, label).clicked() {
                        self.prefs.face_style = st;
                    }
                }
                icon_check(ui, &mut self.prefs.xray, Icon::StyleXRay, "X-Ray");
                ui.separator();
                ui.label(egui::RichText::new("Edge Style").small().weak());
                icon_check(ui, self.prefs.edges_mut(), Icon::StyleEdges, "Edges").on_hover_text("Edges in this face style (Realistic has its own switch)");
                icon_check(ui, &mut self.prefs.profiles, Icon::StyleProfiles, "Profiles");
                ui.separator();
                let mut guides = !self.doc.model.guides_hidden;
                if icon_check(ui, &mut guides, Icon::Guides, "Guides").clicked() {
                    self.actions.push(Action::ToggleGuides);
                }
                icon_check(ui, &mut self.prefs.show_axes, Icon::Axes, "Axes");
                icon_check(ui, &mut self.prefs.show_ground, Icon::Ground, "Ground");
                icon_check(ui, &mut self.prefs.show_sky, Icon::Sky, "Sky").on_hover_text("The sky gradient behind the model; off = a plain light background");
                let mut grid = self.prefs.grid.show;
                if icon_check(ui, &mut grid, Icon::Grid, "Grid").clicked() {
                    self.actions.push(Action::ToggleGrid);
                }
                ui.separator();
                let mut shadows = self.doc.model.shadows.enabled;
                if icon_check(ui, &mut shadows, Icon::Shadows, "Shadows").clicked() {
                    self.actions.push(Action::ToggleShadows);
                }
                ui.separator();
                let d = self.doc.model.section_display;
                for (k, mut on, icon, label) in [
                    (0u8, d.planes, Icon::Section, "Section Planes"),
                    (1, d.cuts, Icon::SectionCut, "Section Cuts"),
                    (2, d.fill, Icon::SectionFill, "Section Fill"),
                ] {
                    if icon_check(ui, &mut on, icon, label).clicked() {
                        self.actions.push(Action::ToggleSectionDisplay(k));
                    }
                }
                ui.separator();
                let panes = [Icon::Panes1, Icon::Panes2, Icon::Panes2, Icon::Panes4, Icon::Panes4];
                icon_menu(ui, Some(panes[self.pane_count.min(4)]), "Viewports", |ui| {
                    for (n, label) in [(1, "One"), (2, "Two (side by side)"), (4, "Four (2 × 2)")] {
                        if icon_radio(ui, self.pane_count == n, panes[n], label).clicked() {
                            self.actions.push(Action::SetPanes(n));
                            ui.close();
                        }
                    }
                    ui.separator();
                    ui.label(egui::RichText::new("Presets").small().weak());
                    for (i, p) in crate::panes::PRESETS.iter().enumerate() {
                        if icon_row(ui, Some(if p.panes.len() == 4 { Icon::Panes4 } else { Icon::Panes2 }), p.name, None, true, false).clicked() {
                            self.actions.push(Action::ViewPreset(i));
                            ui.close();
                        }
                    }
                });
                ui.separator();
                icon_menu(ui, Some(Icon::Film), "Animation", |ui| {
                    let has = !self.doc.model.scenes.is_empty();
                    let (play, icon) = if self.scene_player.is_some() { ("Pause", Icon::Pause) } else { ("Play", Icon::Play) };
                    self.icon_item(ui, Some(icon), play, None, has, Action::ToggleAnimation);
                    ui.separator();
                    self.icon_item(ui, Some(Icon::NextScene), "Next Scene", crate::keymap::label(crate::keymap::KeyCmd::NextScene).as_deref(), has, Action::NextScene);
                    self.icon_item(ui, Some(Icon::PrevScene), "Previous Scene", crate::keymap::label(crate::keymap::KeyCmd::PrevScene).as_deref(), has, Action::PrevScene);
                    ui.separator();
                    self.icon_item(ui, Some(Icon::AddScene), "Add Scene", None, true, Action::AddScene);
                });
            });

            ui.menu_button("Camera", |ui| {
                ui.menu_button("Standard Views", |ui| {
                    for (name, v) in [
                        ("Iso", StandardView::Iso),
                        ("Top", StandardView::Top),
                        ("Bottom", StandardView::Bottom),
                        ("Front", StandardView::Front),
                        ("Back", StandardView::Back),
                        ("Left", StandardView::Left),
                        ("Right", StandardView::Right),
                    ] {
                        self.icon_item(ui, Some(super::view_hud::view_icon(v)), name, None, true, Action::View(v));
                    }
                });
                ui.separator();
                let ortho = self.camera.ortho;
                if ui.radio(ortho, "Parallel Projection").clicked() {
                    self.actions.push(Action::SetParallel(true));
                }
                if ui.radio(!ortho, "Perspective").clicked() {
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

            ui.menu_button("Draw", |ui| {
                self.tool_item(ui, ToolId::Line);
                ui.separator();
                for id in [ToolId::ArcCenter, ToolId::Arc, ToolId::Arc3, ToolId::Pie, ToolId::Bezier, ToolId::SmoothCurve, ToolId::Freehand] {
                    self.tool_item(ui, id);
                }
                ui.separator();
                for id in [ToolId::Rectangle, ToolId::RotatedRectangle, ToolId::Circle, ToolId::Polygon] {
                    self.tool_item(ui, id);
                }
                ui.separator();
                ui.label(egui::RichText::new("Shapes").small().weak());
                for id in crate::tools::SHAPES {
                    self.tool_item(ui, id);
                }
            });

            ui.menu_button("Tools", |ui| {
                for id in [ToolId::Select, ToolId::Eraser, ToolId::Paint, ToolId::TexturePosition, ToolId::Section, ToolId::GuidePlane] {
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
                icon_menu(ui, Some(Icon::Solids(2)), "Solid Tools", |ui| {
                    for op in varstotas_kernel::SolidOp::ALL {
                        self.cmd_item(ui, Cmd::Solid(op));
                    }
                });
                self.cmd_item(ui, Cmd::Unfold);
                ui.separator();
                for id in [ToolId::Tape, ToolId::Dimension, ToolId::Protractor, ToolId::Axes, ToolId::Text] {
                    self.tool_item(ui, id);
                }
                self.cmd_item(ui, Cmd::Text3d);
            });

            ui.menu_button("Snapping", |ui| self.snapping_menu(ui));

            ui.menu_button("Extensions", |ui| self.extensions_menu(ui));

            // Stays open on clicks so several tray cards can be toggled; buttons close it.
            let keep = egui::containers::menu::MenuConfig::new().close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside);
            egui::containers::menu::MenuButton::new("Window").config(keep).ui(ui, |ui| {
                let mut open = self.prefs.tray_open;
                if icon_check(ui, &mut open, Icon::SideBar, "Side Bar").clicked() {
                    self.actions.push(Action::ToggleTray);
                }
                // The tray's cards: a checkbox each (turning one on opens the tray).
                ui.indent("tray_cards", |ui| {
                    for c in crate::ui::tray::TRAY_CARDS {
                        let mut shown = Cmd::Card(c).checked(self).unwrap_or(true);
                        if icon_check(ui, &mut shown, Cmd::Card(c).icon(), c).clicked() {
                            self.actions.push(Action::ToggleCard(c));
                        }
                    }
                });
                ui.separator();
                use Cmd::*;
                // None: a separator
                for c in [Some(Sheets), Some(Render), Some(WorkPlanes), Some(ModelInfo), Some(ComponentParams), Some(Library3d), Some(GeneratorEditor), Some(Console), Some(Log), None, Some(Preferences)] {
                    let Some(c) = c else {
                        ui.separator();
                        continue;
                    };
                    let label = if c == Sheets { "Sheets…" } else { c.menu_label() };
                    let on = c.checked(self).unwrap_or(false);
                    if icon_row(ui, Some(c.icon()), label, c.shortcut().as_deref(), true, on).on_hover_text(c.description()).clicked() {
                        self.actions.push(c.action());
                        ui.close();
                    }
                }
            });

            ui.menu_button("Help", |ui| {
                ui.menu_button("Demos", |ui| {
                    ui.label(egui::RichText::new("Open as a new document").small().weak());
                    self.item(ui, "Country House", None, true, Action::OpenDemoHouse);
                    self.item(ui, "Material Gallery", None, true, Action::OpenDemoGallery);
                    ui.separator();
                    ui.label(egui::RichText::new("Add to this document").small().weak());
                    self.item(ui, "Demo Geometry", None, true, Action::InsertDemo);
                })
                .response
                .on_hover_text("Demo models: furnished house and material gallery with scenes for Window → Render…, test geometry");
                if ui
                    .add(egui::Button::new("User Guide").shortcut_text("F1").selected(self.app_help.open))
                    .on_hover_text("How to use Varstotas: tools, menus, files, keys (F1 over the model: the active tool)")
                    .clicked()
                {
                    self.open_app_help(crate::ui::app_help::Topic::Start);
                    ui.close();
                }
                if ui.add(egui::Button::new("Script Reference").selected(self.script_help.open)).on_hover_text("Scripts, generators and component scripts: functions, properties, examples").clicked() {
                    self.open_script_help(crate::ui::script_help::Topic::Basics, None);
                    ui.close();
                }
                ui.separator();
                self.item(ui, "About Varstotas", None, true, Action::About);
            });
        });
    }

    /// View → Snapping and the magnet button's menu: the master switch and every inference.
    pub(crate) fn snapping_menu(&mut self, ui: &mut Ui) {
        use crate::inference::settings::GROUPS;
        let mut on = self.prefs.snapping.enabled;
        if icon_check(ui, &mut on, Icon::Magnet, "Snapping").on_hover_text(crate::keymap::label(crate::keymap::KeyCmd::ToggleSnapping).unwrap_or_default()).clicked() {
            self.actions.push(Action::ToggleSnapping);
        }
        ui.horizontal(|ui| {
            if ui.small_button("All On").clicked() {
                self.prefs.snapping.off.clear();
            }
            if ui.small_button("All Off").clicked() {
                for k in crate::inference::settings::all_kinds() {
                    self.prefs.snapping.set(k, false);
                }
            }
        });
        for (group, kinds) in GROUPS {
            ui.separator();
            ui.label(egui::RichText::new(group).small().weak());
            if group == "Grid" {
                let mut grid = self.prefs.grid.show;
                if icon_check(ui, &mut grid, Icon::Grid, "Show Grid").clicked() {
                    self.actions.push(Action::ToggleGrid);
                }
            }
            ui.add_enabled_ui(self.prefs.snapping.enabled, |ui| {
                for &k in kinds {
                    let mut v = self.prefs.snapping.is_on(k);
                    let r = icon_check(ui, &mut v, Icon::Snap(k), k.menu_label());
                    super::tray_lib::spot(format!("snap:{}", k.menu_label()), r.rect.center());
                    if r.changed() {
                        self.prefs.snapping.set(k, v);
                    }
                }
            });
        }        // Increments: round free moves, angles and scale factors to a step.
        ui.separator();
        ui.label(egui::RichText::new("Increments").small().weak());
        let unit = self.doc.model.unit;
        ui.add_enabled_ui(self.prefs.snapping.enabled, |ui| {
            let inc = &mut self.prefs.snapping.increments;
            egui::Grid::new("snap_increments").num_columns(2).spacing([8.0, 2.0]).show(ui, |ui| {
                ui.checkbox(&mut inc.move_on, "Move");
                ui.add(
                    egui::DragValue::new(&mut inc.move_step)
                        .range(0.001..=1.0e6)
                        .speed(1.0)
                        .custom_formatter(move |v, _| varstotas_kernel::units::format_length(v, unit))
                        .custom_parser(move |t| varstotas_kernel::units::parse_length(t, unit)),
                );
                ui.end_row();
                ui.checkbox(&mut inc.rotate_on, "Rotate");
                ui.add(egui::DragValue::new(&mut inc.angle_step).range(0.01..=180.0).speed(0.5).suffix("°"));
                ui.end_row();
                ui.checkbox(&mut inc.scale_on, "Scale");
                ui.add(egui::DragValue::new(&mut inc.scale_step).range(0.001..=10.0).speed(0.01).max_decimals(3));
                ui.end_row();
            });
        });
    }
}
