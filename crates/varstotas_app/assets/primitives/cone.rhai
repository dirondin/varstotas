// @generator Cone
// @description A cone (Draw → Shapes → Cone): the base centred at the origin, the apex above it.
// @param radius:length(0.1..1000000) = 500 "Radius"
// @param height:length(0.1..1000000) = 1000 "Height"
// @param segments:int(3..360) = 24 "Segments"

fn build(p) {
    explode(cone(vec3(0, 0, 0), p.radius, p.height, p.segments));
}
