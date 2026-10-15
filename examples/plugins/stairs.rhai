// @name Stairs
// @menu Construction/Stairs…
// @description A straight flight of treads rising along +Y from the clicked point, as one group.
// @param width = 900 "Width"
// @param rise = 2700 "Total rise"
// @param going = 250 "Step depth"
// @param steps:int = 15 "Steps"
// @param tread = 40 "Tread thickness"
// @pick 1 "Click the bottom front corner of the stairs"
let p0 = points[0];
let h = rise / steps;
let parts = [];
for i in 0..steps {
    let t = box(p0 + vec3(0, i * going, (i + 1) * h - tread), width, going + 20, tread);
    t.name = "Tread " + (i + 1);
    parts.push(t);
}
let g = group(parts);
g.name = "Stairs";
select(g);
print("Stairs: " + steps + " steps of " + h + " mm");
