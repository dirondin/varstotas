# Varstotas examples

Download a `.vrst` model and open it with **File → Open**, or pass its path when starting Varstotas.
The examples stay in this repository; the application does not install or download them automatically.
**Help → Examples** and the start screen open this directory in your browser.

## Ready-to-open models

| Model | What it demonstrates | Editable source |
|---|---|---|
| [Country house](demos/country_house.vrst) | A complete house, interiors, textured materials and scenes | — |
| [Material gallery](demos/material_gallery.vrst) | PBR materials, textures, lighting and rendering scenes | — |
| [Bookshelf](demos/bookshelf.vrst) | Named boards, shelves, grouping and materials; stock on the materials, so **Window → Cut List** shows the plywood sheet diagrams, shopping list and cost; rabbets and dowels, so **Window → Assembly** explodes it step by step and writes the instructions booklet | [Rhai](scripts/bookshelf.rhai) · [Python](scripts/bookshelf.py) |
| [Nightstand](demos/nightstand.vrst) | Furniture hardware: a carcass on confirmats standing on legs, a drawer on ball-bearing slides with an inset front and a handle, a shelf on shelf pins, a nailed back. The fittings are drawn in place and priced in **Window → Cut List**; **Window → Assembly** goes through twelve noted steps, the pins, dowels and slides at steps of their own | [Rhai](scripts/nightstand.rhai) |
| [Mobile pedestal](demos/mobile_pedestal.vrst) | A white carcass on confirmats rolling on four swivel casters, a deep drawer on ball-bearing slides with an inset front and a handle, an open shelf above it on shelf pins; twelve noted assembly steps | [Rhai](scripts/mobile_pedestal.rhai) |
| [Wall cabinet](demos/wall_cabinet.vrst) | A lift-up flap on cup hinges along the top, held open by a gas strut each side; a carcass on confirmats, a shelf on pins set back clear of the struts, a nailed back; seven noted assembly steps | [Rhai](scripts/wall_cabinet.rhai) |
| [Cabinet](demos/cabinet.vrst) | A two-door melamine cupboard: the bottom and the top on cam locks, three shelves on pins with rows of holes to move them, doors on cup hinges with handles, legs; eight noted assembly steps | [Rhai](scripts/cabinet.rhai) |
| [Woodworking bench](demos/workbench.vrst) | Solid timber top, four legs and lower rails | [Rhai](scripts/workbench.rhai) |
| [Dining set](demos/dining_set.vrst) | A table and four chairs, reusable functions and nested groups | [Rhai](scripts/dining_set.rhai) |
| [Garden pergola](demos/garden_pergola.vrst) | Posts, footings, beams and repeated rafters | [Rhai](scripts/garden_pergola.rhai) |
| [Staircase](demos/staircase.vrst) | Parameterized rise/going, timber treads and a landing | [Rhai](scripts/staircase.rhai) |
| [Small workshop](demos/small_workshop.vrst) | A room corner, window opening, workbench, shelves and storage | [Rhai](scripts/small_workshop.rhai) |
| [Desk caddy](demos/desk_caddy.vrst) | A 3D print for a 150 mm build volume: rounded corners and chamfers from a profile, pen holes, a tray and a phone slot cut by Solid Tools | [Rhai](scripts/desk_caddy.rhai) |

All construction scripts use millimetres. Run them in an empty document through **File → Run Script**,
or paste them into **Window → Script Console**. Change the parameters near the top to explore variants.
Model files embed their materials; these smaller examples need no external assets.

To regenerate the small models from their checked-in scripts:

```sh
cargo run --offline -p varstotas_script --example build_examples
```

[Plugins](plugins/) add menu commands and importers; [generators](generators/) provide optional parametric components.
