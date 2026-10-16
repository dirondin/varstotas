// @generator Counter
// @description A plain kitchen counter block (QA fixture).
// @param width:length(300..3000/10) = 1200 "Width"
// @param depth:length(300..800/10) = 600 "Depth"
// @param height:length(600..1000/10) = 900 "Height"

fn build(p) {
    box(vec3(0, 0, 0), p.width, p.depth, p.height);
}
