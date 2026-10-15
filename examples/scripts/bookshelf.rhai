// Bookshelf: two sides, top, bottom and shelves as named groups; a parametric back panel.
// Run: File → Run Script… (or paste into Window → Script Console). One undo step.
let w = 800;      // outer width, mm
let h = 1800;     // height
let d = 300;      // depth
let t = 18;       // board thickness
let shelves = 4;  // inner shelves

material("Birch", "#E3C99A");
material("Back", "#B8B2A6");

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
select(shelf);
print("Bookshelf " + w + " x " + d + " x " + h + ", " + (shelves + 5) + " parts");
