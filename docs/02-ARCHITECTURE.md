# Architecture

## Stack
| Layer | Choice | Why |
|---|---|---|
| Language | Rust (stable, edition 2024) | C++ speed, safety, mac/win cross-compilation with one command |
| Window + UI | `eframe`/`egui` 0.36 (wgpu backend) | native window, immediate-mode UI, a tiny binary, no Qt/Electron |
| GPU | `wgpu` (re-export `egui_wgpu::wgpu` — **only that one**, so the versions match) | Metal on macOS, DX12/Vulkan on Windows |
| Math | `glam` (f64 — `DVec3/DMat4` in the kernel, f32 — on the GPU) | |
| Entity IDs | `slotmap` | stable generational keys |
| Serialization | `serde` + `serde_json` (`.vrst`) | readable, easy to migrate |
| File dialogs | `rfd` | native dialogs |

Units inside the kernel — **millimetres, f64**. Point merge tolerance `TOL = 1e-3` mm.
Axes: X — red, Y — green, **Z — blue (up)**, right-handed system.

## Crates (workspace)
```
crates/
  varstotas_kernel/   # geometry kernel + document. No GPU, no UI. 100% unit-testable.
  varstotas_render/   # wgpu renderer + Camera. Does not know about the kernel: takes flat buffers.
  varstotas_app/      # binary: window, UI panels, tools, inference, VCB, input.
```
Dependencies: `app → kernel`, `app → render`. `kernel` and `render` do **not** depend on each other.

## varstotas_kernel
- `Model` — the document: a tree of `Definition` (root + groups/components), materials, tags, scenes, unit settings.
- `Mesh` — the "sticky" geometry of one context: vertices, edges, faces (outer loop + holes), instances of child definitions.
  - `add_line` merges vertices, cuts edges at intersection points, cuts faces, automatically creates faces on closed flat loops.
  - `push_pull`, `offset_face`, `follow_me`, `transform_vertices` (stretching connected geometry), `erase`, `reverse_face`, `intersect`.
- Group = a `Definition` with one `Instance`; component = a `Definition` with N instances.
- `History` — undo/redo on snapshots of `Model` (a clone). Simple and reliable; optimization — later.
- `tessellate` — triangulation of faces (with holes) + a list of edges with references to entities for picking.
- `io` — `.vrst` (JSON), OBJ/STL export, OBJ/STL import.
- `units` — parser/formatter of VCB lengths (`2500`, `2.5m`, `1'6"`, `300,200`).

## varstotas_render
- `Camera` (f64): perspective/ortho, orbit/pan/zoom, `screen_ray`, `world_to_screen`, `zoom_extents`, standard views.
- `Renderer`: draws **into an offscreen texture** (MSAA 4x + depth), registers it in egui as a `TextureId`; the app shows it as an image and draws 2D overlays (inference markers, the selection box, labels) through `egui::Painter`.
- Inputs:
  - `SceneGeometry` — triangles (pos, normal, color) + edges (pos, color). Uploaded **only when the model changes**.
  - `Overlay3d` — lines in 3D every frame (inference axes, guides, tool preview): width in px, dashes, "on top of everything" or with a depth test.
  - `RenderStyle` — background (sky/ground), edges on/off, profiles, X-ray, wireframe, hidden-line, grid, axes.
- Thick lines — expanded into quads in the vertex shader (wgpu has no line width).

## varstotas_app
- `App` (eframe): menu, left toolbar, right tray of panels (Entity Info, Materials, Tags, Outliner, Scenes, Instructor), status bar with the VCB.
- `tools/` — trait `Tool` (state machine): `on_mouse_move/down/up`, `on_key`, `on_vcb(input)`, `overlay()`, `status_text()`.
- `inference/` — the snapping engine: collects candidates (vertices/midpoints/edges/faces/axes/parallels/guides) in screen space, ranks them, returns `InferenceResult { point, kind, constraint_dir, ref_entity }`.
- `nav` — camera: middle button — orbit, Shift+middle — pan, wheel — zoom to the cursor.
- `scene_builder` — `Model + selection + context` → `SceneGeometry` (selection highlight, dimming outside the group being edited).
- `picking` — CPU: faces — raycast, edges/vertices — distance in pixels after projection.

## Frame data flow
```
input → nav / active Tool ── (InferenceEngine) ──→ ops on Model (via History.commit)
                                                   │ model_version++
Model ──scene_builder (if version changed)──→ Renderer.set_geometry
Tool.overlay() ─→ Overlay3d + 2D markers ──→ Renderer.render → egui image + painter
```

## Rules of joint development
1. Each crate has an owner (see `04-TEAM.md`). We do not edit another's crate — we write a request in `docs/requests/`.
2. A crate's public API is described in its `lib.rs`; breaking changes — only with an update of this document.
3. `cargo build`, `cargo test`, `cargo clippy` — green before handing off a task.
4. The kernel is covered by unit tests for every operation (especially face topology).
