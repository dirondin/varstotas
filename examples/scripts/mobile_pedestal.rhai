// Mobile pedestal: a white carcass on confirmats rolling on four swivel casters, a deep drawer
// at the bottom on ball-bearing slides with an inset front and a handle, an open shelf above it
// on shelf pins and a nailed back. The fittings are drawn in place and priced in
// Window → Cut List; Window → Assembly explodes it step by step and writes the instructions
// booklet.
// Run: File → Run Script… (or paste into Window → Script Console). One undo step.
let w = 600;        // outer width, mm
let d = 450;        // depth
let h = 736;        // carcass height
let casters = 70;   // the floor to the casters' plates (the carcass's underside)
let t = 18;         // board thickness
let drawer_h = 220; // the drawer box's height

material("White", "#ECEAE4");
material("Birch", "#E8D2A8");
material("HDF", "#B8B2A6");
// What they are bought as, for Window → Cut List (sheet diagrams, shopping list, cost)
set_stock("White", #{ kind: "sheet", length: 2800, width: 2070, thickness: t, price: 70 });
set_stock("Birch", #{ kind: "sheet", length: 1525, width: 1525, thickness: 15, grain: true, price: 48 });
set_stock("HDF", #{ kind: "sheet", length: 2800, width: 2070, thickness: 4, price: 16 });

// the carcass: the bottom and the top over the full width, the sides between them
let bottom = box(vec3(0, 0, casters), w, d, t);
bottom.name = "Bottom";
let left = box(vec3(0, 0, casters + t), t, d, h - 2 * t);
left.name = "Side L";
let right = box(vec3(w - t, 0, casters + t), t, d, h - 2 * t);
right.name = "Side R";
let top = box(vec3(0, 0, casters + h - t), w, d, t);
top.name = "Top";
let shelf = box(vec3(t, 0, casters + 400), w - 2 * t, d - 20, t);
shelf.name = "Shelf";
paint([bottom, left, right, top, shelf], "White");
let back = box(vec3(0, d, casters), w, 4, h);
back.name = "Back";
paint(back, "HDF");
let pedestal = group([bottom, left, right, top, shelf, back]);
pedestal.name = "Pedestal";

// Joinery (Window → Joinery): confirmats through the bottom and the top into the sides, the
// shelf on pins (two more rows of holes to move it), the back nailed on, casters under it
let part = |n| child(pedestal, n);
for side in ["Side L", "Side R"] {
    joint(part.call("Bottom"), part.call(side), "confirmat");
    joint(part.call("Top"), part.call(side), "confirmat");
    joint(part.call(side), part.call("Shelf"), "shelf_pins", #{ rows: 2 });
    joint(part.call("Back"), part.call(side), "nails");
}
joint(part.call("Bottom"), "caster", #{ size: casters });
// a deep drawer at the bottom of the opening: its box of 15 mm birch with a 4 mm HDF bottom,
// an inset white front and a handle
let drawer_parts = drawer(part.call("Side L"), part.call("Side R"), #{ z: casters + t + 30, height: drawer_h, wall: 15, bottom: 4, front: "inset", front_thickness: t });
paint(drawer_parts.extract(0, 4), "Birch");
paint(drawer_parts[4], "HDF");
paint(drawer_parts[5], "White");

// What else to buy: the fittings come with their names, priced here
cut_part("Confirmat Ø7 × 50", #{ price: 0.15 });
cut_part("Shelf pin Ø5", #{ price: 0.1 });
cut_part("Nail 1.6 × 20", #{ price: 0.02 });
cut_part("Dowel Ø8 × 38", #{ price: 0.05 });
cut_part("Screw Ø4 × 25", #{ price: 0.06 });
cut_part("Swivel caster Ø50, height " + casters, #{ price: 3.2 });
cut_part("Ball-bearing slide 400", #{ price: 10 });
cut_part("Handle, centres 192", #{ price: 3.5 });

// Assembly steps (Window → Assembly): the carcass on its casters, then the drawer; the pins,
// dowels and slides go in at steps of their own
let js = joints();
let of_kind = |k| {
    let out = [];
    for i in 0..js.len() {
        if js[i].kind == k {
            out.push(i);
        }
    }
    out
};
set_assembly_steps([
    #{ parts: part.call("Bottom"), note: "Screw the four casters on under the bottom, then stand it on them." },
    #{ parts: [part.call("Side L"), part.call("Side R")], note: "Stand the sides on the bottom and drive the confirmats in from underneath." },
    #{ parts: part.call("Top"), note: "Put the top on and drive its confirmats into the sides." },
    #{ parts: part.call("Back"), note: "Check the diagonals are equal, then nail the back on." },
    #{ joints: of_kind.call("shelf_pins"), note: "Push the shelf pins into the holes at the height you want." },
    #{ parts: part.call("Shelf"), note: "Slide the shelf in onto the pins." },
    #{ parts: drawer_parts.extract(0, 2), note: "The drawer: lay its two sides out, inside up." },
    #{ joints: of_kind.call("dowels"), note: "Glue the dowels into the sides." },
    #{ parts: drawer_parts.extract(2, 2), note: "Set the box's front and back onto the dowels, square the box and clamp it." },
    #{ parts: drawer_parts[4], note: "Nail the bottom on underneath." },
    #{ joints: of_kind.call("slide"), note: "Screw one half of each slide to the carcass, the other to the drawer side, and slide the drawer in." },
    #{ parts: drawer_parts[5], note: "Screw the front on from inside the box, 2 mm clear all round, and fit the handle." },
]);
select(pedestal);
print("Mobile pedestal " + w + " x " + d + " x " + (h + casters) + ", " + joints().len() + " joints, " + assembly_steps().len() + " steps");
