// Woodworking bench, millimetres. Change the dimensions and run on an empty document.
let w = 1600;
let d = 800;
let h = 850;
let leg = 70;
material("Walnut", "#A87848");
material("Beech", "#D9B57B");
let top = box(vec3(0,0,h), w,d,45);
top.name = "Solid timber top";
paint(top,"Beech");
let parts = [top];
for x in [70,w-140] {
    for y in [70,d-140] {
        let p = box(vec3(x,y,0),leg,leg,h);
        p.name = "Leg";
        paint(p,"Walnut");
        parts.push(p);
    }
}
let z = h/3;
for y in [85,d-125] {
    let p = box(vec3(140,y,z),w-280,40,90);
    p.name = "Long lower rail";
    paint(p,"Walnut");
    parts.push(p);
}
for x in [85,w-125] {
    let p = box(vec3(x,140,z),40,d-280,90);
    p.name = "Short lower rail";
    paint(p,"Walnut");
    parts.push(p);
}
let bench = group(parts);
bench.name = "Woodworking bench";
tag(bench,"Furniture");
select(bench);
