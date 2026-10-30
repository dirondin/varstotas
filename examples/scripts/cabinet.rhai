// Cabinet: a two-door cupboard of white melamine. The bottom and the top sit between the sides
// on cam locks (minifix), the shelves on shelf pins, the back is nailed on, the doors hang on
// cup hinges over the sides with a handle each, and it stands on four legs. The fittings are
// drawn in place and priced in Window → Cut List; Window → Assembly explodes it step by step
// and writes the instructions booklet.
// Run: File → Run Script… (or paste into Window → Script Console). One undo step.
let w = 800;      // outer width, mm
let d = 450;      // depth
let h = 1400;     // carcass height
let legs = 100;   // the carcass's underside above the floor
let t = 18;       // board thickness
let shelves = 3;  // shelves on pins

material("White", "#ECEAE4");
material("HDF", "#B8B2A6");
// What they are bought as, for Window → Cut List (sheet diagrams, shopping list, cost)
set_stock("White", #{ kind: "sheet", length: 2800, width: 2070, thickness: t, price: 70 });
set_stock("HDF", #{ kind: "sheet", length: 2800, width: 2070, thickness: 4, price: 16 });

// the carcass: full-height sides, the bottom and the top between them
let parts = [];
let left = box(vec3(0, 0, legs), t, d, h);
left.name = "Side L";
let right = box(vec3(w - t, 0, legs), t, d, h);
right.name = "Side R";
let bottom = box(vec3(t, 0, legs), w - 2 * t, d, t);
bottom.name = "Bottom";
let top = box(vec3(t, 0, legs + h - t), w - 2 * t, d, t);
top.name = "Top";
parts += [left, right, bottom, top];
let gap = (h - t) / (shelves + 1);
for i in 1..=shelves {
    let s = box(vec3(t, 0, legs + i * gap), w - 2 * t, d - 20, t);
    s.name = "Shelf " + i;
    parts.push(s);
}
// two doors over the carcass's front edges, 3 mm apart in the middle
let door_w = w / 2 - 1.5;
let door_l = box(vec3(0, -t, legs), door_w, t, h);
door_l.name = "Door L";
let door_r = box(vec3(w - door_w, -t, legs), door_w, t, h);
door_r.name = "Door R";
parts += [door_l, door_r];
paint(parts, "White");
let back = box(vec3(0, d, legs), w, 4, h);
back.name = "Back";
paint(back, "HDF");
let cabinet = group(parts + [back]);
cabinet.name = "Cabinet";

// Joinery (Window → Joinery)
let part = |n| child(cabinet, n);
let names = ["Bottom", "Top"];
for i in 1..=shelves {
    names.push("Shelf " + i);
}
for side in ["Side L", "Side R"] {
    joint(part.call(side), part.call("Bottom"), "cam_lock");
    joint(part.call(side), part.call("Top"), "cam_lock");
    for i in 1..=shelves {
        joint(part.call(side), part.call("Shelf " + i), "shelf_pins", #{ rows: 1 });
    }
    joint(part.call("Back"), part.call(side), "nails");
}
joint(part.call("Door L"), part.call("Side L"), "hinge");
joint(part.call("Door R"), part.call("Side R"), "hinge");
// the handles upright by the doors' meeting edges
joint(part.call("Door L"), "handle", #{ size: 128, offset: door_w / 2 - 50 });
joint(part.call("Door R"), "handle", #{ size: 128, offset: 50 - door_w / 2 });
joint(part.call("Bottom"), "leg", #{ size: legs });

// Edge banding (Window → Cut List): 1 mm on every edge of the boards that shows, priced per
// metre
cut_settings(#{ band: 1, band_price: 0.4 });
band_visible();

// What else to buy: the fittings come with their names, priced here
cut_part("Cam lock Ø15 with bolt", #{ price: 0.35 });
cut_part("Shelf pin Ø5", #{ price: 0.1 });
cut_part("Nail 1.6 × 20", #{ price: 0.02 });
cut_part("Cup hinge Ø35, overlay", #{ price: 2.8 });
cut_part("Handle, centres 128", #{ price: 3.5 });
cut_part("Leg, height " + legs, #{ price: 4 });

// Assembly steps (Window → Assembly): the cam bolts and the pins go in at steps of their own
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
let shelf_parts = [];
for i in 1..=shelves {
    shelf_parts.push(part.call("Shelf " + i));
}
set_assembly_steps([
    #{ parts: part.call("Side L"), note: "Lay the left side on a flat surface, inside up." },
    #{ joints: of_kind.call("cam_lock"), note: "Screw the cam bolts into the sides; press the cams into the bottom and the top, arrows to the edge." },
    #{ parts: [part.call("Bottom"), part.call("Top")], note: "Set the bottom and the top onto the left side's bolts and turn their cams to lock." },
    #{ parts: part.call("Side R"), note: "Put the right side on over the bolts and lock the cams." },
    #{ parts: part.call("Back"), note: "Check the diagonals are equal, then nail the back on. Stand the cabinet up." },
    #{ joints: of_kind.call("shelf_pins"), note: "Push the shelf pins into the holes at the heights you want." },
    #{ parts: shelf_parts, note: "Lay the shelves on the pins." },
    #{ parts: [part.call("Door L"), part.call("Door R")], note: "Screw the hinges' plates to the sides and clip the doors on; adjust them to an even gap. Fit the handles." },
]);
select(cabinet);
print("Cabinet " + w + " x " + d + " x " + (h + legs) + ", " + joints().len() + " joints, " + assembly_steps().len() + " steps");
