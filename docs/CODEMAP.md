# Code map (read this first instead of sweeping the code)

Varstotas: a 3D modeller in Rust (eframe/egui 0.36 + wgpu, glam f64, slotmap). Units are millimetres, Z is up.
Status: `docs/06-STATUS.md`. API notes: `docs/requests/{kernel,render}-api-changes.md`. QA reports: `docs/qa/`.

## Crates
- `varstotas_kernel`: geometry and document. It has no UI or GPU code.
- `varstotas_render`: camera and wgpu renderer.
- `varstotas_app`: the `varstotas` binary (UI, tools, inference).

### varstotas_kernel/src
| File | Contents |
|---|---|
| `mesh.rs` | `Mesh` storage (vertices, edges, faces in slotmaps), adjacency caches, queries |
| `topology/insert.rs` | sticky line insertion: vertex merge, edge split, automatic faces |
| `topology/region.rs` | planar region recompute (the core of sticky faces; largest file) |
| `topology/erase.rs` | eraser semantics |
| `ops/push_pull.rs`, `ops/transform.rs`, `ops/offset.rs`, `ops/follow_me.rs`, `ops/intersect.rs` | modelling operations |
| `model.rs` | `Model`: definitions (root, groups, components), instances, materials, tags, axes, raycast |
| `curves.rs` | `CurveInfo` (arc, circle, polygon), `add_curve`, `add_pie`, `set_curve`, tangents |
| `dims.rs` | linear dimensions, stored per definition, anchored to vertices |
| `guides.rs` | guide lines and points, stored per definition |
| `history.rs` | undo; each snapshot is a full model clone |
| `io/` | `.vrst` JSON (`native.rs`), OBJ+MTL, STL |
| `tessellate.rs` | model → world triangles and edges (plus planar texture coordinates) |
| `texture.rs` | `TextureImage` (embedded PNG/JPEG, tile size), `TexPlacement` (per-face texture position), texture axes, base64 |
| `units.rs` | parsing and formatting for the Measurements box (VCB) |
| `grid.rs` | spatial hash |
| `geom.rs` | geometry helpers |
| `triangulate.rs` | polygon triangulation |

Tests are in `tests/`: `sticky`, `ops`, `model`, `intersect`, `qa_m3`, `m4b2`, plus `common/` helpers.
Benchmark: `cargo run --release -p varstotas_kernel --example bench [filter]`.

### varstotas_render/src
| File | Contents |
|---|---|
| `renderer.rs` | `Renderer`: offscreen MSAA, styles, highlight, scene upload, axes |
| `camera.rs` | f64 camera, orbit, pan, zoom, projections, zoom extents |
| `overlay.rs` | `Overlay3d` (lines, polygons, texts, marks, arcs, dimensions, leaders), plus `annot.rs` for the CPU glyph expansion |
| `export.rs` | `render_to_image` (PNG export) |
| `textures.rs` | face textures: `texture_2d_array`, CPU resample and mips (`Renderer::set_textures`) |
| `shader.wgsl` | shaders |

Viewer: `cargo run -p varstotas_render --example viewer` (env vars are listed in `06-STATUS.md`).

### varstotas_app/src
| File | Contents |
|---|---|
| `app.rs` | `VarstotasApp`: layout, actions, input routing, files |
| `document.rs` | open document: model, history, selection, editing context |
| `viewport.rs` | 3D view widget: input, renderer, overlays |
| `scene_builder.rs` | model → `SceneGeometry` (incl. `TexVertex` layers), highlight ids |
| `textures.rs` | image decoding (`image` crate), `TextureSync` (uploads the model's textures), new textured material |
| `nav.rs` | mouse and trackpad camera navigation |
| `inference/mod.rs`, `inference/geo.rs` | snapping (inference) engine |
| `axes.rs` | custom drawing axes, published once per frame |
| `clipboard.rs` | copy, cut, paste |
| `autosave.rs` | autosave and recovery |
| `templates.rs` | File → New templates |
| `debug_capture.rs` | `VARSTOTAS_SCRIPT` harness, including the `raw` and `rawp` real-input commands |
| `tools/mod.rs` | `Tool` trait and tool registry |
| `tools/common.rs` | inference wrapper, context transforms, `dim_after_transform` |
| `tools/select.rs` | Select tool |
| `tools/draw.rs` | Line, Rectangle, Circle, Polygon, 2 Point Arc |
| `tools/arcs.rs` | Arc (centre), Pie, 3 Point Arc, Rotated Rectangle |
| `tools/edit.rs` | Eraser, Paint Bucket, Push/Pull, Offset, Follow Me |
| `tools/transform.rs` | Move, Rotate, Scale (copies, arrays) |
| `tools/construct.rs` | Tape Measure, Protractor, Axes |
| `tools/dims.rs` | Dimension tool |
| `tools/nav_tools.rs` | Orbit, Pan, Zoom tools |
| `ui/menu.rs`, `ui/toolbar.rs`, `ui/icons.rs`, `ui/statusbar.rs` | menus, toolbar, icons, status bar with the VCB |
| `ui/tray.rs` | Entity Info (incl. face texture position), Materials (Texture…, tile size), Tags, Outliner, Scenes (play, settings), Instructor |
| `ui/dialogs.rs` | modal dialogs (export, resize, recovery, text) |
| `ui/theme.rs` | look and feel |

## Checks (filter the output)
- `cargo test --workspace 2>&1 | grep -E "^test result|FAILED|panicked"`
- `cargo clippy --workspace --all-targets 2>&1 | grep -E "^(warning|error)" | sort | uniq -c`
- To run a UI script: `VARSTOTAS_SCRIPT=file cargo run -p varstotas_app`.
  - Check state with `dump` rather than screenshots.
  - Take a screenshot only for visual checks: convert the BMP with `sips -s format png -Z 1200`, then crop.
- Regression scripts for UI behaviour live in `qa/scripts/` and run with `qa/run_all.sh`.
