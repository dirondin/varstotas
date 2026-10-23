//! Remaining print workflow: measured analysis, persistent printer profiles, rigid placement,
//! capped cuts with dowels, scoped export and argument-safe slicer handoff.
use super::printing::JobKey;
use crate::{app::VarstotasApp, document::guarded};
use egui::{Color32, Context, RichText};
use serde::{Deserialize, Serialize};
use std::{
    path::{Path, PathBuf},
    sync::mpsc::{self, Receiver},
};
use varstotas_i18n::tr;
use varstotas_kernel::{
    EntityRef, Model, Plane,
    glam::DVec3,
    printing::{
        PrintSelection,
        analysis::AnalysisOptions,
        preparation::{BedProfile, Placement, PlacementResult},
    },
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PrintTab {
    Analysis,
    Bed,
    Split,
    Export,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PrintMap {
    #[default]
    None,
    Thickness,
    Overhang,
}
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct PrintingPrefs {
    pub bed: BedProfile,
    pub analysis: AnalysisOptions,
    pub show_bed: bool,
    pub slicer: Option<PathBuf>,
}
pub(super) struct ToolResult {
    model: Option<Model>,
    parts: Vec<EntityRef>,
    path: Vec<varstotas_kernel::InstanceId>,
    message: String,
    export: Option<PathBuf>,
}
pub(super) struct PrintToolJob {
    key: JobKey,
    rx: Receiver<Result<ToolResult, String>>,
}
fn value(
    ui: &mut egui::Ui,
    label: String,
    v: &mut f64,
    range: std::ops::RangeInclusive<f64>,
    id: &str,
) {
    ui.horizontal(|ui| {
        ui.label(label);
        let r = ui.add(
            egui::DragValue::new(v)
                .range(range)
                .speed(0.1)
                .suffix(" mm"),
        );
        super::tray_lib::spot(id, r.rect.center());
    });
}
fn button(ui: &mut egui::Ui, label: String, id: &str, enabled: bool) -> bool {
    let r = ui.add_enabled(enabled, egui::Button::new(label));
    super::tray_lib::spot(id, r.rect.center());
    r.clicked()
}
fn selection(app: &VarstotasApp) -> Option<PrintSelection> {
    app.printing.use_selection.then(|| PrintSelection {
        path: app.doc.active_path.clone(),
        entities: app.doc.selection.clone(),
    })
}
fn whole_selection(app: &VarstotasApp) -> Option<PrintSelection> {
    matches!(app.doc.selection.as_slice(), [EntityRef::Instance(_)]).then(|| PrintSelection {
        path: app.doc.active_path.clone(),
        entities: app.doc.selection.clone(),
    })
}
impl VarstotasApp {
    pub(crate) fn open_print_tools(&mut self, ctx: &Context, tab: PrintTab) {
        self.open_printing(ctx, false);
        self.select_print_tab(tab);
    }
    pub(super) fn select_print_tab(&mut self, tab: PrintTab) {
        self.printing.advanced_tab = Some(tab);
        if tab == PrintTab::Bed {
            self.prefs.printing.show_bed = true;
        }
        if tab == PrintTab::Split
            && let [EntityRef::Instance(i)] = self.doc.selection.as_slice()
            && let Some(b) = self
                .doc
                .model
                .instance_world_bounds(&self.doc.active_path, *i)
        {
            let n = self.printing.cut_normal.normalize_or(DVec3::Z);
            self.printing.cut_position = n.dot(b.center());
        }
    }
    pub(super) fn poll_print_tool_job(&mut self, ctx: &Context) {
        let Some(job) = self.printing.tool_job.as_ref() else {
            return;
        };
        let result = match job.rx.try_recv() {
            Ok(r) => r,
            Err(mpsc::TryRecvError::Empty) => return,
            Err(_) => Err(tr!("print-worker-failed")),
        };
        let job = self.printing.tool_job.take().unwrap();
        ctx.request_repaint();
        match result {
            Err(e) => self.printing.error = Some(e),
            Ok(result) => {
                if job.key != self.print_key() {
                    self.printing.error = Some(if result.export.is_some() {
                        tr!("print-export-stale")
                    } else {
                        tr!("print-result-stale")
                    });
                    return;
                }
                if let Some(model) = result.model {
                    let parts = result.parts;
                    let applied = self.doc.try_commit(&tr!("print-cut-undo"), move |m| {
                        *m = model;
                        Ok::<_, String>(parts)
                    });
                    if let Some(parts) = applied {
                        self.doc.active_path = result.path;
                        self.doc.set_selection(parts);
                        self.printing.use_selection = true;
                        self.library.def_thumbs.clear();
                    } else {
                        self.printing.error = self.doc.last_error.clone();
                        return;
                    }
                }
                if let Some(path) = result.export {
                    self.printing.last_export = Some((job.key, path));
                }
                self.printing.message = Some(result.message);
            }
        }
    }
    pub(crate) fn print_place_action(&mut self, mode: Placement) {
        if self.printing_busy() {
            return;
        }
        let scope = selection(self);
        let bed = self.prefs.printing.bed.clone();
        let result = self.doc.try_commit_if(
            &tr!("print-place-undo"),
            |m| m.print_place(scope.as_ref(), &bed, mode),
            |r| r.changed,
        );
        if let Some(r) = result {
            self.apply_print_placement(r);
        } else {
            self.printing.error = self.doc.last_error.clone();
        }
    }
    fn apply_print_placement(&mut self, result: PlacementResult) {
        self.doc.active_path = result.path;
        self.doc.set_selection(result.entities);
        self.printing.use_selection = true;
        self.printing.error = None;
        self.printing.message = Some(tr!("print-placement-done"));
        self.prefs.printing.show_bed = true;
        self.actions.push(crate::app::Action::ZoomExtents);
    }
    pub(crate) fn print_lay_action(&mut self) {
        if self.printing_busy() {
            return;
        }
        let [EntityRef::Face(f)] = self.doc.selection.as_slice() else {
            return;
        };
        let f = *f;
        let path = self.doc.active_path.clone();
        let bed = self.prefs.printing.bed.clone();
        let result = self.doc.try_commit_if(
            &tr!("print-place-undo"),
            |m| m.print_lay_face(&path, f, &bed),
            |r| r.changed,
        );
        if let Some(r) = result {
            self.apply_print_placement(r);
        } else {
            self.printing.error = self.doc.last_error.clone();
        }
    }
    pub(crate) fn start_print_cut(&mut self, ctx: &Context, to_bed: bool) {
        if self.printing_busy() {
            return;
        }
        let Some(scope) = whole_selection(self) else {
            self.printing.error = Some(tr!("print-cut-select"));
            return;
        };
        let key = self.print_key();
        let mut model = self.doc.model.clone();
        let bed = self.prefs.printing.bed.clone();
        let connector = self.printing.connectors.clone();
        let n = self.printing.cut_normal.normalize_or(DVec3::Z);
        let plane = Plane::new(n * self.printing.cut_position, n);
        let (tx, rx) = mpsc::channel();
        let ctx = ctx.clone();
        self.printing.error = None;
        std::thread::spawn(move || {
            let r = guarded(|| {
                let result = if to_bed {
                    model.print_split_to_bed(&scope, &bed)
                } else {
                    model.print_split(&scope, plane, &connector)
                }
                .map_err(|e| e.to_string())?;
                Ok::<_, String>(ToolResult {
                    model: Some(model),
                    parts: result
                        .parts
                        .iter()
                        .map(|i| EntityRef::Instance(*i))
                        .collect(),
                    path: scope.path,
                    message: tr!(
                        "print-cut-done",
                        parts = result.parts.len(),
                        dowels = result.dowels
                    ),
                    export: None,
                })
            })
            .and_then(|r| r);
            let _ = tx.send(r);
            ctx.request_repaint();
        });
        self.printing.tool_job = Some(PrintToolJob { key, rx });
    }
    pub(crate) fn start_print_export(&mut self, ctx: &Context, path: PathBuf) {
        if self.printing_busy() {
            return;
        }
        let scope = selection(self);
        let key = self.print_key();
        let model = self.doc.model.clone();
        let title = self.doc.display_name();
        let (tx, rx) = mpsc::channel();
        let ctx = ctx.clone();
        self.printing.error = None;
        std::thread::spawn(move || {
            let result = guarded(|| {
                let mut snapshot = model
                    .print_export_model(scope.as_ref())
                    .map_err(|e| e.to_string())?;
                snapshot.defs[snapshot.root].name = title;
                let extension = path
                    .extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("")
                    .to_ascii_lowercase();
                let bytes = match extension.as_str() {
                    "3mf" => varstotas_kernel::io::write_3mf(&snapshot),
                    "stl" => varstotas_kernel::io::write_stl_binary(&snapshot),
                    _ => return Err(tr!("print-export-extension")),
                };
                varstotas_kernel::io::atomic_write(&path, &bytes).map_err(|e| e.to_string())?;
                Ok(ToolResult {
                    model: None,
                    parts: Vec::new(),
                    path: Vec::new(),
                    message: tr!(
                        "print-export-done",
                        parts = snapshot.root_mesh().instances.len(),
                        path = path.display().to_string()
                    ),
                    export: Some(path),
                })
            })
            .and_then(|r| r);
            let _ = tx.send(result);
            ctx.request_repaint();
        });
        self.printing.tool_job = Some(PrintToolJob { key, rx });
    }
    fn choose_print_export(&mut self, ctx: &Context, ext: &str) {
        if let Some(mut path) = rfd::FileDialog::new()
            .add_filter(ext.to_uppercase(), &[ext])
            .set_file_name(format!("{}-print.{ext}", self.doc.display_name()))
            .save_file()
        {
            if path.extension().is_none() {
                path.set_extension(ext);
            }
            self.start_print_export(ctx, path);
        }
    }
    pub(crate) fn open_print_slicer(&mut self) {
        if self.printing_busy() {
            return;
        }
        let Some((key, path)) = &self.printing.last_export else {
            return;
        };
        if key != &self.print_key() {
            self.printing.error = Some(tr!("print-export-stale"));
            return;
        }
        let Some(slicer) = self.prefs.printing.slicer.as_deref() else {
            self.printing.error = Some(tr!("print-slicer-choose"));
            return;
        };
        match launch_slicer(slicer, path) {
            Ok(_) => self.printing.message = Some(tr!("print-slicer-opened")),
            Err(e) => self.printing.error = Some(e),
        }
    }
    pub(super) fn print_tools_content(&mut self, ui: &mut egui::Ui, ctx: &Context, tab: PrintTab) {
        let busy = self.printing_busy();
        let current = self.printing.key.as_ref() == Some(&self.print_key());
        if let Some(message) = &self.printing.message {
            ui.label(message);
            ui.separator();
        }
        match tab {
            PrintTab::Analysis => {
                if button(
                    ui,
                    tr!("print-analysis-run"),
                    "printing:analysis-run",
                    !busy,
                ) {
                    self.start_print_job(ctx, false);
                }

                ui.add_enabled_ui(!busy, |ui| {
                    value(
                        ui,
                        tr!("print-min-wall"),
                        &mut self.prefs.printing.analysis.minimum_wall_mm,
                        0.001..=100.,
                        "printing:min-wall",
                    );
                    if button(ui, tr!("print-two-nozzles"), "printing:two-nozzles", true) {
                        self.prefs.printing.analysis.minimum_wall_mm =
                            self.prefs.printing.bed.nozzle_mm * 2.;
                    }
                    ui.horizontal(|ui| {
                        ui.label(tr!("print-sample-budget"));
                        ui.add(
                            egui::DragValue::new(&mut self.prefs.printing.analysis.max_samples)
                                .range(1..=500_000)
                                .speed(100.),
                        );
                    });
                    ui.horizontal(|ui| {
                        ui.label(tr!("print-pair-budget"));
                        ui.add(
                            egui::DragValue::new(&mut self.prefs.printing.analysis.max_pairs)
                                .range(1..=10_000_000)
                                .speed(1000.),
                        );
                    });
                    ui.horizontal(|ui| {
                        ui.label(tr!("print-overhang-angle"));
                        ui.add(
                            egui::DragValue::new(
                                &mut self.prefs.printing.analysis.overhang_degrees,
                            )
                            .range(1.0..=89.9)
                            .suffix("°"),
                        );
                    });
                });
                ui.horizontal_wrapped(|ui| {
                    ui.label(tr!("print-map"));
                    for (mode, key) in [
                        (PrintMap::None, "print-map-none"),
                        (PrintMap::Thickness, "print-map-wall"),
                        (PrintMap::Overhang, "print-map-overhang"),
                    ] {
                        let r = ui.selectable_value(&mut self.printing.map, mode, tr!(key));
                        super::tray_lib::spot(format!("printing:map:{mode:?}"), r.rect.center());
                    }
                });
                if let Some(r) = &self.printing.advanced_report {
                    ui.separator();
                    ui.label(tr!(
                        "print-intersection-count",
                        n = r.intersection_pairs,
                        pairs = r.candidate_pairs
                    ));
                    if !r.intersections_complete {
                        ui.colored_label(
                            Color32::from_rgb(190, 100, 20),
                            tr!("print-intersection-incomplete"),
                        );
                    }
                    ui.label(tr!(
                        "print-wall-coverage",
                        samples = r.samples,
                        possible = r.possible_samples,
                        triangles = r.sampled_triangles,
                        total = r.triangles
                    ));
                    ui.label(tr!(
                        "print-wall-result",
                        minimum = r
                            .minimum_mm
                            .map_or_else(|| "—".into(), |m| format!("{m:.4}")),
                        thin = r.thin_samples,
                        unknown = r.unknown_samples
                    ));
                    ui.label(tr!(
                        "print-overhang-result",
                        area = format!("{:.2}", r.overhang_area_mm2),
                        n = r.overhangs.len()
                    ));
                    if !r.reliable_surface {
                        ui.colored_label(
                            Color32::from_rgb(190, 100, 20),
                            tr!("print-wall-unreliable"),
                        );
                    }
                }
                ui.label(RichText::new(tr!("print-wall-estimate")).small().weak());
                if !current {
                    ui.label(tr!("print-stale"));
                }
            }
            PrintTab::Bed => {
                if button(ui, tr!("print-bed-refresh"), "printing:bed-refresh", !busy) {
                    self.start_print_job(ctx, false);
                }

                ui.add_enabled_ui(!busy, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(tr!("print-profile-name"));
                        ui.text_edit_singleline(&mut self.prefs.printing.bed.name);
                    });
                    for (name, id) in [
                        ("print-bed-width", 0),
                        ("print-bed-depth", 1),
                        ("print-bed-height", 2),
                        ("print-bed-margin", 3),
                        ("print-bed-gap", 4),
                        ("print-nozzle", 5),
                    ] {
                        let bed = &mut self.prefs.printing.bed;
                        let v = match id {
                            0 => &mut bed.width_mm,
                            1 => &mut bed.depth_mm,
                            2 => &mut bed.height_mm,
                            3 => &mut bed.margin_mm,
                            4 => &mut bed.gap_mm,
                            _ => &mut bed.nozzle_mm,
                        };
                        value(
                            ui,
                            tr!(name),
                            v,
                            if id == 3 || id == 4 {
                                0.0..=1000.0
                            } else {
                                0.01..=10000.0
                            },
                            &format!("printing:profile:{id}"),
                        );
                    }
                    ui.checkbox(
                        &mut self.prefs.printing.bed.rotate_parts,
                        tr!("print-arrange-rotate"),
                    );
                    ui.checkbox(&mut self.prefs.printing.show_bed, tr!("print-bed-show"));
                });
                ui.horizontal_wrapped(|ui| {
                    if button(ui, tr!("print-drop"), "printing:drop", !busy) {
                        self.print_place_action(Placement::Drop);
                    }
                    if button(ui, tr!("print-center"), "printing:center", !busy) {
                        self.print_place_action(Placement::Center);
                    }
                    if button(ui, tr!("print-arrange"), "printing:arrange", !busy) {
                        self.print_place_action(Placement::Arrange);
                    }
                    let face = matches!(self.doc.selection.as_slice(), [EntityRef::Face(_)]);
                    if button(
                        ui,
                        tr!("print-lay-face"),
                        "printing:lay-face",
                        !busy && face,
                    ) {
                        self.print_lay_action();
                    }
                });
                if button(ui, tr!("print-bed-frame"), "printing:frame-bed", true) {
                    let bed = &self.prefs.printing.bed;
                    if bed.valid() {
                        let mut camera = self.camera.clone();
                        camera.zoom_extents(
                            DVec3::ZERO,
                            DVec3::new(bed.width_mm, bed.depth_mm, bed.height_mm),
                            self.aspect(),
                        );
                        self.cam_anim = None;
                        self.camera = camera;
                    }
                }
                ui.label(RichText::new(tr!("print-placement-note")).small().weak());
                if current {
                    if let Some(r) = &self.printing.bed_report {
                        ui.label(tr!(
                            "print-bed-result",
                            parts = r.parts.len(),
                            outside = r.parts.iter().filter(|p| !p.fits).count(),
                            floating = r.parts.iter().filter(|p| !p.on_bed).count(),
                            overlaps = r.bounds_overlaps
                        ));
                        for p in &r.parts {
                            ui.label(format!(
                                "{}: {}",
                                p.name,
                                if p.fits {
                                    tr!("print-fits")
                                } else {
                                    tr!("print-outside")
                                }
                            ));
                        }
                    }
                } else {
                    ui.label(tr!("print-stale"));
                }
            }
            PrintTab::Split => {
                ui.label(tr!("print-cut-select"));
                ui.add_enabled_ui(!busy, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(tr!("print-cut-normal"));
                        for (axis, n) in [("X", DVec3::X), ("Y", DVec3::Y), ("Z", DVec3::Z)] {
                            if ui
                                .selectable_label(self.printing.cut_normal == n, axis)
                                .clicked()
                            {
                                self.printing.cut_normal = n;
                                self.select_print_tab(PrintTab::Split);
                            }
                        }
                    });
                    ui.horizontal(|ui| {
                        for i in 0..3 {
                            ui.add(
                                egui::DragValue::new(&mut self.printing.cut_normal[i]).speed(0.1),
                            );
                        }
                    });
                    value(
                        ui,
                        tr!("print-cut-offset"),
                        &mut self.printing.cut_position,
                        -1e9..=1e9,
                        "printing:cut-offset",
                    );
                    ui.checkbox(
                        &mut self.printing.connectors.enabled,
                        tr!("print-connectors"),
                    );
                    if self.printing.connectors.enabled {
                        ui.horizontal(|ui| {
                            ui.label(tr!("print-dowel-count"));
                            ui.add(
                                egui::DragValue::new(&mut self.printing.connectors.count)
                                    .range(1..=16),
                            );
                        });
                        value(
                            ui,
                            tr!("print-dowel-diameter"),
                            &mut self.printing.connectors.diameter_mm,
                            0.1..=100.,
                            "printing:dowel-diameter",
                        );
                        value(
                            ui,
                            tr!("print-dowel-length"),
                            &mut self.printing.connectors.length_mm,
                            0.1..=1000.,
                            "printing:dowel-length",
                        );
                        value(
                            ui,
                            tr!("print-dowel-clearance"),
                            &mut self.printing.connectors.clearance_mm,
                            0.0..=10.,
                            "printing:dowel-clearance",
                        );
                        value(
                            ui,
                            tr!("print-dowel-wall"),
                            &mut self.printing.connectors.wall_mm,
                            0.0..=100.,
                            "printing:dowel-wall",
                        );
                    }
                });
                ui.label(RichText::new(tr!("print-dowel-note")).small().weak());
                let can = whole_selection(self).is_some();
                ui.horizontal_wrapped(|ui| {
                    if button(
                        ui,
                        tr!("print-cut-run"),
                        "printing:cut",
                        !busy && can && self.printing.cut_normal.length_squared() > 1e-12,
                    ) {
                        self.start_print_cut(ctx, false);
                    }
                    if button(
                        ui,
                        tr!("print-cut-to-bed"),
                        "printing:cut-to-bed",
                        !busy && can,
                    ) {
                        self.start_print_cut(ctx, true);
                    }
                });
                ui.label(RichText::new(tr!("print-cut-to-bed-note")).small().weak());
            }
            PrintTab::Export => {
                ui.label(tr!("print-export-scope-note"));
                ui.label(
                    RichText::new(tr!("print-export-materials-note"))
                        .small()
                        .weak(),
                );
                ui.horizontal(|ui| {
                    if button(ui, tr!("print-export-3mf"), "printing:export-3mf", !busy) {
                        self.choose_print_export(ctx, "3mf");
                    }
                    if button(ui, tr!("print-export-stl"), "printing:export-stl", !busy) {
                        self.choose_print_export(ctx, "stl");
                    }
                });
                ui.separator();
                ui.label(tr!("print-slicer"));
                ui.add_enabled_ui(!busy, |ui| {
                    let mut path = self
                        .prefs
                        .printing
                        .slicer
                        .as_ref()
                        .map_or(String::new(), |p| p.display().to_string());
                    if ui.text_edit_singleline(&mut path).changed() {
                        self.prefs.printing.slicer =
                            (!path.trim().is_empty()).then(|| PathBuf::from(path.trim()));
                    }
                    if ui.button(tr!("print-slicer-browse")).clicked()
                        && let Some(path) = rfd::FileDialog::new().pick_file()
                    {
                        self.prefs.printing.slicer = Some(path);
                    }
                });
                let fresh = self
                    .printing
                    .last_export
                    .as_ref()
                    .is_some_and(|(key, path)| key == &self.print_key() && path.is_file());
                if let Some((_, path)) = &self.printing.last_export {
                    ui.label(path.display().to_string());
                }
                if button(
                    ui,
                    tr!("print-slicer-open"),
                    "printing:slicer-open",
                    !busy && fresh && self.prefs.printing.slicer.is_some(),
                ) {
                    self.open_print_slicer();
                }
                ui.label(RichText::new(tr!("print-slicer-note")).small().weak());
            }
        }
    }
    pub(super) fn print_tools_overlay(&self, out: &mut varstotas_render::Overlay3d) {
        let bed = &self.prefs.printing.bed;
        if self.prefs.printing.show_bed && bed.valid() {
            let points = [
                DVec3::ZERO,
                DVec3::X * bed.width_mm,
                DVec3::new(bed.width_mm, bed.depth_mm, 0.),
                DVec3::Y * bed.depth_mm,
            ];
            let color = Color32::from_rgb(40, 140, 180);
            for i in 0..4 {
                crate::inference::push_line(
                    out,
                    points[i],
                    points[(i + 1) % 4],
                    color,
                    2.,
                    false,
                    false,
                );
                let top = points[i] + DVec3::Z * bed.height_mm;
                crate::inference::push_line(out, points[i], top, color, 1., true, false);
                crate::inference::push_line(
                    out,
                    top,
                    points[(i + 1) % 4] + DVec3::Z * bed.height_mm,
                    color,
                    1.,
                    true,
                    false,
                );
            }
        }
        if !self.printing.open {
            return;
        }
        if self.printing.advanced_tab == Some(PrintTab::Split)
            && self.printing.cut_normal.length_squared() > 1e-12
            && let Some(scope) = whole_selection(self)
            && let [EntityRef::Instance(i)] = scope.entities.as_slice()
            && let Some(bounds) = self.doc.model.instance_world_bounds(&scope.path, *i)
        {
            let n = self.printing.cut_normal.normalize();
            let c = bounds.center() + n * (self.printing.cut_position - n.dot(bounds.center()));
            let pr = varstotas_kernel::geom::Projector::new(c, n);
            let size = (bounds.max - bounds.min).length() * 0.55;
            let ps = [
                c + pr.u * size + pr.v * size,
                c - pr.u * size + pr.v * size,
                c - pr.u * size - pr.v * size,
                c + pr.u * size - pr.v * size,
            ];
            for k in 0..4 {
                crate::inference::push_line(
                    out,
                    ps[k],
                    ps[(k + 1) % 4],
                    Color32::from_rgb(30, 150, 220),
                    1.5,
                    false,
                    true,
                );
            }
            out.polygons.push(varstotas_render::OverlayPolygon {
                tris: vec![
                    ps[0].as_vec3().to_array(),
                    ps[1].as_vec3().to_array(),
                    ps[2].as_vec3().to_array(),
                    ps[0].as_vec3().to_array(),
                    ps[2].as_vec3().to_array(),
                    ps[3].as_vec3().to_array(),
                ],
                color: [0.1, 0.6, 1., 0.12],
                on_top: true,
            });
        }
        if self.printing.key.as_ref() != Some(&self.print_key()) {
            return;
        }
        if let Some(report) = &self.printing.advanced_report {
            if self.printing.show_issues {
                for [a, b] in &report.intersections {
                    crate::inference::push_line(
                        out,
                        *a,
                        *b,
                        Color32::from_rgb(220, 20, 150),
                        2.5,
                        false,
                        true,
                    );
                }
            }
            let heat = match self.printing.map {
                PrintMap::None => return,
                PrintMap::Thickness => &report.thickness,
                PrintMap::Overhang => &report.overhangs,
            };
            for t in heat {
                let n = (t.points[1] - t.points[0])
                    .cross(t.points[2] - t.points[0])
                    .normalize_or_zero();
                let color = match (self.printing.map, t.value) {
                    (_, None) => [0.5, 0.5, 0.5, 0.5],
                    (PrintMap::Thickness, Some(v))
                        if v < self.prefs.printing.analysis.minimum_wall_mm =>
                    {
                        [1., 0.12, 0.05, 0.75]
                    }
                    (PrintMap::Thickness, Some(_)) => [0.15, 0.8, 0.3, 0.5],
                    _ => [1., 0.4, 0.05, 0.75],
                };
                out.polygons.push(varstotas_render::OverlayPolygon {
                    tris: t
                        .points
                        .map(|p| (p + n * 0.002).as_vec3().to_array())
                        .to_vec(),
                    color,
                    on_top: false,
                });
            }
        }
    }
    pub(crate) fn print_tools_dump(&self) -> String {
        let r = self.printing.advanced_report.as_ref();
        let bed = self.printing.bed_report.as_ref();
        format!(
            "print_tools=({:?} busy={} intersections={} complete={} samples={} unknown={} thin={} map={:?} bed_parts={} outside={} overlaps={} export={})",
            self.printing.advanced_tab,
            self.printing_busy(),
            r.map_or(0, |r| r.intersection_pairs),
            r.is_some_and(|r| r.intersections_complete),
            r.map_or(0, |r| r.samples),
            r.map_or(0, |r| r.unknown_samples),
            r.map_or(0, |r| r.thin_samples),
            self.printing.map,
            bed.map_or(0, |r| r.parts.len()),
            bed.map_or(0, |r| r.parts.iter().filter(|p| !p.fits).count()),
            bed.map_or(0, |r| r.bounds_overlaps),
            self.printing.last_export.is_some()
        )
    }
}
/// No shell, no user-provided command template: the exported path is exactly one argument.
pub(crate) fn launch_slicer(executable: &Path, file: &Path) -> Result<std::process::Child, String> {
    if !file.is_file() {
        return Err(tr!("print-export-missing"));
    }
    #[cfg(target_os = "macos")]
    if executable.extension().is_some_and(|e| e == "app") {
        return std::process::Command::new("/usr/bin/open")
            .arg("-a")
            .arg(executable)
            .arg("--")
            .arg(file)
            .spawn()
            .map_err(|e| e.to_string());
    }
    if !executable.is_file() {
        return Err(tr!("print-slicer-choose"));
    }
    std::process::Command::new(executable)
        .arg(file)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .map_err(|e| e.to_string())
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    #[test]
    fn slicer_receives_exactly_one_literal_file_argument() {
        use std::os::unix::fs::PermissionsExt;
        let dir = std::env::temp_dir().join(format!("varstotas-slicer-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let executable = dir.join("fake slicer");
        let file = dir.join("деталь ; $(touch INJECTED).3mf");
        let output = dir.join("received");
        std::fs::write(&file, b"file").unwrap();
        // The fixed test script writes next to itself; the model filename is never shell text.
        std::fs::write(&executable,b"#!/bin/sh\n[ \"$#\" -eq 1 ] || exit 20\nprintf '%s' \"$1\" > \"$(dirname \"$0\")/received\"\n").unwrap();
        std::fs::set_permissions(&executable, std::fs::Permissions::from_mode(0o700)).unwrap();
        assert!(
            launch_slicer(&executable, &file)
                .unwrap()
                .wait()
                .unwrap()
                .success()
        );
        assert_eq!(
            std::fs::read_to_string(output).unwrap(),
            file.to_str().unwrap()
        );
        assert!(!dir.join("INJECTED").exists());
        assert!(launch_slicer(&executable, &dir.join("missing")).is_err());
        std::fs::remove_dir_all(dir).unwrap();
    }
}
