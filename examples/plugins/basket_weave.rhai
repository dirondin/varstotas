// @name Basket Weave
// @name.ru Плетёнка
// @name.lt Pintinis raštas
// @name.be Пляцёнка
// @texture_generator 256
// @description Parquet laid in squares of parallel planks, turned 90° from square to square.
// @description.ru Паркет квадратами из параллельных планок, соседние квадраты повёрнуты на 90°.
// @description.lt Parketas kvadratais iš lygiagrečių lentelių, gretimi kvadratai pasukti 90°.
// @description.be Паркет квадратамі з паралельных планак, суседнія квадраты павернутыя на 90°.
// @param squares:int = 4 "Squares across"
// @param.ru squares "Квадратов поперёк"
// @param.lt squares "Kvadratų skersai"
// @param.be squares "Квадратаў упоперак"
// @param planks:int = 3 "Planks per square"
// @param.ru planks "Планок в квадрате"
// @param.lt planks "Lentelių kvadrate"
// @param.be planks "Планак у квадраце"
// @param wood:text = "#b07a4a" "Wood colour"
// @param.ru wood "Цвет дерева"
// @param.lt wood "Medienos spalva"
// @param.be wood "Колер дрэва"
// @param variation:number = 0.25 "Variation"
// @param.ru variation "Разброс"
// @param.lt variation "Įvairovė"
// @param.be variation "Разкід"
// @param seed:int = 1 "Variant"
// @param.ru seed "Вариант"
// @param.lt seed "Variantas"
// @param.be seed "Варыянт"
//
// The Procedural Material window calls texture(size, params) and expects a material spec;
// the colour map is given inline: #{ size: side, rgb: blob } (rows from the top).

/// "#rrggbb" → [r, g, b] (0..255).
fn hex(s) {
    let t = s;
    t.trim();
    if t.starts_with("#") { t = t.sub_string(1); }
    if t.len() != 6 { return [176, 122, 74]; }
    [parse_int(t.sub_string(0, 2), 16), parse_int(t.sub_string(2, 2), 16), parse_int(t.sub_string(4, 2), 16)]
}

fn texture(size, params) {
    let n = max(params.squares, 1).to_int();
    let k = max(params.planks, 1).to_int();
    let wood = hex(params.wood);
    let seed = params.seed;
    let pixels = blob(size * size * 3, 0);
    let i = 0;
    for y in 0..size {
        let v = (y.to_float() + 0.5) / size.to_float() * n.to_float();
        let sy = v.floor();
        for x in 0..size {
            let u = (x.to_float() + 0.5) / size.to_float() * n.to_float();
            let sx = u.floor();
            let fu = u - sx;
            let fv = v - sy;
            // even squares: planks run across, odd ones: up
            let across = (sx + sy) % 2.0 == 0.0;
            let w = if across { fv } else { fu };      // across the planks
            let l = if across { fu } else { fv };      // along a plank
            let p = (w * k.to_float()).floor();
            let fw = w * k.to_float() - p;
            let joint = fw < 0.04 || fw > 0.96 || l < 0.012 || l > 0.988;
            // a plank's tone; grain: noise stretched along the plank
            let tone = 1.0 + (noise(sx * 7.0 + p, sy * 3.0, 0, seed) - 0.5) * params.variation;
            let gu = if across { u * 2.0 } else { u * 24.0 };
            let gv = if across { v * 24.0 } else { v * 2.0 };
            let grain = 0.88 + 0.24 * fbm(gu, gv, 0, 3, seed + 9);
            let f = if joint { 0.45 } else { tone * grain };
            for c in 0..3 {
                pixels[i] = clamp(wood[c] * f, 0, 255).to_int();
                i += 1;
            }
        }
    }
    #{ name: "Basket Weave", texture: #{ size: size, rgb: pixels }, roughness: 0.55 }
}
