// Dining table and four chairs. Millimetres; each piece is a named group.
material("Oak", "#C99C60");
material("Linen", "#718C86");
fn chair(x,y,back_y) {
    let seat = box(vec3(x,y,440),430,430,30);
    seat.name = "Upholstered seat";
    paint(seat,"Linen");
    let parts = [seat];
    for dx in [20,365] {
        for dy in [20,365] {
            let p = box(vec3(x+dx,y+dy,0),45,45,440);
            paint(p,"Oak");
            parts.push(p);
        }
    }
    let back = box(vec3(x+20,y+back_y,470),390,30,390);
    back.name = "Chair back";
    paint(back,"Oak");
    parts.push(back);
    let c = group(parts);
    c.name = "Dining chair";
    return c;
}
let top = box(vec3(0,0,730),1600,850,35);
top.name = "Tabletop";
paint(top,"Oak");
let table_parts = [top];
for x in [80,1450] {
    for y in [80,700] {
        let p = box(vec3(x,y,0),70,70,730);
        paint(p,"Oak");
        table_parts.push(p);
    }
}
let table = group(table_parts);
table.name = "Dining table";
let furniture = [table,chair(180,-550,0),chair(990,-550,0),chair(180,970,400),chair(990,970,400)];
let set = group(furniture);
set.name = "Dining set for four";
tag(set,"Furniture");
select(set);
