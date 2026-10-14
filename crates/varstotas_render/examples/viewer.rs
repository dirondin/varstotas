//! Standalone viewer for `varstotas_render`.
//!
//! cargo run -p varstotas_render --example viewer
//!
//! Left drag = orbit, right drag / Shift+left drag = pan, wheel = zoom to cursor.
//! Env `VARSTOTAS_SCREENSHOT=out.png` saves a window screenshot after a few frames and quits.
//! `VARSTOTAS_STRESS=<faces>` (>= 100) adds a grid of boxes, `=1` a 1M-triangle height field; stress mode
//! orbits every frame without vsync and logs the average frame time. `VARSTOTAS_STRESS_GLASS=1` makes
//! every 8th box translucent. `VARSTOTAS_RENDER_TIMING=1` logs renderer CPU/GPU timings.
//! `VARSTOTAS_STYLE=mono|hidden|wire|xray|noedges|noprofiles|noground` (comma list), `VARSTOTAS_ZOOM=<f>`,
//! `VARSTOTAS_OFFSET=<mm>` (move the model far from the world origin via `SceneGeometry::origin`).
//! `VARSTOTAS_EXPORT=<dir>`: after a few frames, export the view with `Renderer::render_to_image` at
//! 1x and 4x the viewport pixel size, opaque and transparent (guides only, no selection, no tool
//! overlays), write `<dir>/export_{1x,4x}_{opaque,transparent}.png` and quit.
//! `VARSTOTAS_GUIDES=<n>` adds n extra guide lines (stress); a few demo guides are always shown.

use eframe::egui;
use glam::{DVec2, DVec3, Vec3};
use varstotas_render::{
    Camera, ExportOptions, FaceStyle, Highlight, HighlightUpdate, LineVertex, Overlay3d, OverlayLine, OverlayPolygon,
    RenderStyle, Renderer, RgbaImage, SceneGeometry, StandardView, TriVertex,
};

fn main() -> eframe::Result {
    env_logger::init();
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_title("varstotas_render viewer"),
        renderer: eframe::Renderer::Wgpu,
        // Stress mode measures real frame time: no vsync cap.
        wgpu_options: eframe::egui_wgpu::WgpuConfiguration {
            surface: eframe::egui_wgpu::SurfaceConfig {
                present_mode: if std::env::var("VARSTOTAS_STRESS").is_ok() {
                    varstotas_render::wgpu::PresentMode::AutoNoVsync
                } else {
                    varstotas_render::wgpu::PresentMode::AutoVsync
                },
                ..eframe::egui_wgpu::SurfaceConfig::LOW_LATENCY
            },
            ..Default::default()
        },
        ..Default::default()
    };
    eframe::run_native(
        "varstotas_render viewer",
        options,
        Box::new(|cc| Ok(Box::new(Viewer::new(cc)))),
    )
}

// ---------------------------------------------------------------------------
// Scene building
// ---------------------------------------------------------------------------

const BACK: [f32; 4] = varstotas_render::DEFAULT_BACK_COLOR;
const BLACK: [f32; 4] = [0.0, 0.0, 0.0, 1.0];

#[derive(Default)]
struct Builder {
    geo: SceneGeometry,
    /// Highlight id of the next quad (one id per quad = one "face").
    faces: u32,
}

impl Builder {
    /// Quad with corners in CCW order seen from the front side.
    fn quad(&mut self, p: [Vec3; 4], color: [f32; 4], back: [f32; 4]) {
        let n = (p[1] - p[0]).cross(p[2] - p[0]).normalize();
        for i in [0, 1, 2, 0, 2, 3] {
            self.geo.tris.push(TriVertex {
                pos: p[i].into(),
                normal: n.into(),
                color,
            });
            self.geo.back_colors.push(back);
        }
        // two triangles, one face id
        self.geo.face_ids.extend([self.faces; 2]);
        self.faces += 1;
    }

    fn edge(&mut self, a: Vec3, b: Vec3, color: [f32; 4]) {
        self.geo.edges.push(LineVertex { pos: a.into(), color });
        self.geo.edges.push(LineVertex { pos: b.into(), color });
    }

    /// Profile over the edge pushed last (shares its highlight id).
    fn profile(&mut self, a: Vec3, b: Vec3) {
        self.geo.profile_ids.push((self.geo.edges.len() / 2) as u32 - 1);
        self.geo.profile_edges.push(LineVertex { pos: a.into(), color: BLACK });
        self.geo.profile_edges.push(LineVertex { pos: b.into(), color: BLACK });
    }

    fn cuboid(&mut self, min: Vec3, max: Vec3, color: [f32; 4], profiles: bool) {
        // translucent materials get a translucent back side too
        let back = if color[3] < 1.0 { color } else { BACK };
        let c = |i: usize| {
            Vec3::new(
                if i & 1 == 0 { min.x } else { max.x },
                if i & 2 == 0 { min.y } else { max.y },
                if i & 4 == 0 { min.z } else { max.z },
            )
        };
        // faces, CCW from outside
        let faces = [
            [0, 2, 3, 1], // bottom -Z
            [4, 5, 7, 6], // top +Z
            [0, 1, 5, 4], // front -Y
            [2, 6, 7, 3], // back +Y
            [0, 4, 6, 2], // left -X
            [1, 3, 7, 5], // right +X
        ];
        for f in faces {
            self.quad([c(f[0]), c(f[1]), c(f[2]), c(f[3])], color, back);
        }
        let edges = [
            (0, 1), (2, 3), (4, 5), (6, 7), (0, 2), (1, 3),
            (4, 6), (5, 7), (0, 4), (1, 5), (2, 6), (3, 7),
        ];
        for (a, b) in edges {
            self.edge(c(a), c(b), BLACK);
            if profiles {
                self.profile(c(a), c(b));
            }
        }
    }

    fn finish(mut self) -> SceneGeometry {
        let r = self
            .geo
            .tris
            .iter()
            .map(|v| Vec3::from(v.pos).length())
            .fold(0.0f32, f32::max);
        self.geo.radius = r;
        self.geo
    }
}

fn build_scene() -> (SceneGeometry, DVec3, DVec3) {
    let mut b = Builder::default();
    b.cuboid(Vec3::new(0.0, 0.0, 0.0), Vec3::new(2000.0, 1500.0, 1200.0), [1.0; 4], false);
    b.cuboid(
        Vec3::new(2500.0, 500.0, 0.0),
        Vec3::new(3300.0, 1300.0, 2400.0),
        [0.88, 0.69, 0.44, 1.0],
        true,
    );
    // Partially below ground: lower part is tinted by the ground plane.
    b.cuboid(
        Vec3::new(-1500.0, 1500.0, -600.0),
        Vec3::new(-500.0, 2500.0, 800.0),
        [0.55, 0.8, 0.55, 1.0],
        false,
    );
    // Translucent glass box.
    b.cuboid(
        Vec3::new(500.0, -1500.0, 0.0),
        Vec3::new(1500.0, -700.0, 1000.0),
        [0.45, 0.65, 1.0, 0.35],
        false,
    );
    // A single face whose front side points away from the default camera (+Y):
    // the back color is visible.
    let f = [
        Vec3::new(-1500.0, -800.0, 0.0),
        Vec3::new(-1500.0, -800.0, 1500.0),
        Vec3::new(-200.0, -800.0, 1500.0),
        Vec3::new(-200.0, -800.0, 0.0),
    ];
    b.quad(f, [1.0; 4], BACK);
    for i in 0..4 {
        b.edge(f[i], f[(i + 1) % 4], BLACK);
    }
    // a selected (blue) edge
    b.edge(
        Vec3::new(0.0, 0.0, 1200.0),
        Vec3::new(0.0, 1500.0, 1200.0),
        [0.0, 0.31, 1.0, 1.0],
    );
    // Painted floor strips at z = 0 (must not z-fight with the ground plane).
    for (k, col) in [[0.5, 0.5, 0.5, 1.0], [0.8, 0.1, 0.1, 1.0], [0.8, 0.6, 0.4, 1.0]].into_iter().enumerate() {
        let x0 = -4500.0 + 1000.0 * k as f32;
        let q = [
            Vec3::new(x0, 2800.0, 0.0),
            Vec3::new(x0 + 1000.0, 2800.0, 0.0),
            Vec3::new(x0 + 1000.0, 3800.0, 0.0),
            Vec3::new(x0, 3800.0, 0.0),
        ];
        b.quad(q, col, BACK);
        for i in 0..4 {
            b.edge(q[i], q[(i + 1) % 4], BLACK);
        }
    }
    // Painted gable-roof shed: grey (0.3, 0.3, 0.35) like QA-009.
    {
        let c = [0.3, 0.3, 0.35, 1.0];
        let (x0, x1, y0, y1, zr) = (-4500.0, -2500.0, 4200.0, 5400.0, 900.0);
        let ym = (y0 + y1) * 0.5;
        let (a0, a1) = (Vec3::new(x0, y0, 0.0), Vec3::new(x1, y0, 0.0));
        let (b0, b1) = (Vec3::new(x0, y1, 0.0), Vec3::new(x1, y1, 0.0));
        let (r0, r1) = (Vec3::new(x0, ym, zr), Vec3::new(x1, ym, zr));
        b.quad([a0, a1, r1, r0], c, BACK);
        b.quad([b1, b0, r0, r1], c, BACK);
        for (p, q) in [(a0, r0), (b0, r0), (a1, r1), (b1, r1), (r0, r1), (a0, a1), (b0, b1)] {
            b.edge(p, q, BLACK);
        }
    }
    // Gable-roof house: both roof slopes must shade differently.
    {
        let (x0, x1, y0, y1, zw, zr) = (3800.0, 6000.0, -3000.0, -1500.0, 1500.0, 2400.0);
        b.cuboid(Vec3::new(x0, y0, 0.0), Vec3::new(x1, y1, zw), [1.0; 4], false);
        let ym = (y0 + y1) * 0.5;
        let (a0, a1) = (Vec3::new(x0, y0, zw), Vec3::new(x1, y0, zw));
        let (b0, b1) = (Vec3::new(x0, y1, zw), Vec3::new(x1, y1, zw));
        let (r0, r1) = (Vec3::new(x0, ym, zr), Vec3::new(x1, ym, zr));
        b.quad([a0, a1, r1, r0], [1.0; 4], BACK); // -Y slope
        b.quad([b1, b0, r0, r1], [1.0; 4], BACK); // +Y slope
        b.quad([a0, r0, b0, b0], [1.0; 4], BACK); // -X gable (degenerate quad = triangle)
        b.quad([a1, b1, r1, r1], [1.0; 4], BACK); // +X gable
        for (p, q) in [(a0, r0), (b0, r0), (a1, r1), (b1, r1), (r0, r1)] {
            b.edge(p, q, BLACK);
        }
    }
    let stress = std::env::var("VARSTOTAS_STRESS").unwrap_or_default();
    let stress_faces: usize = stress.trim().parse().unwrap_or(if stress.is_empty() { 0 } else { 1 });
    if stress_faces >= 100 {
        // `VARSTOTAS_STRESS=<faces>`: a grid of boxes (6 faces, 12 tris, 12 edges + 12 profiles each).
        // `VARSTOTAS_STRESS_GLASS=1`: every 8th box is translucent (exercises the sorted blend pass).
        let glass = std::env::var("VARSTOTAS_STRESS_GLASS").is_ok_and(|v| v == "1");
        let boxes = stress_faces.div_ceil(6);
        let side = (boxes as f64).sqrt().ceil() as usize;
        let (pitch, w) = (1000.0f32, 600.0f32);
        let x0 = -2000.0 - side as f32 * pitch; // left of the demo scene
        for k in 0..boxes {
            let (i, j) = (k % side, k / side);
            let hgt = 300.0 + ((i * 7 + j * 13) % 17) as f32 * 120.0;
            let min = Vec3::new(x0 + i as f32 * pitch, j as f32 * pitch - side as f32 * pitch * 0.5, 0.0);
            let col = if glass && k % 8 == 0 {
                [0.45, 0.65, 1.0, 0.4]
            } else {
                [0.6 + (i % 5) as f32 * 0.08, 0.7, 0.6 + (j % 5) as f32 * 0.08, 1.0]
            };
            b.cuboid(min, min + Vec3::new(w, w, hgt), col, true);
        }
        eprintln!(
            "stress: {} boxes, {} faces, {} triangles, {} edges",
            boxes,
            boxes * 6,
            b.geo.tris.len() / 3,
            b.geo.edges.len() / 2
        );
    } else if stress_faces > 0 {
        // `VARSTOTAS_STRESS=1`: ~1M triangles: a wavy height-field 708 x 708 cells (2 tris each).
        let n = 708;
        let size = 20000.0f32;
        let h = |i: usize, j: usize| {
            let (x, y) = (i as f32 / n as f32, j as f32 / n as f32);
            300.0 * ((x * 40.0).sin() * (y * 37.0).cos()) + 400.0
        };
        let p = |i: usize, j: usize| {
            Vec3::new(
                -size - 2000.0 + size * i as f32 / n as f32,
                size * j as f32 / n as f32 - size / 2.0,
                h(i, j),
            )
        };
        for i in 0..n {
            for j in 0..n {
                b.quad([p(i, j), p(i + 1, j), p(i + 1, j + 1), p(i, j + 1)], [0.7, 0.8, 0.95, 1.0], BACK);
            }
        }
        eprintln!("stress: {} triangles", b.geo.tris.len() / 3);
    }
    let (mut min, mut max) = (Vec3::splat(f32::MAX), Vec3::splat(f32::MIN));
    for v in &b.geo.tris {
        min = min.min(Vec3::from(v.pos));
        max = max.max(Vec3::from(v.pos));
    }
    (b.finish(), min.as_dvec3(), max.as_dvec3())
}

// ---------------------------------------------------------------------------
// App
// ---------------------------------------------------------------------------

struct Viewer {
    renderer: Renderer,
    camera: Camera,
    style: RenderStyle,
    bbox: (DVec3, DVec3),
    drag_point: Option<DVec3>,
    frame_no: u64,
    screenshot: Option<String>,
    screenshot_sent: bool,
    fps_window: Option<std::time::Instant>,
    /// World offset of the demo model (`VARSTOTAS_OFFSET`).
    offset: DVec3,
    /// Stress mode: the scene, re-uploaded every 180 frames to time `set_geometry` (buffer reuse).
    stress_geo: Option<SceneGeometry>,
    /// (face ids, edge ids) for the stress-mode selection toggling.
    highlight_counts: (usize, usize),
    /// `VARSTOTAS_EXPORT` output directory.
    export_dir: Option<String>,
}

/// Demo guides (world space, shifted by `offset`): a few lines (one selected, one passing right
/// by the default eye so it is clipped at the near plane), two guide points, plus
/// `VARSTOTAS_GUIDES=<n>` extra lines.
fn guides(overlay: &mut Overlay3d, camera: &Camera, viewport: DVec2, offset: DVec3) {
    let o = |x: f64, y: f64, z: f64| DVec3::new(x, y, z) + offset;
    overlay.add_guide_line(camera, o(0.0, -500.0, 0.0), DVec3::X, false);
    overlay.add_guide_line(camera, o(3300.0, 0.0, 2400.0), DVec3::Y, true);
    overlay.add_guide_line(camera, o(-1000.0, 0.0, 0.0), DVec3::new(1.0, 1.0, 0.0), false);
    overlay.add_guide_line(camera, camera.eye + DVec3::new(300.0, 0.0, -300.0), DVec3::new(0.3, 1.0, -0.2), false);
    overlay.add_guide_point(camera, viewport, o(2000.0, 1500.0, 2400.0), false);
    overlay.add_guide_point(camera, viewport, o(3300.0, 500.0, 2400.0), true);
    let n: usize = std::env::var("VARSTOTAS_GUIDES").ok().and_then(|v| v.parse().ok()).unwrap_or(0);
    for i in 0..n {
        let a = i as f64 * 0.37;
        overlay.add_guide_line(camera, o(-8000.0 + 40.0 * i as f64, 0.0, 0.0), DVec3::new(a.cos(), a.sin(), 0.05), i % 5 == 0);
    }
}

impl Viewer {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let rs = cc.wgpu_render_state.as_ref().expect("wgpu backend required");
        let mut renderer = Renderer::new(rs);
        let (mut geo, mut min, mut max) = build_scene();
        // `VARSTOTAS_OFFSET=<mm>`: move the whole model far along +X (e.g. 5000000 = 5 km) via
        // `SceneGeometry::origin`, to check relative-to-eye precision.
        let offset = std::env::var("VARSTOTAS_OFFSET").ok().and_then(|o| o.parse::<f64>().ok()).unwrap_or(0.0);
        let off = DVec3::new(offset, 0.0, 0.0);
        geo.origin = off.to_array();
        min += off;
        max += off;
        renderer.set_geometry(rs, &geo);
        let highlight_faces = geo.face_ids.last().map_or(0, |&f| f as usize + 1);
        let highlight_edges = geo.edges.len() / 2;
        if std::env::var("VARSTOTAS_HIGHLIGHT").is_ok() {
            // Demo: white box dimmed, brown pillar selected (faces 0..6 / 6..12, edges 0..12 / 12..24).
            let mut f = vec![Highlight::None; highlight_faces];
            let mut e = vec![Highlight::None; highlight_edges];
            f[..6].fill(Highlight::Dimmed);
            e[..12].fill(Highlight::Dimmed);
            f[6..12].fill(Highlight::Selected);
            e[12..24].fill(Highlight::Selected);
            renderer.update_highlight(rs, &HighlightUpdate { faces: &f, edges: &e });
        }

        // Env knobs for automated screenshots.
        let env = |k: &str| std::env::var(k).unwrap_or_default().to_lowercase();
        let mut camera = Camera::default();
        camera.eye += off;
        camera.target += off;
        let mut style = RenderStyle::default();
        let view = match env("VARSTOTAS_VIEW").as_str() {
            "top" => Some(StandardView::Top),
            "front" => Some(StandardView::Front),
            "right" => Some(StandardView::Right),
            "back" => Some(StandardView::Back),
            "left" => Some(StandardView::Left),
            "bottom" => Some(StandardView::Bottom),
            _ => None,
        };
        if let Some(v) = view {
            camera.set_standard_view(v);
        }
        if env("VARSTOTAS_ORTHO") == "1" {
            camera.set_ortho(true);
        }
        if env("VARSTOTAS_EXTENTS") == "1" || std::env::var("VARSTOTAS_STRESS").is_ok() {
            camera.zoom_extents(min, max, 1.5);
        }
        if let Some(z) = std::env::var("VARSTOTAS_ZOOM").ok().and_then(|z| z.parse::<f64>().ok()) {
            camera.zoom(z, camera.target);
        }
        if let Ok(o) = std::env::var("VARSTOTAS_ORBIT") {
            let v: Vec<f64> = o.split(',').filter_map(|x| x.trim().parse().ok()).collect();
            if v.len() == 2 {
                camera.orbit(DVec2::new(v[0], v[1]), camera.target);
            }
        }
        for s in env("VARSTOTAS_STYLE").split(',') {
            match s {
                "mono" => style.face_style = FaceStyle::Monochrome,
                "hidden" => style.face_style = FaceStyle::HiddenLine,
                "wire" => style.face_style = FaceStyle::Wireframe,
                "xray" => style.xray = true,
                "noedges" => style.show_edges = false,
                "noprofiles" => style.profiles = false,
                "noground" => style.show_ground = false,
                _ => {}
            }
        }
        Viewer {
            renderer,
            camera,
            style,
            bbox: (min, max),
            offset: off,
            highlight_counts: (highlight_faces, highlight_edges),
            stress_geo: std::env::var("VARSTOTAS_STRESS").is_ok().then_some(geo),
            drag_point: None,
            frame_no: 0,
            screenshot: std::env::var("VARSTOTAS_SCREENSHOT").ok(),
            screenshot_sent: false,
            fps_window: None,
            export_dir: std::env::var("VARSTOTAS_EXPORT").ok(),
        }
    }

    /// `VARSTOTAS_EXPORT`: write 1x / 4x, opaque / transparent images of the current view.
    fn export(&mut self, rs: &eframe::egui_wgpu::RenderState, dir: &str, size_px: [u32; 2], ppp: f32) {
        std::fs::create_dir_all(dir).expect("create export dir");
        let vp = DVec2::new(size_px[0] as f64 / ppp as f64, size_px[1] as f64 / ppp as f64);
        let mut overlay = Overlay3d::default(); // guides only: no tool previews / selection boxes
        guides(&mut overlay, &self.camera, vp, self.offset);
        for k in [1u32, 4] {
            for transparent in [false, true] {
                let opts = ExportOptions {
                    transparent_background: transparent,
                    line_scale: ppp * k as f32,
                    ..Default::default()
                };
                let size = [size_px[0] * k, size_px[1] * k];
                let t0 = std::time::Instant::now();
                let img = self
                    .renderer
                    .render_to_image(rs, &self.camera, &overlay, &self.style, size, &opts)
                    .expect("render_to_image");
                let t_render = t0.elapsed();
                let name = format!("{dir}/export_{k}x_{}.png", if transparent { "transparent" } else { "opaque" });
                img.write_png(&name).expect("write png");
                eprintln!(
                    "exported {name}: {}x{} render {:.0} ms, total {:.0} ms",
                    img.width,
                    img.height,
                    t_render.as_secs_f64() * 1000.0,
                    t0.elapsed().as_secs_f64() * 1000.0
                );
            }
        }
        let too_big = self.renderer.render_to_image(
            rs,
            &self.camera,
            &overlay,
            &self.style,
            [varstotas_render::MAX_EXPORT_DIM + 1, 10],
            &ExportOptions::default(),
        );
        eprintln!("oversized export: {}", too_big.unwrap_err());
    }

    /// Point under the cursor: ray ∩ ground plane, else at target depth.
    fn point_under(&self, px: DVec2, vp: DVec2) -> DVec3 {
        let (o, d) = self.camera.screen_ray(px, vp);
        if d.z.abs() > 1e-9 {
            let t = -o.z / d.z;
            if t > 0.0 && t < self.camera.distance() * 20.0 {
                return o + d * t;
            }
        }
        let depth = (self.camera.target - o).dot(self.camera.basis().forward);
        o + d * (depth / d.dot(self.camera.basis().forward)).max(1.0)
    }

    fn toolbar(&mut self, ui: &mut egui::Ui, aspect: f64) {
        ui.horizontal_wrapped(|ui| {
            for (v, name) in [
                (StandardView::Iso, "Iso"),
                (StandardView::Top, "Top"),
                (StandardView::Front, "Front"),
                (StandardView::Right, "Right"),
                (StandardView::Back, "Back"),
                (StandardView::Left, "Left"),
                (StandardView::Bottom, "Bottom"),
            ] {
                if ui.button(name).clicked() {
                    self.camera.set_standard_view(v);
                }
            }
            if ui.button("Zoom extents").clicked() {
                self.camera.zoom_extents(self.bbox.0, self.bbox.1, aspect);
            }
            let mut ortho = self.camera.ortho;
            if ui.checkbox(&mut ortho, "Parallel").changed() {
                self.camera.set_ortho(ortho);
            }
            ui.separator();
            for (s, name) in [
                (FaceStyle::Shaded, "Shaded"),
                (FaceStyle::Monochrome, "Mono"),
                (FaceStyle::HiddenLine, "Hidden line"),
                (FaceStyle::Wireframe, "Wireframe"),
            ] {
                ui.radio_value(&mut self.style.face_style, s, name);
            }
            ui.checkbox(&mut self.style.xray, "X-ray");
            ui.checkbox(&mut self.style.show_edges, "Edges");
            ui.checkbox(&mut self.style.profiles, "Profiles");
            ui.checkbox(&mut self.style.show_axes, "Axes");
            ui.checkbox(&mut self.style.show_ground, "Ground");
        });
    }
}

impl eframe::App for Viewer {
    fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        self.frame_no += 1;
        if std::env::var("VARSTOTAS_STRESS").is_ok() {
            let now = std::time::Instant::now();
            let start = *self.fps_window.get_or_insert(now);
            if self.frame_no.is_multiple_of(120) {
                let ms = now.duration_since(start).as_secs_f64() * 1000.0 / 120.0;
                eprintln!("frame {} avg frame time {:.2} ms ({:.0} fps)", self.frame_no, ms, 1000.0 / ms);
                self.fps_window = Some(now);
            }
        }
        if std::env::var("VARSTOTAS_STRESS").is_ok() {
            self.camera.orbit(DVec2::new(1.0, 0.0), self.camera.target);
            ui.ctx().request_repaint();
        }
        let Some(rs) = frame.wgpu_render_state() else {
            return;
        };
        if let Some(geo) = &self.stress_geo
            && self.frame_no.is_multiple_of(180)
        {
            self.renderer.set_geometry(rs, geo);
        }
        if std::env::var("VARSTOTAS_STRESS").is_ok() && self.frame_no.is_multiple_of(60) {
            // Selection fast path: select / deselect every other face and edge.
            let on = self.frame_no.is_multiple_of(120);
            let (nf, ne) = self.highlight_counts;
            let pick = |i: usize| if on && i.is_multiple_of(2) { Highlight::Selected } else { Highlight::None };
            let f: Vec<Highlight> = (0..nf).map(pick).collect();
            let e: Vec<Highlight> = (0..ne).map(pick).collect();
            self.renderer.update_highlight(rs, &HighlightUpdate { faces: &f, edges: &e });
        }
        let aspect_hint = {
            let r = ui.available_rect_before_wrap();
            (r.width() / r.height().max(1.0)) as f64
        };
        egui::Panel::top("toolbar").show(ui, |ui| self.toolbar(ui, aspect_hint));

        egui::CentralPanel::default()
            .frame(egui::Frame::NONE)
            .show(ui, |ui| {
                let rect = ui.available_rect_before_wrap();
                let response = ui.allocate_rect(rect, egui::Sense::click_and_drag());
                let vp = DVec2::new(rect.width() as f64, rect.height() as f64);
                let to_local =
                    |p: egui::Pos2| DVec2::new((p.x - rect.min.x) as f64, (p.y - rect.min.y) as f64);

                // --- navigation ---
                let (shift, scroll, hover) = ui.input(|i| {
                    (i.modifiers.shift, i.smooth_scroll_delta.y, i.pointer.hover_pos())
                });
                if response.drag_started()
                    && let Some(p) = response.interact_pointer_pos()
                {
                    self.drag_point = Some(self.point_under(to_local(p), vp));
                }
                if response.dragged() {
                    let d = response.drag_delta();
                    let delta = DVec2::new(d.x as f64, d.y as f64);
                    let pan = response.dragged_by(egui::PointerButton::Secondary)
                        || response.dragged_by(egui::PointerButton::Middle) && shift
                        || response.dragged_by(egui::PointerButton::Primary) && shift;
                    let p = self.drag_point.unwrap_or(self.camera.target);
                    if pan {
                        self.camera.pan(delta, p, vp);
                    } else {
                        self.camera.orbit(delta, p);
                    }
                }
                if response.drag_stopped() {
                    self.drag_point = None;
                }
                if response.hovered()
                    && scroll != 0.0
                    && let Some(h) = hover
                {
                    let p = self.point_under(to_local(h), vp);
                    self.camera.zoom((scroll as f64 * 0.003).exp(), p);
                }

                // --- overlay demo ---
                let overlay = Overlay3d {
                    lines: vec![
                        OverlayLine {
                            a: [2000.0, 1500.0, 1200.0],
                            b: [2000.0, 1500.0, 3200.0],
                            color: [0.0, 0.0, 1.0, 1.0],
                            width_px: 2.0,
                            dashed: true,
                            on_top: true,
                        },
                        OverlayLine {
                            a: [0.0, 0.0, 1200.0],
                            b: [2000.0, 1500.0, 1200.0],
                            color: [1.0, 0.0, 1.0, 1.0],
                            width_px: 4.0,
                            dashed: false,
                            on_top: false,
                        },
                    ],
                    polygons: vec![OverlayPolygon {
                        tris: vec![
                            [2000.0, 0.0, 1200.0],
                            [2000.0, 1500.0, 1200.0],
                            [0.0, 1500.0, 1200.0],
                        ],
                        color: [1.0, 0.85, 0.1, 0.35],
                        on_top: false,
                    }],
                };

                let mut overlay = overlay;
                if self.offset != DVec3::ZERO {
                    let sh = |p: &mut [f32; 3]| *p = (DVec3::from(p.map(|x| x as f64)) + self.offset).as_vec3().to_array();
                    for l in &mut overlay.lines {
                        sh(&mut l.a);
                        sh(&mut l.b);
                    }
                    for p in &mut overlay.polygons {
                        p.tris.iter_mut().for_each(sh);
                    }
                }

                if std::env::var("VARSTOTAS_HIGHLIGHT").is_ok() {
                    // Selected "group": classic bounding box around the green cube.
                    let o = self.offset.as_vec3();
                    let p = |v: [f32; 3]| (Vec3::from(v) + o).to_array();
                    overlay.add_aabb(
                        p([-1500.0, 1500.0, -600.0]),
                        p([-500.0, 2500.0, 800.0]),
                        varstotas_render::HIGHLIGHT_SELECTED,
                        1.0,
                    );
                }

                guides(&mut overlay, &self.camera, vp, self.offset);

                // --- render ---
                let ppp = ui.ctx().pixels_per_point();
                let size_px = [
                    (rect.width() * ppp).round().max(1.0) as u32,
                    (rect.height() * ppp).round().max(1.0) as u32,
                ];
                let tex = self
                    .renderer
                    .render(rs, &self.camera, &overlay, &self.style, size_px, ppp);
                if let Some(dir) = self.export_dir.clone()
                    && self.frame_no == 5
                {
                    self.export(rs, &dir, size_px, ppp);
                    ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                }
                if self.export_dir.is_some() {
                    ui.ctx().request_repaint();
                }
                ui.painter().image(
                    tex,
                    rect,
                    egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                    egui::Color32::WHITE,
                );

                // 2D marker example: project the origin
                if let Some(s) = self.camera.world_to_screen(self.offset, vp) {
                    let p = rect.min + egui::vec2(s.x as f32, s.y as f32);
                    ui.painter().circle_stroke(p, 5.0, (2.0, egui::Color32::from_rgb(0, 192, 0)));
                }
            });

        // --- automated screenshot ---
        if let Some(path) = self.screenshot.clone() {
            if self.frame_no == 10 && !self.screenshot_sent {
                ui.ctx()
                    .send_viewport_cmd(egui::ViewportCommand::Screenshot(Default::default()));
                self.screenshot_sent = true;
            }
            let shot = ui.input(|i| {
                i.raw.events.iter().find_map(|e| match e {
                    egui::Event::Screenshot { image, .. } => Some(image.clone()),
                    _ => None,
                })
            });
            if let Some(img) = shot {
                let img = RgbaImage { width: img.size[0] as u32, height: img.size[1] as u32, pixels: img.as_raw().to_vec() };
                img.write_png(&path).expect("write png");
                eprintln!("screenshot saved to {path}");
                ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
            }
            ui.ctx().request_repaint();
        }
    }
}
