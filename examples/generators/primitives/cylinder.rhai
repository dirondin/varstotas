// @generator Cylinder
// @description A cylinder (Draw → Shapes → Cylinder) standing on its base, centred at the origin.
// @param radius:length(0.1..1000000) = 500 "Radius"
// @param height:length(0.1..1000000) = 1000 "Height"
// @param segments:int(3..360) = 24 "Segments"

fn build(p) {
    explode(cylinder(vec3(0, 0, 0), p.radius, p.height, p.segments));
}
