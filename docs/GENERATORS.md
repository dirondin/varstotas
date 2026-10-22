# Generators — parametric components (track K)

A generator is a component with parameters: a Rhai script with a parameter header. Its `build(p)` function builds the component's content in local axes: the component's origin is the insertion point. Each instance has its own parameter values, edited in Entity Info. Instances with equal values share one component definition.

## Header
`// @…` comments at the start of the file, before the first line of code:

```rhai
// @generator Shelf
// @description An open shelf unit.
// @param width:length(200..3000/10) = 800 "Width"
// @param shelves:int(0..12) = 3 "Shelves"
// @param back:bool = true "Back panel"
// @param finish:choice(Oak|Walnut|White) = Oak "Finish"
// @param lid:angle(0..120/5) = 30 "Lid opening"
// @param fill:percent = 50 "Filled"
// @param paint_with:material = "" "Material"

fn build(p) {
    box(vec3(0, 0, 0), p.width, 300, 18);
    // ...
}
```

- `@generator Name` — required. The name serves as the generator's key in the document.
- `@description` — a hint in the tray.
- `@param key:type(range) = value "Label"`:
  - the range is written as `min..max/step`, each part optional: `(..2000)`, `(/10)`;
  - values are clamped to the range and rounded to the step (from `min`, otherwise from 0);
  - the label is shown in Entity Info; without it the key serves as the label.

| Type | In `p` | Field in Entity Info |
|---|---|---|
| `length` (default for numbers) | a number, mm; units can be written in the value: `1.2m` | text with the model's units |
| `int` | an integer | a counter (drag or type) |
| `number` | a number | a counter |
| `bool` | `true` / `false` | a checkbox |
| `choice(a\|b\|c)` | the chosen string | a drop-down list |
| `angle` | degrees | a slider (without a range — 0–360°) |
| `percent` | 0–100 (or its own range) | a slider |
| `material` | a material name, `""` — none | a list of the document's materials |

## Formulas and checks
```rhai
// @formula count:int = ceil(height / max_riser) "Risers"
// @formula riser = height / count "Riser height"
// @formula first:int = if layout == "landing" { count / 2 } else { count }
// @check 2 * riser + going >= 550 && 2 * riser + going <= 700 "Uncomfortable step"
```
- `@formula key[:type] = expression "Label"` — a value computed from the parameters. The expression is Rhai; parameters and previous formulas are available by name. The result goes into `p` on a par with the parameters (`p.riser`) and is shown in Entity Info in grey, read-only (the expression is in the hint). The type only sets how the value looks: `length` (default), `int` (rounded to an integer), `number`, `bool`, `angle`, `percent`, `text`. Formula and parameter names must not coincide.
- `@check expression "Message"` — a condition on the values. If it is false, the values are not accepted: nothing changes in the document, and the message is shown in the status bar.
- A quoted string at the end of a header line is the label or message. If the expression itself ends with a string (`finish == "Oak"`), this is recognised by the operator before it.
- Besides the standard Rhai functions (`floor`, `round`, `abs`, `sqrt`, …) there are `min(a, b)`, `max(a, b)`, `clamp(x, lo, hi)` and `ceil(x)`; they are available in scripts too.

Inside `build` the whole API from `docs/SCRIPTING.md` is available. The script runs in an empty model that contains the document's materials and tags, so `material("Oak", "#C8A26B")` creates the material only once. Everything that is built becomes the component's content; nested groups are kept. The running time is limited to 10 s. An error in the script changes nothing in the document — the message is shown in the status bar.

## Working in the application
- **Catalogue.** Generators are components with parameters, so they live in the same catalogue as components: `~/.varstotas/components` (or the folder chosen in the tray), `.rhai` files next to `.vrst`. Subfolders are categories: each has a header with an arrow (collapse / expand, remembered) and a link that opens the folder in Finder. The **Components** tray is one common list of tiles with thumbnails, by folder and by name; a generator tile has a gear in its top right corner. The thumbnail is built with the default values. Optional examples (Shelf, Drawer Box, Stair, Cabinet, Railing, Table) live only in the repository's `examples/generators/` (Help → Examples); copy selected `.rhai` files into your components library. Generators that exist only in the document are shown in the In Model section (also with "Hide" and the "…" → "Show In Model" checkbox); "Delete…" removes such a generator from the document — placed components remain ordinary components without parameters, Undo brings it back. Files from the former `~/.varstotas/generators` folder are moved to the components catalogue at startup.
- **Placement.** Click a generator — a component with the default values follows the cursor, the next click places it. The generator's source is saved in the document (`Model::generators`), so the file opens on another machine too. If the library has a newer version, placing replaces the copy in the document with it, and all its components are rebuilt (the Undo step "Update Generator …").
- **Editor (Window → Generator Editor…, "+" in the tray, a tile's context menu → Edit Generator…).** The "? Reference" button on the right opens the reference on the Generators topic, F1 — on the word under the cursor. On the left is the source in the same editor as the Script Console (highlighting, completion, hints for the `@formula` and `@check` tags). On the right are a preview that is rebuilt 0.4 s after an edit (dragging rotates it), parameter fields for the preview, formula values and errors. Buttons:
  - **New** — a template;
  - **Open…** — a `.rhai` file;
  - **Save to Library** — write to the components catalogue: to its own file if the generator was opened from there, otherwise to `<name>.rhai` (someone else's file with the same name is not overwritten);
  - **Apply to Document** — replace the source in the document, its components are rebuilt (one Undo step);
  - **Place** — place a component with the preview's values.
- **Entity Info.** When generated components are selected, their parameter fields are shown.
  - Every change rebuilds the component and gives one Undo step. A slider and a counter are applied when the mouse is released.
  - If several instances of one generator are selected, the change is applied to all of them, and their other values are not changed.
  - **Reset** returns the default values.
  - **Make Unique** (Entity Info or right click) copies the instance's generator: "<name> 2" appears in the document (the same script, the `@generator` header renamed, a tile in In Model), and the instance moves to it. Editing the copy's script changes only its instances. The button is available when other instances use the generator too (even with other values).
- **Document generators (In Model).**
  - A generator placed from the library is stored in the document and is always shown as a tile in In Model, separately from the library one. Edit on this tile opens the Generator Editor for the document's copy: **Apply** rebuilds all its instances; if the name in the header is changed, the generator and its components are renamed.
  - Inserting again from the library takes the document's generator with the same script. If the script in the document was edited, the library one arrives next to it as "<name> 2".
  - All of a generator's definitions are named after it (in the Outliner, Shelf copies are "Shelf" with any values).
- **Shared definitions.**
  - If another instance already has a definition with the same values, the instance moves to it.
  - If the definition is used only by this instance, it is rebuilt in place.
  - Otherwise a new one is built.
  - Definitions that nobody uses any more are deleted.
- **Scale.** Scaling a single generated component changes its parameters `width` (or `length`) along local X, `depth` along Y and `height` along Z, if they exist and have the `length` type. The geometry is not stretched: boards keep their thickness. The component's origin shifts the same way as in an ordinary scale. Axes for which there is no parameter do not change. If the generator has none of these parameters, the component is scaled in the ordinary way. The preview during dragging still shows a simple stretch.
- **Copying.** Copy/Paste carries the generator to another document. Instances with equal values and the same source share one definition.

## Scripts, Python, MCP
| Function | What it does |
|---|---|
| `add_generator(src)` | add or update a generator from source; return the name (with a new source all its components are rebuilt) |
| `generators()` | the document's generator names |
| `generate(name, at)`, `generate(name, at, #{ width: 1200, finish: "White" })` | place a component at point `at`; return it |
| `params(g)` | the values of a generated component (map) |
| `set_params(g, #{ shelves: 5 })` | change some of the values (the others are kept) |
| `reset_params(g)` | the default values |

Python: `m.add_generator(open("shelf.rhai").read())`, `g = m.generate("Shelf", varstotas.vec3(0, 0, 0), {"width": 1200})`, `m.set_params(g, {"back": False})`. The MCP agent uses the same functions through `run_script`.

## Internals
- `varstotas_kernel::generator`:
  - header parsing (`Generator::parse`);
  - the types `GenKind`, `GenParam`, `GenValue`, value checking (`normalize`);
  - `Generated` on a definition (generator + values);
  - `Model::generated_def` looks up a shared definition;
  - `graft_content` carries the built content into the definition;
  - `SIZE_KEYS` — the size keys for Scale.
- `varstotas_script::generator`: `build` (runs `build(p)` in an empty model), `def_for`, `place`, `set_values`, `scale`, `add`, `rebuild_all`.
- `varstotas_script::generator::derive` computes formulas and checks (`formula_values` — for display).
- Application: `ui/generators.rs` (generator tiles in the common list `ui/library.rs`, built-in examples, moving the old folder, placement through `PasteTool`, Entity Info fields `param_field`, formula rows), `ui/gen_editor.rs` (the editor window). Scale — `tools/transform.rs::scale_generated`.
- Harness: `action generator <name>`, `action gen key=value …` (`_` in a value means a space), `action gen_reset`, `action make_unique`; the editor — `action gen_editor [name]`, `action ged key=value …`, `action ged_sub old|new`, `action ged_save`, `action ged_apply`, `action ged_place`, `action ged_close`. In the dump: `gen=` (values and size of the selected), `gens=`, `gen_defs=`, `ged=Name{p=…,f=…}:preview(edges,WxDxH)|err(…):status:hcode_height`.

## Parameters of ordinary components
A generator builds a component with a script. A component that already exists (drawn or imported) can get parameters too: for example, an "opening degree" turns a cabinet door.

- **Parts** — groups and components directly inside the component, by name. Unnamed ones get names when parameters are added (the definition name, repeats with a number). A solid model must be split first: open the component and make the door a group.
- **Window → Component Parameters** (or Entity Info → Edit Parameters…, or Components → Document → right click → Parameters…) for the selected component:
  - **Parameters**: name, label, type (length, angle, number, int, percent, bool, choice, material), range, step, default value — as with `@param`.
  - **Formulas**: `name = expression` — as with `@formula`.
  - **Motions**: a part, a motion, a parameter or formula (or an expression), From / To:
    - Rotate — rotation about an axis (a point and a direction; the Pick… button — two clicks in the model);
    - Slide — a shift along a direction;
    - Show / hide — shown by a flag or a number ≠ 0;
    - Paint — a material by name ("" — as drawn);
    - Set parameter — the part's own parameter, if it has one. This is a generated component (for example, a cylinder from Draw → Shapes) or a component with its own parameters. The value is passed as is: table height → the height of a cylindrical leg. Copies with equal values share one part definition. Edits inside a copy return its own part to the master.
    The parameter's range (min..max) is converted to From..To (degrees or length). Without a range, and with an expression, the value is the motion itself.
  - **Script** (optional): `fn update(p) { … }` — runs after the motions, inside the component:
    - the whole API of ordinary scripts and generators is available (`rotate`, `move`, `scale`, `paint`, `show` / `hide`, `set_params`, `box`, `generate`…), with no special functions;
    - a part is found by name: `child("Door")`, deeper — `child("Knob", true)`, inside a part — `child(door, "Knob")`;
    - extents are fields of the part itself: `door.min`, `door.max`, `door.size`, `door.bounds`;
    - `p` — the values and formulas;
    - the script changes the copy: changed parts are copied into the variant, the master is not touched. Edits inside a copy are returned to the master minus what the script did (what the script added is discarded; geometry of the master itself changed by the script stays as in the master);
    - the old form `fn update(p, parts)` (`parts[name]` — the extents at rest) still works.
  - The script is stored in the component.
  - Apply applies; Revert cancels the edits in the window; Remove Parameters returns the copies to the "as drawn" look.
- **Copies.** Each copy has its own values: in Entity Info there are sliders and fields, Reset, Make Unique. Copies with equal values use one *variant* — a definition with the master's content in the required pose. Make Unique gives a copy its own master "<name> 2" with the same parameters: edits inside it go to its master, and the other copies do not change. The master keeps the "as drawn" pose and is not placed itself: a copy placed from the Document tile or pasted gets a variant with the default values.
- **Edits inside a copy** (double click) go to the master after every step. Poses are removed in the process: a door moved in an open copy reaches the master closed and moved. The other copies are rebuilt in the same undo step. Edits inside a painted part are lost; change its shape with an empty material.
- **Scenes** remember the values of all copies. Going to a scene changes the numbers smoothly, together with the camera; flags and texts switch in the middle of the transition. The same happens in the File → Export → Animation video.
- **Library.** Save to Library keeps the component whole: the file holds the master, the parameters and the variant. From the library it is placed into any document with its parameters and saved values. Inserting again uses the same master — by name and identical parameters, whatever the file is called. Copy / Paste between documents works the same way.
- **Internals:** `varstotas_kernel::rig` (data, poses, variants, write-back), `varstotas_script::rig` (motions, expressions, the `update` script), `ui/rig_editor.rs`; a hook in `Document::commit`.
- **Harness:**
  - window: `action rig_edit`;
  - rows: `action rig_param <@param text>`, `action rig_formula <@formula text>`, `action rig_joint part|motion(0 rotate,1 slide,2 show,3 paint,4 part parameter)|parameter|from|to|ox oy oz|ax ay az|expression|key`, `action rig_script <file>`, `action rig_clear`;
  - applying: `action rig_apply`, `action rig_remove`, `action rig_pick <number>`;
  - values of the selected copies: `action rig key=value …`, `action rig_reset`;
  - dump: `rig=Some("copies Name [key=value…] joints=N variants=N") ed=(open name p f j error first_axis)`.

## Batches
- Batch 1 ✅ — the mechanism.
- Batch 2 ✅ — stair, cabinet, railing, table; formulas and checks; an editor with a preview; generators and components — one catalogue and one list.
- Primitives (track U) ✅ — `crates/varstotas_app/assets/primitives/` (box, cylinder, cone, pyramid, sphere, torus): application resources used by Draw → Shapes; they are not shown in the Components tab.
