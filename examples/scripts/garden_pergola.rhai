// A 3.2 x 2.6 m timber pergola with exposed rafters. All dimensions are millimetres.
let w = 3200;
let d = 2600;
let h = 2400;
material("Cedar", "#BB8A55");
material("Stone", "#B1ADA4");
let parts = [];
for x in [0,w-100] {
    for y in [0,d-100] {
        let foot = box(vec3(x-40,y-40,-100),180,180,100);
        foot.name = "Stone footing";
        paint(foot,"Stone");
        parts.push(foot);
        let post = box(vec3(x,y,0),100,100,h);
        post.name = "Post";
        paint(post,"Cedar");
        parts.push(post);
    }
}
for y in [0,d-100] {
    let beam = box(vec3(-150,y,h-180),w+300,100,180);
    beam.name = "Main beam";
    paint(beam,"Cedar");
    parts.push(beam);
}
for i in 0..10 {
    let r = box(vec3(100+i*325,-200,h),65,d+400,140);
    r.name = "Rafter " + (i+1);
    paint(r,"Cedar");
    parts.push(r);
}
let pergola = group(parts);
pergola.name = "Garden pergola";
tag(pergola,"Garden");
select(pergola);
