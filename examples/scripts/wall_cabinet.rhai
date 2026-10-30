// Wall cabinet: a white carcass on confirmats with a lift-up flap: the flap hangs on two cup
// hinges along the top and a gas strut each side holds it open. A shelf on pins inside, set
// back to clear the struts, and a nailed back. The fittings are drawn in place and priced in
// Window → Cut List; Window → Assembly explodes it step by step and writes the instructions
// booklet.
// Run: File → Run Script… (or paste into Window → Script Console). One undo step.
let w = 600;      // outer width, mm
let d = 320;      // depth
let h = 400;      // height
let t = 18;       // board thickness
let setback = 150; // the shelf's front edge back from the front, clear of the struts

material("White", "#ECEAE4");
material("HDF", "#B8B2A6");
// What they are bought as, for Window → Cut List (sheet diagrams, shopping list, cost)
set_stock("White", #{ kind: "sheet", length: 2800, width: 2070, thickness: t, price: 70 });
set_stock("HDF", #{ kind: "sheet", length: 2800, width: 2070, thickness: 4, price: 16 });

// the carcass: full-height sides, the bottom and the top between them
let left = box(vec3(0, 0, 0), t, d, h);
left.name = "Side L";
let right = box(vec3(w - t, 0, 0), t, d, h);
right.name = "Side R";
let bottom = box(vec3(t, 0, 0), w - 2 * t, d, t);
bottom.name = "Bottom";
let top = box(vec3(t, 0, h - t), w - 2 * t, d, t);
top.name = "Top";
let shelf = box(vec3(t, setback, h / 2 - t / 2), w - 2 * t, d - setback, t);
shelf.name = "Shelf";
// the flap over the whole front
let flap = box(vec3(0, -t, 0), w, t, h);
flap.name = "Flap";
let parts = [left, right, bottom, top, shelf, flap];
paint(parts, "White");
let back = box(vec3(0, d, 0), w, 4, h);
back.name = "Back";
paint(back, "HDF");
let cabinet = group(parts + [back]);
cabinet.name = "Wall cabinet";

// Joinery (Window → Joinery)
let part = |n| child(cabinet, n);
for side in ["Side L", "Side R"] {
    joint(part.call(side), part.call("Bottom"), "confirmat");
    joint(part.call(side), part.call("Top"), "confirmat");
    joint(part.call(side), part.call("Shelf"), "shelf_pins", #{ rows: 1 });
    joint(part.call("Back"), part.call(side), "nails");
    // a gas strut each side holds the flap open
    joint(part.call("Flap"), part.call(side), "gas_strut");
}
// the flap hangs on hinges along the top's front edge, a long handle by its bottom edge
joint(part.call("Flap"), part.call("Top"), "hinge");
joint(part.call("Flap"), "handle", #{ size: 320, offset: 40 - h / 2 });

// What else to buy: the fittings come with their names, priced here
cut_part("Confirmat Ø7 × 50", #{ price: 0.15 });
cut_part("Shelf pin Ø5", #{ price: 0.1 });
cut_part("Nail 1.6 × 20", #{ price: 0.02 });
cut_part("Cup hinge Ø35, overlay", #{ price: 2.8 });
cut_part("Gas strut 90 N", #{ price: 6.5 });
cut_part("Handle, centres 320", #{ price: 5 });

// Assembly steps (Window → Assembly): the pins go in at a step of their own
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
    #{ parts: part.call("Side L"), note: "Lay the left side on a flat surface, inside up." },
    #{ parts: [part.call("Bottom"), part.call("Top")], note: "Set the bottom and the top on and drive their confirmats in through the side." },
    #{ parts: part.call("Side R"), note: "Put the right side on and drive its confirmats in." },
    #{ parts: part.call("Back"), note: "Check the diagonals are equal, then nail the back on. Stand the cabinet up." },
    #{ joints: of_kind.call("shelf_pins"), note: "Push the shelf pins into the holes at the height you want." },
    #{ parts: part.call("Shelf"), note: "Lay the shelf on the pins, its back edge against the back." },
    #{ parts: part.call("Flap"), note: "Screw the hinges' plates under the top and clip the flap on; screw the gas struts to the sides and the flap. Fit the handle." },
]);
select(cabinet);
print("Wall cabinet " + w + " x " + d + " x " + h + ", " + joints().len() + " joints, " + assembly_steps().len() + " steps");
