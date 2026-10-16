// @generator Drawer Box
// @description A box with a lid opened by an angle, to try every parameter type.
// @param width:length(100..2000/5) = 400 "Width"
// @param depth:length(100..2000/5) = 300 "Depth"
// @param height:length(50..1000/5) = 200 "Height"
// @param wall:length(4..40) = 12 "Wall"
// @param lid:angle(0..120/5) = 30 "Lid opening"
// @param hinge:choice(back|left|right) = back "Hinge side"
// @param fill:percent = 0 "Filled"
// @param color:material = "" "Material"

fn build(p) {
    let w = p.wall;
    let parts = [
        box(vec3(0, 0, 0), p.width, p.depth, w),
        box(vec3(0, 0, w), p.width, w, p.height - w),
        box(vec3(0, p.depth - w, w), p.width, w, p.height - w),
        box(vec3(0, w, w), w, p.depth - 2 * w, p.height - w),
        box(vec3(p.width - w, w, w), w, p.depth - 2 * w, p.height - w),
    ];
    if p.fill > 0 {
        parts.push(box(vec3(w, w, w), p.width - 2 * w, p.depth - 2 * w, (p.height - w) * p.fill / 100.0));
    }
    let lid = box(vec3(0, 0, p.height), p.width, p.depth, w);
    let a = p.lid;
    if p.hinge == "back" {
        rotate(lid, vec3(0, p.depth, p.height), vec3(-1, 0, 0), a);
    } else if p.hinge == "left" {
        rotate(lid, vec3(0, 0, p.height), vec3(0, -1, 0), a);
    } else {
        rotate(lid, vec3(p.width, 0, p.height), vec3(0, 1, 0), a);
    }
    parts.push(lid);
    if p.color != "" {
        paint(parts, p.color);
    }
}
