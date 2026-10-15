// @name Beam
// @menu Construction/Beam Between Points
// @description A rectangular beam from the first clicked point to the second, centred on the line.
// @param w = 100 "Width"
// @param h = 200 "Height"
// @pick 2 "Click the start, then the end of the beam"
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
