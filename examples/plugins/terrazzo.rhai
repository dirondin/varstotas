// @name Terrazzo
// @name.ru Терраццо
// @name.lt Terrazzo
// @name.be Тэраца
// @texture_generator 256
// @description Polished terrazzo: chips of white, grey, terracotta and black stone in a light, slightly cloudy base.
// @description.ru Полированное терраццо: крошка белого, серого, терракотового и чёрного камня в светлой, слегка облачной основе.
// @description.lt Poliruotas terrazzo: baltų, pilkų, terakotos ir juodų akmenų trupiniai šviesiame, šiek tiek debesuotame pagrinde.
// @description.be Паліраванае тэраца: крошка белага, шэрага, теракотавага і чорнага каменю ў светлай, крыху воблачнай аснове.
// @param chips:int = 12 "Chips across"
// @param.ru chips "Крошек поперёк"
// @param.lt chips "Trupinių skersai"
// @param.be chips "Крошак упоперак"
// @param chip_size:number = 0.7 "Chip size (0..1)"
// @param.ru chip_size "Размер крошки (0..1)"
// @param.lt chip_size "Trupinio dydis (0..1)"
// @param.be chip_size "Памер крошкі (0..1)"
// @param seed:int = 3 "Variant"
// @param.ru seed "Вариант"
// @param.lt seed "Variantas"
// @param.be seed "Варыянт"
//
// One chip per cell of a chips × chips grid (a periodic grid, so the tile is seamless); the
// built-in noise() and fbm() take a period for that.

fn texture(size, params) {
    let n = max(params.chips, 1).to_int();
    let nf = n.to_float();
    let seed = params.seed;
    let palette = [[244, 242, 236], [150, 150, 146], [184, 92, 62], [40, 40, 42], [214, 196, 168]];
    let pixels = blob(size * size * 3, 0);
    let i = 0;
    for y in 0..size {
        let v = (y.to_float() + 0.5) / size.to_float();
        for x in 0..size {
            let u = (x.to_float() + 0.5) / size.to_float();
            // the base: warm light grey, a little cloudy (period 4: seamless)
            let cloud = fbm(u * 4.0, v * 4.0, 4, 3, seed);
            let base = 205.0 + 22.0 * cloud;
            let col = [base, base - 3.0, base - 9.0];
            // this cell's chip: centre, radius and colour from noise at the lattice point
            let cx = (u * nf).floor();
            let cy = (v * nf).floor();
            let r = (0.18 + 0.27 * noise(cx, cy, n, seed + 1)) * params.chip_size;
            let px = r + (1.0 - 2.0 * r) * noise(cx, cy, n, seed + 2);
            let py = r + (1.0 - 2.0 * r) * noise(cx, cy, n, seed + 3);
            let dx = u * nf - cx - px;
            let dy = v * nf - cy - py;
            // ragged edges: the distance bent by fine noise
            let d = (dx * dx + dy * dy).sqrt() + (noise(u * nf * 6.0, v * nf * 6.0, n * 6, seed + 4) - 0.5) * 0.12;
            if d < r {
                let c = palette[(noise(cx, cy, n, seed + 5) * 4.999).floor().to_int()];
                let speck = 0.92 + 0.16 * noise(u * nf * 20.0, v * nf * 20.0, n * 20, seed + 6);
                col = [c[0].to_float() * speck, c[1].to_float() * speck, c[2].to_float() * speck];
            }
            for c in 0..3 {
                pixels[i] = clamp(col[c], 0, 255).to_int();
                i += 1;
            }
        }
    }
    #{ name: "Terrazzo", texture: #{ size: size, rgb: pixels }, roughness: 0.25, width: 600 }
}
