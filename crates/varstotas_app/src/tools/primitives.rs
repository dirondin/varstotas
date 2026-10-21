//! Draw → Shapes: Box, Cylinder, Cone, Pyramid, Sphere, Torus in one operation. The base is
//! drawn on a face (or the axes' plane) like Rectangle / Circle, then the height (the torus:
//! the tube) is set; the result is a component of the bundled generator of that shape
//! ([`SOURCES`]), so its sizes stay editable in Entity Info.

use varstotas_i18n::tr;
use super::common::*;
use super::draw::{Clicker, plane_point, shape_normal};
use super::{OverlayView, Tool, ToolContext, ToolId};
use crate::inference::{self, Constraint, InferenceEngine, InferenceKind, InferenceResult};
use crate::overlay::Marker2d;
use egui::{Color32, CursorIcon, Key};
use varstotas_kernel::glam::{DMat4, DVec3, DVec4};
use varstotas_kernel::primitives::Primitive;
use varstotas_kernel::{EntityRef, GenValue, GenValues, Plane, TOL};
use varstotas_render::Overlay3d;

/// The shape tools' generators: (tool, generator name, source). Each puts the shape's base on
/// the component's red–green plane at the origin (round shapes centred on it).
pub const SOURCES: [(ToolId, &str, &str); 6] = [
    (ToolId::Box, "Box", include_str!("../../../../examples/generators/primitives/box.rhai")), // i18n-ignore
    (ToolId::Cylinder, "Cylinder", include_str!("../../../../examples/generators/primitives/cylinder.rhai")), // i18n-ignore
    (ToolId::Cone, "Cone", include_str!("../../../../examples/generators/primitives/cone.rhai")), // i18n-ignore
    (ToolId::Pyramid, "Pyramid", include_str!("../../../../examples/generators/primitives/pyramid.rhai")), // i18n-ignore
    (ToolId::Sphere, "Sphere", include_str!("../../../../examples/generators/primitives/sphere.rhai")), // i18n-ignore
    (ToolId::Torus, "Torus", include_str!("../../../../examples/generators/primitives/torus.rhai")), // i18n-ignore
];

/// Torus tube sides per ring segment.
const TORUS_SIDES: u32 = 12;

#[derive(Clone, Copy, Debug)]
struct Base {
    /// First corner (box, pyramid) or centre (round shapes).
    o: DVec3,
    n: DVec3,
    /// In-plane axes: fixed for box / pyramid, `u` towards the radius point for round shapes.
    u: DVec3,
    v: DVec3,
    /// Signed sizes along `u` / `v` (box, pyramid), or the radius in `a`.
    a: f64,
    b: f64,
}

#[derive(Clone, Copy, Debug)]
enum Stage {
    Idle,
    /// First click done: the base follows the pointer.
    Base { o: DVec3, n: DVec3 },
    /// Base fixed: the height (torus: tube radius) follows the pointer.
    Height(Base),
}

/// A shape ready to place: the primitive (for the preview), its frame in world coordinates and
/// the generator values.
struct Placed {
    prim: Primitive,
    frame: DMat4,
    values: GenValues,
}

pub struct PrimitiveTool {
    id: ToolId,
    engine: InferenceEngine,
    segs: u32,
    stage: Stage,
    /// The base or the height under the pointer.
    base: Option<Base>,
    height: f64,
    cur: Option<InferenceResult>,
    clicker: Clicker,
    unit: Option<varstotas_kernel::LengthUnit>,
}

fn round(id: ToolId) -> bool {
    matches!(id, ToolId::Cylinder | ToolId::Cone | ToolId::Sphere | ToolId::Torus)
}

fn num(v: f64) -> GenValue {
    GenValue::Num(v)
}

impl PrimitiveTool {
    pub fn new(id: ToolId) -> Self {
        Self { id, engine: InferenceEngine::default(), segs: 24, stage: Stage::Idle, base: None, height: 0.0, cur: None, clicker: Clicker::default(), unit: None }
    }

    fn name(&self) -> &'static str {
        SOURCES.iter().find(|s| s.0 == self.id).map(|s| s.1).unwrap_or("Box") // i18n-ignore
    }

    fn source(&self) -> &'static str {
        SOURCES.iter().find(|s| s.0 == self.id).map(|s| s.2).unwrap_or(SOURCES[0].2)
    }

    /// The base for corner / centre `o` on the plane with normal `n` and the pointer at `q`.
    fn base_at(&self, o: DVec3, n: DVec3, q: DVec3) -> Base {
        let (u, v) = plane_axes(n);
        let d = q - o;
        if round(self.id) {
            let r = d.length();
            let u = if r > 1e-9 { d / r } else { u };
            Base { o, n, u, v: n.cross(u), a: r, b: 0.0 }
        } else {
            Base { o, n, u, v, a: d.dot(u), b: d.dot(v) }
        }
    }

    /// The shape for `base` and height (torus: tube) `h`; None when degenerate.
    fn placed(&self, base: &Base, h: f64) -> Option<Placed> {
        let Base { o, n, u, v, a, b } = *base;
        let frame = |o: DVec3, u: DVec3, v: DVec3, n: DVec3| DMat4::from_cols(u.extend(0.0), v.extend(0.0), n.extend(0.0), DVec4::new(o.x, o.y, o.z, 1.0));
        let segs = self.segs;
        let (prim, frame, values) = match self.id {
            ToolId::Box | ToolId::Pyramid => {
                let o = o + u * a.min(0.0) + v * b.min(0.0) + if self.id == ToolId::Box { n * h.min(0.0) } else { DVec3::ZERO };
                let (w, d, hh) = (a.abs(), b.abs(), h.abs());
                // A pyramid pointing down is turned over (apex below the base).
                let (o, v2, n2) = if self.id == ToolId::Pyramid && h < 0.0 { (o + v * d, -v, -n) } else { (o, v, n) };
                let prim = if self.id == ToolId::Box { Primitive::Box { w, d, h: hh } } else { Primitive::Pyramid { w, d, h: hh } };
                let values = GenValues::from([("width".into(), num(w)), ("depth".into(), num(d)), ("height".into(), num(hh))]);
                (prim, frame(o, u, v2, n2), values)
            }
            ToolId::Cylinder => {
                let o = if h < 0.0 { o + n * h } else { o };
                let values = GenValues::from([("radius".into(), num(a)), ("height".into(), num(h.abs())), ("segments".into(), num(segs as f64))]);
                (Primitive::Cylinder { r: a, h: h.abs(), segs }, frame(o, u, v, n), values)
            }
            ToolId::Cone => {
                let (v, n) = if h < 0.0 { (-v, -n) } else { (v, n) };
                let values = GenValues::from([("radius".into(), num(a)), ("height".into(), num(h.abs())), ("segments".into(), num(segs as f64))]);
                (Primitive::Cone { r: a, h: h.abs(), segs }, frame(o, u, v, n), values)
            }
            ToolId::Sphere => {
                let values = GenValues::from([("radius".into(), num(a)), ("segments".into(), num(segs as f64))]);
                (Primitive::Sphere { r: a, segs }, frame(o, u, v, n), values)
            }
            _ => {
                let values = GenValues::from([
                    ("radius".into(), num(a)),
                    ("tube".into(), num(h)),
                    ("segments".into(), num(segs as f64)),
                    ("sides".into(), num(TORUS_SIDES as f64)),
                ]);
                (Primitive::Torus { r: a, tube: h, segs, sides: TORUS_SIDES }, frame(o, u, v, n), values)
            }
        };
        prim.is_valid().then_some(Placed { prim, frame, values })
    }

    /// Where the generator puts the primitive's own axes in the component (round shapes stand
    /// on the base plane).
    fn lift(prim: &Primitive) -> DVec3 {
        match *prim {
            Primitive::Sphere { r, .. } => DVec3::Z * r,
            Primitive::Torus { tube, .. } => DVec3::Z * tube,
            _ => DVec3::ZERO,
        }
    }

    fn update(&mut self, ctx: &ToolContext) {
        self.unit = Some(ctx.doc.model.unit);
        match self.stage {
            Stage::Idle => {
                self.cur = Some(infer(&mut self.engine, ctx, None, None, None));
                self.base = None;
            }
            Stage::Base { o, n } => {
                let plane = Plane::new(o, n);
                let r = infer(&mut self.engine, ctx, Some(o), Some(Constraint::Plane(plane)), Some(plane));
                let q = plane_point(ctx, &r, &plane);
                self.base = Some(self.base_at(o, n, q));
                self.cur = Some(r);
            }
            Stage::Height(b) => {
                if self.id == ToolId::Torus {
                    // The tube: the pointer's distance from the ring on the base plane.
                    let plane = Plane::new(b.o, b.n);
                    let r = infer(&mut self.engine, ctx, Some(b.o), Some(Constraint::Plane(plane)), Some(plane));
                    let q = plane_point(ctx, &r, &plane);
                    self.height = ((q - b.o).length() - b.a).abs().min(b.a * 0.95);
                    self.cur = Some(r);
                } else {
                    let anchor = b.o + b.u * b.a * 0.5 * if round(self.id) { 0.0 } else { 1.0 } + b.v * b.b * 0.5;
                    let c = Constraint::Line { origin: anchor, dir: b.n, kind: InferenceKind::Free };
                    let r = infer(&mut self.engine, ctx, Some(anchor), Some(c), None);
                    self.height = (r.point - anchor).dot(b.n);
                    self.cur = Some(r);
                }
            }
        }
    }

    fn click(&mut self, ctx: &mut ToolContext) {
        self.update(ctx);
        match self.stage {
            Stage::Idle => {
                let Some(r) = self.cur.clone() else { return };
                let (n, _) = shape_normal(&self.engine, &r);
                self.stage = Stage::Base { o: r.point, n };
                self.engine.arrow_lock = None;
            }
            Stage::Base { .. } => {
                let Some(b) = self.base.filter(|b| b.a.abs() > TOL && (round(self.id) || b.b.abs() > TOL)) else { return };
                if self.id == ToolId::Sphere {
                    self.finish(ctx, &b, 0.0);
                } else {
                    self.stage = Stage::Height(b);
                    self.height = 0.0;
                }
            }
            Stage::Height(b) => {
                if self.height.abs() > TOL {
                    self.finish(ctx, &b, self.height);
                }
            }
        }
    }

    /// Place the shape: the generator is added to the document (once) and a component of it
    /// placed in the active context and selected — one undo step.
    fn finish(&mut self, ctx: &mut ToolContext, base: &Base, h: f64) {
        self.stage = Stage::Idle;
        self.base = None;
        let Some(p) = self.placed(base, h) else { return };
        let local = context_xf(ctx.doc).inverse() * p.frame;
        let path = ctx.doc.active_path.clone();
        let (name, src) = (self.name(), self.source());
        let r = ctx.doc.try_commit(name, |m| -> Result<_, String> {
            varstotas_script::generator::add(m, src)?;
            varstotas_script::generator::place(m, &path, name, &p.values, local)
        });
        if let Some(id) = r {
            ctx.doc.set_selection(vec![EntityRef::Instance(id)]);
        }
    }

    /// The current preview shape.
    fn preview(&self) -> Option<Placed> {
        match self.stage {
            Stage::Idle => None,
            Stage::Base { .. } => {
                let b = self.base?;
                // Before the height: a flat outline of the base (the sphere: the whole ball).
                match self.id {
                    ToolId::Sphere => self.placed(&b, 0.0),
                    ToolId::Torus => self.placed(&b, b.a * 0.2),
                    _ => self.placed(&b, 1e-3),
                }
            }
            Stage::Height(b) => self.placed(&b, if self.height.abs() > 1e-3 { self.height } else { 1e-3 }),
        }
    }
}

impl Tool for PrimitiveTool {
    fn id(&self) -> ToolId {
        self.id
    }
    fn on_pointer_move(&mut self, ctx: &mut ToolContext) {
        self.update(ctx);
    }
    fn on_pointer_down(&mut self, ctx: &mut ToolContext) {
        if self.clicker.down(ctx) {
            self.click(ctx);
        }
    }
    fn on_pointer_up(&mut self, ctx: &mut ToolContext) {
        if self.clicker.up_is_drag(ctx) && !matches!(self.stage, Stage::Idle) {
            self.click(ctx);
        }
    }
    fn on_key(&mut self, ctx: &mut ToolContext, key: Key, pressed: bool) -> bool {
        if !pressed {
            return false;
        }
        if key == Key::Escape {
            self.stage = Stage::Idle;
            self.base = None;
            self.engine.clear_locks();
            return true;
        }
        let r = self.engine.handle_arrow(key);
        if r {
            // After the first click an arrow sets the base plane's normal.
            if let (Stage::Base { o, .. }, Some((d, _))) = (self.stage, self.engine.arrow_dir()) {
                self.stage = Stage::Base { o, n: d };
                self.engine.arrow_lock = None;
            }
            self.update(ctx);
        }
        r
    }
    fn on_vcb(&mut self, ctx: &mut ToolContext, input: &str) -> bool {
        if let Some(n) = parse_segments(input) {
            if !round(self.id) || (self.id == ToolId::Sphere && n < 4) {
                return false;
            }
            self.segs = n;
            self.update(ctx);
            return true;
        }
        let unit = ctx.doc.model.unit;
        let Some(vals) = parse_lens(unit, input).filter(|v| !v.is_empty()) else { return false };
        match self.stage {
            Stage::Idle => false,
            Stage::Base { o, n } => {
                let Some(cur) = self.base else { return false };
                // Typed sizes keep the pointer's directions.
                let sign = |typed: f64, cur: f64| if cur < 0.0 { -typed.abs() } else { typed };
                let mut b = cur;
                let rest: &[f64] = if round(self.id) {
                    b.a = vals[0].abs();
                    &vals[1..]
                } else {
                    let [w, d, rest @ ..] = vals.as_slice() else { return false };
                    b.a = sign(*w, cur.a);
                    b.b = sign(*d, cur.b);
                    rest
                };
                b.o = o;
                b.n = n;
                if b.a.abs() <= TOL || (!round(self.id) && b.b.abs() <= TOL) {
                    return false;
                }
                match (self.id, rest.first()) {
                    (ToolId::Sphere, _) => self.finish(ctx, &b, 0.0),
                    (_, Some(h)) => self.finish(ctx, &b, *h),
                    (_, None) => {
                        self.stage = Stage::Height(b);
                        self.height = 0.0;
                    }
                }
                true
            }
            Stage::Height(b) => {
                let h = if self.height < 0.0 && self.id != ToolId::Torus { -vals[0].abs() } else { vals[0] };
                if h.abs() <= TOL {
                    return false;
                }
                self.finish(ctx, &b, h);
                true
            }
        }
    }
    fn vcb_label(&self) -> String {
        match (self.stage, self.id) {
            (Stage::Height(_), ToolId::Torus) => tr!("tool-vcb-tube"),
            (Stage::Height(_), _) => tr!("tool-vcb-height"),
            (_, ToolId::Box | ToolId::Pyramid) => tr!("tool-vcb-dimensions"),
            _ => tr!("tool-vcb-radius"),
        }
    }
    fn vcb_value(&self) -> String {
        let Some(u) = self.unit else { return String::new() };
        let f = |v: f64| varstotas_kernel::units::format_length(v.abs(), u);
        match (self.stage, self.base) {
            (Stage::Height(_), _) => f(self.height),
            (Stage::Base { .. }, Some(b)) if round(self.id) => f(b.a),
            (Stage::Base { .. }, Some(b)) => format!("{}, {}", f(b.a), f(b.b)),
            _ => String::new(),
        }
    }
    fn status_text(&self) -> String {
        let what = self.id.name().to_lowercase();
        match (self.stage, self.id) {
            (Stage::Idle, ToolId::Box | ToolId::Pyramid) => tr!("tool-prim-box-first", what = what),
            (Stage::Idle, _) => tr!("tool-prim-round-first", what = what, segs = self.segs),
            (Stage::Base { .. }, ToolId::Box | ToolId::Pyramid) => tr!("tool-prim-box-second"),
            (Stage::Base { .. }, ToolId::Sphere) => tr!("tool-prim-sphere-radius"),
            (Stage::Base { .. }, ToolId::Torus) => tr!("tool-prim-torus-radius"),
            (Stage::Base { .. }, _) => tr!("tool-prim-round-radius"),
            (Stage::Height(_), ToolId::Torus) => tr!("tool-prim-torus-tube"),
            (Stage::Height(_), _) => tr!("tool-prim-height"),
        }
    }
    fn overlay(&self, view: &OverlayView, out3d: &mut Overlay3d, out2d: &mut Vec<Marker2d>) {
        if let Some(p) = self.preview() {
            let lift = Self::lift(&p.prim);
            let soft = Color32::from_rgba_unmultiplied(0x10, 0x10, 0x10, 70);
            for (poly, curved) in p.prim.polygons() {
                let pts: Vec<DVec3> = poly.iter().map(|q| p.frame.transform_point3(*q + lift)).collect();
                for i in 0..pts.len() {
                    let (a, b) = (pts[i], pts[(i + 1) % pts.len()]);
                    if curved { line(out3d, a, b, soft, 1.0) } else { line(out3d, a, b, PREVIEW, 2.0) }
                }
            }
            if let Some(b) = self.base {
                let m = &view.doc.model;
                match self.stage {
                    Stage::Base { .. } if round(self.id) => label(out2d, b.o + b.u * b.a * 0.5, fmt_len(m, b.a)),
                    Stage::Base { .. } => label(out2d, b.o + b.u * b.a + b.v * b.b * 0.5, format!("{} × {}", fmt_len(m, b.a.abs()), fmt_len(m, b.b.abs()))),
                    _ => {}
                }
            }
            if let Stage::Height(b) = self.stage {
                label(out2d, b.o + b.n * self.height * 0.5, fmt_len(&view.doc.model, self.height.abs()));
            }
        }
        if let Some(r) = &self.cur {
            inference::draw_result(r, out3d, out2d);
        }
    }
    fn cursor(&self) -> CursorIcon {
        CursorIcon::None
    }
    fn is_busy(&self) -> bool {
        !matches!(self.stage, Stage::Idle)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shape_generators_parse_and_build() {
        for (id, name, src) in SOURCES {
            let g = varstotas_kernel::Generator::parse(src).unwrap_or_else(|e| panic!("{name}: {e}"));
            assert_eq!(g.name, name);
            let mut m = varstotas_kernel::Model::new();
            varstotas_script::generator::add(&mut m, src).unwrap();
            let t = PrimitiveTool::new(id);
            let base = Base { o: DVec3::ZERO, n: DVec3::Z, u: DVec3::X, v: DVec3::Y, a: 300.0, b: 200.0 };
            let p = t.placed(&base, 100.0).unwrap_or_else(|| panic!("{name}"));
            let inst = varstotas_script::generator::place(&mut m, &[], name, &p.values, p.frame).unwrap_or_else(|e| panic!("{name}: {e}"));
            let def = m.root_mesh().instances[inst].def;
            let mesh = &m.defs[def].mesh;
            // The content is the shape itself (no nested group) and a closed solid.
            assert!(mesh.instances.is_empty(), "{name}");
            assert_eq!(mesh.faces.len(), p.prim.mesh().unwrap().faces.len(), "{name}");
            assert!(mesh.volume() > 0.0, "{name}");
        }
    }
}
