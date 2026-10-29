// Bookshelf: two sides, top, bottom and shelves as named groups; a parametric back panel;
// stock on the materials, so Window → Cut List lays the parts out on plywood sheets; joints
// (top and bottom in rabbets, shelves on dowels) and assembly steps, so Window → Assembly
// explodes it and writes the instructions booklet.
// Run: File → Run Script… (or paste into Window → Script Console). One undo step.
let w = 800;      // outer width, mm
let h = 1800;     // height
let d = 300;      // depth
let t = 18;       // board thickness
let shelves = 4;  // inner shelves

material("Birch", "#E3C99A");
material("Back", "#B8B2A6");
// What they are bought as, for Window → Cut List (sheet diagrams, shopping list, cost)
set_stock("Birch", #{ kind: "sheet", length: 2440, width: 1220, thickness: t, grain: true, price: 85 });
set_stock("Back", #{ kind: "sheet", length: 2440, width: 1220, thickness: 4, price: 18 });

let parts = [];
let left = box(vec3(0, 0, 0), t, d, h);
left.name = "Side L";
let right = box(vec3(w - t, 0, 0), t, d, h);
right.name = "Side R";
let bottom = box(vec3(t, 0, 0), w - 2 * t, d, t);
bottom.name = "Bottom";
let top = box(vec3(t, 0, h - t), w - 2 * t, d, t);
top.name = "Top";
parts += [left, right, bottom, top];

let step = (h - t) / (shelves + 1);
for i in 1..=shelves {
    let s = box(vec3(t, 0, i * step), w - 2 * t, d - 10, t);
    s.name = "Shelf " + i;
    parts.push(s);
}
paint(parts, "Birch");

let back = box(vec3(0, d, 0), w, 4, h);
back.name = "Back";
paint(back, "Back");

let all_parts = parts + [back];
let shelf = group(all_parts);
shelf.name = "Bookshelf";

// Joinery (Window → Joinery): the top and bottom sit in rabbets cut into the sides, the shelves
// on dowels; the dowels go into the cut list's hardware
let side_l = child(shelf, "Side L");
let side_r = child(shelf, "Side R");
let names = ["Bottom", "Top"];
for i in 1..=shelves {
    names.push("Shelf " + i);
}
for n in names {
    let kind = if n == "Bottom" || n == "Top" { "housing" } else { "dowels" };
    joint(side_l, child(shelf, n), kind);
    joint(side_r, child(shelf, n), kind);
}
// What else to buy: the dowels are priced by name, glue and the back's nails added by hand
cut_part("Dowel Ø8 × 40", #{ price: 0.05 });
cut_hardware([#{ name: "Wood glue 250 ml", count: 1, price: 6 }, #{ name: "Nail 1.6 × 25", count: 30, price: 0.02 }]);

// Assembly steps (Window → Assembly): the case is built lying on its left side
let boards = [];
for n in names {
    boards.push(child(shelf, n));
}
set_assembly_steps([
    #{ parts: side_l, note: "Lay the left side on a flat surface, inside up." },
    #{ parts: boards, note: "Glue the rabbets and the dowels, then set the bottom, the top and the shelves into the left side." },
    #{ parts: side_r, note: "Put the right side on, the dowels in their holes, and tap it down evenly. Clamp until the glue sets." },
    #{ parts: child(shelf, "Back"), note: "Check the diagonals are equal, then nail or screw the back on." },
]);
select(shelf);
print("Bookshelf " + w + " x " + d + " x " + h + ", " + (shelves + 5) + " parts, " + joints().len() + " joints");
