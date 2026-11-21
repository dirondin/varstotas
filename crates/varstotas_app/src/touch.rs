//! Fingers on the view: one runs the tool (the point goes where the finger is let go, so the
//! finger does not hide it, with a loupe above it meanwhile), two navigate (moved together: pan,
//! pinched: zoom, turned: turn the model about the vertical), three orbit.
//!
//! The browser and the touch screens turn the first finger into the left mouse button as
//! well; the other fingers come only as [`egui::Event::Touch`].

use std::collections::BTreeMap;

use egui::{Color32, Event, Id, LayerId, Order, Pos2, Rect, Stroke, TouchPhase, Vec2, pos2, vec2};
use varstotas_kernel::glam::DVec2;

use crate::overlay::{Marker2d, paint_marker, tooltip};

/// The loupe: its radius, how much it magnifies and how far above the finger it sits.
const LOUPE_R: f32 = 56.0;
const LOUPE_ZOOM: f32 = 2.0;
const LOUPE_LIFT: f32 = 96.0;
/// The loupe waits this long (seconds) for a finger held alone: fingers seldom land together,
/// and the first of a two- or three-finger gesture is no finger for the tool.
const LOUPE_DELAY: f64 = 0.25;

/// A twist smaller than this (radians, since the fingers met) is a pinch's wobble, not a turn.
const TWIST_START: f32 = 0.2;

#[derive(Default)]
pub struct Touch {
    /// The fingers down, by id, at their screen points.
    fingers: BTreeMap<u64, Pos2>,
    /// A finger touched the screen since the start (a touch screen without `pointer: coarse`).
    seen: bool,
    /// Touches that are the pen's (a browser reports the pen as a touch too).
    pen_touches: std::collections::BTreeSet<u64>,
    /// Two or more fingers were down since the first one went down: its press is no click.
    gestured: bool,
    /// A finger's press kept for its release (view-local point where it went down).
    pub pending: Option<Pos2>,
    /// When `pending` was pressed (seconds, egui's input time).
    pub pending_at: f64,
    /// The points a finger's taps gave the tool so far, while it is busy with them (a rectangle's
    /// first corner, a line's points…): shown, as the finger hides where it tapped.
    pub marks: Vec<varstotas_kernel::glam::DVec3>,
    /// The last frame's fingers: how many, their middle, spread and turn.
    shape: Option<(usize, Pos2, f32, f32)>,
    /// The turn since the fingers met, and whether it got past [`TWIST_START`].
    twist: (f32, bool),
}

/// What two or more fingers did in a frame.
pub struct Gesture {
    pub fingers: usize,
    /// Their middle (screen points).
    pub center: Pos2,
    /// The fingers came or went: nothing moved yet.
    pub started: bool,
    pub pan: Vec2,
    /// Spread now / before.
    pub zoom: f32,
    /// Radians, clockwise on the screen.
    pub twist: f32,
}

impl Touch {
    /// Takes a frame's events, `pen` being where the pen touches the screen; the navigation
    /// asked for by two or more fingers.
    pub fn update(&mut self, events: &[Event], pen: Option<Pos2>) -> Option<Gesture> {
        if self.fingers.is_empty() {
            self.gestured = false;
        }
        for e in events {
            if let Event::Touch { id, phase, pos, .. } = e {
                // the pen's own touch is no finger
                if *phase == TouchPhase::Start && pen.is_some_and(|p| p.distance(*pos) < 6.0) {
                    self.pen_touches.insert(id.0);
                }
                if self.pen_touches.contains(&id.0) {
                    if matches!(phase, TouchPhase::End | TouchPhase::Cancel) {
                        self.pen_touches.remove(&id.0);
                    }
                    continue;
                }
                self.seen = true;
                match phase {
                    TouchPhase::Start | TouchPhase::Move => {
                        self.fingers.insert(id.0, *pos);
                    }
                    TouchPhase::End | TouchPhase::Cancel => {
                        self.fingers.remove(&id.0);
                    }
                }
            }
        }
        let n = self.fingers.len();
        if n < 2 {
            self.shape = None;
            return None;
        }
        self.gestured = true;
        self.pending = None;
        let pts: Vec<Pos2> = self.fingers.values().copied().collect();
        let center = (pts.iter().fold(Vec2::ZERO, |s, p| s + p.to_vec2()) / n as f32).to_pos2();
        let spread = pts.iter().map(|p| p.distance(center)).sum::<f32>() / n as f32;
        let d = pts[1] - pts[0];
        let angle = d.y.atan2(d.x);
        let g = match self.shape {
            Some((m, c0, s0, a0)) if m == n => {
                let mut turn = angle - a0;
                if turn > std::f32::consts::PI {
                    turn -= std::f32::consts::TAU;
                } else if turn < -std::f32::consts::PI {
                    turn += std::f32::consts::TAU;
                }
                self.twist.0 += turn;
                self.twist.1 |= self.twist.0.abs() > TWIST_START;
                let zoom = if s0 > 1.0 && spread > 1.0 { spread / s0 } else { 1.0 };
                Gesture { fingers: n, center, started: false, pan: center - c0, zoom, twist: if self.twist.1 { turn } else { 0.0 } }
            }
            _ => {
                self.twist = (0.0, false);
                Gesture { fingers: n, center, started: true, pan: Vec2::ZERO, zoom: 1.0, twist: 0.0 }
            }
        };
        self.shape = Some((n, center, spread, angle));
        Some(g)
    }

    /// The fingers' press is not the tool's (the pen came: a palm on the screen).
    pub fn call_off(&mut self) {
        self.pending = None;
        self.gestured = true;
    }

    /// A finger has touched the screen since the start.
    pub fn seen(&self) -> bool {
        self.seen
    }

    /// A finger is down.
    pub fn active(&self) -> bool {
        !self.fingers.is_empty()
    }

    /// More than one finger went down since the first: the press is navigation's.
    pub fn gestured(&self) -> bool {
        self.gestured
    }

    /// Where the loupe looks (view-local): a finger held alone for the tool for [`LOUPE_DELAY`]
    /// by `now`; otherwise how long until it may show, if it may.
    pub fn loupe(&self, pointer: Option<Pos2>, now: f64) -> Result<Pos2, Option<f64>> {
        let Some(at) = self.pending.and(pointer).filter(|_| !self.gestured) else { return Err(None) };
        let wait = self.pending_at + LOUPE_DELAY - now;
        if wait > 0.0 { Err(Some(wait)) } else { Ok(at) }
    }
}

impl crate::app::VarstotasApp {
    /// The loupe over a finger held for the tool in pane `pane` (`rect`): the scene under the
    /// finger magnified, with the tool's point markers and its tooltip.
    pub(crate) fn paint_loupe(&self, ctx: &egui::Context, pane: usize, rect: Rect, markers: &[Marker2d]) {
        let at = match self.touch.loupe(self.last_pointer, ctx.input(|i| i.time)) {
            Ok(at) => at,
            Err(wait) => {
                if let Some(w) = wait {
                    ctx.request_repaint_after(std::time::Duration::from_secs_f64(w));
                }
                return;
            }
        };
        let Some(tex) = self.viewport.scene_texture(pane) else { return };
        let finger = rect.min + at.to_vec2();
        // above the finger; beside it near the top
        let mut c = finger - vec2(0.0, LOUPE_LIFT);
        if c.y - LOUPE_R < rect.top() {
            c = finger + vec2(if finger.x > rect.center().x { -LOUPE_LIFT } else { LOUPE_LIFT }, 0.0);
        }
        c.x = c.x.clamp(rect.left() + LOUPE_R, (rect.right() - LOUPE_R).max(rect.left() + LOUPE_R));
        c.y = c.y.max(rect.top() + LOUPE_R);
        let painter = ctx.layer_painter(LayerId::new(Order::Foreground, Id::new("touch_loupe")));
        painter.circle_filled(c + vec2(0.0, 3.0), LOUPE_R + 2.0, Color32::from_black_alpha(60));
        let uv = |p: Pos2| pos2((p.x - rect.min.x) / rect.width(), (p.y - rect.min.y) / rect.height());
        let mut mesh = egui::Mesh::with_texture(tex);
        mesh.vertices.push(egui::epaint::Vertex { pos: c, uv: uv(finger), color: Color32::WHITE });
        const N: u32 = 48;
        for i in 0..=N {
            let a = i as f32 / N as f32 * std::f32::consts::TAU;
            let d = vec2(a.cos(), a.sin());
            mesh.vertices.push(egui::epaint::Vertex { pos: c + d * LOUPE_R, uv: uv(finger + d * LOUPE_R / LOUPE_ZOOM), color: Color32::WHITE });
            if i > 0 {
                mesh.add_triangle(0, i, i + 1);
            }
        }
        painter.add(egui::Shape::mesh(mesh));
        // the snap markers near the finger, where they are in the loupe
        let vp = DVec2::new(rect.width() as f64, rect.height() as f64);
        let mut tip = None;
        for m in markers {
            match m {
                Marker2d::Point { at, shape, color } => {
                    if let Some(s) = self.camera.world_to_screen(*at, vp) {
                        let p = rect.min + vec2(s.x as f32, s.y as f32);
                        if p.distance(finger) < LOUPE_R / LOUPE_ZOOM - 4.0 {
                            paint_marker(&painter, c + (p - finger) * LOUPE_ZOOM, *shape, *color);
                        }
                    }
                }
                Marker2d::CursorTooltip { text } => tip = Some(text),
                _ => {}
            }
        }
        // the finger's point
        let gray = Color32::from_black_alpha(140);
        painter.line_segment([c - vec2(6.0, 0.0), c + vec2(6.0, 0.0)], Stroke::new(1.0, gray));
        painter.line_segment([c - vec2(0.0, 6.0), c + vec2(0.0, 6.0)], Stroke::new(1.0, gray));
        painter.circle_stroke(c, LOUPE_R, Stroke::new(2.5, Color32::WHITE));
        painter.circle_stroke(c, LOUPE_R + 1.5, Stroke::new(1.0, Color32::from_black_alpha(80)));
        if let Some(t) = tip {
            tooltip(&painter, c + vec2(-LOUPE_R, LOUPE_R + 6.0), t);
        }
    }

    /// The points tapped for the tool in pane `rect`, numbered.
    pub(crate) fn paint_tap_marks(&self, ctx: &egui::Context, rect: Rect) {
        if self.touch.marks.is_empty() {
            return;
        }
        let painter = ctx.layer_painter(LayerId::new(Order::Foreground, Id::new("touch_marks"))).with_clip_rect(rect);
        let vp = DVec2::new(rect.width() as f64, rect.height() as f64);
        let many = self.touch.marks.len() > 1;
        for (i, m) in self.touch.marks.iter().enumerate() {
            let Some(s) = self.camera.world_to_screen(*m, vp) else { continue };
            let p = rect.min + vec2(s.x as f32, s.y as f32);
            let a = crate::ui::theme::ACCENT;
            painter.circle(p, 20.0, a.gamma_multiply(0.18), Stroke::new(1.5, a.gamma_multiply(0.6)));
            painter.circle_filled(p + vec2(0.0, 1.5), 9.0, Color32::from_black_alpha(50));
            painter.circle(p, 8.0, a, Stroke::new(2.5, Color32::WHITE));
            if many {
                let at = p + vec2(0.0, -30.0);
                let g = painter.layout_no_wrap((i + 1).to_string(), egui::FontId::proportional(12.0), Color32::WHITE);
                let r = Rect::from_center_size(at, g.size() + vec2(10.0, 4.0));
                painter.rect_filled(r, 8.0, crate::ui::theme::ACCENT);
                painter.galley(r.center() - g.size() / 2.0, g, Color32::WHITE);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn touch(id: u64, phase: TouchPhase, x: f32, y: f32) -> Event {
        Event::Touch { device_id: egui::TouchDeviceId(0), id: egui::TouchId(id), phase, pos: Pos2::new(x, y), force: None }
    }

    #[test]
    fn two_fingers_pan_and_pinch() {
        let mut t = Touch::default();
        assert!(t.update(&[touch(0, TouchPhase::Start, 100.0, 100.0)], None).is_none());
        assert!(t.update(&[touch(1, TouchPhase::Start, 200.0, 100.0)], None).unwrap().started);
        let g = t.update(&[touch(0, TouchPhase::Move, 90.0, 110.0), touch(1, TouchPhase::Move, 230.0, 110.0)], None).unwrap();
        assert_eq!(g.pan, Vec2::new(10.0, 10.0));
        assert!((g.zoom - 1.4).abs() < 1e-4);
        assert_eq!(g.twist, 0.0);
        assert!(t.gestured());
        t.update(&[touch(0, TouchPhase::End, 90.0, 110.0), touch(1, TouchPhase::End, 230.0, 110.0)], None);
        assert!(!t.active());
    }

    #[test]
    fn the_loupe_waits_for_a_second_finger() {
        let mut t = Touch::default();
        t.update(&[touch(0, TouchPhase::Start, 100.0, 100.0)], None);
        t.pending = Some(Pos2::new(100.0, 100.0));
        t.pending_at = 1.0;
        let p = Some(Pos2::new(100.0, 100.0));
        assert!(matches!(t.loupe(p, 1.1), Err(Some(_))));
        assert_eq!(t.loupe(p, 1.3), Ok(Pos2::new(100.0, 100.0)));
        // the second finger came in time: no loupe at all
        t.pending_at = 2.0;
        t.update(&[touch(1, TouchPhase::Start, 200.0, 100.0)], None);
        t.pending = p;
        assert_eq!(t.loupe(p, 2.5), Err(None));
    }

    #[test]
    fn a_turn_past_the_wobble() {
        let mut t = Touch::default();
        t.update(&[touch(0, TouchPhase::Start, 0.0, 0.0), touch(1, TouchPhase::Start, 100.0, 0.0)], None);
        // a small turn is a pinch's wobble
        let g = t.update(&[touch(1, TouchPhase::Move, 100.0, 10.0)], None).unwrap();
        assert_eq!(g.twist, 0.0);
        let g = t.update(&[touch(1, TouchPhase::Move, 100.0, 40.0)], None).unwrap();
        assert!(g.twist > 0.2);
    }
}
