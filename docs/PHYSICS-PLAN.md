# Physics plan

One-shot physics for preparing still scenes: run a simulation, look at the result, keep it as
one undo step or put everything back. Not a game engine: nothing keeps moving after Apply.

Engine: [rapier3d](https://rapier.rs) 0.36 (pure Rust, Apache-2.0), in its own crate
`crates/varstotas_physics` so the UI never talks to rapier directly. The model is in millimetres;
the simulation runs in metres (rapier's tolerances are tuned for them). Jolt (C++) was considered
and set aside: it would need CMake and C++ on every platform, and a crash inside it cannot be
caught.

## Ph1 — Drop (done)

Window → Physics… (`ui/physics.rs`), engine `varstotas_physics::DropSim`.

- **Bodies**: the selected groups and components of the open context that are neither locked
  nor hidden (`drop_bodies`). Everything else that is visible is static (one triangle mesh).
  Optional floor at Z = 0 (on by default).
- **Shapes**: Hull (convex hull, fast and steady) or Pieces (V-HACD convex decomposition: a
  bowl can hold things). A flat body (one face) gets a slightly rounded hull or a thin box.
- **Run**: on a thread; each frame of the window shows the newest state by writing the bodies'
  transforms into the model without an undo step (`Document::preview_change`). Stops when every
  body sleeps or fell into the void (far below the scene: it keeps its start position), or at
  the time limit.
- **Apply** records one undo step from the model as it was before the run. **Cancel** (or Esc,
  or closing the window) restores it. While a run is shown the view can be turned, the tools
  wait, and any action other than the view's cancels the run first (Undo only cancels it).
- Settings: friction, bounce, gravity, time limit, shape, floor.
- Tests: `crates/varstotas_physics/src/tests.rs`; QA: `qa/scripts/physics.txt`.

## Ph2 — Cloth (done)

The Cloth mode of the same window, engine `varstotas_physics::cloth::ClothSim` (a rapier soft
body: particles, structural / shear / bending edges, contacts with rigid colliders).

- **Input**: one selected face, or one unlocked group / component whose content is a single face
  (`cloth_source`). `cloth_grid` lays a regular grid along the face's first edge, clipped to the
  face with its holes (whole cells only), at most 25 600 points.
- **Colliders**: every other visible face (one static triangle mesh), optional floor.
- **Settings**: spacing, stiffness and bending (0..1, mapped to spring frequencies), thickness
  (the contact gap; at least a tenth of the spacing, thinner slips through), friction, gravity,
  self-collision, time limit. At rest: no point moved faster than 8 mm/s over 0.5 s.
- **Preview**: the first frame replaces the source (`place_cloth`: the face with the edges only
  it used, or the group) by a new group; later frames swap its mesh (`set_cloth_mesh`). The mesh
  is built on the thread (`cloth_mesh`, ~20 ms for 20 000 triangles) at most every 30 ms.
  `Model::add_group_mesh` adds the group directly (`make_group` takes ~0.7 s for 20 000 faces).
- **Look**: each triangle keeps the face's front / back materials and gets the texture placement
  that maps its corners to where they lay on the flat face (`carried_placement`), so the texture
  follows the folds; a material painted on the source group goes to the cloth group.
- **Result**: a group of triangles, edges soft and smooth (`io::mesh_from_triangles`); Apply is
  one undo step and selects it, Cancel brings the face and the selection back.
- Tests: `crates/varstotas_physics/src/cloth_tests.rs`; QA: `qa/scripts/physics_cloth.txt`.

## Ph3 — Later

- A box as a pillow: `trimesh` with volume preservation.
- Cloth: pinned points (selected edges / vertices stay), wind, tearing; self-collision quality.
- Drop: remembered settings, pieces by default for concave bodies.
