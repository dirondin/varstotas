//! Vector toolbar icons drawn with egui's painter on a 24×24 design grid.
//! Consistent style: ~2px dark-grey strokes, soft fills, one accent colour per icon.

use egui::{Color32, Painter, Pos2, Rect, Shape, Stroke, Vec2, vec2};
use std::f32::consts::PI;

mod cmd;

#[allow(dead_code)] // full icon set; some are only used by later tools
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Icon {
    Select,
    Eraser,
    Paint,
    Line,
    Arc,
    ArcCenter,
    Arc3,
    Pie,
    Bezier,
    EditCurve,
    ModelInfo,
    SmoothCurve,
    Freehand,
    Rectangle,
    RotatedRectangle,
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
    Axes,
    Dimension,
    Section,
    Slice,
    Orbit,
    Pan,
    Zoom,
    ZoomExtents,
    PositionCamera,
    LookAround,
    Walk,
    Shadows,
    Text,
    Text3d,
    Chamfer,
    Fillet,
    Solid,
    ViewIso,
    ViewTop,
    ViewFront,
    ViewRight,
    ViewBack,
    ViewLeft,
    ViewBottom,
    StyleXRay,
    StyleWireframe,
    StyleHiddenLine,
    StyleShaded,
    StyleTextured,
    StyleMonochrome,
    StyleRealistic,
    Perspective,
    Parallel,
    StyleEdges,
    StyleProfiles,
    Guides,
    Ground,
    Sky,
    Grid,
    SectionCut,
    SectionFill,
    Panes1,
    Panes2,
    Panes4,
    Undo,
    Redo,
    Eye,
    EyeOff,
    Lock,
    Unlock,
    Plus,
    Cloud,
    More,
    Trash,
    Refresh,
    Info,
    Group,
    Magnet,
    GuidePlane,
    Box3d,
    Cylinder,
    Cone,
    Pyramid,
    Sphere,
    Torus,
    // menu commands (`cmd.rs`)
    FileNew,
    FileOpen,
    FileSave,
    FileSaveAs,
    /// The document's preview: 0 show, 1 from a file, 2 from the view, 3 remove.
    Picture(u8),
    /// A format's import / export (the format is written in where there is room).
    Import(&'static str),
    Export(&'static str),
    Film,
    Drawing,
    Sheets,
    Report,
    Json,
    Credits,
    Script,
    Quit,
    Cut,
    Copy,
    Paste,
    PasteInPlace,
    Purge,
    DeleteGuides,
    SelectAll,
    SelectNone,
    Component,
    Parametric,
    Explode,
    CloseGroup,
    /// Intersect Faces: 0 with the model, 1 with the selection, 2 with the context.
    Intersect(u8),
    ReverseFaces,
    OrientFaces,
    /// Flip Along: 0 red, 1 green, 2 blue.
    Flip(u8),
    /// A solid tool, by its index in `SolidOp::ALL`.
    Solids(u8),
    Unfold,
    Snap(crate::inference::InferenceKind),
    // View and Window menus (`cmd.rs`)
    SideBar,
    Outliner,
    Tag,
    Render,
    Console,
    Log,
    Gear,
    Generator,
    Play,
    Pause,
    NextScene,
    PrevScene,
    AddScene,
}

/// Palette used by icons.
pub mod pal {
    use egui::Color32;
    use super::super::theme::pick;
    // ink and the neutral fills follow the light / dark look; the colours stay
    pub fn ink() -> Color32 {
        pick(Color32::from_rgb(0x3A, 0x3F, 0x48), Color32::from_rgb(0xD5, 0xDA, 0xE2))
    }
    pub fn ink_active() -> Color32 {
        pick(Color32::from_rgb(0x12, 0x4A, 0x9C), Color32::from_rgb(0x8C, 0xC0, 0xFF))
    }
    pub fn fill() -> Color32 {
        pick(Color32::from_rgb(0xF4, 0xF6, 0xF9), Color32::from_rgb(0x4A, 0x50, 0x59))
    }
    pub fn shade() -> Color32 {
        pick(Color32::from_rgb(0xD5, 0xDB, 0xE4), Color32::from_rgb(0x5E, 0x66, 0x72))
    }
    pub fn shade_dark() -> Color32 {
        pick(Color32::from_rgb(0xAE, 0xB8, 0xC6), Color32::from_rgb(0x7A, 0x84, 0x93))
    }
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
    /// Points of an ellipse arc (degrees, 0 = +x, clockwise on screen).
    fn ell(&self, c: (f32, f32), rx: f32, ry: f32, a0: f32, a1: f32) -> Vec<(f32, f32)> {
        let n = ((a1 - a0).abs() / 10.0).ceil().max(2.0) as usize;
        (0..=n)
            .map(|i| {
                let a = (a0 + (a1 - a0) * i as f32 / n as f32) * PI / 180.0;
                (c.0 + rx * a.cos(), c.1 + ry * a.sin())
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
    let pen = Pen { painter, origin, s: side / 24.0, ink: if active { pal::ink_active() } else { pal::ink() } };
    use pal::*;
    #[allow(non_snake_case)]
    let (FILL, SHADE, SHADE_DARK) = (pal::fill(), pal::shade(), pal::shade_dark());
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
        Icon::ArcCenter => {
            pen.arc((8.0, 16.0), 11.0, 270.0, 360.0, pen.stroke());
            pen.line_c(&[(8.0, 16.0), (19.0, 16.0)], Stroke::new(1.0 * pen.s, SHADE_DARK));
            pen.line_c(&[(8.0, 16.0), (8.0, 5.0)], Stroke::new(1.0 * pen.s, SHADE_DARK));
            pen.dot((8.0, 16.0), 1.8, BLUE);
            pen.dot((19.0, 16.0), 2.0, RED);
            pen.dot((8.0, 5.0), 2.0, RED);
        }
        Icon::Arc3 => {
            pen.arc((12.0, 17.0), 9.0, 180.0, 360.0, pen.stroke());
            pen.dot((3.0, 17.0), 2.0, RED);
            pen.dot((18.4, 10.6), 2.0, RED);
            pen.dot((21.0, 17.0), 2.0, RED);
        }
        Icon::Bezier => {
            let curve: Vec<(f32, f32)> = (0..=16)
                .map(|k| {
                    let t = k as f32 / 16.0;
                    let u = 1.0 - t;
                    let (x0, y0, x1, y1, x2, y2, x3, y3) = (3.0, 18.0, 6.0, 3.0, 18.0, 21.0, 21.0, 6.0);
                    (
                        u * u * u * x0 + 3.0 * u * u * t * x1 + 3.0 * u * t * t * x2 + t * t * t * x3,
                        u * u * u * y0 + 3.0 * u * u * t * y1 + 3.0 * u * t * t * y2 + t * t * t * y3,
                    )
                })
                .collect();
            let thin = Stroke::new(1.0 * pen.s, BLUE);
            pen.line_c(&[(3.0, 18.0), (6.0, 3.0)], thin);
            pen.line_c(&[(21.0, 6.0), (18.0, 21.0)], thin);
            pen.line(&curve);
            pen.dot((6.0, 3.0), 1.6, BLUE);
            pen.dot((18.0, 21.0), 1.6, BLUE);
            pen.dot((3.0, 18.0), 2.0, RED);
            pen.dot((21.0, 6.0), 2.0, RED);
        }
        Icon::SmoothCurve => {
            // a wave through the clicked points (no handles)
            let curve: Vec<(f32, f32)> = (0..=24)
                .map(|k| {
                    let x = 3.0 + 18.0 * k as f32 / 24.0;
                    (x, 12.0 - 6.0 * ((x - 3.0) / 18.0 * 2.0 * PI).sin())
                })
                .collect();
            pen.line(&curve);
            for c in [(3.0, 12.0), (7.5, 6.0), (16.5, 18.0), (21.0, 12.0)] {
                pen.dot(c, 2.0, RED);
            }
        }
        Icon::Freehand => {
            // a looping hand-drawn stroke
            let curve: Vec<(f32, f32)> = (0..=48)
                .map(|k| {
                    let t = PI * (0.5 + 2.5 * k as f32 / 48.0);
                    (3.0 + 18.0 * (t - 0.5 * PI) / (2.5 * PI) - 3.4 * (t.sin() - 1.0), 12.0 - 6.0 * t.cos())
                })
                .collect();
            pen.line(&curve);
            pen.dot(curve[0], 2.0, RED);
        }
        Icon::Pie => {
            let mut pts = vec![pen.p(6.0, 18.0)];
            pts.extend(pen.arc_pts((6.0, 18.0), 14.0, 270.0, 360.0));
            pen.painter.add(Shape::convex_polygon(pts, BLUE_SOFT, pen.stroke()));
            pen.dot((6.0, 18.0), 1.8, RED);
        }
        Icon::RotatedRectangle => {
            pen.poly(&[(3.0, 13.0), (12.0, 4.0), (21.0, 13.0), (12.0, 22.0)], BLUE_SOFT);
            for c in [(3.0, 13.0), (12.0, 4.0), (21.0, 13.0)] {
                pen.dot(c, 2.0, RED);
            }
        }
        Icon::Axes => {
            let o = (8.0, 15.0);
            pen.line_c(&[o, (21.0, 15.0)], Stroke::new(1.8 * pen.s, RED));
            pen.line_c(&[o, (16.0, 8.0)], Stroke::new(1.8 * pen.s, GREEN));
            pen.line_c(&[o, (8.0, 3.0)], Stroke::new(1.8 * pen.s, BLUE));
            pen.line_c(&[o, (3.0, 20.0)], Stroke::new(1.0 * pen.s, SHADE_DARK));
            pen.dot(o, 2.0, YELLOW);
        }
        Icon::Dimension => {
            let thin = Stroke::new(1.0 * pen.s, SHADE_DARK);
            pen.line_c(&[(3.0, 20.0), (3.0, 9.0)], thin);
            pen.line_c(&[(21.0, 20.0), (21.0, 9.0)], thin);
            pen.line_c(&[(4.0, 12.0), (20.0, 12.0)], pen.thin());
            pen.head((3.5, 12.0), (-1.0, 0.0), 3.0, pen.ink);
            pen.head((20.5, 12.0), (1.0, 0.0), 3.0, pen.ink);
            pen.painter.rect_filled(Rect::from_min_max(pen.p(8.0, 4.5), pen.p(16.0, 9.5)), 1.0 * pen.s, YELLOW);
            pen.line_c(&[(3.0, 20.0), (21.0, 20.0)], pen.stroke());
        }
        Icon::Box3d => {
            pen.poly(&[(3.0, 10.0), (13.0, 10.0), (13.0, 21.0), (3.0, 21.0)], SHADE);
            pen.poly(&[(13.0, 10.0), (21.0, 5.0), (21.0, 16.0), (13.0, 21.0)], SHADE_DARK);
            pen.poly(&[(3.0, 10.0), (11.0, 5.0), (21.0, 5.0), (13.0, 10.0)], BLUE_SOFT);
        }
        Icon::Cylinder => {
            let mut side = pen.ell((12.0, 18.0), 8.0, 3.0, 0.0, 180.0);
            side.extend([(4.0, 6.0), (20.0, 6.0)]);
            pen.poly(&side, SHADE);
            pen.poly(&pen.ell((12.0, 6.0), 8.0, 3.0, 0.0, 360.0), BLUE_SOFT);
        }
        Icon::Cone => {
            let mut side = pen.ell((12.0, 18.0), 8.0, 3.0, 0.0, 180.0);
            side.push((12.0, 3.0));
            pen.poly(&side, SHADE);
            pen.line_c(&pen.ell((12.0, 18.0), 8.0, 3.0, 180.0, 360.0), pen.thin());
            pen.dot((12.0, 3.0), 1.6, RED);
        }
        Icon::Pyramid => {
            pen.poly(&[(12.0, 3.0), (3.0, 17.0), (12.0, 21.0)], SHADE);
            pen.poly(&[(12.0, 3.0), (12.0, 21.0), (21.0, 16.0)], SHADE_DARK);
            pen.dot((12.0, 3.0), 1.6, RED);
        }
        Icon::Sphere => {
            pen.circle((12.0, 12.0), 8.5, BLUE_SOFT);
            pen.line_c(&pen.ell((12.0, 12.0), 8.5, 3.0, 0.0, 180.0), pen.thin());
            pen.line_c(&pen.ell((12.0, 12.0), 3.0, 8.5, 270.0, 450.0), pen.thin());
        }
        Icon::Torus => {
            let outer = pen.ell((12.0, 12.0), 9.5, 6.0, 0.0, 360.0);
            let inner = pen.ell((12.0, 11.0), 4.0, 2.0, 0.0, 360.0);
            pen.line_c(&pen.ell((12.0, 11.5), 6.75, 4.0, 0.0, 360.0), Stroke::new(5.0 * pen.s, BLUE_SOFT));
            pen.line_c(&outer, pen.stroke());
            pen.line_c(&inner, pen.thin());
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
        Icon::ViewBottom => {
            // The hidden bottom face, tinted, with its hidden edges dashed.
            cube(&pen, FILL, FILL, FILL);
            let st = Stroke::new(1.0 * pen.s, BLUE);
            pen.poly_c(&[(12.0, 12.0), (20.0, 16.5), (12.0, 21.0), (4.0, 16.5)], Color32::from_rgba_unmultiplied(0x3A, 0x7B, 0xD5, 70), Stroke::NONE);
            pen.dashed((12.0, 12.0), (20.0, 16.5), st);
            pen.dashed((12.0, 12.0), (4.0, 16.5), st);
            pen.line_c(&[(4.0, 16.5), (12.0, 21.0), (20.0, 16.5)], Stroke::new(1.6 * pen.s, BLUE));
        }
        Icon::StyleShaded => cube(&pen, Color32::from_rgb(0xF6, 0xE0, 0xA8), Color32::from_rgb(0xE4, 0xB8, 0x6A), Color32::from_rgb(0xC8, 0x92, 0x48)),
        Icon::Text => {
            // A boxed "A" with a leader to a dot.
            pen.poly(&[(9.0, 3.0), (21.0, 3.0), (21.0, 13.0), (9.0, 13.0)], FILL);
            pen.line_c(&[(12.0, 11.0), (15.0, 5.0), (18.0, 11.0)], Stroke::new(1.4 * pen.s, BLUE));
            pen.line_c(&[(13.2, 8.8), (16.8, 8.8)], Stroke::new(1.2 * pen.s, BLUE));
            pen.line(&[(9.0, 13.0), (4.5, 19.5)]);
            pen.dot((4.0, 20.0), 2.0, RED);
        }
        Icon::Text3d => {
            // An extruded "A".
            let side = Stroke::new(3.2 * pen.s, SHADE_DARK);
            pen.line_c(&[(6.0, 21.0), (12.0, 5.0), (18.0, 21.0)], side);
            let front = Stroke::new(2.0 * pen.s, BLUE);
            pen.line_c(&[(4.5, 19.5), (10.5, 3.5), (16.5, 19.5)], front);
            pen.line_c(&[(7.0, 13.5), (14.0, 13.5)], front);
        }
        Icon::Chamfer => {
            // A block profile with the top-right corner cut flat.
            pen.poly(&[(3.0, 21.0), (3.0, 4.0), (13.0, 4.0), (20.0, 11.0), (20.0, 21.0)], FILL);
            pen.line_c(&[(13.0, 4.0), (20.0, 11.0)], Stroke::new(2.2 * pen.s, RED));
            pen.dashed((13.0, 4.0), (20.0, 4.0), pen.thin());
            pen.dashed((20.0, 4.0), (20.0, 11.0), pen.thin());
        }
        Icon::Solid => {
            // Two overlapping blocks, the overlap highlighted.
            pen.poly(&[(3.0, 9.0), (15.0, 9.0), (15.0, 21.0), (3.0, 21.0)], FILL);
            pen.poly(&[(9.0, 3.0), (21.0, 3.0), (21.0, 15.0), (9.0, 15.0)], FILL);
            pen.poly(&[(9.0, 9.0), (15.0, 9.0), (15.0, 15.0), (9.0, 15.0)], Color32::from_rgba_unmultiplied(0xD0, 0x30, 0x30, 0x90));
        }
        Icon::Fillet => {
            // A block profile with the top-right corner rounded.
            let mut pts = pen.pts(&[(3.0, 21.0), (3.0, 4.0)]);
            pts.extend(pen.arc_pts((13.0, 11.0), 7.0, 270.0, 360.0));
            pts.push(pen.p(20.0, 21.0));
            pen.painter.add(Shape::convex_polygon(pts, FILL, pen.stroke()));
            pen.arc((13.0, 11.0), 7.0, 270.0, 360.0, Stroke::new(2.2 * pen.s, RED));
            pen.dashed((13.0, 4.0), (20.0, 4.0), pen.thin());
            pen.dashed((20.0, 4.0), (20.0, 11.0), pen.thin());
        }
        Icon::Shadows => {
            // A sun over a cube with its shadow on the ground.
            pen.poly_c(&[(10.0, 20.0), (22.0, 20.0), (17.0, 15.0), (10.0, 15.0)], SHADE_DARK, Stroke::NONE);
            pen.poly(&[(4.0, 12.0), (10.0, 12.0), (10.0, 20.0), (4.0, 20.0)], FILL);
            let sun = Color32::from_rgb(0xF2, 0xB1, 0x2E);
            pen.dot((17.0, 6.0), 3.0, sun);
            for k in 0..8 {
                let a = k as f32 * std::f32::consts::FRAC_PI_4;
                pen.line_c(&[(17.0 + 4.3 * a.cos(), 6.0 + 4.3 * a.sin()), (17.0 + 5.8 * a.cos(), 6.0 + 5.8 * a.sin())], Stroke::new(1.2 * pen.s, sun));
            }
        }
        Icon::PositionCamera => {
            // A standing figure over a red ground mark.
            pen.circle((12.0, 4.5), 2.5, SKIN);
            pen.line(&[(12.0, 7.0), (12.0, 14.0)]);
            pen.line(&[(7.5, 10.0), (12.0, 8.5), (16.5, 10.0)]);
            pen.line(&[(9.0, 19.0), (12.0, 14.0), (15.0, 19.0)]);
            let st = Stroke::new(1.6 * pen.s, RED);
            pen.line_c(&[(5.0, 20.0), (19.0, 23.0)], st);
            pen.line_c(&[(5.0, 23.0), (19.0, 20.0)], st);
        }
        Icon::LookAround => {
            // An eye under a curved double arrow.
            let mut pts = pen.arc_pts((12.0, 23.0), 9.0, 220.0, 320.0);
            pts.extend(pen.arc_pts((12.0, 11.0), 9.0, 40.0, 140.0));
            pen.painter.add(Shape::closed_line(pts, pen.stroke()));
            pen.dot((12.0, 17.0), 2.5, pen.ink);
            let st = Stroke::new(1.5 * pen.s, BLUE);
            pen.arc((12.0, 14.0), 10.0, 205.0, 335.0, st);
            pen.head((2.6, 10.5), (-0.5, 0.9), 3.5, BLUE);
            pen.head((21.4, 10.5), (0.5, 0.9), 3.5, BLUE);
        }
        Icon::Walk => {
            // Two footprints.
            for (x, y) in [(8.0, 13.0), (16.0, 7.0)] {
                let mut pts = pen.arc_pts((x, y), 3.0, 180.0, 360.0);
                pts.extend(pen.arc_pts((x, y + 4.0), 2.6, 0.0, 180.0));
                pen.painter.add(Shape::convex_polygon(pts, SHADE, pen.thin()));
                pen.dot((x, y + 8.6), 2.0, pen.ink);
            }
        }
        Icon::Magnet => {
            // Horseshoe magnet opening up: red band, light pole tips.
            let band = Stroke::new(4.4 * pen.s, RED);
            pen.arc((12.0, 13.0), 5.8, 0.0, 180.0, band);
            pen.line_c(&[(6.2, 13.2), (6.2, 8.0)], band);
            pen.line_c(&[(17.8, 13.2), (17.8, 8.0)], band);
            let tip = Stroke::new(1.0 * pen.s, pen.ink);
            pen.poly_c(&[(4.0, 3.5), (8.4, 3.5), (8.4, 8.0), (4.0, 8.0)], FILL, tip);
            pen.poly_c(&[(15.6, 3.5), (20.0, 3.5), (20.0, 8.0), (15.6, 8.0)], FILL, tip);
        }
        Icon::GuidePlane => {
            // A tilted orange plane with a blue normal arrow and a red in-plane axis.
            let orange = Color32::from_rgb(0xE8, 0x7A, 0x1E);
            pen.poly_c(&[(2.5, 15.0), (14.0, 9.5), (21.5, 14.5), (10.0, 20.5)], Color32::from_rgba_unmultiplied(0xE8, 0x7A, 0x1E, 70), Stroke::new(1.4 * pen.s, orange));
            let blue = Color32::from_rgb(0x2A, 0x5C, 0xD6);
            pen.line_c(&[(12.0, 15.0), (12.0, 5.0)], Stroke::new(1.6 * pen.s, blue));
            pen.head((12.0, 3.0), (0.0, -1.0), 3.5, blue);
            pen.line_c(&[(12.0, 15.0), (17.5, 12.3)], Stroke::new(1.6 * pen.s, RED));
        }
        Icon::Section => {
            // A cube cut by an orange plane with an arrow.
            cube(&pen, FILL, SHADE, SHADE_DARK);
            let orange = Color32::from_rgb(0xE8, 0x7A, 0x1E);
            pen.poly_c(&[(2.5, 9.0), (13.0, 3.5), (21.5, 9.5), (11.0, 15.0)], Color32::from_rgba_unmultiplied(0xE8, 0x7A, 0x1E, 70), Stroke::new(1.4 * pen.s, orange));
            pen.line_c(&[(12.0, 9.25), (12.0, 20.5)], Stroke::new(1.4 * pen.s, orange));
            pen.head((12.0, 22.0), (0.0, 1.0), 3.5, orange);
        }
        Icon::Slice => {
            // A block cut in two: the upper part lifted off, the cut face tinted red.
            let red = Color32::from_rgb(0xD6, 0x3A, 0x2A);
            let st = Stroke::new(1.2 * pen.s, Color32::from_rgb(0x3A, 0x3A, 0x40));
            pen.poly_c(&[(4.0, 13.0), (12.0, 17.0), (12.0, 21.5), (4.0, 17.5)], SHADE, st);
            pen.poly_c(&[(12.0, 17.0), (20.0, 13.0), (20.0, 17.5), (12.0, 21.5)], SHADE_DARK, st);
            pen.poly_c(&[(4.0, 13.0), (12.0, 9.0), (20.0, 13.0), (12.0, 17.0)], Color32::from_rgba_unmultiplied(0xD6, 0x3A, 0x2A, 90), Stroke::new(1.2 * pen.s, red));
            pen.poly_c(&[(4.0, 5.5), (12.0, 9.5), (12.0, 13.0), (4.0, 9.0)], SHADE, st);
            pen.poly_c(&[(12.0, 9.5), (20.0, 5.5), (20.0, 9.0), (12.0, 13.0)], SHADE_DARK, st);
            pen.poly_c(&[(4.0, 5.5), (12.0, 1.5), (20.0, 5.5), (12.0, 9.5)], FILL, st);
        }
        Icon::StyleTextured => {
            // Brick cube: mortar rows on the sides, staggered joints.
            cube(&pen, Color32::from_rgb(0xD8, 0x7A, 0x5C), Color32::from_rgb(0xB8, 0x55, 0x3A), Color32::from_rgb(0x96, 0x40, 0x2C));
            let st = Stroke::new(0.9 * pen.s, Color32::from_rgb(0xF2, 0xE6, 0xD8));
            for k in [3.0, 6.0] {
                pen.line_c(&[(4.0, 7.5 + k), (12.0, 12.0 + k)], st);
                pen.line_c(&[(12.0, 12.0 + k), (20.0, 7.5 + k)], st);
            }
            for (x, k) in [(8.0, 0.0), (6.0, 3.0), (10.0, 3.0), (8.0, 6.0)] {
                let y = 7.5 + (x - 4.0) * 4.5 / 8.0 + k;
                pen.line_c(&[(x, y), (x, y + 3.0)], st);
                let xr = 24.0 - x;
                pen.line_c(&[(xr, y), (xr, y + 3.0)], st);
            }
            pen.line_c(&[(8.0, 5.25), (16.0, 9.75)], st);
            cube(&pen, Color32::TRANSPARENT, Color32::TRANSPARENT, Color32::TRANSPARENT);
        }
        Icon::StyleMonochrome => cube(&pen, FILL, SHADE, SHADE_DARK),
        Icon::StyleRealistic => {
            // A glossy ball on the ground: soft shadow, shading rings, a sharp highlight.
            pen.painter.add(Shape::ellipse_filled(pen.p(12.5, 20.5), vec2(8.0, 1.8) * pen.s, Color32::from_black_alpha(55)));
            for (k, c) in [(0.0, 0x3A5F9E), (1.6, 0x4F78BA), (3.2, 0x6C93D2), (4.8, 0x93B4E6)] {
                let r = 8.5 - k;
                let c = Color32::from_rgb((c >> 16) as u8, (c >> 8) as u8, c as u8);
                pen.dot((12.0 - k * 0.45, 11.0 - k * 0.45), r, c);
            }
            pen.dot((8.6, 7.6), 1.6, Color32::WHITE);
            pen.arc((12.0, 11.0), 8.5, 0.0, 360.0, pen.thin());
        }
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
        Icon::Parallel => {
            // Like Perspective, but the sides stay parallel.
            pen.poly(&[(4.0, 6.0), (20.0, 6.0), (20.0, 19.0), (4.0, 19.0)], FILL);
            pen.line_c(&[(12.0, 6.0), (12.0, 19.0)], pen.thin());
            pen.line_c(&[(4.0, 12.5), (20.0, 12.5)], pen.thin());
        }
        Icon::StyleEdges => {
            cube(&pen, Color32::WHITE, FILL, FILL);
            let st = Stroke::new(1.8 * pen.s, BLUE);
            pen.line_c(&[(4.0, 7.5), (12.0, 12.0), (20.0, 7.5)], st);
            pen.line_c(&[(12.0, 12.0), (12.0, 21.0)], st);
        }
        Icon::StyleProfiles => {
            cube(&pen, Color32::WHITE, FILL, FILL);
            pen.poly_c(&[(12.0, 3.0), (20.0, 7.5), (20.0, 16.5), (12.0, 21.0), (4.0, 16.5), (4.0, 7.5)], Color32::TRANSPARENT, Stroke::new(2.8 * pen.s, pen.ink));
        }
        Icon::Guides => {
            // A dashed construction line across a tape measure's end point.
            pen.dashed((2.5, 20.5), (21.5, 3.5), pen.stroke());
            pen.dot((7.0, 16.5), 2.2, RED);
            pen.dot((17.0, 7.6), 2.2, RED);
        }
        Icon::Ground => {
            // A ground plane under the horizon, the sky above.
            pen.poly_c(&[(2.0, 4.0), (22.0, 4.0), (22.0, 12.0), (2.0, 12.0)], BLUE_SOFT, Stroke::NONE);
            pen.poly_c(&[(2.0, 12.0), (22.0, 12.0), (22.0, 20.0), (2.0, 20.0)], Color32::from_rgb(0xC9, 0xB8, 0x96), Stroke::NONE);
            pen.line(&[(2.0, 12.0), (22.0, 12.0)]);
        }
        Icon::Sky => {
            // The sky over the horizon, with the sun.
            pen.poly_c(&[(2.0, 4.0), (22.0, 4.0), (22.0, 18.0), (2.0, 18.0)], BLUE_SOFT, Stroke::NONE);
            pen.circle((15.0, 10.0), 3.2, Color32::from_rgb(0xF2, 0xC2, 0x3A));
            pen.line(&[(2.0, 18.0), (22.0, 18.0)]);
        }
        Icon::Grid => {
            // A grid on the ground plane, seen at an angle.
            let (a, b, c, d) = ((2.0, 15.0), (12.0, 9.0), (22.0, 15.0), (12.0, 21.0));
            pen.poly_c(&[a, b, c, d], FILL, pen.stroke());
            let lerp = |p: (f32, f32), q: (f32, f32), t: f32| (p.0 + (q.0 - p.0) * t, p.1 + (q.1 - p.1) * t);
            for t in [1.0 / 3.0, 2.0 / 3.0] {
                pen.line_c(&[lerp(a, b, t), lerp(d, c, t)], pen.thin());
                pen.line_c(&[lerp(a, d, t), lerp(b, c, t)], pen.thin());
            }
        }
        Icon::SectionCut => {
            // A cube whose cut outline is drawn thick in orange.
            cube(&pen, FILL, SHADE, SHADE_DARK);
            let orange = Color32::from_rgb(0xE8, 0x7A, 0x1E);
            pen.line_c(&[(4.0, 12.0), (12.0, 16.5), (20.0, 12.0)], Stroke::new(2.6 * pen.s, orange));
        }
        Icon::SectionFill => {
            // A cube cut in half: the cut face filled dark.
            pen.poly(&[(4.0, 12.0), (12.0, 16.5), (12.0, 21.0), (4.0, 16.5)], SHADE);
            pen.poly(&[(12.0, 16.5), (20.0, 12.0), (20.0, 16.5), (12.0, 21.0)], SHADE_DARK);
            pen.poly(&[(12.0, 7.5), (20.0, 12.0), (12.0, 16.5), (4.0, 12.0)], pen.ink);
        }
        Icon::Panes1 | Icon::Panes2 | Icon::Panes4 => {
            // The view's frame split into its panes.
            pen.poly_c(&[(3.0, 5.0), (21.0, 5.0), (21.0, 19.0), (3.0, 19.0)], BLUE_SOFT, pen.stroke());
            if icon != Icon::Panes1 {
                pen.line(&[(12.0, 5.0), (12.0, 19.0)]);
            }
            if icon == Icon::Panes4 {
                pen.line(&[(3.0, 12.0), (21.0, 12.0)]);
            }
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
        Icon::Lock | Icon::Unlock => {
            pen.line(&[(6.0, 11.0), (6.0, 21.0), (19.0, 21.0), (19.0, 11.0), (6.0, 11.0)]);
            if icon == Icon::Unlock {
                // Swing outward around the right attachment; keep the hinge in place.
                pen.line(&[(16.0, 11.0), (16.0, 6.0), (18.0, 3.0), (21.0, 3.0), (23.0, 6.0), (23.0, 8.0)]);
            } else {
                pen.line(&[(7.0, 11.0), (7.0, 6.0), (9.0, 3.0), (14.0, 3.0), (16.0, 6.0), (16.0, 11.0)]);
            }
            pen.line(&[(12.0, 15.0), (12.0, 18.0)]);
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
        Icon::Cloud => {
            // A cloud with an arrow down: the online library.
            pen.arc((8.5, 13.0), 4.0, 90.0, 270.0, pen.stroke());
            pen.arc((12.5, 10.0), 5.0, 180.0, 340.0, pen.stroke());
            pen.arc((16.5, 13.5), 3.5, -90.0, 90.0, pen.stroke());
            pen.line(&[(8.5, 17.0), (10.0, 17.0)]);
            pen.line(&[(14.0, 17.0), (16.5, 17.0)]);
            pen.line(&[(12.0, 12.0), (12.0, 21.0)]);
            pen.line(&[(9.5, 18.5), (12.0, 21.0), (14.5, 18.5)]);
        }
        Icon::More => {
            for x in [6.0, 12.0, 18.0] {
                pen.dot((x, 12.0), 1.7, pen.ink);
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
        _ => cmd::paint(&pen, icon),
    }
}
