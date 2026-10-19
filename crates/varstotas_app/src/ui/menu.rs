//! Menu bar: File Edit View Camera Draw Tools Window Help.

use crate::app::{Action, FaceStylePref, VarstotasApp};
use crate::templates::Template;
use varstotas_kernel::{EntityRef, IntersectScope};
use crate::tools::ToolId;
use egui::{Button, Ui};
use varstotas_render::StandardView;

fn cmd(key: &str) -> String {
    if cfg!(target_os = "macos") { format!("⌘{key}") } else { format!("Ctrl+{key}") }
}

fn cmd_shift(key: &str) -> String {
    if cfg!(target_os = "macos") { format!("⇧⌘{key}") } else { format!("Ctrl+Shift+{key}") }
}

impl VarstotasApp {
    fn item(&mut self, ui: &mut Ui, label: &str, shortcut: Option<&str>, enabled: bool, action: Action) {
        let mut b = Button::new(label);
        if let Some(s) = shortcut {
            b = b.shortcut_text(s);
        }
        if ui.add_enabled(enabled, b).clicked() {
            self.actions.push(action);
        }
    }

    fn tool_item(&mut self, ui: &mut Ui, id: ToolId) {
        let sel = self.tool_id() == id;
        let mut b = Button::new(id.name()).selected(sel);
        if let Some(s) = id.shortcut_label() {
            b = b.shortcut_text(s);
        }
        if ui.add(b).clicked() {
            self.actions.push(Action::SetTool(id));
        }
    }

    /// Intersect / Reverse / Orient / Flip entries (Edit menu and the viewport context menu).
    fn face_ops_items(&mut self, ui: &mut Ui) {
        let sel = &self.doc.selection;
        let has_sel = !sel.is_empty();
        let has_faces = sel.iter().any(|e| matches!(e, EntityRef::Face(_)));
        let can_intersect = sel.iter().any(|e| matches!(e, EntityRef::Face(_) | EntityRef::Instance(_)));
        ui.add_enabled_ui(can_intersect, |ui| {
            ui.menu_button("Intersect Faces", |ui| {
                self.item(ui, "With Model", None, true, Action::IntersectFaces(IntersectScope::Model));
                self.item(ui, "With Selection", None, true, Action::IntersectFaces(IntersectScope::Selection));
                self.item(ui, "With Context", None, true, Action::IntersectFaces(IntersectScope::Context));
            });
        });
        self.item(ui, "Reverse Faces", None, has_faces, Action::ReverseFaces);
        self.item(ui, "Orient Faces", None, has_faces, Action::OrientFaces);
        ui.add_enabled_ui(has_sel, |ui| {
            ui.menu_button("Flip Along", |ui| {
                self.item(ui, "Red Direction", None, true, Action::FlipAlong(0));
                self.item(ui, "Green Direction", None, true, Action::FlipAlong(1));
                self.item(ui, "Blue Direction", None, true, Action::FlipAlong(2));
            });
        });
    }

    fn clipboard_items(&mut self, ui: &mut Ui) {
        let has_sel = self.doc.has_selection();
        let has_clip = self.clipboard.is_some();
        self.item(ui, "Cut", Some(&cmd("X")), has_sel, Action::Cut);
        self.item(ui, "Copy", Some(&cmd("C")), has_sel, Action::Copy);
        self.item(ui, "Paste", Some(&cmd("V")), has_clip, Action::Paste);
        self.item(ui, "Paste in Place", Some(&cmd_shift("V")), has_clip, Action::PasteInPlace);
    }

    /// Right-click menu of the 3D view.
    pub(crate) fn viewport_context_menu(&mut self, ui: &mut Ui) {
        if self.ctx_on_axes {
            self.item(ui, "Place", None, true, Action::PlaceAxes);
            self.item(ui, "Move…", None, true, Action::MoveAxes);
            self.item(ui, "Reset", None, self.doc.model.axes.is_some(), Action::ResetAxes);
            return;
        }
        let has_sel = self.doc.has_selection();
        self.clipboard_items(ui);
        self.item(ui, "Erase", Some("Delete"), has_sel, Action::Delete);
        ui.separator();
        self.item(ui, "Hide", None, !self.doc.selection.is_empty(), Action::HideSelected);
        self.item(ui, "Make Group", crate::keymap::label(crate::keymap::KeyCmd::MakeGroup).as_deref(), has_sel, Action::MakeGroup);
        self.item(ui, "Make Component…", None, has_sel, Action::MakeComponent);
        let has_inst = self.doc.selection.iter().any(|e| matches!(e, EntityRef::Instance(_)));
        let one_plain = matches!(self.doc.selection.as_slice(), [varstotas_kernel::EntityRef::Instance(i)]
            if self.doc.model.instance_params(&self.doc.active_path, *i).is_none());
        self.item(ui, "Make Parametric", None, one_plain, Action::MakeParametric);
        self.item(ui, "Explode", None, has_inst, Action::Explode);
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
        self.item(ui, "Unfold…", None, can_unfold, Action::Unfold);
        if self.selected_textured_face() {
            ui.menu_button("Texture", |ui| {
                self.item(ui, "Position", None, true, Action::SetTool(ToolId::TexturePosition));
            });
        }
        if !self.doc.active_path.is_empty() {
            ui.separator();
            self.item(ui, "Close Group", Some("Esc"), true, Action::ExitContext);
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
                self.item(ui, "New", crate::keymap::label(crate::keymap::KeyCmd::New).as_deref(), true, Action::New);
                ui.menu_button("New from Template", |ui| {
                    for t in Template::ALL {
                        let b = Button::new(t.name()).selected(self.prefs.template == Some(t));
                        if ui.add(b).on_hover_text(t.description()).clicked() {
                            self.actions.push(Action::NewFromTemplate(t));
                        }
                    }
                });
                self.item(ui, "Open…", crate::keymap::label(crate::keymap::KeyCmd::Open).as_deref(), true, Action::Open);
                let recent = self.prefs.recent.clone();
                ui.add_enabled_ui(!recent.is_empty(), |ui| {
                    ui.menu_button("Open Recent", |ui| {
                        for p in recent {
                            let name = p.file_name().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
                            if ui.button(name).on_hover_text(p.display().to_string()).clicked() {
                                self.actions.push(Action::OpenPath(p));
                            }
                        }
                    });
                });
                ui.separator();
                self.item(ui, "Save", crate::keymap::label(crate::keymap::KeyCmd::Save).as_deref(), true, Action::Save);
                self.item(ui, "Save As…", crate::keymap::label(crate::keymap::KeyCmd::SaveAs).as_deref(), true, Action::SaveAs);
                ui.menu_button("Preview Image", |ui| {
                    self.item(ui, "Show…", None, true, Action::PreviewShow);
                    ui.separator();
                    self.item(ui, "From Image File…", None, true, Action::PreviewFromFile);
                    self.item(ui, "From Current View", None, true, Action::PreviewFromView);
                    self.item(ui, "Remove", None, self.doc.model.preview.is_some(), Action::PreviewRemove);
                })
                .response
                .on_hover_text("The picture the Components library shows for this file (else it draws the model)");
                ui.separator();
                ui.menu_button("Import", |ui| {
                    self.item(ui, "OBJ…", None, true, Action::ImportObj);
                    self.item(ui, "STL…", None, true, Action::ImportStl);
                    self.item(ui, "DXF…", None, true, Action::ImportDxf);
                    self.item(ui, "glTF / GLB…", None, true, Action::ImportGltf);
                    self.item(ui, "3MF…", None, true, Action::ImportAs("3mf"));
                    self.item(ui, "COLLADA (DAE)…", None, true, Action::ImportAs("dae"));
                    self.item(ui, "FBX…", None, true, Action::ImportAs("fbx"));
                    self.item(ui, "PLY…", None, true, Action::ImportAs("ply"));
                    self.item(ui, "Points (CSV)…", None, true, Action::ImportAs("csv"));
                    self.plugin_file_items(ui, false);
                    self.model_import_items(ui);
                });
                ui.menu_button("Export", |ui| {
                    self.item(ui, "2D Graphic (PNG)…", None, true, Action::ExportImage);
                    self.item(ui, "Animation (Video / PNG)…", None, true, Action::ExportAnimation);
                    ui.separator();
                    self.item(ui, "OBJ…", None, true, Action::ExportObj);
                    self.item(ui, "STL…", None, true, Action::ExportStl);
                    self.item(ui, "DXF…", None, true, Action::ExportDxf);
                    self.item(ui, "glTF (GLB)…", None, true, Action::ExportGlb);
                    self.item(ui, "3MF (3D printing)…", None, true, Action::Export3mf);
                    self.item(ui, "PLY…", None, true, Action::ExportPly);
                    self.item(ui, "2D Drawing (SVG / PDF)…", None, true, Action::ExportDrawing);
                    self.item(ui, "Unfold (SVG / PDF / DXF)…", None, true, Action::Unfold);
                    self.item(ui, "Sheets (PDF / SVG)…", None, true, Action::Sheets);
                    ui.separator();
                    self.item(ui, "Report: Parts and Areas (CSV)…", None, true, Action::ExportReport);
                    self.item(ui, "Faces (JSON)…", None, true, Action::ExportFacesJson);
                    self.item(ui, "Attributions (credits of downloaded models)…", None, true, Action::ExportAttributions);
                    self.plugin_file_items(ui, true);
                });
                ui.separator();
                self.item(ui, "Run Script…", None, true, Action::RunScript);
                ui.separator();
                self.item(ui, "Quit", crate::keymap::label(crate::keymap::KeyCmd::Quit).as_deref(), true, Action::Quit);
            });

            ui.menu_button("Edit", |ui| {
                let undo = self.doc.history.undo_label().map(|l| format!("Undo {l}"));
                let redo = self.doc.history.redo_label().map(|l| format!("Redo {l}"));
                self.item(ui, undo.as_deref().unwrap_or("Undo"), crate::keymap::label(crate::keymap::KeyCmd::Undo).as_deref(), undo.is_some(), Action::Undo);
                self.item(ui, redo.as_deref().unwrap_or("Redo"), crate::keymap::label(crate::keymap::KeyCmd::Redo).as_deref(), redo.is_some(), Action::Redo);
                ui.separator();
                let any_sel = self.doc.has_selection();
                let has_sel = !self.doc.selection.is_empty();
                self.clipboard_items(ui);
                self.item(ui, "Delete", Some("Delete"), any_sel, Action::Delete);
                self.item(ui, "Purge Unused", None, true, Action::PurgeUnused);
                let has_guides = self.doc.model.has_any_guides();
                self.item(ui, "Delete Guides", None, has_guides, Action::DeleteGuides);
                ui.separator();
                self.item(ui, "Select All", crate::keymap::label(crate::keymap::KeyCmd::SelectAll).as_deref(), true, Action::SelectAll);
                self.item(ui, "Select None", crate::keymap::label(crate::keymap::KeyCmd::SelectNone).as_deref(), any_sel, Action::SelectNone);
                ui.separator();
                self.item(ui, "Hide", None, has_sel, Action::HideSelected);
                self.item(ui, "Unhide All", None, true, Action::UnhideAll);
                ui.separator();
                self.item(ui, "Make Group", crate::keymap::label(crate::keymap::KeyCmd::MakeGroup).as_deref(), has_sel, Action::MakeGroup);
                self.item(ui, "Make Component…", None, has_sel, Action::MakeComponent);
                let has_inst = self.doc.selection.iter().any(|e| matches!(e, varstotas_kernel::EntityRef::Instance(_)));
                let one_plain = matches!(self.doc.selection.as_slice(), [varstotas_kernel::EntityRef::Instance(i)]
                    if self.doc.model.instance_params(&self.doc.active_path, *i).is_none());
                self.item(ui, "Make Parametric", None, one_plain, Action::MakeParametric);
                self.item(ui, "Explode", None, has_inst, Action::Explode);
                self.item(ui, "Close Group", Some("Esc"), !self.doc.active_path.is_empty(), Action::ExitContext);
                ui.separator();
                self.face_ops_items(ui);
            });

            ui.menu_button("View", |ui| {
                ui.label(egui::RichText::new("Face Style").small().weak());
                let fs = &mut self.prefs.face_style;
                ui.radio_value(fs, FaceStylePref::ShadedColor, "Shaded");
                ui.radio_value(fs, FaceStylePref::Shaded, "Shaded with Textures");
                ui.radio_value(fs, FaceStylePref::Monochrome, "Monochrome");
                ui.radio_value(fs, FaceStylePref::HiddenLine, "Hidden Line");
                ui.radio_value(fs, FaceStylePref::Wireframe, "Wireframe");
                ui.radio_value(fs, FaceStylePref::Realistic, "Realistic");
                ui.checkbox(&mut self.prefs.xray, "X-Ray");
                ui.separator();
                ui.label(egui::RichText::new("Edge Style").small().weak());
                ui.checkbox(self.prefs.edges_mut(), "Edges").on_hover_text("Edges in this face style (Realistic has its own switch)");
                ui.checkbox(&mut self.prefs.profiles, "Profiles");
                ui.separator();
                let mut guides = !self.doc.model.guides_hidden;
                if ui.checkbox(&mut guides, "Guides").clicked() {
                    self.actions.push(Action::ToggleGuides);
                }
                ui.checkbox(&mut self.prefs.show_axes, "Axes");
                ui.checkbox(&mut self.prefs.show_ground, "Ground");
                ui.checkbox(&mut self.prefs.show_sky, "Sky").on_hover_text("The sky gradient behind the model; off = a plain light background");
                let mut grid = self.prefs.grid.show;
                if ui.checkbox(&mut grid, "Grid").clicked() {
                    self.actions.push(Action::ToggleGrid);
                }
                ui.separator();
                let mut shadows = self.doc.model.shadows.enabled;
                if ui.checkbox(&mut shadows, "Shadows").clicked() {
                    self.actions.push(Action::ToggleShadows);
                }
                ui.separator();
                let d = self.doc.model.section_display;
                for (k, mut on, label) in [(0u8, d.planes, "Section Planes"), (1, d.cuts, "Section Cuts"), (2, d.fill, "Section Fill")] {
                    if ui.checkbox(&mut on, label).clicked() {
                        self.actions.push(Action::ToggleSectionDisplay(k));
                    }
                }
                ui.separator();
                ui.menu_button("Viewports", |ui| {
                    for (n, label) in [(1, "One"), (2, "Two (side by side)"), (4, "Four (2 × 2)")] {
                        if ui.radio(self.pane_count == n, label).clicked() {
                            self.actions.push(Action::SetPanes(n));
                            ui.close();
                        }
                    }
                    ui.separator();
                    ui.label(egui::RichText::new("Presets").small().weak());
                    for (i, p) in crate::panes::PRESETS.iter().enumerate() {
                        if ui.button(p.name).clicked() {
                            self.actions.push(Action::ViewPreset(i));
                            ui.close();
                        }
                    }
                });
                ui.separator();
                ui.menu_button("Animation", |ui| {
                    let has = !self.doc.model.scenes.is_empty();
                    let play = if self.scene_player.is_some() { "Pause" } else { "Play" };
                    self.item(ui, play, None, has, Action::ToggleAnimation);
                    ui.separator();
                    self.item(ui, "Next Scene", crate::keymap::label(crate::keymap::KeyCmd::NextScene).as_deref(), has, Action::NextScene);
                    self.item(ui, "Previous Scene", crate::keymap::label(crate::keymap::KeyCmd::PrevScene).as_deref(), has, Action::PrevScene);
                    ui.separator();
                    self.item(ui, "Add Scene", None, true, Action::AddScene);
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
                        self.item(ui, name, None, true, Action::View(v));
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
                self.item(ui, "Zoom Extents", crate::keymap::label(crate::keymap::KeyCmd::ZoomExtents).as_deref(), true, Action::ZoomExtents);
                ui.separator();
                self.tool_item(ui, ToolId::PositionCamera);
                self.tool_item(ui, ToolId::LookAround);
                self.tool_item(ui, ToolId::Walk);
            });

            ui.menu_button("Draw", |ui| {
                self.tool_item(ui, ToolId::Line);
                ui.separator();
                for id in [ToolId::ArcCenter, ToolId::Arc, ToolId::Arc3, ToolId::Pie, ToolId::Bezier] {
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
                ui.menu_button("Solid Tools", |ui| {
                    for op in varstotas_kernel::SolidOp::ALL {
                        self.item(ui, op.name(), None, true, Action::SolidTool(op));
                    }
                });
                self.item(ui, "Unfold…", None, true, Action::Unfold);
                ui.separator();
                for id in [ToolId::Tape, ToolId::Dimension, ToolId::Protractor, ToolId::Axes, ToolId::Text] {
                    self.tool_item(ui, id);
                }
                self.item(ui, "3D Text…", None, true, Action::Text3d);
            });

            ui.menu_button("Snapping", |ui| self.snapping_menu(ui));

            ui.menu_button("Extensions", |ui| self.extensions_menu(ui));

            // Stays open on clicks so several tray cards can be toggled; buttons close it.
            let keep = egui::containers::menu::MenuConfig::new().close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside);
            egui::containers::menu::MenuButton::new("Window").config(keep).ui(ui, |ui| {
                let mut open = self.prefs.tray_open;
                if ui.checkbox(&mut open, "Side Bar").clicked() {
                    self.actions.push(Action::ToggleTray);
                }
                // The tray's cards: a checkbox each (turning one on opens the tray).
                ui.indent("tray_cards", |ui| {
                    for c in crate::ui::tray::TRAY_CARDS {
                        let mut shown = !self.prefs.tray_hidden.iter().any(|h| h == c);
                        if ui.checkbox(&mut shown, c).clicked() {
                            if shown {
                                self.prefs.tray_hidden.retain(|h| h != c);
                                if !self.prefs.tray_open {
                                    self.actions.push(Action::ToggleTray);
                                }
                            } else {
                                self.prefs.tray_hidden.push(c.to_string());
                            }
                        }
                    }
                });
                ui.separator();
                if ui.add(egui::Button::new("Sheets…").selected(self.sheets.open)).clicked() {
                    self.actions.push(Action::Sheets);
                    ui.close();
                }
                if ui.add(egui::Button::new("Render…").selected(self.render_window.open)).clicked() {
                    self.render_window.open = !self.render_window.open;
                    ui.close();
                }
                if ui.add(egui::Button::new("Work Planes…").selected(self.planes_window.open)).clicked() {
                    self.actions.push(Action::WorkPlanes);
                    ui.close();
                }
                if ui.add(egui::Button::new("Model Info…").selected(self.model_info.open)).on_hover_text("Statistics of the document, Purge Unused").clicked() {
                    self.actions.push(Action::ModelInfo);
                    ui.close();
                }
                if ui
                    .add(egui::Button::new("Component Parameters…").selected(self.rig_editor.open))
                    .on_hover_text("Parameters of the selected component: values that turn, slide, show or paint its parts")
                    .clicked()
                {
                    self.actions.push(Action::RigEditor);
                    ui.close();
                }
                if ui.add(egui::Button::new("3D Library…").selected(self.models.open)).clicked() {
                    self.actions.push(Action::ModelLibrary);
                    ui.close();
                }
                if ui.add(egui::Button::new("Generator Editor…").selected(self.gen_editor.open)).clicked() {
                    self.actions.push(Action::GeneratorEditor);
                    ui.close();
                }
                let b = egui::Button::new("Script Console").selected(self.console.open);
                if ui.add(b).clicked() {
                    self.actions.push(Action::ToggleConsole);
                    ui.close();
                }
                if ui.add(egui::Button::new("Log").selected(self.log_window.open)).on_hover_text("Errors and warnings from everywhere: scripts, import / export, libraries, downloads …").clicked() {
                    self.toggle_log(!self.log_window.open);
                    ui.close();
                }
                ui.separator();
                if ui.button("Preferences…").clicked() {
                    self.actions.push(Action::Preferences);
                    ui.close();
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
        if ui.add(egui::Checkbox::new(&mut on, "Snapping")).on_hover_text(crate::keymap::label(crate::keymap::KeyCmd::ToggleSnapping).unwrap_or_default()).clicked() {
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
                if ui.checkbox(&mut grid, "Show Grid").clicked() {
                    self.actions.push(Action::ToggleGrid);
                }
            }
            ui.add_enabled_ui(self.prefs.snapping.enabled, |ui| {
                for &k in kinds {
                    let mut v = self.prefs.snapping.is_on(k);
                    let r = ui.checkbox(&mut v, k.menu_label());
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
