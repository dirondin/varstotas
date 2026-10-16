// @generator Shelf
// @description An open shelf unit: two sides, a bottom, a top, evenly spaced shelves and a back.
// @param width:length(200..3000/10) = 800 "Width"
// @param depth:length(100..800/10) = 300 "Depth"
// @param height:length(200..3000/10) = 1200 "Height"
// @param shelves:int(0..12) = 3 "Shelves"
// @param board:length(10..50/1) = 18 "Board"
// @param back:bool = true "Back panel"
// @param finish:choice(Oak|Walnut|White) = Oak "Finish"
// @param paint_with:material = "" "Material"
// @formula gap = (height - board) / (shelves + 1) - board "Gap between shelves"
// @check gap >= 100 "Shelves closer than 100 mm: fewer shelves or a taller unit"

fn build(p) {
    let colors = #{ Oak: "#C8A26B", Walnut: "#6B4A2F", White: "#F2F0EB" };
    let m = if p.paint_with == "" { material(p.finish, colors[p.finish]) } else { p.paint_with };
    let t = p.board;
    let inner = p.width - 2 * t;
    let parts = [
        box(vec3(0, 0, 0), t, p.depth, p.height),
        box(vec3(p.width - t, 0, 0), t, p.depth, p.height),
        box(vec3(t, 0, 0), inner, p.depth, t),
        box(vec3(t, 0, p.height - t), inner, p.depth, t),
    ];
    let step = p.gap + t;
    for i in 1..=p.shelves {
        parts.push(box(vec3(t, 0, i * step), inner, p.depth - 10, t));
    }
    if p.back {
        parts.push(box(vec3(t, p.depth - 6, t), inner, 6, p.height - 2 * t));
    }
    paint(parts, m);
}
