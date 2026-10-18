# Code map (read this first instead of sweeping the code)

Varstotas: a 3D modeller in Rust (eframe/egui 0.36 + wgpu, glam f64, slotmap). Units are millimetres, Z is up.
Status: `docs/06-STATUS.md`. API notes: `docs/requests/{kernel,render}-api-changes.md`. QA reports: `docs/qa/`.

## Crates
- `varstotas_kernel`: geometry and document. It has no UI or GPU code.
- `varstotas_render`: camera and wgpu renderer.
- `varstotas_script`: Rhai scripting API over the kernel (no UI); used by the Script Console and the bridge.
- `python/`: `varstotas.py` (Python client of the bridge) and `varstotas_mcp.py` (MCP server, stdio); embedded into the app and installed to `~/.varstotas/python/`.
- `varstotas_app`: the `varstotas` binary (UI, tools, inference).

### varstotas_kernel/src
| File | Contents |
|---|---|
| `mesh.rs` | `Mesh` storage (vertices, edges, faces in slotmaps), adjacency caches, queries |
| `topology/insert.rs` | sticky line insertion: vertex merge, edge split, automatic faces |
| `topology/region.rs` | planar region recompute (the core of sticky faces; largest file) |
| `topology/erase.rs` | eraser semantics |
| `ops/push_pull.rs`, `ops/transform.rs`, `ops/offset.rs`, `ops/follow_me.rs`, `ops/intersect.rs`, `ops/chamfer.rs` (Chamfer / Fillet), `ops/solid.rs` (Solid Tools: booleans of closed meshes) | modelling operations |
| `model.rs` | `Model`: definitions (root, groups, components), instances, materials, tags (`tag_visible(def, tag)`, `untagged_hidden`, `delete_tag`), axes, raycast |
| `curves.rs` | `CurveInfo` (arc, circle, polygon), `add_curve`, `add_pie`, `set_curve`, tangents |
| `dims.rs` | linear dimensions, stored per definition, anchored to vertices |
| `guides.rs` | guide lines and points, stored per definition |
| `history.rs` | undo; each snapshot is a full model clone |
| `io/` | `.vrst` JSON (`native.rs`), OBJ+MTL, STL, DXF (`dxf.rs`: R12 export, LINE/POLYLINE/3DFACE/CIRCLE/ARC import), glTF (`gltf.rs`: GLB export, .gltf/.glb import), 3MF export (`threemf.rs`, own stored ZIP) |
| `params.rs` | parametric groups: `Parametric` on a definition (size X/Y/Z by stretch or scale, copies in a row), `make_parametric`, `set_instance_params`, regeneration |
| `generator.rs` | generators (track К): header parsing (`Generator::parse`, `@param key:type(min..max/step)`), `GenKind` / `GenParam` / `GenValue` / `normalize`, `GenFormula` / `GenCheck` (`@formula`, `@check`), `Generated` on a definition, `Model::generators`, `generated_def` (shared definition by values), `graft_content` (built model → definition, ids remapped), `remove_unused`, `SIZE_KEYS` for Scale |
| `layout.rs` | Layout sheets (data): `LayoutSheet`, `SheetViewport` (rect on paper, captured view or scene name, scale, line flags), `SheetText`, `TitleBlock`, `PAPERS`; `Model::viewport_scene`, `with_scene_state` |
| `unfold.rs` | Unfold: `UnfoldInput::from_model` (selection → world faces, soft edges), `unfold` (pieces without overlaps, mountain/valley folds, tabs, edge numbers, sheet packing), `to_dxf` |
| `report.rs` | parts list (cut-list sizes, counts, area, volume) and material areas → CSV |
| `tessellate.rs` | model → world triangles and edges (plus planar texture coordinates, smooth vertex normals across smooth edges) |
| `workplane.rs` | `WorkPlane` (guide planes: origin, red / green, angles tilt-azimuth-spin), `Model::work_planes` / `active_plane`, `drawing_frame` (active plane, else axes), `set_drawing_frame` |
| `section.rs` | `SectionPlane`, `SectionDisplay`, scene section state, `Model::section_cut`, `section_fill` |
| `shadows.rs` | `ShadowSettings` (date, time, place, tones), sun direction |
| `lights.rs` | `Light` (`LightKind` point / spot / area: position, direction, colour, power W, radius, cone, size), `RenderSettings` (size, samples, time limit, bounces, denoise, match view) — `Model::lights`, `Model::render` |
| `pbr.rs` | `Pbr` (physical material: metallic, roughness, emission, transmission, IOR, maps), `Environment` (`SkyKind` incl. `Physical` + `turbidity`, `Tonemap`, rotation, exposure, `Effects` with `EFFECT_PRESETS` and `Stylize`), `CustomShader` + `shader_params` (`// @param` header) |
| `text3d.rs` | 3D Text: outlines → faces (even-odd nesting), extrusion, group (`Model::add_text3d_in`) |
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
| `grid.rs` | `GridStyle` (plane, step, major ratio), `level` / `snap_step` (shared by the `fs_grid` shader pass and grid snapping), `origin_near` (f32-safe origin) |
| `camera.rs` | f64 camera, orbit, pan, zoom, projections, zoom extents |
| `overlay.rs` | `Overlay3d` (lines, polygons, texts, marks, arcs, dimensions, leaders), plus `annot.rs` for the CPU glyph expansion |
| `export.rs` | `render_to_image` (PNG export), `encode_exr` (uncompressed float OpenEXR) |
| `bvh.rs` | binned-SAH BVH over triangles, flattened to 2 `vec4` per node for the GPU |
| `pathtrace.rs`, `pathtrace.wgsl` | GPU path tracer (`Renderer::pt_start / pt_step / pt_resolve / pt_image / pt_linear`): triangle / light records, compute passes `pt_trace` (BVH walk, GGX + diffuse, NEE to the sun and one light picked by its estimated share, any-hit shadows, glass, ground plane, custom surfaces, several samples per dispatch), `pt_denoise` (à-trous), `pt_resolve`, `pt_linear`; the WGSL is part of the scene module (`custom::BASE`); OIDN pass (`oidn`), `accel_structures` (BLAS / TLAS) |
| `pathtrace_rq.wgsl` | ray query `intersect` / `transmittance` replacing the BVH section of `pathtrace.wgsl` (`rq_source`) when the device has `EXPERIMENTAL_RAY_QUERY` (requested in `main.rs`) |
| `oidn.rs` | Intel Open Image Denoise loaded at run time (libloading): `available`, `denoise` |
| `textures.rs` | face textures (group 1): colour `texture_2d_array`, data array (ORM / normal), PBR material table; CPU resample and mips (`set_textures`, `set_data_textures`, `set_materials`) |
| `env.rs` | environment for Realistic: Rgba16Float equirect with mips, SH9 irradiance, `EnvUniform` (group 0, bindings 3–5) |
| `shader.wgsl` | shaders (`HDR_OUT` override: the Realistic scene pass writes linear HDR + a G-buffer when effects are on; PCSS soft shadows; `Surface` + helpers for custom shaders) |
| `post.rs`, `post.wgsl` | Realistic effects: `Post` (pipelines), `PostTargets` (HDR, G-buffer, linear depth, AO, bloom mips; `encode`), `PostUniform`; AO, SSR, fog, DOF, bloom, tone curve, NPR stylize, vignette |
| `custom.rs` | custom WGSL surfaces: `compose` (per-shader name suffix, `Params` structs, `custom_surface` dispatch), `validate` (naga), `check` (message with the shader's line) |

Viewer: `cargo run -p varstotas_render --example viewer` (env vars are listed in `06-STATUS.md`).

### varstotas_script/src
| File | Contents |
|---|---|
| `lib.rs` | `run(model, ctx, src) -> ScriptRun` (works on a copy; output, error, selection), `ScriptCtx`, `Entity` |
| `api.rs` | the Rhai API: `Vec3` type, `Entity` properties, geometry / operations / context functions (reference: `docs/SCRIPTING.md`) |
| `json.rs` | `eval_json` (JSON variables in, last value out), `entity_id` / `parse_entity_id` (for the bridge) |
| `generator.rs` | running generators: `build` (`build(p)` in an empty model with the document's materials; nested runs keep the outer script state), `def_for`, `place`, `set_values` (shared / in place / new definition), `scale` (Scale → size values), `add`, `rebuild_all`, `values_map`, `derive` / `formula_values` (`@formula` / `@check`), `register_math` (`min` / `max` / `clamp` / `ceil`, also in the API) |

Tests: `tests/api.rs` (also runs `examples/scripts/bookshelf.rhai`), `tests/generator.rs` (runs every `examples/generators/*.rhai`, formulas and checks).

### varstotas_app/src
| File | Contents |
|---|---|
| `app.rs` | `VarstotasApp`: layout, actions, input routing, files |
| `document.rs` | open document: model, history, selection, editing context |
| `viewport.rs` | 3D view widget: input, renderer, overlays; `PtJob` (path traced render advanced each paint, save PNG / EXR) |
| `scene_builder.rs` | model → `SceneGeometry` (incl. `TexVertex` layers), highlight ids |
| `textures.rs` | image decoding (`image` crate), `TextureSync` (uploads the model's textures + emission maps), new textured material |
| `pbr.rs` | Realistic style data: `material_table` (+ shader id / params), `custom_surfaces`, `animated`, `emission_layers` / `data_layers`, `DataSync` (ORM packing), `EnvSync` (HDRI decode), `env_style`, `pt_lights` |
| `ui/realistic.rs` | Materials → "Physical (Realistic)" editor (sphere preview, presets, maps, Shader row + parameter sliders), shader editor window (`ShaderEdit`, live apply), Environment card with Effects, HDRI / map loading |
| `shaders.rs` | bundled custom shaders (`PRESETS`: Wood, Water, Reeded Glass, Glow; docs/SHADERS.md) |
| `ui/console.rs` | Window → Script Console and File → Run Script… (`run_script_source`: one undo step; `.py` → `run_python_file`); splitter between log and code (`code_h`) |
| `ui/code_editor.rs` | console editor: `egui_code_editor` (highlighting, line numbers, completion) + our completion list: members after a dot by the receiver's type (`member_context`, `members_for`), header tags / `@param` types (`tag_context`); header lines highlighted (`with_doc_tags`, vendored) + parameter hints from `varstotas_script::help` (`call_context`, `fn_doc`) |
| `bridge.rs` | local server (127.0.0.1 + token, JSON lines / HTTP `POST /rpc`), `~/.varstotas/bridge.json`, installs `python/*.py` |
| `plugins.rs` | plugin header parsing (`@name/@menu/@param/@pick/@select/@import/@export`), scan of `~/.varstotas/plugins`, bundled examples |
| `ui/extensions.rs` | Extensions menu, File → Import / Export plugin entries, parameter dialog, running plugins; console code with a header runs as the pseudo plugin `CONSOLE` (`run_console_code`, `plugin_at`); steps `plugin_go`: points → objects → run |
| `ui/generators.rs` | generators in the Components list: `.rhai` scan of the components folder, bundled examples (folder Examples, not on disk), old `~/.varstotas/generators` moved in (`migrate_legacy`), `generator_items`, `generator_tile`, `place_generator` (temp model → `Clipboard::from_parts` → `PasteTool`), Entity Info (`generator_info`, `param_field` per type, `formula_rows`, Reset, Make Unique), `set_gen_values` |
| `ui/gen_editor.rs` | Window → Generator Editor: source (`ScriptEditor::with_id`), preview rebuilt after edits (`build_preview`, drag to turn), preview values, New / Open / Save to Library (`save_path`) / Apply to Document / Place |
| `ui/library.rs` | Components tray card: one list of components (`.vrst`) and generators (`.rhai`) by folder (`Tile`) under `folder_header` (fold arrow, link revealing the folder, Hide for Examples), thumbnails, place (`PasteTool::named`), Save Selection to Library |
| `ui/mat_library.rs` | Materials library in the Materials card: `.vrstmat` files in `~/.varstotas/materials` (subfolders = categories, `scan`), bundled Examples (the gallery demo materials), `use_material` (adds to the document, one undo step; same name + different look → new name), Save to Library / Export… (`model_material_menu`), Import…, Delete… dialog, `material_swatch` (texture thumbnail, metal / glass / emissive corner mark) |
| `tools/plugin_pick.rs` | point picking for `@pick` plugins |
| `tools/plugin_select.rs` | object selection for `@select` (click toggles, Enter → `ToolRequest::PluginObjects`) |
| `bridge_ops.rs` | bridge calls on the UI thread (`bridge_tick`: eval, operations, camera, screenshot, save/export with confirmation), running `.py` files |
| `vector_export.rs` | 2D drawing of the view (visibility by raycast, profiles, sections, dims) → SVG / PDF (`to_pdf_pages`: one page per drawing) |
| `layout.rs` | Sheets on paper: `viewport_drawing` (scene camera → `view_lines`, fit or 1:N, clipped), `sheet_drawing` (border, title block, texts, parts tables), `standard_views` (3 views + iso at a standard scale), overall dims (`auto_dims`), `table_rows` / `table_size`, `sheets_pdf`, `scene_camera` / `camera_scene` |
| `ui/sheets.rs` | Window → Sheets…: sheet list, paper, title block, item properties, preview with pick / move / resize (working copy → one undo step), `export_sheets`, harness dumps |
| `ui/unfold.rs` | Tools → Unfold… window (live preview, paper, scale, tabs), `sheet_drawing`, `export_unfold` (SVG / PDF / DXF) |
| `nav.rs` | mouse and trackpad camera navigation |
| `animation.rs` | animation export frames: `scene_frames` (scene tour: holds + `CameraAnimation` transitions), `orbit_frames`, `frame_path`, `ffmpeg()` lookup, `encode` (H.264 MP4) |
| `inference/mod.rs`, `inference/geo.rs` | snapping (inference) engine |
| `inference/settings.rs` | which snaps are on: `SnapPrefs` (in `Prefs::snapping`), `GROUPS` for the menus, `publish` once per frame, `allowed(kind)` checked by the engine and the tools; `GridPrefs` (in `Prefs::grid`) → `GridStyle` on the axes plane, `publish_grid` / `grid()` for grid snapping (`grid_snap`, `grid_crossing` in `inference/mod.rs`); `Increments` (in `SnapPrefs`): `move_step` / `angle_step` / `scale_step`, `round_move` (Move tool) |
| `axes.rs` | drawing frame (active work plane, else custom axes), published once per frame |
| `clipboard.rs` | copy, cut, paste |
| `autosave.rs` | autosave and recovery |
| `templates.rs` | File → New templates |
| `demo/mod.rs` | File → Open Demo: Country House — `country_house()` builds the furnished demo model in code (walls with openings via `prism`, gable roof, partitions, fireplace, windows / doors as components, furniture components, garden), its materials, lights, render settings and 8 scenes |
| `demo/textures.rs` | procedural tileable textures for the demo (siding, stone, roof metal, oak floor, decking, tiles, marble, woods, fabrics, rug, grass, gravel, brick, plaster): colour + normal + roughness maps as PNG (`Tex`, `Maps`) |
| `demo/gallery.rs` | File → Open Demo: Material Gallery — `material_gallery()`: 24 bevelled cubes (`bevelled_cube`) in different materials, a studio cove (tag Cove, hidden outdoors), 8 scenes with different environments / styles |
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
| `tools/nav_tools.rs` | Orbit, Pan, Zoom tools (Zoom VCB = field of view) |
| `tools/camera_tools.rs` | Position Camera, Look Around, Walk |
| `tools/chamfer.rs` | Chamfer / Fillet tools (size by moving away from the edge, preview on a mesh copy, VCB redo) |
| `tools/solid.rs` | Solid Tools (on the selection or click two solids; `Action::SolidTool(op)`) |
| `tools/text.rs` | Text tool (leader notes: dimensions with `leader`) |
| `tools/text3d.rs` | placing 3D Text (follows the cursor, lies on the face) |
| `text3d.rs` | glyph outlines of egui's bundled fonts (`skrifa`), `Text3dParams` |
| `tools/texture.rs` | Texture Position tool (red/green/blue pins) |
| `tools/section.rs` | Section Plane tool; drawing and picking of section planes, cut lines and fill |
| `tools/workplane.rs` | Guide Plane tool (3 points, face, normal by arrows; gizmo drags of the active plane), `draw_work_planes` |
| `ui/animation.rs` | File → Export → Animation… (`AnimWindow`, `AnimRun`): `start_animation`, `anim_tick` (one frame per app frame via `export_png_to` or the path tracer job, ffmpeg in a thread), the window |
| `ui/render.rs` | Window → Render… (`RenderWindow`: settings, Render / Stop / Continue / More Samples, preview, Save PNG / EXR, Render All Scenes batch, lights list + editor, light markers; edits → one undo step) |
| `ui/workplanes.rs` | Window → Work Planes… (list, active, visibility, names, origin and angles; edits → one undo step) |
| `ui/menu.rs`, `ui/toolbar.rs`, `ui/icons.rs`, `ui/statusbar.rs` | menus, toolbar, icons, status bar with the VCB |
| `ui/tray.rs` | cards toggled by `TRAY_CARDS` / `Prefs::tray_hidden` (Window menu); Entity Info (incl. face texture position), Materials (Texture…, tile size), Tags (Untagged eye, delete),  Outliner, Scenes (play, settings), Soften Edges, Instructor; Outliner rows: context menu, double-click = `FocusInstance`, drag & drop (`OutlinerDrag`, `drop_target` → `MoveInstance`, `outliner_drag_label`) |
| `ui/dialogs.rs` | modal dialogs (export, resize, recovery, text; library delete confirmation lives in `library.rs`) |
| `ui/theme.rs` | look and feel |

## Checks (filter the output)
- `cargo test --workspace 2>&1 | grep -E "^test result|FAILED|panicked"`
- `cargo clippy --workspace --all-targets 2>&1 | grep -E "^(warning|error)" | sort | uniq -c`
- To run a UI script: `VARSTOTAS_SCRIPT=file cargo run -p varstotas_app`.
  - Check state with `dump` rather than screenshots.
  - Take a screenshot only for visual checks: convert the BMP with `sips -s format png -Z 1200`, then crop.
- Regression scripts for UI behaviour live in `qa/scripts/` and run with `qa/run_all.sh`; `qa/fixtures/<script>/components` is copied into the run's home first.
