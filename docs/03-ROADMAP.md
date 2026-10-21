# Roadmap

## M0 — Skeleton (orchestrator) ✅
- workspace, crates, API contracts, documents.

## M1 — Foundation (parallel, 3 streams) ✅
- **Kernel:** Mesh with sticky geometry (add_line, auto-faces, splitting edges/faces), erase, push_pull, transform_vertices, tessellate, History, save/load, OBJ/STL export, units parser. Tests.
- **Render:** Camera, Renderer (offscreen MSAA, lit faces, front/back colours, edges, thick/dashed overlays, grid, axes, sky/ground, X-ray, wireframe).
- **App shell:** window, classic-style layout, camera navigation, toolbar, status bar, VCB widget, file menu, panels (stubs with real model data), hotkeys.

**M1 demo:** open the program, orbit a scene with a test cube, save/open a file.

## M2 — Drawing (2 streams) ✅
- **Tools & Inference:** Select (click/double/triple/boxes), Line, Rectangle, Circle, Polygon, Arc, Eraser, Push/Pull, Move, Rotate, Offset, Tape Measure + guides, Paint Bucket. P0 inference engine. VCB for all.
- **Kernel:** offset_face, arcs/curves (edges merged into a curve), groups (make/explode/edit context), materials.

**M2 demo:** draw the house from `00-VISION.md`.

## M3 — MVP polish ✅ (QA: docs/qa/M3-report.md, docs/qa/M3-round2-report.md)
- Groups edit context, Entity Info, Materials, styles, Zoom Extents, standard views, undo for everything, autosave.
- UX QA pass, fixing topology bugs, performance (10k+ faces @ 60 FPS).
- Release builds: macOS `.app` (universal), Windows `.exe`.

## M4 — Second wave ✅
- **Batch 1:** guides as full-fledged objects (selection, Delete, Move/Rotate/copy, Entity Info, Edit → Delete Guides, hide/show), Tape Measure: resizing the model via VCB; Intersect Faces (with model / with selection / with context), Reverse / Orient Faces, Flip Along; Copy / Paste / Paste in Place; PNG export; templates (architectural mm/m, inches).
- **Batch 2:** Arc (centre + 2 points), 3 Point Arc, Pie, the "tangent at vertex" snap; Rotated Rectangle; Axes; Dimensions.
  - Follow-up (together with the batch 2 QA fixes): in the PNG export dialog — a choice of what goes into the image: background/transparency (already there), axes, guides, dimensions.
  - Dimension: at the dimension offset step, snaps are currently disabled — in perspective the line with the text goes anywhere, not into the object's plane. All snaps (axes, points/edges/faces of objects, guides) and direction locking with Shift and the arrows must work during the offset.
- **Batch 3 ✅:** textures (images, UV, saving in `.vrst`); animation of transitions between scenes.
- Already done earlier: Components + Outliner, Tags, Scenes, Follow Me, Scale, Protractor, OBJ/STL import.

## M5 — Onward ✅
- **Batch 1:** Section Planes ✅ (tool, active cut, cut lines and fill, selection/Move/Reverse/Align View, visibility in View, state in scenes); Soften/Smooth edges ✅; camera: Field of View, Position Camera, Look Around, Walk ✅.
- **Batch 2:** shadows (sun by date/time/latitude, shadow map), Text (callouts) and 3D Text ✅.
- **Batch 3:** Solid tools (Union/Subtract/Intersect/Trim/Split, Outer Shell), DXF import/export ✅.

> **Goal after M5 (decision of 2026-10-14):** a genuinely working tool for hobby projects, "without fanaticism". Order: M4 → M5 → M6 → M7, the "Quality" stream in parallel. M8 and "Platforms" — only after the necessary functionality on macOS is fully ready, and by a separate decision of the project owner.

## M6 — A working tool ✅
- **Batch 1 ✅:** glTF/GLB exchange (import/export), 3MF for 3D printing; report: specification of groups/components, sizes for cutting, areas/volumes (CSV).
- **Batch 2 ✅:** 2D documentation from the model: views/sections with dimensions → SVG/PDF export (light sheet layout).
- **Batch 3 ✅:** parametric components (simple attributes: width/height/count).

## M7 — Extensibility ✅
- **Batch 1 ✅:** built-in scripting API (Rhai, crate `varstotas_script`) over the kernel; Script Console, File → Run Script…; reference `docs/SCRIPTING.md`.
- **Batch 2 ✅:** a bridge to the running application (127.0.0.1 + token, JSON line by line or HTTP `POST /rpc`); a dependency-free Python client `varstotas` (the same API as Rhai, `with m.operation()` = one Undo step), running `.py` from File → Run Script…; the MCP server `varstotas_mcp.py` (stdio) with the tools below; confirmation in the application for destructive actions.
- **Batch 3 ✅:** `.rhai` / `.py` plugins in `~/.varstotas/plugins` with a header made of comments: Extensions menu items, a parameter dialog, picking points in the model (`@pick`), importers and exporters in File → Import / Export (`docs/PLUGINS.md`). Local component library: the Components section in the tray (thumbnails, search, placing by click, one definition per document, Save Selection to Library).
- MCP server for AI agents (Claude Code and others): tools over the same API as the scripts — reading the model (entities, sizes, materials, tags), building/editing geometry, selection, camera and a view screenshot, saving/export; transport stdio + local HTTP to the running application; all agent changes go through undo (one operation = one step), confirmation for destructive actions.

## D — Documentation (decision of 2026-10-15)
On top of M6: drawings and patterns "for paper" without fanaticism.
- **Batch 1 ✅:** unfolds (Tools → Unfold…): the faces of the selection are laid out in the plane in pieces without overlaps, "mountain"/"valley" folds, glue flaps, numbers of paired edges, scale, sheets A4–A2 / by size; export to SVG / PDF (a page per sheet) / DXF.
- **Batch 2 ✅:** sheets — light sheet layout (Window → Sheets…): sheets A4–A1 in the document, viewports (a captured view or a scene with its own hidden tags and sections; fit or 1:N in parallel projection; clipping by the frame), texts, title block (name, author, date, scale, sheet number); selecting, moving and resizing by mouse in the preview; PDF of all sheets / SVG of a sheet.
- **Batch 3 ✅:** finishing the drawings: "+ Standard Views" — the main view, the top and left views (first angle, GOST 2.305) at a common standard scale (GOST 2.302) with overall dimensions, and an isometric; automatic overall dimensions for any orthogonal viewport; "+ Parts Table" — a parts table (groups and components, count, sizes, material) on a sheet. Track D closed.

## P — Snaps and precision (owner's request of 2026-10-16)
Control of all snaps, grid, work planes, stepped moves.
- **Batch 1 ✅ — snapping menu.** A toolbar button with a magnet icon with a drop-down menu and the main menu item **View → Snapping ▸** with the same submenu. Every snap the application knows is switched on and off separately:
  - points: endpoints, midpoints, centres (of circles, arcs, faces), intersections, the origin;
  - on objects: on an edge, on a face, on guide lines and points;
  - directions: along axes (red, green, blue), parallel and perpendicular to an edge, "from point" alignment, tangent;
  - shapes: square, golden ratio, half circle;
  - grid (batch 2), angles (batch 4), move step (batch 4).

  There is a common "Snapping on/off" switch with a hotkey. The state is saved in the settings; the magnet icon shows that some snaps are off. Hints at the cursor and markers are visible only for enabled snaps.
- **Batch 2 ✅ — grid.** Displayed on the ground or on the active work plane: step, major lines every N steps, adaptive thinning when zooming out, fading towards the horizon. Enabled in View → Grid and in the magnet menu; the step is set in the settings and in the Measurements box. Snapping to grid nodes and lines — with a priority lower than the model geometry.
- **Batch 3 ✅ — work (guide) planes.**
  - Creation: the "Guide Plane" tool — by three points, on a face (by its plane) or through a point with a normal along an axis or an edge.
  - Editing: the origin position, the normal or angles (tilt and rotation), rotation around the normal. Set in a dialog with fields and by mouse — a gizmo with a normal arrow and rotation arcs.
  - Storage: several planes in the document, one active. Drawing tools put points "in the void" on the active plane instead of the ground; inference axes and the grid are taken from its coordinate system.
  - Visibility, renaming, deletion; Undo.
- **Batch 4 ✅ — moving by increments.** Track P closed. Step values for Move / Copy (default 10 mm), Rotate and the protractor (default 15°) and Scale (0.1). Enabled in the magnet menu; the hint shows the rounded value. Input in Measurements and snaps to geometry work as before and take priority over the step.

## B — Model library from open sources (owner's request of 2026-10-16) ✅ closed 2026-10-19
Searching and downloading ready-made models and materials from free sources right into the document (only sources with an open API or a free licence).
- **Batch 1 — skeleton and first sources.** ✅ (2026-10-18, together with part of batches 2 and 3: Import from URL, Sketchfab and Poly Pizza as example plugins, model sources and importers as plugins; see `docs/PLUGINS.md`. Not done: filters by licence / format / polygons, cancelling a download, Kenney / Quaternius, favourites and history, a "semi-automatic" mode for sites without an API.)
  - Window **Window → 3D Library**: a search bar, filters (source, licence, format, polygon count), a grid of results with previews, a model card (author, licence, size, link).
  - Download: files to the cache `~/.varstotas/downloads`, unpacking zip. Import with the existing importers (glTF/GLB, OBJ, STL, DXF) as a component: units are converted to mm (glTF — metres), the model is put on the ground.
  - Protection from heavy models: a warning about a large polygon count.
  - The component attributes keep the source, author, licence and link. File → Export → Attributions produces a list of credits (needed for CC-BY).
  - Network: HTTP client, timeouts and cancelling, offline work from the cache.
  - Sources without keys or with a free key: Poly Haven (CC0, public API), Smithsonian Open Access 3D (CC0), Poly Pizza (CC0/CC-BY, free key), NASA 3D Resources (open repository).
- **Batch 2 — more sources and import by link.** ✅ (2026-10-19: Kenney, Quaternius via the browser, filters, favourites and history, cancelling, semi-automatic mode for sites without an API, 3MF / DAE / FBX import; see `docs/06-STATUS.md`. Not done: STEP; the ND badge is shown at once, not "after an edit".)
  - Sketchfab: only downloadable models with CC licences, with the user's personal token.
  - CC0 sets from Kenney and Quaternius (a catalogue with previews).
  - **Import from URL**: a direct link to `.glb` / `.gltf` / `.obj` / `.stl` / `.zip`.
  - Favourites and download history.
  - **Sites without an API** (Printables, Thingiverse, MyMiniFactory, Cults3D, GrabCAD, Free3D, the free sections of CGTrader and TurboSquid, etc.) — a "semi-automatic" mode in which everything concerning the site is done by the user in their own browser. The application does not parse pages and results, does not download from sites by itself, does not bypass login, captcha and download limits; its job is to remove the routine before and after.
    - **Search everywhere with one query.** For each site a template of the address of its own search page is stored (`…/search?q={query}`) with a filter for free models and suitable licences, where the site supports such a filter. In the 3D Library window the results of API sources are shown as a grid, and below it is a row "Search on: Printables · Thingiverse · …": a click opens the site's results in the system browser. The templates live in an editable file (`~/.varstotas/sources.toml`): a broken address is fixed without a new version, you can add your own site.
    - ~~**Sending a page to Varstotas.**~~ *Removed 2026-10-20: the user fills in the page in the Import Download window.* A bookmarklet or a small browser extension "Send to Varstotas" passes the page address, title, author and licence text through the local M7 bridge (127.0.0.1 + token). This data is visible to the user on the page, and the browser reads it only on their click. The application shows a card "Waiting for a file from this page" with the attribution fields already filled in.
    - **Download interception (with consent).** While the waiting card is open or the setting is on, the application watches the "Downloads" folder. A new `.stl` / `.obj` / `.glb` / `.gltf` / `.3mf` / `.dae` / `.zip` is immediately offered for import. The source is taken from the file's system metadata (`kMDItemWhereFroms` on macOS, Zone.Identifier on Windows) and matched with the submitted page. The archive is unpacked, suitable files are shown in a list, textures are picked up.
    - **Drag and drop.** A file or link can be dropped from the browser onto the window. A direct link to a file is downloaded as an ordinary user download; a link to a page opens the waiting card.
    - **Licences without surprises.**
      - For each import a record is stored: CC0, CC-BY, CC-BY-SA, CC-BY-NC, CC-BY-ND, "standard / personal use" or "unknown". It is filled in from the page data, the site's default or manually and confirmed by the user.
      - **Warnings:** NC — on export and printing for sale; ND — a "do not modify" badge on the component after an edit; SA — a reminder on export; unknown licence — a "for myself only" mark.
      - **Attribution:** File → Export → Attributions and the caption on the "D" sheets are assembled automatically.
    - **Your own library from what you downloaded.** What was imported goes into the M7 component library together with the source, licence and preview. Duplicates are detected by hash, local search works over the downloaded — there is no need to go to the site again.
    - **Formats missing for such sites:** 3MF and Collada (`.dae`) import, then FBX (via ufbx); STEP (GrabCAD) — after assessing the weight (OpenCASCADE is heavy).
- **Batch 3 — materials and extensibility.** ✅ (2026-10-18)
  - Textures and PBR materials from Poly Haven and ambientCG (CC0) right into the Materials tray (the diffuse map; other maps — after batch 1 of track "R").
  - Sources as plugins: `.py` / `.rhai` with `search` / `download` functions modelled on M7, to add your own catalogues and local folders.

## K — Parametric generator components (owner's request of 2026-10-16)
Evolving the M6 parametric groups (only size and a row of copies) into full-fledged parametric components: a component is built by a script from parameters, parameters are edited in Entity Info.
- **Batch 1 — mechanism.** ✅ (2026-10-16, see `docs/GENERATORS.md`)
  - A generator is a Rhai script with a parameter header, like the M7 plugins. Parameter types: length, integer, number, yes/no, choice from a list (for example "left / right / up", "inward / outward"), angle, percent, material. For each parameter — a range, a step and a default value. The `build(p)` function builds geometry in the component's local axes.
  - Each instance has its own values. Instances with equal values share one definition. The component is rebuilt when a parameter changes, and each change is one Undo step.
  - **Entity Info:** fields by type — length with units, a counter, a drop-down, a checkbox, an angle or percent slider. "Reset" and "Make Unique" buttons. Several selected instances are edited together.
  - Scale on such a component changes the width, height and depth parameters instead of stretching the geometry.
  - The generator source is stored in the document: the file opens on another machine. Generators are shown in the Components tray. The same parameters are available from Rhai, Python and MCP.
- **Batch 2 — more generators and authoring.** ✅ (2026-10-16; windows and doors cut into a wall are not made as an application mechanism — owner's decision)
  - New generators: a staircase (flights, tread and riser by floor height), a shelf or rack, a cabinet with doors and drawers, a railing and handrails, a table.
  - Formulas between parameters: for example, shelf width = width − 2 × board thickness.
  - A generator editor window with live preview. The "Save generator to library" command.
  - Generators and components — one catalogue (`~/.varstotas/components`) and one list in the tray (owner's request).

## R — Realistic visualization (formerly M8; owner's request of 2026-10-16)
A photorealistic picture from the same model: PBR in the viewport, shader effects and styles, path tracing. The same stack — wgpu (WGSL), no external renderers.
- **Batch 1 — real-time PBR.** ✅ (2026-10-16; a separate HDR buffer is not needed — tone mapping in the face shader; the HDR buffer and post-processing — in batch 2)
  - **Metallic-roughness materials** (as in glTF): base colour, metallic, roughness, normal, AO, emission, transparency and transmission (glass). Each parameter — a number or a texture map.
  - A material editor in the Materials tray with a preview on a sphere.
  - glTF import and export with maps. Poly Haven and ambientCG textures (track "B") — full sets of maps.
  - **Image-based lighting (IBL):** HDRI maps, a sky background or colour, environment rotation. The sun from the M4 shadows gives the physical light strength.
  - Tone mapping (ACES / AgX), exposure, linear colour and sRGB, MSAA.
  - A new face style **Realistic** next to Shaded and the others; environment parameters are saved in scenes.
- **Batch 2 — shader effects and styles.** ✅ (2026-10-16; GTAO replaced by hemispherical SSAO, hot reload — applying from the editor window after 0.5 s)
  - **Effects:** soft shadows (PCSS), ambient occlusion (SSAO/GTAO), screen-space reflections, bloom, depth of field, fog and haze, vignette. Each is enabled separately, with a set of "quality / speed" presets.
  - **Stylized (NPR) styles:** pencil sketch, watercolour, technical lines with variable thickness, "clay".
  - **Custom shaders:** WGSL materials (procedural wood, water, glass, glowing) with parameters in the material editor and hot reload. Stored in the document.
- **Batch 3 — path tracing (ray tracing).** ✅ (2026-10-16; Preetham sky instead of Hosek–Wilkie; lights shine only in the render. 2026-10-18: wgpu ray query where available, otherwise its own BVH; Intel OIDN if installed, otherwise its own à-trous; 4–7 times faster)
  - **Render:** a progressive GPU path tracer (wgpu compute shaders, BVH from the tessellation). Where supported, the hardware wgpu ray query is used; without it — its own BVH traversal.
  - **Light physics:** PBR BSDF as in batch 1, refraction and caustics of glass (simplified), emission, HDRI and a physical sky (the Hosek–Wilkie model) with the sun.
  - **Light sources as objects:** point, spotlight, area; IES profiles — later.
  - **Render window:** resolution, number of samples or time, progress, stop and resume, denoising (its own à-trous, OIDN when available). Saving PNG / EXR, batch render of all scenes.
- **Batch 4 — animation.** ✅ (2026-10-16; without motion blur; track "R" closed) Exporting an animation of scenes and a fly-around of the model to video (via ffmpeg, if installed) or to a PNG sequence — in real time (Realistic) or by path tracing with a limit of samples per frame. Motion blur — optional.

## U — Convenience and finishing touches (owner's request of 2026-10-19)
Items 2 and 4 agreed with the owner on 2026-10-19.
1. ✅ **Viewing the document preview.** The preview image of the current document can be set (File → Preview Image) but cannot be viewed. A viewer is needed — for example, a File → Preview Image → Show item and a thumbnail in Model Info.
2. ✅ **A unified look of the Materials and Components trays.** Both cards are arranged identically: search, Refresh, +, cloud (online library), "…"; sections In Model → My Library (folders) → Examples; the Small / Large / List tile size is shared; identical tile menus. Owner's decisions of 2026-10-19: the document on top, a size switch, the cloud in the top row, plus favourites, recents, folders and drag and drop.
   - ✅ 2a — the shared frame (2026-10-19).
   - ✅ 2b — favourites ☆ and recents, folders from the tray (New Folder, move to a folder, rename a folder) (2026-10-19).
   - ✅ 2c — dragging from the tray into the viewport (a material — onto a face, a component — into place) (2026-10-19).
   *Along the way: Delete for a model material (with Undo); a material from the library is copied into the model only on the first painting.*
3. ✅ **Demos in one menu.** All demos and demo scenes (house, material gallery, render scenes, etc.) — in one menu, for example Help → Demos.
4. **Preferences window.** Owner's decisions of 2026-10-19: a window with sections on the left and search; nothing is removed from the menu — the window duplicates the snaps and steps, grid, library folders, downloads and the default view; new settings — mouse navigation, hotkeys, interface, render and external programs.
   - ✅ 4a — the window: General (autosave, template), Drawing (snaps, steps, grid), Libraries (component and material folders, Examples, tile size), Downloads (watched folder), View (view of a new document: style, edges, axes, ground, sky, grid, shadows, projection, FOV); search, Reset per section.
   - ✅ 4b — mouse navigation: zoom inversion and speed, orbit speed, zoom to cursor, trackpad.
   - ✅ 4c — interface: scale, light / dark theme, toolbar icon size.
   - ✅ 4d — hotkeys: a table of commands, remapping, reset.
   - ✅ 4e — default render, paths to ffmpeg and OIDN, the number of Undo steps.
5. ✅ **3D primitives.** Tools for drawing solid bodies in one operation: a parallelepiped (box), a cylinder; candidates — cone, sphere, torus, prism, pyramid. Entering sizes in Measurements, snaps, the result is a group (or component); parameters are editable afterwards where possible.
   *Done 2026-10-19: Draw → Shapes (box, cylinder, cone, pyramid, sphere, torus) — generated components; along the way, component thumbnails without a picture are rendered with textures.*
6. ✅ **Passing parameters down to nested components.** Component parameters (Component Parameters) can drive the parameters of nested generated components (for example, the height of a cylinder inside).
7. ✅ **A parameter script with the ordinary API.** The component's `update(p)` script — the same functions as in ordinary scripts and generators, with no special ones; parts are found by name (`child(name, depth)`), dimensions are fields of the object itself (`size`, `min`, `max`, `bounds`).
8. ✅ **Renaming components** — a Rename… item in the Components tile menu (document and library components).
9. ✅ **A script reference in the application and a shared error log.** Help → Script Reference with topics for the console, generators and the component parameter script, "? Reference" and F1 in the editors; Window → Log — all errors and warnings (scripts, import / export, libraries…), a counter in the status bar.
10. ✅ **Component previews from the viewport.** Set Preview from View in the Components tile menu: for library files and document components (they also get a picture from a file and Remove); the picture is saved with the document and carried over on insertion from the library and Save to Library.
11. ✅ **In-app application help.** Help → User Guide: 19 topics from first steps to files and scripts, tool cards from their own texts with a Use button, search, F1 over the model — the topic of the active tool.
12. ✅ **View and style right in the viewport.** Drop-downs in the top right corner: standard views, perspective / parallel projection, display style, X-Ray / edges / profiles, icons on menu items; the Show menu (guides, axes, ground, grid, shadows, sections) and the FOV field. This is preparation for several viewports.
13. ✅ **Several viewports (1 / 2 / 4).** Each pane has its own camera, style and display of axes, ground, sky and grid; the active pane is chosen by click. The widget on the viewport — icons only. Layout presets (Front + Iso, Front, Left, Top + Iso …).
14. **Vertex editing** *(planned, not started)*. Vertex selection (click, box), Move with snaps and distance input, vertex deletion (adjacent edges merge or the face is rebuilt).
15. ✅ **Slice tool** *(done 2026-10-20: Tools → Slice; kernel `Model::slice`)*. A plane (by three points, by an axis or by a face) cuts the selected object or model — edges are added along the line of intersection; optionally — split into two groups. Currently the same is done by hand: a helper rectangle → Intersect Faces → With Model → erase the rectangle; the Section Plane only shows the cut and does not create geometry.
16. **Bezier curves** *(owner's request 2026-10-19)*.
   - ✅ 16a — the Draw → Bezier Curve tool: control points and handles (a cubic curve), snaps and entering length / segments in Measurements, a curve is a chain of edges of one curve; a closed flat curve gives a face (2026-10-19).
   - ✅ 16a+ — modes Handles / Smooth (smooth by clicks) / Freehand; closed — face, open — edges; three separate tools — Bezier Curve, Smooth Curve, Freehand, Bezier Curve in the toolbar; a toolbar button (2026-10-19).
   - ✅ Icons in the menu; Preferences → Toolbar — the order of toolbar buttons and the reserve (2026-10-19).
   - ✅ Icons for all File / Edit / Tools / Snapping commands; any command can go on the toolbar via the reserve (2026-10-19).
   - ✅ Deleting library folders (including non-empty ones) with confirmation and a warning (2026-10-19).
   - ✅ Icons in View and Window; Window commands on the toolbar; the reserve by categories (2026-10-19).
   - ✅ 16b — editing a drawn curve: control points are stored next to the edges (`Mesh::splines`), the Draw → Edit Curve tool (double click on a curve): drag points and handles, add a point by click, delete with Delete, corner ↔ smooth by double click, segments; Entity Info of a curve (2026-10-20).
   - ✅ Follow Me along curves: sections are no longer inverted (a profile at an angle / wider than the bend), edges along curves are soft (2026-10-20).
   - ✅ A new document opens in the standard Iso view (2026-10-20).
   - ✅ Modifiers in hints by platform: macOS — Option / Cmd / ⌘, Windows — Ctrl / Alt (`keys.rs`) (2026-10-20).
   - ✅ Purge Unused right in Materials / Components: a button next to "In Model" and an item in the "…" menu (2026-10-20).
   - ✅ Licences: MIT for Varstotas, Help → Licences… (components, formats, online libraries), the source's terms above the catalogue (`@notice` for plugins), a warning about a site's "own" licence on placing and export; the Send to Varstotas bookmarklet removed (2026-10-20).
   - ✅ COLLADA (DAE) export: File → Export → COLLADA (DAE)…, mm and Z up, a node per group, materials and textures (files alongside); we do not make FBX export — a plugin `@export fbx` if needed (2026-10-20).
   - ✅ The name Varstotas: crates, binary, `~/.varstotas`, the `.vrst` / `.vrstmat` format, git history without the old name, author Dirondin (2026-10-20).
   - ✅ App icon: a workbench corner (half wood, half translucent), axes, sky / ground; `assets/icon.png` from `scripts/make_icon.py`, window / Dock / `.icns` / `.ico` (2026-10-20).
   - ✅ Slice tool: a plane by three points / face / axis cuts the selection or context (edges along the cut, a shared component made unique), Split (copy modifier) — two closed groups (2026-10-20).
   - ✅ The Model Info button in the top right corner of Entity Info (2026-10-20).
   - ✅ Hide in Components / Materials — a crossed-out eye icon instead of the word (2026-10-20).
   - ✅ Localization: crate `varstotas_i18n` (Fluent, `locales/en.ftl` / `ru.ftl` / `lt.ftl`), every UI text by key, platform variants, plural forms, Preferences → Interface → Language, user translations in `~/.varstotas/locales/`, localized plugin headers (`@name.ru` …); Russian and Lithuanian translations (2026-10-21).
   - ✅ Make Unique: a component with parameters has its own wizard, unique generated ones, an item in the context menus; re-inserting from the library after a model edit gives the original (2026-10-20).
   - ✅ Document generators are always in In Model with their own script (Apply rebuilds all instances), Make Unique copies the generator ("Shelf 2") (2026-10-20).

## Quality (parallel with M4–M7)
- Topology robustness on large and "dirty" models, precision (f64 up to the renderer).
- Kernel fuzz tests, benchmarks and clippy/tests in CI, application auto-update.

## Postponed (owner's decision after the macOS functionality is ready)
- **Platforms:** Linux, web (WASM + WebGPU), iPad, collaborative editing.

## Open questions (to be decided in the future)
- Intersect Faces of two groups in empty space: currently only edges are created (faces — only where the loop lies in an existing face). Options: keep or create faces from closed flat loops (a switch in `imprint_segments`, `Classify::None` → `Classify::NewEdges`). The owner's decision — later.
- Release bundles (`.app`/`.exe`) are rebuilt only at the owner's request; for checking — `target/release/varstotas`.
