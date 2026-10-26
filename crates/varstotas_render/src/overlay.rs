//! Annotation overlays (M4 batch 2): screen-aligned text labels, end marks / point markers,
//! polylines with a continuous dash pattern, arcs, angle indicators, linear dimensions, leader
//! text and axes for custom frames. All of them are drawn in 3D by the renderer, so they also
//! appear in `Renderer::render_to_image` (sizes in points are scaled by `ExportOptions::line_scale`).
//!
//! Screen-space orientation (text along a direction, arrowheads) is resolved by the renderer for
//! the frame / image being drawn, so the same `Overlay3d` works in the viewport and in exports.

use crate::{Camera, Overlay3d, OverlayLine, OverlayPolygon};
use glam::{DMat4, DVec2, DVec3};

/// Default label font size in points.
pub const DEFAULT_TEXT_SIZE: f32 = 12.0;
/// Padding between the text and its background box / border, in points.
pub const TEXT_PADDING: f32 = 3.0;
/// Default end mark / marker size in points (arrow length, dot diameter ≈ 0.7 of it).
pub const DEFAULT_MARK_SIZE: f32 = 9.0;
/// Gap between a dimension line and text placed above it, in points.
pub const DIMENSION_TEXT_GAP: f32 = 3.0;

fn f3(p: DVec3) -> [f32; 3] {
    p.as_vec3().to_array()
}

/// Horizontal anchor of a text box (in the text's own frame when it runs `along` a direction).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum HAlign {
    Left,
    #[default]
    Center,
    Right,
}

/// Vertical anchor of a text box (`Bottom` = the box sits above the anchor).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum VAlign {
    Top,
    #[default]
    Center,
    Bottom,
}

/// Screen-aligned text label anchored at a 3D point, fixed size in points.
///
/// Build with `OverlayText::new(pos, "text")` and the builder methods; add with `Overlay3d::add_text`.
#[derive(Clone, Debug, PartialEq)]
pub struct OverlayText {
    /// Anchor, world space.
    pub pos: [f32; 3],
    /// May contain `\n` (multi-line, lines centred / aligned per `h_align`).
    pub text: String,
    /// Font size in points (egui's proportional font).
    pub size_pt: f32,
    /// sRGB text colour.
    pub color: [f32; 4],
    /// Filled box behind the text (`TEXT_PADDING` around it). classic dimension text: none;
    /// leader text: white.
    pub background: Option<[f32; 4]>,
    /// 1-point frame around the box.
    pub border: Option<[f32; 4]>,
    pub h_align: HAlign,
    pub v_align: VAlign,
    /// Extra offset after alignment, in points (x along the text, y down).
    pub offset_pt: [f32; 2],
    /// World direction the baseline follows on screen (e.g. a dimension line). The text is
    /// flipped as needed to stay readable (never upside down). `None` = horizontal on screen.
    pub along: Option<[f32; 3]>,
    /// Draw over everything (default); `false` = hidden by geometry in
    /// front of the anchor.
    pub on_top: bool,
}

impl OverlayText {
    /// Black 12-point text centred on `pos`, on top, no box.
    pub fn new(pos: DVec3, text: impl Into<String>) -> Self {
        OverlayText {
            pos: f3(pos),
            text: text.into(),
            size_pt: DEFAULT_TEXT_SIZE,
            color: [0.0, 0.0, 0.0, 1.0],
            background: None,
            border: None,
            h_align: HAlign::Center,
            v_align: VAlign::Center,
            offset_pt: [0.0, 0.0],
            along: None,
            on_top: true,
        }
    }
    pub fn color(mut self, c: [f32; 4]) -> Self {
        self.color = c;
        self
    }
    pub fn size(mut self, pt: f32) -> Self {
        self.size_pt = pt;
        self
    }
    pub fn background(mut self, c: [f32; 4]) -> Self {
        self.background = Some(c);
        self
    }
    pub fn border(mut self, c: [f32; 4]) -> Self {
        self.border = Some(c);
        self
    }
    pub fn align(mut self, h: HAlign, v: VAlign) -> Self {
        self.h_align = h;
        self.v_align = v;
        self
    }
    pub fn offset(mut self, x_pt: f32, y_pt: f32) -> Self {
        self.offset_pt = [x_pt, y_pt];
        self
    }
    pub fn along(mut self, dir: DVec3) -> Self {
        self.along = Some(f3(dir));
        self
    }
    pub fn depth_tested(mut self) -> Self {
        self.on_top = false;
        self
    }
}

/// Shape of an `OverlayMark`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MarkShape {
    /// Filled triangle, tip at the point ("closed arrow").
    #[default]
    Arrow,
    /// Two strokes, tip at the point.
    OpenArrow,
    /// Filled disc (diameter ≈ 0.7 × size).
    Dot,
    /// Architectural tick: a stroke at 45° to the line through the point.
    Slash,
    /// "+" cross (centre point marker); arms = size / 2.
    Cross,
    /// "×" cross.
    XCross,
    /// Filled square (side ≈ 0.6 × size).
    Square,
    /// Nothing (e.g. `DimensionStyle::end_mark` without marks).
    None,
}

/// Screen-aligned mark at a 3D point: dimension end marks, arrowheads, centre markers.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OverlayMark {
    pub pos: [f32; 3],
    pub shape: MarkShape,
    /// Size in points (arrow length, cross width, ...).
    pub size_pt: f32,
    /// Stroke width in points (OpenArrow, Slash, Cross, XCross).
    pub width_pt: f32,
    /// Orientation: the mark points away from this world point, i.e. an arrowhead at the end of
    /// the segment `from -> pos`. `None` = screen axes (Cross is "+", Arrow points right).
    pub from: Option<[f32; 3]>,
    /// sRGB.
    pub color: [f32; 4],
    pub on_top: bool,
}

/// Polyline drawn as one stroke: the dash pattern continues across vertices (short arc
/// segments stay dashed), unlike separate `OverlayLine`s whose pattern restarts at every segment.
#[derive(Clone, Debug, PartialEq)]
pub struct OverlayPolyline {
    pub points: Vec<[f32; 3]>,
    pub closed: bool,
    pub color: [f32; 4],
    pub width_px: f32,
    pub dashed: bool,
    pub on_top: bool,
}

/// An orthonormal-ish frame for drawing axes (directions need not be unit length).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AxesFrame {
    pub origin: DVec3,
    pub x: DVec3,
    pub y: DVec3,
    pub z: DVec3,
}

impl AxesFrame {
    pub const WORLD: AxesFrame = AxesFrame { origin: DVec3::ZERO, x: DVec3::X, y: DVec3::Y, z: DVec3::Z };

    /// Frame of a transform (e.g. a group / component instance, or the model's custom axes):
    /// origin = translation, axes = the transformed unit vectors.
    pub fn from_transform(m: &DMat4) -> Self {
        AxesFrame {
            origin: m.transform_point3(DVec3::ZERO),
            x: m.transform_vector3(DVec3::X),
            y: m.transform_vector3(DVec3::Y),
            z: m.transform_vector3(DVec3::Z),
        }
    }
}

impl Default for AxesFrame {
    fn default() -> Self {
        Self::WORLD
    }
}

/// Extra axes drawn as overlay (Axes tool preview, component axes): red/green/blue, solid in the
/// positive direction and dotted in the negative one, like the main axes.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OverlayAxes {
    pub frame: AxesFrame,
    /// Half-length of each axis in world units; `None` = "infinite" like the main axes.
    pub length: Option<f64>,
    pub width_px: f32,
    pub on_top: bool,
}

/// Look of `Overlay3d::add_linear_dimension`.
#[derive(Clone, Debug, PartialEq)]
pub struct DimensionStyle {
    /// Extension lines, dimension line and end marks (sRGB).
    pub color: [f32; 4],
    pub text_color: [f32; 4],
    pub width_px: f32,
    pub end_mark: MarkShape,
    pub mark_size_pt: f32,
    pub text_size_pt: f32,
    /// Defaults: no box. With `text_above = false` a background keeps the line from
    /// crossing the text.
    pub text_background: Option<[f32; 4]>,
    /// Text runs along the dimension line (kept upright) instead of horizontally on screen.
    pub align_to_line: bool,
    /// Text sits above the line (`DIMENSION_TEXT_GAP`) instead of centred on it.
    pub text_above: bool,
    /// Extension lines continue past the dimension line by this fraction of the offset length.
    pub extension_overshoot: f64,
    /// Lines and marks ignore depth (default false: hidden by geometry in front).
    pub lines_on_top: bool,
    /// Text over everything (default true).
    pub text_on_top: bool,
}

impl Default for DimensionStyle {
    fn default() -> Self {
        DimensionStyle {
            color: [0.0, 0.0, 0.0, 1.0],
            text_color: [0.0, 0.0, 0.0, 1.0],
            width_px: 1.0,
            end_mark: MarkShape::Arrow,
            mark_size_pt: DEFAULT_MARK_SIZE,
            text_size_pt: DEFAULT_TEXT_SIZE,
            text_background: None,
            align_to_line: true,
            text_above: true,
            extension_overshoot: 0.12,
            lines_on_top: false,
            text_on_top: true,
        }
    }
}

/// Look of `Overlay3d::add_leader_text` (Text tool: boxed text with a leader).
#[derive(Clone, Debug, PartialEq)]
pub struct LeaderStyle {
    pub color: [f32; 4],
    pub text_color: [f32; 4],
    pub background: Option<[f32; 4]>,
    pub border: Option<[f32; 4]>,
    pub width_px: f32,
    /// Mark at the pointed-at end.
    pub mark: MarkShape,
    pub mark_size_pt: f32,
    pub text_size_pt: f32,
    pub on_top: bool,
}

impl Default for LeaderStyle {
    fn default() -> Self {
        LeaderStyle {
            color: [0.0, 0.0, 0.0, 1.0],
            text_color: [0.0, 0.0, 0.0, 1.0],
            background: Some([1.0, 1.0, 1.0, 1.0]),
            border: Some([0.0, 0.0, 0.0, 1.0]),
            width_px: 1.0,
            mark: MarkShape::Dot,
            mark_size_pt: 7.0,
            text_size_pt: DEFAULT_TEXT_SIZE,
            on_top: true,
        }
    }
}

/// Points of a circular arc around `center` in the plane with normal `normal`, starting in the
/// direction `start_dir` (projected into the plane), sweeping `sweep` radians counter-clockwise
/// about `normal` (negative = clockwise). `segments` >= 1; returns `segments + 1` points
/// (a full circle repeats the first point last). Empty for degenerate input.
pub fn arc_points(center: DVec3, normal: DVec3, start_dir: DVec3, radius: f64, sweep: f64, segments: u32) -> Vec<DVec3> {
    let n = normal.normalize_or_zero();
    let u = (start_dir - n * start_dir.dot(n)).normalize_or_zero();
    if n == DVec3::ZERO || u == DVec3::ZERO || !radius.is_finite() || !sweep.is_finite() {
        return Vec::new();
    }
    let v = n.cross(u);
    let segs = segments.max(1);
    (0..=segs)
        .map(|i| {
            let t = sweep * i as f64 / segs as f64;
            center + (u * t.cos() + v * t.sin()) * radius
        })
        .collect()
}

/// Segment count for a smooth arc preview: ~48 per full turn, at least 4.
pub fn arc_segments(sweep: f64) -> u32 {
    ((sweep.abs() / std::f64::consts::TAU * 48.0).ceil() as u32).clamp(4, 512)
}

impl Overlay3d {
    /// Add a text label (see `OverlayText`).
    pub fn add_text(&mut self, t: OverlayText) {
        if !t.text.is_empty() && t.pos.iter().all(|c| c.is_finite()) {
            self.texts.push(t);
        }
    }

    /// Screen-aligned marker at `p` (centre point "+", dot, square...), `size_pt` points.
    pub fn add_marker(&mut self, p: DVec3, shape: MarkShape, size_pt: f32, color: [f32; 4], on_top: bool) {
        self.add_mark(OverlayMark { pos: f3(p), shape, size_pt, width_pt: 1.5, from: None, color, on_top });
    }

    /// End mark at `at` for the segment `from -> at` (arrowhead pointing at `at`, slash, dot).
    pub fn add_end_mark(&mut self, from: DVec3, at: DVec3, shape: MarkShape, size_pt: f32, color: [f32; 4], on_top: bool) {
        self.add_mark(OverlayMark { pos: f3(at), shape, size_pt, width_pt: 1.0, from: Some(f3(from)), color, on_top });
    }

    pub fn add_mark(&mut self, m: OverlayMark) {
        if m.shape != MarkShape::None && m.pos.iter().all(|c| c.is_finite()) {
            self.marks.push(m);
        }
    }

    /// Line `a -> b` with an arrowhead at `b`.
    pub fn add_arrow(&mut self, a: DVec3, b: DVec3, color: [f32; 4], width_px: f32, on_top: bool) {
        self.lines.push(OverlayLine { a: f3(a), b: f3(b), color, width_px, dashed: false, on_top });
        self.add_end_mark(a, b, MarkShape::Arrow, DEFAULT_MARK_SIZE, color, on_top);
    }

    /// Polyline with a continuous dash pattern (see `OverlayPolyline`).
    pub fn add_polyline(&mut self, points: &[DVec3], closed: bool, color: [f32; 4], width_px: f32, dashed: bool, on_top: bool) {
        if points.len() < 2 {
            return;
        }
        self.polylines.push(OverlayPolyline {
            points: points.iter().map(|&p| f3(p)).collect(),
            closed,
            color,
            width_px,
            dashed,
            on_top,
        });
    }

    /// Arc preview (see `arc_points`), `arc_segments(sweep)` segments, one continuous stroke.
    #[allow(clippy::too_many_arguments)]
    pub fn add_arc(
        &mut self,
        center: DVec3,
        normal: DVec3,
        start_dir: DVec3,
        radius: f64,
        sweep: f64,
        color: [f32; 4],
        width_px: f32,
        dashed: bool,
    ) {
        let pts = arc_points(center, normal, start_dir, radius, sweep, arc_segments(sweep));
        self.add_polyline(&pts, false, color, width_px, dashed, false);
    }

    /// Full circle preview with `segments` sides (the Circle tool's side count).
    #[allow(clippy::too_many_arguments)]
    pub fn add_circle(
        &mut self,
        center: DVec3,
        normal: DVec3,
        radius: f64,
        segments: u32,
        color: [f32; 4],
        width_px: f32,
        dashed: bool,
    ) {
        let n = normal.normalize_or_zero();
        let start = n.any_orthonormal_vector();
        let mut pts = arc_points(center, n, start, radius, std::f64::consts::TAU, segments.max(3));
        pts.pop();
        self.add_polyline(&pts, true, color, width_px, dashed, false);
    }

    /// Protractor-like angle indicator at `center` in the plane with normal `normal`: two radial
    /// strokes (start direction and `start_dir` rotated by `sweep`), the arc between them and a
    /// translucent sector fill. Radius is `radius_pt` points on screen at `center`.
    #[allow(clippy::too_many_arguments)]
    pub fn add_angle_indicator(
        &mut self,
        camera: &Camera,
        viewport: DVec2,
        center: DVec3,
        normal: DVec3,
        start_dir: DVec3,
        sweep: f64,
        radius_pt: f64,
        color: [f32; 4],
    ) {
        let r = camera.pixel_size_at(center, viewport) * radius_pt;
        let pts = arc_points(center, normal, start_dir, r, sweep, arc_segments(sweep));
        let (Some(&first), Some(&last)) = (pts.first(), pts.last()) else { return };
        let line = |a: DVec3, b: DVec3| OverlayLine { a: f3(a), b: f3(b), color, width_px: 1.0, dashed: false, on_top: false };
        self.lines.push(line(center, first));
        self.lines.push(line(center, last));
        self.add_polyline(&pts, false, color, 1.5, false, false);
        let mut tris = Vec::with_capacity((pts.len() - 1) * 3);
        for w in pts.windows(2) {
            tris.extend([f3(center), f3(w[0]), f3(w[1])]);
        }
        self.polygons.push(OverlayPolygon { tris, color: [color[0], color[1], color[2], color[3] * 0.18], on_top: false });
    }

    /// classic linear dimension between `a` and `b`, the dimension line moved by `offset`
    /// (world): extension lines from the points past the dimension line, the dimension line with
    /// end marks, and `text` centred on it (above it / along it per `style`, kept readable).
    pub fn add_linear_dimension(&mut self, a: DVec3, b: DVec3, offset: DVec3, text: impl Into<String>, style: &DimensionStyle) {
        if !(a.is_finite() && b.is_finite() && offset.is_finite()) || a.distance_squared(b) == 0.0 {
            return;
        }
        let (a2, b2) = (a + offset, b + offset);
        let line = |p: DVec3, q: DVec3| OverlayLine {
            a: f3(p),
            b: f3(q),
            color: style.color,
            width_px: style.width_px,
            dashed: false,
            on_top: style.lines_on_top,
        };
        let off_len = offset.length();
        if off_len > 0.0 {
            let over = offset / off_len * (off_len * style.extension_overshoot.max(0.0));
            self.lines.push(line(a, a2 + over));
            self.lines.push(line(b, b2 + over));
        }
        self.lines.push(line(a2, b2));
        for (from, at) in [(b2, a2), (a2, b2)] {
            self.add_mark(OverlayMark {
                pos: f3(at),
                shape: style.end_mark,
                size_pt: style.mark_size_pt,
                width_pt: 1.5,
                from: Some(f3(from)),
                color: style.color,
                on_top: style.lines_on_top,
            });
        }
        let mut t = OverlayText::new((a2 + b2) * 0.5, text).color(style.text_color).size(style.text_size_pt);
        if style.text_above {
            t = t.align(HAlign::Center, VAlign::Bottom).offset(0.0, -DIMENSION_TEXT_GAP);
        }
        if style.align_to_line {
            t = t.along(b - a);
        }
        t.background = style.text_background;
        t.on_top = style.text_on_top;
        self.add_text(t);
    }

    /// Leader text (Text tool): a leader from `label_pos` to `at` with a mark at `at`,
    /// and a boxed `text` centred on `label_pos`.
    pub fn add_leader_text(&mut self, at: DVec3, label_pos: DVec3, text: impl Into<String>, style: &LeaderStyle) {
        if at.distance_squared(label_pos) > 0.0 {
            self.lines.push(OverlayLine {
                a: f3(label_pos),
                b: f3(at),
                color: style.color,
                width_px: style.width_px,
                dashed: false,
                on_top: style.on_top,
            });
            self.add_mark(OverlayMark {
                pos: f3(at),
                shape: style.mark,
                size_pt: style.mark_size_pt,
                width_pt: 1.0,
                from: Some(f3(label_pos)),
                color: style.color,
                on_top: style.on_top,
            });
        }
        let mut t = OverlayText::new(label_pos, text).color(style.text_color).size(style.text_size_pt);
        t.background = style.background;
        t.border = style.border;
        t.on_top = style.on_top;
        self.add_text(t);
    }

    /// Extra axes for a custom frame (see `OverlayAxes`); the main axes are set with
    /// `Renderer::set_axes_frame`.
    pub fn add_axes(&mut self, frame: AxesFrame, length: Option<f64>, on_top: bool) {
        self.axes.push(OverlayAxes { frame, length, width_px: 1.0, on_top });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arc_points_quarter_and_full() {
        let pts = arc_points(DVec3::ZERO, DVec3::Z, DVec3::X, 2.0, std::f64::consts::FRAC_PI_2, 4);
        assert_eq!(pts.len(), 5);
        assert!((pts[0] - DVec3::new(2.0, 0.0, 0.0)).length() < 1e-12);
        assert!((pts[4] - DVec3::new(0.0, 2.0, 0.0)).length() < 1e-12); // CCW about +Z
        assert!(pts.iter().all(|p| (p.length() - 2.0).abs() < 1e-12));
        // start_dir is projected into the plane
        let p = arc_points(DVec3::ZERO, DVec3::Z, DVec3::new(1.0, 0.0, 5.0), 1.0, 1.0, 1);
        assert!(p[0].z.abs() < 1e-12);
        assert!(arc_points(DVec3::ZERO, DVec3::Z, DVec3::Z, 1.0, 1.0, 4).is_empty());
        assert_eq!(arc_segments(std::f64::consts::TAU), 48);
        assert_eq!(arc_segments(0.01), 4);
    }

    #[test]
    fn linear_dimension_parts() {
        let mut o = Overlay3d::default();
        let style = DimensionStyle::default();
        o.add_linear_dimension(DVec3::ZERO, DVec3::new(1000.0, 0.0, 0.0), DVec3::new(0.0, -300.0, 0.0), "1000", &style);
        assert_eq!(o.lines.len(), 3); // 2 extension lines + dimension line
        assert_eq!(o.marks.len(), 2);
        assert_eq!(o.texts.len(), 1);
        let ext = &o.lines[0];
        assert_eq!(ext.a, [0.0, 0.0, 0.0]);
        assert!((ext.b[1] + 300.0 * 1.12).abs() < 1e-3);
        let dim = &o.lines[2];
        assert_eq!((dim.a, dim.b), ([0.0, -300.0, 0.0], [1000.0, -300.0, 0.0]));
        // arrows point outward: tip at each end, coming from the other end
        assert_eq!(o.marks[0].pos, dim.a);
        assert_eq!(o.marks[0].from, Some(dim.b));
        assert_eq!(o.marks[1].pos, dim.b);
        let t = &o.texts[0];
        assert_eq!(t.pos, [500.0, -300.0, 0.0]);
        assert_eq!(t.along, Some([1000.0, 0.0, 0.0]));
        assert!(t.on_top && t.background.is_none() && t.v_align == VAlign::Bottom);
        // zero offset: no extension lines; degenerate: nothing
        let mut o = Overlay3d::default();
        o.add_linear_dimension(DVec3::ZERO, DVec3::X, DVec3::ZERO, "1", &DimensionStyle { end_mark: MarkShape::None, ..style });
        assert_eq!((o.lines.len(), o.marks.len(), o.texts.len()), (1, 0, 1));
        o.add_linear_dimension(DVec3::X, DVec3::X, DVec3::Y, "0", &DimensionStyle::default());
        assert_eq!(o.lines.len(), 1);
    }

    #[test]
    fn leader_and_markers() {
        let mut o = Overlay3d::default();
        o.add_leader_text(DVec3::ZERO, DVec3::new(100.0, 0.0, 100.0), "Note", &LeaderStyle::default());
        assert_eq!((o.lines.len(), o.marks.len(), o.texts.len()), (1, 1, 1));
        assert!(o.texts[0].background.is_some() && o.texts[0].border.is_some());
        o.add_marker(DVec3::ONE, MarkShape::Cross, 8.0, [1.0; 4], true);
        o.add_text(OverlayText::new(DVec3::ZERO, "")); // empty: skipped
        o.add_polyline(&[DVec3::ZERO], false, [1.0; 4], 1.0, true, false); // < 2 points: skipped
        assert_eq!((o.marks.len(), o.texts.len(), o.polylines.len()), (2, 1, 0));
        o.add_circle(DVec3::ZERO, DVec3::Z, 10.0, 24, [1.0; 4], 1.0, true);
        assert_eq!(o.polylines[0].points.len(), 24);
        assert!(o.polylines[0].closed);
        let cam = Camera::default();
        o.add_angle_indicator(&cam, DVec2::new(800.0, 600.0), DVec3::ZERO, DVec3::Z, DVec3::X, 1.0, 40.0, [1.0; 4]);
        assert_eq!(o.polygons.len(), 1);
        let f = AxesFrame::from_transform(&DMat4::from_translation(DVec3::new(1.0, 2.0, 3.0)));
        assert_eq!(f.origin, DVec3::new(1.0, 2.0, 3.0));
        assert_eq!(f.x, DVec3::X);
    }
}
