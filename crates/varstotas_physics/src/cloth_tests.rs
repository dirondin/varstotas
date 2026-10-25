use super::cloth::*;
use super::tests_util::*;
use varstotas_kernel::glam::{DMat4, DVec3};
use varstotas_kernel::{EntityRef, Model};

fn sheet(m: &mut Model, lo: DVec3, size: f64) -> varstotas_kernel::FaceId {
    let pts = [lo, lo + DVec3::X * size, lo + DVec3::new(size, size, 0.0), lo + DVec3::Y * size];
    m.root_mesh_mut().add_face(&pts).unwrap()
}

fn run(m: &Model, src: &ClothSource, s: ClothSettings) -> (ClothSim, ClothFrame) {
    let mut sim = ClothSim::new(m, src, s).unwrap();
    loop {
        let f = sim.step(30);
        if f.done {
            return (sim, f);
        }
    }
}

fn zs(f: &ClothFrame) -> (f64, f64) {
    f.positions.iter().fold((f64::MAX, f64::MIN), |(a, b), p| (a.min(p.z), b.max(p.z)))
}

#[test]
fn a_rectangle_makes_a_regular_grid() {
    let mut m = Model::new();
    let f = sheet(&mut m, DVec3::new(0.0, 0.0, 100.0), 100.0);
    let src = cloth_source(&m, &[], &[EntityRef::Face(f)]).unwrap();
    let g = cloth_grid(&src, 10.0).unwrap();
    assert_eq!(g.positions.len(), 11 * 11);
    assert_eq!(g.triangles.len(), 10 * 10 * 2);
    assert!(g.positions.iter().all(|p| (p.z - 100.0).abs() < 1e-9));
    assert_eq!(cloth_grid(&src, 0.5).err(), Some(ClothError::TooFine(201 * 201)));
    assert_eq!(cloth_grid(&src, 200.0).err(), Some(ClothError::TooSmall));
    // nothing or two faces: no source
    assert!(cloth_source(&m, &[], &[]).is_none());
}

#[test]
fn a_cloth_drapes_over_a_box() {
    let mut m = Model::new();
    add_box(&mut m, DVec3::new(100.0, 100.0, 0.0), DVec3::splat(100.0), false);
    let f = sheet(&mut m, DVec3::new(0.0, 0.0, 150.0), 300.0);
    let src = cloth_source(&m, &[], &[EntityRef::Face(f)]).unwrap();
    let s = ClothSettings { spacing: 15.0, ..ClothSettings::default() };
    let (sim, fr) = run(&m, &src, s);
    let (lo, hi) = zs(&fr);
    eprintln!("cloth at rest after {:.2} s: z {lo:.1}..{hi:.1}", fr.time);
    // it lies on the top of the box and hangs down its sides
    assert!(hi > 99.0 && hi < 110.0, "top {hi}");
    assert!(lo < 40.0, "hangs down: {lo}");
    assert!(lo > -1.0, "stays above the floor: {lo}");
    // a mesh of it, smooth
    let mesh = cloth_mesh(&sim.grid, &fr.positions, DMat4::IDENTITY);
    assert_eq!(mesh.faces.len(), sim.grid.triangles.len());
    assert!(mesh.edges.values().filter(|e| e.soft && e.smooth).count() > mesh.edges.len() / 2);
}

#[test]
fn a_cloth_in_a_group_ignores_its_own_face() {
    let mut m = Model::new();
    let f = sheet(&mut m, DVec3::new(0.0, 0.0, 50.0), 100.0);
    let mesh = m.root_mesh();
    let mut ents = vec![EntityRef::Face(f)];
    ents.extend(mesh.face_edges(f).into_iter().map(EntityRef::Edge));
    let g = m.make_group(&[], &ents).unwrap();
    let src = cloth_source(&m, &[], &[EntityRef::Instance(g)]).unwrap();
    assert_eq!(src.group, Some(g));
    let (_, fr) = run(&m, &src, ClothSettings { spacing: 10.0, ..ClothSettings::default() });
    let (lo, hi) = zs(&fr);
    assert!(lo > -1.0 && hi < 5.0, "flat on the floor: {lo}..{hi}");
}

#[test]
fn placing_a_cloth_replaces_its_source() {
    let mut m = Model::new();
    // the top of a box: its edges stay with the box's sides
    add_box(&mut m, DVec3::ZERO, DVec3::splat(100.0), false);
    let top = m.root_mesh().faces.iter().find(|(_, f)| f.normal.z > 0.5).map(|(id, _)| id).unwrap();
    let edges = m.root_mesh().edges.len();
    let src = cloth_source(&m, &[], &[EntityRef::Face(top)]).unwrap();
    let grid = cloth_grid(&src, 20.0).unwrap();
    let id = place_cloth(&mut m, &[], &src, cloth_mesh(&grid, &grid.positions, DMat4::IDENTITY)).unwrap();
    assert_eq!(m.root_mesh().faces.len(), 5);
    assert_eq!(m.root_mesh().edges.len(), edges);
    let def = m.root_mesh().instances[id].def;
    assert_eq!(m.defs[def].mesh.faces.len(), grid.triangles.len());
    // a loose sheet goes with its edges
    let mut m = Model::new();
    let f = sheet(&mut m, DVec3::ZERO, 100.0);
    let src = cloth_source(&m, &[], &[EntityRef::Face(f)]).unwrap();
    let grid = cloth_grid(&src, 20.0).unwrap();
    let id = place_cloth(&mut m, &[], &src, cloth_mesh(&grid, &grid.positions, DMat4::IDENTITY)).unwrap();
    assert!(m.root_mesh().faces.is_empty() && m.root_mesh().edges.is_empty());
    let moved: Vec<DVec3> = grid.positions.iter().map(|p| *p + DVec3::Z * 5.0).collect();
    set_cloth_mesh(&mut m, &[], id, cloth_mesh(&grid, &moved, DMat4::IDENTITY));
    let b = m.instance_world_bounds(&[], id).unwrap();
    assert!((b.min.z - 5.0).abs() < 1e-9, "{b:?}");
}
