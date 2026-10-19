// @name Points from CSV (plugin)
// @import csv
// @description An example importer (Varstotas has it built in: File → Import → Points (CSV)…). Lines through the points of a CSV file (x,y[,z] per line, mm; other lines are skipped); a closed planar outline becomes a face.
fn num(s) {
    let t = s;
    t.trim();
    parse_float(t)
}
let pts = [];
for l in read_text(path).split("\n") {
    let c = l.split(",");
    if c.len() < 2 { continue; }
    try {
        let z = if c.len() > 2 { num(c[2]) } else { 0.0 };
        pts.push(vec3(num(c[0]), num(c[1]), z));
    } catch {
        // a header or a comment
    }
}
if pts.len() < 2 { throw "no points in " + path; }
let closed = pts.len() > 3 && pts[0] == pts[pts.len() - 1];
if closed { pts.pop(); }
polyline(pts, closed);
print("Imported " + pts.len() + " points");
