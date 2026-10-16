// @generator Cabinet
// @description A floor cabinet: drawers on top, doors below (opened by an angle), a plinth and handles.
// @param width:length(300..2400/10) = 800 "Width"
// @param depth:length(250..700/10) = 560 "Depth"
// @param height:length(300..2400/10) = 860 "Height"
// @param board:length(12..30) = 18 "Board"
// @param plinth:length(0..200/5) = 100 "Plinth"
// @param drawers:int(0..8) = 1 "Drawers"
// @param drawer_h:length(80..400/5) = 160 "Drawer height"
// @param doors:int(0..4) = 2 "Doors"
// @param hinge:choice(left|right) = left "Single door hinge"
// @param open:angle(0..110/5) = 0 "Doors open"
// @param handles:bool = true "Handles"
// @param body:choice(White|Oak|Graphite) = White "Body"
// @param front:choice(White|Oak|Graphite|Sage) = Oak "Fronts"
// @formula inner = width - 2 * board "Inner width"
// @formula door_zone = height - plinth - board - drawers * (drawer_h + 3) "Door height"
// @formula door_w = if doors > 0 { (width - 3 * (doors + 1)) / doors } else { 0.0 } "Door width"
// @check door_zone >= 0 "The drawers do not fit: fewer drawers or a lower drawer height"
// @check doors == 0 || door_zone >= 150 "No room for doors under the drawers"

fn colour(name) {
    let c = #{ White: "#F2F0EB", Oak: "#C8A26B", Graphite: "#4A4C50", Sage: "#A9B79C" };
    material(name, c[name])
}

// A handle: a bar 12 × 25 mm standing off the front (y < 0), vertical or horizontal.
fn handle(at, len, vertical) {
    if vertical { box(at - vec3(6, 25, len / 2), 12, 25, len) } else { box(at - vec3(len / 2, 25, 6), len, 25, 12) }
}

fn build(p) {
    let t = p.board;
    let z0 = p.plinth;
    let body = [
        box(vec3(0, 0, z0), t, p.depth, p.height - z0),
        box(vec3(p.width - t, 0, z0), t, p.depth, p.height - z0),
        box(vec3(t, 0, z0), p.inner, p.depth, t),
        box(vec3(t, 0, p.height - t), p.inner, p.depth, t),
        box(vec3(t, p.depth - 6, z0 + t), p.inner, 6, p.height - z0 - 2 * t),
    ];
    if p.plinth > 0 {
        body.push(box(vec3(t, 40, 0), p.inner, t, p.plinth));
    }
    let fronts = [];
    let bars = [];
    // Drawers from the top down: a front and a box behind it.
    let top = p.height - 3;
    for i in 0..p.drawers {
        let z = top - (i + 1) * (p.drawer_h + 3) + 3;
        fronts.push(box(vec3(3, -t, z), p.width - 6, t, p.drawer_h));
        body.push(box(vec3(t + 10, 0, z + 20), p.inner - 20, p.depth - 40, p.drawer_h - 50));
        if p.handles {
            bars.push(handle(vec3(p.width / 2, -t, z + p.drawer_h / 2), min(400, p.width / 3), false));
        }
    }
    // Doors under the drawers, hinged on their outer edges (a single one on `hinge`).
    let dz = z0 + 3;
    let dh = p.door_zone - 6;
    for i in 0..p.doors {
        let x = 3 + i * (p.door_w + 3);
        let door = box(vec3(x, -t, dz), p.door_w, t, dh);
        fronts.push(door);
        let left = if p.doors == 1 { p.hinge == "left" } else { i < p.doors / 2 };
        let parts = [door];
        if p.handles {
            let hx = if left { x + p.door_w - 40 } else { x + 40 };
            let bar = handle(vec3(hx, -t, dz + dh * 0.75), min(300, dh / 3), true);
            bars.push(bar);
            parts.push(bar);
        }
        if p.open > 0 {
            if left {
                rotate(parts, vec3(x, -t, 0), vec3(0, 0, -1), p.open);
            } else {
                rotate(parts, vec3(x + p.door_w, -t, 0), vec3(0, 0, 1), p.open);
            }
        }
    }
    paint(body, colour(p.body));
    paint(fronts, colour(p.front));
    if bars.len() > 0 {
        paint(bars, material("Steel", "#9DA3A8"));
    }
}
