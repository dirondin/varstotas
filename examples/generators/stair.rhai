// @generator Stair
// @description A wooden stair up to the floor height: one flight or two with a landing; the number of risers follows from the height.
// @param height:length(300..6000/10) = 2800 "Floor height"
// @param width:length(500..2000/10) = 900 "Width"
// @param going:length(200..350/5) = 270 "Going (tread)"
// @param max_riser:length(120..220/5) = 180 "Max riser"
// @param layout:choice(straight|landing) = straight "Layout"
// @param tread:length(20..60) = 40 "Tread thickness"
// @param risers:bool = true "Riser boards"
// @param finish:choice(Oak|Pine|Walnut) = Oak "Finish"
// @formula count:int = ceil(height / max_riser) "Risers"
// @formula riser = height / count "Riser height"
// @formula first:int = if layout == "landing" { count / 2 } else { count } "Risers before the landing"
// @formula run = if layout == "landing" { (count - 2) * going + width } else { (count - 1) * going } "Run"
// @check layout != "landing" || count >= 6 "A landing needs at least 6 risers"
// @check 2 * riser + going >= 550 && 2 * riser + going <= 700 "Uncomfortable step: 2 × riser + going should be 550–700 mm"

// One flight: risers `n0+1 ..= n1` from x = x0 (the nosing of its first tread), stringers and
// treads (the last riser lands on the floor above or on the landing).
fn flight(p, x0, n0, n1, wood) {
    let parts = [];
    let t = p.tread;
    let k = p.riser / p.going;
    for i in (n0 + 1)..n1 {
        let x = x0 + (i - n0 - 1) * p.going;
        parts.push(box(vec3(x - 25, 0, i * p.riser - t), p.going + 25, p.width, t));
        if p.risers {
            parts.push(box(vec3(x, 0, (i - 1) * p.riser), 18, p.width, p.riser - t));
        }
    }
    if p.risers {
        let x = x0 + (n1 - n0 - 1) * p.going;
        parts.push(box(vec3(x, 0, (n1 - 1) * p.riser), 18, p.width, p.riser));
    }
    if n1 - n0 < 2 {
        return parts;
    }
    // Stringers: the top line 50 mm above the nosings, `d` deep (vertically).
    let len = (n1 - n0 - 1) * p.going;
    let z0 = n0 * p.riser;
    let d = max(260, p.riser + 90);
    let top0 = z0 + p.riser + 50;
    let foot = x0 + (d - p.riser - 50) / k;
    for y in [0, p.width - 40] {
        let f = face([
            vec3(x0, y, z0), vec3(x0, y, top0), vec3(x0 + len, y, top0 + len * k),
            vec3(x0 + len, y, top0 + len * k - d), vec3(foot, y, z0),
        ]);
        pushpull(f, if f.normal.y > 0 { 40 } else { -40 });
    }
    parts
}

fn build(p) {
    let colors = #{ Oak: "#C8A26B", Pine: "#E3C58F", Walnut: "#6B4A2F" };
    let wood = material(p.finish, colors[p.finish]);
    let parts = [];
    if p.layout == "landing" {
        parts += flight(p, 0, 0, p.first, wood);
        // The landing: a slab one width deep at the height of riser `first`.
        let x1 = (p.first - 1) * p.going;
        let z1 = p.first * p.riser;
        parts.push(box(vec3(x1, 0, z1 - 150), p.width, p.width, 150));
        parts += flight(p, x1 + p.width, p.first, p.count, wood);
    } else {
        parts += flight(p, 0, 0, p.count, wood);
    }
    let stringers = group(faces());
    parts.push(stringers);
    paint(parts, wood);
}
