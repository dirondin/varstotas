//! Vector toolbar icons drawn with egui's painter on a 24×24 design grid.
//! Consistent style: ~2px dark-grey strokes, soft fills, one accent colour per icon.

use egui::{Color32, Painter, Pos2, Rect, Shape, Stroke, Vec2, vec2};
use std::f32::consts::PI;

#[allow(dead_code)] // full icon set; some are only used by later tools
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Icon {
    Select,
    Eraser,
    Paint,
    Line,
    Arc,
    Rectangle,
    Circle,
    Polygon,
    PushPull,
    Move,
    Rotate,
    Scale,
    Offset,
    FollowMe,
    Tape,
    Protractor,
    Orbit,
    Pan,
    Zoom,
    ZoomExtents,
    ViewIso,
    ViewTop,
    ViewFront,
    ViewRight,
    ViewBack,
    ViewLeft,
    StyleXRay,
    StyleWireframe,
    StyleHiddenLine,
    StyleShaded,
    StyleMonochrome,
    Perspective,
    Undo,
    Redo,
    Eye,
    EyeOff,
    Plus,
    Trash,
    Refresh,
    Info,
    Group,
}

/// Palette used by icons.
pub mod pal {
    use egui::Color32;
    pub const INK: Color32 = Color32::from_rgb(0x3A, 0x3F, 0x48);
    pub const INK_ACTIVE: Color32 = Color32::from_rgb(0x12, 0x4A, 0x9C);
    pub const FILL: Color32 = Color32::from_rgb(0xF4, 0xF6, 0xF9);
    pub const SHADE: Color32 = Color32::from_rgb(0xD5, 0xDB, 0xE4);
    pub const SHADE_DARK: Color32 = Color32::from_rgb(0xAE, 0xB8, 0xC6);
    pub const BLUE: Color32 = Color32::from_rgb(0x2F, 0x7D, 0xE1);
    pub const BLUE_SOFT: Color32 = Color32::from_rgb(0xB8, 0xD4, 0xF5);
    pub const RED: Color32 = Color32::from_rgb(0xE0, 0x53, 0x3D);
    pub const GREEN: Color32 = Color32::from_rgb(0x3B, 0xA5, 0x5C);
    pub const YELLOW: Color32 = Color32::from_rgb(0xF2, 0xC1, 0x4E);
    pub const PINK: Color32 = Color32::from_rgb(0xF0, 0x8F, 0xA8);
    pub const SKIN: Color32 = Color32::from_rgb(0xF6, 0xD9, 0xB8);
}

struct Pen<'a> {
    painter: &'a Painter,
    origin: Pos2,
    s: f32,
    ink: Color32,
}

impl Pen<'_> {
    fn p(&self, x: f32, y: f32) -> Pos2 {
        self.origin + vec2(x * self.s, y * self.s)
    }
    fn pts(&self, pts: &[(f32, f32)]) -> Vec<Pos2> {
        pts.iter().map(|&(x, y)| self.p(x, y)).collect()
    }
    fn stroke(&self) -> Stroke {
        Stroke::new(1.7 * self.s, self.ink)
    }
    fn thin(&self) -> Stroke {
        Stroke::new(1.1 * self.s, self.ink)
    }
    fn line(&self, pts: &[(f32, f32)]) {
        self.painter.add(Shape::line(self.pts(pts), self.stroke()));
    }
    fn line_c(&self, pts: &[(f32, f32)], stroke: Stroke) {
        self.painter.add(Shape::line(self.pts(pts), stroke));
    }
    fn poly(&self, pts: &[(f32, f32)], fill: Color32) {
        self.painter.add(Shape::convex_polygon(self.pts(pts), fill, self.stroke()));
    }
    fn poly_c(&self, pts: &[(f32, f32)], fill: Color32, stroke: Stroke) {
        self.painter.add(Shape::convex_polygon(self.pts(pts), fill, stroke));
    }
    /// Closed outline for non-convex shapes (no fill).
    fn closed(&self, pts: &[(f32, f32)]) {
        self.painter.add(Shape::closed_line(self.pts(pts), self.stroke()));
    }
    fn circle(&self, c: (f32, f32), r: f32, fill: Color32) {
        self.painter.circle(self.p(c.0, c.1), r * self.s, fill, self.stroke());
    }
    fn dot(&self, c: (f32, f32), r: f32, color: Color32) {
        self.painter.circle_filled(self.p(c.0, c.1), r * self.s, color);
    }
    fn arc_pts(&self, c: (f32, f32), r: f32, a0: f32, a1: f32) -> Vec<Pos2> {
        let n = ((a1 - a0).abs() / 10.0).ceil().max(2.0) as usize;
        (0..=n)
            .map(|i| {
                let a = (a0 + (a1 - a0) * i as f32 / n as f32) * PI / 180.0;
                self.p(c.0 + r * a.cos(), c.1 + r * a.sin())
            })
            .collect()
    }
    fn arc(&self, c: (f32, f32), r: f32, a0: f32, a1: f32, stroke: Stroke) {
        self.painter.add(Shape::line(self.arc_pts(c, r, a0, a1), stroke));
    }
    /// Filled arrow head with tip at `tip`, pointing along `dir`.
    fn head(&self, tip: (f32, f32), dir: (f32, f32), size: f32, color: Color32) {
        let d = Vec2::new(dir.0, dir.1).normalized();
        let n = vec2(-d.y, d.x);
        let t = self.p(tip.0, tip.1);
        let back = t - d * size * self.s;
        self.painter.add(Shape::convex_polygon(
            vec![t, back + n * size * 0.6 * self.s, back - n * size * 0.6 * self.s],
            color,
            Stroke::NONE,
        ));
    }
    fn dashed(&self, a: (f32, f32), b: (f32, f32), stroke: Stroke) {
        self.painter.extend(Shape::dashed_line(
            &[self.p(a.0, a.1), self.p(b.0, b.1)],
            stroke,
            2.0 * self.s,
            1.6 * self.s,
        ));
    }
}

/// Isometric cube helper: (top, left, right) faces.
const CUBE_TOP: [(f32, f32); 4] = [(12.0, 3.0), (20.0, 7.5), (12.0, 12.0), (4.0, 7.5)];
const CUBE_LEFT: [(f32, f32); 4] = [(4.0, 7.5), (12.0, 12.0), (12.0, 21.0), (4.0, 16.5)];
const CUBE_RIGHT: [(f32, f32); 4] = [(12.0, 12.0), (20.0, 7.5), (20.0, 16.5), (12.0, 21.0)];

fn cube(pen: &Pen, top: Color32, left: Color32, right: Color32) {
    pen.poly(&CUBE_TOP, top);
    pen.poly(&CUBE_LEFT, left);
    pen.poly(&CUBE_RIGHT, right);
}

/// Paint `icon` into `rect` (square recommended). `active` uses the accent ink.
pub fn paint_icon(painter: &Painter, rect: Rect, icon: Icon, active: bool) {
    let side = rect.width().min(rect.height());
    let origin = rect.center() - vec2(side, side) * 0.5;
    let pen = Pen { painter, origin, s: side / 24.0, ink: if active { pal::INK_ACTIVE } else { pal::INK } };
    use pal::*;
    match icon {
        Icon::Select => {
            let arrow = [
                (6.0, 3.0),
                (6.0, 19.0),
                (10.0, 15.2),
                (13.0, 21.0),
                (15.6, 19.8),
                (12.7, 14.2),
                (18.0, 14.0),
            ];
            // Fill as two convex pieces, then outline.
            pen.poly_c(&[(6.0, 3.0), (6.0, 19.0), (10.0, 15.2), (18.0, 14.0)], FILL, Stroke::NONE);
            pen.poly_c(&[(10.0, 15.2), (13.0, 21.0), (15.6, 19.8), (12.7, 14.2)], FILL, Stroke::NONE);
            pen.closed(&arrow);
        }
        Icon::Eraser => {
            pen.poly(&[(4.0, 14.0), (11.5, 6.5), (19.0, 14.0), (11.5, 21.5)], FILL);
            pen.poly(&[(11.5, 6.5), (19.0, 14.0), (15.25, 17.75), (7.75, 10.25)], PINK);
            pen.line(&[(13.0, 21.5), (21.0, 21.5)]);
        }
        Icon::Paint => {
            pen.poly(&[(3.5, 11.0), (10.5, 4.0), (18.5, 12.0), (11.5, 19.0)], FILL);
            pen.poly_c(
                &[(7.0, 14.5), (15.0, 15.5), (11.5, 19.0)],
                BLUE,
                Stroke::NONE,
            );
            pen.poly(&[(3.5, 11.0), (10.5, 4.0), (18.5, 12.0), (11.5, 19.0)], Color32::TRANSPARENT);
            pen.arc((10.5, 7.0), 4.5, 200.0, 340.0, pen.thin());
            pen.poly_c(&[(19.5, 13.0), (21.5, 18.0), (17.5, 18.0)], BLUE, Stroke::NONE);
            pen.dot((19.5, 18.2), 2.0, BLUE);
        }
        Icon::Line => {
            pen.poly(&[(6.5, 14.5), (15.5, 5.5), (18.5, 8.5), (9.5, 17.5)], YELLOW);
            pen.poly(&[(15.5, 5.5), (17.0, 4.0), (20.0, 7.0), (18.5, 8.5)], PINK);
            pen.poly(&[(6.5, 14.5), (9.5, 17.5), (4.0, 20.0)], SKIN);
            pen.dot((4.6, 19.4), 1.0, pen.ink);
        }
        Icon::Rectangle => {
            pen.poly(&[(4.0, 6.0), (20.0, 6.0), (20.0, 18.0), (4.0, 18.0)], BLUE_SOFT);
            for c in [(4.0, 6.0), (20.0, 18.0)] {
                pen.dot(c, 2.0, RED);
            }
        }
        Icon::Circle => {
            pen.circle((12.0, 12.0), 8.5, BLUE_SOFT);
            pen.line_c(&[(12.0, 12.0), (20.5, 12.0)], pen.thin());
            pen.dot((12.0, 12.0), 1.7, RED);
        }
        Icon::Arc => {
            pen.arc((12.0, 17.0), 9.0, 180.0, 360.0, pen.stroke());
            pen.line_c(&[(3.0, 17.0), (21.0, 17.0)], Stroke::new(1.0 * pen.s, SHADE_DARK));
            pen.dot((3.0, 17.0), 2.0, RED);
            pen.dot((21.0, 17.0), 2.0, RED);
            pen.dot((12.0, 8.0), 1.6, BLUE);
        }
        Icon::Polygon => {
            let pts: Vec<(f32, f32)> = (0..6)
                .map(|i| {
                    let a = (i as f32 * 60.0 - 90.0) * PI / 180.0;
                    (12.0 + 8.5 * a.cos(), 12.0 + 8.5 * a.sin())
                })
                .collect();
            pen.poly(&pts, BLUE_SOFT);
            pen.dot((12.0, 12.0), 1.5, RED);
        }
        Icon::PushPull => {
            pen.poly(&[(4.0, 15.0), (14.0, 15.0), (14.0, 21.0), (4.0, 21.0)], SHADE);
            pen.poly(&[(14.0, 15.0), (20.0, 11.0), (20.0, 17.0), (14.0, 21.0)], SHADE_DARK);
            pen.poly(&[(4.0, 15.0), (10.0, 11.0), (20.0, 11.0), (14.0, 15.0)], BLUE_SOFT);
            pen.line_c(&[(12.0, 13.0), (12.0, 5.5)], Stroke::new(2.2 * pen.s, RED));
            pen.head((12.0, 1.8), (0.0, -1.0), 4.5, RED);
        }
        Icon::Move => {
            let st = Stroke::new(1.8 * pen.s, pen.ink);
            pen.line_c(&[(12.0, 5.0), (12.0, 19.0)], st);
            pen.line_c(&[(5.0, 12.0), (19.0, 12.0)], st);
            pen.head((12.0, 2.0), (0.0, -1.0), 4.0, RED);
            pen.head((12.0, 22.0), (0.0, 1.0), 4.0, RED);
            pen.head((2.0, 12.0), (-1.0, 0.0), 4.0, RED);
            pen.head((22.0, 12.0), (1.0, 0.0), 4.0, RED);
        }
        Icon::Rotate => {
            pen.arc((12.0, 12.0), 8.0, 40.0, 320.0, Stroke::new(1.9 * pen.s, pen.ink));
            let a = 320.0f32 * PI / 180.0;
            let tip = (12.0 + 8.0 * a.cos() + 1.2, 12.0 + 8.0 * a.sin() + 1.2);
            pen.head(tip, (0.75, 0.9), 4.5, RED);
            pen.dot((12.0, 12.0), 2.0, BLUE);
            pen.line_c(&[(12.0, 12.0), (20.0, 12.0)], pen.thin());
        }
        Icon::Scale => {
            pen.painter.rect_stroke(
                Rect::from_min_max(pen.p(3.5, 3.5), pen.p(20.5, 20.5)),
                1.5 * pen.s,
                Stroke::new(1.0 * pen.s, SHADE_DARK),
                egui::StrokeKind::Middle,
            );
            pen.poly(&[(3.5, 12.0), (12.0, 12.0), (12.0, 20.5), (3.5, 20.5)], BLUE_SOFT);
            pen.line_c(&[(10.0, 14.0), (17.0, 7.0)], Stroke::new(1.8 * pen.s, RED));
            pen.head((20.0, 4.0), (1.0, -1.0), 4.5, RED);
            for c in [(3.5, 3.5), (20.5, 3.5), (20.5, 20.5)] {
                pen.dot(c, 1.6, GREEN);
            }
        }
        Icon::Offset => {
            pen.poly(&[(3.0, 3.0), (21.0, 3.0), (21.0, 21.0), (3.0, 21.0)], FILL);
            pen.poly(&[(8.0, 8.0), (16.0, 8.0), (16.0, 16.0), (8.0, 16.0)], BLUE_SOFT);
            pen.line_c(&[(5.0, 12.0), (7.0, 12.0)], Stroke::new(1.5 * pen.s, RED));
            pen.head((8.0, 12.0), (1.0, 0.0), 3.0, RED);
        }
        Icon::FollowMe => {
            // path
            let mut path = vec![pen.p(5.0, 21.0), pen.p(5.0, 13.0)];
            path.extend(pen.arc_pts((13.0, 13.0), 8.0, 180.0, 270.0));
            path.push(pen.p(20.0, 5.0));
            pen.painter.add(Shape::line(path, Stroke::new(1.6 * pen.s, RED)));
            pen.head((22.5, 5.0), (1.0, 0.0), 3.5, RED);
            // profile
            pen.poly(&[(2.0, 17.0), (8.0, 17.0), (8.0, 22.0), (2.0, 22.0)], BLUE_SOFT);
        }
        Icon::Tape => {
            pen.painter.rect(
                Rect::from_min_max(pen.p(3.0, 7.0), pen.p(15.0, 19.0)),
                3.0 * pen.s,
                YELLOW,
                pen.stroke(),
                egui::StrokeKind::Middle,
            );
            pen.circle((9.0, 13.0), 2.6, FILL);
            pen.poly(&[(15.0, 15.5), (22.0, 15.5), (22.0, 19.0), (15.0, 19.0)], FILL);
            for x in [17.0, 19.0, 21.0] {
                pen.line_c(&[(x, 15.5), (x, 17.0)], pen.thin());
            }
        }
        Icon::Protractor => {
            let mut pts = pen.arc_pts((12.0, 18.0), 9.5, 180.0, 360.0);
            pts.push(pen.p(2.5, 18.0));
            pen.painter.add(Shape::convex_polygon(pts, FILL, pen.stroke()));
            for i in 1..6 {
                let a = (180.0 + i as f32 * 30.0) * PI / 180.0;
                let (c, s) = (a.cos(), a.sin());
                pen.line_c(
                    &[(12.0 + 7.5 * c, 18.0 + 7.5 * s), (12.0 + 9.5 * c, 18.0 + 9.5 * s)],
                    pen.thin(),
                );
            }
            pen.line_c(&[(12.0, 18.0), (18.5, 9.0)], Stroke::new(1.6 * pen.s, RED));
            pen.dot((12.0, 18.0), 1.6, BLUE);
        }
        Icon::Orbit => {
            pen.circle((12.0, 12.0), 4.0, BLUE_SOFT);
            let st = Stroke::new(1.7 * pen.s, pen.ink);
            pen.arc((12.0, 12.0), 9.0, 200.0, 330.0, st);
            pen.arc((12.0, 12.0), 9.0, 20.0, 150.0, st);
            let a = 330.0f32 * PI / 180.0;
            pen.head((12.0 + 9.0 * a.cos() + 0.8, 12.0 + 9.0 * a.sin() + 1.4), (0.5, 0.87), 4.0, RED);
            let a = 150.0f32 * PI / 180.0;
            pen.head((12.0 + 9.0 * a.cos() - 0.8, 12.0 + 9.0 * a.sin() - 1.4), (-0.5, -0.87), 4.0, RED);
        }
        Icon::Pan => {
            // Palm
            pen.painter.rect(
                Rect::from_min_max(pen.p(6.5, 10.5), pen.p(18.0, 21.0)),
                4.0 * pen.s,
                SKIN,
                pen.stroke(),
                egui::StrokeKind::Middle,
            );
            // Fingers
            for (x, top) in [(8.3, 6.0), (11.2, 3.5), (14.1, 4.0), (16.8, 6.5)] {
                let r = Rect::from_min_max(pen.p(x - 1.4, top), pen.p(x + 1.4, 14.0));
                pen.painter.rect(r, 1.4 * pen.s, SKIN, pen.thin(), egui::StrokeKind::Middle);
            }
            // Thumb
            pen.painter.add(Shape::line(
                vec![pen.p(7.0, 17.0), pen.p(3.5, 13.0)],
                Stroke::new(3.0 * pen.s, pen.ink),
            ));
            pen.painter.add(Shape::line(
                vec![pen.p(7.0, 17.0), pen.p(3.6, 13.1)],
                Stroke::new(1.4 * pen.s, SKIN),
            ));
            // cover finger bottoms
            pen.painter.rect_filled(
                Rect::from_min_max(pen.p(7.4, 12.0), pen.p(17.2, 15.0)),
                0.0,
                SKIN,
            );
        }
        Icon::Zoom | Icon::ZoomExtents => {
            pen.circle((10.0, 10.0), 6.0, FILL);
            pen.painter.add(Shape::line(
                vec![pen.p(14.5, 14.5), pen.p(20.5, 20.5)],
                Stroke::new(3.2 * pen.s, pen.ink),
            ));
            if icon == Icon::Zoom {
                pen.line_c(&[(7.0, 10.0), (13.0, 10.0)], Stroke::new(1.5 * pen.s, BLUE));
                pen.line_c(&[(10.0, 7.0), (10.0, 13.0)], Stroke::new(1.5 * pen.s, BLUE));
            } else {
                let st = Stroke::new(1.4 * pen.s, RED);
                for (a, b, c) in [
                    ((1.5, 5.0), (1.5, 1.5), (5.0, 1.5)),
                    ((15.0, 1.5), (18.5, 1.5), (18.5, 5.0)),
                    ((1.5, 15.0), (1.5, 18.5), (5.0, 18.5)),
                ] {
                    pen.line_c(&[a, b, c], st);
                }
            }
        }
        Icon::ViewIso => cube(&pen, FILL, SHADE, SHADE_DARK),
        Icon::ViewTop => cube(&pen, BLUE, FILL, FILL),
        Icon::ViewFront => cube(&pen, FILL, BLUE, FILL),
        Icon::ViewRight => cube(&pen, FILL, FILL, BLUE),
        Icon::ViewBack | Icon::ViewLeft => {
            cube(&pen, FILL, FILL, FILL);
            let st = Stroke::new(1.0 * pen.s, SHADE_DARK);
            pen.dashed((12.0, 3.0), (12.0, 12.0), st);
        }
        Icon::StyleShaded => cube(&pen, Color32::from_rgb(0xF6, 0xE0, 0xA8), Color32::from_rgb(0xE4, 0xB8, 0x6A), Color32::from_rgb(0xC8, 0x92, 0x48)),
        Icon::StyleMonochrome => cube(&pen, FILL, SHADE, SHADE_DARK),
        Icon::StyleHiddenLine => cube(&pen, Color32::WHITE, Color32::WHITE, Color32::WHITE),
        Icon::StyleWireframe | Icon::StyleXRay => {
            let tr = if icon == Icon::StyleXRay {
                Color32::from_rgba_unmultiplied(0x9C, 0xB8, 0xE0, 90)
            } else {
                Color32::TRANSPARENT
            };
            cube(&pen, tr, tr, tr);
            let st = Stroke::new(1.0 * pen.s, if icon == Icon::StyleXRay { BLUE } else { pen.ink });
            pen.dashed((12.0, 12.0), (12.0, 3.0), st);
            pen.dashed((4.0, 16.5), (12.0, 12.0), st);
            pen.dashed((20.0, 16.5), (12.0, 12.0), st);
        }
        Icon::Perspective => {
            pen.poly(&[(8.0, 6.0), (16.0, 6.0), (21.0, 19.0), (3.0, 19.0)], FILL);
            pen.line_c(&[(12.0, 6.0), (12.0, 19.0)], pen.thin());
            pen.line_c(&[(6.0, 12.0), (18.0, 12.0)], pen.thin());
        }
        Icon::Undo | Icon::Redo => {
            let flip = if icon == Icon::Redo { -1.0 } else { 1.0 };
            let fx = |x: f32| if flip < 0.0 { 24.0 - x } else { x };
            let mut pts: Vec<Pos2> = Vec::new();
            for i in 0..=12 {
                let a = (-90.0 + i as f32 * 15.0) * PI / 180.0;
                pts.push(pen.p(fx(12.0 + 6.0 * a.cos()), 13.0 + 6.0 * a.sin()));
            }
            pts.insert(0, pen.p(fx(7.0), 7.0));
            pts.push(pen.p(fx(8.0), 19.0));
            pen.painter.add(Shape::line(pts, pen.stroke()));
            pen.head((fx(3.5), 7.0), (-flip, 0.0), 4.5, pen.ink);
        }
        Icon::Eye | Icon::EyeOff => {
            let mut pts = pen.arc_pts((12.0, 19.0), 11.0, 220.0, 320.0);
            pts.extend(pen.arc_pts((12.0, 5.0), 11.0, 40.0, 140.0));
            pen.painter.add(Shape::closed_line(pts, pen.stroke()));
            if icon == Icon::Eye {
                pen.dot((12.0, 12.0), 3.0, pen.ink);
            } else {
                pen.line_c(&[(4.0, 20.0), (20.0, 4.0)], pen.stroke());
            }
        }
        Icon::Plus => {
            pen.line(&[(12.0, 5.0), (12.0, 19.0)]);
            pen.line(&[(5.0, 12.0), (19.0, 12.0)]);
        }
        Icon::Trash => {
            pen.poly(&[(6.0, 8.0), (18.0, 8.0), (17.0, 21.0), (7.0, 21.0)], FILL);
            pen.line(&[(4.0, 7.0), (20.0, 7.0)]);
            pen.line(&[(9.5, 7.0), (10.0, 4.0), (14.0, 4.0), (14.5, 7.0)]);
            pen.line_c(&[(10.5, 11.0), (10.5, 18.0)], pen.thin());
            pen.line_c(&[(13.5, 11.0), (13.5, 18.0)], pen.thin());
        }
        Icon::Refresh => {
            pen.arc((12.0, 12.0), 7.0, -60.0, 230.0, pen.stroke());
            let a = -60.0f32 * PI / 180.0;
            pen.head((12.0 + 7.0 * a.cos() + 0.5, 12.0 + 7.0 * a.sin() + 3.5), (-0.2, 1.0), 4.5, pen.ink);
        }
        Icon::Info => {
            pen.circle((12.0, 12.0), 9.0, Color32::TRANSPARENT);
            pen.dot((12.0, 7.5), 1.4, pen.ink);
            pen.line(&[(12.0, 10.5), (12.0, 17.0)]);
        }
        Icon::Group => {
            pen.painter.rect_stroke(
                Rect::from_min_max(pen.p(3.0, 3.0), pen.p(21.0, 21.0)),
                2.0 * pen.s,
                Stroke::new(1.2 * pen.s, BLUE),
                egui::StrokeKind::Middle,
            );
            pen.poly(&[(7.0, 9.0), (13.0, 9.0), (13.0, 17.0), (7.0, 17.0)], BLUE_SOFT);
            pen.poly(&[(13.0, 9.0), (17.0, 7.0), (17.0, 15.0), (13.0, 17.0)], SHADE);
        }
    }
}
