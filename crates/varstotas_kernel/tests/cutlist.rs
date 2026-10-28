//! Cut list: parts from the model, guillotine sheet packing, boards, settings, CSV.

use varstotas_kernel::cutlist::{
    Issue, Item, Layout, PartGrain, PartOptions, Placement, Stock, StockKind, cut_parts, pack_boards, pack_sheets, plan, plan_csv,
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
    assert!(csv.contains("Huge,1,3000,400,18,Plywood,length,larger than the stock"));
    assert!(csv.contains("Bought,1,500,400,18,Plywood,length,excluded"));
    assert!(csv.contains("Pine,95 × 45,3600,1,"), "{csv}");

    // turning allowed per part: grain Any on a grained sheet
    m.cut_list.parts.insert("Shelf".into(), PartOptions { grain: PartGrain::Any, exclude: false });
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
    m.cut_list.parts.insert("Door".into(), PartOptions { grain: PartGrain::Width, exclude: false });
    let json = serde_json::to_string(&m).unwrap();
    let back: Model = serde_json::from_str(&json).unwrap();
    assert_eq!(back.materials[ply].stock, m.materials[ply].stock);
    assert_eq!(back.cut_list, m.cut_list);
    // a model without them writes neither
    let plain = serde_json::to_string(&Model::default()).unwrap();
    assert!(!plain.contains("cut_list") && !plain.contains("\"stock\""));
}
