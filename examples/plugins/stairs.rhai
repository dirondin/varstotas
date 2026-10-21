// @name Stairs
// @name.ru Лестница
// @name.lt Laiptai
// @menu Construction/Stairs…
// @menu.ru Построение/Лестница…
// @menu.lt Konstrukcija/Laiptai…
// @description A straight flight of treads rising along +Y from the clicked point, as one group.
// @description.ru Прямой марш ступеней, поднимающийся вдоль +Y от указанной точки, одной группой.
// @description.lt Tiesus laiptų maršas, kylantis +Y kryptimi nuo nurodyto taško, viena grupe.
// @param width = 900 "Width"
// @param.ru width "Ширина"
// @param.lt width "Plotis"
// @param rise = 2700 "Total rise"
// @param.ru rise "Общая высота"
// @param.lt rise "Bendras aukštis"
// @param going = 250 "Step depth"
// @param.ru going "Глубина ступени"
// @param.lt going "Pakopos gylis"
// @param steps:int = 15 "Steps"
// @param.ru steps "Ступени"
// @param.lt steps "Pakopos"
// @param tread = 40 "Tread thickness"
// @param.ru tread "Толщина проступи"
// @param.lt tread "Pakopos storis"
// @pick 1 "Click the bottom front corner of the stairs"
// @pick.ru "Щёлкните нижний передний угол лестницы"
// @pick.lt "Spustelėkite apatinį priekinį laiptų kampą"
let p0 = points[0];
let h = rise / steps;
let parts = [];
for i in 0..steps {
    let t = box(p0 + vec3(0, i * going, (i + 1) * h - tread), width, going + 20, tread);
    t.name = "Tread " + (i + 1);
    parts.push(t);
}
let g = group(parts);
g.name = "Stairs";
select(g);
print("Stairs: " + steps + " steps of " + h + " mm");
