//! Icon contact sheet: tessellates every icon with egui and rasterizes it on the CPU, so stroke
//! joins can be audited without a window. `VARSTOTAS_ICON_SHEET=<out.png> cargo test -p
//! varstotas_app icon_sheet -- --ignored` writes the sheet.

use super::{Icon, paint_icon};
use egui::{Color32, Pos2, Rect, epaint::Primitive, vec2};
use varstotas_kernel::SolidOp;

pub(super) fn all_icons() -> Vec<Icon> {
    use Icon::*;
    let mut v = vec![
        Select, Eraser, Paint, Line, Arc, ArcCenter, Arc3, Pie, Bezier, EditCurve, ModelInfo, SmoothCurve,
        Freehand, Rectangle, RotatedRectangle, Circle, Polygon, PushPull, Move, Rotate, Scale, Offset, FollowMe,
        Tape, Protractor, Axes, Dimension, Section, Slice, Orbit, Pan, Zoom, ZoomExtents, PositionCamera,
        LookAround, Walk, Shadows, Text, Text3d, Chamfer, Fillet, Solid, ViewIso, ViewTop, ViewFront, ViewRight,
        ViewBack, ViewLeft, ViewBottom, StyleXRay, StyleWireframe, StyleHiddenLine, StyleShaded, StyleTextured,
        StyleMonochrome, StyleRealistic, Perspective, Parallel, StyleEdges, StyleProfiles, Guides, Ground, Sky,
        Grid, SectionCut, SectionFill, Panes1, Panes2, Panes4, Undo, Redo, Eye, EyeOff, Lock, Unlock, Plus, Cloud,
        More, Trash, Refresh, Info, Group, Magnet, GuidePlane, Box3d, Cylinder, Cone, Pyramid, Sphere, Torus,
        FileNew, FileOpen, FileSave, FileSaveAs, Film, Drawing, Sheets, Report, Json, Credits, Script, Quit, Cut,
        Copy, Paste, PasteInPlace, Purge, DeleteGuides, SelectAll, SelectNone, Component, Parametric, Explode,
        CloseGroup, ReverseFaces, OrientFaces, Unfold, Physics, UvEditor, TexturePosition, Print3d, StartScreen, SideBar, Outliner, Tag, Render, Console, Log, Gear,
        Generator, Play, Pause, NextScene, PrevScene, AddScene,
    ];
    v.extend((0..4).map(Picture));
    v.extend([Import("OBJ"), Export("STL")]);
    v.extend((0..3).map(Intersect));
    v.extend((0..3).map(Flip));
    v.extend(super::uv::all());
    v.extend((0..SolidOp::ALL.len() as u8).map(Solids));
    use crate::inference::InferenceKind as K;
    v.extend(
        [K::Endpoint, K::Midpoint, K::Center, K::Intersection, K::OnEdge, K::OnFace, K::ParallelToEdge,
         K::PerpendicularToEdge, K::GuidePoint, K::OnGuide, K::Origin, K::Tangent, K::GridPoint]
            .map(Snap),
    );
    v
}

/// Renders `icons` in a grid of `cell`-point squares at `ppp` pixels per point.
pub(super) fn render(icons: &[Icon], cell: f32, ppp: f32, cols: usize) -> image::RgbaImage {
    let rows = icons.len().div_ceil(cols);
    let size = vec2(cols as f32 * cell, rows as f32 * cell);
    let ctx = egui::Context::default();
    ctx.set_pixels_per_point(ppp);
    let input = egui::RawInput {
        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, size)),
        ..Default::default()
    };
    let run = |paint: bool| {
        ctx.run_ui(input.clone(), |ui| {
            let ctx = ui.ctx();
            if !paint {
                return;
            }
            let painter = ctx.layer_painter(egui::LayerId::background());
            for (i, icon) in icons.iter().enumerate() {
                let min = Pos2::new((i % cols) as f32 * cell, (i / cols) as f32 * cell);
                let r = Rect::from_min_size(min, vec2(cell, cell)).shrink(cell * 0.1);
                paint_icon(&painter, r, *icon, false);
            }
        })
    };
    // the first frame only builds the font atlas
    let mut first = run(false);
    let mut out = run(true);
    let mut font: Option<egui::ColorImage> = None;
    for set in [std::mem::take(&mut first.textures_delta.set), std::mem::take(&mut out.textures_delta.set)] {
        for d in set.into_iter().filter(|(id, _)| *id == egui::TextureId::default()).flat_map(|(_, d)| d) {
            let egui::ImageData::Color(c) = d.image;
            match (d.pos, font.as_mut()) {
                (Some([x0, y0]), Some(f)) => {
                    for y in 0..c.size[1] {
                        for x in 0..c.size[0] {
                            f.pixels[(y0 + y) * f.size[0] + x0 + x] = c.pixels[y * c.size[0] + x];
                        }
                    }
                }
                _ => font = Some((*c).clone()),
            }
        }
    }
    let font = font.expect("font atlas");
    let (w, h) = ((size.x * ppp) as u32, (size.y * ppp) as u32);
    let mut img = image::RgbaImage::from_pixel(w, h, image::Rgba([0xF3, 0xF4, 0xF6, 255]));
    for prim in ctx.tessellate(out.shapes, ppp) {
        let Primitive::Mesh(mesh) = prim.primitive else { continue };
        for t in mesh.indices.as_chunks::<3>().0 {
            let v = [0, 1, 2].map(|k| mesh.vertices[t[k] as usize]);
            raster(&mut img, &v, ppp, &font);
        }
    }
    img
}

fn raster(img: &mut image::RgbaImage, v: &[egui::epaint::Vertex; 3], ppp: f32, font: &egui::ColorImage) {
    let p = v.map(|v| (v.pos.x * ppp, v.pos.y * ppp));
    let area = (p[1].0 - p[0].0) * (p[2].1 - p[0].1) - (p[2].0 - p[0].0) * (p[1].1 - p[0].1);
    if area.abs() < 1e-9 {
        return;
    }
    let (x0, x1) = (p.iter().map(|q| q.0).fold(f32::MAX, f32::min), p.iter().map(|q| q.0).fold(f32::MIN, f32::max));
    let (y0, y1) = (p.iter().map(|q| q.1).fold(f32::MAX, f32::min), p.iter().map(|q| q.1).fold(f32::MIN, f32::max));
    for y in (y0.floor().max(0.0) as u32)..(y1.ceil().min(img.height() as f32) as u32) {
        for x in (x0.floor().max(0.0) as u32)..(x1.ceil().min(img.width() as f32) as u32) {
            let (px, py) = (x as f32 + 0.5, y as f32 + 0.5);
            let e = |a: (f32, f32), b: (f32, f32)| ((b.0 - a.0) * (py - a.1) - (b.1 - a.1) * (px - a.0)) / area;
            let w = [e(p[1], p[2]), e(p[2], p[0]), e(p[0], p[1])];
            if w.iter().any(|w| *w < 0.0) {
                continue;
            }
            let mut c = [0f32; 4];
            let (mut u, mut vv) = (0.0, 0.0);
            for k in 0..3 {
                let col = v[k].color.to_array();
                for (ch, x) in c.iter_mut().zip(col) {
                    *ch += w[k] * x as f32 / 255.0;
                }
                u += w[k] * v[k].uv.x;
                vv += w[k] * v[k].uv.y;
            }
            let tx = ((u * font.size[0] as f32) as usize).min(font.size[0] - 1);
            let ty = ((vv * font.size[1] as f32) as usize).min(font.size[1] - 1);
            let tex: Color32 = font.pixels[ty * font.size[0] + tx];
            let tex = tex.to_array().map(|x| x as f32 / 255.0);
            let src: Vec<f32> = c.iter().zip(tex).map(|(a, b)| a * b).collect();
            let dst = img.get_pixel_mut(x, y);
            for ch in 0..3 {
                let d = dst[ch] as f32 / 255.0;
                dst[ch] = ((src[ch] + d * (1.0 - src[3])).clamp(0.0, 1.0) * 255.0).round() as u8;
            }
        }
    }
}

#[test]
#[ignore = "writes a PNG; run on demand with VARSTOTAS_ICON_SHEET"]
fn icon_sheet() {
    let Ok(out) = std::env::var("VARSTOTAS_ICON_SHEET") else { return };
    let icons = all_icons();
    render(&icons, 32.0, 2.0, 16).save(&out).unwrap();
    // a magnified pass makes the joins easy to inspect
    render(&icons, 32.0, 6.0, 16).save(out.replace(".png", "-x6.png")).unwrap();
}
