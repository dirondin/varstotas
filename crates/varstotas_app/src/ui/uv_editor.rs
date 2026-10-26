//! UV Editor: the texture layout of the selected faces (else every textured face of the
//! active context) drawn over the material's image, one tile per square. Faces joined by
//! shared edges that are not seams form islands. Islands (or single faces in face mode) are
//! picked by click or box and moved, turned and scaled right on the canvas with the Move,
//! Rotate and Scale tools (about a pivot that can be dragged; snapping to corners and the
//! tile grid, angle / scale / move increments; exact values typed in the status bar). The
//! toolbar unfolds, projects, turns by 90°, flips, fits, packs and aligns, resets the turn,
//! the scale or the whole placement, and marks seams. The backdrop is one map of the
//! material (colour, normal, roughness, …), one tile or repeated; a checker can stand in for
//! the textures (here and in the 3D view).

use super::icons::Icon;
use super::icons::uv::UvIcon;
use super::theme;
use crate::app::VarstotasApp;
use crate::inference::settings::round_to;
use egui::{Color32, Pos2, Rect, Sense, Shape, Stroke, Ui, pos2, vec2};
use std::collections::HashSet;
use varstotas_i18n::tr;
use varstotas_kernel::glam::DVec2;
use varstotas_kernel::uv::{self, IslandOp, Placements, Projection};
use varstotas_kernel::{EdgeId, EntityRef, FaceId, Material, MaterialId, Mesh, Model};

/// Pixel side of the checker texture and its squares per tile.
const CHECKER_PX: usize = 512;
const CHECKER_SQUARES: usize = 8;
/// Screen distance (points) within which the pointer catches the pivot and points snap.
const CATCH: f32 = 10.0;
/// Height of the status bar.
const STATUS_H: f32 = 22.0;

/// The canvas tool.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum UvTool {
    /// Click / box picking; dragging a picked island moves it.
    #[default]
    Select,
    Move,
    Rotate,
    Scale,
}

impl UvTool {
    pub const ALL: [UvTool; 4] = [UvTool::Select, UvTool::Move, UvTool::Rotate, UvTool::Scale];

    pub fn key(self) -> &'static str {
        match self {
            UvTool::Select => "select",
            UvTool::Move => "move",
            UvTool::Rotate => "rotate",
            UvTool::Scale => "scale",
        }
    }

    fn icon(self) -> Icon {
        match self {
            UvTool::Select => Icon::Select,
            UvTool::Move => Icon::Move,
            UvTool::Rotate => Icon::Rotate,
            UvTool::Scale => Icon::Scale,
        }
    }

    fn hover(self) -> String {
        match self {
            UvTool::Select => tr!("uv-tool-select"),
            UvTool::Move => tr!("uv-tool-move"),
            UvTool::Rotate => tr!("uv-tool-rotate"),
            UvTool::Scale => tr!("uv-tool-scale"),
        }
    }

    fn hint(self) -> String {
        match self {
            UvTool::Select => tr!("uv-hint-select"),
            UvTool::Move => tr!("uv-hint-move"),
            UvTool::Rotate => tr!("uv-hint-rotate"),
            UvTool::Scale => tr!("uv-hint-scale"),
        }
    }

    /// The name of the value typed in the status bar.
    fn value_label(self) -> String {
        match self {
            UvTool::Select | UvTool::Move => tr!("uv-value-move"),
            UvTool::Rotate => tr!("uv-value-rotate"),
            UvTool::Scale => tr!("uv-value-scale"),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum DragKind {
    Move,
    Rotate,
    Scale,
    Pan,
    /// Box picking.
    Marquee,
    /// Dragging the pivot.
    Pivot,
}

/// A transform of picked faces: a move (tile units, or texture mm when typed), a turn
/// (degrees, anticlockwise) or a scale (per axis) about the pivot.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Xform {
    Move(DVec2),
    MoveMm(DVec2),
    Rotate(f64),
    Scale(DVec2),
}

/// A part of the gizmo at the pivot.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Handle {
    /// The centre: moves the pivot (Rotate / Scale), or moves freely (Move).
    Center,
    /// One axis (0 = across, 1 = up): move or scale along it only.
    Axis(usize),
    /// Scale both axes alike.
    Uniform,
}

/// Gizmo arm length (points).
const ARM: f32 = 56.0;

/// The gizmo part under `p` (`g` = the pivot on screen).
fn gizmo_hit(tool: UvTool, g: Pos2, p: Pos2) -> Option<Handle> {
    if tool == UvTool::Select {
        return None;
    }
    let d = p - g;
    if d.length() <= CATCH {
        return Some(Handle::Center);
    }
    if tool == UvTool::Scale && (d - vec2(ARM * 0.62, -ARM * 0.62)).length() <= 8.0 {
        return Some(Handle::Uniform);
    }
    if matches!(tool, UvTool::Move | UvTool::Scale) {
        if d.y.abs() <= 6.0 && d.x > CATCH && d.x <= ARM + 8.0 {
            return Some(Handle::Axis(0));
        }
        if d.x.abs() <= 6.0 && -d.y > CATCH && -d.y <= ARM + 8.0 {
            return Some(Handle::Axis(1));
        }
    }
    None
}

struct UvDrag {
    kind: DragKind,
    /// Constrained to one axis (gizmo arm).
    axis: Option<usize>,
    start: Pos2,
    /// Picked faces grouped by island, with their placements at the start.
    groups: Vec<Vec<FaceId>>,
    orig: Placements,
    /// Pivot (tile units) of a turn / scale.
    pivot: DVec2,
    /// View centre at the start (pan).
    center: DVec2,
    /// The picked corner nearest the press (tile units): it snaps when moving.
    grab: Option<DVec2>,
    /// What the drag does now, and the point it snapped to (tile units).
    now: Option<Xform>,
    snapped: Option<DVec2>,
}

/// The last transform (drag or typed), so a value typed next replaces it.
struct UvLast {
    tool: UvTool,
    version: u64,
    pivot: DVec2,
    groups: Vec<Vec<FaceId>>,
}

/// UV Editor state.
#[derive(Default)]
pub struct UvState {
    pub open: bool,
    /// Picked faces (whole islands, or single faces in face mode).
    pub selected: HashSet<FaceId>,
    pub tool: UvTool,
    /// Clicks pick single faces (else whole islands).
    pub face_mode: bool,
    /// Pivot of Rotate / Scale (tile units); `None` = the centre of the picked faces.
    pub pivot: Option<DVec2>,
    /// View centre (tile units) and zoom (points per tile); 0 = frame the islands.
    center: DVec2,
    zoom: f32,
    drag: Option<UvDrag>,
    last: Option<UvLast>,
    /// The last click (screen point) and which of the overlapping faces / islands under it
    /// it picked: clicking there again picks the next one down.
    cycle: Option<(Pos2, usize)>,
    /// The status bar value (shows the drag's, takes a typed one).
    pub vcb: String,
    /// The checker stands in for textures (UV Editor and 3D view).
    pub checker: bool,
    density: String,
    /// The backdrop's map slot (`textures::SLOTS`; empty = the first the material has).
    pub map: &'static str,
    /// The backdrop shows one tile only (else repeated).
    pub single_tile: bool,
    image: Option<(usize, egui::TextureHandle)>,
    checker_tex: Option<egui::TextureHandle>,
    /// (model version, faces) → islands.
    cache: Option<(u64, Vec<FaceId>, Vec<Vec<FaceId>>)>,
}

/// Text of a transform for the status bar.
fn xform_text(x: Xform, unit: varstotas_kernel::LengthUnit) -> String {
    let num = |v: f64| {
        let t = format!("{v:.3}");
        t.trim_end_matches('0').trim_end_matches('.').to_string()
    };
    match x {
        Xform::Rotate(d) => format!("{}°", num(d)),
        Xform::Scale(k) if (k.x - k.y).abs() < 1e-9 => num(k.x),
        Xform::Scale(k) => format!("{}; {}", num(k.x), num(k.y)),
        Xform::MoveMm(d) => format!("{}; {}", super::fmt_len(d.x, unit), super::fmt_len(d.y, unit)),
        Xform::Move(_) => String::new(),
    }
}

/// Tile size (mm) of a face's own textured material (front, else back).
fn own_tile(model: &Model, mesh: &Mesh, f: FaceId) -> Option<[f64; 2]> {
    let face = mesh.faces.get(f)?;
    [face.front, face.back].into_iter().flatten().find_map(|id| model.materials.get(id)?.image.as_ref().map(|i| i.size))
}

/// The map slots a material has images in.
fn slots_of(m: &Material) -> Vec<&'static str> {
    crate::textures::SLOTS.into_iter().filter(|s| m.slot_image(s).is_some()).collect()
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
        slots_of(m).first().and_then(|s| m.slot_image(s)).map(|i| i.size)
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

    /// The picked faces grouped by island, else every island.
    fn uv_targets(&mut self) -> Vec<Vec<FaceId>> {
        let all = self.uv_islands();
        let sel = picked_groups(&all, &self.uv.selected);
        if sel.is_empty() { all } else { sel }
    }

    /// Face outlines in tile units.
    fn uv_polys(&self, mesh: &Mesh, faces: &[FaceId], fallback: Option<[f64; 2]>) -> Vec<(FaceId, Vec<DVec2>)> {
        faces
            .iter()
            .map(|f| {
                let t = DVec2::from(self.uv_tile(mesh, *f, fallback));
                (*f, uv::face_tex(mesh, *f).into_iter().map(|q| q / t).collect())
            })
            .collect()
    }

    /// The pivot of Rotate / Scale (tile units): the set one, else the centre of the picked
    /// faces (else of all of them).
    pub(crate) fn uv_pivot(&mut self) -> DVec2 {
        if let Some(p) = self.uv.pivot {
            return p;
        }
        let faces: Vec<FaceId> = self.uv_targets().into_iter().flatten().collect();
        let fallback = self.uv_fallback_tile(&faces);
        let Some(mesh) = self.doc.model.mesh_at(&self.doc.active_path) else { return DVec2::ZERO };
        let pts: Vec<DVec2> = self.uv_polys(mesh, &faces, fallback).into_iter().flat_map(|p| p.1).collect();
        match (pts.iter().copied().reduce(DVec2::min), pts.iter().copied().reduce(DVec2::max)) {
            (Some(lo), Some(hi)) => (lo + hi) / 2.0,
            _ => DVec2::ZERO,
        }
    }

    /// Placements after transform `x` of the groups about `pivot` (tile units; each island
    /// in its own tile's millimetres).
    fn uv_xform(&self, groups: &[Vec<FaceId>], x: Xform, pivot: DVec2, fallback: Option<[f64; 2]>) -> Placements {
        let Some(mesh) = self.doc.model.mesh_at(&self.doc.active_path) else { return Vec::new() };
        let mut out = Vec::new();
        for g in groups {
            let t = DVec2::from(g.first().map_or([1000.0; 2], |f| self.uv_tile(mesh, *f, fallback)));
            let c = pivot * t;
            out.extend(match x {
                Xform::Move(d) => uv::map_faces(mesh, g, |q| q + d * t),
                Xform::MoveMm(d) => uv::map_faces(mesh, g, |q| q + d),
                Xform::Rotate(deg) => uv::map_faces(mesh, g, uv::rotation(c, deg)),
                Xform::Scale(k) => uv::map_faces(mesh, g, |q| c + (q - c) * k),
            });
        }
        out
    }

    /// Apply a typed status bar value with the current tool: right after a drag (or another
    /// typed value) of the same tool it replaces that one, else it transforms the picked faces.
    pub(crate) fn uv_typed(&mut self, text: &str) {
        let tool = self.uv.tool;
        let unit = self.doc.model.unit;
        let parts: Vec<&str> = text.split(';').map(str::trim).filter(|t| !t.is_empty()).collect();
        let num = |t: &str| -> Option<f64> { t.trim_end_matches(['°', '×', 'x']).trim().replace(',', ".").parse().ok().filter(|v: &f64| v.is_finite()) };
        let len = |t: &str| varstotas_kernel::units::parse_length(t, unit);
        let k_ok = |k: f64| k.abs() > 1e-3;
        let x = match (tool, parts.as_slice()) {
            (UvTool::Rotate, [a]) => num(a).map(Xform::Rotate),
            (UvTool::Scale, [k]) => num(k).filter(|k| k_ok(*k)).map(|k| Xform::Scale(DVec2::splat(k))),
            (UvTool::Scale, [a, b]) => num(a).zip(num(b)).filter(|(a, b)| k_ok(*a) && k_ok(*b)).map(|(a, b)| Xform::Scale(DVec2::new(a, b))),
            (UvTool::Select | UvTool::Move, [a, b]) => len(a).zip(len(b)).map(|(a, b)| Xform::MoveMm(DVec2::new(a, b))),
            _ => None,
        };
        let Some(x) = x else {
            self.flash_error(tr!("tray-invalid-value"));
            return;
        };
        let again = self.uv.last.as_ref().filter(|l| l.tool == tool && l.version == self.doc.model_version).map(|l| (l.pivot, l.groups.clone()));
        let (pivot, groups) = match again {
            Some(a) if self.doc.undo_last_commit().is_some() => a,
            _ => (self.uv_pivot(), self.uv_targets()),
        };
        let faces: Vec<FaceId> = groups.iter().flatten().copied().collect();
        let list = self.uv_xform(&groups, x, pivot, self.uv_fallback_tile(&faces));
        self.uv_commit_last(list, pivot, groups);
        self.uv.vcb = xform_text(x, unit);
    }

    /// Commit a transform and remember it for a typed value to replace.
    fn uv_commit_last(&mut self, list: Placements, pivot: DVec2, groups: Vec<Vec<FaceId>>) {
        let before = self.doc.model_version;
        self.uv_commit(&tr!("uv-undo"), list);
        self.uv.last = (self.doc.model_version != before).then_some(UvLast { tool: self.uv.tool, version: self.doc.model_version, pivot, groups });
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
    /// texture on the model shrinks), `reset`, `reset_rotation`, `reset_scale`,
    /// `density <px/m>`, `seams_mark`, `seams_clear`, `seams_hard`, `checker`, `tile`,
    /// `select_all`, `select_none`, `frame`, `tool <select|move|rotate|scale>`,
    /// `mode <island|face>`, `pivot <u> <v>` (tile units) or `pivot center`, `type <value>` (the
    /// status bar value of the current tool).
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
            "reset_rotation" => per_island(&|m, i| uv::reset(m, i, true, false)),
            "reset_scale" => per_island(&|m, i| uv::reset(m, i, false, true)),
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
            "tool" => {
                match UvTool::ALL.into_iter().find(|t| t.key() == arg.trim()) {
                    Some(t) => {
                        self.uv.tool = t;
                        self.uv.vcb.clear();
                    }
                    None => log::warn!("uv: unknown tool `{arg}`"),
                }
                return;
            }
            "mode" => {
                self.uv.face_mode = arg.trim() == "face";
                return;
            }
            "pivot" => {
                let v: Vec<f64> = arg.split_whitespace().filter_map(|t| t.parse().ok()).collect();
                self.uv.pivot = if let [u, v] = v[..] { Some(DVec2::new(u, v)) } else { None };
                return;
            }
            "type" => {
                self.uv_typed(arg);
                return;
            }
            "tile" => {
                self.uv.single_tile = !self.uv.single_tile;
                return;
            }
            "checker" => {
                self.uv.checker = !self.uv.checker;
                self.viewport.checker = self.uv.checker;
                return;
            }
            "select_all" => {
                self.uv.selected = self.uv_islands().into_iter().flatten().collect();
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
                ui.separator();
                // the canvas takes exactly what the status bar leaves, and the status bar is a
                // fixed rectangle: the content never pushes the window larger
                let h = (ui.available_height() - STATUS_H - ui.spacing().item_spacing.y).max(120.0);
                self.uv_canvas(ui, vec2(ui.available_width(), h));
                let (r, _) = ui.allocate_exact_size(vec2(ui.available_width(), STATUS_H), Sense::hover());
                let mut bar = ui.new_child(egui::UiBuilder::new().max_rect(r).layout(egui::Layout::left_to_right(egui::Align::Center)));
                bar.set_clip_rect(r.intersect(ui.clip_rect()));
                self.uv_status_bar(&mut bar);
            });
        if !open {
            self.uv.open = false;
            self.uv.drag = None;
        }
    }

    /// A toolbar icon button running `op`.
    fn uv_icon(&mut self, ui: &mut Ui, icon: Icon, selected: bool, hover: String, op: &str) {
        let r = super::icon_button(ui, icon, selected, 28.0).on_hover_text(hover);
        super::tray_lib::spot(format!("uv:{op}"), r.rect.center());
        if r.clicked() {
            self.uv_run(op);
        }
    }

    fn uv_toolbar(&mut self, ui: &mut Ui) {
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing.x = 2.0;
            for t in UvTool::ALL {
                let r = super::icon_button(ui, t.icon(), self.uv.tool == t, 28.0).on_hover_text(t.hover());
                super::tray_lib::spot(format!("uv:tool:{}", t.key()), r.rect.center());
                if r.clicked() && self.uv.tool != t {
                    self.uv.tool = t;
                    self.uv.vcb.clear();
                }
            }
            sep(ui);
            for (face, icon, hover) in [(false, UvIcon::IslandMode, tr!("uv-mode-island")), (true, UvIcon::FaceMode, tr!("uv-mode-face"))] {
                let r = super::icon_button(ui, Icon::Uv(icon), self.uv.face_mode == face, 28.0).on_hover_text(hover);
                super::tray_lib::spot(format!("uv:mode:{}", if face { "face" } else { "island" }), r.rect.center());
                if r.clicked() && self.uv.face_mode != face {
                    self.uv.face_mode = face;
                    self.uv.selected.clear();
                }
            }
            self.uv_icon(ui, Icon::SelectAll, false, tr!("uv-select-all"), "select_all");
            self.uv_icon(ui, Icon::ZoomExtents, false, tr!("uv-frame"), "frame");
            sep(ui);
            self.uv_icon(ui, Icon::Unfold, false, tr!("uv-unfold-hover"), "unfold");
            let pr = super::icon_button(ui, Icon::Uv(UvIcon::Planar), false, 28.0).on_hover_text(tr!("uv-project-hover"));
            super::tray_lib::spot("uv:project", pr.rect.center());
            egui::Popup::menu(&pr).show(|ui| {
                let items = [
                    ("planar", Icon::Uv(UvIcon::Planar), tr!("uv-planar")),
                    ("box", Icon::Box3d, tr!("uv-box")),
                    ("cylinder", Icon::Cylinder, tr!("uv-cylinder")),
                    ("sphere", Icon::Sphere, tr!("uv-sphere")),
                ];
                for (op, icon, label) in items {
                    let r = super::menu::icon_row(ui, Some(icon), &label, None, true, false);
                    super::tray_lib::spot(format!("uv:{op}"), r.rect.center());
                    if r.clicked() {
                        self.uv_run(op);
                        ui.close();
                    }
                }
            });
            sep(ui);
            self.uv_icon(ui, Icon::Uv(UvIcon::RotateCcw), false, tr!("uv-rotate-ccw"), "rotate_ccw");
            self.uv_icon(ui, Icon::Uv(UvIcon::RotateCw), false, tr!("uv-rotate"), "rotate");
            self.uv_icon(ui, Icon::Uv(UvIcon::FlipU), false, tr!("uv-flip-u"), "flip_u");
            self.uv_icon(ui, Icon::Uv(UvIcon::FlipV), false, tr!("uv-flip-v"), "flip_v");
            sep(ui);
            self.uv_icon(ui, Icon::Uv(UvIcon::Fit), false, tr!("uv-fit-hover"), "fit");
            self.uv_icon(ui, Icon::Uv(UvIcon::Pack), false, tr!("uv-pack-hover"), "pack");
            self.uv_icon(ui, Icon::Uv(UvIcon::Align), false, tr!("uv-align-hover"), "align");
            sep(ui);
            self.uv_icon(ui, Icon::Uv(UvIcon::ResetRotation), false, tr!("uv-reset-rotation"), "reset_rotation");
            self.uv_icon(ui, Icon::Uv(UvIcon::ResetScale), false, tr!("uv-reset-scale"), "reset_scale");
            self.uv_icon(ui, Icon::Uv(UvIcon::Reset), false, tr!("uv-reset-hover"), "reset");
            sep(ui);
            self.uv_icon(ui, Icon::Uv(UvIcon::SeamMark), false, tr!("uv-seams-mark-hover"), "seams_mark");
            self.uv_icon(ui, Icon::Uv(UvIcon::SeamClear), false, tr!("uv-seams-clear-hover"), "seams_clear");
            self.uv_icon(ui, Icon::Uv(UvIcon::SeamHard), false, tr!("uv-seams-hard-hover"), "seams_hard");
            sep(ui);
            self.uv_backdrop(ui);
        });
    }

    /// The backdrop controls: the map drop-down, one tile or repeated, the checker.
    fn uv_backdrop(&mut self, ui: &mut Ui) {
        let faces = self.uv_faces();
        let mat = self.uv_material(&faces);
        if let Some(m) = mat.and_then(|id| self.doc.model.materials.get(id)) {
            let name = m.name.clone();
            let slots = slots_of(m);
            let shown = if slots.contains(&self.uv.map) { self.uv.map } else { slots[0] };
            ui.add_space(4.0);
            let cb = egui::ComboBox::from_id_salt("uv_map").width(110.0).selected_text(super::mat_maps::slot_label(shown)).show_ui(ui, |ui| {
                for s in &slots {
                    let r = ui.selectable_label(*s == shown, super::mat_maps::slot_label(s));
                    super::tray_lib::spot(format!("uv:map:{s}"), r.rect.center());
                    if r.clicked() {
                        self.uv.map = s;
                    }
                }
            });
            super::tray_lib::spot("uv:map", cb.response.rect.center());
            cb.response.on_hover_text(format!("{}\n{}", tr!("uv-material", name = name), tr!("uv-map-hover")));
            ui.add_space(4.0);
        } else {
            ui.label(egui::RichText::new(tr!("uv-no-texture")).small().color(theme::text_dim()));
        }
        self.uv_icon(ui, Icon::Uv(UvIcon::Tile), !self.uv.single_tile, tr!("uv-tile-hover"), "tile");
        self.uv_icon(ui, Icon::Uv(UvIcon::Checker), self.uv.checker, tr!("uv-checker-hover"), "checker");
    }

    /// The status bar: what the tool does, the value field (shows a drag's value, takes an
    /// exact one), texel density and the counts.
    fn uv_status_bar(&mut self, ui: &mut Ui) {
        let targets = self.uv_targets();
        let faces: Vec<FaceId> = targets.iter().flatten().copied().collect();
        let now = self.uv_density(&faces);
        let all = self.uv_islands();
        let picked = picked_groups(&all, &self.uv.selected).len();
        let n_faces = self.uv_faces().len();
        // the fields sit on the right (laid out right to left, so in reverse order), the hint
        // takes what is left on the left
        let full = ui.max_rect();
        let mut right = ui.new_child(egui::UiBuilder::new().max_rect(full).layout(egui::Layout::right_to_left(egui::Align::Center)));
        let ui = &mut right;
        ui.label(egui::RichText::new(tr!("uv-summary", faces = n_faces, islands = all.len(), selected = picked)).small().color(theme::text_dim()));
        ui.separator();
        ui.label(egui::RichText::new(tr!("uv-px-per-m")).small());
        let resp = ui
            .add(egui::TextEdit::singleline(&mut self.uv.density).desired_width(56.0).hint_text(now.map(|d| format!("{d:.0}")).unwrap_or_default()))
            .on_hover_text(tr!("uv-density"));
        super::tray_lib::spot("uv:density", resp.rect.center());
        if super::tray::committed(ui, &resp) && !self.uv.density.trim().is_empty() {
            let v = std::mem::take(&mut self.uv.density);
            self.uv_run(&format!("density {v}"));
        }
        ui.separator();
        let out = egui::TextEdit::singleline(&mut self.uv.vcb).desired_width(110.0).show(ui);
        // focusing the field selects its text: what is typed replaces the shown value
        if out.response.gained_focus() {
            let mut state = out.state.clone();
            state.cursor.set_char_range(Some(egui::text::CCursorRange::two(egui::text::CCursor::new(0), egui::text::CCursor::new(self.uv.vcb.chars().count()))));
            state.store(ui.ctx(), out.response.id);
        }
        let resp = out.response.response.on_hover_text(tr!("uv-value-hover"));
        super::tray_lib::spot("uv:value", resp.rect.center());
        if super::tray::committed(ui, &resp) && ui.input(|i| i.key_pressed(egui::Key::Enter)) && !self.uv.vcb.trim().is_empty() {
            let v = self.uv.vcb.clone();
            self.uv_typed(&v);
        }
        ui.label(egui::RichText::new(self.uv.tool.value_label()).small());
        let used = ui.min_rect().left() - ui.spacing().item_spacing.x * 2.0;
        if used > full.left() + 20.0 {
            let r = Rect::from_min_max(full.min, pos2(used, full.max.y));
            let mut left = ui.new_child(egui::UiBuilder::new().max_rect(r).layout(egui::Layout::left_to_right(egui::Align::Center)));
            left.set_clip_rect(r.intersect(ui.clip_rect()));
            left.add(egui::Label::new(egui::RichText::new(self.uv.tool.hint()).small().color(theme::text_dim())).truncate());
        }
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
        let img = m.slot_image(slot)?.clone();
        let key = std::sync::Arc::as_ptr(&img.bytes) as *const u8 as usize;
        if self.uv.image.as_ref().is_none_or(|(k, _)| *k != key) {
            let ci = crate::textures::thumbnail(&img.bytes, 1024)?;
            self.uv.image = Some((key, ctx.load_texture(format!("uv_image{key}"), ci, egui::TextureOptions::LINEAR_REPEAT)));
        }
        self.uv.image.as_ref().map(|(_, t)| t.id())
    }

    fn uv_canvas(&mut self, ui: &mut Ui, size: egui::Vec2) {
        let islands = self.uv_islands();
        let faces: Vec<FaceId> = islands.iter().flatten().copied().collect();
        if faces.is_empty() {
            ui.label(egui::RichText::new(tr!("uv-empty")).color(theme::text_dim()));
            return;
        }
        let bg = self.uv_background(ui.ctx(), &faces);
        let fallback = self.uv_fallback_tile(&faces);
        let (rect, resp) = ui.allocate_exact_size(size, Sense::click_and_drag());
        super::tray_lib::spot("uv:canvas", rect.center());
        super::tray_lib::spot("uv:canvas_tl", rect.min + vec2(6.0, 6.0));
        let pivot = self.uv_pivot();
        let Some(mesh) = self.doc.model.mesh_at(&self.doc.active_path) else { return };
        let polys = self.uv_polys(mesh, &faces, fallback);
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
        // faces: picked ones filled, edges by kind
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
                } else if self.uv.selected.contains(f) {
                    Stroke::new(1.8, Color32::from_rgb(255, 170, 70))
                } else {
                    Stroke::new(1.6, Color32::WHITE)
                };
                painter.line_segment([pts[i], pts[(i + 1) % n]], st);
            }
        }
        painter.rect_stroke(rect, 0.0, Stroke::new(1.0, theme::border()), egui::StrokeKind::Inside);
        // QA: the centre of each island's first face, and of every face
        for (i, isl) in islands.iter().enumerate() {
            if let Some((_, poly)) = isl.first().and_then(|f| polys.iter().find(|p| p.0 == *f)) {
                super::tray_lib::spot(format!("uv:isl{i}"), to_screen(centroid(poly)));
            }
        }
        for (i, (_, poly)) in polys.iter().enumerate() {
            super::tray_lib::spot(format!("uv:face_at{i}"), to_screen(centroid(poly)));
        }
        let tool = self.uv.tool;
        let g = to_screen(pivot);
        super::tray_lib::spot("uv:pivot", g);
        super::tray_lib::spot("uv:gizmo_x", g + vec2(ARM * 0.8, 0.0));
        super::tray_lib::spot("uv:gizmo_y", g + vec2(0.0, -ARM * 0.8));
        super::tray_lib::spot("uv:gizmo_uniform", g + vec2(ARM * 0.62, -ARM * 0.62));
        let face_at = |p: Pos2| -> Option<FaceId> {
            let q = to_uv(p);
            polys.iter().rev().find(|(_, poly)| inside(poly, q)).map(|x| x.0)
        };
        let face_mode = self.uv.face_mode;
        let part_of = |f: FaceId| -> Vec<FaceId> {
            if face_mode { vec![f] } else { islands.iter().find(|i| i.contains(&f)).cloned().unwrap_or_default() }
        };
        let hit = |p: Pos2| gizmo_hit(tool, g, p);
        let pivot_tool = matches!(tool, UvTool::Rotate | UvTool::Scale);
        // a point snapped to a corner of the faces not in `moving`, or to a tile grid corner
        let snap_on = self.prefs.snapping.enabled;
        let snap = |q: DVec2, moving: &HashSet<FaceId>| -> Option<DVec2> {
            let reach = CATCH as f64 / zoom;
            let corners = polys.iter().filter(|(f, _)| !moving.contains(f)).flat_map(|p| p.1.iter().copied());
            corners.chain(std::iter::once(q.round())).filter(|c| c.distance(q) <= reach).min_by(|a, b| a.distance(q).total_cmp(&b.distance(q)))
        };
        // input: wheel zooms about the pointer, middle / right drag pans
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
        if resp.double_clicked() && pivot_tool && hover.and_then(hit) == Some(Handle::Center) {
            self.uv.pivot = None;
        } else if resp.clicked() && hover.and_then(hit).is_none() {
            // every face (island) under the pointer, topmost first; a click on the same spot
            // again goes one further down, so faces lying under others can be picked too
            let stack: Vec<FaceId> = hover.map_or_else(Vec::new, |h| {
                let q = to_uv(h);
                let mut out: Vec<FaceId> = Vec::new();
                for (f, poly) in polys.iter().rev() {
                    if inside(poly, q) && !out.iter().any(|g| part_of(*g).contains(f)) {
                        out.push(*f);
                    }
                }
                out
            });
            let k = match (self.uv.cycle, hover) {
                (Some((p, k)), Some(h)) if p.distance(h) <= 4.0 && !stack.is_empty() => (k + 1) % stack.len(),
                _ => 0,
            };
            self.uv.cycle = hover.map(|h| (h, k));
            match stack.get(k).copied() {
                Some(f) => {
                    let part: HashSet<FaceId> = part_of(f).into_iter().collect();
                    if mods.shift {
                        if part.iter().all(|f| self.uv.selected.contains(f)) {
                            self.uv.selected.retain(|f| !part.contains(f));
                        } else {
                            self.uv.selected.extend(part);
                        }
                    } else {
                        self.uv.selected = part;
                    }
                }
                None if !mods.shift => self.uv.selected.clear(),
                None => {}
            }
        }
        if resp.drag_started()
            && let Some(start) = ui.input(|i| i.pointer.press_origin()).or(resp.interact_pointer_pos())
        {
            let pan = ui.ctx().input(|i| i.pointer.middle_down() || i.pointer.secondary_down());
            // a press on a spot where a picked face lies (even under others, found by clicking
            // through the stack) drags that pick; only a release without a drag cycles
            let q0 = to_uv(start);
            let at = polys.iter().rev().find(|(f, poly)| self.uv.selected.contains(f) && inside(poly, q0)).map(|x| x.0).or_else(|| face_at(start));
            let has_sel = !self.uv.selected.is_empty();
            let acts = at.is_some() || has_sel;
            let (kind, axis) = match (tool, hit(start)) {
                _ if pan => (DragKind::Pan, None),
                (UvTool::Rotate | UvTool::Scale, Some(Handle::Center)) => (DragKind::Pivot, None),
                (UvTool::Move, Some(Handle::Center)) => (DragKind::Move, None),
                (UvTool::Move, Some(Handle::Axis(a))) => (DragKind::Move, Some(a)),
                (UvTool::Scale, Some(Handle::Axis(a))) => (DragKind::Scale, Some(a)),
                (UvTool::Scale, Some(_)) => (DragKind::Scale, None),
                (UvTool::Select, _) if at.is_some() => (DragKind::Move, None),
                (UvTool::Move, _) if acts => (DragKind::Move, None),
                (UvTool::Rotate, _) if acts => (DragKind::Rotate, None),
                (UvTool::Scale, _) if acts => (DragKind::Scale, None),
                _ => (DragKind::Marquee, None),
            };
            // pressing on a face not picked picks it (Select always, the others with nothing picked)
            if matches!(kind, DragKind::Move | DragKind::Rotate | DragKind::Scale)
                && let Some(f) = at
                && !self.uv.selected.contains(&f)
                && (tool == UvTool::Select || !has_sel)
            {
                self.uv.selected = part_of(f).into_iter().collect();
            }
            // nothing picked: the gizmo works on everything
            let mut groups = picked_groups(&islands, &self.uv.selected);
            if groups.is_empty() && matches!(kind, DragKind::Move | DragKind::Rotate | DragKind::Scale) {
                groups = islands.clone();
            }
            let moving: HashSet<FaceId> = groups.iter().flatten().copied().collect();
            let orig: Placements = moving.iter().map(|f| (*f, mesh.faces.get(*f).and_then(|x| x.uv))).collect();
            let pts: Vec<DVec2> = polys.iter().filter(|(f, _)| moving.contains(f)).flat_map(|p| p.1.iter().copied()).collect();
            let grab = pts.iter().filter(|_| axis.is_none()).copied().filter(|q| to_screen(*q).distance(start) <= CATCH * 1.5).min_by(|a, b| to_screen(*a).distance(start).total_cmp(&to_screen(*b).distance(start)));
            let pivot = match (self.uv.pivot, pts.iter().copied().reduce(DVec2::min), pts.iter().copied().reduce(DVec2::max)) {
                (Some(p), _, _) => p,
                (None, Some(lo), Some(hi)) => (lo + hi) / 2.0,
                _ => pivot,
            };
            self.uv.drag = Some(UvDrag { kind, axis, start, groups, orig, pivot, center: self.uv.center, grab, now: None, snapped: None });
        }
        let pointer = resp.interact_pointer_pos().or(hover);
        let inc = self.prefs.snapping.increments;
        if let (Some(d), Some(now)) = (self.uv.drag.as_ref(), pointer)
            && resp.dragged()
        {
            let (kind, start, piv, axis) = (d.kind, d.start, d.pivot, d.axis);
            let moving: HashSet<FaceId> = d.groups.iter().flatten().copied().collect();
            let mut snapped = None;
            let x = match kind {
                DragKind::Pan => {
                    let delta = now - start;
                    self.uv.center = d.center - DVec2::new(delta.x as f64, -delta.y as f64) / zoom;
                    None
                }
                DragKind::Marquee => None,
                DragKind::Pivot => {
                    let q = to_uv(now);
                    let none = HashSet::new();
                    snapped = if snap_on && !mods.alt { snap(q, &none) } else { None };
                    self.uv.pivot = Some(snapped.unwrap_or(q));
                    None
                }
                DragKind::Move => {
                    let mut dv = to_uv(now) - to_uv(start);
                    match axis {
                        Some(0) => dv.y = 0.0,
                        Some(_) => dv.x = 0.0,
                        None => {}
                    }
                    if let Some(g) = d.grab.filter(|_| snap_on && !mods.alt)
                        && let Some(s) = snap(g + dv, &moving)
                    {
                        dv = s - g;
                        snapped = Some(s);
                    } else if inc.move_on != mods.shift {
                        // the move step is in texture millimetres of the first island's tile
                        let t = DVec2::from(d.groups.first().and_then(|g| g.first()).map_or([1000.0; 2], |f| self.uv_tile(mesh, *f, fallback)));
                        dv = DVec2::new(round_to(dv.x * t.x, inc.move_step), round_to(dv.y * t.y, inc.move_step)) / t;
                    }
                    Some(Xform::Move(dv))
                }
                DragKind::Rotate => {
                    let (a0, a1) = (start - to_screen(piv), now - to_screen(piv));
                    let mut deg = (-(a1.y.atan2(a1.x) - a0.y.atan2(a0.x)) as f64).to_degrees();
                    deg = (deg + 540.0).rem_euclid(360.0) - 180.0;
                    if inc.rotate_on != mods.shift {
                        deg = round_to(deg, inc.angle_step);
                    }
                    Some(Xform::Rotate(deg))
                }
                DragKind::Scale => {
                    let p = to_screen(piv);
                    let (a, b) = (start - p, now - p);
                    let ratio = |a: f32, b: f32| if a.abs() < 1.0 { 1.0 } else { (b / a).max(0.01) as f64 };
                    let mut k = match axis {
                        Some(0) => ratio(a.x, b.x),
                        Some(_) => ratio(a.y, b.y),
                        None => (b.length() / a.length().max(1.0)).max(0.01) as f64,
                    };
                    if inc.scale_on != mods.shift {
                        k = round_to(k, inc.scale_step).max(inc.scale_step.max(0.01));
                    }
                    Some(Xform::Scale(match axis {
                        Some(0) => DVec2::new(k, 1.0),
                        Some(_) => DVec2::new(1.0, k),
                        None => DVec2::splat(k),
                    }))
                }
            };
            if let Some(x) = x {
                let (orig, groups) = (d.orig.clone(), d.groups.clone());
                let t = DVec2::from(groups.first().and_then(|g| g.first()).map_or([1000.0; 2], |f| self.uv_tile(mesh, *f, fallback)));
                self.uv.vcb = match x {
                    Xform::Move(dv) => xform_text(Xform::MoveMm(dv * t), self.doc.model.unit),
                    x => xform_text(x, self.doc.model.unit),
                };
                self.uv_set_untracked(&orig);
                let list = self.uv_xform(&groups, x, piv, fallback);
                self.uv_set_untracked(&list);
            }
            if let Some(d) = self.uv.drag.as_mut() {
                d.now = x;
                d.snapped = snapped;
            }
        }
        // overlays: the box being dragged, the snap point, the pivot
        let painter = ui.painter_at(rect);
        if let (Some(d), Some(now)) = (self.uv.drag.as_ref(), pointer) {
            if d.kind == DragKind::Marquee {
                let r = Rect::from_two_pos(d.start, now);
                painter.rect(r, 0.0, Color32::from_rgba_unmultiplied(255, 170, 70, 30), Stroke::new(1.0, Color32::from_rgb(255, 170, 70)), egui::StrokeKind::Middle);
            }
            if let Some(s) = d.snapped {
                painter.rect_stroke(Rect::from_center_size(to_screen(s), vec2(9.0, 9.0)), 0.0, Stroke::new(2.0, Color32::from_rgb(80, 220, 120)), egui::StrokeKind::Middle);
            }
        }
        if tool != UvTool::Select {
            // the gizmo: at the pivot (following a move), its hovered / dragged part lit
            let drag = self.uv.drag.as_ref().filter(|d| d.kind != DragKind::Pan && d.kind != DragKind::Marquee);
            let gp = match drag {
                Some(d) if d.kind == DragKind::Pivot => to_screen(self.uv.pivot.unwrap_or(pivot)),
                Some(d) => to_screen(d.pivot + if let Some(Xform::Move(dv)) = d.now { dv } else { DVec2::ZERO }),
                None => g,
            };
            let lit = match drag {
                Some(d) => match (d.kind, d.axis) {
                    (DragKind::Pivot, _) => Some(Handle::Center),
                    (_, Some(a)) => Some(Handle::Axis(a)),
                    (DragKind::Scale, None) => Some(Handle::Uniform),
                    _ => None,
                },
                None => hover.and_then(|h| gizmo_hit(tool, gp, h)),
            };
            paint_gizmo(&painter, tool, gp, lit);
            if let Some(d) = self.uv.drag.as_ref().filter(|d| d.kind == DragKind::Rotate)
                && let Some(now) = pointer
            {
                painter.line_segment([gp, d.start], Stroke::new(1.0, Color32::from_white_alpha(120)));
                painter.line_segment([gp, now], Stroke::new(1.4, Color32::from_rgb(255, 140, 30)));
            }
        }
        if resp.drag_stopped()
            && let Some(d) = self.uv.drag.take()
        {
            match d.kind {
                DragKind::Marquee => {
                    if let Some(now) = pointer {
                        let r = Rect::from_two_pos(d.start, now);
                        let hit: HashSet<FaceId> = polys.iter().filter(|(_, poly)| r.contains(to_screen(centroid(poly)))).flat_map(|(f, _)| part_of(*f)).collect();
                        if mods.shift {
                            self.uv.selected.extend(hit);
                        } else {
                            self.uv.selected = hit;
                        }
                    }
                }
                DragKind::Move | DragKind::Rotate | DragKind::Scale => {
                    let mesh = self.doc.model.mesh_at(&self.doc.active_path);
                    let now: Placements = d.orig.iter().map(|(f, _)| (*f, mesh.and_then(|m| m.faces.get(*f)).and_then(|x| x.uv))).collect();
                    if now != d.orig {
                        self.uv_set_untracked(&d.orig);
                        self.uv_commit_last(now, d.pivot, d.groups);
                    }
                }
                DragKind::Pan | DragKind::Pivot => {}
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

    /// Script dump of the canvas tool: (tool, island / face mode, pivot or `-`, status value,
    /// picked faces as indices into [`Self::uv_faces`]).
    pub(crate) fn uv_tool_dump(&self) -> String {
        let pivot = self.uv.pivot.map_or("-".to_string(), |p| format!("{:.3},{:.3}", p.x, p.y));
        let faces = self.uv_faces();
        let picked: Vec<usize> = faces.iter().enumerate().filter(|(_, f)| self.uv.selected.contains(f)).map(|(i, _)| i).collect();
        format!("({} {} {} {:?} {:?})", self.uv.tool.key(), if self.uv.face_mode { "face" } else { "island" }, pivot, self.uv.vcb, picked)
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

/// The gizmo of `tool` at `g`: the axes (arrows for Move, square ends for Scale, plus the
/// uniform corner), the ring of Rotate and the centre; `lit` is drawn highlighted.
fn paint_gizmo(painter: &egui::Painter, tool: UvTool, g: Pos2, lit: Option<Handle>) {
    let hi = Color32::from_rgb(255, 225, 60);
    let pick = |h: Handle, c: Color32| if lit == Some(h) { hi } else { c };
    let (red, green, orange) = (Color32::from_rgb(235, 75, 60), Color32::from_rgb(80, 205, 95), Color32::from_rgb(255, 140, 30));
    for (axis, dir, base) in [(0, vec2(1.0, 0.0), red), (1, vec2(0.0, -1.0), green)] {
        if tool == UvTool::Rotate {
            break;
        }
        let c = pick(Handle::Axis(axis), base);
        let w = if lit == Some(Handle::Axis(axis)) { 3.0 } else { 2.0 };
        let end = g + dir * ARM;
        painter.line_segment([g + dir * CATCH, end], Stroke::new(w, c));
        if tool == UvTool::Move {
            let n = vec2(-dir.y, dir.x);
            painter.add(Shape::convex_polygon(vec![end + dir * 9.0, end + n * 5.0, end - n * 5.0], c, Stroke::NONE));
        } else {
            painter.rect_filled(Rect::from_center_size(end, vec2(9.0, 9.0)), 1.0, c);
        }
    }
    if tool == UvTool::Scale {
        let c = pick(Handle::Uniform, orange);
        painter.rect(Rect::from_center_size(g + vec2(ARM * 0.62, -ARM * 0.62), vec2(11.0, 11.0)), 1.0, c.gamma_multiply(0.5), Stroke::new(1.5, c), egui::StrokeKind::Middle);
    }
    if tool == UvTool::Rotate {
        painter.circle_stroke(g, ARM, Stroke::new(2.0, orange.gamma_multiply(0.8)));
    }
    let c = pick(Handle::Center, if tool == UvTool::Move { Color32::WHITE } else { orange });
    painter.circle(g, 6.0, c.gamma_multiply(0.3), Stroke::new(1.8, c));
    if tool != UvTool::Move {
        painter.line_segment([g - vec2(10.0, 0.0), g + vec2(10.0, 0.0)], Stroke::new(1.2, c));
        painter.line_segment([g - vec2(0.0, 10.0), g + vec2(0.0, 10.0)], Stroke::new(1.2, c));
    }
}

/// The picked faces of each island (islands with none left out).
fn picked_groups(islands: &[Vec<FaceId>], picked: &HashSet<FaceId>) -> Vec<Vec<FaceId>> {
    islands.iter().map(|i| i.iter().copied().filter(|f| picked.contains(f)).collect::<Vec<_>>()).filter(|g| !g.is_empty()).collect()
}

/// Mean of a polygon's corners.
fn centroid(poly: &[DVec2]) -> DVec2 {
    poly.iter().copied().sum::<DVec2>() / poly.len().max(1) as f64
}

/// A thin gap between toolbar groups.
fn sep(ui: &mut Ui) {
    ui.add_space(3.0);
    ui.separator();
    ui.add_space(3.0);
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
