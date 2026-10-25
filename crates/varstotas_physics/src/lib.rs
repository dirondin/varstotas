//! One-shot physics for preparing still scenes, on [rapier](https://rapier.rs).
//!
//! [`DropSim`]: the chosen groups / components fall under gravity and come to rest on the rest
//! of the visible model (static), optionally on a floor at Z = 0. The simulation never touches
//! the model: each [`DropFrame`] carries the bodies' new transforms, and [`set_transforms`]
//! writes them into the instances.
//!
//! The model is in millimetres; the simulation runs in metres (rapier is tuned for them).

use glam::{DMat4, DQuat, DVec3};
use rapier3d::prelude::*;
use varstotas_kernel::{EntityRef, InstanceId, Model, tessellate::tessellate};

const MM: f64 = 0.001;

/// How a falling body collides.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DropShape {
    /// The convex hull of the body: fast and steady; hollows are filled in.
    #[default]
    Hull,
    /// Convex pieces of the body (V-HACD): a cup can hold things, slower to prepare.
    Pieces,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DropSettings {
    /// m/s², down along -Z.
    pub gravity: f64,
    pub friction: f64,
    /// Bounciness, 0..1.
    pub restitution: f64,
    /// A floor at Z = 0 under everything.
    pub floor: bool,
    pub shape: DropShape,
    /// Simulated seconds at most.
    pub max_seconds: f64,
}

impl Default for DropSettings {
    fn default() -> Self {
        Self { gravity: 9.81, friction: 0.6, restitution: 0.0, floor: true, shape: DropShape::Hull, max_seconds: 10.0 }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DropError {
    /// Nothing to drop: no group or component among the selected entities.
    NoBodies,
    /// A body has no faces to collide with.
    Empty(InstanceId),
}

impl std::fmt::Display for DropError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DropError::NoBodies => write!(f, "no group or component to drop"),
            DropError::Empty(_) => write!(f, "a group or component has no faces"),
        }
    }
}

/// The groups / components of `selection` in the context `path` that may fall: not locked.
pub fn drop_bodies(model: &Model, path: &[InstanceId], selection: &[EntityRef]) -> Vec<InstanceId> {
    let Some(mesh) = model.mesh_at(path) else { return Vec::new() };
    selection
        .iter()
        .filter_map(|e| match e {
            EntityRef::Instance(i) => Some(*i),
            _ => None,
        })
        .filter(|i| mesh.instances.get(*i).is_some_and(|inst| !inst.locked && !inst.hidden))
        .collect()
}

/// A running drop. `Send`: it can be stepped on a worker thread.
pub struct DropSim {
    world: PhysicsWorld,
    bodies: Vec<RigidBodyHandle>,
    /// World transform of the context, and each body's transform in it at the start.
    parent: DMat4,
    start: Vec<DMat4>,
    /// Lowest point of the scene: a body far below it fell into the void.
    floor_z: f64,
    void_z: f64,
    settings: DropSettings,
    time: f64,
}

/// The state after a step.
#[derive(Clone, Debug, PartialEq)]
pub struct DropFrame {
    /// Each body's transform in its context (in the order of the bodies); a body that fell
    /// keeps its start transform.
    pub transforms: Vec<DMat4>,
    /// Bodies with nothing under them.
    pub fell: Vec<bool>,
    pub time: f64,
    /// Every body is at rest (or fell), or the time ran out.
    pub done: bool,
}

impl DropSim {
    /// Build the world: `bodies` of the context `path` fall, every other visible face is static.
    pub fn new(model: &Model, path: &[InstanceId], bodies: &[InstanceId], settings: DropSettings) -> Result<Self, DropError> {
        if bodies.is_empty() {
            return Err(DropError::NoBodies);
        }
        let tess = tessellate(model);
        let mut body_points: Vec<Vec<Vector>> = vec![Vec::new(); bodies.len()];
        let mut body_tris: Vec<Vec<[u32; 3]>> = vec![Vec::new(); bodies.len()];
        let mut fixed_points = Vec::new();
        let mut fixed_tris = Vec::new();
        let mut min_z = f64::INFINITY;
        let (mut lo, mut hi) = (DVec3::splat(f64::INFINITY), DVec3::splat(f64::NEG_INFINITY));
        for face in &tess.faces {
            let owner = owner_body(&face.owner_path, path, bodies);
            for t in face.positions.as_chunks::<3>().0 {
                if (t[1] - t[0]).cross(t[2] - t[0]).length_squared() < 1e-12 {
                    continue;
                }
                let (points, tris) = match owner {
                    Some(b) => (&mut body_points[b], &mut body_tris[b]),
                    None => (&mut fixed_points, &mut fixed_tris),
                };
                let n = points.len() as u32;
                for p in t {
                    points.push(to_m(*p));
                    min_z = min_z.min(p.z);
                    lo = lo.min(*p);
                    hi = hi.max(*p);
                }
                tris.push([n, n + 1, n + 2]);
            }
        }
        if let Some(b) = body_points.iter().position(Vec::is_empty) {
            return Err(DropError::Empty(bodies[b]));
        }
        let size = (hi - lo).max_element();
        let mut world = PhysicsWorld::new();
        world.gravity = Vector::new(0.0, 0.0, -settings.gravity as f32);
        // contact tolerances follow the size of the falling things, not 1 m
        let body_size = body_points
            .iter()
            .map(|p| {
                let (lo, hi) = bounds(p);
                (hi - lo).max_element()
            })
            .fold(f32::MAX, f32::min);
        world.integration_parameters.length_unit = body_size.clamp(0.001, 1.0);
        world.integration_parameters.dt = 1.0 / 120.0;
        let friction = settings.friction as f32;
        let restitution = settings.restitution as f32;
        if !fixed_tris.is_empty()
            && let Ok(c) = ColliderBuilder::trimesh_with_flags(fixed_points, fixed_tris, TriMeshFlags::FIX_INTERNAL_EDGES | TriMeshFlags::MERGE_DUPLICATE_VERTICES)
        {
            world.colliders.insert(c.friction(friction).restitution(restitution));
        }
        if settings.floor {
            let c = ColliderBuilder::new(SharedShape::halfspace(Vector::Z));
            world.colliders.insert(c.friction(friction).restitution(restitution));
        }
        let mut handles = Vec::new();
        for (points, tris) in body_points.into_iter().zip(body_tris) {
            // a body without volume (one face) gets a slightly rounded hull, never pieces
            let has_volume = |c: &ColliderBuilder| c.shape.mass_properties(1.0).mass() > 1e-9;
            let hull = ColliderBuilder::convex_hull(&points).filter(has_volume);
            let builder = match (settings.shape, hull) {
                (DropShape::Pieces, Some(_)) => Some(ColliderBuilder::convex_decomposition(&points, &tris)),
                (_, Some(hull)) => Some(hull),
                (_, None) => ColliderBuilder::round_convex_hull(&points, 0.0005).filter(has_volume),
            };
            // points in one plane have no hull at all: a thin box over them
            let builder = builder.unwrap_or_else(|| {
                let (lo, hi) = bounds(&points);
                let half = ((hi - lo) * 0.5).max(Vector::splat(0.0005));
                ColliderBuilder::cuboid(half.x, half.y, half.z).translation((lo + hi) * 0.5)
            });
            let rb = RigidBodyBuilder::dynamic().ccd_enabled(true).angular_damping(0.2);
            let (h, _) = world.insert(rb, builder.friction(friction).restitution(restitution).density(1000.0));
            handles.push(h);
        }
        let floor_z = if settings.floor { min_z.min(0.0) } else { min_z };
        let parent = model.transform_at(path);
        let start = model
            .mesh_at(path)
            .map(|m| bodies.iter().map(|b| m.instances.get(*b).map_or(DMat4::IDENTITY, |i| i.transform)).collect())
            .unwrap_or_default();
        Ok(Self { world, bodies: handles, parent, start, floor_z, void_z: floor_z - size.max(100.0) * 2.0, settings, time: 0.0 })
    }

    /// Simulated seconds so far.
    pub fn time(&self) -> f64 {
        self.time
    }

    /// Advance by `steps` steps of 1/120 s and report the state.
    pub fn step(&mut self, steps: usize) -> DropFrame {
        for _ in 0..steps {
            if self.finished() {
                break;
            }
            self.world.step();
            self.time += self.world.integration_parameters.dt as f64;
            // a body in the void stops there
            for h in &self.bodies {
                let b = &mut self.world.bodies[*h];
                if (b.position().translation.z as f64) < self.void_z * MM && !b.is_sleeping() {
                    b.set_body_type(RigidBodyType::Fixed, false);
                }
            }
        }
        self.frame()
    }

    fn fell(&self, h: RigidBodyHandle) -> bool {
        let b = &self.world.bodies[h];
        b.is_fixed() || (b.position().translation.z as f64) < self.void_z * MM
    }

    fn finished(&self) -> bool {
        self.time >= self.settings.max_seconds || self.bodies.iter().all(|h| self.world.bodies[*h].is_sleeping() || self.fell(*h))
    }

    pub fn frame(&self) -> DropFrame {
        let inv = self.parent.inverse();
        let fell: Vec<bool> = self.bodies.iter().map(|h| self.fell(*h)).collect();
        let transforms = self
            .bodies
            .iter()
            .zip(&self.start)
            .zip(&fell)
            .map(|((h, start), fell)| {
                if *fell { *start } else { inv * motion(self.world.bodies[*h].position()) * self.parent * *start }
            })
            .collect();
        DropFrame { transforms, fell, time: self.time, done: self.finished() }
    }

    /// The lowest point the bodies can rest on (the floor or the scene's bottom), in mm.
    pub fn floor_z(&self) -> f64 {
        self.floor_z
    }
}

/// Index of the body whose subtree `owner` is in.
fn owner_body(owner: &[InstanceId], path: &[InstanceId], bodies: &[InstanceId]) -> Option<usize> {
    if owner.len() <= path.len() || owner[..path.len()] != *path {
        return None;
    }
    bodies.iter().position(|b| *b == owner[path.len()])
}

fn bounds(points: &[Vector]) -> (Vector, Vector) {
    points.iter().fold((Vector::splat(f32::MAX), Vector::splat(f32::MIN)), |(a, b), v| (a.min(*v), b.max(*v)))
}

fn to_m(p: DVec3) -> Vector {
    Vector::new((p.x * MM) as f32, (p.y * MM) as f32, (p.z * MM) as f32)
}

/// A rapier pose (metres) as a world motion in mm.
fn motion(p: &Pose) -> DMat4 {
    let r = p.rotation;
    let q = DQuat::from_xyzw(r.x as f64, r.y as f64, r.z as f64, r.w as f64).normalize();
    let t = DVec3::new(p.translation.x as f64, p.translation.y as f64, p.translation.z as f64) / MM;
    DMat4::from_rotation_translation(q, t)
}

/// Set the transforms of `bodies` of the context `path` (from a [`DropFrame`]).
pub fn set_transforms(model: &mut Model, path: &[InstanceId], bodies: &[InstanceId], transforms: &[DMat4]) {
    let Some(mesh) = model.mesh_at_mut(path) else { return };
    for (id, t) in bodies.iter().zip(transforms) {
        if let Some(inst) = mesh.instances.get_mut(*id) {
            inst.transform = *t;
        }
    }
}

#[cfg(test)]
mod tests;
