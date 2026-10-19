// @generator Pyramid
// @description A pyramid (Draw → Shapes → Pyramid): a rectangular base from the origin, the apex above its middle.
// @param width:length(0.1..1000000) = 1000 "Width"
// @param depth:length(0.1..1000000) = 1000 "Depth"
// @param height:length(0.1..1000000) = 1000 "Height"

fn build(p) {
    explode(pyramid(vec3(0, 0, 0), p.width, p.depth, p.height));
}
