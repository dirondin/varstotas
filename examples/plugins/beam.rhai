// @name Beam
// @name.ru Балка
// @menu Construction/Beam Between Points
// @menu.ru Построение/Балка между точками
// @description A rectangular beam from the first clicked point to the second, centred on the line.
// @description.ru Прямоугольная балка от первой указанной точки ко второй, по центру линии.
// @param w = 100 "Width"
// @param.ru w "Ширина"
// @param h = 200 "Height"
// @param.ru h "Высота"
// @pick 2 "Click the start, then the end of the beam"
// @pick.ru "Щёлкните начало, затем конец балки"
let a = points[0];
let d = points[1] - a;
let len = d.length();
if len < 1.0 { throw "the points are too close"; }
let beam = box(vec3(0, -w / 2, -h / 2), len, w, h);
let flat = sqrt(d.x * d.x + d.y * d.y);
rotate(beam, ORIGIN, Y, -atan(d.z, flat).to_degrees());
rotate(beam, ORIGIN, Z, atan(d.y, d.x).to_degrees());
move(beam, a);
beam.name = "Beam";
select(beam);
