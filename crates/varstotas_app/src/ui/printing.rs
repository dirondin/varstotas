//! 3D Printing → basic surface check and conservative repair. Jobs run on snapshots;
//! version/selection checks prevent a finished repair from replacing newer user edits.
use crate::{app::VarstotasApp, document::guarded};
use egui::{Color32, Context, RichText};
use std::sync::mpsc::{self, Receiver};
use varstotas_i18n::tr;
use varstotas_kernel::{
    EntityRef, Model,
    printing::{
        PrintIssueKind, PrintRepairOptions, PrintRepairSummary, PrintReport, PrintSelection,
    },
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct JobKey {
    document: u64,
    version: u64,
    selection: Option<PrintSelection>,
    tolerance: u64,
    settings: String,
}
struct ResultData {
    report: PrintReport,
    advanced: varstotas_kernel::printing::analysis::AdvancedReport,
    bed: Option<varstotas_kernel::printing::preparation::BedReport>,
    repaired: Option<(Model, PrintRepairSummary)>,
}
struct Job {
    key: JobKey,
    repair: bool,
    rx: Receiver<Result<ResultData, String>>,
}
pub struct PrintingWindow {
    pub open: bool,
    pub repair_tab: bool,
    pub use_selection: bool,
    pub show_issues: bool,
    pub options: PrintRepairOptions,
    pub report: Option<PrintReport>,
    pub(super) key: Option<JobKey>,
    job: Option<Job>,
    pub(super) error: Option<String>,
    pub(super) advanced_tab: Option<super::printing_tools::PrintTab>,
    pub(super) advanced_report: Option<varstotas_kernel::printing::analysis::AdvancedReport>,
    pub(super) bed_report: Option<varstotas_kernel::printing::preparation::BedReport>,
    pub(crate) map: super::printing_tools::PrintMap,
    pub(crate) cut_normal: varstotas_kernel::glam::DVec3,
    pub(crate) cut_position: f64,
    pub(crate) connectors: varstotas_kernel::printing::preparation::ConnectorOptions,
    pub(super) tool_job: Option<super::printing_tools::PrintToolJob>,
    pub(super) last_export: Option<(JobKey, std::path::PathBuf)>,
    pub(super) message: Option<String>,
    pub last: Option<PrintRepairSummary>,
    pub highlight: Option<PrintIssueKind>,
}
impl Default for PrintingWindow {
    fn default() -> Self {
        Self {
            open: false,
            repair_tab: false,
            use_selection: false,
            show_issues: true,
            options: Default::default(),
            report: None,
            advanced_tab: None,
            advanced_report: None,
            bed_report: None,
            map: Default::default(),
            cut_normal: varstotas_kernel::glam::DVec3::Z,
            cut_position: 0.,
            connectors: Default::default(),
            tool_job: None,
            last_export: None,
            message: None,
            key: None,
            job: None,
            error: None,
            last: None,
            highlight: None,
        }
    }
}
fn issue_label(k: PrintIssueKind) -> String {
    tr!(match k {
        PrintIssueKind::OpenEdges => "print-open-edges",
        PrintIssueKind::NonManifoldEdges => "print-nonmanifold",
        PrintIssueKind::InconsistentWinding => "print-winding",
        PrintIssueKind::DegenerateTriangles => "print-degenerate",
        PrintIssueKind::DuplicateTriangles => "print-duplicates",
        PrintIssueKind::InvertedShells => "print-inverted",
        PrintIssueKind::LooseEdges => "print-wire",
    })
}
impl VarstotasApp {
    pub(super) fn print_key(&self) -> JobKey {
        JobKey {
            document: self.doc.uid,
            settings: format!(
                "{:?} {:?}",
                self.prefs.printing.analysis, self.prefs.printing.bed
            ),
            version: self.doc.model_version,
            tolerance: self.printing.options.tolerance_mm.to_bits(),
            selection: self.printing.use_selection.then(|| PrintSelection {
                path: self.doc.active_path.clone(),
                entities: self.doc.selection.clone(),
            }),
        }
    }
    pub(crate) fn open_printing(&mut self, ctx: &Context, repair: bool) {
        self.printing.advanced_tab = None;
        self.printing.open = true;
        self.printing.repair_tab = repair;
        self.printing.use_selection = !self.doc.selection.is_empty();
        if self.printing.key.as_ref() != Some(&self.print_key()) && self.printing.job.is_none() {
            self.start_print_job(ctx, false);
        }
    }
    pub(super) fn printing_busy(&self) -> bool {
        self.printing.job.is_some() || self.printing.tool_job.is_some()
    }
    pub(crate) fn start_print_job(&mut self, ctx: &Context, repair: bool) {
        if self.printing_busy() {
            return;
        }
        let key = self.print_key();
        let selection = key.selection.clone();
        let mut model = self.doc.model.clone();
        let options = self.printing.options.clone();
        let analysis = self.prefs.printing.analysis.clone();
        let bed = self.prefs.printing.bed.clone();
        let (tx, rx) = mpsc::channel();
        let ctx = ctx.clone();
        self.printing.error = None;
        std::thread::spawn(move || {
            let result = guarded(|| {
                let summary = if repair {
                    Some(
                        model
                            .repair_for_printing(selection.as_ref(), &options)
                            .map_err(|e| e.to_string())?,
                    )
                } else {
                    None
                };
                let report = model.print_check(selection.as_ref(), options.tolerance_mm);
                let advanced =
                    model.print_analyze(selection.as_ref(), options.tolerance_mm, &analysis);
                let bed = model.print_bed_report(selection.as_ref(), &bed).ok();
                Ok::<_, String>(ResultData {
                    report,
                    advanced,
                    bed,
                    repaired: summary.map(|s| (model, s)),
                })
            })
            .and_then(|v| v);
            let _ = tx.send(result);
            ctx.request_repaint();
        });
        self.printing.job = Some(Job { key, repair, rx });
    }
    fn poll_print_job(&mut self, ctx: &Context) {
        let Some(job) = self.printing.job.as_ref() else {
            return;
        };
        let result = match job.rx.try_recv() {
            Ok(r) => r,
            Err(mpsc::TryRecvError::Empty) => return,
            Err(_) => Err(tr!("print-worker-failed")),
        };
        let job = self.printing.job.take().unwrap();
        ctx.request_repaint();
        if job.key != self.print_key() {
            if job.repair {
                self.printing.error = Some(tr!("print-result-stale"));
            }
            return;
        }
        match result {
            Err(e) => self.printing.error = Some(e),
            Ok(result) => {
                if let Some((model, summary)) = result.repaired {
                    let applied = self.doc.try_commit_if(
                        &tr!("print-repair-undo"),
                        move |m| {
                            *m = model;
                            Ok::<_, String>(summary)
                        },
                        PrintRepairSummary::changed,
                    );
                    let Some(summary) = applied else {
                        self.printing.error = self.doc.last_error.clone();
                        return;
                    };
                    if summary.changed() {
                        self.library.def_thumbs.clear();
                    }
                    self.printing.last = Some(summary);
                }
                self.printing.advanced_report = Some(result.advanced);
                self.printing.bed_report = result.bed;
                self.printing.report = Some(result.report);
                self.printing.key = Some(self.print_key());
            }
        }
    }
    pub(crate) fn printing_overlay(&self, out: &mut varstotas_render::Overlay3d) {
        self.print_tools_overlay(out);
        if !self.printing.open
            || !self.printing.show_issues
            || self.printing.key.as_ref() != Some(&self.print_key())
        {
            return;
        }
        if let Some(report) = &self.printing.report {
            for issue in &report.issues {
                if self.printing.highlight.is_some_and(|k| k != issue.kind) {
                    continue;
                }
                let color = if issue.kind == PrintIssueKind::LooseEdges {
                    Color32::from_rgb(220, 140, 20)
                } else {
                    Color32::from_rgb(235, 55, 45)
                };
                for [a, b] in &issue.segments {
                    crate::inference::push_line(out, *a, *b, color, 2., false, true);
                }
            }
        }
    }
    pub(crate) fn printing_window(&mut self, ctx: &Context) {
        self.poll_print_tool_job(ctx);
        self.poll_print_job(ctx);
        if !self.printing.open {
            return;
        }
        if self.printing_busy() {
            ctx.request_repaint_after(std::time::Duration::from_millis(60));
        }
        let busy = self.printing_busy();
        let stale = self.printing.key.as_ref() != Some(&self.print_key());
        let mut open = true;
        let mut check = false;
        let mut repair = false;
        egui::Window::new(tr!("print-title"))
            .id(egui::Id::new("printing"))
            .open(&mut open)
            .resizable(true)
            .default_size([560., 650.])
            .min_size([420., 380.])
            .show(ctx, |ui| {
                ui.add_enabled_ui(!busy, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(tr!("print-scope"));
                        let r = ui.selectable_value(
                            &mut self.printing.use_selection,
                            false,
                            tr!("print-visible"),
                        );
                        super::tray_lib::spot("printing:visible", r.rect.center());
                        let r = ui.selectable_value(
                            &mut self.printing.use_selection,
                            true,
                            tr!("print-selection"),
                        );
                        super::tray_lib::spot("printing:selection", r.rect.center());
                    });
                });
                ui.horizontal_wrapped(|ui| {
                    for (repair, key, id) in [
                        (false, "print-tab-check", "printing:check-tab"),
                        (true, "print-tab-repair", "printing:repair-tab"),
                    ] {
                        let r = ui.selectable_label(
                            self.printing.advanced_tab.is_none()
                                && self.printing.repair_tab == repair,
                            tr!(key),
                        );
                        super::tray_lib::spot(id, r.rect.center());
                        if r.clicked() {
                            self.printing.advanced_tab = None;
                            self.printing.repair_tab = repair;
                        }
                    }
                    use super::printing_tools::PrintTab;
                    for (tab, key) in [
                        (PrintTab::Analysis, "print-analysis"),
                        (PrintTab::Bed, "print-bed"),
                        (PrintTab::Split, "print-split"),
                        (PrintTab::Export, "print-export"),
                    ] {
                        let response =
                            ui.selectable_label(self.printing.advanced_tab == Some(tab), tr!(key));
                        super::tray_lib::spot(
                            format!("printing:tab:{tab:?}"),
                            response.rect.center(),
                        );
                        if response.clicked() {
                            self.select_print_tab(tab);
                        }
                    }
                });
                ui.separator();
                if busy {
                    ui.horizontal(|ui| {
                        ui.spinner();
                        ui.label(tr!("print-working"));
                    });
                }
                if let Some(error) = &self.printing.error {
                    ui.colored_label(Color32::from_rgb(210, 65, 45), error);
                }
                egui::ScrollArea::vertical()
                    .id_salt("printing-content")
                    .show(ui, |ui| {
                        if let Some(tab) = self.printing.advanced_tab {
                            self.print_tools_content(ui, ctx, tab);
                            return;
                        }
                        if stale && self.printing.report.is_some() {
                            ui.label(
                                RichText::new(tr!("print-stale"))
                                    .color(Color32::from_rgb(190, 120, 20)),
                            );
                        }

                        if self.printing.repair_tab {
                            if stale && ui.button(tr!("print-go-check")).clicked() {
                                self.printing.repair_tab = false;
                            }
                            ui.add_enabled_ui(!busy, |ui| {
                                ui.checkbox(
                                    &mut self.printing.options.weld_vertices,
                                    tr!("print-repair-weld"),
                                );
                                ui.checkbox(
                                    &mut self.printing.options.remove_degenerate,
                                    tr!("print-repair-degenerate"),
                                );
                                ui.checkbox(
                                    &mut self.printing.options.remove_duplicates,
                                    tr!("print-repair-duplicates"),
                                );
                                ui.checkbox(
                                    &mut self.printing.options.orient_faces,
                                    tr!("print-repair-orient"),
                                );
                                ui.checkbox(
                                    &mut self.printing.options.orient_shells,
                                    tr!("print-repair-shells"),
                                );
                                let r = ui.checkbox(
                                    &mut self.printing.options.fill_holes,
                                    tr!("print-repair-holes"),
                                );
                                super::tray_lib::spot("printing:fill-holes", r.rect.center());
                                if self.printing.options.fill_holes {
                                    ui.horizontal(|ui| {
                                        ui.label(tr!("print-hole-size"));
                                        ui.add(
                                            egui::DragValue::new(
                                                &mut self.printing.options.max_hole_mm,
                                            )
                                            .range(0.001..=1000.)
                                            .speed(0.1)
                                            .suffix(" mm"),
                                        );
                                    });
                                }
                            });
                            ui.label(RichText::new(tr!("print-repair-note")).small().weak());
                            let partial = self.printing.use_selection
                                && self
                                    .doc
                                    .selection
                                    .iter()
                                    .any(|e| !matches!(e, EntityRef::Instance(_)));
                            if partial {
                                ui.label(tr!("print-repair-whole-objects"));
                            }
                            let has = self
                                .printing
                                .report
                                .as_ref()
                                .is_some_and(|r| !r.faces.is_empty());
                            let r = ui.add_enabled(
                                !busy && !stale && !partial && has,
                                egui::Button::new(tr!("print-repair-run")),
                            );
                            super::tray_lib::spot("printing:repair", r.rect.center());
                            repair = r.clicked();
                            if let Some(s) = &self.printing.last {
                                ui.label(tr!(
                                    "print-repaired",
                                    welded = s.welded_vertices,
                                    removed = s.removed_faces,
                                    reversed = s.reversed_faces,
                                    holes = s.filled_holes,
                                    skipped = s.skipped_objects
                                ));
                            }
                            ui.separator();
                        }
                        if let Some(report) = &self.printing.report {
                            if report.faces.is_empty() {
                                ui.label(tr!("print-empty"));
                            } else {
                                ui.label(tr!(
                                    "print-counts",
                                    faces = report.faces.len(),
                                    triangles = report.triangles,
                                    shells = report.shells.len(),
                                    closed = report.shells.iter().filter(|s| s.closed).count()
                                ));
                                if let Some(b) = report.bounds {
                                    let size = b.max - b.min;
                                    ui.label(tr!(
                                        "print-size",
                                        x = format!("{:.3}", size.x),
                                        y = format!("{:.3}", size.y),
                                        z = format!("{:.3}", size.z)
                                    ));
                                }
                                ui.label(tr!(
                                    "print-area",
                                    area = format!("{:.3}", report.area_mm2)
                                ));
                                if report.shells.iter().all(|s| s.closed)
                                    && !report.has_surface_errors()
                                {
                                    ui.label(tr!(
                                        "print-volume",
                                        volume = format!(
                                            "{:.3}",
                                            report
                                                .shells
                                                .iter()
                                                .map(|s| s.volume_mm3)
                                                .sum::<f64>()
                                                .abs()
                                        )
                                    ));
                                }
                                ui.add_space(6.);
                                ui.label(
                                    RichText::new(if report.has_surface_errors() {
                                        tr!("print-found-problems")
                                    } else {
                                        tr!("print-basic-pass")
                                    })
                                    .strong(),
                                );
                                egui::Grid::new("printing-issues")
                                    .num_columns(2)
                                    .show(ui, |ui| {
                                        for issue in &report.issues {
                                            let selected =
                                                self.printing.highlight == Some(issue.kind);
                                            let r = ui.selectable_label(
                                                selected,
                                                issue_label(issue.kind),
                                            );
                                            super::tray_lib::spot(
                                                format!("printing:issue:{:?}", issue.kind),
                                                r.rect.center(),
                                            );
                                            if r.clicked() {
                                                self.printing.highlight =
                                                    if selected { None } else { Some(issue.kind) };
                                            }
                                            ui.label(issue.count.to_string());
                                            ui.end_row();
                                        }
                                    });
                            }
                        }
                        ui.add_space(8.);
                        ui.label(RichText::new(tr!("print-limitations")).small().weak());
                        if !self.printing.repair_tab {
                            ui.separator();
                            ui.add_enabled_ui(!busy, |ui| {
                                ui.horizontal(|ui| {
                                    ui.label(tr!("print-tolerance"));
                                    ui.add(
                                        egui::DragValue::new(
                                            &mut self.printing.options.tolerance_mm,
                                        )
                                        .range(1e-9..=1.)
                                        .speed(0.00001)
                                        .min_decimals(5)
                                        .max_decimals(9)
                                        .suffix(" mm"),
                                    );
                                });
                            });
                            ui.checkbox(&mut self.printing.show_issues, tr!("print-highlight"));
                            ui.horizontal(|ui| {
                                let r = ui
                                    .add_enabled(!busy, egui::Button::new(tr!("print-check-run")));
                                super::tray_lib::spot("printing:check", r.rect.center());
                                check = r.clicked();
                                if let Some(report) = &self.printing.report
                                    && ui
                                        .add_enabled(!stale, egui::Button::new(tr!("print-copy")))
                                        .clicked()
                                {
                                    let text = std::iter::once(tr!("print-title"))
                                        .chain(report.issues.iter().map(|i| {
                                            format!("{}: {}", issue_label(i.kind), i.count)
                                        }))
                                        .chain(std::iter::once(tr!("print-limitations")))
                                        .collect::<Vec<_>>()
                                        .join("\n");
                                    ctx.copy_text(text);
                                }
                            });
                        }
                    });
            });
        self.printing.open = open;
        if check {
            self.start_print_job(ctx, false);
        } else if repair {
            self.start_print_job(ctx, true);
        }
    }
    pub(crate) fn printing_dump(&self) -> String {
        let r = self.printing.report.as_ref();
        format!(
            "printing=({} {} busy={} stale={} faces={} open={} nonmanifold={} winding={} duplicates={} inverted={} repaired={})",
            self.printing.open,
            self.printing.repair_tab,
            self.printing_busy(),
            self.printing.key.as_ref() != Some(&self.print_key()),
            r.map_or(0, |r| r.faces.len()),
            r.map_or(0, |r| r.count(PrintIssueKind::OpenEdges)),
            r.map_or(0, |r| r.count(PrintIssueKind::NonManifoldEdges)),
            r.map_or(0, |r| r.count(PrintIssueKind::InconsistentWinding)),
            r.map_or(0, |r| r.count(PrintIssueKind::DuplicateTriangles)),
            r.map_or(0, |r| r.count(PrintIssueKind::InvertedShells)),
            self.printing
                .last
                .as_ref()
                .is_some_and(PrintRepairSummary::changed)
        )
    }
}
