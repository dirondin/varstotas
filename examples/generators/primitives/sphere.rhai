// @generator Sphere
// @description A sphere (Draw → Shapes → Sphere) resting on the origin.
// @param radius:length(0.1..1000000) = 500 "Radius"
// @param segments:int(4..360) = 24 "Segments"

fn build(p) {
    explode(sphere(vec3(0, 0, p.radius), p.radius, p.segments));
}
