// Chest of drawers: an oak carcass on confirmats standing on a plinth set back from the front,
// a back in grooves, four drawers one above another filling it, on concealed undermount slides
// that push to open, so the overlay fronts have no handles. The fittings are drawn in place and
// priced in Window → Cut List; Window → Assembly explodes it step by step and writes the
// instructions booklet.
// Run: File → Run Script… (or paste into Window → Script Console). One undo step.
let w = 800;       // outer width, mm
let d = 500;       // depth
let h = 860;       // carcass height
let plinth = 100;  // the floor to the carcass's underside
let t = 18;        // board thickness
let count = 4;     // drawers

material("Oak", "#C8A27A");
material("Birch", "#E8D2A8");
material("HDF", "#B8B2A6");
// What they are bought as, for Window → Cut List (sheet diagrams, shopping list, cost)
set_stock("Oak", #{ kind: "sheet", length: 2800, width: 2070, thickness: t, grain: true, price: 95 });
set_stock("Birch", #{ kind: "sheet", length: 1525, width: 1525, thickness: 15, grain: true, price: 48 });
set_stock("HDF", #{ kind: "sheet", length: 2800, width: 2070, thickness: 4, price: 16 });

// the carcass: the sides full height, the bottom and the top between them
let left = box(vec3(0, 0, plinth), t, d, h);
left.name = "Side L";
let right = box(vec3(w - t, 0, plinth), t, d, h);
right.name = "Side R";
let bottom = box(vec3(t, 0, plinth), w - 2 * t, d, t);
bottom.name = "Bottom";
let top = box(vec3(t, 0, plinth + h - t), w - 2 * t, d, t);
top.name = "Top";
paint([left, right, bottom, top], "Oak");
let chest = group([left, right, bottom, top]);
chest.name = "Chest";

// Joinery (Window → Joinery): confirmats through the sides into the bottom and the top; the
// back in grooves 10 mm in from the back edges; the plinth frame under it all, 40 mm back
// from the front, the bottom screwed down onto it
let part = |n| child(chest, n);
for side in ["Side L", "Side R"] {
    joint(part.call(side), part.call("Bottom"), "confirmat");
    joint(part.call(side), part.call("Top"), "confirmat");
}
let back = back(part.call("Side L"), part.call("Side R"));
paint(back, "HDF");
let rails = plinth([part.call("Side L"), part.call("Side R"), part.call("Bottom"), part.call("Top")], #{ height: plinth, setback: 40 });
paint(rails, "Oak");
// the drawers fill the height between the bottom and the top: boxes of 15 mm birch with 4 mm
// HDF bottoms, oak fronts over the carcass's edges, 2 mm apart
let drawers = drawers(part.call("Side L"), part.call("Side R"), #{ count: count, wall: 15, bottom: 4, front: "overlay", front_thickness: t, slide: "undermount", push: true });
for dr in drawers {
    paint(dr.extract(0, 4), "Birch");
    paint(dr[4], "HDF");
    paint(dr[5], "Oak");
}

// Edge banding (Window → Cut List): 1 mm on every edge of the boards that shows, priced per
// metre
cut_settings(#{ band: 1, band_price: 0.4 });
band_visible();

// What else to buy: the fittings come with their names, priced here
cut_part("Confirmat Ø7 × 50", #{ price: 0.15 });
cut_part("Nail 1.6 × 20", #{ price: 0.02 });
cut_part("Dowel Ø8 × 38", #{ price: 0.05 });
cut_part("Screw Ø4 × 25", #{ price: 0.06 });
cut_part("Screw Ø4 × 50", #{ price: 0.08 });
cut_part("Undermount slide 450, push to open", #{ price: 24 });

// Assembly steps (Window → Assembly): the plinth, the carcass on it, then the drawers
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
let each = |from, n| {
    let out = [];
    for dr in drawers {
        out += dr.extract(from, n);
    }
    out
};
set_assembly_steps([
    #{ parts: rails, note: "Join the plinth's front and back rails between its sides with confirmats." },
    #{ parts: part.call("Bottom"), note: "Lay the bottom on the plinth and screw it down into the front and back rails from inside." },
    #{ parts: [part.call("Side L"), part.call("Side R")], note: "Stand the sides against the bottom and drive the confirmats in through them." },
    #{ parts: back, note: "Slide the back down into the grooves in the sides and the bottom." },
    #{ parts: part.call("Top"), note: "Put the top on over the back's edge, between the sides, and drive its confirmats." },
    #{ parts: each.call(0, 2), note: "The drawers: lay each one's two sides out, inside up." },
    #{ joints: of_kind.call("dowels"), note: "Glue the dowels into the sides." },
    #{ parts: each.call(2, 2), note: "Set each box's front and back onto the dowels, square the box and clamp it." },
    #{ parts: each.call(4, 1), note: "Nail the bottoms on underneath." },
    #{ joints: of_kind.call("slide"), note: "Screw the slides to the sides of the carcass, set their push-to-open units, clip each drawer onto its pair from above and push it home." },
    #{ parts: each.call(5, 1), note: "Screw the fronts on from inside the boxes, 2 mm apart, and adjust the slides until the gaps are even." },
]);
select(chest);
print("Chest of drawers " + w + " x " + d + " x " + (h + plinth) + ", " + count + " drawers, " + joints().len() + " joints, " + assembly_steps().len() + " steps");
