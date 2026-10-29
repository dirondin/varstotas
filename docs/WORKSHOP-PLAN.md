# Workshop: from the model to the cuts (track W)

Owner's decision of 2026-10-28: the next track should make Varstotas stand out and draw
attention. Free modellers stop at the model; woodworkers and DIY makers then work out the cut
list, the sheet layout and the shopping list by hand or with paid or third-party plugins. Track W
takes a furniture model all the way to the workshop: what to buy, how to cut it, how the parts
join and in what order to assemble them. The roadmap row is in `03-ROADMAP.md`.

What already exists and is reused:

- `varstotas_kernel/src/report.rs`: the parts list. Every group or component instance is sized
  by the bounding box in its own axes (length ≥ width ≥ thickness), and equal parts share a row.
  Only CSV today (File → Export → Report).
- Track D: sheets with viewports, title block, parts table, PDF / SVG export.
- 3D printing: cutting a model with dowels (`printing.rs`). Slice tool: a plane cut into two
  closed groups.
- Generators (track K): parametric components rebuilt on a parameter change. This is the model
  for editable joints.

## Batch 1: stock and the cut-list optimizer

Status (2026-10-29): done. Stock on materials (also through My Library): sheets, boards and
linear stock (a named profile, its section only a limit), user stock templates in Preferences →
Cut List, boards priced per metre or per piece, several stock items per material (thicknesses,
board sections, sheet sizes chosen by cost), offcuts kept across projects (Cut List → Keep
offcuts, listed in Preferences → Cut List, cut from first), Window → Cut List, PDF / SVG / CSV,
"+ Cut Diagram" on sheets, `cut_list()` in Rhai, Python and MCP; the Bookshelf example carries
stock; README media.

**Stock.** A material can describe what it is bought as:

- kind: sheet (plywood, MDF, chipboard), board (sawn timber of a fixed section) or linear
  (moulding, rod, profile);
- thickness, plus the sheet size, or the board section and lengths on sale;
- grain: along the length, along the width, or none;
- price per sheet, per piece or per metre.

Stock lives in the document (`FILE-FORMAT.md`), and in My Library together with the material so
it can be reused. A few presets ship as examples: plywood 18 / 12 / 6 mm at 2440 × 1220 and
1525 × 1525, MDF 16 mm, timber 45 × 95.

**Parts.** The parts come from `report.rs`. A part's thickness picks the stock of its material
(the nearest thickness, warning on a mismatch). Grain follows the part's length axis unless the
part overrides it. A part that is not a rectangle is cut from its bounding rectangle, and its
outline appears on the diagram. Per part: exclude from the cut list, count multiplier.

**Optimizer** (kernel, pure functions, tested):

- sheets: guillotine packing, since straight through-cuts are what panel, table and track saws
  make. The settings are:
  - kerf (default 3 mm) and edge trim;
  - rotation only where the grain allows it;
  - several orderings and split rules tried within a time limit, keeping the result with the
    fewest sheets, then the largest offcut;
- boards and linear stock: 1D cutting stock (first fit decreasing, then improvement) over the
  lengths on sale, with the kerf;
- offcuts: leftover rectangles above a minimum size are listed. "Keep offcuts" stores them in
  the user's offcuts (with the project they came from) and takes out the ones the plan used;
  other projects cut from them first, the smallest first, matched by material name and stock.

**Window → Cut List:**

- the parts table, editable for grain, exclude and count;
- the stock in use;
- the settings: kerf, trim and minimum offcut;
- the result: one diagram per sheet or board, with numbered parts, dimensions and waste shaded;
- totals: sheets and boards to buy, waste %, cost.

Clicking a part on a diagram selects it in the model, and the other way round.

**Output:**

- PDF / SVG cutting diagrams, one sheet per page, in the shop-drawing style of track D;
- CSV of parts and of the shopping list;
- "+ Cut Diagram" on a sheet (track D);
- `cut_list()` in Rhai, Python and MCP. An agent can then answer "how many sheets does this
  wardrobe take".

## Batch 2: edge banding, labels and cost

Status (2026-10-29): edge banding done (per-part edges clicked in Cut List → Parts, drawn on the
diagrams, metres and cost per part thickness, optionally subtracted from the cut size; CSV, Rhai,
Python). Labels and hardware to do.

- Edge banding per part edge (which of the four edges, band thickness), shown on diagrams and
  summed into metres of banding to buy. Banding thickness optionally subtracted from the cut
  size.
- Part numbers shown on the model (a view overlay) and on the diagrams. Printable part labels
  (number, name, size, grain arrow, banded edges) as a PDF for label sheets.
- Hardware: components can carry a price and a "hardware" flag (hinges, screws, slides). The
  shopping list adds them, and the cost totals cover stock, banding and hardware.

## Batch 3: joinery

A Joint tool: pick two parts that touch, or an edge between them, then choose a joint:

- butt joints: screws, confirmat or dowels (holes drilled in both parts);
- housings: groove/dado and rabbet;
- mortise and tenon, through or blind;
- box joint and dovetail (through or half-blind) on board ends;
- pocket hole.

Each joint has parameters (sizes, count, spacing, fit clearance) with defaults that follow the
stock thickness. The geometry is cut into both parts. Like a generator, the joint stays editable:
change the parameters or the parts and it is rebuilt, and deleting it restores the parts.
Cut-list sizes include tenons and the like, since the bounding box grows with them. Drilling
appears on the diagrams as hole positions.

## Batch 4: exploded view and assembly instructions

- Exploded view. Parts move apart along the joint directions from the contact graph. The
  distance is set by a slider, and the view can be captured in a scene.
- Assembly steps. The order is suggested from the joints (the base first, then parts that join
  what is already placed) and edited by hand. Each step is a scene showing the new parts
  highlighted and the earlier ones greyed.
- An instructions booklet as a PDF through track D sheets: a cover, a parts and hardware list,
  then one step per page with its picture, the parts used and the hardware counts.

## Showcase (after batch 1, again after batch 4)

- A demo project: a bookshelf or a wardrobe from 18 mm plywood, with cut diagrams, the shopping
  list and, later, the instructions booklet. It joins the Examples.
- README media: a short GIF "model → Cut List → diagrams", made by `scripts/readme-media.py`,
  and a still of a cutting diagram.
- A post with the demo for woodworking / DIY communities and Show HN.

## Open questions

- True nesting of non-rectangular parts (CNC) vs bounding rectangles only. Rectangles in batch 1.
- Non-guillotine packing as an option for CNC routers.
- Stock units in inches and imperial board sizes (4 × 8 ft) for US users. At least the presets.
- Whether a joint is a document object of its own (like a generator) or an attribute of the
  part pair. To be decided in batch 3.
