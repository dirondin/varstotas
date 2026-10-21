# Plugins and the component library

## Plugins
A plugin is a `.rhai` or `.py` file in the `~/.varstotas/plugins/` folder (subfolders are scanned too, two levels deep). Plugins are loaded at startup; after editing, choose **Extensions → Reload Plugins**. **Open Plugins Folder** is there too.

Examples are in `examples/plugins/`. At every launch the application copies them to `~/.varstotas/plugins/examples/` — keep your own plugins in another subfolder, otherwise they will be overwritten:
- `stairs.rhai` — a staircase: parameters and one point.
- `beam.rhai` — a beam between two points.
- `import_points_csv.rhai` — import a contour from CSV ("Points from CSV (plugin)"; the format is built in: File → Import → Points (CSV)).
- `export_faces_json.py` — export faces to JSON in Python ("Faces as JSON (plugin)"; the format is built in: File → Export → Faces (JSON)).
- `unity_material.rhai` — import Unity materials (`.mat`), see "Material plugins".
- `texture_folders.rhai` — a material source from folders of textures on disk.
- `ambientcg_python.py` — a material source in Python: search ambientCG.

### Header
A plugin's description is comments at the start of the file (`//` for Rhai, `#` for Python). The header ends at the first line that is not a comment.

```rhai
// @name Stairs
// @menu Construction/Stairs…
// @description A straight flight of treads rising along +Y from the clicked point.
// @param width = 900 "Width"
// @param steps:int = 15 "Steps"
// @pick 1 "Click the bottom front corner of the stairs"
```

| Tag | Meaning |
|---|---|
| `@name` | the name (by default — the file name) |
| `@menu A/B/C` | the path in the **Extensions** menu (by default — the name) |
| `@description` | a hint and the text in the dialog |
| `@param key = default "Label"` | a parameter the user enters in a dialog before the run. A number by default is a length: it can be entered with units (`1.2m`), and arrives in the script in mm. The type is set explicitly like this: `key:int`, `key:number`, `key:text`, `key:bool`, `key:length`. A quoted string is text, `true` / `false` is a checkbox. The last entered values are remembered until the end of the session. |
| `@pick N "Prompt"` | before the run the user clicks N points in the model, with snapping. They arrive in the `points` array (world mm). |
| `@select "Prompt"` | before the run (after the points) the user selects objects by clicking: a face, edge or group of the open context is added to or removed from the selection, Enter — continue, Esc — cancel. The objects arrive in the `objects` array (as script entities, in the same order as in the selection); what was already selected before the run is included too. |
| `@import csv,txt` | an importer: an item in **File → Import**, the file path arrives in `path`. Such a plugin does not appear in the Extensions menu. |
| `@export json` | an exporter: an item in **File → Export**, the path arrives in `path` |
| `@material_import mat,material` | a material importer: **Materials → Import…** reads these files with the plugin (see below) |
| `@material_source` | a material source in the **Online Materials** window (see below) |
| `@model_import ply,…` | a 3D model importer: an item in **File → Import**, these files are also read by the **3D Library** (see "Model plugins") |
| `@model_source` / `@model_source search` | a model source in the **3D Library** window: the whole catalogue at once / search on the site's side |
| `@notice text` | the source's terms (licence, whether a key or account is needed, whom to credit as the author): shown with ⚠ above the catalogue in **3D Library** / **Online Materials**. Several lines are allowed, they are joined with a space. |

Parameters, `points`, `objects` and `path` are ordinary script variables. In the Script Console editor they are highlighted in dark teal and are in the autocompletion list: you can see that the value comes from the header.

| Variable | From | Contents |
|---|---|---|
| a `@param` key | the dialog | a number (a length — in mm), text or `true` / `false` |
| `points` | `@pick N` | an array of N points (world mm) |
| `objects` | `@select` | an array of entities (faces, edges, groups) |
| `path` | `@import` / `@export` | the path of the chosen file |

The same header works in the **Script Console** code: Run with `@param` / `@pick` / `@select` goes through the dialog, clicks and selection, then runs the console code:

```rhai
// @param h = 200 "Height"
// @select "Select faces to raise"
for o in objects { pushpull(o, h); }
```

### Rhai
Running a Rhai plugin is one Undo step named after the plugin; if the plugin fails with an error, the model is not changed. `print` output goes to the Script Console, the last line appears in the status bar. For importers and exporters the API has `read_text(path)`, `write_text(path, text)`, `file_exists`, `list_files(dir, ext)`, `list_dirs(dir)`, `parse_json(text)`, `fetch_text(url [, headers])`, `fetch_file(url, path [, headers])` (headers — a map of HTTP headers, for example `#{ "x-auth-token": key }`), `url_encode(text)` and `all_faces()` — all faces of the model, including nested groups (`f.vertices` returns world coordinates).

### Python
A Python plugin is run the same way as File → Run Script…: through `python3`, the output goes to the Script Console. Values arrive through `varstotas.params()`:

```python
# @name Faces as JSON
# @export json
import varstotas
m = varstotas.connect()
path = varstotas.params()["path"]
```

Make model changes inside `with m.operation("…")` so that they become one Undo step.

### Material plugins
Two kinds of plugins, they are not in the Extensions menu.

**Importer** (`@material_import ext,…`). The plugin's extensions appear in the filter of the **Materials → Library → Import…** dialog ("All materials" and a separate filter for the plugin). The materials from the file are added to the document; the last one becomes current for Paint. Such a plugin's `@param`s take their default values.

**Source** (`@material_source`). The plugin appears in the **Source** drop-down of the **Online Materials** window (after Poly Haven and ambientCG). Below the list are the plugin's description and its `@param`s. The **Load** button (or Enter in the field) requests the catalogue again with these values. Parameters with `folder` / `dir` / `path` / `file` in the name have a "…" button for choosing, and such a path is passed as absolute. After that everything is as with the built-in sources: search, categories, previews, Download & Use / Download / Use / Web Page, double click. What is downloaded is saved to the library, in `<library>/<plugin name>/`.

The plugin's functions in Rhai (the code runs separately from the model, in the background; files and network are handled by API functions):

```rhai
// @name My Site
// @material_source
// @param api_key:text = "" "API key"
fn catalogue(params) {            // → an array of materials
    let list = parse_json(fetch_text("https://example.com/api?key=" + params.api_key));
    list.map(|a| #{ id: a.id, name: a.title, category: a.kind, tags: a.tags, width: 2000,
                    thumb: a.preview, page: a.url, license: "CC0" })
}
fn download(id, dir, params) {    // → a material specification
    fetch_file("https://example.com/files/" + id + ".zip", dir + "/m.zip");
    #{ zip: "m.zip" }
}

// importer: @material_import mat
fn import_material(path, params) { … }   // → a specification or an array of specifications
```

Material fields in the catalogue: `id` is required; then `name`, `category`, `keywords` / `tags` (a string or an array, search works on them), `width` (the real width of the texture, mm), `thumb` (a URL or a path to an image), `page` (the page for Web Page), `license`. `dir` in `download` is the working folder `~/.varstotas/cache/online/<plugin>/<id>`, relative paths in the reply are counted from it. For an importer, relative paths are counted from the file's folder.

**Python**: `python3 plugin.py catalogue|download|import`. Arguments arrive as a JSON object on stdin: `params`, `id`, `dir`, `path`. The reply is JSON on the last line of stdout; other lines go to the log. An error is a non-zero exit code, the text is the last line of stderr (`sys.exit("message")`). Example: `ambientcg_python.py`.

**Material specification** — a JSON object (in Rhai — a map):

| Field | Meaning |
|---|---|
| `name` | the name (by default — the name from the catalogue / file) |
| `width` | the texture width, mm |
| `color` | `"#rrggbb"` or `[r, g, b]` (0..1, or 0..255); with a colour map by default — the map's average colour |
| `opacity` | 0..1 |
| `maps` | `{ color, normal, roughness, ao, metallic, emission }` — paths to images (PNG, JPEG, TGA, BMP; normal — OpenGL); `texture` — the same as `maps.color` |
| `folder`, `zip` | a folder / zip archive whose maps are recognised by name: `_color` / `_diff` / `_albedo` / `_basecolor`, `_normal` / `_nor_gl` / `_nrm`, `_rough`, `_ao` / `_occlusion`, `_metal` / `_metalness`, `_emission`. DirectX normals (`_dx`) and height maps are skipped; if there is no colour map by name, the first unrecognised image becomes it |
| `metallic`, `roughness`, `transmission`, `ior`, `emission` (colour), `emission_strength`, `normal_scale`, `clearcoat`, `clearcoat_roughness`, `sheen`, `sheen_tint`, `anisotropy`, `anti_tiling`, `bevel` (mm) | physical parameters (Physical in the Materials card) |

An example importer is `unity_material.rhai`. It reads `.mat` (Standard / URP / HDRP Lit): `m_Name`, `_BaseColor` / `_Color`, `_Metallic`, `_Smoothness` / `_Glossiness` (roughness = 1 − smoothness), `_BumpScale`, `_EmissionColor` and the textures `_BaseMap` / `_MainTex`, `_BumpMap`, `_OcclusionMap`, `_EmissionMap`. Textures are looked up by GUID in the `.meta` files of the project's `Assets` folder. The texture width is set by the `width` parameter. PSD / TIFF and `_MetallicGlossMap` (metal in the R channel) are not carried over.

### Model plugins (3D Library)
The **Window → 3D Library…** window (or the cloud in the top row of the Components card) searches for and downloads ready-made models. Built-in sources: **Poly Haven** (CC0, glTF with 1K textures, real size), **Smithsonian** (CC0, scans of museum objects; search on the server — Enter), **NASA** (GLB from the NASA-3D-Resources repository). What is downloaded is imported, wrapped into one component, put on the ground (bounding box from the origin) and saved to the component library: `<library>/<source>/<name>.vrst`. Download files are in `~/.varstotas/downloads/<source>/<id>/`. Double click — download and place (by a click in the model, like a component from the library), the buttons Download & Place / Download / Place / Download Again / Web Page. The **From URL** field downloads a direct link to a `.glb` / `.gltf` / `.obj` / `.stl` / `.dxf` / `.zip` (library folder "Web"). Models over 100 thousand triangles are marked ⚠. glTF with Draco compression is read (the `draco-core` decoder), meshopt is not.

A component remembers its source: name, author, licence, link (Entity Info → the Source row). **File → Export → Attributions…** saves the list of credits of all downloaded components of the model — needed for CC-BY.

**Source** (`@model_source`): `fn catalogue(params)` → an array of models, search and categories filter it in the window. With `@model_source search`, instead of it `fn search(query, params)` — called on Enter in the search field, the reply is shown as is. In both cases `fn download(id, dir, params)` → a model specification. Model fields in the catalogue: `id` is required; then `name`, `category`, `keywords` / `tags`, `author`, `license`, `thumb` (a URL or path), `page`, `polycount` (triangles), `size` (`[x, y, z]`, mm), `bytes`. Parameters and the Load / Apply button — as with material sources.

**Importer** (`@model_import ext,…`): `fn import_model(path, params)` → a model specification. The "<plugin> (.ext)…" item in **File → Import** inserts the model as a component; the 3D Library imports downloaded files with this extension with the plugin (including from a zip and from another plugin's reply, up to three levels).

**Model specification** — a JSON object (in Rhai — a map); relative paths are from `dir` (for an importer — from the file's folder):

| Field | Meaning |
|---|---|
| `file` | the model file: `glb`, `gltf`, `obj`, `stl`, `dxf`, `varstotas`, `zip` or the extension of a `@model_import` plugin |
| `url` | download the file from a link (a zip is unpacked) |
| `zip` | an archive: it is unpacked (paths with `..` are dropped), the first model in the order glb → gltf → obj → stl → dxf → varstotas → importer files is taken |
| `meshes` | geometry right in the reply: `[{ name, vertices: [[x,y,z], …], faces: [[i,j,k,…], …], material }]`, each one is a group; `materials` — material specifications (as for material plugins) |
| `unit` | `m` / `cm` / `mm` / `in` / `ft` — the units of `meshes` or of a file without units (OBJ, STL); or `scale` — a multiplier to mm |
| `up` | `"y"`: the file's Y axis is up |
| `name` | the component name |
| `preview` | a preview image (a path or URL); without it the catalogue's `thumb` is used. It is saved in the file and visible in the Components card |

Python plugins: `python3 plugin.py catalogue|search|download|import_model`, arguments (`params`, `query`, `id`, `dir`, `path`) — JSON on stdin, the reply — JSON on the last line of stdout. Examples in `examples/plugins`: `model_folders.rhai` (models from a folder on disk; an image with the same name is the preview), `poly_pizza.rhai` (Poly Pizza, a free API key, search), `sketchfab.py` (Sketchfab: search is open, download with a personal token), `import_ply.py` (a PLY importer, ASCII and binary; "PLY Import (Python) (plugin)" — PLY is now built in, the plugin is kept as an example). Unity `.mat` remains a plugin (the format is closed).

## Component library
The **Components** section in the Default Tray (enabled by a checkbox in the **Window** menu) shows the `.vrst` files from the library folder. By default this is `~/.varstotas/components`; the "…" button chooses another folder, "Default" in the root folder's header returns the default folder. Subfolders are shown as categories with a collapse arrow.

- **A click on a thumbnail** places the file's content as a component: it follows the cursor, with the corner of its bounding box at the snap point, a click fixes it. There is one definition in the document no matter how many times the component is placed (matched by file name). Esc cancels.
- **+ → Save Selection to Library…** saves the selection to `<folder>/<name>.vrst` under a new name if that one is already taken. The file's origin is the corner of the bounding box. If a single group or component is selected, its content goes into the file.
- The **Document** folder shows the components of the open document (unused ones too). Right click: Place, Select Instances, **Parameters…** (the Component Parameters window, see `docs/GENERATORS.md`), **Delete from Document…** — the component is deleted together with all placed copies (if there are copies, it asks; Edit → Undo brings it back). Everything unused is deleted at once by **Purge Unused** (Window → Model Info… or Edit → Purge Unused).
- The thumbnail is a wireframe in isometric view, and if the file has a **preview image** — that. Right click on a thumbnail: **Set Preview Image…** (PNG / JPEG / TGA / BMP; reduced to 512 px and written to the file) and **Remove Preview Image**. For the open document the preview is set in **File → Preview Image** (from a file or as a snapshot of the current view). Models from the 3D Library get a preview by themselves — the source's image. A hint above the thumbnail shows the dimensions. Search filters by name. The ⟳ button rereads the folder; changed files are also reread when the section is simply shown.

Parametric components built by a script (`@generator`) are described in `docs/GENERATORS.md`.
