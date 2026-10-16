use varstotas_kernel::glam::DVec3;
use varstotas_kernel::{Model, WorkPlane};

fn close(a: DVec3, b: DVec3) -> bool {
    a.distance(b) < 1e-9
}

#[test]
fn planes_from_points_normal_and_angles() {
    // Three points on a wall facing -Y: red along X, normal -Y... right-hand: (X) × (Z) = -Y.
    let p = WorkPlane::from_points("", DVec3::ZERO, DVec3::new(100.0, 0.0, 0.0), DVec3::new(0.0, 0.0, 50.0)).unwrap();
    assert!(close(p.x, DVec3::X) && close(p.normal(), -DVec3::Y) && close(p.y, DVec3::Z));
    assert!(WorkPlane::from_points("", DVec3::ZERO, DVec3::X, DVec3::X * 2.0).is_none());
    let (tilt, az, spin) = p.angles();
    assert_eq!((tilt, az, spin), (90.0, -90.0, 0.0), "a front wall keeps red level");
    let mut q = WorkPlane::from_normal("", DVec3::ZERO, DVec3::Z, None).unwrap();
    assert_eq!(q.angles(), (0.0, 0.0, 0.0));
    q.set_angles(tilt, az, spin);
    assert!(close(q.x, p.x) && close(q.y, p.y));
    // Tilted 30° toward red, turned 45°.
    q.set_angles(30.0, 0.0, 45.0);
    let (t, a, s) = q.angles();
    assert!((t - 30.0).abs() < 1e-9 && a.abs() < 1e-9 && (s - 45.0).abs() < 1e-9);
    assert!((q.normal().dot(DVec3::Z) - 30f64.to_radians().cos()).abs() < 1e-12);
    q.spin_by(-45.0);
    assert!(q.angles().2.abs() < 1e-9);
    // A new normal keeps red as close as possible.
    assert!(q.set_normal(DVec3::Y));
    assert!(close(q.normal(), DVec3::Y) && q.x.y.abs() < 1e-12);
}

#[test]
fn active_plane_is_the_drawing_frame_and_saved() {
    let mut m = Model::new();
    let wall = WorkPlane::from_normal("", DVec3::new(0.0, 500.0, 0.0), DVec3::Y, Some(DVec3::X)).unwrap();
    assert_eq!(m.add_work_plane(wall.clone()), 0);
    assert_eq!(m.work_planes[0].name, "Plane 1");
    let f = m.drawing_frame();
    assert!(close(f.translation, DVec3::new(0.0, 500.0, 0.0)) && close(f.matrix3.z_axis, DVec3::Y));
    m.add_work_plane(WorkPlane::from_normal("Roof", DVec3::Z * 3000.0, DVec3::Z, None).unwrap());
    assert_eq!(m.active_plane, Some(1));
    let json = varstotas_kernel::io::to_json(&m).unwrap();
    let back = varstotas_kernel::io::from_json(&json).unwrap();
    assert_eq!(back.work_planes.len(), 2);
    assert_eq!(back.active_plane, Some(1));
    m.remove_work_plane(0);
    assert_eq!(m.active_plane, Some(0));
    m.remove_work_plane(0);
    assert_eq!(m.active_plane, None);
    assert!(close(m.drawing_frame().translation, DVec3::ZERO));
}
