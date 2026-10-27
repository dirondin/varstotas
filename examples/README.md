# Varstotas examples

Download a `.vrst` model and open it with **File → Open**, or pass its path when starting Varstotas.
The examples stay in this repository; the application does not install or download them automatically.
**Help → Examples** and the start screen open this directory in your browser.

## Ready-to-open models

| Model | What it demonstrates | Editable source |
|---|---|---|
| [Country house](demos/country_house.vrst) | A complete house, interiors, textured materials and scenes | — |
| [Material gallery](demos/material_gallery.vrst) | PBR materials, textures, lighting and rendering scenes | — |
| [Bookshelf](demos/bookshelf.vrst) | Named boards, shelves, grouping and materials | [Rhai](scripts/bookshelf.rhai) · [Python](scripts/bookshelf.py) |
| [Woodworking bench](demos/workbench.vrst) | Solid timber top, four legs and lower rails | [Rhai](scripts/workbench.rhai) |
| [Dining set](demos/dining_set.vrst) | A table and four chairs, reusable functions and nested groups | [Rhai](scripts/dining_set.rhai) |
| [Garden pergola](demos/garden_pergola.vrst) | Posts, footings, beams and repeated rafters | [Rhai](scripts/garden_pergola.rhai) |
| [Staircase](demos/staircase.vrst) | Parameterized rise/going, timber treads and a landing | [Rhai](scripts/staircase.rhai) |
| [Small workshop](demos/small_workshop.vrst) | A room corner, window opening, workbench, shelves and storage | [Rhai](scripts/small_workshop.rhai) |
| [Desk caddy](demos/desk_caddy.vrst) | A 3D print for a 150 mm build volume: rounded corners and chamfers from a profile, pen holes, a tray and a phone slot cut by Solid Tools | [Rhai](scripts/desk_caddy.rhai) |

All construction scripts use millimetres. Run them in an empty document through **File → Run Script**,
or paste them into **Window → Script Console**. Change the parameters near the top to explore variants.
Model files embed their materials; these six smaller examples need no external assets.

To regenerate the small models from their checked-in scripts:

```sh
cargo run --offline -p varstotas_script --example build_examples
```

[Plugins](plugins/) add menu commands and importers; [generators](generators/) provide optional parametric components.
