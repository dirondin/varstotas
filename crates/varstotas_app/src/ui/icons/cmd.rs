//! Icons of the menu commands (File, Edit, Solid Tools, Snapping), same 24×24 grid and style.

use super::{W, W_BOLD, W_THIN, CUBE_INNER, CUBE_LEFT, CUBE_OUTLINE, CUBE_RIGHT, CUBE_TOP, Icon, Pen, pal};
use crate::inference::InferenceKind;
use egui::{Align2, Color32, FontId, Rect, Stroke};
use std::f32::consts::PI;

const ORANGE: Color32 = Color32::from_rgb(0xE8, 0x7A, 0x1E);
const CYAN: Color32 = Color32::from_rgb(0x1F, 0xA9, 0xC4);
const MAGENTA: Color32 = Color32::from_rgb(0xC8, 0x3C, 0xC0);
const PURPLE: Color32 = Color32::from_rgb(0x9B, 0x59, 0xD0);
const BACK: Color32 = Color32::from_rgb(0x9D, 0xB0, 0xC9);
const WOOD: Color32 = Color32::from_rgb(0xE6, 0xC8, 0x96);

fn rect(pen: &Pen, (x0, y0): (f32, f32), (x1, y1): (f32, f32), fill: Color32, stroke: Stroke) {
    pen.poly_c(&[(x0, y0), (x1, y0), (x1, y1), (x0, y1)], fill, stroke);
}

fn rounded(pen: &Pen, a: (f32, f32), b: (f32, f32), stroke: Stroke) {
    pen.painter.rect_stroke(Rect::from_min_max(pen.p(a.0, a.1), pen.p(b.0, b.1)), 2.0 * pen.s, stroke, egui::StrokeKind::Middle);
}

fn dashed_rect(pen: &Pen, (x0, y0): (f32, f32), (x1, y1): (f32, f32), stroke: Stroke) {
    for (a, b) in [((x0, y0), (x1, y0)), ((x1, y0), (x1, y1)), ((x1, y1), (x0, y1)), ((x0, y1), (x0, y0))] {
        pen.dashed(a, b, stroke);
    }
}

/// A sheet of paper with its top-right corner folded.
fn page(pen: &Pen, (x0, y0): (f32, f32), (x1, y1): (f32, f32), fold: f32) {
    pen.poly(&[(x0, y0), (x1 - fold, y0), (x1, y0 + fold), (x1, y1), (x0, y1)], pal::fill());
    pen.line_c(&[(x1 - fold, y0), (x1 - fold, y0 + fold), (x1, y0 + fold)], pen.thin());
}

/// The isometric cube scaled by `k` about (12, 12) and moved to `c`.
fn mini_cube(pen: &Pen, c: (f32, f32), k: f32) {
    let t = |pts: &[(f32, f32)]| pts.iter().map(|(x, y)| (c.0 + (x - 12.0) * k, c.1 + (y - 12.0) * k)).collect::<Vec<_>>();
    let (top, left, right) = (t(&CUBE_TOP), t(&CUBE_LEFT), t(&CUBE_RIGHT));
    let inner = CUBE_INNER.map(t);
    pen.solid(
        &[(&top, pal::fill()), (&left, pal::shade()), (&right, pal::shade_dark())],
        &t(&CUBE_OUTLINE),
        &[&inner[0], &inner[1]],
        pen.thin(),
    );
}

/// A red cross on a light disc (remove / delete marks).
fn red_x(pen: &Pen, c: (f32, f32), r: f32) {
    pen.painter.circle(pen.p(c.0, c.1), r * pen.s, pal::fill(), pen.thin());
    let d = r * 0.45;
    let st = Stroke::new(W * pen.s, pal::RED);
    pen.line_c(&[(c.0 - d, c.1 - d), (c.0 + d, c.1 + d)], st);
    pen.line_c(&[(c.0 - d, c.1 + d), (c.0 + d, c.1 - d)], st);
}

fn floppy(pen: &Pen, o: (f32, f32), k: f32) {
    let t = |x: f32, y: f32| (o.0 + x * k, o.1 + y * k);
    pen.poly(&[t(3.0, 3.0), t(17.0, 3.0), t(21.0, 7.0), t(21.0, 21.0), t(3.0, 21.0)], pal::BLUE);
    rect(pen, t(7.0, 3.0), t(16.0, 9.0), pal::fill(), pen.thin());
    rect(pen, t(12.5, 4.3), t(14.5, 7.7), pal::BLUE, Stroke::NONE);
    rect(pen, t(6.5, 13.0), t(17.5, 21.0), pal::fill(), pen.thin());
}

/// Text inside an icon, only where it is big enough to read (the toolbar, not the menus).
fn tag(pen: &Pen, c: (f32, f32), size: f32, text: &str, color: Color32) {
    if pen.s >= 0.9 {
        pen.painter.text(pen.p(c.0, c.1), Align2::CENTER_CENTER, text, FontId::proportional(size * pen.s), color);
    }
}

/// A tray with an arrow going into it (import) or out of it (export).
fn file_io(pen: &Pen, tag_text: &str, import: bool) {
    let big = pen.s >= 0.9;
    let top = if big { 12.0 } else { 13.0 };
    rect(pen, (2.5, top), (21.5, 21.5), pal::fill(), pen.stroke());
    let col = if import { pal::GREEN } else { pal::BLUE };
    let st = Stroke::new(W_BOLD * pen.s, col);
    if import {
        pen.line_c(&[(12.0, 1.5), (12.0, top - 2.0)], st);
        pen.head((12.0, top + 1.5), (0.0, 1.0), 4.5, col);
    } else {
        pen.line_c(&[(12.0, top + 2.0), (12.0, 5.0)], st);
        pen.head((12.0, 1.5), (0.0, -1.0), 4.5, col);
    }
    if big {
        tag(pen, (12.0, 17.2), 7.2, tag_text, pal::ink());
    } else {
        pen.line_c(&[(7.0, 17.5), (17.0, 17.5)], pen.thin());
    }
}

pub(super) fn paint(pen: &Pen, icon: Icon) {
    use pal::*;
    #[allow(non_snake_case)]
    let (FILL, SHADE, SHADE_DARK) = (pal::fill(), pal::shade(), pal::shade_dark());
    match icon {
        Icon::ModelInfo => {
            // the document's page with an info badge
            page(pen, (3.0, 2.0), (17.0, 21.0), 4.5);
            for y in [8.0, 11.5, 15.0] {
                pen.line_c(&[(6.0, y), (12.0, y)], pen.thin());
            }
            pen.circle((17.0, 17.0), 5.5, BLUE);
            let white = Stroke::new(W * pen.s, Color32::WHITE);
            pen.dot((17.0, 14.4), 0.9, Color32::WHITE);
            pen.line_c(&[(17.0, 16.3), (17.0, 19.8)], white);
        }
        Icon::EditCurve => {
            // a curve with its points, one point grabbed with its handle
            let curve: Vec<(f32, f32)> = (0..=20)
                .map(|k| {
                    let t = k as f32 / 20.0;
                    let u = 1.0 - t;
                    let (x0, y0, x1, y1, x2, y2, x3, y3) = (2.0, 20.0, 6.0, 4.0, 18.0, 4.0, 22.0, 20.0);
                    (
                        u * u * u * x0 + 3.0 * u * u * t * x1 + 3.0 * u * t * t * x2 + t * t * t * x3,
                        u * u * u * y0 + 3.0 * u * u * t * y1 + 3.0 * u * t * t * y2 + t * t * t * y3,
                    )
                })
                .collect();
            pen.line(&curve);
            let thin = Stroke::new(W_THIN * pen.s, BLUE);
            pen.line_c(&[(5.0, 8.0), (19.0, 8.0)], thin);
            pen.circle((5.0, 8.0), 1.7, FILL);
            pen.circle((19.0, 8.0), 1.7, FILL);
            for c in [(2.0, 20.0), (22.0, 20.0)] {
                rect(pen, (c.0 - 1.8, c.1 - 1.8), (c.0 + 1.8, c.1 + 1.8), FILL, pen.thin());
            }
            rect(pen, (10.0, 6.0), (14.0, 10.0), BLUE, pen.thin());
        }
        Icon::FileNew => {
            page(pen, (4.0, 2.0), (18.0, 21.0), 5.0);
            pen.dot((17.0, 17.0), 5.0, GREEN);
            let st = Stroke::new(W * pen.s, Color32::WHITE);
            pen.line_c(&[(17.0, 14.0), (17.0, 20.0)], st);
            pen.line_c(&[(14.0, 17.0), (20.0, 17.0)], st);
        }
        Icon::FileOpen => {
            pen.poly(&[(2.0, 5.0), (9.0, 5.0), (11.0, 7.5), (20.0, 7.5), (20.0, 19.0), (2.0, 19.0)], SHADE);
            pen.poly(&[(5.5, 10.5), (22.5, 10.5), (19.5, 19.0), (2.0, 19.0)], YELLOW);
        }
        Icon::FileSave => floppy(pen, (0.0, 0.0), 1.0),
        Icon::FileSaveAs => {
            floppy(pen, (-0.5, -0.5), 0.82);
            pen.line_c(&[(14.5, 19.5), (21.0, 13.0)], Stroke::new(3.6 * pen.s, ORANGE));
            pen.poly_c(&[(11.8, 22.2), (13.0, 18.3), (15.7, 21.0)], pen.ink, Stroke::NONE);
        }
        Icon::Picture(v) => {
            let (o, w) = if v == 0 { ((3.0, 5.0), 18.0) } else { ((2.0, 3.0), 17.0) };
            let (a, b) = (o, (o.0 + w, o.1 + w * 0.75));
            rect(pen, a, b, BLUE_SOFT, Stroke::NONE);
            pen.poly_c(&[(a.0, b.1), (a.0 + w * 0.33, a.1 + w * 0.3), (a.0 + w * 0.66, b.1)], GREEN, Stroke::NONE);
            pen.poly_c(&[(a.0 + w * 0.45, b.1), (a.0 + w * 0.72, a.1 + w * 0.45), (b.0, b.1)], Color32::from_rgb(0x2E, 0x84, 0x4A), Stroke::NONE);
            pen.dot((a.0 + w * 0.72, a.1 + w * 0.22), 1.8 * w / 18.0, YELLOW);
            rect(pen, a, b, Color32::TRANSPARENT, pen.stroke());
            match v {
                1 => {
                    // a folder: the image comes from a file
                    pen.poly_c(&[(13.0, 14.5), (16.5, 14.5), (17.5, 15.8), (22.5, 15.8), (22.5, 22.0), (13.0, 22.0)], YELLOW, pen.thin());
                }
                2 => {
                    // a camera: the image comes from the view
                    pen.poly_c(&[(15.5, 14.0), (19.5, 14.0), (20.5, 15.5), (15.0, 15.5)], pen.ink, Stroke::NONE);
                    rect(pen, (12.5, 15.5), (22.5, 22.0), pen.ink, Stroke::NONE);
                    pen.painter.circle(pen.p(17.5, 18.7), 2.3 * pen.s, BLUE_SOFT, Stroke::new(W_THIN * pen.s, Color32::WHITE));
                }
                3 => red_x(pen, (18.0, 18.0), 4.8),
                _ => {}
            }
        }
        Icon::Import(t) => file_io(pen, t, true),
        Icon::Export(t) => file_io(pen, t, false),
        Icon::Film => {
            rect(pen, (2.0, 5.0), (22.0, 19.0), pen.ink, Stroke::NONE);
            for x in [4.5, 8.5, 12.5, 16.5, 20.5] {
                rect(pen, (x - 1.0, 6.2), (x + 0.6, 7.9), FILL, Stroke::NONE);
                rect(pen, (x - 1.0, 16.1), (x + 0.6, 17.8), FILL, Stroke::NONE);
            }
            rect(pen, (3.5, 9.3), (11.3, 14.7), BLUE_SOFT, Stroke::NONE);
            rect(pen, (12.7, 9.3), (20.5, 14.7), BLUE_SOFT, Stroke::NONE);
        }
        Icon::Drawing => {
            rect(pen, (3.0, 2.0), (21.0, 22.0), FILL, pen.stroke());
            pen.line_c(&[(7.0, 15.0), (7.0, 10.0), (12.0, 6.0), (17.0, 10.0), (17.0, 15.0), (7.0, 15.0)], Stroke::new(W_THIN * pen.s, BLUE));
            let st = Stroke::new(W_THIN * pen.s, RED);
            pen.line_c(&[(7.0, 18.5), (17.0, 18.5)], st);
            pen.line_c(&[(7.0, 17.0), (7.0, 20.0)], st);
            pen.line_c(&[(17.0, 17.0), (17.0, 20.0)], st);
        }
        Icon::Sheets => {
            rect(pen, (7.5, 2.0), (21.5, 17.0), SHADE, pen.stroke());
            rect(pen, (2.5, 6.0), (16.5, 22.0), FILL, pen.stroke());
            rect(pen, (4.5, 8.0), (14.5, 20.0), Color32::TRANSPARENT, pen.thin());
            rect(pen, (9.5, 17.0), (14.5, 20.0), BLUE_SOFT, pen.thin());
        }
        Icon::Report => {
            rect(pen, (3.0, 4.0), (21.0, 20.0), FILL, pen.stroke());
            rect(pen, (3.0, 4.0), (21.0, 8.5), SHADE, pen.stroke());
            for y in [12.3, 16.2] {
                pen.line_c(&[(3.0, y), (21.0, y)], pen.thin());
            }
            pen.line_c(&[(10.0, 4.0), (10.0, 20.0)], pen.thin());
            pen.line_c(&[(12.5, 10.4), (18.5, 10.4)], Stroke::new(W * pen.s, GREEN));
        }
        Icon::Json => {
            let l = [(9.0, 3.0), (7.2, 3.3), (6.6, 5.0), (6.6, 10.0), (4.5, 12.0), (6.6, 14.0), (6.6, 19.0), (7.2, 20.7), (9.0, 21.0)];
            pen.line(&l);
            pen.line(&l.map(|(x, y)| (24.0 - x, y)));
            for x in [9.5, 12.0, 14.5] {
                pen.dot((x, 12.0), 1.2, BLUE);
            }
        }
        Icon::Credits => {
            pen.circle((9.5, 7.5), 3.6, SKIN);
            let pts = pen.ell((9.5, 20.0), 7.0, 6.5, 180.0, 360.0);
            pen.poly(&pts, BLUE_SOFT);
            pen.circle((18.0, 17.0), 4.5, YELLOW);
            pen.arc((18.0, 17.0), 2.1, 45.0, 315.0, pen.thin());
        }
        Icon::Script => {
            page(pen, (4.0, 2.0), (20.0, 22.0), 5.0);
            pen.poly_c(&[(9.0, 9.0), (9.0, 18.0), (16.5, 13.5)], GREEN, Stroke::NONE);
        }
        Icon::Quit => {
            let st = Stroke::new(W_BOLD * pen.s, RED);
            pen.arc((12.0, 13.0), 7.5, -55.0, 235.0, st);
            pen.line_c(&[(12.0, 3.0), (12.0, 12.0)], st);
        }
        Icon::Cut => {
            pen.line(&[(9.0, 14.5), (16.5, 2.5)]);
            pen.line(&[(15.0, 14.5), (7.5, 2.5)]);
            let ring = Stroke::new(W * pen.s, RED);
            pen.painter.circle(pen.p(7.0, 18.0), 3.3 * pen.s, FILL, ring);
            pen.painter.circle(pen.p(17.0, 18.0), 3.3 * pen.s, FILL, ring);
        }
        Icon::Copy => {
            rect(pen, (8.5, 2.5), (20.5, 16.0), FILL, pen.stroke());
            rect(pen, (3.5, 8.0), (15.5, 21.5), FILL, pen.stroke());
            for (y, x1) in [(12.5, 12.5), (15.5, 12.5), (18.5, 10.0)] {
                pen.line_c(&[(6.5, y), (x1, y)], pen.thin());
            }
        }
        Icon::Paste | Icon::PasteInPlace => {
            rect(pen, (4.0, 4.0), (20.0, 22.0), Color32::from_rgb(0xC8, 0x9A, 0x5E), pen.stroke());
            rect(pen, (7.0, 8.0), (17.0, 20.0), FILL, pen.thin());
            rect(pen, (8.5, 2.5), (15.5, 6.0), SHADE_DARK, pen.stroke());
            if icon == Icon::PasteInPlace {
                let st = Stroke::new(W_THIN * pen.s, RED);
                pen.painter.circle_stroke(pen.p(12.0, 14.0), 3.0 * pen.s, st);
                pen.line_c(&[(12.0, 9.5), (12.0, 18.5)], st);
                pen.line_c(&[(7.5, 14.0), (16.5, 14.0)], st);
            } else {
                for y in [11.5, 14.5, 17.5] {
                    pen.line_c(&[(9.0, y), (15.0, y)], pen.thin());
                }
            }
        }
        Icon::Purge => {
            pen.line_c(&[(20.0, 2.0), (13.0, 11.5)], Stroke::new(W_BOLD * pen.s, Color32::from_rgb(0x9A, 0x6B, 0x3C)));
            pen.poly(&[(10.0, 10.0), (16.0, 13.5), (12.5, 21.5), (3.5, 17.0)], YELLOW);
            pen.line_c(&[(10.5, 14.0), (8.0, 19.5)], pen.thin());
            pen.line_c(&[(13.0, 15.0), (11.0, 20.5)], pen.thin());
            pen.dot((20.0, 18.0), 1.2, BLUE);
            pen.dot((18.0, 21.5), 0.9, BLUE);
        }
        Icon::DeleteGuides => {
            pen.dashed((2.5, 20.5), (21.5, 3.5), pen.stroke());
            pen.dot((6.5, 16.9), 2.2, RED);
            pen.dot((14.5, 9.8), 2.2, RED);
            red_x(pen, (17.5, 17.5), 4.8);
        }
        Icon::SelectAll | Icon::SelectNone => {
            let all = icon == Icon::SelectAll;
            let col = if all { BLUE } else { SHADE_DARK };
            dashed_rect(pen, (3.0, 3.0), (21.0, 21.0), Stroke::new(W_THIN * pen.s, if all { BLUE } else { pen.ink }));
            let fill = if all { BLUE_SOFT } else { Color32::TRANSPARENT };
            rect(pen, (6.5, 6.5), (12.5, 12.5), fill, Stroke::new(W_THIN * pen.s, col));
            pen.painter.circle(pen.p(15.5, 15.5), 3.2 * pen.s, fill, Stroke::new(W_THIN * pen.s, col));
        }
        Icon::Component => {
            rounded(pen, (2.5, 2.5), (21.5, 21.5), Stroke::new(W_THIN * pen.s, PURPLE));
            mini_cube(pen, (12.0, 12.0), 0.62);
        }
        Icon::Parametric => {
            mini_cube(pen, (12.0, 9.0), 0.55);
            pen.line_c(&[(3.5, 20.0), (20.5, 20.0)], pen.stroke());
            pen.painter.circle(pen.p(14.5, 20.0), 2.5 * pen.s, BLUE, pen.thin());
        }
        Icon::Explode => {
            mini_cube(pen, (12.0, 12.0), 0.45);
            let st = Stroke::new(W * pen.s, BLUE);
            for (dx, dy) in [(1.0, -1.0), (1.0, 1.0), (-1.0, 1.0), (-1.0, -1.0)] {
                pen.line_c(&[(12.0 + 5.5 * dx, 12.0 + 5.5 * dy), (12.0 + 8.0 * dx, 12.0 + 8.0 * dy)], st);
                pen.head((12.0 + 10.5 * dx, 12.0 + 10.5 * dy), (dx, dy), 3.4, BLUE);
            }
        }
        Icon::CloseGroup => {
            rounded(pen, (2.5, 4.0), (14.5, 20.0), Stroke::new(W_THIN * pen.s, BLUE));
            rect(pen, (5.0, 8.5), (10.5, 15.5), BLUE_SOFT, pen.thin());
            pen.line(&[(9.0, 12.0), (19.5, 12.0)]);
            pen.head((22.5, 12.0), (1.0, 0.0), 4.0, pen.ink);
        }
        Icon::Intersect(v) => {
            pen.poly(&[(2.0, 15.0), (12.0, 10.5), (22.0, 14.5), (12.0, 19.5)], FILL);
            pen.poly(&[(9.0, 3.5), (15.0, 6.0), (15.0, 19.0), (9.0, 16.5)], BLUE_SOFT);
            pen.line_c(&[(9.0, 13.0), (15.0, 15.5)], Stroke::new(W_BOLD * pen.s, RED));
            match v {
                1 => dashed_rect(pen, (0.8, 1.0), (23.2, 23.0), Stroke::new(W_THIN * pen.s, BLUE)),
                2 => rounded(pen, (0.8, 1.0), (23.2, 23.0), Stroke::new(W_THIN * pen.s, PURPLE)),
                _ => {}
            }
        }
        Icon::ReverseFaces => {
            pen.poly(&[(4.0, 4.0), (20.0, 4.0), (4.0, 20.0)], FILL);
            pen.poly(&[(20.0, 4.0), (20.0, 20.0), (4.0, 20.0)], BACK);
            pen.line(&[(9.5, 14.5), (14.5, 9.5)]);
            pen.head((17.0, 7.0), (1.0, -1.0), 4.0, pen.ink);
            pen.head((7.0, 17.0), (-1.0, 1.0), 4.0, pen.ink);
        }
        Icon::OrientFaces => {
            rect(pen, (2.5, 12.0), (10.5, 21.0), FILL, pen.stroke());
            rect(pen, (13.5, 12.0), (21.5, 21.0), FILL, pen.stroke());
            let st = Stroke::new(W * pen.s, BLUE);
            for x in [6.5, 17.5] {
                pen.line_c(&[(x, 16.5), (x, 6.0)], st);
                pen.head((x, 3.0), (0.0, -1.0), 3.8, BLUE);
            }
        }
        Icon::Flip(axis) => {
            let c = [RED, GREEN, BLUE][axis.min(2) as usize];
            pen.dashed((12.0, 2.0), (12.0, 22.0), Stroke::new(W * pen.s, c));
            pen.poly(&[(3.0, 16.0), (10.0, 16.0), (10.0, 5.0)], SHADE);
            pen.closed(&[(21.0, 16.0), (14.0, 16.0), (14.0, 5.0)]);
            let st = Stroke::new(W * pen.s, c);
            pen.line_c(&[(6.5, 20.0), (17.5, 20.0)], st);
            pen.head((20.5, 20.0), (1.0, 0.0), 3.6, c);
            pen.head((3.5, 20.0), (-1.0, 0.0), 3.6, c);
        }
        Icon::Solids(op) => solid(pen, op),
        Icon::Unfold => {
            let cells = [(9.5, 2.0), (9.5, 7.0), (9.5, 12.0), (9.5, 17.0), (4.5, 7.0), (14.5, 7.0)];
            for (i, (x, y)) in cells.into_iter().enumerate() {
                let fill = if i == 1 { BLUE_SOFT } else { FILL };
                rect(pen, (x, y), (x + 5.0, y + 5.0), fill, pen.thin());
            }
        }
        Icon::CutList => {
            // a sheet with parts laid out on it and the saw's cut line
            rect(pen, (2.5, 5.0), (21.5, 19.0), FILL, pen.stroke());
            rect(pen, (2.5, 5.0), (11.0, 13.0), WOOD, pen.thin());
            rect(pen, (11.8, 5.0), (21.5, 10.0), WOOD, pen.thin());
            rect(pen, (2.5, 13.8), (8.0, 19.0), WOOD, pen.thin());
            pen.line_c(&[(11.4, 2.5), (11.4, 21.5)], Stroke::new(W * pen.s, ORANGE));
        }
        Icon::Assembly => {
            // a cabinet taken apart: two sides out, the top lifted, a dashed line of travel
            rect(pen, (2.5, 8.0), (6.5, 21.5), WOOD, pen.stroke());
            rect(pen, (17.5, 8.0), (21.5, 21.5), WOOD, pen.stroke());
            rect(pen, (5.0, 2.5), (19.0, 5.5), WOOD, pen.stroke());
            rect(pen, (9.0, 13.0), (15.0, 15.5), WOOD, pen.stroke());
            pen.line_c(&[(12.0, 6.5), (12.0, 11.5)], Stroke::new(W * pen.s, ORANGE));
        }
        Icon::Joinery => {
            // a side standing up, a shelf meeting it, the dowels across the joint
            rect(pen, (3.0, 2.5), (9.0, 21.5), WOOD, pen.stroke());
            rect(pen, (9.0, 10.0), (21.5, 15.0), WOOD, pen.stroke());
            for y in [11.4, 13.6] {
                pen.line_c(&[(5.5, y), (13.5, y)], Stroke::new(W * pen.s, ORANGE));
            }
        }
        Icon::UvEditor => {
            // a checkered tile with an island laid on it: four corners joined by the accent
            rect(pen, (3.0, 3.0), (21.0, 21.0), FILL, pen.thin());
            rect(pen, (3.0, 3.0), (12.0, 12.0), BLUE_SOFT, pen.thin());
            rect(pen, (12.0, 12.0), (21.0, 21.0), BLUE_SOFT, pen.thin());
            let island = [(6.0, 17.5), (9.5, 6.0), (18.0, 8.5), (16.0, 18.0)];
            pen.path(pen.pts(&island), true, Stroke::new(W * pen.s, ORANGE));
            for p in island {
                pen.dot(p, 1.5, pen.ink);
            }
        }
        Icon::TexturePosition => {
            // a face in perspective with brick courses, the red pin and the green turn handle
            let face = [(2.5, 9.0), (15.0, 4.0), (21.5, 12.0), (9.0, 19.5)];
            pen.poly_c(&face, Color32::from_rgb(0xE9, 0xB8, 0x8F), Stroke::NONE);
            let lerp = |a: (f32, f32), b: (f32, f32), t: f32| (a.0 + (b.0 - a.0) * t, a.1 + (b.1 - a.1) * t);
            let mortar = Stroke::new(W_THIN * pen.s, Color32::from_rgb(0xA8, 0x5A, 0x3A));
            for (i, t) in [0.25f32, 0.5, 0.75].into_iter().enumerate() {
                let (a, b) = (lerp(face[0], face[1], t), lerp(face[3], face[2], t));
                pen.line_c(&[a, b], mortar);
                // head joints, staggered course by course
                for u in [0.3f32, 0.7] {
                    let u = if i % 2 == 0 { u } else { u - 0.2 };
                    let (c0, c1) = (lerp(face[0], face[1], t - 0.25), lerp(face[3], face[2], t - 0.25));
                    let p0 = lerp(c0, c1, u);
                    let p1 = lerp(a, b, u);
                    pen.line_c(&[p0, p1], mortar);
                }
            }
            pen.path(pen.pts(&face), true, pen.thin());
            pen.line_c(&[(7.0, 11.5), (15.5, 7.0)], Stroke::new(W * pen.s, pal::GREEN));
            pen.circle((15.5, 7.0), 2.1, pal::GREEN);
            pen.circle((7.0, 11.5), 2.8, pal::RED);
        }
        Icon::Physics => {
            // a box falling onto a block, speed lines above it, tilted as it comes down
            rect(pen, (3.0, 16.0), (21.0, 21.0), SHADE, pen.stroke());
            pen.poly_c(&[(9.5, 5.5), (16.5, 7.0), (15.0, 14.0), (8.0, 12.5)], BLUE_SOFT, pen.stroke());
            for (x, y0, y1) in [(9.0, 1.5, 3.5), (12.5, 1.0, 4.0), (16.0, 2.0, 4.0)] {
                pen.line_c(&[(x, y0), (x, y1)], Stroke::new(W_THIN * pen.s, ORANGE));
            }
        }
        Icon::Print3d => {
            // the printer's frame and bed, the hot end on its gantry, a part growing layer by layer
            rect(pen, (2.5, 19.0), (21.5, 21.5), SHADE, pen.thin());
            pen.line_c(&[(4.0, 19.0), (4.0, 3.0), (20.0, 3.0), (20.0, 19.0)], pen.stroke());
            pen.line_c(&[(4.0, 7.0), (20.0, 7.0)], pen.thin());
            rect(pen, (9.5, 5.0), (14.5, 9.5), FILL, pen.thin());
            pen.poly_c(&[(10.5, 9.5), (13.5, 9.5), (12.0, 12.0)], ORANGE, pen.thin());
            rect(pen, (7.5, 13.5), (16.5, 19.0), BLUE_SOFT, pen.thin());
            for y in [15.3, 17.1] {
                pen.line_c(&[(7.5, y), (16.5, y)], pen.thin());
            }
            pen.line_c(&[(12.0, 12.5), (12.0, 13.5)], Stroke::new(W_BOLD * pen.s, ORANGE));
        }
        Icon::StartScreen => {
            // a window (its title bar) with a house in it: home
            rect(pen, (2.5, 3.5), (21.5, 20.5), FILL, pen.stroke());
            pen.line_c(&[(2.5, 7.5), (21.5, 7.5)], pen.thin());
            rect(pen, (8.0, 13.5), (16.0, 18.5), FILL, pen.thin());
            pen.poly_c(&[(6.0, 14.0), (12.0, 9.5), (18.0, 14.0)], BLUE_SOFT, pen.thin());
            rect(pen, (11.0, 15.5), (13.0, 18.5), ORANGE, pen.thin());
        }
        Icon::Snap(k) => snap(pen, k),
        Icon::SideBar => {
            rect(pen, (2.5, 4.0), (21.5, 20.0), FILL, pen.stroke());
            rect(pen, (14.5, 4.0), (21.5, 20.0), BLUE_SOFT, pen.stroke());
            for y in [8.0, 11.5, 15.0] {
                pen.line_c(&[(16.0, y), (20.0, y)], pen.thin());
            }
        }
        Icon::Outliner => {
            let st = pen.thin();
            pen.line_c(&[(6.0, 7.0), (6.0, 18.0)], st);
            pen.line_c(&[(6.0, 12.5), (11.0, 12.5)], st);
            pen.line_c(&[(6.0, 18.0), (11.0, 18.0)], st);
            rect(pen, (3.0, 3.5), (9.0, 8.5), BLUE_SOFT, pen.thin());
            for (y, c) in [(12.5, SHADE), (18.0, SHADE)] {
                rect(pen, (11.0, y - 2.3), (15.0, y + 2.3), c, pen.thin());
                pen.line_c(&[(16.5, y), (21.5, y)], pen.stroke());
            }
            pen.line_c(&[(10.5, 6.0), (21.5, 6.0)], pen.stroke());
        }
        Icon::Tag => {
            pen.poly(&[(3.0, 5.0), (12.0, 5.0), (21.0, 13.5), (13.5, 21.0), (3.0, 12.0)], YELLOW);
            pen.painter.circle(pen.p(7.5, 9.5), 1.8 * pen.s, FILL, pen.thin());
        }
        Icon::Render => {
            // a lit ball on its shadow and a sparkle: a realistic picture
            pen.painter.add(egui::Shape::ellipse_filled(pen.p(11.0, 20.0), egui::vec2(7.0, 1.6) * pen.s, Color32::from_black_alpha(55)));
            pen.circle((11.0, 12.5), 6.5, BLUE_SOFT);
            pen.dot((8.8, 10.3), 1.6, Color32::WHITE);
            let st = Stroke::new(W * pen.s, ORANGE);
            pen.line_c(&[(19.0, 1.5), (19.0, 8.5)], st);
            pen.line_c(&[(15.5, 5.0), (22.5, 5.0)], st);
        }
        Icon::Console => {
            rect(pen, (2.5, 4.0), (21.5, 20.0), Color32::from_rgb(0x2B, 0x30, 0x38), pen.stroke());
            let st = Stroke::new(W * pen.s, GREEN);
            pen.line_c(&[(6.0, 9.0), (9.5, 12.0), (6.0, 15.0)], st);
            pen.line_c(&[(11.5, 15.5), (17.0, 15.5)], Stroke::new(W * pen.s, Color32::WHITE));
        }
        Icon::Log => {
            rect(pen, (3.5, 2.5), (18.5, 21.5), FILL, pen.stroke());
            for y in [7.0, 10.5, 14.0] {
                pen.line_c(&[(6.5, y), (15.5, y)], pen.thin());
            }
            pen.dot((18.0, 18.0), 4.5, RED);
            pen.line_c(&[(18.0, 15.5), (18.0, 18.6)], Stroke::new(W * pen.s, Color32::WHITE));
            pen.dot((18.0, 20.4), 0.9, Color32::WHITE);
        }
        Icon::Gear => gear(pen, (12.0, 12.0), 1.0),
        Icon::Generator => {
            mini_cube(pen, (10.0, 11.0), 0.62);
            gear(pen, (17.5, 17.5), 0.5);
        }
        Icon::Play => pen.poly_c(&[(7.0, 4.0), (7.0, 20.0), (20.0, 12.0)], GREEN, pen.thin()),
        Icon::Pause => {
            rect(pen, (6.0, 4.5), (10.5, 19.5), pen.ink, Stroke::NONE);
            rect(pen, (13.5, 4.5), (18.0, 19.5), pen.ink, Stroke::NONE);
        }
        Icon::NextScene | Icon::PrevScene => {
            let f = |x: f32| if icon == Icon::PrevScene { 24.0 - x } else { x };
            pen.poly_c(&[(f(5.0), 5.0), (f(5.0), 19.0), (f(16.0), 12.0)], BLUE, Stroke::NONE);
            rect(pen, (f(16.5).min(f(19.5)), 5.0), (f(16.5).max(f(19.5)), 19.0), BLUE, Stroke::NONE);
        }
        Icon::AddScene => {
            rect(pen, (2.5, 3.5), (18.5, 15.5), BLUE_SOFT, pen.stroke());
            mini_cube(pen, (10.5, 9.5), 0.42);
            pen.dot((17.5, 17.0), 5.0, GREEN);
            let st = Stroke::new(W * pen.s, Color32::WHITE);
            pen.line_c(&[(17.5, 14.0), (17.5, 20.0)], st);
            pen.line_c(&[(14.5, 17.0), (20.5, 17.0)], st);
        }
        _ => {}
    }
}

/// A cog wheel of radius ~9·`k` at `c`.
fn gear(pen: &Pen, c: (f32, f32), k: f32) {
    let teeth = Stroke::new(3.2 * k * pen.s, pen.ink);
    for i in 0..8 {
        let a = i as f32 * PI / 4.0;
        pen.line_c(&[(c.0 + 5.5 * k * a.cos(), c.1 + 5.5 * k * a.sin()), (c.0 + 9.5 * k * a.cos(), c.1 + 9.5 * k * a.sin())], teeth);
    }
    pen.painter.circle(pen.p(c.0, c.1), 6.8 * k * pen.s, pal::shade(), Stroke::new(W * k * pen.s, pen.ink));
    pen.painter.circle(pen.p(c.0, c.1), 2.6 * k * pen.s, pal::fill(), Stroke::new(W_THIN * k * pen.s, pen.ink));
}

/// Two overlapping boxes (the first one bottom-left), what each solid tool keeps.
/// `op` indexes `SolidOp::ALL`: Outer Shell, Intersect, Union, Subtract, Trim, Split.
fn solid(pen: &Pen, op: u8) {
    let (a0, a1, b0, b1, o0, o1) = ((3.0, 9.0), (15.0, 21.0), (9.0, 3.0), (21.0, 15.0), (9.0, 9.0), (15.0, 15.0));
    let none = Stroke::NONE;
    let thin = pen.thin();
    let union_outline = [(3.0, 9.0), (9.0, 9.0), (9.0, 3.0), (21.0, 3.0), (21.0, 15.0), (15.0, 15.0), (15.0, 21.0), (3.0, 21.0)];
    // the first box without the shared part (an L)
    let a_rest = |c: Color32| {
        rect(pen, (3.0, 15.0), (15.0, 21.0), c, none);
        rect(pen, (3.0, 9.0), (9.0, 15.0), c, none);
    };
    let b_rest = |c: Color32| {
        rect(pen, (9.0, 3.0), (21.0, 9.0), c, none);
        rect(pen, (15.0, 9.0), (21.0, 15.0), c, none);
    };
    let dash = Stroke::new(W_THIN * pen.s, pal::shade_dark());
    match op {
        0 => {
            rect(pen, a0, a1, pal::shade(), none);
            rect(pen, b0, b1, pal::shade(), none);
            pen.path(pen.pts(&union_outline), true, Stroke::new(W_BOLD * pen.s, pen.ink));
        }
        1 => {
            rect(pen, o0, o1, pal::BLUE, none);
            rect(pen, a0, a1, Color32::TRANSPARENT, dash);
            rect(pen, b0, b1, Color32::TRANSPARENT, dash);
            rect(pen, o0, o1, Color32::TRANSPARENT, pen.stroke());
        }
        2 => {
            rect(pen, a0, a1, pal::BLUE_SOFT, none);
            rect(pen, b0, b1, pal::BLUE_SOFT, none);
            pen.closed(&union_outline);
            pen.dashed((9.0, 9.0), (9.0, 15.0), dash);
            pen.dashed((9.0, 15.0), (15.0, 15.0), dash);
        }
        3 | 4 => {
            if op == 4 {
                rect(pen, b0, b1, pal::shade(), thin);
            } else {
                rect(pen, b0, b1, Color32::TRANSPARENT, dash);
            }
            a_rest(pal::BLUE_SOFT);
            pen.closed(&[(3.0, 9.0), (9.0, 9.0), (9.0, 15.0), (15.0, 15.0), (15.0, 21.0), (3.0, 21.0)]);
        }
        _ => {
            a_rest(pal::BLUE_SOFT);
            b_rest(pal::shade());
            rect(pen, o0, o1, pal::BLUE, none);
            rect(pen, a0, a1, Color32::TRANSPARENT, thin);
            rect(pen, b0, b1, Color32::TRANSPARENT, thin);
        }
    }
}

/// Each inference: a little scene and its marker in the colours the view uses.
fn snap(pen: &Pen, k: InferenceKind) {
    use pal::*;
    use InferenceKind as K;
    #[allow(non_snake_case)]
    let FILL = pal::fill();
    let thin = pen.thin();
    let ring = |c: (f32, f32), col: Color32| pen.painter.circle(pen.p(c.0, c.1), 3.0 * pen.s, col, Stroke::new(W_THIN * pen.s, pen.ink));
    let grid = |hi: bool| {
        let st = Stroke::new(W_THIN * pen.s, pal::shade_dark());
        for v in [4.0, 10.0, 16.0, 22.0] {
            pen.line_c(&[(v - 2.0, 2.0), (v - 2.0, 22.0)], st);
            pen.line_c(&[(2.0, v - 2.0), (22.0, v - 2.0)], st);
        }
        if hi {
            pen.line_c(&[(2.0, 14.0), (22.0, 14.0)], Stroke::new(W_BOLD * pen.s, ORANGE));
        }
    };
    match k {
        K::Endpoint => {
            pen.line(&[(4.0, 20.0), (17.0, 7.0)]);
            ring((17.0, 7.0), GREEN);
        }
        K::Midpoint => {
            pen.line(&[(3.0, 21.0), (21.0, 3.0)]);
            ring((12.0, 12.0), CYAN);
        }
        K::Center => {
            pen.circle((12.0, 12.0), 8.0, Color32::TRANSPARENT);
            ring((12.0, 12.0), BLUE);
        }
        K::Intersection => {
            pen.line(&[(3.0, 20.0), (21.0, 4.0)]);
            pen.line(&[(3.0, 4.0), (21.0, 20.0)]);
            let st = Stroke::new(W_BOLD * pen.s, RED);
            pen.line_c(&[(9.5, 9.0), (14.5, 15.0)], st);
            pen.line_c(&[(9.5, 15.0), (14.5, 9.0)], st);
        }
        K::Origin => {
            let o = (8.0, 16.0);
            pen.line_c(&[o, (22.0, 16.0)], Stroke::new(W * pen.s, RED));
            pen.line_c(&[o, (16.0, 9.0)], Stroke::new(W * pen.s, GREEN));
            pen.line_c(&[o, (8.0, 2.0)], Stroke::new(W * pen.s, BLUE));
            ring(o, YELLOW);
        }
        K::OnEdge => {
            pen.line(&[(3.0, 18.0), (21.0, 8.0)]);
            rect(pen, (9.8, 10.8), (14.2, 15.2), RED, Stroke::new(W_THIN * pen.s, pen.ink));
        }
        K::OnFace => {
            pen.poly(&[(2.0, 14.0), (12.0, 7.0), (22.0, 11.0), (12.0, 19.0)], FILL);
            pen.poly_c(&[(12.0, 9.5), (15.0, 12.8), (12.0, 16.0), (9.0, 12.8)], BLUE, Stroke::new(W_THIN * pen.s, pen.ink));
        }
        K::OnGuide => {
            pen.dashed((2.5, 20.5), (21.5, 3.5), pen.stroke());
            ring((12.0, 12.0), pen.ink);
        }
        K::GuidePoint => {
            pen.dashed((2.5, 20.5), (14.0, 10.2), pen.stroke());
            let st = Stroke::new(W * pen.s, pen.ink);
            pen.line_c(&[(17.0, 3.0), (17.0, 13.0)], st);
            pen.line_c(&[(12.0, 8.0), (22.0, 8.0)], st);
        }
        K::OnRedAxis | K::OnGreenAxis | K::OnBlueAxis => {
            let c = match k {
                K::OnRedAxis => RED,
                K::OnGreenAxis => GREEN,
                _ => BLUE,
            };
            pen.dashed((3.0, 20.0), (21.0, 4.0), Stroke::new(W_BOLD * pen.s, c));
            ring((15.0, 9.3), c);
        }
        K::ParallelToEdge => {
            pen.line(&[(3.0, 15.0), (14.0, 4.0)]);
            pen.line_c(&[(10.0, 21.0), (21.0, 10.0)], Stroke::new(W_BOLD * pen.s, MAGENTA));
        }
        K::PerpendicularToEdge => {
            pen.line(&[(3.0, 19.0), (21.0, 19.0)]);
            pen.line_c(&[(10.0, 19.0), (10.0, 3.0)], Stroke::new(W_BOLD * pen.s, MAGENTA));
            pen.line_c(&[(10.0, 15.0), (14.0, 15.0), (14.0, 19.0)], thin);
        }
        K::FromPoint => {
            ring((5.0, 19.0), pen.ink);
            pen.dashed((7.5, 16.5), (19.0, 5.0), Stroke::new(W * pen.s, MAGENTA));
            pen.dot((19.5, 4.5), 1.8, MAGENTA);
        }
        K::Tangent => {
            pen.circle((11.0, 14.0), 7.0, Color32::TRANSPARENT);
            pen.line_c(&[(2.0, 7.0), (22.0, 7.0)], Stroke::new(W_BOLD * pen.s, MAGENTA));
            pen.dot((11.0, 7.0), 2.0, pen.ink);
        }
        K::Square => {
            rect(pen, (4.0, 4.0), (20.0, 20.0), FILL, pen.stroke());
            pen.dashed((4.0, 20.0), (20.0, 4.0), Stroke::new(W * pen.s, MAGENTA));
        }
        K::GoldenSection => {
            rect(pen, (2.5, 6.5), (21.5, 18.2), FILL, pen.stroke());
            pen.line_c(&[(14.2, 6.5), (14.2, 18.2)], thin);
            pen.arc((14.2, 18.2), 11.7, 180.0, 270.0, Stroke::new(W * pen.s, MAGENTA));
        }
        K::HalfCircle => {
            pen.arc((12.0, 16.0), 9.0, 180.0, 360.0, pen.stroke());
            pen.line_c(&[(3.0, 16.0), (21.0, 16.0)], Stroke::new(W * pen.s, MAGENTA));
            pen.dot((12.0, 16.0), 1.6, MAGENTA);
        }
        K::GridPoint => {
            grid(false);
            ring((14.0, 14.0), ORANGE);
        }
        K::OnGridLine => grid(true),
        K::Free => pen.dot((12.0, 12.0), 2.0, pen.ink),
    }
}
