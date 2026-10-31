# Scripts (Rhai)

Varstotas embeds the [Rhai](https://rhai.rs/book/) language — a simple scripting language with a syntax similar to Rust/JavaScript. Where to run scripts:

- **Window → Script Console** — an editor and an output log. Cmd+Enter runs, Cmd+↑/↓ browses history, Open…/Save… handle `.rhai` files. The editor text is kept between launches.
  - The editor highlights syntax (keywords, API functions, constants, strings, numbers, comments — dark green) and numbers the lines.
  - **Autocompletion** as you type: API functions, keywords, `ORIGIN/X/Y/Z` and words from the script itself. ↑/↓ choose, Tab inserts, Esc hides.
  - **Properties and methods after a dot** — by the type of what is on the left: `let g = box(…)` → `g.` offers `name`, `definition`, `material`…; `let f = rect(…)` → `area`, `normal`, `edges`…; `for e in edges()` → edge properties; `f.normal.` and `points[0].` → `x`, `y`, `z`, `dot(b)`, `distance(b)`…; arrays offer `len()`, `filter(…)`, and `bounds(x)` offers `min`, `max`, `size`, `center`. If the type is unknown, the properties of all entities are shown. The list shows the result type and a description; ↑/↓, Tab or Enter insert (methods — with the parenthesis), Esc hides.
  - **Script header:** `// @tag …` lines are highlighted (the tag like a keyword, then strings, numbers, `true`/`false`, the type after `:`). After `// @` a list of tags (`@name`, `@menu`, `@description`, `@param`, `@pick`, `@select`, `@import`, `@export`) with their syntax is offered, after `// @param key:` — the types (`length`, `int`, `number`, `text`, `bool`); while the cursor is in a tag line, its syntax is shown above it.
  - **Variables from the header** — the `@param` keys, `points` (with `@pick`), `objects` (with `@select`), `path` (with `@import` / `@export`) — are highlighted in dark teal so they stand out among ordinary variables, and are offered in autocompletion; their type is known to the after-the-dot completion (`points[0].` → `x`, `distance(b)`…).
  - **Parameter hint**: inside a call (`box(ORIGIN, 10, |`) the function signature is shown above the cursor, the current argument is underlined, and a short description is on the right. Signatures come from `varstotas_script::help`.
  - The strip between the log and the editor can be dragged: the editor becomes taller or shorter; the window can be resized too.
  - **Parameters as in plugins:** if the code starts with a `// @param …`, `// @pick N "…"` or `// @select "…"` header (see `docs/PLUGINS.md`), Run first opens the parameters dialog, then asks you to click points and select objects, and only then runs the code (one Undo step, "Script" or `@name`).
- **The in-app reference — Help → Script Reference.** Topics: Basics (language, objects, context), Console and plugins (the `// @…` header, parameter types), Generators (`build(p)`, `@param` / `@formula` / `@check`), Component script (`update(p)`, `child`, how a variant works), Functions (all API functions by section — from `varstotas_script::help`, a click on a signature copies it), Properties (properties and methods after a dot). Search covers functions, properties and tags. There is a "? Reference" button in the Script Console, Generator Editor and Component Parameters — the reference opens on its own topic; **F1** in any of these editors looks up the word under the cursor (or the function whose call the cursor is inside). Examples are copied with the Copy button.
- **Window → Log** — the common log: errors and warnings from everywhere (Script Console, plugins, generators, Component Parameters, import / export, libraries, downloads, everything that was shown in red in the status bar, as well as the application's own `log::warn!` warnings). Time, source, repeat count (the same message in a row within 5 s counts as one line ×N); filters Errors / Warnings and by text, Copy, Clear. Unseen errors are a red counter on the right of the status bar; a click opens the log.
- **File → Run Script…** — run a `.rhai` file. Example — `examples/scripts/bookshelf.rhai`.

Rules:
- One run is **one Undo step** ("Script"). If the script fails with an error, the model is not changed. A message with the line and position is written to the log.
- `e.locked` reads the effective editing lock (including tags and ancestor groups). Locked entities remain readable, but `select` excludes them. Scripts, installed plugins and Python/MCP operations cannot modify protected geometry, shared definitions or materials, or clear locks. A rejected edit rolls back the whole model operation. Bridge Undo/Redo also preserves current locks; interactive Undo/Redo remains available to the user.
- Coordinates are **world, in millimetres**. Numbers can be written as integers (`1000`) or fractions; lengths — as a string with units: `"1.2m"`, `"30cm"`, `"6\""`.
- New geometry is created in the **current context**: the group that was open when the script started, or the one entered through `enter(g)`.
- Rhai execution has a 30-second deadline and operation/allocation limits. Downloads share the deadline. Native geometry calls are synchronous and cannot be forcibly interrupted; this is not an OS sandbox.
- `print(x)` writes to the log. The value of the last expression is printed too.
- The Rhai string methods `trim()`, `to_upper()` and the like change the string in place and return `()`: write `s.trim(); if s == "" { … }`.
- Plugins (custom menu items, parameter dialogs, point picking, importers and exporters) are described in `docs/PLUGINS.md`.

Entities (faces, edges, groups) are "references". After operations that merge or cut geometry (a new face over an old one, Push/Pull all the way through), an old reference may become invalid. To check, use `e.valid`; using such a reference gives a clear error.

## Vectors
| Notation | Meaning |
|---|---|
| `vec3(x, y, z)`, `vec3(x, y)`, `[x, y, z]` | point / vector (an array is also accepted everywhere a point is needed) |
| `ORIGIN`, `X`, `Y`, `Z` | the origin and the unit axes |
| `a + b`, `a - b`, `a * k`, `k * a`, `a / k`, `-a` | arithmetic |
| `v.x`, `v.y`, `v.z` | components (can be assigned) |
| `v.length()`, `v.normalize()`, `a.dot(b)`, `a.cross(b)`, `a.distance(b)` | |
| `length("2.5m")` | parse a length with units → mm |
| `min(a, b)`, `max(a, b)`, `clamp(x, lo, hi)`, `ceil(x)` | numbers (integers and fractions mixed); the result is a fraction (`.to_int()` for an index or a byte) |
| `noise(x, y)`, `noise(x, y, period, seed)`, `fbm(x, y, period, octaves, seed)` | smooth value noise 0..1 on a unit grid; with `period > 0` it repeats every `period` cells (seamless tiles); `fbm` sums octaves |

## Construction
| Function | Result |
|---|---|
| `line(a, b)` | an array of edges |
| `polyline([p1, p2, …])`, `polyline(pts, true)` | edges (closed — with a face, if planar; a face lying flat faces up) |
| `face([p1, p2, p3, …])` | a face (a free face lying horizontally faces up) |
| `rect(origin, w, d)` | a rectangular face in the XY plane |
| `circle(center, r)`, `circle(center, r, segments)`, `circle(center, r, segments, normal)` | a circle face (or edges, if no face was formed) |
| `polygon(center, r, sides)`, `polygon(center, r, sides, normal)` | a polygon face |
| `arc(center, start_point, degrees)`, `…, segments)`, `…, segments, normal)` | arc edges |
| `box(origin, sx, sy, sz)` | a new box **group** (the minimum corner is `origin`; negative sizes extend in the other direction) |
| `cylinder(center, r, h)`, `cylinder(center, r, h, segments)` | a new cylinder group (bottom at `center`) |
| `cone(center, r, h)`, `cone(center, r, h, segments)` | a new cone group (base at `center`, apex `h` above) |
| `pyramid(origin, w, d, h)` | a new pyramid group: a `w × d` base from `origin`, apex `h` above its middle |
| `sphere(center, r)`, `sphere(center, r, segments)` | a new sphere group centred at `center` |
| `torus(center, r, tube)`, `torus(center, r, tube, segments, sides)` | a new torus group: `r` — to the middle of the tube, `tube` — the tube radius |
| `dimension(a, b, offset)` | a linear dimension between points, the extension is the `offset` vector |
| `note(at, offset, "text")` | a text callout |

## Operations
| Function | Result |
|---|---|
| `pushpull(face, d)` | the "cap" face (d > 0 — along the normal) |
| `offset(face, d)` | a new inner (d < 0) or outer face |
| `follow_me(face, edges)` | sweep a profile along a path |
| `erase(x)` | delete an entity / array |
| `move(x, v)`, `copy(x, v)` | move / copy by a vector; return entities |
| `rotate(x, center, axis, degrees)` | rotation |
| `scale(x, center, k)` or `scale(x, center, vec3(kx, ky, kz))` | scaling |
| `group(x)`, `component(x, "Name")` | group → group / component |
| `explode(g)`, `make_unique(g)` | |
| `union(a, b)`, `intersect(a, b)`, `subtract(a, b)` (a minus b), `trim(a, b)` (a minus b, b stays), `split(a, b)`, `outer_shell([…])` | Solid Tools on solid groups; return new groups |
| `reverse(faces)` | flip faces |
| `soften(x, degrees)` | soften edges (fold ≤ angle); `x` — edges or faces |
| `material("Name", "#RRGGBB")` or `material("Name", [r, g, b])` | create / recolour a material, return the name |
| `paint(x, "Name")`, `paint(x, ())` | paint faces / groups, remove the material |
| `tag(x, "Name")` | assign a tag (created if necessary) |
| `make_parametric(g)` | make a parametric group |
| `set_params(g, #{ x: 1200, y: (), z: 36, count: 3, axis: "y", gap: 100, stretch: true })` | parameters (any subset of keys; `()` = "as modelled") |
| `params(g)` | the current parameters (map) or `()`; for a generated component — the values of its generator |
| `add_generator(src)`, `generators()` | add or update a generator from source (returns the name), the document's generator names. The joints its `build` makes (`joint`, `drawers`, `doors`, `back`, `plinth`) come with every component it builds: each copy has their fasteners and hardware |
| `generate(name, at)`, `generate(name, at, #{ width: 1200 })` | place the generator's component at a point |
| `set_params(g, #{ shelves: 5 })`, `reset_params(g)` | change the values of a generated component (some of the keys) or return the defaults; see `docs/GENERATORS.md` |

## Context and queries
| Function | Result |
|---|---|
| `faces()`, `edges()`, `groups()`, `all()` | entities of the current context |
| `entities(g)` | everything inside a group |
| `child("Door")`, `child("Knob", true)`, `child(g, "Knob")`, `child(g, "Knob", true)` | the group / component with this name in the current context (or inside `g`); `true` — search nested ones too (breadth-first, the nearest); none — `()`. The name is the copy's own name, otherwise the definition name |
| `children()`, `children(true)`, `children(g)`, `children(g, true)` | groups / components of the context (or `g`); `true` — with nested ones |
| `show(x)`, `show(x, on)`, `hide(x)` | show / hide |
| `enter(g)`, `exit()` | enter a group / exit (to build inside) |
| `selection()`, `select(x)`, `select(())` | selection (in the run context) |
| `bounds(x)` | `#{ min, max, size, center }` in world coordinates |
| `model_bounds()` | the model's extents |
| `units()`, `materials()` | the model units, material names; `units("in")` sets the units the model is shown in: `"mm"`, `"cm"`, `"m"`, `"in"`, `"ft"` (coordinates stay millimetres) |
| `all_faces()` | all faces of the model, including nested groups (for exporters) |
| `read_text(path)`, `write_text(path, text)` | read / write a text file |
| `file_exists(path)`, `list_files(dir, ext)`, `list_dirs(dir)` | whether a file exists; files with an extension (`""` — any, with subfolders, up to 5000); folders inside |
| `parse_json(text)` | JSON → map / array / numbers / strings |
| `fetch_text(url)`, `fetch_file(url, path)` | download over the network (HTTP(S)): text / to a file |

Entity properties: `e.kind` (`"face"`, `"edge"`, `"vertex"`, `"group"`, `"component"`, `"parametric"`), `e.valid`, `e.area` (face, mm²), `e.length` (edge), `e.normal` (face), `e.center`, `e.vertices` (points), `e.edges` (edges of a face), `e.faces` (faces of an edge / neighbours of a face), `e.name` (group, can be assigned), `e.definition`, `e.material`, `e.tag`, `e.soft`, `e.hidden` (can be assigned), extents in world coordinates: `e.bounds` (`#{ min, max, size, center }`), `e.size`, `e.min`, `e.max`.

Instead of an entity, a string can be passed anywhere — the name of a child group / component of the current context: `move("Door", vec3(0, 10, 0))`.

## Cut list
| Function | Result |
|---|---|
| `set_stock("Plywood", #{ kind: "sheet", length: 2440, width: 1220, thickness: 18, grain: true, price: 60 })` | what a material is bought as; an array sets several items, `()` clears it. A board: `#{ kind: "board", width: 95, thickness: 45, lengths: [2400, "3 m"], price: 5 }` (the price per metre); per piece: `prices: [9, 16]`, one per length, or a single price for every length. Linear stock (mouldings, rods): `#{ kind: "linear", name: "Ø 20 rod", width: 20, thickness: 20, lengths: [1000, 2000], price: 3 }`, the section optional (any). Several items: thicknesses, board sections or sheet sizes to choose from |
| `cut_settings(#{ kerf: 3, trim: 10, min_offcut: 100, tolerance: 0.5, band: 1, band_price: 0.5, band_subtract: false })` | the document's cutting settings (any subset of keys); `band` is the edge band thickness, `band_subtract` cuts banded parts smaller by it |
| `band_visible()` | band the edges of the sheet parts that nothing covers: an edge is covered where other parts' faces lie on it (within 1 mm, as assembled) over at least half of it, such as a side's top edge under a top, the back edges under a back, the front edges behind an overlay door, a shelf's ends against the sides. Replaces those parts' `bands` (by part name; an edge visible on any copy counts); parts thinner than 10 mm, parts of board or linear stock and hardware are left out. Returns how many parts were banded |
| `cut_part("Shelf", #{ grain: "length", exclude: false, bands: [true, false, false, false] })` | per part name: grain `"length"`, `"width"` or `"any"` (may rotate), exclude from the cut list, edge-banded edges (the two long edges, then the two short ones); `#{ hardware: true, price: 2.5 }` buys the parts so named instead of cutting them (a group marked so is one part) |
| `cut_hardware([#{ name: "Screw 4 × 16", count: 24, price: 0.05 }])` | hardware not in the model, listed by hand (replaces the list) |
| `cut_list()` | `#{ parts, stock, banding, hardware, issues, cost }`: parts with name, count, length, width, thickness, material, `bands`, `hardware` and `holes` (the joints' holes in the part's face, `#{ x, y, diameter }` along its length and width); `banding` per part thickness (`#{ width, thickness, length, cost }`); `hardware` (`#{ name, count, price, cost, in_model }`); per stock item its material, kind, sizes, `count`, `buy` (`[#{ length, count }]`), `waste` (%), `cost` and `layouts` (pieces with part index, x, y, w, h, rotated) |

## Joinery
| Function | Result |
|---|---|
| `joint("Side", "Shelf", "dowels")` | joins two touching groups or components, cut into both; kinds `"dowels"`, `"screws"`, `"pocket"` (pocket holes), `"housing"` (dado / rabbet), `"tenon"` (mortise and tenon), `"box"` (box joint), `"dovetail"`. The sizes suit the parts; a map sets some: `joint(side, top, "dovetail", #{ count: 5, pin: 8, angle: 9.5 })`. Returns the joint's index |
| `joints()` | the joints: `#{ kind, a, b, count, diameter, depth, depth_b, thickness, shoulder, clearance, angle, pin, through, flip, margin, pitch, at, offset, size, inset, rows, slide, push, positions, hardware, warnings }` `positions` are where the fasteners are, from the joint's start; `a` is the part joined into, `b` the one whose end meets it; `hardware` e.g. `"3 × Dowel Ø8 × 40"`; `warnings` what about it is likely a mistake, as Window → Joinery marks it ⚠ (fasteners too near an end or each other, holes too thick for the board or coming out of its other face, too few hinges, holes of two joints meeting) |
| `set_joint(i, #{ depth: 9 })` | changes joint `i`'s sizes; `kind` in the map turns it into another kind with sizes that suit it |
| `fitting(door, "handle", #{ size: 128 })` | puts a fitting on a part: `"handle"`, `"knob"` (on its front face, min y; `flip`: the back), `"leg"` or `"caster"` (four under it). A fitting is a component of its own beside the part, mounted on one of its faces: it follows the part when it moves or is resized, drills its holes into it and is a hardware line in the cut list, not a part. Moved by hand along the face it stays mounted where it is put. Returns the copies put on it |
| `remove_joint(i)` | deletes joint `i`; its parts get their shape back |
| `drawer("Left", "Right", #{ z: 150, height: 160, front: "overlay" })` | a drawer for the opening between two parts facing each other: its box on the longest standard slides that fit 10 mm short of the opening's end or a part behind (a back between the sides), dowels, a nailed bottom, a `front` (`"inset"`, `"overlay"`, `"none"`) screwed on with a handle. Keys `z` (the box's underside; default the opening's bottom + 30), `height`, `wall`, `bottom`, `front_thickness`, `slide` (`"ball"`: ball-bearing on the box's sides, 12.7 mm each side; `"roller"`: along their bottom, 12.5 mm; `"undermount"`: concealed under the bottom, 5 mm, the slide joints on the bottom's edges and 14 mm under the box), `push` (push-to-open slides, no handle). Returns its parts |
| `drawers("Left", "Right", #{ count: 4, front: "overlay" })` | a chest: `count` drawers one above another filling the free height of the opening, the stretch of the parts' shared height that no other part at its front crosses (between a bottom and a top, under a drawer already there); `from` and `to` set the height instead. The fronts are of equal height, 2 mm apart (inset: also 2 mm from the parts above and below; overlay: over them by 16 mm), each box 10 mm inside its front and the free height. The other keys as for `drawer` but `z` and `height`. Returns each drawer's parts, the lowest first |
| `doors("Side L", "Side R", #{ count: 2 })` | one door or a pair over the opening between two parts facing each other, from the part across its front at the bottom to the one at the top (shelves behind the doors); `from` and `to` set the height instead. `front`: `"overlay"` (default; over the parts round the opening by 16 mm) or `"inset"` (in it, 2 mm clear); a pair meets 2 mm apart in the middle. Each door hangs on cup hinges on its side, as many as its height wants (a single door on the first part), with a handle upright by its free edge; `push: true` leaves the handles off (push latches); `thickness` (18). Returns the doors |
| `back("Side L", "Side R")` | a back for the opening between two parts facing each other: in grooves (housings) in them and in the parts across the back at the bottom and the top (a bottom, a top), `inset` 10 mm in from their back edges and `depth` 8 mm deep; `groove: false` nails it over the back edges of all of them instead; `thickness` (4). Returns the back |
| `plinth(children(cabinet), #{ height: 100, setback: 40 })` | a plinth frame under parts (a carcass): as wide and deep as they are, `height` under their underside, front and back rails between two side rails on confirmats, the front `setback` back from the carcass's front; a bottom lying on it (a part thinner than 40 mm) is screwed down into the front and back rails, 60 mm clear of their ends; `thickness` (18). Returns the rails: front, back, sides |

Keys (mm): `count` (dowels, screws, pocket holes; fingers of a box joint on `b`; dovetail tails),
`diameter`, `depth` (into `a`), `depth_b` (into `b`), `thickness` and `shoulder` (tenon),
`clearance` (gap around tenons, fingers, tails, in housings), `angle` (dovetail slope, degrees),
`pin` (dovetail pins at their narrow end), `through` (tenon), `flip` (pocket holes from `b`'s other
face). Box joints and dovetails join a corner: `a`'s end flush with `b`'s outer face.

Furniture hardware kinds: `"confirmat"` (`size`: the screw's length), `"cam_lock"` (minifix:
`diameter` of the cam, `size` from `a`'s face to its centre, `flip` puts it in `b`'s other face),
`"shelf_pins"` (under `b`, `flip`: over it; `rows` of holes 32 mm apart above and below),
`"hinge"` (cup hinges, `diameter` 35 and cup `depth`; overlay: the door is `a`, the side's edge
on its back, half overlay when it covers about half that edge or less; `inset: true`: the door is `b`,
between the sides; the count follows the door's height; `angle`: the angle it opens to, 110 unless
set, in the hinge's name when not), `"gas_strut"` (holds a lift-up flap open: the flap `a`,
hinged along its top and over the side's front edge, the side `b`; `size`: its force in newtons,
about three times the flap's weight unless set; one each side), `"slide"` (`slide`: `"ball"`, ball-bearing, the parts 12.7 mm apart; `"roller"`, 12.5 mm;
`"undermount"`, 5 mm, `b` the drawer's bottom; `push`: push to open, in the name; `size` 0: the
longest standard length that fits; `flip`: the front at the other end), `"table_top"` (figure 8 fasteners in the edge of
an apron `b`, `flip`: the other side), `"nails"` (through a back panel `a`; `size`: the length),
and the fittings (`fitting(part, kind, #{…})`): `"handle"` (`size`: centres, `offset` along the
part), `"knob"`, `"leg"` (`size`: height, `diameter`, `margin` from the edges), `"caster"` (swivel
casters: `diameter` of the wheel, `size` from the floor to the plate, `margin`).

Dowels, screws, pocket holes and the fittings along a joint are laid out along it, from its start (the end with the
lower coordinate): evenly between `margin` from both ends (0: a quarter of the joint, at most
32 mm), from `margin` every `pitch` (`set_joint(0, #{ margin: 37, pitch: 32 })`, the 32 mm system),
or at the distances in `at` (`#{ at: [40, 150, 260] }`, which sets `count`; an empty array goes
back to the count). `offset` moves them (and a tenon) across the joint from its middle. Lengths
take units as strings (`"1 1/2 in"`).

The parts are the innermost groups and components that have faces, sized by the bounding box in
their own axes (length ≥ width ≥ thickness). A part takes its material's nearest stock thickness;
of the boards of that thickness the narrowest that holds its width; of several sheet sizes the
cheapest for the job (by price, by area while a size has none), and the last sheet may come from
a smaller size. Sheets are packed with guillotine cuts (what panel
and track saws make), boards with first fit decreasing over the lengths on sale. The same plan is
in Window → Cut List, with diagrams and PDF / SVG / CSV export.

## Assembly
| Function | Result |
|---|---|
| `suggest_steps()` | sets the assembly steps from the parts that touch: the lowest part touching another first, then the lowest part touching what is placed, with its copies; dowels, shelf pins and cam lock bolts get a fastener step of their own before the part that comes onto them. Returns the number of steps |
| `assembly_steps()` | the steps: `#{ parts, joints, note }`, `parts` the groups and components the step adds, `joints` the indices of the joints whose fasteners go in at it (a fastener step has joints and no parts) |
| `set_assembly_steps([bottom, [left, right], #{ parts: "Top", note: "Glue the dowels" }])` | sets the steps; each is a part, a name, an array of parts or a map with `parts`, `joints` and `note` (`#{ joints: [0, 1], note: "Dowels into the bottom" }`: a fastener step). A part may be in one step only; a joint not given to a step has its fasteners go in with the later of its parts |
| `assembly_view(150)` | explodes the model by 150 mm (0: assembled); `assembly_view(150, 2)` also shows step 2 (from 0; -1 for all), the later steps' parts hidden; `assembly_view()` is `#{ explode, step }` |

The exploded view stretches the model: each part's middle moves away from the model's middle
across, and its bottom up from the floor, in proportion, the outermost parts by the distance
(sides sideways, a top up, a back backwards, doors to the front, shelves apart between the
sides, the bottom on the floor). Parts clear of each other as assembled only get further
apart, so none goes into another, and everything moves evenly with the distance. Parts that
touch nothing of the carcass (a drawer) also come out of it as a whole, the first clear way
along an axis, the front first; fasteners are pulled just clear of their holes. The whole is lifted so nothing goes below the floor. Joints and the cut list keep working from the assembled model while it is
exploded. Window → Assembly does the same and writes the instructions booklet (PDF); from
Python, `m.export(path, what="assembly")`.

A group or component with steps of its own is a sub-assembly (a drawer): put together first,
it is one unit of the steps it is placed in, and the booklet has its pages first. In Window →
Assembly, Suggest steps gives every group or component with joints inside steps of its own;
opening one shows and edits its steps. A generator's `set_assembly_steps` gives the component
it builds steps of its own.

## Example
```rhai
material("Oak", "#C8A064");
let top = box(vec3(0, 0, 720), 1200, 800, 30);
top.name = "Table top";
for p in [[50, 50], [1110, 50], [50, 710], [1110, 710]] {
    let leg = box(vec3(p[0], p[1], 0), 40, 40, 720);
    leg.name = "Leg";
    paint(leg, "Oak");
}
paint(top, "Oak");
let hole = cylinder(vec3(600, 400, 700), 30, 60);
subtract(top, hole);
print(bounds(groups()).size);
```

## Python
While Varstotas is open, it can be controlled from Python 3. Only the standard library is needed: the client `varstotas.py` lives in the repository's `python/` folder, and the application copies it to `python/` in the data folder (Preferences → General → Data folder: macOS `~/Library/Application Support/Varstotas`, Linux `~/.local/share/varstotas`, Flatpak `~/.var/app/io.github.dirondin.Varstotas/data/varstotas`, Windows `%APPDATA%\Varstotas\data`) when the bridge starts (on the first Python script, or at once with the setting below).

- **File → Run Script…** runs a `.py` file through `python3` (another interpreter can be set with the `VARSTOTAS_PYTHON` variable). The script's output appears in the Script Console, the "Stop Python" button interrupts it.
- **From a terminal:** `PYTHONPATH="<data folder>/python" python3 my_script.py` (or the repository's `python/` folder). This needs **Preferences → General → Let scripts and AI agents outside Varstotas connect** (off by default). The client takes the address and token from `bridge.json` in the data folder; it looks in the places listed above (`VARSTOTAS_DATA_DIR` overrides them).

```python
import varstotas
from varstotas import vec3, ORIGIN, X, Y, Z

m = varstotas.connect()
with m.operation("Table"):            # everything inside is one Undo step; an exception rolls back
    m.material("Oak", "#C8A064")
    top = m.box(vec3(0, 0, 720), 1200, 800, 30)
    top.name = "Table top"
    m.paint(top, "Oak")
print(m.bounds(m.groups())["size"])   # vec3(1200, 800, 30)
```

- Every Rhai API function is a method of `m` with the same name and arguments: `m.pushpull(f, 300)`, `m.rect(ORIGIN, 1000, 500)`. Entity properties are the same too: `e.area`, `e.normal`, `e.name = "…"`.
- Tuples and lists `(x, y, z)` can be passed instead of vectors. `vec3` supports arithmetic, `.length()`, `.dot()`, `.cross()`.
- A Rhai map arrives as a `dict`, `()` as `None`. A Varstotas error raises `varstotas.VarstotasError`.
- A call outside `operation()` is a separate Undo step, "Python".
- `m.rhai(code, **vars)` runs Rhai code in a single call; keyword arguments become variables. This is faster for big loops.
- Application: `m.info()` (document summary), `m.undo()` / `m.redo()`, `m.view("iso" | "top" | … | "zoom_extents")`, `m.camera(eye=…, target=…, parallel=True)`, `m.screenshot(path=None, width=…, height=…)` (path to a PNG), `m.save(path=None)`, `m.export(path)` (format by extension: stl, obj, dxf, glb, 3mf, csv, svg, pdf, png; `m.export(path, what="cut_list")` the cut list as pdf / svg / csv, `what="labels"` the part labels, `what="assembly"` the assembly instructions, both pdf), `m.open(path)`, `m.new()`.
- Before destructive actions the application shows an "External script" dialog, and a refusal raises an error. These are: saving over another existing file, exporting over a file (including OBJ/DAE materials and textures), replacing a screenshot, and also opening a file or a new document when there are unsaved changes.

Example — `examples/scripts/bookshelf.py`.

## Bridge (protocol)
The application listens only on `127.0.0.1`, on a random port, from the first Python script it runs (they get the port and a random token in `VARSTOTAS_BRIDGE` / `VARSTOTAS_TOKEN`). Only with **Preferences → General → Let scripts and AI agents outside Varstotas connect** on (off by default) does it listen from the start and write them to `bridge.json` in the data folder (permissions 0600) for other programs; switching it off removes the file. The bridge can be disabled with the `VARSTOTAS_NO_BRIDGE=1` variable.

A request is a JSON line `{"id": 1, "token": "…", "method": "…", "params": {…}}`. The token is needed in the first request of a connection. The reply: `{"id": 1, "result": …}` or `{"id": 1, "error": {"message": "…"}}`. The same JSON is accepted as an HTTP `POST /rpc`, with a required `Authorization: Bearer …` header (validated before reading the body). Requests from a browser (with an `Origin` header) are rejected.

| Method | Parameters → result |
|---|---|
| `hello` | → `{app, version, build, api}` (`build`: `0.1.0 (build 42, 4cc73e5)`, or `dev` instead of the number in local builds) |
| `eval` | `code`, `vars` (JSON), `label` → `{value, output, changed}`; entities — `{"$entity": id, "kind"}`, vectors — `{"$vec": [x, y, z]}` (passed the same way in `vars`) |
| `begin` / `commit` / `abort` | an operation (one Undo step); connection-owned staged model; closing or aborting discards it; commit rejects concurrent document/context changes |
| `info`, `docs` | document summary; this reference |
| `undo`, `redo`, `view`, `camera`, `screenshot` (`path`, `width`, `height`, `base64`) | |
| `save`, `export`, `open`, `new` | with the user's confirmation (see above) |

## MCP server for AI agents
`<data folder>/python/varstotas_mcp.py` is an MCP server (stdio) that works with a running Varstotas through the bridge; switch on **Preferences → General → Let scripts and AI agents outside Varstotas connect** first. Connecting in Claude Code:

```
claude mcp add varstotas -- python3 "$HOME/Library/Application Support/Varstotas/python/varstotas_mcp.py"   # macOS
claude mcp add varstotas -- python3 ~/.local/share/varstotas/python/varstotas_mcp.py                    # Linux
```

Tools:
- `api_reference` — this reference;
- `model_info`;
- `run_script` — Rhai code; one Undo step "Agent", an error changes nothing;
- `screenshot` — a PNG of the current view;
- `cut_list` — the cut list as data: sheets and boards to buy, waste, cost, issues;
- `set_view`;
- `undo`, `redo`;
- `save`, `export`, `open`, `new_document` — marked as destructive, the application asks the user.

## Trust and execution limits

`ScriptCtx::default()` exposes geometry only. Document generators (including automatic previews),
component scripts and every bridge/MCP `eval` run without `read_text`, `write_text`, `file_exists`,
`list_files`, `list_dirs`, `fetch_text` or `fetch_file`. Nested generators cannot inherit host IO
from a trusted caller. Manually running a local script in the console or File → Run Script,
and invoking an installed plugin, enables host IO explicitly: run only code you trust there.
Python scripts and plugins run as the current user; they are not isolated by OS permissions.
Undo and script-error rollback cover the model, never external file writes or network effects
of trusted code. File tasks from agents use the dedicated `save`, `export`, `open` methods.

Rhai limits: source 1 MiB, strings 8 MiB, arrays 100,000 elements, maps 10,000 entries,
5 million operations, 128 call levels, console output 1 MiB. Trusted runs (a local script run by
hand, an installed plugin) have no operation limit (the time limit stays) and arrays / blobs up to
32 Mi elements, enough for a 2048² RGBA texture. Formula engines use smaller limits.
HTTP(S) downloads (the app's own client: system certificates, proxy from `HTTPS_PROXY` / `HTTP_PROXY`) allow no other schemes, also in redirects, cap output at
256 MiB, and time out within 60 seconds or the remaining script deadline. Material-source
Python calls have a 60-second limit and 8 MiB stdout; interactive Python scripts have a
10-minute limit and bounded output. Unix children run in a process group that is terminated
on cancellation/timeout. On Windows descendant process termination is not guaranteed;
pipe waits are bounded but this is not a replacement for a Windows Job Object.

Bridge requests are limited to 1 MiB, 16 concurrent connections and a 64-request queue.
Socket reads/writes time out after 10 seconds; transactions expire after 5 minutes.
`begin` works on a private copy, visible to its connection; the viewport/live model changes
only on `commit`. Other connections cannot mutate the document until the transaction ends.
A concurrent edit in the UI makes commit fail rather than overwrite that edit.
