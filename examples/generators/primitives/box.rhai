// @generator Box
// @description A box (Draw → Shapes → Box): its base from the origin along red and green.
// @param width:length(0.1..1000000) = 1000 "Width"
// @param depth:length(0.1..1000000) = 1000 "Depth"
// @param height:length(0.1..1000000) = 1000 "Height"

fn build(p) {
    explode(box(vec3(0, 0, 0), p.width, p.depth, p.height));
}
