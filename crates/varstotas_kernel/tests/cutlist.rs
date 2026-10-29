//! Cut list: parts from the model, guillotine sheet packing, boards, settings, CSV.

use varstotas_kernel::cutlist::{
    Issue, Item, Layout, Offcut, PartGrain, PartOptions, Placement, Stock, StockKind, cut_parts, pack_boards, pack_sheets, plan, plan_csv, plan_with,
};
use varstotas_kernel::glam::{DMat4, DVec3};
use varstotas_kernel::primitives::Primitive;
use varstotas_kernel::{MaterialId, Model};

const EPS: f64 = 1e-6;

/// A box group `l` × `w` × `t` (x, y, z) named `name` at `at`, painted `mat`.
fn board(m: &mut Model, name: &str, l: f64, w: f64, t: f64, at: DVec3, mat: Option<MaterialId>) {
    let mesh = Primitive::Box { w: l, d: w, h: t }.mesh().unwrap();
    let id = m.add_group_mesh(&[], mesh, "Group", DMat4::from_translation(at)).unwrap();
    let inst = &mut m.root_mesh_mut().instances[id];
    inst.name = name.into();
    inst.material = mat;
}

/// No two pieces overlap (kerf included), every piece lies on the stock.
fn check_layout(l: &Layout, kerf: f64) {
    for p in &l.pieces {
        assert!(p.x >= -EPS && p.y >= -EPS && p.x + p.w <= l.length + EPS && p.y + p.h <= l.width + EPS, "{p:?} off {l:?}");
    }
    for (i, a) in l.pieces.iter().enumerate() {
        for b in &l.pieces[i + 1..] {
            let apart = a.x + a.w + kerf <= b.x + EPS
                || b.x + b.w + kerf <= a.x + EPS
                || a.y + a.h + kerf <= b.y + EPS
                || b.y + b.h + kerf <= a.y + EPS;
            assert!(apart, "{a:?} and {b:?} overlap (kerf {kerf})");
        }
    }
}

fn item(id: usize, w: f64, h: f64, can_rotate: bool) -> Item {
    Item { id, w, h, can_rotate }
}

#[test]
fn exact_quarters_fill_one_sheet() {
    // four 1000 × 500 parts on a 2000 × 1000 sheet, no kerf
    let items: Vec<Item> = (0..4).map(|i| item(i, 1000.0, 500.0, false)).collect();
    let l = pack_sheets(&items, 2000.0, 1000.0, 0.0, 100.0);
    assert_eq!(l.len(), 1);
    assert_eq!(l[0].pieces.len(), 4);
    check_layout(&l[0], 0.0);
    assert!(l[0].offcuts.is_empty());
}

#[test]
fn kerf_is_left_between_pieces_but_not_at_the_edge() {
    // two 1000 × 1000 on 2000 × 1000: the kerf makes the second one not fit
    let items = vec![item(0, 1000.0, 1000.0, false), item(1, 1000.0, 1000.0, false)];
    assert_eq!(pack_sheets(&items, 2000.0, 1000.0, 3.0, 100.0).len(), 2);
    // 998.5 + 3 + 998.5 = 2000: fits, the last piece ends at the sheet's edge
    let items = vec![item(0, 998.5, 1000.0, false), item(1, 998.5, 1000.0, false)];
    let l = pack_sheets(&items, 2000.0, 1000.0, 3.0, 100.0);
    assert_eq!(l.len(), 1);
    check_layout(&l[0], 3.0);
}

#[test]
fn grain_keeps_parts_from_turning() {
    // 900 × 1100 parts on 2000 × 1200: only lengthwise (900 along x) two fit in a row;
    // turned (1100 along x) only one would
    let items: Vec<Item> = (0..2).map(|i| item(i, 900.0, 1100.0, false)).collect();
    let l = pack_sheets(&items, 2000.0, 1200.0, 3.0, 100.0);
    assert_eq!(l.len(), 1);
    assert!(l[0].pieces.iter().all(|p| !p.rotated && p.w == 900.0));
    // a part that only fits turned is placed turned when it may turn
    let l = pack_sheets(&[item(0, 1100.0, 1900.0, true)], 2000.0, 1200.0, 3.0, 100.0);
    assert_eq!(l.len(), 1);
    assert!(l[0].pieces[0].rotated);
    assert_eq!((l[0].pieces[0].w, l[0].pieces[0].h), (1900.0, 1100.0));
}

#[test]
fn many_parts_pack_tightly_and_never_overlap() {
    // a wardrobe-like set on 2440 × 1220 plywood
    let mut items = Vec::new();
    let sizes = [(2000.0, 580.0, 2), (1164.0, 580.0, 4), (1164.0, 100.0, 2), (1996.0, 580.0, 1), (560.0, 400.0, 6), (400.0, 150.0, 12)];
    let mut id = 0;
    let mut area = 0.0;
    for (w, h, n) in sizes {
        for _ in 0..n {
            items.push(item(id, w, h, false));
            id += 1;
            area += w * h;
        }
    }
    let l = pack_sheets(&items, 2440.0, 1220.0, 3.2, 100.0);
    assert_eq!(l.iter().map(|s| s.pieces.len()).sum::<usize>(), items.len());
    for s in &l {
        check_layout(s, 3.2);
    }
    let lower = (area / (2440.0 * 1220.0)).ceil() as usize;
    assert!(l.len() <= lower + 1, "{} sheets, lower bound {lower}", l.len());
}

#[test]
fn guillotine_cuts_run_through() {
    // every layout can be cut by through-cuts: recursively, some line along x or y crosses no
    // piece and splits the pieces into two non-empty sets
    fn separable(pieces: &[Placement]) -> bool {
        if pieces.len() <= 1 {
            return true;
        }
        for vertical in [true, false] {
            let mut cuts: Vec<f64> = pieces.iter().map(|p| if vertical { p.x + p.w } else { p.y + p.h }).collect();
            cuts.sort_by(f64::total_cmp);
            for c in cuts {
                let (lo, hi): (Vec<Placement>, Vec<Placement>) =
                    pieces.iter().partition(|p| if vertical { p.x + p.w <= c + EPS } else { p.y + p.h <= c + EPS });
                let crosses = pieces.iter().any(|p| {
                    let (a, b) = if vertical { (p.x, p.x + p.w) } else { (p.y, p.y + p.h) };
                    a < c - EPS && b > c + EPS
                });
                if !crosses && !lo.is_empty() && !hi.is_empty() {
                    return separable(&lo) && separable(&hi);
                }
            }
        }
        false
    }
    let items: Vec<Item> = (0..20).map(|i| item(i, 300.0 + (i * 37 % 400) as f64, 200.0 + (i * 53 % 300) as f64, true)).collect();
    for l in pack_sheets(&items, 2440.0, 1220.0, 3.0, 100.0) {
        assert!(separable(&l.pieces), "not guillotine: {:?}", l.pieces);
    }
}

#[test]
fn boards_take_the_shortest_length_that_holds_them() {
    // pieces 1200, 800, 700, 500 from 2000 / 3000 boards, kerf 4
    let pieces = [(0, 1200.0, 90.0), (1, 800.0, 90.0), (2, 700.0, 90.0), (3, 500.0, 90.0)];
    let l = pack_boards(&pieces, &[2000.0, 3000.0], 95.0, 4.0, 100.0);
    // 1200 + 4 + 800 + 4 + 700 + 4 + 500 = 3212 > 3000: two boards
    assert_eq!(l.len(), 2);
    let total: usize = l.iter().map(|b| b.pieces.len()).sum();
    assert_eq!(total, 4);
    for b in &l {
        assert!(b.used_length() <= b.length + EPS);
        check_layout(b, 4.0);
    }
    // a 2500 piece goes on a 3000 board; one longer than every board is left out
    let l = pack_boards(&[(0, 2500.0, 90.0), (1, 3500.0, 90.0)], &[2000.0, 3000.0], 95.0, 4.0, 100.0);
    assert_eq!(l.len(), 1);
    assert_eq!(l[0].length, 3000.0);
    assert_eq!(l[0].offcuts, vec![[2504.0, 0.0, 496.0, 95.0]]);
}

#[test]
fn parts_come_from_leaf_groups_with_their_material() {
    let mut m = Model::default();
    let ply = m.add_material("Plywood", [0.8, 0.7, 0.5, 1.0]);
    board(&mut m, "Side", 600.0, 400.0, 18.0, DVec3::ZERO, Some(ply));
    board(&mut m, "Side", 600.0, 400.0, 18.0, DVec3::new(1000.0, 0.0, 0.0), Some(ply));
    board(&mut m, "Top", 800.0, 400.0, 18.0, DVec3::new(0.0, 1000.0, 0.0), Some(ply));
    board(&mut m, "Leg", 45.0, 45.0, 700.0, DVec3::new(0.0, 2000.0, 0.0), None);
    // a cabinet group around the parts: they still count, the cabinet does not
    let ids: Vec<_> = m.root_mesh().instances.keys().collect();
    let ents: Vec<_> = ids.iter().map(|i| varstotas_kernel::EntityRef::Instance(*i)).collect();
    m.make_group(&[], &ents).unwrap();
    let parts = cut_parts(&m);
    assert_eq!(parts.len(), 3, "{parts:#?}");
    let side = parts.iter().find(|p| p.name == "Side").unwrap();
    assert_eq!(side.count, 2);
    assert_eq!(side.paths.len(), 2);
    assert_eq!(side.paths[0].len(), 2, "cabinet → side");
    assert_eq!(side.size, [600.0, 400.0, 18.0]);
    assert_eq!(side.material_name, "Plywood");
    let leg = parts.iter().find(|p| p.name == "Leg").unwrap();
    assert_eq!(leg.size, [700.0, 45.0, 45.0]);
    assert_eq!(leg.material, None);
}

#[test]
fn plan_uses_stock_settings_and_reports_issues() {
    let mut m = Model::default();
    let ply = m.add_material("Plywood", [0.8, 0.7, 0.5, 1.0]);
    let pine = m.add_material("Pine", [0.9, 0.8, 0.6, 1.0]);
    let mut sheet = Stock::sheet(2440.0, 1220.0, 18.0, true);
    sheet.price = 60.0;
    m.materials[ply].stock.push(sheet);
    let mut boards = Stock::board(95.0, 45.0, &[2400.0, 3600.0]);
    boards.price = 4.0;
    m.materials[pine].stock.push(boards);
    for i in 0..4 {
        board(&mut m, "Shelf", 1000.0, 400.0, 18.0, DVec3::new(0.0, i as f64 * 500.0, 0.0), Some(ply));
    }
    board(&mut m, "Huge", 3000.0, 400.0, 18.0, DVec3::new(0.0, 3000.0, 0.0), Some(ply));
    board(&mut m, "Thin", 300.0, 200.0, 6.0, DVec3::new(0.0, 4000.0, 0.0), Some(ply));
    board(&mut m, "Rail", 1500.0, 95.0, 45.0, DVec3::new(0.0, 5000.0, 0.0), Some(pine));
    board(&mut m, "Rail", 1500.0, 95.0, 45.0, DVec3::new(0.0, 5200.0, 0.0), Some(pine));
    board(&mut m, "Glass", 500.0, 400.0, 4.0, DVec3::new(0.0, 6000.0, 0.0), None);
    board(&mut m, "Bought", 500.0, 400.0, 18.0, DVec3::new(0.0, 7000.0, 0.0), Some(ply));
    m.cut_list.parts.insert("Bought".into(), PartOptions { exclude: true, ..Default::default() });

    let p = plan(&m);
    let idx = |name: &str| p.parts.iter().position(|x| x.name == name).unwrap();
    assert!(p.issues.contains(&Issue::TooBig { part: idx("Huge") }), "{:?}", p.issues);
    assert!(p.issues.contains(&Issue::Thickness { part: idx("Thin"), stock: 18.0 }));
    assert!(p.issues.contains(&Issue::NoStock { part: idx("Glass") }));
    assert!(!p.issues.iter().any(|i| matches!(i, Issue::NoStock { part } if *part == idx("Bought"))));

    let sheets = p.plans.iter().find(|s| s.stock.kind == StockKind::Sheet).unwrap();
    assert_eq!(sheets.layouts.len(), 1, "4 shelves and the thin part fit one sheet");
    assert_eq!(sheets.layouts[0].pieces.len(), 5);
    assert!(sheets.layouts[0].pieces.iter().all(|x| !x.rotated), "plywood with grain");
    assert_eq!(sheets.cost(), 60.0);
    let pine_plan = p.plans.iter().find(|s| s.stock.kind == StockKind::Board).unwrap();
    // two 1500 rails + kerf = 3003: one 3600 board
    assert_eq!(pine_plan.to_buy(), vec![(3600.0, 1)]);
    assert!((pine_plan.cost() - 3.6 * 4.0).abs() < EPS);
    assert!((p.cost() - (60.0 + 14.4)).abs() < EPS);

    let csv = plan_csv(&m, &p);
    assert!(csv.starts_with("Part,Count,Length (mm)"), "{csv}");
    assert!(csv.contains("Shelf,4,1000,400,18,Plywood,length,"));
    assert!(csv.contains("Huge,1,3000,400,18,Plywood,length,,larger than the stock"));
    assert!(csv.contains("Bought,1,500,400,18,Plywood,length,,excluded"));
    assert!(csv.contains("Pine,95 × 45,3600,1,"), "{csv}");

    // turning allowed per part: grain Any on a grained sheet
    m.cut_list.parts.insert("Shelf".into(), PartOptions { grain: PartGrain::Any, ..Default::default() });
    let p2 = plan(&m);
    assert_eq!(p2.plans.iter().find(|s| s.stock.kind == StockKind::Sheet).unwrap().layouts.len(), 1);
    assert!(plan_csv(&m, &p2).contains("Shelf,4,1000,400,18,Plywood,any,"));
}

#[test]
fn stock_and_settings_survive_save_and_load() {
    let mut m = Model::default();
    let ply = m.add_material("Plywood", [0.8, 0.7, 0.5, 1.0]);
    m.materials[ply].stock.push(Stock::sheet(2440.0, 1220.0, 18.0, true));
    m.cut_list.kerf = 2.5;
    m.cut_list.parts.insert("Door".into(), PartOptions { grain: PartGrain::Width, ..Default::default() });
    let json = serde_json::to_string(&m).unwrap();
    let back: Model = serde_json::from_str(&json).unwrap();
    assert_eq!(back.materials[ply].stock, m.materials[ply].stock);
    assert_eq!(back.cut_list, m.cut_list);
    // a model without them writes neither
    let plain = serde_json::to_string(&Model::default()).unwrap();
    assert!(!plain.contains("cut_list") && !plain.contains("\"stock\""));
}

#[test]
fn boards_by_section_and_per_piece_prices() {
    let mut m = Model::default();
    let pine = m.add_material("Pine", [0.9, 0.8, 0.6, 1.0]);
    let mut narrow = Stock::board(95.0, 45.0, &[3000.0, 6000.0]);
    narrow.piece_prices = vec![9.0, 14.0];
    let mut wide = Stock::board(145.0, 45.0, &[3000.0]);
    wide.price = 5.0;
    m.materials[pine].stock = vec![wide, narrow];
    board(&mut m, "Rail", 2800.0, 95.0, 45.0, DVec3::ZERO, Some(pine));
    board(&mut m, "Rail", 2800.0, 95.0, 45.0, DVec3::new(0.0, 200.0, 0.0), Some(pine));
    board(&mut m, "Wide", 1000.0, 140.0, 45.0, DVec3::new(0.0, 400.0, 0.0), Some(pine));
    board(&mut m, "Short", 500.0, 60.0, 45.0, DVec3::new(0.0, 600.0, 0.0), Some(pine));
    let p = plan(&m);
    assert!(p.issues.is_empty(), "{:?}", p.issues);
    let of = |w: f64| p.plans.iter().find(|s| s.stock.width == w).unwrap();
    // both rails on one 6 m board (14 < 2 × 9), the short piece on a 3 m one: the narrowest
    // section that holds 60 mm
    assert_eq!(of(95.0).to_buy(), vec![(6000.0, 1), (3000.0, 1)]);
    assert!((of(95.0).cost() - 23.0).abs() < EPS);
    // the 140 mm part needs the 145 section, priced per metre
    assert_eq!(of(145.0).to_buy(), vec![(3000.0, 1)]);
    assert!((of(145.0).cost() - 15.0).abs() < EPS);
    assert!(plan_csv(&m, &p).contains("Pine,95 × 45,6000,1,"));
}

#[test]
fn sheet_sizes_are_chosen_by_cost() {
    let mut m = Model::default();
    let ply = m.add_material("Plywood", [0.8, 0.7, 0.5, 1.0]);
    let mut big = Stock::sheet(2440.0, 1220.0, 18.0, false);
    big.price = 60.0;
    let mut small = Stock::sheet(1220.0, 610.0, 18.0, false);
    small.price = 20.0;
    m.materials[ply].stock = vec![big, small];
    for i in 0..5 {
        board(&mut m, "Panel", 1200.0, 600.0, 18.0, DVec3::new(0.0, i as f64 * 700.0, 0.0), Some(ply));
    }
    // four panels on a big sheet, the fifth on a small one: 80 (five small ones are 100)
    let p = plan(&m);
    assert!(p.issues.is_empty(), "{:?}", p.issues);
    let count = |w: f64| p.plans.iter().find(|s| s.stock.width == w).map_or(0, |s| s.layouts.len());
    assert_eq!((count(1220.0), count(610.0)), (1, 1), "{:?}", p.plans);
    assert!((p.cost() - 80.0).abs() < EPS);
    for sp in &p.plans {
        for l in &sp.layouts {
            check_layout(l, 3.0);
        }
    }
    // a part only the big size holds goes there, the rest still choose
    board(&mut m, "Top", 2000.0, 1000.0, 18.0, DVec3::new(0.0, 5000.0, 0.0), Some(ply));
    let p = plan(&m);
    assert!(p.issues.is_empty(), "{:?}", p.issues);
    let pieces: usize = p.plans.iter().flat_map(|s| &s.layouts).map(|l| l.pieces.len()).sum();
    assert_eq!(pieces, 6);
    assert!(p.plans.iter().any(|s| s.stock.width == 1220.0 && s.layouts.iter().any(|l| l.pieces.iter().any(|x| p.parts[x.part].name == "Top"))));
}

#[test]
fn bookshelf_example_has_a_cut_list() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/demos/bookshelf.vrst");
    let m = varstotas_kernel::io::load(&path).unwrap();
    let p = plan(&m);
    assert!(p.issues.is_empty(), "{:?}", p.issues);
    let names: Vec<&str> = p.plans.iter().map(|s| s.material_name.as_str()).collect();
    assert_eq!(names, ["Birch", "Back"], "{names:?}");
    for sp in &p.plans {
        println!("{} {:?} waste {:.0}% cost {}", sp.material_name, sp.to_buy(), sp.waste() * 100.0, sp.cost());
    }
}

#[test]
fn linear_stock_by_profile() {
    let mut m = Model::new();
    let oak = m.add_material("Oak", [0.7, 0.5, 0.3, 1.0]);
    // a round rod (no section given) and a skirting profile 70 × 15
    m.materials[oak].stock = vec![Stock::linear("Skirting 70", 70.0, 15.0, &[2500.0]), Stock::linear("Rod", 0.0, 0.0, &[1000.0, 2000.0])];
    m.materials[oak].stock[1].price = 10.0;
    for i in 0..3 {
        board(&mut m, "Skirting", 1200.0, 70.0, 15.0, DVec3::new(0.0, i as f64 * 200.0, 0.0), Some(oak));
    }
    for i in 0..2 {
        board(&mut m, "Dowel", 600.0, 20.0, 20.0, DVec3::new(0.0, 1000.0 + i as f64 * 100.0, 0.0), Some(oak));
    }
    board(&mut m, "Long", 3000.0, 20.0, 20.0, DVec3::new(0.0, 2000.0, 0.0), Some(oak));
    let p = plan(&m);
    let idx = |name: &str| p.parts.iter().position(|x| x.name == name).unwrap();
    // no thickness warnings for linear stock, the long one is too long
    assert_eq!(p.issues, vec![Issue::TooBig { part: idx("Long") }]);
    let skirting = p.plans.iter().find(|s| s.stock.name == "Skirting 70").unwrap();
    assert_eq!(skirting.stock.kind, StockKind::Linear);
    // 1200 + 3 + 1200 on one 2500, the third on another
    assert_eq!(skirting.to_buy(), vec![(2500.0, 2)]);
    let rod = p.plans.iter().find(|s| s.stock.name == "Rod").unwrap();
    // 600 + 3 + 600 = 1203: one 2000 rod, as wide as the part
    assert_eq!(rod.to_buy(), vec![(2000.0, 1)]);
    assert_eq!(rod.layouts[0].width, 20.0);
    assert!((rod.cost() - 20.0).abs() < EPS);
    assert!(plan_csv(&m, &p).contains("Oak,Rod,2000,1,"));
}

#[test]
fn offcuts_are_used_first_and_kept() {
    let mut m = Model::new();
    let ply = m.add_material("Plywood", [0.8, 0.7, 0.5, 1.0]);
    let pine = m.add_material("Pine", [0.9, 0.8, 0.6, 1.0]);
    let mut sheet = Stock::sheet(2440.0, 1220.0, 18.0, true);
    sheet.price = 60.0;
    m.materials[ply].stock = vec![sheet.clone()];
    let mut rail = Stock::board(95.0, 45.0, &[2400.0]);
    rail.price = 5.0;
    m.materials[pine].stock = vec![rail.clone()];
    board(&mut m, "Shelf", 700.0, 300.0, 18.0, DVec3::ZERO, Some(ply));
    board(&mut m, "Side", 2000.0, 1000.0, 18.0, DVec3::new(0.0, 500.0, 0.0), Some(ply));
    board(&mut m, "Rail", 900.0, 95.0, 45.0, DVec3::new(0.0, 2000.0, 0.0), Some(pine));

    let off = |material: &str, st: Stock, count| Offcut { material: material.into(), stock: Stock { price: 0.0, ..st }, count, source: String::new() };
    let mut inv = vec![
        // too small for anything, a thinner one, then the one that takes the shelf
        off("Plywood", Stock::sheet(500.0, 200.0, 18.0, true), 1),
        off("Plywood", Stock::sheet(800.0, 400.0, 12.0, true), 1),
        off("Plywood", Stock::sheet(800.0, 400.0, 18.0, true), 2),
        off("Pine", Stock::board(95.0, 45.0, &[1000.0]), 1),
    ];
    let p = plan_with(&m, &inv);
    assert!(p.issues.is_empty(), "{:?}", p.issues);
    let from_offcut: Vec<_> = p.plans.iter().filter_map(|s| s.offcut.map(|i| (i, s.layouts.len(), s.layouts[0].pieces.len()))).collect();
    assert_eq!(from_offcut, vec![(2, 1, 1), (3, 1, 1)], "the shelf and the rail from offcuts");
    // only the side is bought: one sheet, no boards
    assert_eq!(p.cost(), 60.0);
    assert!(p.plans.iter().all(|s| s.offcut.is_some() || s.stock.kind == StockKind::Sheet));
    assert!(plan_csv(&m, &p).contains("Pine,95 × 45 (offcut),1000,1,"));

    // after cutting: one 800 × 400 offcut used, the rail offcut gone, the leftovers kept
    let left = p.leftovers();
    assert!(left.iter().all(|o| o.stock.price == 0.0 && o.count >= 1));
    p.keep_offcuts(&mut inv, "Shelf.vrst");
    assert!(inv.iter().any(|o| o.source == "Shelf.vrst"));
    assert_eq!(inv.iter().find(|o| o.stock.length == 800.0 && o.stock.thickness == 18.0).map(|o| o.count), Some(1));
    assert!(!inv.iter().any(|o| o.material == "Pine" && o.stock.length == 1000.0));
    // the side's sheet leaves 440 × 1220 and 2000 × 217 (min 100), the shelf's offcut 97 × 400 is too narrow
    assert!(inv.iter().any(|o| o.material == "Plywood" && o.stock.length == 437.0 && o.stock.width == 1220.0), "{inv:?}");
    // the rail offcut 1000 - 900 - 3 = 97 is under the minimum
    assert!(!inv.iter().any(|o| o.material == "Pine"));
    // planned again with what is left, the shelf comes from the second 800 × 400
    let p2 = plan_with(&m, &inv);
    assert!(p2.plans.iter().any(|s| s.offcut.is_some_and(|i| inv[i].stock.length == 800.0)));
}

#[test]
fn edge_banding_is_summed_and_optionally_cut_off() {
    let mut m = Model::default();
    let ply = m.add_material("Plywood", [0.8, 0.7, 0.5, 1.0]);
    m.materials[ply].stock.push(Stock::sheet(2440.0, 1220.0, 18.0, true));
    board(&mut m, "Side", 800.0, 400.0, 18.0, DVec3::ZERO, Some(ply));
    board(&mut m, "Side", 800.0, 400.0, 18.0, DVec3::new(0.0, 500.0, 0.0), Some(ply));
    board(&mut m, "Back", 600.0, 300.0, 6.0, DVec3::new(0.0, 1000.0, 0.0), Some(ply));
    // both long edges and one short one of each side: 2 × (800 + 800 + 400)
    m.cut_list.parts.insert("Side".into(), PartOptions { bands: [true, true, true, false], ..Default::default() });
    m.cut_list.band_price = 2.0;
    let p = plan(&m);
    assert_eq!(p.banding.len(), 1);
    assert!((p.banding[0].length - 4000.0).abs() < EPS && p.banding[0].width == 18.0, "{:?}", p.banding);
    assert!((p.banding[0].cost() - 8.0).abs() < EPS);
    assert!((p.cost() - p.plans.iter().map(|s| s.cost()).sum::<f64>() - 8.0).abs() < EPS);
    // finished size on the pieces until the band is subtracted
    let side = p.parts.iter().position(|x| x.name == "Side").unwrap();
    let piece = |p: &varstotas_kernel::cutlist::CutPlan| *p.plans.iter().flat_map(|s| &s.layouts).flat_map(|l| &l.pieces).find(|x| x.part == side).unwrap();
    let pc = piece(&p);
    assert!((pc.w.max(pc.h) - 800.0).abs() < EPS && (pc.w.min(pc.h) - 400.0).abs() < EPS);
    m.cut_list.band = 2.0;
    m.cut_list.band_subtract = true;
    let p = plan(&m);
    let pc = piece(&p);
    assert!((pc.w.max(pc.h) - 798.0).abs() < EPS && (pc.w.min(pc.h) - 396.0).abs() < EPS, "{pc:?}");
    assert_eq!(p.parts[side].size, [800.0, 400.0, 18.0]);
    let csv = plan_csv(&m, &p);
    assert!(csv.contains("Side,2,800,400,18,Plywood,length,L1 L2 W1,cut 798 × 396"), "{csv}");
    assert!(csv.contains("Edge banding,18 × 2,4000,1,,8"), "{csv}");
    // survives save and load
    let back: Model = serde_json::from_str(&serde_json::to_string(&m).unwrap()).unwrap();
    assert_eq!(back.cut_list, m.cut_list);
}

#[test]
fn rotated_says_how_the_part_lies() {
    // grain across: the part's width runs along the sheet's grain (its length)
    let mut m = Model::default();
    let ply = m.add_material("Plywood", [0.8, 0.7, 0.5, 1.0]);
    m.materials[ply].stock.push(Stock::sheet(2440.0, 1220.0, 18.0, true));
    board(&mut m, "Door", 1000.0, 500.0, 18.0, DVec3::ZERO, Some(ply));
    m.cut_list.parts.insert("Door".into(), PartOptions { grain: PartGrain::Width, ..Default::default() });
    let pc = plan(&m).plans[0].layouts[0].pieces[0];
    assert!(pc.rotated && (pc.w - 500.0).abs() < EPS, "{pc:?}");
}
