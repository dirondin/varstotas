// @name Stairs
// @name.ru Лестница
// @menu Construction/Stairs…
// @menu.ru Построение/Лестница…
// @description A straight flight of treads rising along +Y from the clicked point, as one group.
// @description.ru Прямой марш ступеней, поднимающийся вдоль +Y от указанной точки, одной группой.
// @param width = 900 "Width"
// @param.ru width "Ширина"
// @param rise = 2700 "Total rise"
// @param.ru rise "Общая высота"
// @param going = 250 "Step depth"
// @param.ru going "Глубина ступени"
// @param steps:int = 15 "Steps"
// @param.ru steps "Ступени"
// @param tread = 40 "Tread thickness"
// @param.ru tread "Толщина проступи"
// @pick 1 "Click the bottom front corner of the stairs"
// @pick.ru "Щёлкните нижний передний угол лестницы"
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
