//! The demo models: ordinary `.vrst` files in `examples/demos/`, embedded at build time and
//! installed (refreshed on every start) to `~/.varstotas/examples`, where File → Open finds them.
//! - Country House: a furnished one-storey house on a lawn (walls with openings, a gable roof,
//!   rooms, a terrace and a garden), PBR materials with textures, lights, render settings and
//!   scenes (day, dusk, rooms, plan).
//! - Material Gallery: 24 bevelled cubes in different materials on a studio cove; its materials
//!   are also the Examples folder of the materials library.

use std::path::PathBuf;
use varstotas_kernel::Model;

const COUNTRY_HOUSE: &str = include_str!("../../../examples/demos/country_house.vrst");
const MATERIAL_GALLERY: &str = include_str!("../../../examples/demos/material_gallery.vrst");
const FILES: [(&str, &str); 2] = [("country_house.vrst", COUNTRY_HOUSE), ("material_gallery.vrst", MATERIAL_GALLERY)];

/// `~/.varstotas/examples`.
pub fn examples_dir() -> Option<PathBuf> {
    crate::bridge::home_dir().map(|d| d.join("examples"))
}

/// Write the demo models to `~/.varstotas/examples` (replacing changed or older copies).
pub fn install_examples() {
    let Some(dir) = examples_dir() else { return };
    let r = std::fs::create_dir_all(&dir).and_then(|_| {
        for (name, text) in FILES {
            let p = dir.join(name);
            if std::fs::read(&p).ok().as_deref() != Some(text.as_bytes()) {
                std::fs::write(p, text)?;
            }
        }
        Ok(())
    });
    if let Err(e) = r {
        log::warn!("demos: cannot install the examples to {}: {e}", dir.display());
    }
}

fn load(text: &str) -> Model {
    varstotas_kernel::io::from_json(text).expect("embedded demo model")
}

/// The Material Gallery model (its materials fill the library's Examples folder).
pub fn material_gallery() -> Model {
    load(MATERIAL_GALLERY)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_demo_house_is_complete() {
        let m = load(COUNTRY_HOUSE);
        assert!(m.materials.len() > 40);
        assert!(m.materials.values().filter(|x| x.image.is_some()).count() >= 18);
        assert_eq!(m.scenes.len(), 8);
        assert!(m.lights.len() >= 12);
        // Every group has geometry; the front wall is a solid with its openings.
        for (_, d) in &m.defs {
            if d.is_group && d.mesh.instances.is_empty() {
                assert!(!d.mesh.faces.is_empty(), "empty group {}", d.name);
            }
        }
        let front = m.defs.values().find(|d| d.name == "Front Wall").unwrap();
        let holes: usize = front.mesh.faces.values().map(|f| f.holes.len()).sum();
        assert_eq!(holes, 4, "two windows through two faces (the door notches the outline)");
    }

    #[test]
    fn the_gallery_is_complete() {
        let m = material_gallery();
        assert_eq!(m.materials.len(), 26);
        assert_eq!(m.scenes.len(), 8);
        assert_eq!(m.materials.values().filter(|x| x.pbr.as_ref().is_some_and(|p| p.shader.is_some())).count(), 4);
        // Every cube is a closed bevelled box (6 faces, 12 bevels, 8 corners).
        let cubes: Vec<_> = m.defs.values().filter(|d| d.mesh.faces.len() == 26).collect();
        assert_eq!(cubes.len(), 24);
        for d in cubes {
            assert!(d.mesh.edges.keys().all(|e| d.mesh.edge_faces(e).len() == 2), "{} is not closed", d.name);
        }
    }
}
