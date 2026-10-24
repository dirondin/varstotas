//! Icons of the UV Editor toolbar, same 24×24 grid and style.

#[cfg(test)]
use super::Icon;
use super::{Pen, W, pal};
use egui::{Color32, Stroke};

const ORANGE: Color32 = Color32::from_rgb(0xE8, 0x7A, 0x1E);
const ISLAND_FILL: Color32 = Color32::from_rgb(0xF7, 0xC8, 0x9A);

/// The UV Editor's own icons.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum UvIcon {
    /// Pick whole islands.
    IslandMode,
    /// Pick single faces.
    FaceMode,
    Planar,
    RotateCcw,
    RotateCw,
    FlipU,
    FlipV,
    Fit,
    Pack,
    Align,
    ResetRotation,
    ResetScale,
    Reset,
    SeamMark,
    SeamClear,
    SeamHard,
    Tile,
    Checker,
}

#[cfg(test)]
impl UvIcon {
    pub const ALL: [UvIcon; 18] = [
        UvIcon::IslandMode,
        UvIcon::FaceMode,
        UvIcon::Planar,
        UvIcon::RotateCcw,
        UvIcon::RotateCw,
        UvIcon::FlipU,
        UvIcon::FlipV,
        UvIcon::Fit,
        UvIcon::Pack,
        UvIcon::Align,
        UvIcon::ResetRotation,
        UvIcon::ResetScale,
        UvIcon::Reset,
        UvIcon::SeamMark,
        UvIcon::SeamClear,
        UvIcon::SeamHard,
        UvIcon::Tile,
        UvIcon::Checker,
    ];
}

fn quad(pen: &Pen, pts: &[(f32, f32)], fill: Color32) {
    pen.poly_c(pts, fill, pen.thin());
}

/// A small island (an L of three cells) at `o`, `k` units a cell.
fn island(pen: &Pen, o: (f32, f32), k: f32, fill: Color32, stroke: Stroke) {
    let (x, y) = o;
    let pts = [(x, y), (x + k, y), (x + k, y + k), (x + 2.0 * k, y + k), (x + 2.0 * k, y + 2.0 * k), (x, y + 2.0 * k)];
    pen.poly_c(&pts, fill, Stroke::NONE);
    pen.path(pen.pts(&pts), true, stroke);
}

/// A turn arrow round (12, 12): anticlockwise on screen when `ccw`.
fn turn(pen: &Pen, r: f32, ccw: bool, color: Color32) {
    let st = Stroke::new(W * pen.s, color);
    if ccw {
        pen.arc((12.0, 12.0), r, 200.0, 340.0, st);
        let a = 200.0f32.to_radians();
        pen.head((12.0 + r * a.cos(), 12.0 + r * a.sin() + 0.5), (-0.35, 1.0), 4.2, color);
    } else {
        pen.arc((12.0, 12.0), r, 200.0, 340.0, st);
        let a = 340.0f32.to_radians();
        pen.head((12.0 + r * a.cos(), 12.0 + r * a.sin() + 0.5), (0.35, 1.0), 4.2, color);
    }
}

pub(super) fn paint(pen: &Pen, icon: UvIcon) {
    use pal::*;
    let (fill, shade, shade_dark) = (pal::fill(), pal::shade(), pal::shade_dark());
    let accent = Stroke::new(W * pen.s, ORANGE);
    match icon {
        UvIcon::IslandMode => {
            island(pen, (3.0, 3.0), 5.0, ISLAND_FILL, accent);
            island(pen, (13.0, 11.0), 4.0, fill, pen.thin());
        }
        UvIcon::FaceMode => {
            for (i, (x, y)) in [(4.0, 4.0), (12.0, 4.0), (4.0, 12.0), (12.0, 12.0)].into_iter().enumerate() {
                let c = if i == 1 { ISLAND_FILL } else { fill };
                quad(pen, &[(x, y), (x + 8.0, y), (x + 8.0, y + 8.0), (x, y + 8.0)], c);
            }
            pen.path(pen.pts(&[(12.0, 4.0), (20.0, 4.0), (20.0, 12.0), (12.0, 12.0)]), true, accent);
        }
        UvIcon::Planar => {
            // a tile under three arrows falling straight onto it
            pen.poly_c(&[(3.0, 15.0), (15.0, 15.0), (21.0, 20.5), (9.0, 20.5)], BLUE_SOFT, pen.stroke());
            for x in [7.0, 12.0, 17.0] {
                pen.line_c(&[(x, 3.0), (x, 11.0)], pen.thin());
                pen.head((x, 14.0), (0.0, 1.0), 3.2, pen.ink);
            }
        }
        UvIcon::RotateCcw | UvIcon::RotateCw => {
            let ccw = icon == UvIcon::RotateCcw;
            quad(pen, &[(7.0, 10.0), (17.0, 10.0), (17.0, 20.0), (7.0, 20.0)], fill);
            // the 90° mark: a corner tick
            pen.line_c(&[(7.0, 16.0), (11.0, 16.0), (11.0, 20.0)], pen.thin());
            turn(pen, 8.0, ccw, ORANGE);
        }
        UvIcon::FlipU | UvIcon::FlipV => {
            let u = icon == UvIcon::FlipU;
            let tf = |p: (f32, f32)| if u { p } else { (p.1, p.0) };
            let left: Vec<(f32, f32)> = [(10.0, 5.0), (10.0, 19.0), (3.0, 19.0)].into_iter().map(tf).collect();
            let right: Vec<(f32, f32)> = [(14.0, 5.0), (14.0, 19.0), (21.0, 19.0)].into_iter().map(tf).collect();
            pen.poly_c(&left, ISLAND_FILL, pen.thin());
            pen.poly_c(&right, fill, Stroke::NONE);
            pen.dashed(right[0], right[1], pen.thin());
            pen.dashed(right[1], right[2], pen.thin());
            pen.dashed(right[2], right[0], pen.thin());
            pen.dashed(tf((12.0, 2.5)), tf((12.0, 21.5)), Stroke::new(W * pen.s, ORANGE));
        }
        UvIcon::Fit => {
            pen.painter.rect_stroke(egui::Rect::from_min_max(pen.p(3.0, 3.0), pen.p(21.0, 21.0)), 0.0, pen.thin(), egui::StrokeKind::Middle);
            island(pen, (8.0, 8.0), 4.0, ISLAND_FILL, accent);
            for (c, d) in [((3.5, 3.5), (-1.0, -1.0)), ((20.5, 3.5), (1.0, -1.0)), ((20.5, 20.5), (1.0, 1.0)), ((3.5, 20.5), (-1.0, 1.0))] {
                pen.head(c, d, 3.6, pen.ink);
            }
        }
        UvIcon::Pack => {
            pen.painter.rect_stroke(egui::Rect::from_min_max(pen.p(3.0, 3.0), pen.p(21.0, 21.0)), 0.0, pen.thin(), egui::StrokeKind::Middle);
            quad(pen, &[(5.0, 5.0), (13.0, 5.0), (13.0, 13.0), (5.0, 13.0)], ISLAND_FILL);
            quad(pen, &[(15.0, 5.0), (19.0, 5.0), (19.0, 13.0), (15.0, 13.0)], fill);
            quad(pen, &[(5.0, 15.0), (19.0, 15.0), (19.0, 19.0), (5.0, 19.0)], shade);
        }
        UvIcon::Align => {
            // a tilted island ghost and the straightened one on a baseline
            pen.path(pen.pts(&[(6.0, 6.5), (15.5, 3.5), (17.0, 8.5), (7.5, 11.5)]), true, Stroke::new(1.0 * pen.s, shade_dark));
            quad(pen, &[(5.0, 13.0), (16.0, 13.0), (16.0, 18.0), (5.0, 18.0)], ISLAND_FILL);
            pen.line_c(&[(3.0, 20.5), (21.0, 20.5)], accent);
        }
        UvIcon::ResetRotation => {
            pen.path(pen.pts(&[(12.0, 3.0), (20.0, 11.0), (12.0, 19.0), (4.0, 11.0)]), true, Stroke::new(1.0 * pen.s, shade_dark));
            quad(pen, &[(7.0, 6.0), (17.0, 6.0), (17.0, 16.0), (7.0, 16.0)], ISLAND_FILL);
            pen.line_c(&[(4.0, 20.5), (20.0, 20.5)], accent);
            pen.dot((12.0, 11.0), 1.4, pen.ink);
        }
        UvIcon::ResetScale => {
            pen.painter.rect_stroke(egui::Rect::from_min_max(pen.p(3.0, 3.0), pen.p(21.0, 21.0)), 0.0, Stroke::new(1.0 * pen.s, shade_dark), egui::StrokeKind::Middle);
            quad(pen, &[(3.0, 13.0), (11.0, 13.0), (11.0, 21.0), (3.0, 21.0)], ISLAND_FILL);
            pen.line_c(&[(19.0, 5.0), (13.5, 10.5)], accent);
            pen.head((12.0, 12.0), (-1.0, 1.0), 4.0, ORANGE);
        }
        UvIcon::Reset => {
            quad(pen, &[(5.0, 5.0), (19.0, 5.0), (19.0, 19.0), (5.0, 19.0)], fill);
            for x in [9.67, 14.33] {
                pen.line_c(&[(x, 5.0), (x, 19.0)], Stroke::new(1.0 * pen.s, shade_dark));
                pen.line_c(&[(5.0, x), (19.0, x)], Stroke::new(1.0 * pen.s, shade_dark));
            }
            pen.arc((12.0, 12.0), 9.0, -60.0, 200.0, accent);
            let a = (-60.0f32).to_radians();
            pen.head((12.0 + 9.0 * a.cos() + 0.5, 12.0 + 9.0 * a.sin() + 3.5), (-0.2, 1.0), 4.2, ORANGE);
        }
        UvIcon::SeamMark | UvIcon::SeamClear => {
            quad(pen, &[(3.0, 6.0), (12.0, 4.0), (12.0, 20.0), (3.0, 18.0)], fill);
            quad(pen, &[(12.0, 4.0), (21.0, 6.0), (21.0, 18.0), (12.0, 20.0)], shade);
            let st = Stroke::new(W * 1.4 * pen.s, RED);
            if icon == UvIcon::SeamMark {
                pen.line_c(&[(12.0, 4.0), (12.0, 20.0)], st);
            } else {
                pen.dashed((12.0, 4.0), (12.0, 20.0), Stroke::new(W * pen.s, RED));
                let x = Stroke::new(W * pen.s, pen.ink);
                pen.line_c(&[(15.0, 9.0), (20.0, 14.0)], x);
                pen.line_c(&[(20.0, 9.0), (15.0, 14.0)], x);
            }
        }
        UvIcon::SeamHard => {
            let top = [(12.0, 3.0), (20.0, 7.5), (12.0, 12.0), (4.0, 7.5)];
            pen.poly_c(&top, fill, Stroke::NONE);
            pen.poly_c(&[(4.0, 7.5), (12.0, 12.0), (12.0, 21.0), (4.0, 16.5)], shade, Stroke::NONE);
            pen.poly_c(&[(12.0, 12.0), (20.0, 7.5), (20.0, 16.5), (12.0, 21.0)], shade_dark, Stroke::NONE);
            pen.path(pen.pts(&[(12.0, 3.0), (20.0, 7.5), (20.0, 16.5), (12.0, 21.0), (4.0, 16.5), (4.0, 7.5)]), true, pen.thin());
            let st = Stroke::new(W * 1.3 * pen.s, RED);
            pen.line_c(&[(4.0, 7.5), (12.0, 12.0), (20.0, 7.5)], st);
            pen.line_c(&[(12.0, 12.0), (12.0, 21.0)], st);
        }
        UvIcon::Tile => {
            for (i, (x, y)) in [(3.0, 3.0), (12.0, 3.0), (3.0, 12.0), (12.0, 12.0)].into_iter().enumerate() {
                let c = if i == 2 { BLUE_SOFT } else { fill };
                quad(pen, &[(x, y), (x + 9.0, y), (x + 9.0, y + 9.0), (x, y + 9.0)], c);
            }
            pen.path(pen.pts(&[(3.0, 12.0), (12.0, 12.0), (12.0, 21.0), (3.0, 21.0)]), true, Stroke::new(W * pen.s, YELLOW));
        }
        UvIcon::Checker => {
            for i in 0..4 {
                for j in 0..4 {
                    let (x, y) = (3.0 + i as f32 * 4.5, 3.0 + j as f32 * 4.5);
                    let c = if (i + j) % 2 == 0 { fill } else { pen.ink };
                    pen.poly_c(&[(x, y), (x + 4.5, y), (x + 4.5, y + 4.5), (x, y + 4.5)], c, Stroke::NONE);
                }
            }
            pen.painter.rect_stroke(egui::Rect::from_min_max(pen.p(3.0, 3.0), pen.p(21.0, 21.0)), 0.0, pen.thin(), egui::StrokeKind::Middle);
        }
    }
}

/// All UV icons (contact sheet).
#[cfg(test)]
pub(super) fn all() -> impl Iterator<Item = Icon> {
    UvIcon::ALL.into_iter().map(Icon::Uv)
}
