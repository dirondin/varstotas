// An open workshop corner with a window, workbench and storage. Millimetres.
material("Floor", "#A7A096");
material("Plaster", "#E6E2D8");
material("Birch", "#D8BB87");
material("Steel", "#455A64");
material("Storage blue", "#668A9A");
let floor = box(vec3(-120,-120,-120),5240,4040,120);
floor.name = "Concrete floor";
paint(floor,"Floor");
tag(floor,"Floor");
let walls = [];
let back = box(vec3(-120,3800,0),5240,120,2800);
back.name = "Back wall";
walls.push(back);
// Opening in the left wall: build around the window instead of covering it.
for spec in [[-120,0,0,120,3800,900],[-120,0,900,120,1000,1100],[-120,2400,900,120,1400,1100],[-120,0,2000,120,3800,800]] {
    walls.push(box(vec3(spec[0],spec[1],spec[2]),spec[3],spec[4],spec[5]));
}
paint(walls,"Plaster");
let shell = group(walls);
shell.name = "Walls with window opening";
tag(shell,"Walls");
let parts = [];
let top = box(vec3(300,2850,850),2200,700,45);
top.name = "Bench top";
paint(top,"Birch");
parts.push(top);
for x in [350,2370] {
    for y in [2900,3420] {
        let leg = box(vec3(x,y,0),80,80,850);
        paint(leg,"Steel");
        parts.push(leg);
    }
}
let bench = group(parts);
bench.name = "Workshop bench";
tag(bench,"Furniture");
let shelves = [];
for x in [3750,4590] {
    let side = box(vec3(x,2850,0),40,650,2100);
    paint(side,"Steel");
    shelves.push(side);
}
for i in 0..5 {
    let shelf = box(vec3(3790,2850,120+i*460),800,650,30);
    paint(shelf,"Birch");
    shelves.push(shelf);
}
let rack = group(shelves);
rack.name = "Storage rack";
tag(rack,"Furniture");
let bins = [];
for z in [610,1070,1530] {
    for x in [3840,4250] {
        let bin = box(vec3(x,2900,z),320,430,260);
        bin.name = "Storage bin";
        paint(bin,"Storage blue");
        bins.push(bin);
    }
}
let storage = group(bins);
storage.name = "Storage bins";
tag(storage,"Storage");
select([bench,rack]);
