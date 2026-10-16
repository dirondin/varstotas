// @generator Railing
// @description A straight railing: posts at even spacing, a handrail, vertical balusters or horizontal bars, an optional bottom rail.
// @param length:length(300..20000/10) = 3000 "Length"
// @param height:length(500..1500/10) = 1000 "Height"
// @param spacing:length(500..3000/10) = 1500 "Max post spacing"
// @param post:length(30..150/5) = 60 "Post size"
// @param infill:choice(balusters|bars|none) = balusters "Infill"
// @param gap:length(50..300/5) = 100 "Max gap"
// @param bottom_rail:bool = true "Bottom rail"
// @param finish:choice(Steel|Black|Oak) = Black "Finish"
// @formula posts:int = ceil(length / spacing) + 1 "Posts"
// @formula bay = (length - post) / (posts - 1) "Post spacing"
// @formula per_bay:int = ceil((bay - post) / (gap + 20)) - 1 "Balusters per bay"
// @check height >= 900 || infill != "none" "A railing under 900 mm needs an infill"

fn build(p) {
    let c = #{ Steel: "#A7ADB2", Black: "#2B2B2D", Oak: "#C8A26B" };
    let m = material(p.finish, c[p.finish]);
    let s = p.post;
    let parts = [];
    for i in 0..p.posts {
        parts.push(box(vec3(i * p.bay, 0, 0), s, s, p.height - 50));
    }
    // Handrail on top, overhanging the posts a little.
    parts.push(box(vec3(-10, -(70 - s) / 2, p.height - 50), p.length + 20, 70, 50));
    let low = if p.bottom_rail { 100 } else { 0 };
    if p.bottom_rail {
        parts.push(box(vec3(0, s / 2 - 20, 80), p.length, 40, 20));
    }
    let top = p.height - 50;
    for b in 0..(p.posts - 1) {
        let x0 = b * p.bay + s;
        let free = p.bay - s;
        if p.infill == "balusters" && p.per_bay > 0 {
            let step = free / (p.per_bay + 1);
            for k in 1..=p.per_bay {
                parts.push(box(vec3(x0 + k * step - 10, s / 2 - 10, low), 20, 20, top - low));
            }
        } else if p.infill == "bars" {
            let n = ((top - low) / (p.gap + 12)).to_int();
            let step = (top - low) / (n + 1);
            for k in 1..=n {
                parts.push(box(vec3(x0, s / 2 - 6, low + k * step - 6), free, 12, 12));
            }
        }
    }
    paint(parts, m);
}
