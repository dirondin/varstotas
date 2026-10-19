// @generator Torus
// @description A torus (Draw → Shapes → Torus) lying on the origin's plane.
// @param radius:length(0.1..1000000) = 500 "Radius (to the middle of the tube)"
// @param tube:length(0.1..1000000) = 100 "Tube radius"
// @param segments:int(3..360) = 24 "Segments"
// @param sides:int(3..360) = 12 "Tube sides"
// @check tube < radius "The tube must be thinner than the radius"

fn build(p) {
    explode(torus(vec3(0, 0, p.tube), p.radius, p.tube, p.segments, p.sides));
}
