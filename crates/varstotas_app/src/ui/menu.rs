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
        let has_sel = self.doc.has_selection();
        self.clipboard_items(ui);
        self.item(ui, "Erase", Some("Delete"), has_sel, Action::Delete);
        ui.separator();
        self.item(ui, "Hide", None, !self.doc.selection.is_empty(), Action::HideSelected);
        self.item(ui, "Make Group", Some("G"), !self.doc.selection.is_empty(), Action::MakeGroup);
        self.item(ui, "Make Component…", None, !self.doc.selection.is_empty(), Action::MakeComponent);
        let has_inst = self.doc.selection.iter().any(|e| matches!(e, EntityRef::Instance(_)));
        self.item(ui, "Explode", None, has_inst, Action::Explode);
        ui.separator();
        self.face_ops_items(ui);
        if !self.doc.active_path.is_empty() {
            ui.separator();
            self.item(ui, "Close Group", Some("Esc"), true, Action::ExitContext);
        }
    }

    pub(crate) fn menu_bar(&mut self, ui: &mut Ui) {
        egui::MenuBar::new().ui(ui, |ui| {
            ui.menu_button("File", |ui| {
                self.item(ui, "New", Some(&cmd("N")), true, Action::New);
                ui.menu_button("New from Template", |ui| {
                    for t in Template::ALL {
                        let b = Button::new(t.name()).selected(self.prefs.template == Some(t));
                        if ui.add(b).on_hover_text(t.description()).clicked() {
                            self.actions.push(Action::NewFromTemplate(t));
                        }
                    }
                });
                self.item(ui, "Open…", Some(&cmd("O")), true, Action::Open);
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
                self.item(ui, "Save", Some(&cmd("S")), true, Action::Save);
                self.item(ui, "Save As…", Some(&cmd_shift("S")), true, Action::SaveAs);
                ui.separator();
                ui.menu_button("Import", |ui| {
                    self.item(ui, "OBJ…", None, true, Action::ImportObj);
                    self.item(ui, "STL…", None, true, Action::ImportStl);
                });
                ui.menu_button("Export", |ui| {
                    self.item(ui, "2D Graphic (PNG)…", None, true, Action::ExportImage);
                    ui.separator();
                    self.item(ui, "OBJ…", None, true, Action::ExportObj);
                    self.item(ui, "STL…", None, true, Action::ExportStl);
                });
                ui.separator();
                self.item(ui, "Insert Demo Geometry", None, true, Action::InsertDemo);
                ui.separator();
                self.item(ui, "Quit", Some(&cmd("Q")), true, Action::Quit);
            });

            ui.menu_button("Edit", |ui| {
                let undo = self.doc.history.undo_label().map(|l| format!("Undo {l}"));
                let redo = self.doc.history.redo_label().map(|l| format!("Redo {l}"));
                self.item(ui, undo.as_deref().unwrap_or("Undo"), Some(&cmd("Z")), undo.is_some(), Action::Undo);
                self.item(ui, redo.as_deref().unwrap_or("Redo"), Some(&cmd_shift("Z")), redo.is_some(), Action::Redo);
                ui.separator();
                let any_sel = self.doc.has_selection();
                let has_sel = !self.doc.selection.is_empty();
                self.clipboard_items(ui);
                self.item(ui, "Delete", Some("Delete"), any_sel, Action::Delete);
                let has_guides = self.doc.model.has_any_guides();
                self.item(ui, "Delete Guides", None, has_guides, Action::DeleteGuides);
                ui.separator();
                self.item(ui, "Select All", Some(&cmd("A")), true, Action::SelectAll);
                self.item(ui, "Select None", Some(&cmd("T")), any_sel, Action::SelectNone);
                ui.separator();
                self.item(ui, "Hide", None, has_sel, Action::HideSelected);
                self.item(ui, "Unhide All", None, true, Action::UnhideAll);
                ui.separator();
                self.item(ui, "Make Group", Some("G"), has_sel, Action::MakeGroup);
                self.item(ui, "Make Component…", None, has_sel, Action::MakeComponent);
                let has_inst = self.doc.selection.iter().any(|e| matches!(e, varstotas_kernel::EntityRef::Instance(_)));
                self.item(ui, "Explode", None, has_inst, Action::Explode);
                self.item(ui, "Close Group", Some("Esc"), !self.doc.active_path.is_empty(), Action::ExitContext);
                ui.separator();
                self.face_ops_items(ui);
            });

            ui.menu_button("View", |ui| {
                ui.label(egui::RichText::new("Face Style").small().weak());
                let fs = &mut self.prefs.face_style;
                ui.radio_value(fs, FaceStylePref::Shaded, "Shaded with Textures");
                ui.radio_value(fs, FaceStylePref::Monochrome, "Monochrome");
                ui.radio_value(fs, FaceStylePref::HiddenLine, "Hidden Line");
                ui.radio_value(fs, FaceStylePref::Wireframe, "Wireframe");
                ui.checkbox(&mut self.prefs.xray, "X-Ray");
                ui.separator();
                ui.label(egui::RichText::new("Edge Style").small().weak());
                ui.checkbox(&mut self.prefs.show_edges, "Edges");
                ui.checkbox(&mut self.prefs.profiles, "Profiles");
                ui.separator();
                let mut guides = !self.doc.model.guides_hidden;
                if ui.checkbox(&mut guides, "Guides").clicked() {
                    self.actions.push(Action::ToggleGuides);
                }
                ui.checkbox(&mut self.prefs.show_axes, "Axes");
                ui.checkbox(&mut self.prefs.show_ground, "Ground");
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
                self.item(ui, "Zoom Extents", Some("Shift+Z"), true, Action::ZoomExtents);
            });

            ui.menu_button("Draw", |ui| {
                for id in [ToolId::Line, ToolId::Arc, ToolId::Rectangle, ToolId::Circle, ToolId::Polygon] {
                    self.tool_item(ui, id);
                }
            });

            ui.menu_button("Tools", |ui| {
                for id in [ToolId::Select, ToolId::Eraser, ToolId::Paint] {
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
                ] {
                    self.tool_item(ui, id);
                }
                ui.separator();
                for id in [ToolId::Tape, ToolId::Protractor] {
                    self.tool_item(ui, id);
                }
            });

            ui.menu_button("Window", |ui| {
                let mut open = self.prefs.tray_open;
                if ui.checkbox(&mut open, "Default Tray").clicked() {
                    self.actions.push(Action::ToggleTray);
                }
                ui.separator();
                self.item(ui, "Preferences…", None, true, Action::Preferences);
            });

            ui.menu_button("Help", |ui| {
                self.item(ui, "About Varstotas", None, true, Action::About);
            });
        });
    }
}
