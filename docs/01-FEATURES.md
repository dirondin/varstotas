# Feature inventory and plan

Priorities: **P0** — MVP (M1–M3), **P1** — second wave (M4–M5), **P2** — later/optional.
Status: ⬜ not started · 🟨 in progress · ✅ done.

## 1. Camera navigation
| Function | Behaviour | Prio. | Status |
|---|---|---|---|
| Orbit (O, middle button) | rotation around the point under the cursor | P0 | ✅ |
| Pan (H, Shift+middle) | shift in the screen plane | P0 | ✅ |
| Zoom (Z, wheel) | zoom to the cursor | P0 | ✅ |
| Zoom Extents (Shift+Z) | fit the whole model | P0 | ✅ |
| Zoom Window | zoom into a box | P2 | ⬜ |
| Standard views (Top/Front/Right/Back/Left/Bottom/Iso) | | P0 | ✅ |
| Perspective / Parallel projection | switching; also the view drop-down in the top right corner of the viewport (together with the display style and the Show menu: guides, axes, ground, sky (off — a plain light background), grid, shadows, sections; Field of View there too) | P0 | ✅ |
| Several viewports 1 / 2 / 4 (as in 3D editors) | View → Viewports or the view widget; each pane has its own camera and style, the active one is chosen by click; presets (Front + Iso, Front, Left, Top + Iso …) | P2 | ✅ |
| Field of view, Walk/Look Around/Position Camera | | P2 | ✅ |
| Previous/Next view | | P2 | ⬜ |

## 2. Drawing
| Tool | Behaviour | Prio. | Status |
|---|---|---|---|
| Line (L) | chain of segments; click-click; Measurements box length; auto-face on closing; splitting faces/edges | P0 | ✅ |
| Rectangle (R) | 2 corners, on the plane under the cursor or along an axis; Measurements box `w,h`; square/golden ratio inference | P0 | ✅ |
| Rotated Rectangle | 3 points, arbitrary plane | P1 | ✅ |
| Circle (C) | centre + radius; Measurements box radius, `24s` — number of segments; orientation by the face | P0 | ✅ |
| Polygon | centre + radius, N sides | P0 | ✅ |
| Arc (2 Point, A) | chord + bulge; tangent inference | P0 | ✅ |
| Arc (centre+2 points), 3 Point Arc, Pie | | P1 | ✅ |
| Bezier Curve (cubic splines: click — corner, drag — handles; smooth by clicks; freehand; closed — face, open — edges; separate Smooth Curve and Freehand tools; Enter / double click, closing on the first point; one curve of edges; toolbar button; editing after drawing — Edit Curve: points and handles, add / delete a point, corner ↔ smooth, segments) | usually a plugin | P2 | ✅ |
| Freehand | polyline following the cursor | P2 | ⬜ |

## 3. Editing
| Tool | Behaviour | Prio. | Status |
|---|---|---|---|
| Select (Space) | click; double click — face+edges; triple — everything connected; box left-to-right (window) / right-to-left (crossing); Shift/Ctrl modifiers | P0 | ✅ |
| Eraser (E) | deleting edges (with faces), by dragging; Shift — hide, Ctrl — soften | P0 | ✅ |
| 3D primitives: Draw → Shapes — Box, Cylinder, Cone, Pyramid, Sphere, Torus; base and height by mouse or in Measurements, `Ns` — segments; the result is a generated component, sizes are edited in Properties | P2 | ✅ |
| Push/Pull (P) | extrude/push in a face; Measurements box distance; double click — repeat the last; Ctrl — new start face; through subtraction up to the opposite face | P0 | ✅ |
| Move (M) | moving the selection/vertices/edges/faces with stretching of connected geometry; Ctrl — copy; `5x` / `/5` — array | P0 | ✅ |
| Rotate (Q) | protractor: centre, zero angle, angle; Measurements box degrees; Ctrl — copy, `Nx` — circular array | P0 | ✅ |
| Scale (S) | bbox handles; uniform/along an axis; Measurements box factor | P1 | ✅ |
| Offset (F) | offset contour of a face/edges inward/outward | P0 | ✅ |
| Sweep | extruding a profile along a path; along curves — smooth edges, no inverted sections | P1 | ✅ |
| Chamfer / Fillet | a flat chamfer and rounding of the edge between two faces | P2 | ✅ |
| Intersect Faces | intersection with the model/selection | P1 | ✅ |
| Slice | a plane (3 points / face / axis) cuts the selection or context; Split — two closed groups | P2 | ✅ |
| Flip Along / Reverse Faces / Orient Faces | | P1 | ✅ |
| Soften/Smooth edges | | P2 | ✅ |
| Solid tools (Union/Subtract/Intersect/Trim/Split) | Outer Shell, Intersect, Union, Subtract, Trim, Split of solid groups; Properties: volume | P2 | ✅ |

## 4. Model structure
| Function | Behaviour | Prio. | Status |
|---|---|---|---|
| Sticky geometry (vertex/edge/face, auto-faces, splitting) | model kernel | P0 | ✅ |
| Groups (Ctrl+G) | geometry isolation, double click — enter editing, the rest fades | P0 | ✅ |
| Components | definition + instances; editing one changes all; Make Unique (right click, Properties, Outliner; a component with parameters has its own wizard); re-inserting from the library after an edit in the model gives the original as a new definition | P1 | ✅ |
| Script Console (Rhai): API for building/editing the model, an editor with highlighting, autocompletion and parameter hints, a reference right in the application (Help → Script Reference, "? Reference" in the editors, F1 — the word under the cursor), a shared error log (Window → Log, a red counter in the status bar), script parameters (`@param` / `@pick` / `@select`), File → Run Script…, one Undo step per run | `.rhai` scripts, `docs/SCRIPTING.md` | P2 | ✅ |
| External API: Python client via a local bridge, MCP server for AI agents | `python/varstotas.py`, `varstotas_mcp.py`, `.py` from File → Run Script… | P2 | ✅ |
| Plugins: `.rhai` / `.py` plugins: menus, parameter dialog, point picking, importers/exporters | `docs/PLUGINS.md` | P2 | ✅ |
| Local component library (thumbnails, search, placing, saving the selection) | Components section in the tray; a preview image in the file (File → Preview Image, viewing — Show… and Document Info; right click on a thumbnail, Set Preview from View — the current view for a library file or a document component; for 3D Library models — the source's image), without it — a Shaded with Textures render (cache `~/.varstotas/cache/thumbs`); Rename… for document components and library files; a frame shared with Materials: search, +, cloud (3D Library), "…", sections In Model / Favourites / Recent / My Library, Small / Large / List tiles; favourites ☆, recents, folders from the tray (New Folder…, Move to Folder, Rename Folder…, Delete Folder… — with confirmation, a non-empty folder is deleted with its files after a warning), dragging a tile into the viewport — the component lands at the drop point | P2 | ✅ |
| Online models | Window → 3D Online Library or the cloud in the Components card: Poly Haven, Smithsonian, NASA (glTF, including Draco), import by link; what is downloaded becomes a component in the library with attribution (Properties → Source, File → Export → Attributions); Kenney sets (a pack → a folder of components) and Quaternius (via the browser); filters by licence / format / triangles, favourites, a history of downloads with search, cancelling a download; sites without an API: "Search on" by templates in `~/.varstotas/sources.toml`, a "Downloads" watcher, dragging a file or link, an Import Download window with the licence and duplicates by hash; warnings about NC / SA / ND / the site's "own" / unknown licences on placing and export, the source's terms above its catalogue (for plugins — `@notice`), Help → Licences…, `{credits}` on sheets; custom sources and formats — `@model_source` / `@model_import` plugins (examples: a folder of models, Poly Pizza, Sketchfab, PLY) | P3 | ✅ |
| Materials browser: material libraries across documents, saving / loading a material to a file | Materials in the tray, the same frame as Components: sections In Model / My Library (`~/.varstotas/materials` or your own folder, `.vrstmat`, subfolders); Rename…, Save to Library, Export…, + → Import…, Show in Finder; favourites, recents, folders, dragging onto a face; Delete for a model material (with Undo); a library material enters the model only on the first painting | P2 | ✅ |
| Online material library | the cloud in the Materials card: PBR scans from Poly Haven and ambientCG (CC0, 1K maps at real size) are downloaded into the library; custom sources — `@material_source` plugins (the Source drop-down), importing foreign formats — `@material_import` plugins (example: Unity `.mat`) | P3 | ✅ |
| Parametric components | size X/Y/Z by stretching or scaling, a row of copies with a gap, in Properties | P2 | ✅ |
| Generators: a Rhai script `build(p)` with typed parameters (length, integer, number, checkbox, choice, angle, percent, material; range and step), fields in Properties, shared definitions for equal values, Scale changes width / depth / height, the source in the document, generators in the common list and the component catalogue | `docs/GENERATORS.md` | P2 | ✅ |
| Parameters of ordinary components (imported or drawn): parameters, formulas, part motions (rotation around an axis, shift, show / hide, painting) from a parameter or an expression, a custom `update(p)` script inside the component with the full scripting API (`child(name, depth)`, dimensions `part.size` / `bounds`); values per copy (sliders in Properties), equal values — a shared variant; edits inside a copy go to the master; values in scenes with smooth transitions and in video; saved to the library and carried to other documents (Window → Component Parameters) | `docs/GENERATORS.md` | P2 | ✅ |
| Generators: formulas (`@formula`) and checks (`@check`) between parameters, an editor with live preview (Window → Generator Editor), examples of a staircase, a cabinet, a railing, a table | `docs/GENERATORS.md` | P2 | ✅ |
| Explode | ungroup | P0 | ✅ |
| Outliner | tree of groups/components; context menu (Zoom To, Edit, Hide, Explode, Move Out of Group, Delete), double click — focus on the object, dragging rows between groups and to the top level (undo) | P1 | ✅ |
| Tags (Layers) | visibility (and for Untagged: top level only), colour, renaming, deletion (contents → Untagged) | P1 | ✅ |
| Hide / Unhide / Lock | | P1 | ✅ |
| Properties | properties of the selection: area, length, tag, material | P0 | ✅ |

## 5. Construction and annotations
| Tool | Behaviour | Prio. | Status |
|---|---|---|---|
| Tape Measure (T) | measuring; Ctrl — guide line/point; Measurements box — resize the model | P0 | ✅ |
| Guides (lines/points) | selection, Delete, Move/Rotate/copy, Edit → Delete Guides, hide/show | P1 | ✅ |
| Protractor | angular guides | P1 | ✅ |
| Guides: Delete All | | P0 | ✅ |
| Axes | redefine the drawing coordinate system | P1 | ✅ |
| Dimensions | linear dimensions | P1 | ✅ |
| Text / 3D Text | callouts / solid text | P2 | ✅ |
| Section Plane | cutting plane: active cut, lines and fill, Reverse/Align View, in scenes | P2 | ✅ |

## 6. Inference engine
| Type | Colour/marker | Prio. | Status |
|---|---|---|---|
| Endpoint | green dot | P0 | ✅ |
| Midpoint | cyan dot | P0 | ✅ |
| On Edge | red square | P0 | ✅ |
| On Face | blue diamond | P0 | ✅ |
| Center (circle/arc) | | P1 | ✅ |
| Intersection | | P1 | ✅ |
| On Red/Green/Blue Axis (direction) | coloured line | P0 | ✅ |
| Parallel / Perpendicular to edge | magenta line | P0 | ✅ |
| From Point (dashed line from a point along an axis) | dashed line | P0 | ✅ |
| Tangent at vertex (arcs) | cyan | P1 | ✅ |
| Shift — lock inference; arrows ↑→← ↓ — axis locking | | P0 | ✅ |
| Guide points / guide lines | | P0 | ✅ |
| Preferences: a window with sections and search — General, Interface (light / dark / system theme, scale 80–150 %, toolbar icon size), Drawing (grid, snaps, steps), Libraries (folders, tiles), Downloads (watched folder), Shortcuts (remapping hotkeys of commands and tools, reset), Render (render of a new document, paths to ffmpeg and OIDN), the number of Undo steps, Navigation (orbit and zoom speed, inversion, zoom to cursor or centre, middle button, two-finger gesture), View (view of a new document: style, edges, axes, ground, sky, grid, shadows, projection, FOV); Reset per section; the menu items remain | | P2 | ✅ |
| Localization | every UI text from `locales/<lang>.ftl` (Fluent): English, Russian, Lithuanian and Belarusian, Preferences → Interface → Language (as the system), platform variants of texts, plural forms, own translations in `~/.varstotas/locales/`; plugins localize their headers (`@name.ru`, `@param.ru` …) | P2 | ✅ |
| Snapping menu (magnet, the Snapping menu after Tools): each snap on/off, a common Shift+S switch, saved in the settings | | P1 | ✅ |
| Grid (View → Grid): step, major lines, thinning when zooming out; snapping to nodes and lines, directions stop at grid lines | grey dot / square | P1 | ✅ |
| Work planes (Guide Plane, Window → Work Planes): by three points, by a face, by a normal with an arrow; the active plane — drawing axes and grid, points "in the void" on it; a gizmo (origin, tilt, rotation), angles in the window | orange frame | P1 | ✅ |
| Increments (in the magnet menu): Move / Copy (10 mm), Rotate and protractor (15°), Scale (0.1); snaps to geometry and input in Measurements take priority over the increment | | P1 | ✅ |

## 7. Measurements box
- Input without focus: just type digits while a tool is active — P0.
- Units: mm / cm / m / inches / feet (`1'6"`), comma/dot — P0.
- Repeated input corrects the last operation — P0.
- Arrays `Nx`, `/N`, segments `Ns` — P0/P1.

## 8. Appearance
| Function | Prio. | Status |
|---|---|---|
| Materials: colour, transparency; Paint Bucket (B) + eyedropper (Alt) | P0 | ✅ |
| Textures (images): a material from PNG/JPEG, tile size, position on a face (offset, rotation, scale), embedding in `.vrst`, OBJ with `vt`/`map_Kd` (export and import), the Texture Position tool (pins), the Shaded without textures style | P1 | ✅ |
| Styles: edges, profiles (thick silhouettes), X-ray, Wireframe, Hidden line, Shaded, Monochrome | P0 (shaded+edges+X-ray), P1 the rest | ✅ |
| Shadows (directional light, time of day) | P2 | ✅ |
| Realistic style (track R): metallic-roughness PBR materials (metal, roughness, glass, emission, normal / roughness / metallic / AO / emission maps), environment lighting (sky, colour, HDRI with rotation), sun, exposure, ACES / AgX; environment in scenes, glTF with physical materials | P2 | ✅ |
| Realistic effects (track R): soft shadows, AO, SSR reflections, bloom, depth of field, fog, vignette, presets; Pencil / Watercolor / Technical styles, Clay; custom WGSL material shaders with parameters and presets (wood, water, ribbed glass, glow) | P2 | ✅ |
| Material layers: clear coat, fabric sheen, anisotropy (brushed metal), texture anti-tiling, bevel — edge rounding in the render; Physical → "Coat, Sheen, Edges", glTF KHR_materials_clearcoat / sheen / anisotropy | P2 | ✅ |
| Path-traced render (track R): Window → Render — a progressive GPU path tracer (BVH, PBR, glass with refraction, emission, HDRI / physical sky with sun, ground), point / spot / area lights, denoising (Intel OIDN, if installed), hardware ray query where available, PNG / EXR, rendering all scenes | P2 | ✅ |
| Light sources in the scene: always visible (marker, spotlight cone, area-light rectangle), selected by click, Move / Rotate (with axis locking, Measurements box, copying), an aiming handle (drag onto an object), properties in Properties, Delete, undo | P2 | ✅ |
| Render settings: quality presets (Draft / Preview / Final / Best), frame sizes, transparent background, firefly clamp, denoiser choice, depth of field (aperture f/, focus), sun / sky / exposure, on-the-fly colour correction (exposure, tone curve, white balance, tint, saturation, contrast, vignette) | P2 | ✅ |
| Demo model "country house" (repository `examples/demos/country_house.vrst`, Help → Examples → download → File → Open): a furnished house with PBR materials, procedural textures, light sources and scenes for checking visualisation | P2 | ✅ |
| Demo "material gallery" (repository `examples/demos/material_gallery.vrst`): 24 cubes of different materials (metals, glass, neon, textures, shaders) and 8 scenes with different environments and render styles | P2 | ✅ |
| Shader editor: WGSL highlighting, hints for Surface fields / parameters / swizzles, call hint, reference, templates | P2 | ✅ |
| Animation export (File → Export → Animation…): a walk through the scenes or a fly-around to MP4 (ffmpeg) or a PNG sequence; frames as a view or path-traced | P2 | ✅ |
| Front/back side of faces (white/grey-blue) | P0 | ✅ |
| Ground grid, axes, horizon/sky | P0 | ✅ |
| Scenes (saved views, style, tag visibility) + tabs and animation (Play, PgDn/PgUp, transition time and pauses) | P1 | ✅ |

## 9. Files and other
| Function | Prio. | Status |
|---|---|---|
| Own `.vrst` format (save/open, autosave) | P0 | ✅ |
| Export OBJ, STL | P0 | ✅ |
| Import OBJ / STL | P1 | ✅ |
| Export PNG (view screenshot) | P1 | ✅ |
| Import DXF / Export DXF | P2 | ✅ |
| glTF / GLB: import and export | P2 | ✅ |
| Export 3MF for 3D printing | P2 | ✅ |
| Import 3MF, COLLADA (DAE), FBX (ufbx); export COLLADA (DAE) with materials and textures; PLY import and export; points from CSV, faces in JSON | P2 | ✅ |
| 2D drawing of the current view → SVG / PDF (profiles, hidden lines, sections, dimensions, scale 1:N; light sheet layout) | P2 | ✅ |
| Sheets (light sheet layout): scene viewports at scale 1:N or fitted, texts, title block, multi-page PDF; sheets are stored in the document | P2 | ✅ |
| Standard views on a sheet with one command (3 projections + iso, standard scale), automatic overall dimensions, a parts table | P2 | ✅ |
| Report: parts of groups/components with sizes for cutting, areas by material (CSV) | P2 | ✅ |
| Unfold of faces / groups → SVG / PDF / DXF: mountain/valley folds, flaps, edge numbers, scale, sheets | P2 | ✅ |
| Physics for still scenes (Window → Physics): drop the selected groups / components under gravity onto the model, or let a face fall as cloth and drape over it (a group of smooth triangles); live preview, Apply as one undo step (`PHYSICS-PLAN.md`) | P2 | ✅ |
| Undo/Redo (Ctrl+Z / Ctrl+Y / Cmd+Shift+Z) | P0 | ✅ |
| Copy/Paste/Paste in place | P1 | ✅ |
| Model units, Document Info (Window → Document Info: polygons / triangles / edges / groups / components in the scene and in the file, components / groups / materials "used across everything", textures, tags, scenes, libraries; Purge Unused — also Edit → Purge Unused and a button next to "In Model" / the "…" menu in Materials and Components) | P0 | ✅ |
| Templates: built-in unit presets plus `.vrst` files from `~/.varstotas/templates` or a custom directory (Preferences → General); New from Template creates an untitled copy, Open Templates Folder reveals the directory | P1 | ✅ |
| Tool Guide / tool hints; application help (Help → User Guide: topics, search, tool cards, F1 — the topic of the active tool) | P0 (status bar), P1 (panel) | ✅ |
