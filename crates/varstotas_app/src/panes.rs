//! Split view: 1, 2 (side by side) or 4 (2×2) panes over the same model, each with its own
//! camera and view settings (style, X-Ray, edges, profiles, axes, ground, sky, grid).
//!
//! The active pane's settings are the app's own (`camera`, `cam_anim`, `prefs.face_style` …),
//! so the tools, menus and toolbars work on it unchanged; the others wait in `panes`. Painting
//! an inactive pane swaps its settings in and back out (`exchange`). A click (any button) in a
//! pane makes it the active one, and the click then acts there.
//! Guides, shadows and the section display belong to the model: shared by all panes.

use crate::app::{FaceStylePref, VarstotasApp};
use crate::nav::{self, CameraAnimation};
use egui::{Pos2, Rect, Vec2, pos2, vec2};
use varstotas_render::{Camera, StandardView};

/// Gap between panes, points.
const GAP: f32 = 2.0;
/// Half the width of a splitter's grab strip, points.
const SPLIT_GRAB: f32 = 4.0;
/// The narrowest a pane gets, as a fraction of the view.
const MIN_SHARE: f32 = 0.1;

/// What a pane keeps of its own.
pub struct PaneView {
    /// Set up (camera framed for its view) — false for a pane never shown yet.
    ready: bool,
    camera: Camera,
    cam_anim: Option<CameraAnimation>,
    face_style: FaceStylePref,
    xray: bool,
    show_edges: bool,
    show_triangles: bool,
    realistic_edges: bool,
    profiles: bool,
    show_axes: bool,
    show_ground: bool,
    show_sky: bool,
    grid: bool,
    hud_rect: Rect,
}

impl Default for PaneView {
    fn default() -> Self {
        Self {
            ready: false,
            camera: Camera::default(),
            cam_anim: None,
            face_style: FaceStylePref::default(),
            xray: false,
            show_edges: true,
            show_triangles: false,
            realistic_edges: false,
            profiles: false,
            show_axes: true,
            show_ground: true,
            show_sky: true,
            grid: false,
            hud_rect: Rect::NOTHING,
        }
    }
}

/// The panes' rectangles in `full`: 1, 2 (left | right) or 4 (rows: 0 1 / 2 3), the
/// splitters at `split` (fractions of the width and the height).
pub fn pane_rects(full: Rect, count: usize, split: Vec2) -> Vec<Rect> {
    let mid = split_point(full, split);
    let (l, r, t, b) = (full.left(), full.right(), full.top(), full.bottom());
    let h = GAP / 2.0;
    match count {
        2 => vec![Rect::from_min_max(full.min, pos2(mid.x - h, b)), Rect::from_min_max(pos2(mid.x + h, t), full.max)],
        4 => vec![
            Rect::from_min_max(full.min, pos2(mid.x - h, mid.y - h)),
            Rect::from_min_max(pos2(mid.x + h, t), pos2(r, mid.y - h)),
            Rect::from_min_max(pos2(l, mid.y + h), pos2(mid.x - h, b)),
            Rect::from_min_max(pos2(mid.x + h, mid.y + h), full.max),
        ],
        _ => vec![full],
    }
}

/// Where the splitters cross in `full`.
fn split_point(full: Rect, split: Vec2) -> Pos2 {
    full.min + split * full.size()
}

/// The grab strips of the splitters: (strip, axis — 0 the vertical one, moved along x).
pub fn splitter_rects(full: Rect, count: usize, split: Vec2) -> Vec<(Rect, usize)> {
    let mid = split_point(full, split);
    let w = SPLIT_GRAB;
    let v = (Rect::from_min_max(pos2(mid.x - w, full.top()), pos2(mid.x + w, full.bottom())), 0);
    let h = (Rect::from_min_max(pos2(full.left(), mid.y - w), pos2(full.right(), mid.y + w)), 1);
    match count {
        2 => vec![v],
        4 => vec![v, h],
        _ => vec![],
    }
}

/// A viewport preset: its name and each pane's view and projection (`true` = parallel), in
/// pane order (2: left, right; 4: rows top-left, top-right / bottom-left, bottom-right).
pub struct Preset {
    /// Key of the preset's name.
    pub name: &'static str,
    pub panes: &'static [(StandardView, bool)],
}

impl Preset {
    /// The shown name.
    pub fn name(&self) -> String {
        varstotas_i18n::tr(self.name)
    }
}

pub const PRESETS: &[Preset] = &[
    Preset { name: "view-preset-front-iso", panes: &[(StandardView::Front, true), (StandardView::Iso, false)] },
    Preset { name: "view-preset-top-iso", panes: &[(StandardView::Top, true), (StandardView::Iso, false)] },
    Preset { name: "view-preset-front-right", panes: &[(StandardView::Front, true), (StandardView::Right, true)] },
    Preset {
        name: "view-preset-front-left-top-iso",
        panes: &[(StandardView::Front, true), (StandardView::Left, true), (StandardView::Top, true), (StandardView::Iso, false)],
    },
    Preset {
        name: "view-preset-top-front-right-iso",
        panes: &[(StandardView::Top, true), (StandardView::Front, true), (StandardView::Right, true), (StandardView::Iso, false)],
    },
    Preset {
        name: "view-preset-front-back-left-right",
        panes: &[(StandardView::Front, true), (StandardView::Back, true), (StandardView::Left, true), (StandardView::Right, true)],
    },
];

/// The view a pane opens with the first time: the model from the top, front, right.
fn first_view(k: usize) -> StandardView {
    match k {
        1 => StandardView::Top,
        2 => StandardView::Front,
        _ => StandardView::Right,
    }
}

impl VarstotasApp {
    /// Swap the settings of pane `k` with the app's own (an involution: twice = no change).
    pub(crate) fn exchange(&mut self, k: usize) {
        use std::mem::swap;
        let p = &mut self.panes[k];
        let prefs = &mut self.prefs;
        swap(&mut p.camera, &mut self.camera);
        swap(&mut p.cam_anim, &mut self.cam_anim);
        swap(&mut p.face_style, &mut prefs.face_style);
        swap(&mut p.xray, &mut prefs.xray);
        swap(&mut p.show_edges, &mut prefs.show_edges);
        swap(&mut p.show_triangles, &mut prefs.show_triangles);
        swap(&mut p.realistic_edges, &mut prefs.realistic_edges);
        swap(&mut p.profiles, &mut prefs.profiles);
        swap(&mut p.show_axes, &mut prefs.show_axes);
        swap(&mut p.show_ground, &mut prefs.show_ground);
        swap(&mut p.show_sky, &mut prefs.show_sky);
        swap(&mut p.grid, &mut prefs.grid.show);
        swap(&mut p.hud_rect, &mut self.hud_rect);
    }

    /// Make pane `k` the active one.
    pub(crate) fn activate_pane(&mut self, k: usize) {
        if k == self.active_pane || k >= self.panes.len() {
            return;
        }
        let a = self.active_pane;
        self.exchange(a);
        // (`ready` stays with the slot: the active pane's settings were never "not set up")
        self.panes[a].ready = true;
        self.exchange(k);
        self.active_pane = k;
        // the tools' pointer positions are relative to the pane
        self.last_pointer = None;
    }

    /// 1, 2 or 4 panes; a pane that goes away keeps its settings for the next time.
    pub(crate) fn set_pane_count(&mut self, n: usize) {
        let n = if n >= 4 { 4 } else if n >= 2 { 2 } else { 1 };
        if self.active_pane >= n {
            self.activate_pane(0);
        }
        self.pane_count = n;
    }

    /// Show the panes of `PRESETS[i]`, each framed in its view; the panes keep their styles
    /// (new ones take the active pane's). The top-left pane becomes the active one.
    pub(crate) fn apply_preset(&mut self, i: usize) {
        let Some(p) = PRESETS.get(i) else { return };
        self.activate_pane(0);
        self.set_pane_count(p.panes.len());
        let rects = pane_rects(self.viewport.view_rect, self.pane_count, self.pane_split);
        self.init_panes(&rects);
        for (k, (&(v, ortho), r)) in p.panes.iter().zip(&rects).enumerate() {
            let aspect = (r.width() / r.height().max(1.0)) as f64;
            let mut cam = self.camera.clone();
            cam.set_ortho(ortho);
            let cam = nav::standard_view_camera(&self.doc.model, &cam, v, aspect);
            if k == self.active_pane {
                self.cam_anim = None;
                self.camera = cam;
            } else {
                self.panes[k].cam_anim = None;
                self.panes[k].camera = cam;
            }
        }
    }

    /// Panes shown for the first time: framed in their standard view, parallel, in the
    /// active pane's style.
    pub(crate) fn init_panes(&mut self, rects: &[Rect]) {
        for (k, r) in rects.iter().enumerate() {
            if k == self.active_pane || self.panes[k].ready {
                continue;
            }
            let aspect = (r.width() / r.height().max(1.0)) as f64;
            let mut cam = self.camera.clone();
            cam.set_ortho(true);
            let cam = nav::standard_view_camera(&self.doc.model, &cam, first_view(k), aspect);
            let prefs = &self.prefs;
            self.panes[k] = PaneView {
                ready: true,
                camera: cam,
                cam_anim: None,
                face_style: prefs.face_style,
                xray: prefs.xray,
                show_edges: prefs.show_edges,
                show_triangles: prefs.show_triangles,
                realistic_edges: prefs.realistic_edges,
                profiles: prefs.profiles,
                show_axes: prefs.show_axes,
                show_ground: prefs.show_ground,
                show_sky: prefs.show_sky,
                grid: prefs.grid.show,
                hud_rect: Rect::NOTHING,
            };
        }
    }

    /// Dragging a splitter between the panes resizes them; a double click centres it again.
    /// Returns the grab strips (the viewport leaves presses there alone).
    pub(crate) fn splitters(&mut self, ui: &mut egui::Ui) -> Vec<Rect> {
        let full = self.viewport.view_rect;
        let strips = splitter_rects(full, self.pane_count, self.pane_split);
        for &(r, axis) in &strips {
            let resp = ui.interact(r, ui.id().with(("pane_splitter", axis)), egui::Sense::click_and_drag());
            crate::ui::tray_lib::spot(if axis == 0 { "pane:split_v" } else { "pane:split_h" }, r.center() + vec2(0.0, if axis == 0 { 40.0 } else { 0.0 }));
            if resp.hovered() || resp.dragged() {
                ui.ctx().set_cursor_icon(if axis == 0 { egui::CursorIcon::ResizeHorizontal } else { egui::CursorIcon::ResizeVertical });
            }
            if resp.double_clicked() {
                self.pane_split[axis] = 0.5;
            } else if resp.dragged()
                && let Some(p) = resp.interact_pointer_pos()
            {
                let t = (p - full.min)[axis] / full.size()[axis].max(1.0);
                self.pane_split[axis] = t.clamp(MIN_SHARE, 1.0 - MIN_SHARE);
            }
            if resp.hovered() || resp.dragged() {
                ui.painter().rect_filled(r.shrink2(vec2(if axis == 0 { 2.5 } else { 0.0 }, if axis == 0 { 0.0 } else { 2.5 })), 0.0, crate::ui::theme::accent_soft());
            }
            if resp.dragged() {
                self.splitter_drag = true;
            }
        }
        strips.into_iter().map(|s| s.0).collect()
    }

    /// The pane clicked in becomes the active one.
    pub(crate) fn click_activate(&mut self, ui: &egui::Ui, rects: &[Rect], strips: &[Rect]) {
        if rects.len() < 2 || self.tool_pressed || self.viewport.nav.is_dragging() {
            return;
        }
        let ctx = ui.ctx();
        let press = ctx.input(|i| {
            i.events.iter().find_map(|e| match e {
                egui::Event::PointerButton { pos, pressed: true, .. } => Some(*pos),
                _ => None,
            })
        });
        let Some(p) = press else { return };
        if strips.iter().any(|s| s.contains(p)) {
            return;
        }
        // not through a window or the tray over the view
        if ctx.layer_id_at(p) != Some(ui.layer_id()) {
            return;
        }
        if let Some(k) = rects.iter().position(|r| r.contains(p)) {
            self.activate_pane(k);
        }
    }

    /// Each shown pane: its standard view, projection and style (the harness dump).
    pub(crate) fn panes_dump(&self) -> Vec<(Option<StandardView>, bool, FaceStylePref)> {
        (0..self.pane_count)
            .map(|k| {
                if k == self.active_pane {
                    (self.camera.standard_view(), self.camera.ortho, self.prefs.face_style)
                } else {
                    let p = &self.panes[k];
                    (p.camera.standard_view(), p.camera.ortho, p.face_style)
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rects_tile_the_view() {
        let full = Rect::from_min_max(pos2(0.0, 0.0), pos2(800.0, 600.0));
        let half = vec2(0.5, 0.5);
        assert_eq!(pane_rects(full, 1, half), vec![full]);
        let two = pane_rects(full, 2, half);
        assert_eq!(two.len(), 2);
        assert_eq!(two[0].right() + GAP, two[1].left());
        let four = pane_rects(full, 4, half);
        assert_eq!(four.len(), 4);
        assert_eq!(four[3].max, full.max);
        assert!((four[0].area() - four[3].area()).abs() < 1.0);
        // moved splitters: the left column a quarter wide, the top row three quarters high
        let moved = pane_rects(full, 4, vec2(0.25, 0.75));
        assert!((moved[0].right() - (200.0 - GAP / 2.0)).abs() < 1e-3);
        assert!((moved[3].top() - (450.0 + GAP / 2.0)).abs() < 1e-3);
        assert_eq!(splitter_rects(full, 4, vec2(0.25, 0.75)).len(), 2);
    }
}
