// Ten 170 mm risers, 250 mm going, timber treads and a supported upper landing.
let n = 10;
let rise = 170;
let going = 250;
let width = 1000;
material("Concrete", "#B7BCC1");
material("Timber treads", "#BD8C56");
material("Steel", "#495560");
let parts = [];
for i in 0..n {
    let step = box(vec3(0,i*going,0),width,going,(i+1)*rise-30);
    step.name = "Riser " + (i+1);
    paint(step,"Concrete");
    parts.push(step);
    let tread = box(vec3(-15,i*going-15,(i+1)*rise-30),width+30,going+15,30);
    tread.name = "Tread " + (i+1);
    paint(tread,"Timber treads");
    parts.push(tread);
}
let landing = box(vec3(0,n*going,n*rise-150),width,1100,150);
landing.name = "Upper landing";
paint(landing,"Concrete");
parts.push(landing);
for x in [40,width-120] {
    let post = box(vec3(x,n*going+950,0),80,80,n*rise-150);
    post.name = "Landing support";
    paint(post,"Steel");
    parts.push(post);
}
let stairs = group(parts);
stairs.name = "Staircase with landing";
tag(stairs,"Structure");
select(stairs);
