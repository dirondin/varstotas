// Desk caddy for a 3D printer (fits a 150 x 150 x 150 mm build volume), millimetres.
// A block with rounded corners and chamfered edges; three pen holes, a tray and a phone slot
// cut out by Solid Tools. Change the dimensions and run on an empty document; export it with
// Window → Check Model….
let w = 140;
let d = 90;
let h = 80;
let r = 10;         // corner radius
let top = 2;        // chamfer along the top edge
let foot = 0.6;     // chamfer along the bottom: against the first layer spreading ("elephant foot")
let pen = 9;        // pen hole radius (Ø18)
let floor = 4;      // material left under every cut-out

// a w × d rectangle with corners of radius r, from (x, y, z), as one face facing up
fn rounded_rect(x, y, z, w, d, r) {
    let pts = [];
    let corners = [[x + w - r, y + r, -90.0], [x + w - r, y + d - r, 0.0],
                   [x + r, y + d - r, 90.0], [x + r, y + r, 180.0]];
    for c in corners {
        for i in 0..=8 {
            let a = (c[2] + i * 90.0 / 8.0) * PI() / 180.0;
            pts.push(vec3(c[0] + r * cos(a), c[1] + r * sin(a), z));
        }
    }
    face(pts)
}

// a rounded block from (x, y, z), its bottom and top edges chamfered by `low` and `high`, as a group
fn rounded_block(x, y, z, w, d, h, r, low, high) {
    let f = rounded_rect(x, y, z, w, d, r);
    let c = vec3(x + w / 2.0, y + d / 2.0, z);
    if low > 0 {
        let bottom = f;
        f = pushpull(f, low);
        // the bottom face shrinks: the faces around it lean into a chamfer
        scale(bottom, c, vec3((w - 2.0 * low) / w, (d - 2.0 * low) / d, 1.0));
    }
    f = pushpull(f, h - low - high);
    if high > 0 {
        f = pushpull(f, high);
        scale(f, c + vec3(0, 0, h), vec3((w - 2.0 * high) / w, (d - 2.0 * high) / d, 1.0));
    }
    soften(edges(), 30);
    group(faces())
}

material("PLA Orange", "#E8772E");
let body = rounded_block(0, 0, 0, w, d, h, r, foot, top);
// three pen holes along the back
for i in 0..3 {
    let hole = cylinder(vec3(20 + i * 24, d - 22, floor), pen, h, 48);
    body = subtract(body, hole)[0];
}
// a deep tray on the right
let tray = rounded_block(84, 8, floor, w - 92, d - 16, h, 6, 0, 0);
body = subtract(body, tray)[0];
// a phone / card slot across the front left
let slot = rounded_block(8, 10, h - 40, 70, 14, h, 4, 0, 0);
body = subtract(body, slot)[0];
body.name = "Desk caddy";
paint(body, "PLA Orange");
tag(body, "3D print");
select(body);
