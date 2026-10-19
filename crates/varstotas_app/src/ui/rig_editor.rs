//! Parameters of ordinary components (rigs, see `varstotas_kernel::rig`): the values of selected
//! copies in Entity Info, and Window → Component Parameters — the parameters, formulas, the
//! parts' joints (turn / slide / show / paint, driven by a parameter or an expression) and the
//! script `update(p)` of a component.

use super::code_editor::ScriptEditor;
use super::generators::show_value;
use super::theme;
use crate::app::{Action, VarstotasApp};
use egui::{Color32, Context, RichText, Ui, vec2};
use varstotas_kernel::glam::{DMat4, DVec3};
use varstotas_kernel::{DefId, EntityRef, GenFormula, GenKind, GenParam, GenValue, GenValues, Generator, InstanceId, Joint, JointKind, Rig};

const KINDS: [&str; 8] = ["length", "angle", "number", "int", "percent", "bool", "choice", "material"];
const FORMULA_KINDS: [&str; 7] = ["length", "angle", "number", "int", "percent", "bool", "text"];
const MOTIONS: [&str; 5] = ["Rotate", "Slide", "Show / hide", "Paint", "Set parameter"];

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ParamRow {
    pub key: String,
    pub label: String,
    pub kind: usize,
    pub min: String,
    pub max: String,
    pub step: String,
    pub default: String,
    /// Choice options, `a|b|c`.
    pub choices: String,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct FormulaRow {
    pub key: String,
    pub label: String,
    pub kind: usize,
    pub expr: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct JointRow {
    pub part: String,
    pub motion: usize,
    /// Driving parameter / formula ("" = the expression).
    pub param: String,
    pub from: f64,
    pub to: f64,
    pub expr: String,
    /// Rotate: a point on the axis; slide: unused.
    pub origin: [f64; 3],
    /// Rotate: the axis; slide: the direction.
    pub axis: [f64; 3],
    /// Set parameter: the part's parameter.
    pub key: String,
}

impl Default for JointRow {
    fn default() -> Self {
        Self { part: String::new(), motion: 0, param: String::new(), from: 0.0, to: 90.0, expr: String::new(), origin: [0.0; 3], axis: [0.0, 0.0, 1.0], key: String::new() }
    }
}

pub struct RigEditor {
    pub open: bool,
    /// The component edited (its master, or a plain component getting parameters).
    pub def: Option<DefId>,
    /// World transform of the copy the editor was opened from (for picking axes).
    pub xf: Option<DMat4>,
    pub params: Vec<ParamRow>,
    pub formulas: Vec<FormulaRow>,
    pub joints: Vec<JointRow>,
    pub script: String,
    pub error: Option<String>,
    /// The joint whose axis is being picked.
    pub pick: Option<usize>,
    editor: ScriptEditor,
    /// The rows as loaded (Revert, unsaved changes).
    loaded: Option<Rig>,
}

impl Default for RigEditor {
    fn default() -> Self {
        Self {
            open: false,
            def: None,
            xf: None,
            params: Vec::new(),
            formulas: Vec::new(),
            joints: Vec::new(),
            script: String::new(),
            error: None,
            pick: None,
            editor: ScriptEditor::with_id("rig_script"),
            loaded: None,
        }
    }
}

/// A number for a text field: `12.5`, `90`, "" for none.
fn num_text(v: Option<f64>) -> String {
    v.map(|x| format!("{}", (x * 1e6).round() / 1e6)).unwrap_or_default()
}

fn kind_index(k: &GenKind) -> usize {
    match k {
        GenKind::Length => 0,
        GenKind::Angle => 1,
        GenKind::Number => 2,
        GenKind::Int => 3,
        GenKind::Percent => 4,
        GenKind::Bool => 5,
        GenKind::Choice(_) => 6,
        GenKind::Material => 7,
    }
}

impl ParamRow {
    pub(crate) fn of(p: &GenParam) -> Self {
        let default = match &p.default {
            GenValue::Num(x) => num_text(Some(*x)),
            GenValue::Bool(b) => b.to_string(),
            GenValue::Text(s) => s.clone(),
        };
        let choices = match &p.kind {
            GenKind::Choice(o) => o.join("|"),
            _ => String::new(),
        };
        Self { key: p.key.clone(), label: p.label.clone(), kind: kind_index(&p.kind), min: num_text(p.min), max: num_text(p.max), step: num_text(p.step), default, choices }
    }

    /// The `@param` header text of the row.
    fn header(&self) -> String {
        let kind = KINDS[self.kind];
        let label = if self.label.trim().is_empty() { String::new() } else { format!(" \"{}\"", self.label.trim()) };
        let decl = match kind {
            "choice" => format!("{}:choice({})", self.key.trim(), self.choices.trim()),
            "bool" | "material" => format!("{}:{kind}", self.key.trim()),
            _ if self.min.trim().is_empty() && self.max.trim().is_empty() && self.step.trim().is_empty() => format!("{}:{kind}", self.key.trim()),
            _ => {
                let step = if self.step.trim().is_empty() { String::new() } else { format!("/{}", self.step.trim()) };
                format!("{}:{kind}({}..{}{step})", self.key.trim(), self.min.trim(), self.max.trim())
            }
        };
        let d = self.default.trim();
        let default = match kind {
            "material" => format!("\"{d}\""),
            "bool" if d.is_empty() => "false".into(),
            "choice" if d.is_empty() => self.choices.split('|').next().unwrap_or("").trim().to_string(),
            _ if d.is_empty() => "0".into(),
            _ => d.to_string(),
        };
        format!("{decl} = {default}{label}")
    }
}

impl FormulaRow {
    pub(crate) fn of(f: &GenFormula) -> Self {
        let kind = match f.kind {
            GenKind::Material => 6,
            ref k => kind_index(k).min(5),
        };
        Self { key: f.key.clone(), label: f.label.clone(), kind, expr: f.expr.clone() }
    }

    fn header(&self) -> String {
        let label = if self.label.trim().is_empty() { String::new() } else { format!(" \"{}\"", self.label.trim()) };
        format!("{}:{} = {}{label}", self.key.trim(), FORMULA_KINDS[self.kind], self.expr.trim())
    }
}

impl JointRow {
    fn of(j: &Joint) -> Self {
        let (motion, origin, axis) = match &j.kind {
            JointKind::Rotate { origin, axis } => (0, *origin, *axis),
            JointKind::Slide { dir } => (1, DVec3::ZERO, *dir),
            JointKind::Visible => (2, DVec3::ZERO, DVec3::Z),
            JointKind::Material => (3, DVec3::ZERO, DVec3::Z),
            JointKind::Param { .. } => (4, DVec3::ZERO, DVec3::Z),
        };
        let key = if let JointKind::Param { key } = &j.kind { key.clone() } else { String::new() };
        Self { part: j.part.clone(), motion, param: j.param.clone(), from: j.from, to: j.to, expr: j.expr.clone(), origin: origin.to_array(), axis: axis.to_array(), key }
    }

    fn joint(&self) -> Joint {
        let kind = match self.motion {
            0 => JointKind::Rotate { origin: DVec3::from_array(self.origin), axis: DVec3::from_array(self.axis) },
            1 => JointKind::Slide { dir: DVec3::from_array(self.axis) },
            2 => JointKind::Visible,
            3 => JointKind::Material,
            _ => JointKind::Param { key: self.key.trim().to_string() },
        };
        let param = if self.expr.trim().is_empty() { self.param.clone() } else { String::new() };
        Joint { part: self.part.clone(), kind, param, from: self.from, to: self.to, expr: self.expr.trim().to_string() }
    }
}

impl RigEditor {
    /// Load the rows of `rig` (an empty rig: one parameter to start with).
    fn load(&mut self, def: DefId, rig: Option<&Rig>, xf: Option<DMat4>) {
        self.def = Some(def);
        self.xf = xf;
        self.error = None;
        self.pick = None;
        match rig {
            Some(r) => {
                self.params = r.params.iter().map(ParamRow::of).collect();
                self.formulas = r.formulas.iter().map(FormulaRow::of).collect();
                self.joints = r.joints.iter().map(JointRow::of).collect();
                self.script = r.script.clone();
            }
            None => {
                self.params = vec![ParamRow { key: "open".into(), label: "Opening".into(), kind: 1, min: "0".into(), max: "90".into(), step: "1".into(), default: "0".into(), choices: String::new() }];
                self.formulas.clear();
                self.joints.clear();
                self.script.clear();
            }
        }
        self.loaded = rig.cloned();
        self.open = true;
    }

    /// The rows as a rig (`Err` names the first bad row).
    pub fn rig(&self) -> Result<Rig, String> {
        let params = self.params.iter().map(|r| GenParam::parse(&r.header())).collect::<Result<Vec<_>, _>>()?;
        let formulas = self.formulas.iter().map(|r| GenFormula::parse(&r.header())).collect::<Result<Vec<_>, _>>()?;
        Ok(Rig { params, formulas, joints: self.joints.iter().map(JointRow::joint).collect(), script: self.script.clone() })
    }

    fn changed(&self) -> bool {
        self.rig().ok() != self.loaded
    }
}

impl VarstotasApp {
    /// Selected copies of one component with parameters: (instances, master, its parameters as
    /// a generator, values of each).
    pub(crate) fn selected_rigged(&self) -> Option<(Vec<InstanceId>, DefId, Generator, Vec<GenValues>)> {
        let path = &self.doc.active_path;
        let model = &self.doc.model;
        let mut ids = Vec::new();
        let mut values = Vec::new();
        let mut master = None;
        for e in &self.doc.selection {
            let EntityRef::Instance(i) = e else { return None };
            let (_, r) = model.instance_rigged(path, *i)?;
            if master.is_some_and(|m| m != r.master) {
                return None;
            }
            master = Some(r.master);
            ids.push(*i);
            values.push(r.values.clone());
        }
        let g = varstotas_script::rig::generator(model, master?)?;
        Some((ids, master?, g, values))
    }

    /// Entity Info for selected copies of a component with parameters: a field per parameter,
    /// the formulas, Reset, Make Unique and Edit Parameters…. False when the selection is not
    /// that.
    pub(crate) fn rig_info(&mut self, ui: &mut Ui) -> bool {
        let Some((ids, master, g, values)) = self.selected_rigged() else { return false };
        ui.add_space(6.0);
        let title = if ids.len() > 1 { format!("{} — {} copies", g.name, ids.len()) } else { g.name.clone() };
        ui.label(RichText::new(title).strong());
        let change = if g.params.is_empty() {
            ui.label(RichText::new("No parameters yet").color(theme::TEXT_DIM));
            None
        } else {
            self.value_fields(ui, "entity_info_rig", &ids, &g, &values)
        };
        ui.horizontal_wrapped(|ui| {
            if ui.button("Reset").on_hover_text("Back to the default values").clicked() {
                self.actions.push(Action::SetRigValues(GenValues::new(), true));
            }
            self.make_unique_button(ui, &ids);
            if ui.button("Edit Parameters…").on_hover_text("Window → Component Parameters: parameters, formulas, parts' motions, script").clicked() {
                self.actions.push(Action::RigEditor);
            }
        });
        let parts = self.doc.model.defs.get(master).map_or(0, |d| d.rig.as_ref().map_or(0, |r| r.joints.len()));
        ui.label(RichText::new(format!("Component with parameters ({parts} motion(s))")).small().color(theme::TEXT_DIM));
        if let Some(change) = change {
            self.actions.push(Action::SetRigValues(change, false));
        }
        true
    }

    /// Apply values to every selected copy (one undo step).
    pub(crate) fn set_rig_values(&mut self, values: GenValues, reset: bool) {
        let Some((ids, ..)) = self.selected_rigged() else {
            return self.flash("Select components with parameters");
        };
        let path = self.doc.active_path.clone();
        let label = if reset { "Reset Parameters" } else { "Component Parameters" };
        let r = self.doc.try_commit(label, |m| {
            for id in &ids {
                varstotas_script::rig::set_values(m, &path, *id, &values, reset)?;
            }
            Ok::<_, String>(())
        });
        if r.is_none() {
            let e = self.doc.last_error.take().unwrap_or_default();
            self.flash_error(e);
        }
    }

    /// Open the editor for the selected component (or the open one's, inside it).
    pub(crate) fn open_rig_editor(&mut self) {
        let m = &self.doc.model;
        let path = self.doc.active_path.clone();
        let sel = match self.doc.selection.as_slice() {
            [EntityRef::Instance(i)] => m.mesh_at(&path).and_then(|x| x.instances.get(*i)).map(|x| (x.def, m.transform_at(&path) * x.transform)),
            _ => None,
        };
        // Inside a component: that one.
        let open = || {
            let (last, parent) = path.split_last()?;
            let i = m.mesh_at(parent)?.instances.get(*last)?;
            Some((i.def, m.transform_at(&path)))
        };
        match sel.or_else(open) {
            Some((d, xf)) => self.open_rig_editor_for(d, Some(xf)),
            None => {
                self.rig_editor.open = !self.rig_editor.open;
                if self.rig_editor.open && self.rig_editor.def.is_none() {
                    self.flash("Select a component to give it parameters");
                }
            }
        }
    }

    /// Open the editor for component definition `d` (a copy's world transform `xf` lets it
    /// pick axes in the model).
    pub(crate) fn open_rig_editor_for(&mut self, d: DefId, xf: Option<DMat4>) {
        let m = &self.doc.model;
        let master = m.rig_master(d).unwrap_or(d);
        let Some(def) = m.defs.get(master) else { return };
        if def.is_group {
            return self.flash_error("Groups can't have parameters: make it a component first (Edit → Make Component)".to_string());
        }
        if def.generated.is_some() || def.params.is_some() {
            return self.flash_error("This component is built by a generator or a parametric group".to_string());
        }
        let xf = xf.or_else(|| self.copy_transform(master));
        let rig = def.rig.clone();
        self.rig_editor.load(master, rig.as_ref(), xf);
    }

    /// World transform of a top-level copy of master `master` (or of the plain component).
    fn copy_transform(&self, master: DefId) -> Option<DMat4> {
        let m = &self.doc.model;
        m.root_mesh().instances.values().find(|i| i.def == master || m.defs.get(i.def).and_then(|d| d.rigged.as_ref()).is_some_and(|r| r.master == master && r.paint.is_none())).map(|i| i.transform)
    }

    /// Apply the editor's rig (one undo step).
    pub(crate) fn rig_apply(&mut self) {
        let Some(d) = self.rig_editor.def else { return };
        let rig = match self.rig_editor.rig() {
            Ok(r) => r,
            Err(e) => {
                self.rig_editor.error = Some(e);
                return;
            }
        };
        let label = if self.doc.model.defs.get(d).is_some_and(|x| x.rig.is_some()) { "Edit Parameters" } else { "Add Parameters" };
        let r = self.doc.try_commit(label, |m| varstotas_script::rig::set_rig(m, d, Some(rig.clone())));
        match r {
            Some(master) => {
                self.rig_editor.def = Some(master);
                self.rig_editor.error = None;
                self.rig_editor.loaded = Some(rig);
                self.library.def_thumbs.clear();
                self.tray.gen_text = None;
                self.flash("Parameters applied".to_string());
            }
            None => self.rig_editor.error = self.doc.last_error.take(),
        }
    }

    /// Remove the parameters of the edited component: its copies show it as drawn again.
    pub(crate) fn rig_remove(&mut self) {
        let Some(d) = self.rig_editor.def else { return };
        if self.doc.model.defs.get(d).is_none_or(|x| x.rig.is_none()) {
            self.rig_editor.open = false;
            return;
        }
        if self.doc.try_commit("Remove Parameters", |m| varstotas_script::rig::set_rig(m, d, None)).is_some() {
            self.rig_editor.load(d, None, self.rig_editor.xf);
            self.rig_editor.loaded = None;
            self.flash("Parameters removed".to_string());
        } else {
            self.rig_editor.error = self.doc.last_error.take();
        }
    }

    /// Pick the axis of joint `j` in the model: two points (rotate: a point on the axis and
    /// one along it; slide: from → to).
    pub(crate) fn rig_pick(&mut self, j: usize) {
        if self.rig_editor.xf.is_none() {
            return self.flash_error("Place a copy of the component to pick in the model".to_string());
        }
        self.rig_editor.pick = Some(j);
        // Out of the way while picking (back when done or cancelled).
        self.rig_editor.open = false;
        let prompt = if self.rig_editor.joints.get(j).is_some_and(|r| r.motion == 0) { "Click two points on the axis of rotation" } else { "Click two points: the direction of the slide" };
        self.install_tool(Box::new(crate::tools::plugin_pick::PluginPickTool::new(2, prompt.to_string())));
    }

    /// The picked points of [`Self::rig_pick`] (world): into the component's axes. False when no
    /// pick was waiting.
    pub(crate) fn rig_pick_done(&mut self, pts: &[DVec3]) -> bool {
        let Some(j) = self.rig_editor.pick.take() else { return false };
        let (Some(xf), [a, b, ..]) = (self.rig_editor.xf, pts) else { return true };
        let inv = xf.inverse();
        let (la, lb) = (inv.transform_point3(*a), inv.transform_point3(*b));
        let dir = lb - la;
        if dir.length() < 1e-6 {
            self.flash_error("The two points are the same".to_string());
            return true;
        }
        if let Some(r) = self.rig_editor.joints.get_mut(j) {
            r.origin = la.to_array();
            r.axis = dir.normalize().map(|x| (x * 1e9).round() / 1e9).to_array();
        }
        self.rig_editor.open = true;
        true
    }

    pub(crate) fn rig_editor_window(&mut self, ctx: &Context) {
        // A pick cancelled (another tool chosen): the editor comes back.
        if self.rig_editor.pick.is_some() && self.tool_id() != crate::tools::ToolId::Move {
            self.rig_editor.pick = None;
            self.rig_editor.open = true;
        }
        if !self.rig_editor.open {
            return;
        }
        // A deleted component closes the editor.
        if let Some(d) = self.rig_editor.def
            && !self.doc.model.defs.contains_key(d)
        {
            self.rig_editor.def = None;
        }
        let model = &self.doc.model;
        let def = self.rig_editor.def;
        let name = def.and_then(|d| model.defs.get(d)).map(|d| d.name.clone());
        let has_rig = def.and_then(|d| model.defs.get(d)).is_some_and(|d| d.rig.is_some());
        let parts: Vec<String> = def.map(|d| model.rig_parts(d).into_iter().map(|(_, n)| n).collect()).unwrap_or_default();
        let materials: Vec<String> = model.materials.values().map(|m| m.name.clone()).collect();
        // Parameters of the parts that have them (generated components, components with
        // parameters): part → (key, label).
        let part_params: std::collections::HashMap<String, Vec<(String, String)>> = def
            .map(|d| {
                parts
                    .iter()
                    .filter_map(|p| {
                        let g = varstotas_script::rig::part_generator(model, d, p)?;
                        Some((p.clone(), g.params.iter().map(|x| (x.key.clone(), x.label.clone())).collect()))
                    })
                    .collect()
            })
            .unwrap_or_default();
        let warnings = match (def, self.rig_editor.rig()) {
            (Some(d), Ok(r)) => varstotas_script::rig::warnings(model, d, &r),
            _ => Vec::new(),
        };
        let mut open = true;
        let mut act: Option<&str> = None;
        let mut pick = None;
        let pos = self.viewport.rect.left_top() + vec2(60.0, 40.0);
        let mut win = egui::Window::new("Component Parameters").id(egui::Id::new("rig_editor_window")).open(&mut open).collapsible(false).resizable(true).default_pos(pos).default_width(760.0);
        if self.debug.script_active() {
            win = win.fixed_pos(pos);
        }
        let ed = &mut self.rig_editor;
        win.show(ctx, |ui| {
            let Some(name) = name else {
                ui.label("Select a component (not a group) and choose Window → Component Parameters, or Edit Parameters… in Entity Info.");
                return;
            };
            ui.horizontal(|ui| {
                ui.label(RichText::new(&name).strong());
                ui.label(RichText::new(if has_rig { "— with parameters" } else { "— no parameters yet: Apply adds them" }).color(theme::TEXT_DIM));
            });
            if parts.is_empty() {
                ui.label(RichText::new("The component has no parts to move: group its pieces inside it (a door, a drawer…) — open it and use Make Group.").color(Color32::from_rgb(190, 110, 20)).small());
            }
            egui::ScrollArea::vertical().max_height(560.0).show(ui, |ui| {
                // Parameters.
                ui.add_space(4.0);
                ui.label(RichText::new("Parameters").strong()).on_hover_text("The values each copy has (Entity Info shows them)");
                let mut del = None;
                egui::Grid::new("rig_params").num_columns(9).spacing([6.0, 4.0]).show(ui, |ui| {
                    for h in ["Name", "Label", "Type", "Min", "Max", "Step", "Default", "Options", ""] {
                        ui.label(RichText::new(h).small().color(theme::TEXT_DIM));
                    }
                    ui.end_row();
                    for (i, r) in ed.params.iter_mut().enumerate() {
                        ui.add(egui::TextEdit::singleline(&mut r.key).desired_width(80.0));
                        ui.add(egui::TextEdit::singleline(&mut r.label).desired_width(100.0));
                        egui::ComboBox::from_id_salt(("rig_pk", i)).width(80.0).selected_text(KINDS[r.kind]).show_ui(ui, |ui| {
                            for (k, n) in KINDS.iter().enumerate() {
                                ui.selectable_value(&mut r.kind, k, *n);
                            }
                        });
                        let ranged = r.kind <= 4;
                        for f in [&mut r.min, &mut r.max, &mut r.step] {
                            ui.add_enabled(ranged, egui::TextEdit::singleline(f).desired_width(50.0));
                        }
                        match KINDS[r.kind] {
                            "bool" => {
                                let mut b = r.default == "true";
                                if ui.checkbox(&mut b, "").changed() {
                                    r.default = b.to_string();
                                }
                            }
                            "material" => {
                                egui::ComboBox::from_id_salt(("rig_pm", i)).width(90.0).selected_text(if r.default.is_empty() { "(as drawn)" } else { r.default.as_str() }).show_ui(ui, |ui| {
                                    ui.selectable_value(&mut r.default, String::new(), "(as drawn)");
                                    for m in &materials {
                                        ui.selectable_value(&mut r.default, m.clone(), m);
                                    }
                                });
                            }
                            _ => {
                                ui.add(egui::TextEdit::singleline(&mut r.default).desired_width(70.0));
                            }
                        }
                        ui.add_enabled(KINDS[r.kind] == "choice", egui::TextEdit::singleline(&mut r.choices).desired_width(90.0).hint_text("a|b|c"));
                        if ui.small_button("×").on_hover_text("Remove the parameter").clicked() {
                            del = Some(i);
                        }
                        ui.end_row();
                    }
                });
                if let Some(i) = del {
                    ed.params.remove(i);
                }
                if ui.button("+ Parameter").clicked() {
                    let n = (1..).map(|k| format!("p{k}")).find(|k| !ed.params.iter().any(|r| &r.key == k)).unwrap_or_default();
                    ed.params.push(ParamRow { key: n.clone(), label: n, kind: 0, default: "0".into(), ..Default::default() });
                }
                // Formulas.
                ui.add_space(6.0);
                ui.label(RichText::new("Formulas").strong()).on_hover_text("Values computed from the parameters (Rhai expressions), usable by the motions and the script");
                let mut del = None;
                egui::Grid::new("rig_formulas").num_columns(5).spacing([6.0, 4.0]).show(ui, |ui| {
                    for (i, r) in ed.formulas.iter_mut().enumerate() {
                        ui.add(egui::TextEdit::singleline(&mut r.key).desired_width(80.0));
                        ui.label("=");
                        ui.add(egui::TextEdit::singleline(&mut r.expr).desired_width(260.0).hint_text("open / 2"));
                        egui::ComboBox::from_id_salt(("rig_fk", i)).width(70.0).selected_text(FORMULA_KINDS[r.kind]).show_ui(ui, |ui| {
                            for (k, n) in FORMULA_KINDS.iter().enumerate() {
                                ui.selectable_value(&mut r.kind, k, *n);
                            }
                        });
                        if ui.small_button("×").clicked() {
                            del = Some(i);
                        }
                        ui.end_row();
                    }
                });
                if let Some(i) = del {
                    ed.formulas.remove(i);
                }
                if ui.button("+ Formula").clicked() {
                    let n = (1..).map(|k| format!("f{k}")).find(|k| !ed.formulas.iter().any(|r| &r.key == k)).unwrap_or_default();
                    ed.formulas.push(FormulaRow { key: n, kind: 0, ..Default::default() });
                }
                // Motions of the parts.
                ui.add_space(6.0);
                ui.label(RichText::new("Motions").strong()).on_hover_text(
                    "A part (a group / component directly inside) turned, slid, shown, painted or given a parameter value (Set parameter: a part that has parameters itself — a generated component such as a Draw → Shapes cylinder, or a component with parameters) by a parameter. Turn / slide: the parameter's range maps onto From..To (degrees / length); without a range, or with an expression, the value is the motion itself",
                );
                let keys: Vec<String> = ed.params.iter().map(|r| r.key.clone()).chain(ed.formulas.iter().map(|r| r.key.clone())).collect();
                let mut del = None;
                for (i, r) in ed.joints.iter_mut().enumerate() {
                    egui::Frame::group(ui.style()).inner_margin(6.0).show(ui, |ui| {
                        ui.horizontal(|ui| {
                            egui::ComboBox::from_id_salt(("rig_part", i)).width(120.0).selected_text(if r.part.is_empty() { "(part)" } else { r.part.as_str() }).show_ui(ui, |ui| {
                                for p in &parts {
                                    ui.selectable_value(&mut r.part, p.clone(), p);
                                }
                            });
                            egui::ComboBox::from_id_salt(("rig_motion", i)).width(90.0).selected_text(MOTIONS[r.motion]).show_ui(ui, |ui| {
                                for (k, n) in MOTIONS.iter().enumerate() {
                                    ui.selectable_value(&mut r.motion, k, *n);
                                }
                            });
                            if r.motion == 4 {
                                let own = part_params.get(&r.part).cloned().unwrap_or_default();
                                let shown = own.iter().find(|(k, _)| *k == r.key).map(|(k, l)| if l.is_empty() { k.clone() } else { l.clone() }).unwrap_or_else(|| if r.key.is_empty() { "(parameter)".into() } else { r.key.clone() });
                                let combo = egui::ComboBox::from_id_salt(("rig_pkey", i)).width(90.0).selected_text(shown).show_ui(ui, |ui| {
                                    if own.is_empty() {
                                        ui.label(RichText::new("The part has no parameters").small().color(theme::TEXT_DIM));
                                    }
                                    for (k, l) in &own {
                                        ui.selectable_value(&mut r.key, k.clone(), if l.is_empty() { k.as_str() } else { l.as_str() });
                                    }
                                });
                                combo.response.on_hover_text("The part's own parameter set to the value");
                            }
                            ui.label("by");
                            let shown = if r.param.is_empty() { "(expression)" } else { r.param.as_str() };
                            egui::ComboBox::from_id_salt(("rig_jp", i)).width(90.0).selected_text(shown).show_ui(ui, |ui| {
                                for k in &keys {
                                    ui.selectable_value(&mut r.param, k.clone(), k);
                                }
                                ui.selectable_value(&mut r.param, String::new(), "(expression)");
                            });
                            if r.motion <= 1 {
                                let suffix = if r.motion == 0 { "°" } else { "" };
                                ui.label("from");
                                ui.add(egui::DragValue::new(&mut r.from).speed(1.0).suffix(suffix));
                                ui.label("to");
                                ui.add(egui::DragValue::new(&mut r.to).speed(1.0).suffix(suffix));
                            }
                            if ui.small_button("×").on_hover_text("Remove the motion").clicked() {
                                del = Some(i);
                            }
                        });
                        if r.param.is_empty() || !r.expr.is_empty() {
                            ui.horizontal(|ui| {
                                ui.label("=");
                                ui.add(egui::TextEdit::singleline(&mut r.expr).desired_width(360.0).hint_text("open * 2   |   open > 0   |   if open > 45 { \"Red\" } else { \"\" }"));
                            });
                        }
                        if r.motion <= 1 {
                            ui.horizontal(|ui| {
                                if r.motion == 0 {
                                    ui.label("Axis through");
                                    for k in 0..3 {
                                        ui.add(egui::DragValue::new(&mut r.origin[k]).speed(1.0).max_decimals(2));
                                    }
                                    ui.label("along");
                                } else {
                                    ui.label("Direction");
                                }
                                for k in 0..3 {
                                    ui.add(egui::DragValue::new(&mut r.axis[k]).speed(0.01).max_decimals(4));
                                }
                                if ui.small_button("Pick…").on_hover_text("Click two points in the model").clicked() {
                                    pick = Some(i);
                                }
                                ui.label(RichText::new("mm, the component's axes").small().color(theme::TEXT_DIM));
                            });
                        }
                    });
                }
                if let Some(i) = del {
                    ed.joints.remove(i);
                }
                if ui.button("+ Motion").clicked() {
                    let key = ed.params.first().map(|r| r.key.clone()).unwrap_or_default();
                    ed.joints.push(JointRow { part: parts.first().cloned().unwrap_or_default(), param: key, ..Default::default() });
                }
                // Script.
                ui.add_space(6.0);
                ui.label(RichText::new("Script").strong()).on_hover_text(
                    "Optional Rhai code run after the motions inside the component: fn update(p) { … } with the whole script API (rotate, move, paint, show / hide, set_params, box…); child(name) / child(name, true) finds a part by name (deeply), part.min / max / size / bounds — its extents; p — the values and formulas",
                );
                if ed.script.is_empty() && ui.button("+ Script").clicked() {
                    ed.script = "fn update(p) {\n    // let door = child(\"Door\", true);\n    // rotate(door, door.min, vec3(0, 0, 1), -p.open);\n}\n".into();
                }
                if !ed.script.is_empty() {
                    ed.editor.show(ui, &mut ed.script, 140.0);
                }
            });
            ui.separator();
            for w in &warnings {
                ui.label(RichText::new(w).small().color(Color32::from_rgb(190, 110, 20)));
            }
            if let Some(e) = &ed.error {
                ui.label(RichText::new(e).color(Color32::from_rgb(200, 50, 40)));
            }
            ui.horizontal(|ui| {
                if ui.button(if has_rig { "Apply" } else { "Add Parameters" }).clicked() {
                    act = Some("apply");
                }
                if ui.add_enabled(ed.changed(), egui::Button::new("Revert")).clicked() {
                    act = Some("revert");
                }
                if ui.add_enabled(has_rig, egui::Button::new("Remove Parameters")).on_hover_text("The copies show the component as drawn again").clicked() {
                    act = Some("remove");
                }
            });
        });
        match act {
            Some("apply") => self.rig_apply(),
            Some("revert") => {
                if let Some(d) = self.rig_editor.def {
                    let rig = self.rig_editor.loaded.clone();
                    let xf = self.rig_editor.xf;
                    self.rig_editor.load(d, rig.as_ref(), xf);
                }
            }
            Some("remove") => self.rig_remove(),
            _ => {}
        }
        if let Some(j) = pick {
            self.rig_pick(j);
        }
        if !open {
            self.rig_editor.open = false;
            self.rig_editor.pick = None;
        }
    }

    /// The selection's rig for the harness: (component, [(key, value)…], joints, variants).
    pub(crate) fn rig_dump(&self) -> String {
        let m = &self.doc.model;
        let sel = self.selected_rigged().map(|(ids, master, g, values)| {
            let vals: Vec<String> = g.params.iter().map(|p| format!("{}={}", p.key, show_value(values[0].get(&p.key).unwrap_or(&p.default), &p.kind, m.unit))).collect();
            let r = m.defs[master].rig.as_ref();
            format!("{} {} [{}] joints={} variants={}", ids.len(), m.defs[master].name, vals.join(" "), r.map_or(0, |r| r.joints.len()), m.rig_variants(master).len())
        });
        let ed = &self.rig_editor;
        let j0 = ed.joints.first().map(|j| {
            let r = |v: [f64; 3]| v.map(|x| (x * 100.0).round() / 100.0);
            (r(j.origin), r(j.axis))
        });
        format!("{:?} ed=({} {:?} p={} f={} j={} {:?} {:?})", sel, ed.open, ed.def.and_then(|d| m.defs.get(d)).map(|d| d.name.clone()), ed.params.len(), ed.formulas.len(), ed.joints.len(), ed.error, j0)
    }
}

/// Values of components with parameters moving to a scene's (with the camera transition).
pub struct RigAnimation {
    /// (instance path, values at the start, the scene's values).
    items: Vec<(Vec<InstanceId>, GenValues, GenValues)>,
    t: f64,
    secs: f64,
}

/// `a` → `b` at `k` (0..1): numbers in between, flags and texts switch half way.
fn blend(a: &GenValues, b: &GenValues, k: f64) -> GenValues {
    b.iter()
        .map(|(key, vb)| {
            let v = match (a.get(key), vb) {
                (Some(GenValue::Num(x)), GenValue::Num(y)) => GenValue::Num(x + (y - x) * k),
                (Some(va), _) if k < 0.5 => va.clone(),
                _ => vb.clone(),
            };
            (key.clone(), v)
        })
        .collect()
}

fn smooth(s: f64) -> f64 {
    let s = s.clamp(0.0, 1.0);
    s * s * (3.0 - 2.0 * s)
}

impl VarstotasApp {
    /// Set the values of copies (instance paths) — not an undo step, as scenes switch tags.
    fn apply_rig_values(&mut self, items: &[(Vec<InstanceId>, GenValues)]) {
        let m = &self.doc.model;
        let todo: Vec<&(Vec<InstanceId>, GenValues)> = items
            .iter()
            .filter(|(p, v)| p.split_last().and_then(|(i, parent)| m.instance_rigged(parent, *i)).is_some_and(|(_, r)| varstotas_script::rig::normalize(m, r.master, v).is_ok_and(|n| n != r.values)))
            .collect();
        if todo.is_empty() {
            return;
        }
        self.doc.mutate_untracked(|m| {
            for (p, v) in todo {
                if let Some((i, parent)) = p.split_last()
                    && let Err(e) = varstotas_script::rig::set_values(m, parent, *i, v, false)
                {
                    log::warn!("scene parameters: {e}");
                }
            }
        });
        self.tray.gen_text = None;
    }

    /// Going to scene `s`: its values for the copies, over `secs` (0 = at once).
    pub(crate) fn scene_rigs(&mut self, s: &varstotas_kernel::Scene, secs: f64) {
        self.rig_anim = None;
        let Some(rigs) = &s.rigs else { return };
        let m = &self.doc.model;
        let items: Vec<(Vec<InstanceId>, GenValues, GenValues)> = rigs
            .iter()
            .filter_map(|r| {
                let (i, parent) = r.path.split_last()?;
                let cur = m.instance_rigged(parent, *i)?.1.values.clone();
                (cur != r.values).then(|| (r.path.clone(), cur, r.values.clone()))
            })
            .collect();
        if items.is_empty() {
            return;
        }
        if secs > 0.0 {
            self.rig_anim = Some(RigAnimation { items, t: 0.0, secs });
        } else {
            let to: Vec<_> = items.into_iter().map(|(p, _, v)| (p, v)).collect();
            self.apply_rig_values(&to);
        }
    }

    /// One frame of the values' transition; false when it is over.
    pub(crate) fn step_rig_anim(&mut self, dt: f64) -> bool {
        let Some(a) = self.rig_anim.as_mut() else { return false };
        a.t += dt;
        let k = smooth(a.t / a.secs);
        let done = a.t >= a.secs;
        let now: Vec<_> = a.items.iter().map(|(p, from, to)| (p.clone(), if done { to.clone() } else { blend(from, to, k) })).collect();
        if done {
            self.rig_anim = None;
        }
        self.apply_rig_values(&now);
        !done
    }

    /// A frame of an exported scene animation: the values of scene `from` moving to those of
    /// `to` at `s` (0..1 of the transition; `from == to` holds the scene's values).
    pub(crate) fn frame_rigs(&mut self, from: usize, to: usize, s: f64) {
        self.rig_anim = None;
        let scenes = &self.doc.model.scenes;
        let Some(b) = scenes.get(to).and_then(|x| x.rigs.clone()) else { return };
        let a = scenes.get(from).and_then(|x| x.rigs.clone()).unwrap_or_default();
        let k = smooth(s);
        let items: Vec<_> = b
            .iter()
            .map(|r| {
                let v = match a.iter().find(|x| x.path == r.path) {
                    Some(x) if from != to => blend(&x.values, &r.values, k),
                    _ => r.values.clone(),
                };
                (r.path.clone(), v)
            })
            .collect();
        self.apply_rig_values(&items);
    }
}
