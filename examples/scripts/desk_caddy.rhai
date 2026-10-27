// Desk caddy for a 3D printer (fits a 150 x 150 x 150 mm build volume), millimetres.
// A solid block with three pen holes, a tray and a phone slot cut out by Solid Tools.
// Change the dimensions and run on an empty document; export it with Window → Check Model….
let w = 140;
let d = 90;
let h = 80;
let pen = 9;      // pen hole radius (Ø18)
let floor = 4;    // material left under every cut-out
material("PLA Orange", "#E8772E");
let body = box(vec3(0,0,0), w,d,h);
body.name = "Desk caddy";
// three pen holes along the back
for i in 0..3 {
    let hole = cylinder(vec3(20 + i*24, d-22, floor), pen, h, 48);
    body = subtract(body, hole)[0];
}
// a deep tray on the right
let tray = box(vec3(84, 8, floor), w-92, d-16, h);
body = subtract(body, tray)[0];
// a phone / card slot across the front left
let slot = box(vec3(8, 10, h-40), 70, 14, h);
body = subtract(body, slot)[0];
body.name = "Desk caddy";
paint(body, "PLA Orange");
tag(body, "3D print");
select(body);
