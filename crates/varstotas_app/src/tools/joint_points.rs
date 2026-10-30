//! Joint Points: move the fasteners of a joint along it (Window → Joinery → Points). Drag a
//! point along the joint (whole millimetres), or pick one and type its distance from the joint's
//! start; a double-click on the joint adds a point, Delete removes the picked one. Every change
//! is one undo step and gives the joint its own list of distances (`WoodJoint::at`).

use varstotas_i18n::tr;
use super::common::*;
use super::{OverlayView, Tool, ToolContext, ToolId, ToolRequest};
use crate::overlay::{Marker2d, MarkerShape};
use egui::{Color32, CursorIcon, Key, PointerButton};
use varstotas_kernel::glam::DVec3;
use varstotas_kernel::joinery::{Contact, WoodJoint};
use varstotas_kernel::{Model, Ray};
use varstotas_render::Overlay3d;

/// Screen distance (points) for grabbing a point or clicking the joint.
const GRIP_PX: f32 = 9.0;
const LINE: Color32 = Color32::from_rgb(0x00, 0x50, 0xFF);
const HOT: Color32 = Color32::from_rgb(0xE0, 0x60, 0x10);

pub struct JointPointsTool {
    joint: usize,
    /// The points' distances from the joint's start while being edited (committed on release).
    work: Option<Vec<f64>>,
    /// The point dragged.
    drag: Option<usize>,
    /// The picked point (Delete, typed distance).
    picked: Option<usize>,
    hover: Option<usize>,
}

impl JointPointsTool {
    pub fn new(joint: usize) -> Self {
        Self { joint, work: None, drag: None, picked: None, hover: None }
    }

    /// The joint, where its parts touch and where the joint lies now (with part a, exploded).
    fn joint<'a>(&self, m: &'a Model) -> Option<(&'a WoodJoint, Contact, DVec3)> {
        let j = m.placed_joints.get(self.joint)?;
        let c = (*m.joint_contacts.get(self.joint)?)?;
        Some((j, c, m.assembly_offset(&j.a)))
    }

    fn positions(&self, m: &Model) -> Vec<f64> {
        match (&self.work, self.joint(m)) {
            (Some(w), _) => w.clone(),
            (None, Some((j, c, _))) => j.positions(&c),
            _ => Vec::new(),
        }
    }

    /// The world point at `s` along the joint.
    fn at(j: &WoodJoint, c: &Contact, off: DVec3, s: f64) -> DVec3 {
        j.point(c, c.lo[c.along] + s) + off
    }

    fn under(&self, ctx: &ToolContext) -> Option<usize> {
        let (j, c, off) = self.joint(&ctx.doc.model)?;
        let mut best = None;
        for (k, s) in self.positions(&ctx.doc.model).into_iter().enumerate() {
            let Some(p) = ctx.to_screen(Self::at(j, &c, off, s)) else { continue };
            let d = (p - ctx.pointer.pos).length();
            if d <= GRIP_PX && best.is_none_or(|(_, b)| d < b) {
                best = Some((k, d));
            }
        }
        best.map(|b| b.0)
    }

    /// Where the pointer's ray passes closest to the joint, from its start (whole mm, on it).
    fn along(&self, ctx: &ToolContext) -> Option<f64> {
        let (j, c, off) = self.joint(&ctx.doc.model)?;
        let base = Self::at(j, &c, off, 0.0);
        let mut u = DVec3::ZERO;
        u[c.along] = 1.0;
        let Ray { origin, dir } = ctx.pointer.ray;
        let w = base - origin;
        let (b, d, e) = (u.dot(dir), u.dot(w), dir.dot(w));
        let den = dir.length_squared() - b * b;
        if den.abs() < 1e-9 {
            return None;
        }
        let t = (b * e - dir.length_squared() * d) / den;
        Some(t.round().clamp(0.0, c.length()))
    }

    fn commit(&mut self, ctx: &mut ToolContext, mut at: Vec<f64>, label: &str) -> bool {
        let Some(mut j) = ctx.doc.model.placed_joints.get(self.joint).cloned() else { return false };
        at.sort_by(f64::total_cmp);
        j.count = at.len();
        j.at = at;
        let i = self.joint;
        self.work = None;
        ctx.doc.try_commit(label, move |m| m.set_wood_joint(i, j)).is_some()
    }

    fn finish(&mut self, ctx: &mut ToolContext) {
        self.work = None;
        self.drag = None;
        ctx.requests.push(ToolRequest::SwitchTool(ToolId::Select));
    }
}

impl Tool for JointPointsTool {
    fn id(&self) -> ToolId {
        ToolId::Move
    }

    fn activate(&mut self, ctx: &mut ToolContext) {
        if self.joint(&ctx.doc.model).is_none() {
            ctx.doc.last_error = Some(tr!("joint-err-no-contact"));
            self.finish(ctx);
        }
    }

    fn on_pointer_move(&mut self, ctx: &mut ToolContext) {
        if let Some(k) = self.drag {
            if !ctx.pointer.primary_down {
                return;
            }
            let Some(s) = self.along(ctx) else { return };
            let mut w = self.positions(&ctx.doc.model);
            if k < w.len() {
                w[k] = s;
                self.work = Some(w);
            }
            return;
        }
        self.hover = self.under(ctx);
    }

    fn on_pointer_down(&mut self, ctx: &mut ToolContext) {
        if ctx.pointer.button != Some(PointerButton::Primary) {
            return;
        }
        match self.under(ctx) {
            Some(k) => {
                self.picked = Some(k);
                self.drag = Some(k);
            }
            None if self.joint(&ctx.doc.model).is_none() => self.finish(ctx),
            None => self.picked = None,
        }
    }

    fn on_pointer_up(&mut self, ctx: &mut ToolContext) {
        if ctx.pointer.button != Some(PointerButton::Primary) || self.drag.take().is_none() {
            return;
        }
        if let Some(w) = self.work.take() {
            let moved = self.joint(&ctx.doc.model).is_some_and(|(j, c, _)| j.positions(&c) != w);
            if moved {
                let k = self.picked;
                let s = k.and_then(|k| w.get(k).copied());
                if self.commit(ctx, w, &tr!("tool-undo-joint-points")) {
                    // the picked point after sorting
                    self.picked = s.and_then(|s| self.positions(&ctx.doc.model).iter().position(|p| *p == s));
                }
            }
        }
    }

    fn on_double_click(&mut self, ctx: &mut ToolContext) {
        if self.under(ctx).is_some() {
            return;
        }
        let Some(s) = self.along(ctx) else { return };
        // on the joint only
        let Some((j, c, off)) = self.joint(&ctx.doc.model) else { return };
        let p = Self::at(j, &c, off, s);
        if ctx.to_screen(p).is_none_or(|q| (q - ctx.pointer.pos).length() > GRIP_PX) {
            return;
        }
        let mut w = self.positions(&ctx.doc.model);
        w.push(s);
        if self.commit(ctx, w, &tr!("tool-undo-joint-points")) {
            self.picked = self.positions(&ctx.doc.model).iter().position(|x| *x == s);
        }
    }

    fn on_key(&mut self, ctx: &mut ToolContext, key: Key, pressed: bool) -> bool {
        if !pressed {
            return false;
        }
        match key {
            Key::Escape if self.drag.is_some() => {
                self.drag = None;
                self.work = None;
                true
            }
            Key::Escape | Key::Enter => {
                self.finish(ctx);
                true
            }
            Key::Delete | Key::Backspace => {
                let mut w = self.positions(&ctx.doc.model);
                if let Some(k) = self.picked.filter(|&k| k < w.len()) {
                    if w.len() <= 1 {
                        ctx.doc.last_error = Some(tr!("tool-joint-points-last"));
                    } else {
                        w.remove(k);
                        if self.commit(ctx, w, &tr!("tool-undo-joint-points")) {
                            self.picked = None;
                        }
                    }
                }
                true
            }
            _ => false,
        }
    }

    fn on_vcb(&mut self, ctx: &mut ToolContext, input: &str) -> bool {
        let unit = ctx.doc.model.unit;
        let (Some(v), Some(k)) = (parse_len(unit, input), self.picked) else { return false };
        let mut w = self.positions(&ctx.doc.model);
        if k >= w.len() {
            return false;
        }
        w[k] = v;
        if self.commit(ctx, w, &tr!("tool-undo-joint-points")) {
            self.picked = self.positions(&ctx.doc.model).iter().position(|x| *x == v);
            return true;
        }
        false
    }

    fn vcb_label(&self) -> String {
        tr!("tool-vcb-distance")
    }

    fn vcb_value(&self) -> String {
        String::new()
    }

    fn status_text(&self) -> String {
        tr!("tool-joint-points", joint = self.joint + 1)
    }

    fn overlay(&self, view: &OverlayView, out3d: &mut Overlay3d, out2d: &mut Vec<Marker2d>) {
        let m = &view.doc.model;
        let Some((j, c, off)) = self.joint(m) else { return };
        line(out3d, Self::at(j, &c, off, 0.0), Self::at(j, &c, off, c.length()), LINE, 2.0);
        let k_hot = self.drag.or(self.hover).or(self.picked);
        for (k, s) in self.positions(m).into_iter().enumerate() {
            let p = Self::at(j, &c, off, s);
            let hot = Some(k) == k_hot;
            out2d.push(Marker2d::Point { at: p, shape: if hot { MarkerShape::Square } else { MarkerShape::Dot }, color: if hot { HOT } else { LINE } });
            if hot {
                out2d.push(Marker2d::Label { at: p, text: crate::ui::fmt_len(s, m.unit) });
            }
        }
    }

    fn cursor(&self) -> CursorIcon {
        if self.drag.is_some() || self.hover.is_some() { CursorIcon::Grab } else { CursorIcon::Default }
    }

    fn cursor_glyph(&self) -> Option<Marker2d> {
        None
    }
}
