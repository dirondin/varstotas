use std::collections::HashSet;
use varstotas_kernel::glam::DVec3;
use varstotas_kernel::{EntityRef, InstanceId, Model};

/// A box `size` with its corner at `at`, as a group of the root (or loose when `group` is false).
pub(crate) fn add_box(m: &mut Model, at: DVec3, size: DVec3, group: bool) -> Option<InstanceId> {
    let before: HashSet<EntityRef> = all(m).into_iter().collect();
    let mesh = m.root_mesh_mut();
    let pts = [at, at + DVec3::new(size.x, 0.0, 0.0), at + DVec3::new(size.x, size.y, 0.0), at + DVec3::new(0.0, size.y, 0.0)];
    let f = mesh.add_face(&pts).unwrap();
    let up = mesh.faces[f].normal.z >= 0.0;
    mesh.push_pull(f, if up { size.z } else { -size.z }, false).unwrap();
    if !group {
        return None;
    }
    let new: Vec<_> = all(m).into_iter().filter(|e| !before.contains(e)).collect();
    Some(m.make_group(&[], &new).unwrap())
}

pub(crate) fn all(m: &Model) -> Vec<EntityRef> {
    let mesh = m.root_mesh();
    mesh.faces.keys().map(EntityRef::Face).chain(mesh.edges.keys().map(EntityRef::Edge)).collect()
}

