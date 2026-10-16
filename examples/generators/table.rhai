// @generator Table
// @description A table: a top on four square or round legs set in from the edge, with an optional apron.
// @param length:length(400..4000/10) = 1600 "Length"
// @param depth:length(300..1500/10) = 900 "Depth"
// @param height:length(300..1200/10) = 750 "Height"
// @param top:length(15..60) = 30 "Top thickness"
// @param legs:choice(square|round) = square "Legs"
// @param leg:length(30..120/5) = 60 "Leg size"
// @param inset:length(0..300/5) = 50 "Leg inset"
// @param apron:bool = true "Apron"
// @param apron_h:length(40..200/5) = 80 "Apron height"
// @param finish:choice(Oak|Walnut|White|Black) = Oak "Finish"
// @formula clearance = height - top - if apron { apron_h } else { 0 } "Knee clearance"
// @check clearance >= 200 "The apron leaves no room under the top"

fn build(p) {
    let c = #{ Oak: "#C8A26B", Walnut: "#6B4A2F", White: "#F2F0EB", Black: "#2B2B2D" };
    let m = material(p.finish, c[p.finish]);
    let h = p.height - p.top;
    let parts = [box(vec3(0, 0, h), p.length, p.depth, p.top)];
    let a = p.inset;
    let s = p.leg;
    for at in [vec3(a, a, 0), vec3(p.length - a - s, a, 0), vec3(a, p.depth - a - s, 0), vec3(p.length - a - s, p.depth - a - s, 0)] {
        if p.legs == "round" {
            parts.push(cylinder(at + vec3(s / 2, s / 2, 0), s / 2, h, 24));
        } else {
            parts.push(box(at, s, s, h));
        }
    }
    if p.apron {
        let z = h - p.apron_h;
        let t = 20;
        let xs = p.length - 2 * (a + s);
        let ys = p.depth - 2 * (a + s);
        if xs > 0 && ys > 0 {
            parts.push(box(vec3(a + s, a + s / 2 - t / 2, z), xs, t, p.apron_h));
            parts.push(box(vec3(a + s, p.depth - a - s / 2 - t / 2, z), xs, t, p.apron_h));
            parts.push(box(vec3(a + s / 2 - t / 2, a + s, z), t, ys, p.apron_h));
            parts.push(box(vec3(p.length - a - s / 2 - t / 2, a + s, z), t, ys, p.apron_h));
        }
    }
    paint(parts, m);
}
