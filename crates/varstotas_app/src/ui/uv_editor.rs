//! UV Editor: the texture layout of the selected faces (else every textured face of the
//! active context) drawn over the material's image, one tile per square. Faces joined by
//! shared edges that are not seams form islands; islands are picked, dragged (Alt: rotate,
//! Cmd/Ctrl: scale), unfolded, projected, turned, flipped, fitted, packed, aligned and given
//! one texel density, turned / scaled by typed amounts or at random. Seams are marked on the
//! selected edges or on the hard edges. The backdrop is one map of the material (colour,
//! normal, roughness, …), one tile or repeated; a checker can stand in for the textures
//! (here and in the 3D view).

use super::theme;
use crate::app::VarstotasApp;
use egui::{Color32, Pos2, Rect, Sense, Shape, Stroke, Ui, pos2, vec2};
use std::collections::HashSet;
use varstotas_i18n::tr;
use varstotas_kernel::glam::DVec2;
use varstotas_kernel::texture::TextureImage;
use varstotas_kernel::uv::{self, IslandOp, Jitter, Placements, Projection};
use varstotas_kernel::{EdgeId, EntityRef, FaceId, Material, MaterialId, Mesh, Model};

/// Pixel side of the checker texture and its squares per tile.
const CHECKER_PX: usize = 512;
const CHECKER_SQUARES: usize = 8;

#[derive(Clone, Copy, Debug, PartialEq)]
enum DragKind {
    Move,
    Rotate,
    Scale,
    Pan,
}

struct UvDrag {
    kind: DragKind,
    start: Pos2,
    /// Islands being dragged (faces), with their placements at the start.
    islands: Vec<Vec<FaceId>>,
    orig: Placements,
    /// Screen centre of the dragged islands (rotate / scale about it).
    pivot: Pos2,
    /// View centre at the start (pan).
    center: DVec2,
}

/// UV Editor state.
#[derive(Default)]
pub struct UvState {
    pub open: bool,
    /// Faces of the selected islands.
    pub selected: HashSet<FaceId>,
    /// View centre (tile units) and zoom (points per tile); 0 = frame the islands.
    center: DVec2,
    zoom: f32,
    drag: Option<UvDrag>,
    /// The checker stands in for textures (UV Editor and 3D view).
    pub checker: bool,
    density: String,
    /// Variant of the last Randomize (each press gives a new one).
    seed: u64,
    /// Randomize: any angle (else 90° steps) and the scale spread in percent (0 = kept).
    pub rand_any_angle: bool,
    pub rand_scale_pct: f64,
    /// Typed rotation (degrees) and scale factor.
    angle: String,
    scale: String,
    /// Text of the face fields (face, offset x / y, rotation, scale x / y, tile w / h).
    face_bufs: Option<(FaceId, [String; 7])>,
    /// The backdrop's map slot (`textures::SLOTS`; empty = the first the material has).
    pub map: &'static str,
    /// The backdrop shows one tile only (else repeated).
    pub single_tile: bool,
    image: Option<(usize, egui::TextureHandle)>,
    checker_tex: Option<egui::TextureHandle>,
    /// (model version, faces) → islands.
    cache: Option<(u64, Vec<FaceId>, Vec<Vec<FaceId>>)>,
}

/// Tile size (mm) of a face's own textured material (front, else back).
fn own_tile(model: &Model, mesh: &Mesh, f: FaceId) -> Option<[f64; 2]> {
    let face = mesh.faces.get(f)?;
    [face.front, face.back].into_iter().flatten().find_map(|id| model.materials.get(id)?.image.as_ref().map(|i| i.size))
}

/// The image in `slot` (`textures::SLOTS`) of a material.
fn slot_image<'a>(m: &'a Material, slot: &str) -> Option<&'a TextureImage> {
    match slot {
        "base" => m.image.as_ref(),
        _ => m.pbr.as_ref().and_then(|p| p.map(slot)),
    }
}

/// The map slots a material has images in.
fn slots_of(m: &Material) -> Vec<&'static str> {
    crate::textures::SLOTS.into_iter().filter(|s| slot_image(m, s).is_some()).collect()
}

/// A checker image (`CHECKER_SQUARES` squares a side, numbered colours down the diagonal so
/// mirrored or turned tiles show).
pub fn checker_image() -> egui::ColorImage {
    let n = CHECKER_PX;
    let sq = n / CHECKER_SQUARES;
    let mut rgba = vec![0u8; n * n * 4];
    for y in 0..n {
        for x in 0..n {
            let (i, j) = (x / sq, (n - 1 - y) / sq);
            let dark = (i + j) % 2 == 1;
            let mut c = if dark { [70, 70, 78] } else { [225, 225, 230] };
            // the first row runs red → yellow, the first column blue: orientation at a glance
            if j == 0 && !dark {
                c = [235, 120 + (i * 16) as u8, 90];
            }
            if i == 0 && !dark && j > 0 {
                c = [110, 150, 235];
            }
            let k = (y * n + x) * 4;
            rgba[k..k + 4].copy_from_slice(&[c[0], c[1], c[2], 255]);
        }
    }
    egui::ColorImage::from_rgba_unmultiplied([n, n], &rgba)
}

impl VarstotasApp {
    /// The faces the UV Editor works on: the selected faces of the active context, else
    /// every face of it painted with a textured material.
    pub(crate) fn uv_faces(&self) -> Vec<FaceId> {
        let Some(mesh) = self.doc.model.mesh_at(&self.doc.active_path) else { return Vec::new() };
        let sel: Vec<FaceId> = self.doc.selection.iter().filter_map(|e| if let EntityRef::Face(f) = e { Some(*f) } else { None }).filter(|f| mesh.faces.contains_key(*f)).collect();
        if !sel.is_empty() {
            return sel;
        }
        let model = &self.doc.model;
        // faces with no material of their own show the open groups' material
        let path = &self.doc.active_path;
        let inherits = (0..path.len()).rev().find_map(|i| model.mesh_at(&path[..i])?.instances.get(path[i])?.material).and_then(|id| model.materials.get(id)).is_some_and(|m| m.image.is_some());
        mesh.faces
            .iter()
            .filter(|(_, f)| {
                [f.front, f.back].into_iter().flatten().any(|id| model.materials.get(id).is_some_and(|m| m.image.is_some())) || (inherits && f.front.is_none() && f.back.is_none())
            })
            .map(|(k, _)| k)
            .collect()
    }

    /// The material whose maps the UV Editor shows: the first face's textured material, else
    /// the one the faces inherit from the open groups, else the current paint material.
    pub(crate) fn uv_material(&self, faces: &[FaceId]) -> Option<MaterialId> {
        let model = &self.doc.model;
        let textured = |id: MaterialId| model.materials.get(id).is_some_and(|m| !slots_of(m).is_empty());
        let path = &self.doc.active_path;
        let mesh = model.mesh_at(path)?;
        let own = faces.iter().find_map(|f| {
            let face = mesh.faces.get(*f)?;
            [face.front, face.back].into_iter().flatten().find(|id| textured(*id))
        });
        let inherited = || (0..path.len()).rev().find_map(|i| model.mesh_at(&path[..i])?.instances.get(path[i])?.material.filter(|id| textured(*id)));
        own.or_else(inherited).or_else(|| self.current_material_id().filter(|id| textured(*id)))
    }

    /// Tile size (mm) of a face: its own textured material's, else the backdrop material's.
    fn uv_tile(&self, mesh: &Mesh, f: FaceId, fallback: Option<[f64; 2]>) -> [f64; 2] {
        own_tile(&self.doc.model, mesh, f).or(fallback).unwrap_or([1000.0, 1000.0])
    }

    /// The backdrop material's colour tile size.
    fn uv_fallback_tile(&self, faces: &[FaceId]) -> Option<[f64; 2]> {
        let m = self.doc.model.materials.get(self.uv_material(faces)?)?;
        slots_of(m).first().and_then(|s| slot_image(m, s)).map(|i| i.size)
    }

    /// Open the UV Editor (on the selected / textured faces), framed, with no island picked.
    pub(crate) fn open_uv_editor(&mut self) {
        self.uv.open = true;
        self.uv.selected.clear();
        self.uv_run("frame");
    }

    /// Islands of [`Self::uv_faces`] (cached per model version).
    pub(crate) fn uv_islands(&mut self) -> Vec<Vec<FaceId>> {
        let faces = self.uv_faces();
        let version = self.doc.model_version;
        if let Some((v, f, isl)) = &self.uv.cache
            && *v == version
            && *f == faces
        {
            return isl.clone();
        }
        let isl = self.doc.model.mesh_at(&self.doc.active_path).map(|m| uv::islands(m, &faces)).unwrap_or_default();
        self.uv.cache = Some((version, faces, isl.clone()));
        isl
    }

    /// The selected islands, else all of them.
    fn uv_targets(&mut self) -> Vec<Vec<FaceId>> {
        let all = self.uv_islands();
        let sel: Vec<Vec<FaceId>> = all.iter().filter(|i| i.iter().any(|f| self.uv.selected.contains(f))).cloned().collect();
        if sel.is_empty() { all } else { sel }
    }

    /// One undo step setting the placements.
    fn uv_commit(&mut self, label: &str, list: Placements) {
        if list.is_empty() {
            return;
        }
        let path = self.doc.active_path.clone();
        self.doc.commit(label, |m| {
            if let Some(mesh) = m.mesh_at_mut(&path) {
                for (f, p) in list {
                    if let Some(face) = mesh.faces.get_mut(f) {
                        face.uv = p.filter(|p| !p.is_identity());
                    }
                }
            }
        });
    }

    fn uv_set_untracked(&mut self, list: &Placements) {
        let path = self.doc.active_path.clone();
        self.doc.mutate_untracked(|m| {
            if let Some(mesh) = m.mesh_at_mut(&path) {
                for (f, p) in list {
                    if let Some(face) = mesh.faces.get_mut(*f) {
                        face.uv = p.filter(|p| !p.is_identity());
                    }
                }
            }
        });
    }

    /// Mark (or clear) the seam on `edges`: one undo step.
    fn uv_set_seams(&mut self, edges: &[EdgeId], seam: bool) {
        let path = self.doc.active_path.clone();
        let label = if seam { tr!("uv-undo-mark-seams") } else { tr!("uv-undo-clear-seams") };
        let n = edges.len();
        self.doc.commit(&label, |m| {
            if let Some(mesh) = m.mesh_at_mut(&path) {
                for e in edges {
                    if let Some(ed) = mesh.edges.get_mut(*e) {
                        ed.seam = seam;
                    }
                }
            }
        });
        self.flash(if seam { tr!("uv-seams-marked", count = n) } else { tr!("uv-seams-cleared", count = n) });
    }

    /// Pixels per metre of the islands (texture width in pixels over its tile).
    fn uv_density(&self, faces: &[FaceId]) -> Option<f64> {
        let mesh = self.doc.model.mesh_at(&self.doc.active_path)?;
        let f = *faces.first()?;
        let face = mesh.faces.get(f)?;
        let img = [face.front, face.back].into_iter().flatten().find_map(|id| self.doc.model.materials.get(id)?.image.as_ref())?;
        let px_per_mm = img.pixels[0] as f64 / img.size[0];
        Some(px_per_mm * uv::tex_per_world(mesh, faces) * 1000.0)
    }

    /// Run a UV Editor command (buttons and scripts): `unfold`, `planar`, `box`, `cylinder`,
    /// `sphere`, `rotate`, `rotate_ccw`, `flip_u`, `flip_v`, `fit`, `pack`, `align`,
    /// `rotate_by <deg>` (the islands, anticlockwise), `scale_by <k>` (the islands grow, so the
    /// texture on the model shrinks), `reset`, `randomize`, `rand_scale <percent>`, `density <px/m>`, `seams_mark`, `seams_clear`, `seams_hard`, `checker`, `select_all`,
    /// `select_none`, `frame`.
    pub(crate) fn uv_run(&mut self, op: &str) {
        let (op, arg) = op.split_once(' ').unwrap_or((op, ""));
        let targets = self.uv_targets();
        let faces: Vec<FaceId> = targets.iter().flatten().copied().collect();
        let fallback = self.uv_fallback_tile(&faces);
        let Some(mesh) = self.doc.model.mesh_at(&self.doc.active_path) else { return };
        let label = tr!("uv-undo");
        let tile = targets.first().and_then(|i| i.first()).map_or([1000.0; 2], |f| self.uv_tile(mesh, *f, fallback));
        let number = |t: &str| -> Option<f64> { t.trim().trim_end_matches(['°', '×', 'x', '%']).trim().replace(',', ".").parse().ok().filter(|v: &f64| v.is_finite()) };
        let per_island = |f: &dyn Fn(&Mesh, &[FaceId]) -> Placements| -> Placements { targets.iter().flat_map(|i| f(mesh, i)).collect() };
        let list: Placements = match op {
            "unfold" => per_island(&|m, i| uv::largest_face(m, i).map(|r| uv::unfold(m, i, r)).unwrap_or_default()),
            "planar" => uv::project(mesh, &faces, Projection::planar_for(mesh, &faces)),
            "box" => uv::project(mesh, &faces, Projection::Box),
            "cylinder" => uv::project(mesh, &faces, Projection::cylinder_for(mesh, &faces)),
            "sphere" => uv::project(mesh, &faces, Projection::sphere_for(mesh, &faces)),
            "rotate" => per_island(&|m, i| uv::transform(m, i, IslandOp::Rotate(-90.0))),
            "rotate_ccw" => per_island(&|m, i| uv::transform(m, i, IslandOp::Rotate(90.0))),
            "flip_u" => per_island(&|m, i| uv::transform(m, i, IslandOp::FlipU)),
            "flip_v" => per_island(&|m, i| uv::transform(m, i, IslandOp::FlipV)),
            "fit" => per_island(&|m, i| uv::fit_to_tile(m, i, tile)),
            "reset" => faces.iter().map(|f| (*f, None)).collect(),
            "pack" => uv::pack(mesh, &targets, tile, 0.02),
            "align" => per_island(&|m, i| uv::align(m, i)),
            "rotate_by" | "scale_by" => {
                let op = match (op, number(arg)) {
                    ("rotate_by", Some(deg)) => IslandOp::Rotate(deg),
                    ("scale_by", Some(k)) if k > 1e-3 => IslandOp::Scale(k),
                    _ => {
                        self.flash_error(tr!("tray-invalid-value"));
                        return;
                    }
                };
                per_island(&|m, i| uv::transform(m, i, op))
            }
            "randomize" => {
                self.uv.seed = self.uv.seed.wrapping_add(1);
                let base = self.uv.seed.wrapping_mul(0x1000_0000);
                let p = (self.uv.rand_scale_pct / 100.0).clamp(0.0, 0.9);
                let jitter = Jitter { any_angle: self.uv.rand_any_angle, scale: [1.0 - p, 1.0 + p] };
                targets.iter().enumerate().flat_map(|(k, i)| uv::randomize(mesh, i, self.uv_tile(mesh, i[0], fallback), base + k as u64, jitter)).collect()
            }
            "density" => {
                let want: Option<f64> = arg.trim().replace(',', ".").parse().ok().filter(|v: &f64| *v > 0.0);
                match want {
                    Some(want) => {
                        let mut out = Vec::new();
                        for i in &targets {
                            if let Some(now) = self.uv_density(i).filter(|d| *d > 1e-9) {
                                out.extend(uv::transform(mesh, i, IslandOp::Scale(want / now)));
                            }
                        }
                        out
                    }
                    None => {
                        self.flash_error(tr!("tray-invalid-value"));
                        return;
                    }
                }
            }
            "seams_mark" | "seams_clear" | "seams_hard" => {
                let sel_edges: Vec<EdgeId> = self.doc.selection.iter().filter_map(|e| if let EntityRef::Edge(e) = e { Some(*e) } else { None }).collect();
                let edges = match op {
                    "seams_hard" => uv::hard_edges(mesh, &faces),
                    "seams_clear" if sel_edges.is_empty() => uv::edges_of(mesh, &self.uv_faces()).into_iter().filter(|e| mesh.edges[*e].seam).collect(),
                    _ => sel_edges,
                };
                if edges.is_empty() {
                    self.flash(tr!("uv-no-edges"));
                } else {
                    self.uv_set_seams(&edges, op != "seams_clear");
                }
                return;
            }
            "rand_scale" => {
                self.uv.rand_scale_pct = number(arg).unwrap_or(0.0).clamp(0.0, 90.0);
                return;
            }
            "checker" => {
                self.uv.checker = !self.uv.checker;
                self.viewport.checker = self.uv.checker;
                return;
            }
            "select_all" => {
                self.uv.selected = faces.into_iter().collect();
                return;
            }
            "select_none" => {
                self.uv.selected.clear();
                return;
            }
            "frame" => {
                self.uv.zoom = 0.0;
                return;
            }
            _ => {
                log::warn!("uv: unknown command `{op}`");
                return;
            }
        };
        if list.is_empty() {
            self.flash(tr!("uv-nothing"));
        }
        self.uv_commit(&label, list);
    }

    /// Seams of the active context in the 3D view while the UV Editor is open.
    pub(crate) fn seam_overlay(&self, out: &mut varstotas_render::Overlay3d) {
        if !self.uv.open {
            return;
        }
        let Some(mesh) = self.doc.model.mesh_at(&self.doc.active_path) else { return };
        let xf = crate::tools::common::context_xf(&self.doc);
        for ed in mesh.edges.values().filter(|e| e.seam) {
            let (a, b) = (xf.transform_point3(mesh.vertices[ed.v[0]].pos), xf.transform_point3(mesh.vertices[ed.v[1]].pos));
            out.lines.push(varstotas_render::OverlayLine {
                a: a.as_vec3().to_array(),
                b: b.as_vec3().to_array(),
                color: [0.95, 0.2, 0.25, 1.0],
                width_px: 3.0,
                dashed: false,
                on_top: false,
            });
        }
    }

    /// The UV Editor window.
    pub(crate) fn uv_editor_window(&mut self, ctx: &egui::Context) {
        if !self.uv.open {
            return;
        }
        let mut open = true;
        egui::Window::new(tr!("uv-title"))
            .id(egui::Id::new("uv_editor_window"))
            .open(&mut open)
            .default_size([620.0, 640.0])
            .default_pos([260.0, 150.0])
            .resizable(true)
            .show(ctx, |ui| {
                self.uv_toolbar(ui);
                if let Some(f) = self.uv_single_face() {
                    self.uv_face_fields(ui, f);
                }
                ui.separator();
                self.uv_canvas(ui);
            });
        if !open {
            self.uv.open = false;
            self.uv.drag = None;
        }
    }

    /// The one face the face fields edit: the only selected face, else the only face of the
    /// picked islands.
    fn uv_single_face(&self) -> Option<FaceId> {
        let sel: Vec<FaceId> = self.doc.selection.iter().filter_map(|e| if let EntityRef::Face(f) = e { Some(*f) } else { None }).collect();
        match (sel.as_slice(), self.uv.selected.len()) {
            ([f], _) => Some(*f),
            ([], 1) => self.uv.selected.iter().next().copied(),
            _ => None,
        }
    }

    /// The numbers of one face's texture placement (the numeric form of Texture Position):
    /// offset of the texture origin, rotation, scale and the tile size on the face.
    fn uv_face_fields(&mut self, ui: &mut Ui, fid: FaceId) {
        let path = self.doc.active_path.clone();
        let unit = self.doc.model.unit;
        let model = &self.doc.model;
        let Some(face) = model.mesh_at(&path).and_then(|m| m.faces.get(fid)) else { return };
        let Some(tile) = own_tile(model, model.mesh_at(&path).expect("face mesh"), fid).or_else(|| self.uv_fallback_tile(&[fid])) else {
            return;
        };
        let pl = face.uv.unwrap_or_default();
        let texts = [
            super::fmt_len(pl.offset[0], unit),
            super::fmt_len(pl.offset[1], unit),
            format!("{:.1}°", pl.angle_deg),
            format!("{}", pl.scale[0]),
            format!("{}", pl.scale[1]),
            super::fmt_len(tile[0] * pl.scale[0].abs(), unit),
            super::fmt_len(tile[1] * pl.scale[1].abs(), unit),
        ];
        if self.uv.face_bufs.as_ref().map(|t| t.0) != Some(fid) {
            self.uv.face_bufs = Some((fid, texts.clone()));
        }
        let mut new = None;
        let mut bad = false;
        let r = egui::CollapsingHeader::new(egui::RichText::new(tr!("tray-tex-title")).strong()).id_salt("uv_face").default_open(true).show(ui, |ui| {
            let Some((_, bufs)) = self.uv.face_bufs.as_mut() else { return };
            let labels = [
                tr!("tray-tex-offset-x"),
                tr!("tray-tex-offset-y"),
                tr!("tray-tex-rotation"),
                tr!("tray-tex-scale-x"),
                tr!("tray-tex-scale-y"),
                tr!("tray-tex-tile-w"),
                tr!("tray-tex-tile-h"),
            ];
            // pairs side by side: offset, scale, tile size; the rotation last
            egui::Grid::new("uv_face_fields").num_columns(4).spacing([8.0, 4.0]).show(ui, |ui| {
                for row in [[0, 1], [3, 4], [5, 6], [2, usize::MAX]] {
                    for i in row {
                        if i == usize::MAX {
                            continue;
                        }
                        ui.label(&labels[i]);
                        let resp = ui.add(egui::TextEdit::singleline(&mut bufs[i]).desired_width(80.0));
                        super::tray_lib::spot(format!("uv:face{i}"), resp.rect.center());
                        if super::tray::committed(ui, &resp) && bufs[i] != texts[i] {
                            let t = bufs[i].trim();
                            let mut p = pl;
                            let ok = match i {
                                0 | 1 => varstotas_kernel::units::parse_length(t, unit).map(|v| p.offset[i] = v).is_some(),
                                2 => t.trim_end_matches('°').trim().parse::<f64>().map(|v| p.angle_deg = v).is_ok(),
                                3 | 4 => t.parse::<f64>().ok().filter(|v| v.abs() > 1e-6).map(|v| p.scale[i - 3] = v).is_some(),
                                // tile size on the face: the scale keeps its sign (a flip stays)
                                _ => varstotas_kernel::units::parse_length(t, unit)
                                    .filter(|v| *v > 0.0)
                                    .map(|v| p.scale[i - 5] = v / tile[i - 5] * if pl.scale[i - 5] < 0.0 { -1.0 } else { 1.0 })
                                    .is_some(),
                            };
                            if ok {
                                new = Some(p);
                            } else {
                                bad = true;
                            }
                        } else if !resp.has_focus() && bufs[i] != texts[i] {
                            bufs[i] = texts[i].clone();
                        }
                    }
                    ui.end_row();
                }
            });
        });
        super::tray_lib::spot("uv:face", r.header_response.rect.left_center() + egui::vec2(8.0, 0.0));
        if bad {
            self.uv.face_bufs = None;
            self.flash_error(tr!("tray-invalid-value"));
        }
        if let Some(p) = new {
            self.uv.face_bufs = None;
            self.doc.commit(&tr!("tray-undo-texture-position"), |m| {
                if let Some(f) = m.mesh_at_mut(&path).and_then(|mesh| mesh.faces.get_mut(fid)) {
                    f.uv = (!p.is_identity()).then_some(p);
                }
            });
        }
    }


    fn uv_button(&mut self, ui: &mut Ui, label: String, hover: String, op: &str) {
        let r = ui.button(label).on_hover_text(hover);
        super::tray_lib::spot(format!("uv:{op}"), r.rect.center());
        if r.clicked() {
            self.uv_run(op);
        }
    }

    fn uv_toolbar(&mut self, ui: &mut Ui) {
        ui.horizontal_wrapped(|ui| {
            self.uv_button(ui, tr!("uv-unfold"), tr!("uv-unfold-hover"), "unfold");
            let pr = ui.menu_button(tr!("uv-project"), |ui| {
                for (op, label) in [("planar", tr!("uv-planar")), ("box", tr!("uv-box")), ("cylinder", tr!("uv-cylinder")), ("sphere", tr!("uv-sphere"))] {
                    let r = ui.button(label);
                    super::tray_lib::spot(format!("uv:{op}"), r.rect.center());
                    if r.clicked() {
                        self.uv_run(op);
                        ui.close();
                    }
                }
            });
            super::tray_lib::spot("uv:project", pr.response.rect.center());
            ui.separator();
            self.uv_button(ui, tr!("uv-seams-mark"), tr!("uv-seams-mark-hover"), "seams_mark");
            self.uv_button(ui, tr!("uv-seams-clear"), tr!("uv-seams-clear-hover"), "seams_clear");
            self.uv_button(ui, tr!("uv-seams-hard"), tr!("uv-seams-hard-hover"), "seams_hard");
        });
        ui.horizontal_wrapped(|ui| {
            self.uv_button(ui, "⟲".into(), tr!("uv-rotate-ccw"), "rotate_ccw");
            self.uv_button(ui, "⟳".into(), tr!("uv-rotate"), "rotate");
            self.uv_button(ui, "↔".into(), tr!("uv-flip-u"), "flip_u");
            self.uv_button(ui, "↕".into(), tr!("uv-flip-v"), "flip_v");
            ui.separator();
            self.uv_button(ui, tr!("uv-fit"), tr!("uv-fit-hover"), "fit");
            self.uv_button(ui, tr!("uv-pack"), tr!("uv-pack-hover"), "pack");
            self.uv_button(ui, tr!("uv-align"), tr!("uv-align-hover"), "align");
            self.uv_button(ui, tr!("tray-tex-reset"), tr!("uv-reset-hover"), "reset");
        });
        // typed rotation and scale, then Randomize with its options
        ui.horizontal_wrapped(|ui| {
            for (op, field, label, hover, unit) in [("rotate_by", 0, tr!("uv-rotate-by"), tr!("uv-rotate-by-hover"), "°"), ("scale_by", 1, tr!("uv-scale-by"), tr!("uv-scale-by-hover"), "×")] {
                let buf = if field == 0 { &mut self.uv.angle } else { &mut self.uv.scale };
                let hint = if field == 0 { "15" } else { "1.25" };
                let resp = ui.add(egui::TextEdit::singleline(buf).desired_width(44.0).hint_text(hint)).on_hover_text(hover.clone());
                super::tray_lib::spot(format!("uv:{op}_value"), resp.rect.center());
                ui.label(unit);
                let enter = super::tray::committed(ui, &resp) && !buf.trim().is_empty();
                let b = ui.button(label).on_hover_text(hover);
                super::tray_lib::spot(format!("uv:{op}"), b.rect.center());
                if enter || b.clicked() {
                    let v = if buf.trim().is_empty() { hint.to_string() } else { buf.clone() };
                    self.uv_run(&format!("{op} {v}"));
                }
            }
            ui.separator();
            self.uv_button(ui, tr!("uv-randomize"), tr!("uv-randomize-hover"), "randomize");
            let r = ui.checkbox(&mut self.uv.rand_any_angle, tr!("uv-rand-any-angle")).on_hover_text(tr!("uv-rand-any-angle-hover"));
            super::tray_lib::spot("uv:rand_any", r.rect.center());
            ui.label(tr!("uv-rand-scale"));
            let r = ui.add(egui::DragValue::new(&mut self.uv.rand_scale_pct).range(0.0..=90.0).speed(1.0).suffix(" %")).on_hover_text(tr!("uv-rand-scale-hover"));
            super::tray_lib::spot("uv:rand_scale", r.rect.center());
        });
        self.uv_backdrop_row(ui);
        let targets = self.uv_targets();
        let faces: Vec<FaceId> = targets.iter().flatten().copied().collect();
        let now = self.uv_density(&faces);
        ui.horizontal(|ui| {
            ui.label(tr!("uv-density"));
            let resp = ui.add(egui::TextEdit::singleline(&mut self.uv.density).desired_width(70.0).hint_text(now.map(|d| format!("{d:.0}")).unwrap_or_default()));
            super::tray_lib::spot("uv:density", resp.rect.center());
            ui.label(tr!("uv-px-per-m"));
            if super::tray::committed(ui, &resp) && !self.uv.density.trim().is_empty() {
                let v = std::mem::take(&mut self.uv.density);
                self.uv_run(&format!("density {v}"));
            }
            let all = self.uv_islands();
            let picked = all.iter().filter(|i| i.iter().any(|f| self.uv.selected.contains(f))).count();
            ui.label(
                egui::RichText::new(tr!("uv-summary", faces = self.uv_faces().len(), islands = all.len(), selected = picked))
                    .small()
                    .color(theme::text_dim()),
            );
        });
    }

    /// The backdrop row: which material and map, one tile or repeated, the checker.
    fn uv_backdrop_row(&mut self, ui: &mut Ui) {
        let faces = self.uv_faces();
        let mat = self.uv_material(&faces);
        ui.horizontal_wrapped(|ui| {
            if let Some(m) = mat.and_then(|id| self.doc.model.materials.get(id)) {
                let name = m.name.clone();
                let slots = slots_of(m);
                ui.label(egui::RichText::new(tr!("uv-material", name = name)).color(theme::text_dim()));
                let shown = if slots.contains(&self.uv.map) { self.uv.map } else { slots[0] };
                if slots.len() > 1 {
                    let cb = egui::ComboBox::from_id_salt("uv_map").selected_text(super::mat_maps::slot_label(shown)).show_ui(ui, |ui| {
                        for s in &slots {
                            let r = ui.selectable_label(*s == shown, super::mat_maps::slot_label(s));
                            super::tray_lib::spot(format!("uv:map:{s}"), r.rect.center());
                            if r.clicked() {
                                self.uv.map = s;
                            }
                        }
                    });
                    super::tray_lib::spot("uv:map", cb.response.rect.center());
                    cb.response.on_hover_text(tr!("uv-map-hover"));
                } else {
                    ui.label(super::mat_maps::slot_label(shown));
                }
            } else {
                ui.label(egui::RichText::new(tr!("uv-no-texture")).color(theme::text_dim()));
            }
            ui.separator();
            let mut tiled = !self.uv.single_tile;
            let r = ui.checkbox(&mut tiled, tr!("uv-tile")).on_hover_text(tr!("uv-tile-hover"));
            super::tray_lib::spot("uv:tile", r.rect.center());
            self.uv.single_tile = !tiled;
            let mut c = self.uv.checker;
            let r = ui.checkbox(&mut c, tr!("uv-checker")).on_hover_text(tr!("uv-checker-hover"));
            super::tray_lib::spot("uv:checker", r.rect.center());
            if r.changed() {
                self.uv_run("checker");
            }
        });
    }

    /// The backdrop image: the chosen map of [`Self::uv_material`] (or the checker).
    fn uv_background(&mut self, ctx: &egui::Context, faces: &[FaceId]) -> Option<egui::TextureId> {
        if self.uv.checker {
            let t = self.uv.checker_tex.get_or_insert_with(|| ctx.load_texture("uv_checker", checker_image(), egui::TextureOptions::LINEAR_REPEAT));
            return Some(t.id());
        }
        let m = self.doc.model.materials.get(self.uv_material(faces)?)?;
        let slots = slots_of(m);
        let slot = if slots.contains(&self.uv.map) { self.uv.map } else { *slots.first()? };
        let img = slot_image(m, slot)?.clone();
        let key = std::sync::Arc::as_ptr(&img.bytes) as *const u8 as usize;
        if self.uv.image.as_ref().is_none_or(|(k, _)| *k != key) {
            let ci = crate::textures::thumbnail(&img.bytes, 1024)?;
            self.uv.image = Some((key, ctx.load_texture(format!("uv_image{key}"), ci, egui::TextureOptions::LINEAR_REPEAT)));
        }
        self.uv.image.as_ref().map(|(_, t)| t.id())
    }

    fn uv_canvas(&mut self, ui: &mut Ui) {
        let islands = self.uv_islands();
        let faces: Vec<FaceId> = islands.iter().flatten().copied().collect();
        if faces.is_empty() {
            ui.label(egui::RichText::new(tr!("uv-empty")).color(theme::text_dim()));
            return;
        }
        let bg = self.uv_background(ui.ctx(), &faces);
        let fallback = self.uv_fallback_tile(&faces);
        let size = ui.available_size().max(vec2(200.0, 200.0));
        let (rect, resp) = ui.allocate_exact_size(size, Sense::click_and_drag());
        super::tray_lib::spot("uv:canvas", rect.center());
        let Some(mesh) = self.doc.model.mesh_at(&self.doc.active_path) else { return };
        let model = &self.doc.model;
        // face outlines in tile units
        let polys: Vec<(FaceId, Vec<DVec2>)> = faces
            .iter()
            .map(|f| {
                let t = DVec2::from(own_tile(model, mesh, *f).or(fallback).unwrap_or([1000.0; 2]));
                (*f, uv::face_tex(mesh, *f).into_iter().map(|q| q / t).collect())
            })
            .collect();
        if self.uv.zoom <= 0.0 {
            let lo = polys.iter().flat_map(|p| p.1.iter().copied()).fold(DVec2::splat(0.0), DVec2::min);
            let hi = polys.iter().flat_map(|p| p.1.iter().copied()).fold(DVec2::splat(1.0), DVec2::max);
            self.uv.center = (lo + hi) / 2.0;
            let s = (hi - lo).max(DVec2::splat(1e-3));
            self.uv.zoom = ((rect.width() as f64 / s.x).min(rect.height() as f64 / s.y) * 0.9) as f32;
        }
        let (center, zoom) = (self.uv.center, self.uv.zoom as f64);
        let to_screen = |q: DVec2| -> Pos2 { rect.center() + vec2(((q.x - center.x) * zoom) as f32, (-(q.y - center.y) * zoom) as f32) };
        let to_uv = |p: Pos2| -> DVec2 { center + DVec2::new((p.x - rect.center().x) as f64, -(p.y - rect.center().y) as f64) / zoom };
        let painter = ui.painter_at(rect);
        painter.rect_filled(rect, 0.0, Color32::from_gray(38));
        // the image: repeated over the whole canvas (the first tile a little brighter), or one tile
        let (a, b) = (to_uv(rect.left_bottom()), to_uv(rect.right_top()));
        if let Some(bg) = bg {
            let one = Rect::from_two_pos(to_screen(DVec2::new(0.0, 0.0)), to_screen(DVec2::new(1.0, 1.0)));
            if self.uv.single_tile {
                painter.image(bg, one, Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)), Color32::WHITE);
            } else {
                let uvr = Rect::from_min_max(pos2(a.x as f32, 1.0 - b.y as f32), pos2(b.x as f32, 1.0 - a.y as f32));
                painter.image(bg, rect, uvr, Color32::from_gray(205));
                painter.image(bg, one, Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)), Color32::WHITE);
            }
            painter.rect_stroke(one, 0.0, Stroke::new(1.5, Color32::from_rgba_unmultiplied(255, 210, 90, 200)), egui::StrokeKind::Middle);
        }
        // tile grid
        let grid = Stroke::new(1.0, Color32::from_white_alpha(40));
        if (b.x - a.x) < 200.0 {
            for i in a.x.floor() as i64..=b.x.ceil() as i64 {
                painter.line_segment([to_screen(DVec2::new(i as f64, a.y)), to_screen(DVec2::new(i as f64, b.y))], grid);
            }
            for j in a.y.floor() as i64..=b.y.ceil() as i64 {
                painter.line_segment([to_screen(DVec2::new(a.x, j as f64)), to_screen(DVec2::new(b.x, j as f64))], grid);
            }
        }
        // faces: selected ones filled, edges by kind
        let set: HashSet<FaceId> = faces.iter().copied().collect();
        for (f, poly) in &polys {
            let pts: Vec<Pos2> = poly.iter().map(|q| to_screen(*q)).collect();
            if self.uv.selected.contains(f) {
                let tris = varstotas_kernel::triangulate::triangulate(poly, &[]);
                let mut m = egui::Mesh::default();
                for p in &pts {
                    m.colored_vertex(*p, Color32::from_rgba_unmultiplied(255, 150, 40, 70));
                }
                for t in tris {
                    m.add_triangle(t[0] as u32, t[1] as u32, t[2] as u32);
                }
                painter.add(Shape::mesh(m));
            }
            let Some(face) = mesh.faces.get(*f) else { continue };
            let n = face.outer.len();
            for i in 0..n {
                let (va, vb) = (face.outer[i], face.outer[(i + 1) % n]);
                let e = mesh.edge_between(va, vb);
                let seam = e.and_then(|e| mesh.edges.get(e)).is_some_and(|ed| ed.seam);
                // inner: the texture runs on across the edge into the other face
                let (pa, pb) = (mesh.vertices[va].pos, mesh.vertices[vb].pos);
                let inner = e.is_some_and(|e| {
                    mesh.edge_faces(e).iter().filter(|g| **g != *f && set.contains(*g)).any(|g| {
                        let at = |h: FaceId, p| mesh.faces.get(h).map(|x| varstotas_kernel::texture::face_uv_mm(p, x.normal, x.uv.as_ref()));
                        [pa, pb].into_iter().all(|p| matches!((at(*f, p), at(*g, p)), (Some(a), Some(b)) if a.distance(b) < 1e-3))
                    })
                });
                let st = if seam {
                    Stroke::new(2.5, Color32::from_rgb(240, 60, 70))
                } else if inner {
                    Stroke::new(0.8, Color32::from_white_alpha(120))
                } else {
                    Stroke::new(1.6, Color32::WHITE)
                };
                painter.line_segment([pts[i], pts[(i + 1) % n]], st);
            }
        }
        painter.rect_stroke(rect, 0.0, Stroke::new(1.0, theme::border()), egui::StrokeKind::Inside);
        // QA: the centre of each island's first face
        for (i, isl) in islands.iter().enumerate() {
            if let Some((_, poly)) = isl.first().and_then(|f| polys.iter().find(|p| p.0 == *f)) {
                let c = poly.iter().copied().sum::<DVec2>() / poly.len().max(1) as f64;
                super::tray_lib::spot(format!("uv:isl{i}"), to_screen(c));
            }
        }
        let island_at = |p: Pos2| -> Option<usize> {
            let q = to_uv(p);
            let f = polys.iter().rev().find(|(_, poly)| inside(poly, q))?.0;
            islands.iter().position(|i| i.contains(&f))
        };
        // input: wheel zooms about the pointer, middle / right drag pans, primary drag moves
        // (Alt rotates, Cmd/Ctrl scales) the picked islands
        let hover = resp.hover_pos();
        if let Some(h) = hover {
            let scroll = ui.ctx().input(|i| i.smooth_scroll_delta.y);
            if scroll != 0.0 {
                let before = to_uv(h);
                let k = (scroll as f64 / 200.0).exp();
                self.uv.zoom = (self.uv.zoom as f64 * k).clamp(2.0, 1e6) as f32;
                let z = self.uv.zoom as f64;
                self.uv.center = before - DVec2::new((h.x - rect.center().x) as f64, -(h.y - rect.center().y) as f64) / z;
            }
        }
        let mods = ui.ctx().input(|i| i.modifiers);
        if resp.clicked() {
            match hover.and_then(island_at) {
                Some(i) => {
                    let isl: HashSet<FaceId> = islands[i].iter().copied().collect();
                    if mods.shift {
                        if isl.iter().all(|f| self.uv.selected.contains(f)) {
                            self.uv.selected.retain(|f| !isl.contains(f));
                        } else {
                            self.uv.selected.extend(isl);
                        }
                    } else {
                        self.uv.selected = isl;
                    }
                }
                None if !mods.shift => self.uv.selected.clear(),
                None => {}
            }
        }
        if resp.drag_started()
            && let Some(start) = resp.interact_pointer_pos()
        {
            let pan = ui.ctx().input(|i| i.pointer.middle_down() || i.pointer.secondary_down());
            let picked = island_at(start);
            let kind = if pan || picked.is_none() {
                DragKind::Pan
            } else if mods.alt {
                DragKind::Rotate
            } else if mods.command {
                DragKind::Scale
            } else {
                DragKind::Move
            };
            let mut drag_islands = Vec::new();
            if let Some(i) = picked.filter(|_| kind != DragKind::Pan) {
                if !islands[i].iter().any(|f| self.uv.selected.contains(f)) {
                    self.uv.selected = islands[i].iter().copied().collect();
                }
                drag_islands = islands.iter().filter(|i| i.iter().any(|f| self.uv.selected.contains(f))).cloned().collect();
            }
            let fs: Vec<FaceId> = drag_islands.iter().flatten().copied().collect();
            let orig: Placements = fs.iter().map(|f| (*f, mesh.faces.get(*f).and_then(|x| x.uv))).collect();
            let pts: Vec<DVec2> = polys.iter().filter(|(f, _)| fs.contains(f)).flat_map(|p| p.1.iter().copied()).collect();
            let pivot = match (pts.iter().copied().reduce(DVec2::min), pts.iter().copied().reduce(DVec2::max)) {
                (Some(lo), Some(hi)) => to_screen((lo + hi) / 2.0),
                _ => start,
            };
            self.uv.drag = Some(UvDrag { kind, start, islands: drag_islands, orig, pivot, center: self.uv.center });
        }
        if let (Some(d), Some(now)) = (self.uv.drag.as_ref(), resp.interact_pointer_pos().or(hover))
            && resp.dragged()
        {
            let delta = now - d.start;
            if d.kind == DragKind::Pan {
                self.uv.center = d.center - DVec2::new(delta.x as f64, -delta.y as f64) / self.uv.zoom as f64;
            } else {
                let orig = d.orig.clone();
                let isl = d.islands.clone();
                let (kind, pivot, start) = (d.kind, d.pivot, d.start);
                self.uv_set_untracked(&orig);
                let Some(mesh) = self.doc.model.mesh_at(&self.doc.active_path) else { return };
                let mut out = Vec::new();
                for i in &isl {
                    let t = DVec2::from(i.first().map_or([1000.0; 2], |f| own_tile(&self.doc.model, mesh, *f).or(fallback).unwrap_or([1000.0; 2])));
                    let op = match kind {
                        DragKind::Move => IslandOp::Move(DVec2::new(delta.x as f64, -delta.y as f64) / zoom * t),
                        DragKind::Rotate => {
                            let (a0, a1) = (start - pivot, now - pivot);
                            let mut deg = (-(a1.y.atan2(a1.x) - a0.y.atan2(a0.x)) as f64).to_degrees();
                            if mods.shift {
                                deg = (deg / 15.0).round() * 15.0;
                            }
                            IslandOp::Rotate(deg)
                        }
                        _ => IslandOp::Scale(((now - pivot).length() / (start - pivot).length().max(1.0)).max(0.01) as f64),
                    };
                    // rotate / scale every island about the common pivot, not its own centre
                    out.extend(match op {
                        IslandOp::Rotate(deg) => uv::map_faces(mesh, i, uv::rotation(to_uv(pivot) * t, deg)),
                        IslandOp::Scale(k) => {
                            let c = to_uv(pivot) * t;
                            uv::map_faces(mesh, i, move |q| c + (q - c) * k)
                        }
                        op => uv::transform(mesh, i, op),
                    });
                }
                self.uv_set_untracked(&out);
            }
        }
        if resp.drag_stopped()
            && let Some(d) = self.uv.drag.take()
            && d.kind != DragKind::Pan
        {
            let mesh = self.doc.model.mesh_at(&self.doc.active_path);
            let now: Placements = d.orig.iter().map(|(f, _)| (*f, mesh.and_then(|m| m.faces.get(*f)).and_then(|x| x.uv))).collect();
            if now != d.orig {
                self.uv_set_untracked(&d.orig);
                self.uv_commit(&tr!("uv-undo"), now);
            }
        }
    }

    /// Script dump of the backdrop: (material, map shown, one tile only).
    pub(crate) fn uv_view_dump(&self) -> String {
        let faces = self.uv_faces();
        let m = self.uv_material(&faces).and_then(|id| self.doc.model.materials.get(id));
        let slot = m.map(slots_of).and_then(|s| if s.contains(&self.uv.map) { Some(self.uv.map) } else { s.first().copied() });
        format!("({} {} {})", m.map_or("-", |m| m.name.as_str()), slot.unwrap_or("-"), self.uv.single_tile)
    }

    /// Script dump: (open, faces, islands, selected faces, checker, seams in the active context).
    pub(crate) fn uv_dump(&self) -> String {
        let mesh = self.doc.model.mesh_at(&self.doc.active_path);
        let seams = mesh.map_or(0, |m| m.edges.values().filter(|e| e.seam).count());
        let faces = self.uv_faces();
        let islands = mesh.map_or(0, |m| uv::islands(m, &faces).len());
        format!("({} {} {} {} {} {})", self.uv.open, faces.len(), islands, self.uv.selected.len(), self.uv.checker, seams)
    }
}

/// Point `q` inside polygon `poly` (even-odd).
fn inside(poly: &[DVec2], q: DVec2) -> bool {
    let mut c = false;
    let n = poly.len();
    for i in 0..n {
        let (a, b) = (poly[i], poly[(i + 1) % n]);
        if (a.y > q.y) != (b.y > q.y) && q.x < a.x + (q.y - a.y) / (b.y - a.y) * (b.x - a.x) {
            c = !c;
        }
    }
    c
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn point_in_polygon_and_checker() {
        let sq = [DVec2::new(0.0, 0.0), DVec2::new(1.0, 0.0), DVec2::new(1.0, 1.0), DVec2::new(0.0, 1.0)];
        assert!(inside(&sq, DVec2::new(0.5, 0.5)));
        assert!(!inside(&sq, DVec2::new(1.5, 0.5)));
        let img = checker_image();
        assert_eq!(img.size, [CHECKER_PX, CHECKER_PX]);
    }
}
