// @generator Cabinet
// @description A floor cabinet built as it is made: a carcass on confirmats, drawers on slides at the top over a fixed shelf, doors on cup hinges below, a back in grooves and a plinth. Its joints and fittings come with it: Window → Joinery lists them, the Cut List prices them, every copy has them.
// @param width:length(300..1200/10) = 800 "Width"
// @param depth:length(300..700/10) = 560 "Depth"
// @param height:length(400..1200/10) = 860 "Height"
// @param board:length(15..25) = 18 "Board"
// @param plinth:length(0..200/5) = 100 "Plinth"
// @param drawers:int(0..6) = 1 "Drawers"
// @param drawer_h:length(100..300/5) = 160 "Drawer height"
// @param doors:int(0..2) = 2 "Doors"
// @param front:choice(overlay|inset) = overlay "Fronts"
// @param push:bool = false "Push to open"
// @param back:bool = true "Back in grooves"
// @param body:choice(White|Oak|Graphite) = White "Body"
// @param fronts:choice(White|Oak|Graphite|Sage) = Oak "Front colour"
// @formula inner = width - 2 * board "Inner width"
// @formula drawer_zone = drawers * drawer_h "Drawers' height"
// @formula door_zone = height - plinth - 3 * board - drawer_zone "Doors' height"
// @check drawers == 0 || doors == 0 || door_zone >= 200 "No room for doors under the drawers: fewer drawers or lower ones"
// @check drawers == 0 || doors > 0 || height - plinth - 2 * board >= drawer_zone "The drawers do not fit"

fn colour(name) {
    let c = #{ White: "#F2F0EB", Oak: "#C8A26B", Graphite: "#4A4C50", Sage: "#A9B79C" };
    material(name, c[name])
}

fn build(p) {
    let t = p.board;
    let z0 = p.plinth;
    let ch = p.height - z0;
    let left = box(vec3(0, 0, z0), t, p.depth, ch);
    left.name = "Side L";
    let right = box(vec3(p.width - t, 0, z0), t, p.depth, ch);
    right.name = "Side R";
    let bottom = box(vec3(t, 0, z0), p.inner, p.depth, t);
    bottom.name = "Bottom";
    let top = box(vec3(t, 0, p.height - t), p.inner, p.depth, t);
    top.name = "Top";
    let body = [left, right, bottom, top];
    for side in [left, right] {
        joint(side, bottom, "confirmat");
        joint(side, top, "confirmat");
    }
    // with both drawers and doors, a fixed shelf between them
    let split = p.drawers > 0 && p.doors > 0;
    let shelf_z = p.height - t - p.drawer_zone - t;
    if split {
        let shelf = box(vec3(t, 0, shelf_z), p.inner, p.depth, t);
        shelf.name = "Shelf";
        body.push(shelf);
        joint(left, shelf, "confirmat");
        joint(right, shelf, "confirmat");
    }
    if p.back {
        body.push(back(left, right));
    }
    if p.plinth > 0 {
        body += plinth([left, right, bottom, top], #{ height: p.plinth, setback: 40 });
    }
    paint(body, colour(p.body));

    let fronts = [];
    let fit = #{ front: p.front, push: p.push };
    if p.drawers > 0 {
        let m = fit + #{ count: p.drawers, front_thickness: t, slide: if p.push { "undermount" } else { "ball" } };
        if split {
            m.from = shelf_z + t;
            m.to = p.height - t;
        }
        for dr in drawers(left, right, m) {
            fronts.push(dr[5]);
        }
    }
    if p.doors > 0 {
        let m = fit + #{ count: p.doors, thickness: t };
        if split {
            m.from = z0 + t;
            m.to = shelf_z;
        }
        fronts += doors(left, right, m);
    }
    paint(fronts, colour(p.fronts));
}
