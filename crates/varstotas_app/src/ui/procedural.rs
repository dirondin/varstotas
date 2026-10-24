//! Procedural materials: bricks, tiles, planks, noise and checker drawn into a seamless PNG
//! tile, which becomes an ordinary textured material.

use super::theme;
use crate::app::VarstotasApp;
use crate::document::PaletteMaterial;
use egui::{Color32, RichText};
use varstotas_i18n::tr;
use varstotas_kernel::TextureImage;

/// Side of the generated image in pixels.
const SIDE: u32 = 512;
/// Side of the preview in pixels.
const PREVIEW: u32 = 128;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Bricks,
    Tiles,
    Planks,
    Noise,
    Checker,
}

impl Kind {
    pub const ALL: [Kind; 5] = [Kind::Bricks, Kind::Tiles, Kind::Planks, Kind::Noise, Kind::Checker];

    pub fn key(self) -> &'static str {
        match self {
            Kind::Bricks => "bricks",
            Kind::Tiles => "tiles",
            Kind::Planks => "planks",
            Kind::Noise => "noise",
            Kind::Checker => "checker",
        }
    }

    fn label(self) -> String {
        match self {
            Kind::Bricks => tr!("proc-bricks"),
            Kind::Tiles => tr!("proc-tiles"),
            Kind::Planks => tr!("proc-planks"),
            Kind::Noise => tr!("proc-noise"),
            Kind::Checker => tr!("proc-checker"),
        }
    }

    /// Default colours (main, joints / second) and pieces across × up one tile.
    fn defaults(self) -> ([u8; 3], [u8; 3], [u32; 2]) {
        match self {
            Kind::Bricks => ([168, 74, 52], [205, 198, 186], [4, 8]),
            Kind::Tiles => ([226, 228, 230], [150, 150, 148], [4, 4]),
            Kind::Planks => ([176, 128, 82], [96, 64, 40], [2, 6]),
            Kind::Noise => ([120, 118, 112], [170, 166, 158], [6, 6]),
            Kind::Checker => ([30, 30, 30], [235, 235, 235], [2, 2]),
        }
    }
}

/// Parameters of a procedural tile.
#[derive(Clone, Debug, PartialEq)]
pub struct Params {
    pub kind: Kind,
    pub main: [u8; 3],
    pub joint: [u8; 3],
    /// Pieces across and up one tile (noise: feature scale).
    pub count: [u32; 2],
    /// Joint width as a part of a piece's smaller side (0..0.4).
    pub joint_width: f32,
    /// Colour variation between pieces (0..1).
    pub variation: f32,
    pub seed: u32,
    /// Tile width in mm.
    pub width_mm: f64,
}

impl Params {
    pub fn new(kind: Kind) -> Self {
        let (main, joint, count) = kind.defaults();
        Self { kind, main, joint, count, joint_width: 0.08, variation: 0.25, seed: 1, width_mm: 1000.0 }
    }
}

/// Window state.
pub struct ProcState {
    pub open: bool,
    pub params: Params,
    name: String,
    preview: Option<(Params, egui::TextureHandle)>,
}

impl Default for ProcState {
    fn default() -> Self {
        Self { open: false, params: Params::new(Kind::Bricks), name: String::new(), preview: None }
    }
}

fn hash(a: u32, b: u32, seed: u32) -> u32 {
    let mut h = a.wrapping_mul(0x8da6_b343) ^ b.wrapping_mul(0xd816_3841) ^ seed.wrapping_mul(0xcb1a_b31f);
    h ^= h >> 13;
    h = h.wrapping_mul(0x5bd1_e995);
    h ^ (h >> 15)
}

/// 0..1 from a lattice point.
fn rnd(a: u32, b: u32, seed: u32) -> f32 {
    (hash(a, b, seed) & 0xffff) as f32 / 65535.0
}

/// Value noise with period `p` cells (seamless), at `x, y` in cells.
fn value_noise(x: f32, y: f32, p: u32, seed: u32) -> f32 {
    let (xi, yi) = (x.floor(), y.floor());
    let (fx, fy) = (x - xi, y - yi);
    let s = |t: f32| t * t * (3.0 - 2.0 * t);
    let at = |i: f32, j: f32| rnd((i as i64).rem_euclid(p as i64) as u32, (j as i64).rem_euclid(p as i64) as u32, seed);
    let a = at(xi, yi) + (at(xi + 1.0, yi) - at(xi, yi)) * s(fx);
    let b = at(xi, yi + 1.0) + (at(xi + 1.0, yi + 1.0) - at(xi, yi + 1.0)) * s(fx);
    a + (b - a) * s(fy)
}

/// Fractal noise (4 octaves) with base period `p`, at `u, v` in 0..1 of the tile.
fn fbm(u: f32, v: f32, p: u32, seed: u32) -> f32 {
    let (mut sum, mut amp, mut norm, mut period) = (0.0, 0.5, 0.0, p.max(1));
    for o in 0..4 {
        sum += value_noise(u * period as f32, v * period as f32, period, seed + o) * amp;
        norm += amp;
        amp *= 0.5;
        period *= 2;
    }
    sum / norm
}

fn mix(a: [u8; 3], b: [u8; 3], t: f32) -> [f32; 3] {
    [0, 1, 2].map(|i| a[i] as f32 + (b[i] as f32 - a[i] as f32) * t.clamp(0.0, 1.0))
}

/// Joint width `jw` (a part of a piece's smaller side) as parts of a piece's width and height,
/// so joints are equally wide both ways.
fn joint_parts(jw: f32, cx: u32, cy: u32) -> (f32, f32) {
    let (pw, ph) = (1.0 / cx as f32, 1.0 / cy as f32);
    let j = jw * pw.min(ph);
    (j / pw, j / ph)
}

/// A piece's colour: the main colour varied in lightness and a little in hue.
fn piece(p: &Params, id: (u32, u32)) -> [f32; 3] {
    let r = rnd(id.0, id.1, p.seed) - 0.5;
    let w = rnd(id.1, id.0, p.seed + 7) - 0.5;
    let k = 1.0 + r * p.variation * 0.8;
    [p.main[0] as f32 * (k + w * p.variation * 0.15), p.main[1] as f32 * k, p.main[2] as f32 * (k - w * p.variation * 0.15)]
}

/// Draw the tile, `side` × `side` RGBA pixels (seamless when repeated).
pub fn generate(p: &Params, side: u32) -> Vec<u8> {
    let [cx, cy] = [p.count[0].max(1), p.count[1].max(1)];
    let mut out = Vec::with_capacity((side * side * 4) as usize);
    let jw = p.joint_width.clamp(0.0, 0.4);
    for y in 0..side {
        for x in 0..side {
            let (u, v) = ((x as f32 + 0.5) / side as f32, (y as f32 + 0.5) / side as f32);
            let grain = fbm(u, v, 8, p.seed + 100) - 0.5;
            let c = match p.kind {
                Kind::Bricks | Kind::Tiles => {
                    let row = (v * cy as f32).floor();
                    let shift = if p.kind == Kind::Bricks && row as u32 % 2 == 1 { 0.5 } else { 0.0 };
                    let col_f = u * cx as f32 + shift;
                    let col = col_f.floor();
                    let (fu, fv) = (col_f - col, v * cy as f32 - row);
                    let (ju, jv) = joint_parts(jw, cx, cy);
                    let joint = fu < ju / 2.0 || fu > 1.0 - ju / 2.0 || fv < jv / 2.0 || fv > 1.0 - jv / 2.0;
                    if joint {
                        p.joint.map(|c| c as f32 * (1.0 + grain * 0.3))
                    } else {
                        let id = ((col as i64).rem_euclid(cx as i64) as u32, row as u32);
                        piece(p, id).map(|c| c * (1.0 + grain * 0.25))
                    }
                }
                Kind::Planks => {
                    let row = (v * cy as f32).floor();
                    // each row's planks end at a random offset
                    let off = rnd(row as u32, 3, p.seed);
                    let col_f = u * cx as f32 + off * cx as f32;
                    let col = col_f.floor();
                    let (fu, fv) = (col_f - col, v * cy as f32 - row);
                    let (ju, jv) = joint_parts(jw * 0.5, cx, cy);
                    if fv < jv / 2.0 || fv > 1.0 - jv / 2.0 || fu < ju / 2.0 || fu > 1.0 - ju / 2.0 {
                        p.joint.map(|c| c as f32)
                    } else {
                        let id = ((col as i64).rem_euclid(cx as i64) as u32, row as u32);
                        // grain: noise stretched along the plank, with darker rings
                        let g = fbm(u * 0.25 + rnd(id.0, id.1, p.seed + 5), v * 4.0, 4, p.seed + id.1);
                        let rings = ((g * 18.0).sin() * 0.5 + 0.5).powf(3.0);
                        let base = piece(p, id);
                        let dark = p.joint.map(|c| c as f32);
                        [0, 1, 2].map(|i| base[i] + (dark[i] - base[i]) * rings * 0.35)
                    }
                }
                Kind::Noise => {
                    let n = fbm(u, v, cx, p.seed);
                    mix(p.main, p.joint, (n - 0.5) * (1.0 + p.variation * 3.0) + 0.5)
                }
                Kind::Checker => {
                    let odd = ((u * cx as f32).floor() as u32 + (v * cy as f32).floor() as u32) % 2 == 1;
                    if odd { p.joint } else { p.main }.map(|c| c as f32)
                }
            };
            out.extend(c.map(|c| c.round().clamp(0.0, 255.0) as u8));
            out.push(255);
        }
    }
    out
}

/// The material for `p`: a PNG tile `width_mm` wide.
pub fn material(p: &Params, name: &str) -> Result<PaletteMaterial, String> {
    let rgba = generate(p, SIDE);
    let img = image::RgbaImage::from_raw(SIDE, SIDE, rgba).ok_or("image")?;
    let mut out = std::io::Cursor::new(Vec::new());
    img.write_to(&mut out, image::ImageFormat::Png).map_err(|e| e.to_string())?;
    let tex = TextureImage::new(out.into_inner(), format!("{name}.png"), p.width_mm);
    let avg = {
        let px = img.as_raw();
        let n = (px.len() / 4) as f32;
        let mut s = [0.0f32; 3];
        for c in px.as_chunks::<4>().0 {
            for i in 0..3 {
                s[i] += c[i] as f32;
            }
        }
        [s[0] / n / 255.0, s[1] / n / 255.0, s[2] / n / 255.0, 1.0]
    };
    Ok(PaletteMaterial { name: name.to_string(), color: avg, image: Some(tex), pbr: None })
}

fn color_row(ui: &mut egui::Ui, label: String, c: &mut [u8; 3], id: &str) {
    ui.label(label);
    let mut c32 = Color32::from_rgb(c[0], c[1], c[2]);
    let r = egui::color_picker::color_edit_button_srgba(ui, &mut c32, egui::color_picker::Alpha::Opaque);
    super::tray_lib::spot(format!("proc:{id}"), r.rect.center());
    if r.changed() {
        *c = [c32.r(), c32.g(), c32.b()];
    }
    ui.end_row();
}

impl VarstotasApp {
    /// Add the procedural material to the model and make it the paint material (one undo step).
    pub(crate) fn create_procedural(&mut self) {
        let p = self.procedural.params.clone();
        let stem = if self.procedural.name.trim().is_empty() { p.kind.label() } else { self.procedural.name.trim().to_string() };
        let mut name = stem.clone();
        let mut k = 2;
        while self.doc.model.materials.values().any(|m| m.name == name) {
            name = format!("{stem} {k}");
            k += 1;
        }
        match material(&p, &name) {
            Ok(pm) => {
                let p2 = pm.clone();
                self.doc.commit(&tr!("app-undo-add-material"), |m| {
                    p2.ensure_in(m);
                });
                self.doc.current_material = Some(pm);
                self.set_tool(crate::tools::ToolId::Paint);
                self.flash(tr!("proc-created", name = name.as_str()));
            }
            Err(e) => self.flash_error(e),
        }
    }

    /// Materials → + → New Procedural…: kind, colours, pieces, joints, variation, preview.
    pub(crate) fn procedural_window(&mut self, ctx: &egui::Context) {
        if !self.procedural.open {
            return;
        }
        let mut open = true;
        let mut create = false;
        egui::Window::new(tr!("proc-title")).id(egui::Id::new("procedural_win")).open(&mut open).resizable(false).default_pos([320.0, 160.0]).show(ctx, |ui| {
            let st = &mut self.procedural;
            ui.horizontal(|ui| {
                for k in Kind::ALL {
                    let r = ui.selectable_label(st.params.kind == k, k.label());
                    super::tray_lib::spot(format!("proc:{}", k.key()), r.rect.center());
                    if r.clicked() && st.params.kind != k {
                        let w = st.params.width_mm;
                        st.params = Params { width_mm: w, ..Params::new(k) };
                    }
                }
            });
            ui.separator();
            ui.horizontal_top(|ui| {
                egui::Grid::new("proc_grid").num_columns(2).spacing([10.0, 6.0]).show(ui, |ui| {
                    let p = &mut st.params;
                    color_row(ui, tr!("proc-main"), &mut p.main, "main");
                    color_row(ui, if p.kind == Kind::Noise || p.kind == Kind::Checker { tr!("proc-second") } else { tr!("proc-joint") }, &mut p.joint, "joint");
                    ui.label(if p.kind == Kind::Noise { tr!("proc-scale") } else { tr!("proc-count") });
                    ui.horizontal(|ui| {
                        ui.add(egui::DragValue::new(&mut p.count[0]).range(1..=32));
                        if p.kind != Kind::Noise {
                            ui.label("×"); // i18n-ignore
                            ui.add(egui::DragValue::new(&mut p.count[1]).range(1..=32));
                        }
                    });
                    ui.end_row();
                    if matches!(p.kind, Kind::Bricks | Kind::Tiles | Kind::Planks) {
                        ui.label(tr!("proc-joint-width"));
                        ui.add(egui::Slider::new(&mut p.joint_width, 0.0..=0.4).fixed_decimals(2));
                        ui.end_row();
                    }
                    if p.kind != Kind::Checker {
                        ui.label(tr!("proc-variation"));
                        ui.add(egui::Slider::new(&mut p.variation, 0.0..=1.0).fixed_decimals(2));
                        ui.end_row();
                        ui.label(tr!("proc-seed"));
                        ui.horizontal(|ui| {
                            ui.add(egui::DragValue::new(&mut p.seed).range(1..=9999));
                            if ui.small_button(tr!("proc-another")).clicked() {
                                p.seed = p.seed % 9999 + 1;
                            }
                        });
                        ui.end_row();
                    }
                    ui.label(tr!("proc-width"));
                    ui.add(egui::DragValue::new(&mut p.width_mm).range(10.0..=100_000.0).suffix(" mm")); // i18n-ignore
                    ui.end_row();
                    ui.label(tr!("proc-name"));
                    ui.add(egui::TextEdit::singleline(&mut st.name).hint_text(st.params.kind.label()).desired_width(140.0));
                    ui.end_row();
                });
                // preview: 2 × 2 tiles, so the seams show
                let stale = st.preview.as_ref().is_none_or(|(p, _)| *p != st.params);
                if stale {
                    let rgba = generate(&st.params, PREVIEW);
                    let img = egui::ColorImage::from_rgba_unmultiplied([PREVIEW as usize; 2], &rgba);
                    let opts = egui::TextureOptions { wrap_mode: egui::TextureWrapMode::Repeat, ..egui::TextureOptions::LINEAR };
                    st.preview = Some((st.params.clone(), ui.ctx().load_texture("proc_preview", img, opts)));
                }
                if let Some((_, t)) = &st.preview {
                    let (rect, _) = ui.allocate_exact_size(egui::vec2(160.0, 160.0), egui::Sense::hover());
                    ui.painter().image(t.id(), rect, egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(2.0, 2.0)), Color32::WHITE);
                    ui.painter().rect_stroke(rect, 2.0, egui::Stroke::new(1.0, theme::border()), egui::StrokeKind::Inside);
                }
            });
            ui.label(RichText::new(tr!("proc-hint")).small().color(theme::text_dim()));
            ui.horizontal(|ui| {
                let r = ui.button(tr!("proc-create"));
                super::tray_lib::spot("proc:create", r.rect.center());
                if r.clicked() {
                    create = true;
                }
            });
        });
        if create {
            self.create_procedural();
        }
        self.procedural.open &= open;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tiles_are_seamless_and_kinds_differ() {
        for k in Kind::ALL {
            let p = Params::new(k);
            let px = generate(&p, 64);
            assert_eq!(px.len(), 64 * 64 * 4);
            // opposite edges continue each other (no hard seam): compare the mean step across the
            // wrap with the mean step inside the image
            let at = |x: usize, y: usize| px[(y * 64 + x) * 4] as f32;
            let wrap: f32 = (0..64).map(|y| (at(63, y) - at(0, y)).abs()).sum::<f32>() / 64.0;
            let inner: f32 = (0..64).map(|y| (at(31, y) - at(32, y)).abs()).sum::<f32>() / 64.0;
            assert!(wrap <= inner + 40.0, "{k:?}: wrap {wrap} inner {inner}");
        }
        assert_ne!(generate(&Params::new(Kind::Bricks), 32), generate(&Params::new(Kind::Tiles), 32));
        let m = material(&Params::new(Kind::Checker), "c").unwrap();
        assert!(m.image.as_ref().is_some_and(|i| i.pixels == [SIDE, SIDE] && i.size == [1000.0, 1000.0]));
    }
}
