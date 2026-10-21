// @name Points from CSV (plugin)
// @name.ru Точки из CSV (плагин)
// @name.lt Taškai iš CSV (įskiepis)
// @import csv
// @description An example importer (Varstotas has it built in: File → Import → Points (CSV)…). Lines through the points of a CSV file (x,y[,z] per line, mm; other lines are skipped); a closed planar outline becomes a face.
// @description.ru Пример импортёра (встроен в Varstotas: Файл → Импорт → Точки (CSV)…). Линии через точки CSV-файла (x,y[,z] в строке, мм; прочие строки пропускаются); замкнутый плоский контур становится гранью.
// @description.lt Importo pavyzdys (integruotas į Varstotas: Failas → Importuoti → Taškai (CSV)…). Linijos per CSV failo taškus (x,y[,z] eilutėje, mm; kitos eilutės praleidžiamos); uždaras plokščias kontūras tampa paviršiumi.
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
