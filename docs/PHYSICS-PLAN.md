# Physics plan

One-shot physics for preparing still scenes: run a simulation, look at the result, keep it as
one undo step or put everything back. Not a game engine: nothing keeps moving after Apply.

Engine: [rapier3d](https://rapier.rs) 0.36 (pure Rust, Apache-2.0), in its own crate
`crates/varstotas_physics` so the UI never talks to rapier directly. The model is in millimetres;
the simulation runs in metres (rapier's tolerances are tuned for them). Jolt (C++) was considered
and set aside: it would need CMake and C++ on every platform, and a crash inside it cannot be
caught.

## Ph1 — Drop (done)

Tools → Physics… (`ui/physics.rs`), engine `varstotas_physics::DropSim`.

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

## Ph2 — Cloth (next)

rapier 0.36 has soft bodies (`SoftBodyBuilder`): `cloth` (a rectangular particle grid with
structural, shear and bending edges), `trimesh` (any triangle mesh, dihedral bending), volume
preservation, pinning, collisions with rigid colliders. So no own solver is needed.

- Input: a selected face (or a group with one face) → a uniform triangle grid at a chosen
  resolution (a rectangle maps to `cloth`; any other planar polygon: a clipped grid, `trimesh`).
- Colliders: every other visible face, as in Drop.
- Options: resolution, stiffness / bending, thickness, pinned vertices (selected edges or
  vertices stay in place), time limit.
- Result: a new group of triangles with the inner edges soft and smooth; the source face is
  replaced (or kept hidden — an option).
- Risk to check first: a 100 × 100 cloth is ~20 000 faces — selection, saving and Undo of such
  groups must stay fast.

## Ph3 — Later

- A box as a pillow: `trimesh` with volume preservation.
- Self-collision quality, wind, tearing (rapier has it).
- Drop: remembered settings, pieces by default for concave bodies.
