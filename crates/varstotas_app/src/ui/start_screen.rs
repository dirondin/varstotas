//! Startup hub; it never replaces a document without the usual unsaved-work guard.
use crate::{
    app::{Action, VarstotasApp},
    templates::Template,
};
use egui::{Color32, Context, FontId, Id, Modal, Pos2, RichText, Shape, Stroke, Ui, pos2, vec2};
use varstotas_i18n::tr;

/// The banner's height, points.
const BANNER_H: f32 = 156.0;

impl VarstotasApp {
    pub(crate) fn start_screen(&mut self, ctx: &Context) {
        if !self.show_start || self.confirm.is_some() || !self.autosave.recoveries.is_empty() {
            return;
        }
        let mut action = None;
        let mut close = false;
        let mut guide = false;
        let mut template = self.prefs.template.unwrap_or(Template::ArchitecturalMm);
        let recent = self.prefs.recent.clone();
        if self.about_icon.is_none()
            && let Some(image) = crate::textures::thumbnail(include_bytes!("../../assets/icon.png"), 320)
        {
            self.about_icon = Some(ctx.load_texture("about_app_icon", image, egui::TextureOptions::LINEAR));
        }
        let icon = self.about_icon.clone();
        let response = Modal::new(Id::new("start_screen")).show(ctx, |ui| {
            ui.set_width(580.0);
            banner(ui, icon.as_ref());
            ui.add_space(14.0);
            ui.columns(2, |cols| {
                cols[0].heading(tr!("start-create"));
                egui::ComboBox::from_id_salt("start_template")
                    .selected_text(template.name())
                    .show_ui(&mut cols[0], |ui| {
                        for t in Template::ALL {
                            ui.selectable_value(&mut template, t, t.name());
                        }
                    });
                cols[0].add_space(6.0);
                let r = cols[0].button(tr!("start-new"));
                super::tray_lib::spot("start:new", r.rect.center());
                if r.clicked() {
                    action = Some(Action::NewFromTemplate(template));
                }
                let r = cols[0].button(tr!("start-open"));
                super::tray_lib::spot("start:open", r.rect.center());
                if r.clicked() {
                    action = Some(Action::Open);
                }
                cols[0].add_space(8.0);
                if cols[0].button(tr!("menu-examples")).clicked() {
                    super::online_mats::open_url(crate::EXAMPLES_URL);
                }
                let r = cols[0].button(tr!("tutorial-title"));
                super::tray_lib::spot("start:tutorial", r.rect.center());
                if r.clicked() { action = Some(Action::Tutorial); }
                let r = cols[0].button(tr!("start-guide"));
                super::tray_lib::spot("start:guide", r.rect.center());
                if r.clicked() {
                    guide = true;
                    close = true;
                }
                cols[1].heading(tr!("start-recent"));
                if recent.is_empty() {
                    cols[1].label(tr!("start-no-recent"));
                }
                for (i, path) in recent.iter().take(6).enumerate() {
                    let name = path
                        .file_name()
                        .unwrap_or(path.as_os_str())
                        .to_string_lossy();
                    let r = cols[1]
                        .add_enabled(path.is_file(), egui::Button::new(name).truncate())
                        .on_hover_text(path.display().to_string());
                    super::tray_lib::spot(format!("start:recent:{i}"), r.rect.center());
                    if r.clicked() {
                        action = Some(Action::OpenPath(path.clone()));
                    }
                }
            });
            ui.add_space(12.0);
            ui.separator();
            ui.label(RichText::new(tr!("start-basics-title")).strong());
            ui.label(tr!("start-basics"));
            ui.add_space(10.0);
            ui.horizontal(|ui| {
                let r = ui.checkbox(&mut self.prefs.show_start_screen, tr!("start-show"));
                super::tray_lib::spot("start:show", r.rect.center());
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let r = ui.button(tr!("start-continue"));
                    super::tray_lib::spot("start:continue", r.rect.center());
                    if r.clicked() {
                        close = true;
                    }
                });
            });
        });
        self.prefs.template = Some(template);
        if guide {
            self.open_app_help(super::app_help::Topic::Navigation);
        }
        if response.should_close() || close || action.is_some() {
            self.show_start = false;
        }
        if let Some(action) = action {
            self.actions.push(action);
        }
    }
}

/// The start screen's banner: a blueprint sky with a small scene drawn in isometric (a wooden
/// slab, a glass box and a cylinder, with a dimension), the app icon, name and subtitle.
fn banner(ui: &mut Ui, icon: Option<&egui::TextureHandle>) {
    let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), BANNER_H), egui::Sense::hover());
    let painter = ui.painter_at(rect);
    let dark = super::theme::dark();
    // the sky: top to bottom, as in the app icon
    let (top, bottom) = if dark {
        (Color32::from_rgb(0x2E, 0x38, 0x46), Color32::from_rgb(0x1B, 0x21, 0x2A))
    } else {
        (Color32::from_rgb(0xEC, 0xF0, 0xF5), Color32::from_rgb(0xB9, 0xC6, 0xD7))
    };
    let mut sky = egui::Mesh::default();
    for (p, c) in [(rect.left_top(), top), (rect.right_top(), top), (rect.right_bottom(), bottom), (rect.left_bottom(), bottom)] {
        sky.colored_vertex(p, c);
    }
    sky.add_triangle(0, 1, 2);
    sky.add_triangle(0, 2, 3);
    painter.add(Shape::mesh(sky));
    // isometric projection of the scene (world units) into the right part of the banner
    let s = 24.0;
    let origin = pos2(rect.right() - 128.0, rect.top() + 84.0);
    let iso = |x: f32, y: f32, z: f32| pos2(origin.x + (x - y) * 0.866 * s, origin.y + (x + y) * 0.5 * s - z * s);
    // a faint isometric grid on the ground
    let grid = if dark { Color32::from_white_alpha(14) } else { Color32::from_black_alpha(16) };
    for i in -6..=9 {
        let k = i as f32;
        painter.line_segment([iso(k, -6.0, 0.0), iso(k, 9.0, 0.0)], Stroke::new(1.0, grid));
        painter.line_segment([iso(-6.0, k, 0.0), iso(9.0, k, 0.0)], Stroke::new(1.0, grid));
    }
    let ink = if dark { Color32::from_rgb(0xD8, 0xDE, 0xE8) } else { Color32::from_rgb(0x1E, 0x24, 0x2C) };
    let edge = Stroke::new(1.3, ink);
    let glass = |a: u8| if dark { Color32::from_rgba_unmultiplied(0x9C, 0xB8, 0xDA, a) } else { Color32::from_rgba_unmultiplied(0x9A, 0xB4, 0xD4, a) };
    let quad = |pts: [Pos2; 4], fill: Color32| painter.add(Shape::convex_polygon(pts.to_vec(), fill, edge));
    // a box from (x, y, 0) of size (w, d, h): its three faces seen from the viewer
    let boxed = |x: f32, y: f32, w: f32, d: f32, h: f32, faces: [Color32; 3]| {
        quad([iso(x, y, h), iso(x + w, y, h), iso(x + w, y + d, h), iso(x, y + d, h)], faces[0]);
        quad([iso(x + w, y, 0.0), iso(x + w, y + d, 0.0), iso(x + w, y + d, h), iso(x + w, y, h)], faces[1]);
        quad([iso(x, y + d, 0.0), iso(x + w, y + d, 0.0), iso(x + w, y + d, h), iso(x, y + d, h)], faces[2]);
    };
    // far to near: the cylinder, the glass box, the wooden slab
    cylinder(&painter, &iso, (-0.6, -1.6), 0.8, 1.2, [glass(120), glass(70)], edge);
    boxed(2.0, -2.4, 1.4, 1.4, 2.2, [glass(110), glass(80), glass(60)]);
    let wood = [Color32::from_rgb(0xC0, 0x8A, 0x55), Color32::from_rgb(0x8E, 0x5E, 0x33), Color32::from_rgb(0xA4, 0x6E, 0x40)];
    let (w, d, h) = (3.0, 1.8, 0.5);
    boxed(0.0, 0.0, w, d, h, wood);
    // grain on the slab's top
    for k in 1..5 {
        let t = k as f32 * d / 5.0;
        painter.line_segment([iso(0.1, t, h), iso(w - 0.1, t + 0.05, h)], Stroke::new(0.8, Color32::from_rgba_unmultiplied(0x6B, 0x42, 0x20, 70)));
    }
    // a dimension along the slab's left edge: extension lines, the line with ticks, its label
    let dim = Stroke::new(1.0, ink);
    let off = -0.55;
    painter.line_segment([iso(-0.1, 0.0, h), iso(off - 0.15, 0.0, h)], dim);
    painter.line_segment([iso(-0.1, d, h), iso(off - 0.15, d, h)], dim);
    let (a, b) = (iso(off, 0.0, h), iso(off, d, h));
    painter.line_segment([a, b], dim);
    for p in [a, b] {
        painter.line_segment([p + vec2(-3.0, -3.0), p + vec2(3.0, 3.0)], Stroke::new(1.6, ink));
    }
    let mid = pos2((a.x + b.x) / 2.0, (a.y + b.y) / 2.0);
    painter.text(mid + vec2(-12.0, -9.0), egui::Align2::CENTER_CENTER, "600", FontId::proportional(11.0), ink);
    // the icon, the name and the subtitle on the left
    let mut x = rect.left() + 18.0;
    if let Some(icon) = icon {
        let r = egui::Rect::from_min_size(pos2(x, rect.center().y - 52.0), vec2(104.0, 104.0));
        painter.image(icon.id(), r, egui::Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)), Color32::WHITE);
        x = r.right() + 14.0;
    }
    let text = if dark { Color32::from_rgb(0xF2, 0xF4, 0xF7) } else { Color32::from_rgb(0x16, 0x1B, 0x22) };
    painter.text(pos2(x, rect.center().y - 12.0), egui::Align2::LEFT_BOTTOM, crate::APP_NAME, FontId::proportional(30.0), text);
    let sub = painter.layout(tr!("start-subtitle"), FontId::proportional(13.0), super::theme::text_dim(), 190.0);
    painter.galley(pos2(x, rect.center().y - 4.0), sub, super::theme::text_dim());
}

/// An upright cylinder at `c` (radius `r`, height `h`): its body (the outline of both rims) and
/// its top.
fn cylinder(painter: &egui::Painter, iso: &dyn Fn(f32, f32, f32) -> Pos2, c: (f32, f32), r: f32, h: f32, fill: [Color32; 2], edge: Stroke) {
    let rim = |z: f32| (0..48).map(|i| {
        let a = i as f32 / 48.0 * std::f32::consts::TAU;
        iso(c.0 + r * a.cos(), c.1 + r * a.sin(), z)
    }).collect::<Vec<_>>();
    let mut all = rim(0.0);
    all.extend(rim(h));
    painter.add(Shape::convex_polygon(hull(all), fill[1], edge));
    painter.add(Shape::convex_polygon(rim(h), fill[0], edge));
}

/// The convex hull of points, clockwise on screen (monotone chain).
fn hull(mut p: Vec<Pos2>) -> Vec<Pos2> {
    p.sort_by(|a, b| a.x.total_cmp(&b.x).then(a.y.total_cmp(&b.y)));
    let cross = |o: Pos2, a: Pos2, b: Pos2| (a.x - o.x) * (b.y - o.y) - (a.y - o.y) * (b.x - o.x);
    let mut out: Vec<Pos2> = Vec::new();
    for pass in 0..2 {
        let start = out.len();
        let pts: Box<dyn Iterator<Item = &Pos2>> = if pass == 0 { Box::new(p.iter()) } else { Box::new(p.iter().rev()) };
        for q in pts {
            while out.len() >= start + 2 && cross(out[out.len() - 2], out[out.len() - 1], *q) <= 0.0 {
                out.pop();
            }
            out.push(*q);
        }
        out.pop();
    }
    out
}
