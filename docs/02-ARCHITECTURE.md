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
  varstotas_i18n/     # localization: Fluent messages from locales/<lang>.ftl, tr!("key"), platform variants.
  varstotas_kernel/   # geometry kernel + document. No GPU, no UI. 100% unit-testable.
  varstotas_render/   # wgpu renderer + Camera. Does not know about the kernel: takes flat buffers.
  varstotas_script/   # Rhai scripting API over the kernel (Script Console, script files, generators, the bridge).
  varstotas_app/      # binary: window, UI panels, tools, inference, VCB, input, plugins.
locales/              # en.ftl (base), ru.ftl, lt.ftl, be.ftl — every UI text, see LOCALIZATION.md.
python/               # Python client of the local bridge + MCP server, embedded into the app.
```
Dependencies: `app → kernel, render, script`, `script → kernel`; every crate → `i18n`. `kernel` and `render` do **not** depend on each other.

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

## Rules
1. A crate's public API is described in its `lib.rs`; breaking changes — only with an update of this document.
2. `cargo build`, `cargo test`, `cargo clippy` — green before a commit; UI changes also pass their `tests/ui/scripts/` regressions (`tests/ui/run_all.sh <name>`).
3. The kernel is covered by unit tests for every operation (especially face topology).
4. Every UI text comes from `locales/<lang>.ftl` by key (`tr!`), see `LOCALIZATION.md`.

## Security boundaries (2026-10-22)

`varstotas_script::ScriptCtx` adds `trusted_io: bool` (default false). Only explicit local
script/plugin entry points opt in. Document generators/rigs and bridge evaluation always use
false. `script::network` centralizes HTTP(S)-only bounded downloads; `script::process`
provides cancellation, output limits and Unix process-group cleanup, not OS permission isolation.

`kernel::io::{read_limited, resource_path, validate_model}` bound file reads, confine external
assets to the import directory (canonical paths, symlinks included), and validate native model
graphs before rebuilding caches. `def_bounds` memoizes shared definitions and detects cycles.
`io::{export_bundle, write_bundle}` enumerate multi-file output before any writes and refuse
replacement unless the output path was explicitly approved. `io::atomic_write` uses exclusive temporary
files and rename for native saves and bridge credentials; parent directories must be trusted.

Bridge transactions own a staged Model and source document version/context. Only the owning
connection can mutate/commit/abort them. Commit detects intervening UI edits. Transport uses
CSPRNG tokens, bounded clients/queues/lines/bodies and header authentication before body reads.
