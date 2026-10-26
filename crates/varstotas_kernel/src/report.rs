//! Reports (File → Export → Report): a parts list of groups / components with cut-list sizes,
//! and face areas per material, as CSV.
//!
//! Parts: every visible group / component instance (nested ones too), sized by the bounding
//! box of its definition in its own axes, scaled by the instance (sorted: length ≥ width ≥
//! thickness). Equal parts (same definition / name, size, material and tag) share a row with a
//! count. Volume only for solids.

use crate::model::Model;
use crate::tessellate::triangulate_face;
use crate::{DefId, MaterialId};
use glam::{DMat4, DVec3};
use std::collections::BTreeMap;
use std::fmt::Write as _;

/// One row of the parts list.
#[derive(Clone, Debug, PartialEq)]
pub struct PartRow {
    /// Instance name, else the definition name.
    pub name: String,
    pub definition: String,
    pub component: bool,
    /// Nesting depth (0 = top level).
    pub depth: usize,
    pub tag: String,
    pub material: String,
    pub count: usize,
    /// Sorted bounding box sizes (mm): length ≥ width ≥ thickness.
    pub size: [f64; 3],
    /// Surface area of one part (mm²), its own faces and nested ones.
    pub area: f64,
    /// Volume of one part (mm³) when it is a solid.
    pub volume: Option<f64>,
}

/// Face area per material (mm²) over the visible model ("Default" = no material).
pub type MaterialAreas = Vec<(String, f64)>;

fn axis_scales(m: &DMat4) -> DVec3 {
    DVec3::new(m.x_axis.truncate().length(), m.y_axis.truncate().length(), m.z_axis.truncate().length())
}

/// World area of the faces of `def` (and nested instances) under `xf`, added per material.
fn add_areas(model: &Model, def: DefId, xf: DMat4, inherited: Option<MaterialId>, out: &mut BTreeMap<String, f64>) -> f64 {
    let Some(d) = model.defs.get(def) else { return 0.0 };
    let mut total = 0.0;
    for f in d.mesh.faces.keys() {
        let a: f64 = triangulate_face(&d.mesh, f)
            .iter()
            .map(|t| {
                let p = t.map(|p| xf.transform_point3(p));
                (p[1] - p[0]).cross(p[2] - p[0]).length() * 0.5
            })
            .sum();
        // i18n-ignore-next
        let mat = d.mesh.faces[f].front.or(inherited).and_then(|m| model.materials.get(m)).map_or("Default", |m| &m.name);
        *out.entry(mat.to_string()).or_default() += a;
        total += a;
    }
    for i in d.mesh.instances.values().filter(|i| !i.hidden) {
        total += add_areas(model, i.def, xf * i.transform, i.material.or(inherited), out);
    }
    total
}

/// Parts list and material areas of the visible model.
pub fn model_report(model: &Model) -> (Vec<PartRow>, MaterialAreas) {
    let mut rows: Vec<PartRow> = Vec::new();
    fn walk(model: &Model, def: DefId, xf: DMat4, inherited: Option<MaterialId>, depth: usize, rows: &mut Vec<PartRow>) {
        let Some(d) = model.defs.get(def) else { return };
        for inst in d.mesh.instances.values().filter(|i| !i.hidden) {
            let Some(cd) = model.defs.get(inst.def) else { continue };
            let world = xf * inst.transform;
            let mat = inst.material.or(inherited);
            // Size in the part's own axes.
            let (lo, hi) = part_bounds(model, inst.def, DMat4::IDENTITY);
            let s = axis_scales(&inst.transform) * (hi - lo).max(DVec3::ZERO);
            let mut size = [s.x, s.y, s.z];
            size.sort_by(|a, b| b.total_cmp(a));
            let mut scratch = BTreeMap::new();
            let area = add_areas(model, inst.def, world, mat, &mut scratch);
            let volume = cd.mesh.is_solid().then(|| (cd.mesh.volume() * world.determinant()).abs());
            let row = PartRow {
                name: if inst.name.is_empty() { cd.name.clone() } else { inst.name.clone() },
                definition: cd.name.clone(),
                component: !cd.is_group,
                depth,
                tag: inst.tag.and_then(|t| model.tags.get(t)).map(|t| t.name.clone()).unwrap_or_default(),
                material: mat.and_then(|m| model.materials.get(m)).map(|m| m.name.clone()).unwrap_or_default(),
                count: 1,
                size: size.map(|v| (v * 1e3).round() / 1e3),
                area,
                volume,
            };
            let same = |r: &PartRow| {
                r.name == row.name
                    && r.definition == row.definition
                    && r.depth == row.depth
                    && r.tag == row.tag
                    && r.material == row.material
                    && r.size == row.size
            };
            match rows.iter_mut().find(|r| same(r)) {
                Some(r) => r.count += 1,
                None => rows.push(row),
            }
            walk(model, inst.def, world, mat, depth + 1, rows);
        }
    }
    walk(model, model.root, DMat4::IDENTITY, None, 0, &mut rows);
    let mut areas = BTreeMap::new();
    add_areas(model, model.root, DMat4::IDENTITY, None, &mut areas);
    (rows, areas.into_iter().filter(|(_, a)| *a > 0.0).collect())
}

/// Bounding box of a definition's vertices (nested instances included) under `xf`.
fn part_bounds(model: &Model, def: DefId, xf: DMat4) -> (DVec3, DVec3) {
    let mut lo = DVec3::splat(f64::INFINITY);
    let mut hi = DVec3::splat(f64::NEG_INFINITY);
    if let Some(d) = model.defs.get(def) {
        for v in d.mesh.vertices.values() {
            let p = xf.transform_point3(v.pos);
            lo = lo.min(p);
            hi = hi.max(p);
        }
        for i in d.mesh.instances.values() {
            let (a, b) = part_bounds(model, i.def, xf * i.transform);
            lo = lo.min(a);
            hi = hi.max(b);
        }
    }
    (lo, hi)
}

fn csv_field(s: &str) -> String {
    if s.contains([',', '"', '\n', ';']) { format!("\"{}\"", s.replace('"', "\"\"")) } else { s.to_string() }
}

fn num(v: f64) -> String {
    let s = format!("{v:.3}");
    let s = s.trim_end_matches('0').trim_end_matches('.');
    if s == "-0" { "0".into() } else { s.into() }
}

/// The report as CSV in the model's units: the parts list, a blank line, material areas.
pub fn report_csv(model: &Model) -> String {
    let (rows, areas) = model_report(model);
    let u = model.unit;
    let (k, sx) = (u.mm(), u.suffix());
    let mut s = String::new();
    let _ = writeln!(
        s,
        // i18n-ignore-next
        "Name,Definition,Type,Level,Count,Length ({sx}),Width ({sx}),Thickness ({sx}),Area ({sx}²),Volume ({sx}³),Material,Tag"
    );
    for r in &rows {
        let _ = writeln!(
            s,
            "{},{},{},{},{},{},{},{},{},{},{},{}",
            csv_field(&format!("{}{}", "  ".repeat(r.depth), r.name)),
            csv_field(&r.definition),
            // i18n-ignore-next
            if r.component { "Component" } else { "Group" },
            r.depth + 1,
            r.count,
            num(r.size[0] / k),
            num(r.size[1] / k),
            num(r.size[2] / k),
            num(r.area / (k * k)),
            r.volume.map(|v| num(v / (k * k * k))).unwrap_or_default(),
            csv_field(&r.material),
            csv_field(&r.tag),
        );
    }
    let _ = writeln!(s, "\nMaterial,Area ({sx}²)");
    for (name, a) in &areas {
        let _ = writeln!(s, "{},{}", csv_field(name), num(a / (k * k)));
    }
    s
}
