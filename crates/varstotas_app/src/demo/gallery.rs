//! File → Open Demo: Material Gallery — two dozen bevelled cubes, each in its own material
//! (metals, glass, emissive, textured, custom shaders, plastics), on a polished floor in front
//! of a studio cove, and scenes that light them in different environments and render styles
//! (studio, noon, sunset, neon night, fog, clay, watercolour, a technical plan).

use super::{Builder, face_out, prism, rnd, textures, v};
use crate::shaders;
use varstotas_kernel::glam::{DMat4, DQuat, DVec3};
use varstotas_kernel::{
    CustomShader, EFFECT_PRESETS, Environment, LengthUnit, Mesh, Model, Pbr, RenderSettings, Scene, SceneSettings, SceneStyle,
    ShadowSettings, SkyKind, Stylize, Tonemap,
};
use std::collections::BTreeMap;

/// Cube edge and bevel.
const CUBE: f64 = 700.0;
const BEVEL: f64 = 28.0;
/// Grid pitch: 6 columns × 4 rows.
const PITCH: f64 = 1500.0;
const COLS: usize = 6;

/// The materials in grid order (front row first, left to right).
const ORDER: [&str; 24] = [
    "Gold", "Clear Glass", "Red Car Paint", "Marble", "Neon Pink", "Chrome",
    "Walnut", "Copper", "Water Shader", "Ruby Glass", "Glossy Ceramic", "Brick",
    "Frosted Glass", "Velvet", "Brushed Aluminium", "Glow Shader", "Wood Shader", "Blue Titanium",
    "Stone", "Neon Cyan", "Yellow Plastic", "Reeded Glass", "Rubber", "Subway Tile",
];

/// The demo model.
pub fn material_gallery() -> Model {
    let mut m = Model::new();
    m.unit = LengthUnit::Millimeter;
    let cubes = m.add_tag("Cubes");
    let studio = m.add_tag("Studio");
    let cove_tag = m.add_tag("Cove");
    let mut b = Builder { m, mats: BTreeMap::new(), roof_tag: studio, furniture_tag: cubes, garden_tag: studio };
    materials(&mut b);
    let root = b.m.root;
    let stage = b.group(root, "Studio", Some(studio));
    b.solid(stage, "Floor", Some("Showroom Floor"), |m| super::cuboid(m, v(-150_000.0, -150_000.0, -100.0), v(150_000.0, 150_000.0, 0.0)));
    let wall = b.group(stage, "Cove", Some(cove_tag));
    b.solid(wall, "Backdrop", Some("Backdrop"), cove);
    let grid = b.group(root, "Cubes", Some(cubes));
    for (i, name) in ORDER.iter().enumerate() {
        let (col, row) = ((i % COLS) as f64, (i / COLS) as f64);
        let at = v((col - (COLS as f64 - 1.0) / 2.0) * PITCH, (row - 1.5) * PITCH, 0.0);
        let deg = (rnd(7, i as u32) - 0.5) * 50.0;
        cube(&mut b, grid, name, at, deg);
    }
    settings(&mut b.m);
    let mut m = b.m;
    m.rebuild_caches();
    m
}

/// A cube of edge `CUBE` with bevelled edges and corners, standing on `z = 0` around the origin.
fn bevelled_cube(mesh: &mut Mesh) {
    let (h, a) = (CUBE / 2.0, CUBE / 2.0 - BEVEL);
    let c = v(0.0, 0.0, h);
    // A point by axis order: `p(i, x_i, j, x_j, x_k)` with k the remaining axis.
    let p = |i: usize, xi: f64, j: usize, xj: f64, xk: f64| {
        let mut q = [xk; 3];
        q[i] = xi;
        q[j] = xj;
        c + v(q[0], q[1], q[2])
    };
    let signs = [-1.0, 1.0];
    for i in 0..3 {
        let j = (i + 1) % 3;
        for s in signs {
            let face = [p(i, s * h, j, -a, -a), p(i, s * h, j, a, -a), p(i, s * h, j, a, a), p(i, s * h, j, -a, a)];
            face_out(mesh, &face, c);
            for t in signs {
                // The bevel between the faces across axes i and j.
                let edge = [p(i, s * h, j, t * a, -a), p(i, s * h, j, t * a, a), p(i, s * a, j, t * h, a), p(i, s * a, j, t * h, -a)];
                face_out(mesh, &edge, c);
            }
        }
    }
    for sx in signs {
        for sy in signs {
            for sz in signs {
                let corner = [c + v(sx * h, sy * a, sz * a), c + v(sx * a, sy * h, sz * a), c + v(sx * a, sy * a, sz * h)];
                face_out(mesh, &corner, c);
            }
        }
    }
}

fn cube(b: &mut Builder, parent: varstotas_kernel::DefId, mat: &str, at: DVec3, deg: f64) {
    let d = b.def(mat, true);
    bevelled_cube(&mut b.m.defs[d].mesh);
    let xf = DMat4::from_rotation_translation(DQuat::from_rotation_z(deg.to_radians()), at);
    let id = b.mat(mat);
    b.instance(parent, d, xf, mat, Some(id), None);
}

/// The studio cove behind the cubes: the floor curves up into a back wall.
fn cove(mesh: &mut Mesh) {
    let (y0, r, top, t, x) = (3800.0, 2600.0, 6500.0, 120.0, 13_000.0);
    let n = 12;
    let arc = |rad: f64| (0..=n).map(move |i| {
        let a = i as f64 / n as f64 * std::f64::consts::FRAC_PI_2;
        v(-x, y0 + rad * a.sin(), r - rad * a.cos())
    });
    let mut pts: Vec<DVec3> = arc(r).collect();
    pts.push(v(-x, y0 + r, top));
    pts.push(v(-x, y0 + r + t, top));
    pts.extend(arc(r + t).collect::<Vec<_>>().into_iter().rev().filter(|p| p.z >= -t));
    pts.push(v(-x, y0, -t));
    prism(mesh, &pts, &[], v(2.0 * x, 0.0, 0.0));
}

fn materials(b: &mut Builder) {
    use textures as t;
    let p = |roughness: f32| Pbr { roughness, ..Pbr::default() };
    let metal = |roughness: f32| Pbr { metallic: 1.0, roughness, ..Pbr::default() };
    let glass = |roughness: f32| Pbr { transmission: 1.0, ior: 1.5, roughness, ..Pbr::default() };
    let glow = |c: [f32; 3]| Pbr { emission: c, emission_strength: 10.0, ..p(0.4) };
    let shader = |src: &str, base: Pbr| Pbr { shader: Some(CustomShader::new(src)), ..base };

    b.add_mat("Showroom Floor", [0.16, 0.16, 0.17], p(0.1));
    b.add_mat("Backdrop", [0.78, 0.78, 0.77], p(0.85));

    b.add_mat("Gold", [1.0, 0.77, 0.34], metal(0.14));
    b.add_mat("Copper", [0.95, 0.62, 0.52], metal(0.28));
    b.add_mat("Chrome", [0.92, 0.92, 0.94], metal(0.03));
    b.add_mat("Brushed Aluminium", [0.90, 0.91, 0.92], metal(0.38));
    b.add_mat("Blue Titanium", [0.30, 0.38, 0.62], metal(0.2));

    b.add_mat("Clear Glass", [0.96, 0.98, 0.98], glass(0.0));
    b.add_mat("Ruby Glass", [0.85, 0.08, 0.14], glass(0.0));
    b.add_mat("Frosted Glass", [0.90, 0.95, 1.0], glass(0.35));
    b.add_mat("Reeded Glass", [0.9, 0.96, 0.93], shader(shaders::GLASS, glass(0.02)));

    b.add_mat("Neon Pink", [1.0, 0.25, 0.6], glow([1.0, 0.18, 0.55]));
    b.add_mat("Neon Cyan", [0.2, 0.9, 1.0], glow([0.15, 0.85, 1.0]));
    b.add_mat("Glow Shader", [0.9, 0.5, 0.2], shader(shaders::GLOW, p(0.4)));

    b.add_mat("Red Car Paint", [0.62, 0.02, 0.04], p(0.06));
    b.add_mat("Glossy Ceramic", [0.93, 0.91, 0.87], p(0.08));
    b.add_mat("Yellow Plastic", [0.95, 0.68, 0.05], p(0.32));
    b.add_mat("Rubber", [0.035, 0.035, 0.04], p(0.92));

    b.add_mat("Water Shader", [0.1, 0.35, 0.45], shader(shaders::WATER, p(0.04)));
    b.add_mat("Wood Shader", [0.7, 0.5, 0.3], shader(shaders::WOOD, p(0.5)));

    b.add_tex("Marble", t::marble(), p(0.08));
    b.add_tex("Walnut", t::wood("gallery_walnut", [0.42, 0.27, 0.16], [0.22, 0.13, 0.07]), p(0.35));
    b.add_tex("Brick", t::brick(), p(0.9));
    b.add_tex("Stone", t::stone(), p(0.85));
    b.add_tex("Velvet", t::fabric("gallery_velvet", [0.08, 0.32, 0.22]), p(0.95));
    b.add_tex("Subway Tile", t::subway(), p(0.1));
}

fn settings(m: &mut Model) {
    let mut fx = varstotas_kernel::Effects::default();
    if let Some(p) = fx.with_preset(EFFECT_PRESETS[2]) {
        fx = p;
    }
    let studio = Environment {
        sky: SkyKind::Color,
        color: [0.80, 0.80, 0.82],
        intensity: 1.0,
        background: true,
        sun: 1.6,
        tonemap: Tonemap::Agx,
        effects: fx,
        ..Environment::default()
    };
    m.environment = studio;
    let day = ShadowSettings { enabled: true, day: varstotas_kernel::shadows::day_of_year(6, 20), minutes: 15 * 60, ..ShadowSettings::default() };
    m.shadows = day;
    m.render = RenderSettings { width: 1280, height: 720, samples: 192, bounces: 8, denoise: true, match_view: true, ..RenderSettings::default() };
    m.scene_settings = SceneSettings { transitions: true, transition_secs: 2.5, delay_secs: 1.5 };

    let noon = Environment { sky: SkyKind::Physical, turbidity: 2.2, sun: 3.0, exposure: -0.2, ..studio };
    let sunset = Environment { sky: SkyKind::Physical, turbidity: 6.0, sun: 3.0, exposure: 1.4, ..studio };
    let mut night = Environment { sky: SkyKind::Color, color: [0.02, 0.03, 0.065], intensity: 1.0, sun: 0.0, exposure: 1.5, ..studio };
    night.effects.bloom = true;
    night.effects.bloom_strength = 0.25;
    night.effects.bloom_threshold = 1.2;
    night.effects.vignette = 0.35;
    let mut fog = Environment { sky: SkyKind::Physical, turbidity: 9.0, sun: 2.0, exposure: 0.6, ..studio };
    fog.effects.fog = true;
    fog.effects.fog_distance = 40_000.0;
    fog.effects.fog_color = [0.78, 0.8, 0.84];
    fog.effects.dof = true;
    fog.effects.focus_distance = 3200.0;
    fog.effects.aperture = 0.8;
    let mut clay = studio;
    clay.effects.clay = true;
    clay.effects.ao = true;
    let mut water = Environment { color: [0.93, 0.92, 0.88], ..studio };
    water.effects.stylize = Stylize::Watercolor;
    let mut plan = Environment { color: [1.0, 1.0, 1.0], ..studio };
    plan.effects.stylize = Stylize::Technical;

    let style = Some(SceneStyle { face_style: "realistic".into(), xray: false, edges: false, profiles: false });
    let cove = m.tags.iter().find(|(_, t)| t.name == "Cove").map(|(id, _)| id);
    let mut scene = |name: &str, eye: DVec3, target: DVec3, fov: f64, env: Environment, minutes: u32| {
        let mut s = Scene::camera(name, eye, target, DVec3::Z, fov, false);
        s.style = style.clone();
        s.environment = Some(env);
        s.shadows = Some(ShadowSettings { minutes, ..day });
        // Outdoors (a physical sky) the floor reaches the horizon.
        let outdoor = env.sky == SkyKind::Physical;
        s.hidden_tags = Some(cove.filter(|_| outdoor).into_iter().collect());
        m.scenes.push(s);
    };
    scene("Studio", v(3600.0, -7600.0, 2700.0), v(-200.0, 300.0, 250.0), 40.0, studio, 15 * 60);
    scene("Noon", v(-6500.0, -8200.0, 1500.0), v(300.0, 200.0, 350.0), 42.0, noon, 13 * 60);
    scene("Sunset", v(5600.0, -5200.0, 1100.0), v(-1500.0, 1500.0, 600.0), 44.0, sunset, 20 * 60 + 50);
    scene("Neon Night", v(-1800.0, -5200.0, 900.0), v(800.0, 300.0, 350.0), 46.0, night, 15 * 60);
    scene("Foggy Morning", v(-4000.0, -5600.0, 650.0), v(-1500.0, -1600.0, 380.0), 40.0, fog, 6 * 60 + 30);
    scene("Clay", v(3600.0, -7600.0, 2700.0), v(-200.0, 300.0, 250.0), 40.0, clay, 15 * 60);
    scene("Watercolor", v(-5200.0, -7800.0, 2800.0), v(0.0, 0.0, 300.0), 42.0, water, 11 * 60);
    let mut top = Scene::camera("Plan", v(0.0, 0.0, 30_000.0), v(0.0, 0.0, 0.0), DVec3::Y, 22.0, false);
    top.style = style;
    top.environment = Some(plan);
    top.shadows = Some(ShadowSettings { minutes: 12 * 60, ..day });
    top.hidden_tags = Some(cove.into_iter().collect());
    top.in_animation = false;
    m.scenes.push(top);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_gallery_is_complete() {
        let m = material_gallery();
        assert_eq!(m.materials.len(), ORDER.len() + 2);
        assert_eq!(m.scenes.len(), 8);
        assert!(m.materials.values().filter(|x| x.pbr.as_ref().is_some_and(|p| p.shader.is_some())).count() == 4);
        // Every cube is a closed bevelled box (6 faces, 12 bevels, 8 corners) of its material.
        let cubes: Vec<_> = m.defs.values().filter(|d| ORDER.contains(&d.name.as_str())).collect();
        assert_eq!(cubes.len(), 24);
        for d in cubes {
            assert_eq!(d.mesh.faces.len(), 26, "{}", d.name);
            assert!(d.mesh.edges.keys().all(|e| d.mesh.edge_faces(e).len() == 2), "{} is not closed", d.name);
        }
        let cove = m.defs.values().find(|d| d.name == "Backdrop").unwrap();
        assert!(cove.mesh.faces.len() > 20);
        let dir = std::env::temp_dir().join(format!("varstotas_gallery_{}.vrst", std::process::id()));
        varstotas_kernel::io::save(&m, &dir).unwrap();
        let back = varstotas_kernel::io::load(&dir).unwrap();
        let _ = std::fs::remove_file(&dir);
        assert_eq!(back.scenes.len(), 8);
    }
}
