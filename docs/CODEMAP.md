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
| `io/` | `.vrst` JSON (`native.rs`), OBJ+MTL, STL, DXF (`dxf.rs`: R12 export, LINE/POLYLINE/3DFACE/CIRCLE/ARC import), glTF (`gltf.rs`: GLB export, .gltf/.glb import, Draco primitives via `draco-core`), 3MF export and import (`threemf.rs`, own stored ZIP; components, transforms, colours), COLLADA import (`dae.rs` on the mini-DOM `xml.rs` / quick-xml), FBX import (`fbx.rs`, ufbx: Z-up, mm), PLY import / export (`ply.rs`), CSV points import / faces JSON export (`points.rs`); `meshes.rs` (`ImpMesh` → groups with materials, `add_meshes`), `zip.rs` (`unzip`) |
| `params.rs` | parametric groups: `Parametric` on a definition (size X/Y/Z by stretch or scale, copies in a row), `make_parametric`, `set_instance_params`, regeneration |
| `generator.rs` | generators (track К): header parsing (`Generator::parse`, `@param key:type(min..max/step)`), `GenKind` / `GenParam` / `GenValue` / `normalize`, `GenFormula` / `GenCheck` (`@formula`, `@check`), `Generated` on a definition, `Model::generators`, `generated_def` (shared definition by values), `graft_content` (built model → definition, ids remapped), `remove_unused`, `SIZE_KEYS` for Scale |
| `layout.rs` | Layout sheets (data): `LayoutSheet`, `SheetViewport` (rect on paper, captured view or scene name, scale, line flags), `SheetText`, `TitleBlock`, `PAPERS`; `Model::viewport_scene`, `with_scene_state` |
| `model.rs` (licences) | `Attribution` (source, title, author, licence, url; `line`, `license_kind`), `LicenseKind` (CC0 / CC-BY / SA / NC / ND / Standard / Unknown: `parse`, `label`, `note`) |
| `unfold.rs` | Unfold: `UnfoldInput::from_model` (selection → world faces, soft edges), `unfold` (pieces without overlaps, mountain/valley folds, tabs, edge numbers, sheet packing), `to_dxf` |
| `report.rs` | parts list (cut-list sizes, counts, area, volume) and material areas → CSV |
| `tessellate.rs` | model → world triangles and edges (plus planar texture coordinates, smooth vertex normals across smooth edges) |
| `workplane.rs` | `WorkPlane` (guide planes: origin, red / green, angles tilt-azimuth-spin), `Model::work_planes` / `active_plane`, `drawing_frame` (active plane, else axes), `set_drawing_frame` |
| `section.rs` | `SectionPlane`, `SectionDisplay`, scene section state, `Model::section_cut`, `section_fill` |
| `rig.rs` | parameters of ordinary components: `Rig` (params, formulas, `Joint` — part by name, `JointKind` rotate / slide / visible / material, from..to or expression — and the `update` script) on the master definition, `Rigged` on variants and painted parts, `PartOp`, `SceneRig` (values in scenes); `pose_rig` (master content with the parts moved / hidden / painted), `new_rig_variant`, `rig_variant` (shared by values), `rig_write_back` (an edited variant → master, poses undone), `rig_parts` / `name_rig_parts` / `rig_part_bounds`, `rig_master`, `instance_rigged`, `rig_instances`, `placed_count`; masters kept alive with parametric bases (`param_bases`) |
| `stats.rs` | document statistics: `Model::statistics` → `ModelStats` (scene counts with every placed copy expanded vs stored counts, definitions / materials used of all, textures, tags …), `used_defs` (reachable from the root, parametric base / variant included), `used_materials` (faces, instances, generated values by name), `purge_unused` → `Purged`, `delete_definition` (all copies + nested unused), `delete_material` (faces / instances back to the default) |
| `shadows.rs` | `ShadowSettings` (date, time, place, tones), sun direction |
| `lights.rs` | `Light` (`LightKind` point / spot / area: position, direction, colour, power W, radius, cone, size), `Light::transformed`, `RenderSettings` (size, samples, time limit, bounces, denoise + AI, firefly clamp, match view, `RenderDof` + f-stop / focus, transparent background, grading: exposure, tone curve, white balance, tint, saturation, contrast, vignette), `RENDER_PRESETS`, `RENDER_SIZES` — `Model::lights`, `Model::render` |
| `pbr.rs` | `Pbr` (physical material: metallic, roughness, emission, transmission, IOR, maps, clear coat, sheen, anisotropy, anti-tiling, bevel), `Environment` (`SkyKind` incl. `Physical` + `turbidity`, `Tonemap`, rotation, exposure, `Effects` with `EFFECT_PRESETS` and `Stylize`), `CustomShader` + `shader_params` (`// @param` header) |
| `text3d.rs` | 3D Text: outlines → faces (even-odd nesting), extrusion, group (`Model::add_text3d_in`) |
| `primitives.rs` | 3D primitives: `Primitive` (Box, Cylinder, Cone, Pyramid, Sphere, Torus) — `polygons` in own axes, `mesh` (closed solid, curved surfaces soft + smooth) |
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
| `pathtrace.rs`, `pathtrace.wgsl` | GPU path tracer (`Renderer::pt_start / pt_step / pt_resolve / pt_image / pt_linear`): triangle / light records, compute passes `pt_trace` (BVH walk, GGX + diffuse, clear coat / sheen / anisotropic VNDF lobes, `bevel_normal` (rounded edges by probe rays), anti-tiled maps (`sample_tiled_l`), NEE to the sun and one light picked by its estimated share, any-hit shadows, glass, ground plane, custom surfaces, several samples per dispatch), `pt_denoise` (à-trous), `pt_resolve` (grading: `PtOutput` via `pt_set_output` — exposure, tone curve, white balance (`white_balance`), saturation, contrast, vignette, alpha of a transparent background), `pt_linear`; the WGSL is part of the scene module (`custom::BASE`); OIDN pass (`oidn`), `accel_structures` (BLAS / TLAS) |
| `pathtrace_rq.wgsl` | ray query `intersect` / `transmittance` replacing the BVH section of `pathtrace.wgsl` (`rq_source`) when the device has `EXPERIMENTAL_RAY_QUERY` (requested in `main.rs`) |
| `oidn.rs` | Intel Open Image Denoise loaded at run time (libloading): `available`, `denoise` |
| `textures.rs` | face textures (group 1): colour `texture_2d_array`, data array (ORM / normal), PBR material table; CPU resample and mips (`set_textures`, `set_data_textures`, `set_materials`) |
| `env.rs` | environment for Realistic: Rgba16Float equirect with mips, SH9 irradiance, `EnvUniform` (group 0, bindings 3–5) |
| `shader.wgsl` | shaders (`HDR_OUT` override: the Realistic scene pass writes linear HDR + a G-buffer when effects are on; PCSS soft shadows; `fs_face_pbr` with clear coat, sheen, anisotropic GGX (`d_ggx_aniso`) and anti-tiling (`tile_mix` / `sample_tiled`, 8 vec4 per material); `Surface` + helpers for custom shaders) |
| `post.rs`, `post.wgsl` | Realistic effects: `Post` (pipelines), `PostTargets` (HDR, G-buffer, linear depth, AO, bloom mips; `encode`), `PostUniform`; AO, SSR, fog, DOF, bloom, tone curve, NPR stylize, vignette |
| `custom.rs` | custom WGSL surfaces: `compose` (per-shader name suffix, `Params` structs, `custom_surface` dispatch), `validate` (naga), `check` (message with the shader's line) |

Viewer: `cargo run -p varstotas_render --example viewer` (env vars are listed in `06-STATUS.md`).

### varstotas_script/src
| File | Contents |
|---|---|
| `lib.rs` | `run(model, ctx, src) -> ScriptRun` (works on a copy; output, error, selection), `ScriptCtx`, `Entity` |
| `api.rs` | the Rhai API: `Vec3` type, `Entity` properties (also `hidden`, `bounds` / `size` / `min` / `max`), `child` / `children` by name (deep), a string as a child entity, geometry / operations / context functions (reference: `docs/SCRIPTING.md`) |
| `json.rs` | `eval_json` (JSON variables in, last value out), `entity_id` / `parse_entity_id` (for the bridge) |
| `generator.rs` | running generators: `build` (`build(p)` in an empty model with the document's materials; nested runs keep the outer script state), `def_for`, `place`, `set_values` (shared / in place / new definition), `scale` (Scale → size values), `add`, `rebuild_all`, `values_map`, `derive` / `formula_values` (`@formula` / `@check`), `register_math` (`min` / `max` / `clamp` / `ceil`, also in the API) |

Also `rig.rs`: components with parameters — `rig_ops` (joints: range → from..to or the value / an expression; then `script_pose`: the script `update(p)` runs with the whole API in a sandbox clone whose root is the variant content; changed parts are copied into the variant, `unscript` removes the script's work before write-back), `variant_for`, `set_values` (shared / in place / new), `rebuild_all` (equal variants merged), `attach` (copies placed with a master → default variant), `after_edit` (write back + re-pose, called by `Document` on every commit), `set_rig` (add / change / remove), `warnings`.

Tests: `tests/rig.rs` (joints, shared variants, write-back, script / visibility / paint, remove and attach), `tests/api.rs` (also runs `examples/scripts/bookshelf.rhai`), `tests/generator.rs` (runs every `examples/generators/*.rhai`, formulas and checks).

### varstotas_app/src
| File | Contents |
|---|---|
| `app.rs` | `VarstotasApp`: layout, actions, input routing, files |
| `document.rs` | open document: model, history, selection, editing context |
| `viewport.rs` | 3D view widget: input, renderer, overlays; a `Gpu` (renderer + geometry) per pane, shared tessellation, `rect` = active pane, `view_rect` = all panes (window positions, harness raw origin); `PtJob` (path traced render in pane 0's renderer, save PNG / EXR) |
| `panes.rs` | Split view 1 / 2 / 4: `PaneView` (camera, cam_anim, style, X-Ray, edges, profiles, axes, ground, sky, grid), `pane_rects`, `exchange` (the active pane's settings are the app's own), `activate_pane` / `click_activate`, `init_panes` (Top / Front / Right), `PRESETS` / `apply_preset` (layouts: view + projection per pane), `panes_dump` |
| `scene_builder.rs` | model → `SceneGeometry` (incl. `TexVertex` layers), highlight ids |
| `applog.rs` | the application log behind Window → Log: `push` / `error` (a repeat within 5 s counts up), `snapshot`, `unseen` / `mark_seen` (the status bar count), `clear`, `clock`; `init` installs a `log::Log` in front of env_logger that keeps errors of any crate and warnings of `varstotas*`; `flash_error` logs too (`flash_error_quiet` when the console already did) |
| `thumbs.rs` | `Thumbs`: rendered thumbnails of components without a picture (second `Renderer`, Shaded with Textures, iso, transparent, 160 px, ≤2 per frame — `get` / `process`); library files cached in `~/.varstotas/cache/thumbs` (`disk_path`); paused while a harness screenshot is pending (`DebugCapture::screenshot_pending`) |
| `textures.rs` | image decoding (`image` crate), `preview_bytes` (file preview pictures: ≤512 px, JPEG / PNG), `TextureSync` (uploads the model's textures + emission maps), new textured material |
| `online_models.rs` | 3D Library: `ModelSource` (Poly Haven / Smithsonian / NASA / Kenney / Quaternius (`manual`: in the browser) / Web / Favourites / History / `Plugin`), `ModelAsset` (+ `format`, `pack`), parsers `parse_polyhaven` / `parse_smithsonian` / `parse_nasa` / `parse_kenney` + `kenney_zip` / `parse_quaternius` + `quaternius_download` / `parse_plugin_catalogue`, `import_pack` (a pack folder: each model once in its best format, preview by name), `format_of`, `pretty_name`, `catalogue` (cached, `unique_names`), `download` → `fetch_asset` → `model_from_spec` (file / url / zip / meshes, unit / up) → `import_file` (kernel importers or `@model_import`) → `library_model` (one component with its `Attribution`, on the ground) + `preview_of` (spec `preview` / the asset's thumbnail) → `<library>/<source>/<name>.vrst`; `Online::load_models` / `download_model` (`Msg::Models` / `Msg::Model`) |
| `online_mats.rs` | online PBR materials: Poly Haven / ambientCG catalogues (`parse_polyhaven`, `parse_ambientcg`, cached in `~/.varstotas/cache/online`), `fetch` (system curl), `unzip` (re-export of the kernel's), `fetch` stops curl on the thread's cancel flag (`set_cancel`), `build_material` / `material_from_maps` (1K maps at real-world size), `guess_map` (map kind by file name), `embeddable` (TGA / BMP → PNG), `Source` (built-ins + `Plugin(PluginSource)`), `Online` background workers (catalogue / download / 4 thumbnail threads, `Msg`) |
| `pbr.rs` | Realistic style data: `material_table` (+ shader id / params), `custom_surfaces`, `animated`, `emission_layers` / `data_layers`, `DataSync` (ORM packing), `EnvSync` (HDRI decode), `env_style`, `pt_lights` |
| `ui/realistic.rs` | Materials → "Physical (Realistic)" editor (sphere preview, presets, "Coat, Sheen, Edges" sliders, maps, Shader row + parameter sliders), shader editor window (`ShaderEdit`: `ScriptEditor::wgsl`, Start from…, Reference column, live apply), Environment card with Effects, HDRI / map loading |
| `shader_help.rs` | WGSL completion for the shader editor: Surface fields, `p.` params, swizzles, `// @param` tag (`items_at`), helper / built-in signatures (`fn_doc`) |
| `shaders.rs` | bundled custom shaders (`PRESETS`: Wood, Water, Reeded Glass, Glow; docs/SHADERS.md) |
| `ui/console.rs` | Window → Script Console and File → Run Script… (`run_script_source`: one undo step; `.py` → `run_python_file`); splitter between log and code (`code_h`) |
| `ui/code_editor.rs` | console and shader editor (`ScriptEditor`, `Lang::Rhai` / `Lang::Wgsl`; Esc closes a popup without dropping focus via the focus event filter): `egui_code_editor` (highlighting, line numbers, completion) + our completion list: members after a dot by the receiver's type (`member_context`, `members_for`), header tags / `@param` types (`tag_context`); header lines highlighted (`with_doc_tags`, vendored) + parameter hints from `varstotas_script::help` (`call_context`, `fn_doc`) |
| `ui/view_hud.rs` | The viewport's top-right controls: view drop-down (standard views via `Camera::standard_view`, perspective / parallel, Zoom Extents) and style drop-down (`STYLES`, X-Ray / Edges / Profiles); Show drop-down (`hud_show_items`: guides, axes, ground, sky, grid, shadows, sections), Field of View in the view menu; rows with icons (`item`, `pill` paint over `Atom::custom`, menus close only on a click outside); `view_hud(ui, rect)` per pane, `hud_rect` keeps its clicks from the tools (`handle_viewport_input`) |
| `ui/app_help.rs` | Help → User Guide: topics of the whole app (`TOPICS`, blocks `B`: text, rows, tool cards from `ToolId::description` / `instructor` with Use, links to Script Reference topics), search over topics and tools (`find_blocks`, `find_tools`), the Keyboard page from `ToolId::shortcut`; F1 over the model → `app_help_for_tool` (`topic_of`); test: every tool in one topic |
| `ui/script_help.rs` | Help → Script Reference: topics (Basics, Console and plugins, Generators, Component script — static pages `B::*`; Functions by `help::GROUPS`; Properties from `help::MEMBERS`), search over functions / members / tags; `open_script_help(topic, word)`, `help_button` ("? Reference" in the console, Generator Editor and Component Parameters), F1 in a `ScriptEditor` (`f1`: the word at the cursor or the call it is in) |
| `ui/log_window.rs` | Window → Log (`LogWindow`: Errors / Warnings toggles, filter, Copy, Clear; time, source, ×count) and `log_badge` — the status bar's red count of unseen errors / quiet "Log" button (`toggle_log`) |
| `bridge.rs` | local server (127.0.0.1 + token, JSON lines / HTTP `POST /rpc`), `~/.varstotas/bridge.json`, installs `python/*.py` |
| `plugins.rs` | plugin header parsing (`@name/@menu/@param/@pick/@select/@import/@export/@material_import/@material_source`), `param_value(s)` (text → script values), scan of `~/.varstotas/plugins`, bundled examples |
| `mat_plugins.rs` | material plugins: `call` (Rhai function on an empty model / Python `plugin.py <func>` with JSON stdin → JSON last line), `catalogue` / `download` (sources), `import` (importers), `material_from_spec` (spec JSON: maps / folder / zip / factors → `Material`) |
| `ui/extensions.rs` | Extensions menu, File → Import / Export plugin entries, parameter dialog, running plugins; console code with a header runs as the pseudo plugin `CONSOLE` (`run_console_code`, `plugin_at`); steps `plugin_go`: points → objects → run |
| `ui/generators.rs` | generators in the Components list: `.rhai` scan of the components folder, bundled examples (folder Examples, not on disk), old `~/.varstotas/generators` moved in (`migrate_legacy`), `generator_items`, `generator_tile`, `place_generator` (temp model → `Clipboard::from_parts` → `PasteTool`), Entity Info (`generator_info`, `param_field` per type, `formula_rows`, Reset, Make Unique), `set_gen_values` |
| `ui/gen_editor.rs` | Window → Generator Editor: source (`ScriptEditor::with_id`), preview rebuilt after edits (`build_preview`, drag to turn), preview values, New / Open / Save to Library (`save_path`) / Apply to Document / Place |
| `ui/library.rs` | Components tray card (frame from `ui/tray_lib.rs`: top row with + / cloud = 3D Library / … menu, sections In Model → My Library → Examples) — one list of components (`.vrst`) and generators (`.rhai`) by folder (`Tile`) under `folder_header` (fold arrow, link revealing the folder, Hide for Examples), thumbnails (the file's `Model::preview` picture via `LibItem::picture`, else a rendered thumbnail from `thumbs.rs`, edges until it is ready; `tile_ex`; the Draw → Shapes generators are not listed), preview pictures of the document, a library file or a document component (`Definition::preview`, tile texture in `def_pics`): `PreviewTarget`, `set_preview_of`, `preview_of_from_file`, `set_library_preview`, `set_doc_preview`, `preview_from_file`, `preview_from_view(target)` → `preview_shot_done`, place (`PasteTool::named`), Save Selection to Library (+ menu), Save to Library of a document component (`save_definition_to_library`, `write_to_library`); the Document folder also lists the open document's component definitions (`document_components`, `def_tile`, `place_definition` via `Clipboard::of_definition` — the copy shares the definition; Delete from Document… → `ask_delete_definition` / `def_delete_dialog` / `delete_definition`); Rename… of a document's component, a library file or a material (`RenameTarget` incl. `Mat` / `MatFile`, `component_rename_dialog`, `rename_component`, `rename_library_file`); Favourites / Recent sections (`Prefs::lib_favourites` / `lib_recent`), tile menu actions `LibMenuAct` (favourite, Move to Folder), folder menu `FolderAct` → `folder_act` / `folder_dialog` (New / Rename / Delete Folder), `tray_drag_frame` (a tile dragged into the viewport: starts the place / Paint tool on entering, clicks at the drop point; label at the cursor) |
| `ui/tray_lib.rs` | common frame of the Components and Materials cards: `TileSize` (Small / Large / List, `Prefs::lib_tiles`), `top_row` (search, Refresh, + menu, cloud, … menu ending with the tile size), `section` (fold arrow, title link, count; folded keys in `Prefs::library_collapsed`, Materials keys `mat:`), `tiles` / `tile` / `paint_picture` / `found`; `lib_menu` (Add to Favourites, Move to Folder ▸), `folder_menu`, `FolderAsk`, `move_file`, `subfolders`, `rekey`, `note_recent` (`RECENT_MAX` = 8), `toggle`, `key_names`, `DragItem` / `TrayDrag`; `SPOTS` → `lib_spots=` in the harness dump (cleared at the start of the toolbar; also the magnet, card headers `card:`, Snapping items `snap:`, Tags buttons, the Physical editor) |
| `ui/rig_editor.rs` | components with parameters: Entity Info `rig_info` (fields via `value_fields` shared with generators, Reset, Make Unique, Edit Parameters…), `set_rig_values`; Window → Component Parameters (`RigEditor`: rows of parameters / formulas / motions, Pick… axis via `PluginPickTool` → `rig_pick_done`, script editor, Apply / Revert / Remove); scenes: `scene_rigs` (values with the camera transition, `RigAnimation`), `frame_rigs` (exported animation frames); `rig_dump` |
| `nav.rs` (prefs) | `NavPrefs` (orbit / zoom speed, invert, zoom to cursor, `NavGesture` of the middle button and two fingers), `zoom_factor` / `orbit_delta`; `publish` / `prefs()` — thread-local for the Orbit / Zoom / Look Around tools; `NavController::handle` takes it |
| `keymap.rs` | keyboard shortcuts: `KeyCmd` (commands + every tool, `defaults`, `action`), `Shortcut` (store / label / `reserved`), `Prefs::shortcuts` reassignments (`assign` takes a key from its old command, `reset`), thread-local `publish` / `lookup` / `label` / `labels` used by `handle_keyboard`, menus, toolbar tips, the guide |
| `ui/preferences.rs` | Window → Preferences…: `PrefsWindow` (sections `Section` on the left, search by `Section::matches`, Reset per section `reset_prefs_section`); General, Drawing (grid + `snapping_menu`), Libraries (`folder_row` for `Prefs::library_dir` / `mat_library_dir`, Examples, tile size), Downloads (`Prefs::downloads_watch_dir`, Send to Varstotas), View: `ViewDefaults` in `Prefs::new_view`, applied by `apply_view_defaults` on File → New (`new_from_template`), `current_view_defaults`; Navigation (`Prefs::nav`); Interface (`Prefs::theme` / `ui_scale` / `toolbar_icons`, applied each frame by `apply_interface`); Render (`prefs_render`: `Prefs::new_render` used by `new_from_template`, `file_row` for `ffmpeg_path` / `oidn_path`); General → `undo_steps` (`History::set_limit` each frame); Shortcuts (`prefs_shortcuts`: table, `PrefsWindow::capture` waits for the keys — `handle_keyboard` is silent meanwhile) |
| `ui/model_info.rs` | Window → Model Info…: `ModelInfoWindow` (statistics cached per model version — `model_stats`; scene / file / libraries, Preview image row), Purge Unused (`purge_unused`, also Edit → Purge Unused); Preview Image window (File → Preview Image → Show…: `doc_preview_window`, `doc_preview_texture`) |
| `ui/mat_library.rs` | Materials card in the `tray_lib` frame (`material_library_ui`: + = New from Texture… / Import…, cloud = Online Materials, … = folder / Examples): sections In Model, My Library (`material_library_dir`, `Prefs::mat_library_dir`; subfolders = categories, `scan`), Examples (the gallery demo materials, `Prefs::hide_mat_examples`); tiles `MatLook` (texture, metal / glass / emissive corner mark), menus `MatAct` (Use, Rename…, Save to Library, Export…, Delete…); `pick_material` / `palette_for` (arms Paint with a library material; it joins the document on the first paint via `PaletteMaterial::ensure_in`), `use_material` (adds to the document, one undo step; same name + different look → new name), `delete_model_material` (In Model → Delete, undoable), Favourites / Recent (`Prefs::mat_favourites` / `mat_recent`), `save_material_value`, `rename_material`, `rename_material_file`, Delete… dialog |
| `downloads.rs` | Sites without an API: `Site` / `load_sites` (`~/.varstotas/sources.toml`) / `search_url`; `Expect` (waiting card: `from_send`, `from_asset`), `match_expect`; `Watcher` (Downloads folder thread, `VARSTOTAS_DOWNLOADS`), `where_froms` (mdls / Zone.Identifier), `archive_models`, `file_hash`; `SendServer` (127.0.0.1:47815, token `send_token`, `bookmarklet`, `parse_query`); `Offer` + `import_offer` (copy to `downloads/manual/<hash>`, one component or a pack); `licence_warning`; `LibraryState` (favourites, history, imported hashes: `library_state.json`), `SavedAsset`; `Downloads` (the window's state) |
| `ui/downloads.rs` | «Search on» row, Other Sites section (Watch Downloads, Send to Varstotas + Copy Bookmarklet, waiting cards), Import Download window; `downloads_tick` (watcher / listener on-off, offers, imports, dropped files and `.webloc` / `.url`), `downloads_expect`, `downloads_offer`, `downloads_drop`, `downloads_import` |
| `ui/model_library.rs` | 3D Library window (`ModelWindow`: Source dropdown with `@model_source` plugins and their params, Favourites / Downloaded, filters (`Filter`, `LicFilter`), ☆, Cancel (`models_cancel`), licence badges, search (Enter asks searching sources), categories, thumbnail grid, details + Download & Place / Download / Place / Web Page, From URL; `models_poll`, `models_download`, `models_from_url`); `import_model_file` (File → Import with `@model_import`) |
| `ui/online_mats.rs` | Online Materials window (`OnlineWindow`: Source dropdown (built-ins + `@material_source` plugins, their params + Load: `online_plugin_header`, `plugin_source`, `online_set_source`), search, category, thumbnail grid via `show_rows`, details + Download & Use / Download / Use / Web Page; `online_poll` takes the worker results, `online_download`) |
| `tools/plugin_pick.rs` | point picking for `@pick` plugins |
| `tools/plugin_select.rs` | object selection for `@select` (click toggles, Enter → `ToolRequest::PluginObjects`) |
| `bridge_ops.rs` | bridge calls on the UI thread (`bridge_tick`: eval, operations, camera, screenshot, save/export with confirmation), running `.py` files |
| `vector_export.rs` | 2D drawing of the view (visibility by raycast, profiles, sections, dims) → SVG / PDF (`to_pdf_pages`: one page per drawing) |
| `layout.rs` | Sheets on paper: `viewport_drawing` (scene camera → `view_lines`, fit or 1:N, clipped), `sheet_drawing` (border, title block, texts with `{credits}` (`CREDITS`, `credits_text`), parts tables), `standard_views` (3 views + iso at a standard scale), overall dims (`auto_dims`), `table_rows` / `table_size`, `sheets_pdf`, `scene_camera` / `camera_scene` |
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
| `demo/mod.rs` | Help → Demos → Country House — `country_house()` builds the furnished demo model in code (walls with openings via `prism`, gable roof, partitions, fireplace, windows / doors as components, furniture components, garden), its materials, lights, render settings and 8 scenes |
| `demo/textures.rs` | procedural tileable textures for the demo (siding, stone, roof metal, oak floor, decking, tiles, marble, woods, fabrics, rug, grass, gravel, brick, plaster): colour + normal + roughness maps as PNG (`Tex`, `Maps`) |
| `demo/gallery.rs` | Help → Demos → Material Gallery — `material_gallery()`: 24 bevelled cubes (`bevelled_cube`) in different materials, a studio cove (tag Cove, hidden outdoors), 8 scenes with different environments / styles |
| `debug_capture.rs` | `VARSTOTAS_SCRIPT` harness, including the `raw` and `rawp` real-input commands (`raw scroll` = a trackpad swipe) |
| `tools/mod.rs` | `Tool` trait and tool registry (`SHAPES` — Draw → Shapes) |
| `tools/primitives.rs` | `PrimitiveTool`: Draw → Shapes (base, then height / tube; Measurements `w,d[,h]` / `r[,h]`, `Ns`) → `generator::add` + `place` of the bundled `examples/generators/primitives/*.rhai` (`SOURCES`) |
| `tools/common.rs` | inference wrapper, context transforms, `dim_after_transform` |
| `tools/select.rs` | Select tool |
| `tools/draw.rs` | Line, Rectangle, Circle, Polygon, 2 Point Arc |
| `tools/arcs.rs` | Arc (centre), Pie, 3 Point Arc, Rotated Rectangle |
| `tools/bezier.rs` | Bezier Curve: `Anchor` (point + handles), `bezier_points` (cubic spans), `smooth` (Catmull-Rom handles); `Method` Handles / Smooth / Freehand × `Fill` Line / Area in `mode()` (thread_local), `options_ui` = the bar over the view (drawn by `view_hud`), `ModTap` taps: Alt = next method, Ctrl = fill; Line → `add_polyline_lines` (no face), Area → `add_polyline` closed |
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
| `tools/light.rs` | lights in the viewport: `draw_lights` / `draw_light` (marker, point rays, spot cone, area rectangle, aim line + handle of the selected one), `pick_light`, `on_aim_handle`, `light_screen` (harness); Select picks lights and drags the aim handle, Move / Rotate move them (`transform::Targets::light`), Entity Info edits them (`render::light_editor`) |
| `tools/workplane.rs` | Guide Plane tool (3 points, face, normal by arrows; gizmo drags of the active plane), `draw_work_planes` |
| `ui/animation.rs` | File → Export → Animation… (`AnimWindow`, `AnimRun`): `start_animation`, `anim_tick` (one frame per app frame via `export_png_to` or the path tracer job, ffmpeg in a thread), the window |
| `ui/render.rs` | Window → Render… (`RenderWindow`: `settings_ui` — Quality presets, Image, Sampling, Camera (DOF, `lens_radius` / `focal_length`), Light, Colour; Render / Stop / Continue / More Samples, preview, Save PNG / EXR, Render All Scenes batch, lights list + `light_editor` (shared with Entity Info), `edit_light`; edits → one undo step) |
| `ui/workplanes.rs` | Window → Work Planes… (list, active, visibility, names, origin and angles; edits → one undo step) |
| `ui/menu.rs`, `ui/toolbar.rs`, `ui/icons.rs`, `ui/statusbar.rs` | menus, toolbar (tools, navigation, Snapping; views and styles are in `view_hud.rs`), icons, status bar with the VCB |
| `ui/tray.rs` | cards toggled by `TRAY_CARDS` / `Prefs::tray_hidden` (Window menu); Entity Info (incl. face texture position), Materials (library via `mat_library.rs`), Tags (Untagged eye, delete),  Outliner, Scenes (play, settings), Soften Edges, Instructor; Outliner rows: context menu, double-click = `FocusInstance`, drag & drop (`OutlinerDrag`, `drop_target` → `MoveInstance`, `outliner_drag_label`) |
| `ui/dialogs.rs` | modal dialogs (export, resize, recovery, text, about; library delete confirmation lives in `library.rs`) |
| `ui/theme.rs` | look and feel: `ThemePref`, light / dark `set_look`, `wants_dark`; colours are functions of the theme (`bar_bg()`, `tray_bg()`, `card_bg()`, `field_bg()`, `hover_bg()`, `text()`, `text_dim()`, `pick(light, dark)`); icons use `icons::pal::ink()` / `fill()` / `shade()`; toolbar button size `toolbar::ICON_SIZES` / `set_button_side` |

## Checks (filter the output)
- `cargo test --workspace 2>&1 | grep -E "^test result|FAILED|panicked"`
- `cargo clippy --workspace --all-targets 2>&1 | grep -E "^(warning|error)" | sort | uniq -c`
- To run a UI script: `VARSTOTAS_SCRIPT=file cargo run -p varstotas_app`.
  - Check state with `dump` rather than screenshots.
  - Take a screenshot only for visual checks: convert the BMP with `sips -s format png -Z 1200`, then crop.
- Regression scripts for UI behaviour live in `qa/scripts/` and run with `qa/run_all.sh`; `qa/fixtures/<script>/components` is copied into the run's home first.
