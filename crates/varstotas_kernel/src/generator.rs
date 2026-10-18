//! Parametric generators (track К): a Rhai script with a parameter header whose `build(p)`
//! makes the content of a component in its local axes. The kernel keeps the data — the
//! generator sources in [`Model::generators`] and, on each generated definition, the values
//! it was built with ([`Generated`]); running the script is `varstotas_script::generator`.
//!
//! Instances with equal values share one definition; changing an instance's values moves it
//! to the definition with the new values (built when there is none).
//!
//! Header (comment lines before the code, see `docs/GENERATORS.md`):
//!
//! ```text
//! // @generator Shelf
//! // @description A wall shelf.
//! // @param width:length(300..2000/10) = 800 "Width"
//! // @param shelves:int(1..10) = 3 "Shelves"
//! // @param side:choice(left|right) = left "Side"
//! // @param angle:angle(0..180/5) = 90 "Opening"
//! // @param fill:percent = 50 "Fill"
//! // @param board:material = "" "Board"
//! // @param handles:bool = true "Handles"
//! // @formula inner = width - 2 * 18 "Inner width"
//! // @check shelves * 250 <= width + 2000 "Too many shelves"
//! fn build(p) { ... }
//! ```

use crate::model::Model;
use crate::LengthUnit;
use crate::units::parse_length;
use crate::{DefId, InstanceId, MaterialId, Mesh, TagId};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap, HashSet};

/// Kind of a generator parameter.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GenKind {
    /// Model length (mm).
    Length,
    Int,
    Number,
    Bool,
    /// One of the listed strings.
    Choice(Vec<String>),
    /// Degrees.
    Angle,
    /// 0..100 unless a range is given.
    Percent,
    /// Material name ("" = none).
    Material,
}

impl GenKind {
    pub fn name(&self) -> &'static str {
        match self {
            GenKind::Length => "length",
            GenKind::Int => "int",
            GenKind::Number => "number",
            GenKind::Bool => "bool",
            GenKind::Choice(_) => "choice",
            GenKind::Angle => "angle",
            GenKind::Percent => "percent",
            GenKind::Material => "material",
        }
    }
}

/// A parameter value: numbers (lengths in mm, angles in degrees), flags and texts (choices,
/// material names).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum GenValue {
    Bool(bool),
    Num(f64),
    Text(String),
}

impl GenValue {
    pub fn num(&self) -> Option<f64> {
        match self {
            GenValue::Num(v) => Some(*v),
            _ => None,
        }
    }
    pub fn text(&self) -> Option<&str> {
        match self {
            GenValue::Text(s) => Some(s),
            _ => None,
        }
    }
    pub fn flag(&self) -> Option<bool> {
        match self {
            GenValue::Bool(b) => Some(*b),
            _ => None,
        }
    }
}

/// Parameter values by key.
pub type GenValues = BTreeMap<String, GenValue>;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GenParam {
    pub key: String,
    pub label: String,
    pub kind: GenKind,
    pub default: GenValue,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max: Option<f64>,
    /// Values are rounded to a multiple of the step (from `min`, else from 0).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub step: Option<f64>,
}

impl GenParam {
    /// `v` checked and brought into range: numbers clamped and rounded to the step (ints to
    /// whole numbers), choices must be listed. `Err` names the problem.
    pub fn normalize(&self, v: &GenValue) -> Result<GenValue, String> {
        let bad = || format!("{}: expected {}", self.key, self.kind.name());
        match &self.kind {
            GenKind::Bool => v.flag().map(GenValue::Bool).ok_or_else(bad),
            GenKind::Material => v.text().map(|s| GenValue::Text(s.to_string())).ok_or_else(bad),
            GenKind::Choice(opts) => {
                let s = match v {
                    GenValue::Text(s) => s.clone(),
                    // An index into the list.
                    GenValue::Num(i) if *i >= 0.0 && (*i as usize) < opts.len() => opts[*i as usize].clone(),
                    _ => return Err(bad()),
                };
                match opts.iter().find(|o| o.eq_ignore_ascii_case(s.trim())) {
                    Some(o) => Ok(GenValue::Text(o.clone())),
                    None => Err(format!("{}: `{s}` is not one of {}", self.key, opts.join(" / "))),
                }
            }
            _ => {
                let mut x = match v {
                    GenValue::Num(x) => *x,
                    GenValue::Text(s) => {
                        parse_number(&self.kind, s).ok_or_else(|| format!("{}: not a {}: {s}", self.key, self.kind.name()))?
                    }
                    GenValue::Bool(_) => return Err(bad()),
                };
                if !x.is_finite() {
                    return Err(bad());
                }
                if let Some(st) = self.step.filter(|s| *s > 0.0) {
                    let base = self.min.unwrap_or(0.0);
                    x = base + ((x - base) / st).round() * st;
                }
                if self.kind == GenKind::Int {
                    x = x.round();
                }
                if let Some(lo) = self.min {
                    x = x.max(lo);
                }
                if let Some(hi) = self.max {
                    x = x.min(hi);
                }
                // Tidy the float noise of the step arithmetic.
                Ok(GenValue::Num((x * 1e9).round() / 1e9))
            }
        }
    }
}

fn parse_number(kind: &GenKind, s: &str) -> Option<f64> {
    let t = s.trim();
    match kind {
        GenKind::Length => parse_length(t, LengthUnit::Millimeter),
        GenKind::Angle => t.trim_end_matches('°').trim().parse().ok(),
        GenKind::Percent => t.trim_end_matches('%').trim().parse().ok(),
        _ => t.parse().ok(),
    }
}

/// A generator: its source and the parameters parsed from the header.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Generator {
    pub name: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub description: String,
    pub params: Vec<GenParam>,
    /// Derived values (`@formula`), in order: each may use the parameters and earlier formulas.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub formulas: Vec<GenFormula>,
    /// Conditions on the values (`@check`): a failing one stops the build with its message.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub checks: Vec<GenCheck>,
    pub source: String,
}

/// `@formula key[:type] = expression "Label"`: a value computed from the parameters (a Rhai
/// expression over their keys), passed to `build` in `p` like a parameter and shown read-only
/// in Entity Info. The type only says how it is shown (length by default).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GenFormula {
    pub key: String,
    pub label: String,
    pub kind: GenKind,
    pub expr: String,
}

/// `@check expression "Message"`: values for which the expression is false are refused.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GenCheck {
    pub expr: String,
    pub message: String,
}

/// On a generated definition: the generator and the values it was built with.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Generated {
    pub generator: String,
    pub values: GenValues,
}

/// Parameter keys the Scale tool changes instead of stretching: local X, Y, Z.
pub const SIZE_KEYS: [&[&str]; 3] = [&["width", "length"], &["depth"], &["height"]];

/// `text "quoted label"` → (text, label).
fn split_label(s: &str) -> (&str, Option<String>) {
    match s.find('"') {
        Some(i) => {
            let rest = &s[i + 1..];
            let label = rest.rfind('"').map_or(rest, |j| &rest[..j]);
            (s[..i].trim(), Some(label.to_string()))
        }
        None => (s.trim(), None),
    }
}

fn parse_range(p: &mut GenParam, r: &str) -> Result<(), String> {
    let (range, step) = match r.split_once('/') {
        Some((a, b)) => (a.trim(), Some(b.trim())),
        None => (r.trim(), None),
    };
    let num = |s: &str| -> Result<Option<f64>, String> {
        if s.is_empty() {
            return Ok(None);
        }
        parse_number(&p.kind, s).map(Some).ok_or_else(|| format!("{}: bad number `{s}` in the range", p.key))
    };
    if !range.is_empty() {
        let (a, b) = range.split_once("..").ok_or_else(|| format!("{}: range is `min..max/step`", p.key))?;
        p.min = num(a.trim())?;
        p.max = num(b.trim())?;
    }
    if let Some(s) = step {
        p.step = num(s)?.filter(|s| *s > 0.0);
    }
    if let (Some(a), Some(b)) = (p.min, p.max)
        && a > b
    {
        return Err(format!("{}: min > max", p.key));
    }
    Ok(())
}

/// `expression "Label"` → (expression, label). A quoted string at the end is the label unless
/// it is an operand (`finish == "Oak"`).
fn split_expr_label(s: &str) -> (String, Option<String>) {
    let t = s.trim();
    if let Some(body) = t.strip_suffix('"')
        && let Some(i) = body.rfind('"')
    {
        let expr = body[..i].trim_end();
        let operand = expr.ends_with(|c: char| "=<>!+-*/%(,&|".contains(c));
        if !expr.is_empty() && !operand {
            return (expr.to_string(), Some(body[i + 1..].to_string()));
        }
    }
    (t.to_string(), None)
}

fn is_key(key: &str) -> bool {
    !key.is_empty() && key.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') && !key.starts_with(|c: char| c.is_ascii_digit())
}

pub(crate) fn parse_formula(s: &str) -> Result<GenFormula, String> {
    let (decl, rhs) = s.split_once('=').ok_or_else(|| format!("@formula needs `name = expression`: {s}"))?;
    let (key, ty) = match decl.split_once(':') {
        Some((k, t)) => (k.trim(), Some(t.trim())),
        None => (decl.trim(), None),
    };
    if !is_key(key) {
        return Err(format!("bad formula name `{key}`"));
    }
    let kind = match ty {
        None | Some("length") => GenKind::Length,
        Some("int") => GenKind::Int,
        Some("number") => GenKind::Number,
        Some("bool") => GenKind::Bool,
        Some("angle") => GenKind::Angle,
        Some("percent") => GenKind::Percent,
        Some("text") => GenKind::Material,
        Some(t) => return Err(format!("{key}: a formula is length, int, number, bool, angle, percent or text, not `{t}`")),
    };
    let (expr, label) = split_expr_label(rhs);
    if expr.is_empty() {
        return Err(format!("{key}: empty formula"));
    }
    Ok(GenFormula { key: key.to_string(), label: label.unwrap_or_else(|| key.replace('_', " ")), kind, expr })
}

fn parse_check(s: &str) -> Result<GenCheck, String> {
    let (expr, msg) = split_expr_label(s);
    if expr.is_empty() {
        return Err("empty @check".into());
    }
    let message = msg.unwrap_or_else(|| format!("check failed: {expr}"));
    Ok(GenCheck { expr, message })
}

pub(crate) fn parse_param(s: &str) -> Result<GenParam, String> {
    let (decl, default) = s.split_once('=').ok_or_else(|| format!("@param needs `name = default`: {s}"))?;
    let decl = decl.trim();
    let (key, ty) = match decl.split_once(':') {
        Some((k, t)) => (k.trim(), Some(t.trim())),
        None => (decl, None),
    };
    if !is_key(key) {
        return Err(format!("bad parameter name `{key}`"));
    }
    let (ty, args) = match ty {
        Some(t) => match t.split_once('(') {
            Some((n, a)) => (Some(n.trim()), Some(a.trim_end().strip_suffix(')').ok_or_else(|| format!("{key}: unclosed `(`"))?)),
            None => (Some(t), None),
        },
        None => (None, None),
    };
    let d = default.trim();
    let quoted = d.starts_with('"');
    let (dtext, label) = if let Some(rest) = d.strip_prefix('"') {
        let end = rest.find('"').ok_or_else(|| format!("unclosed quote: {s}"))?;
        (rest[..end].to_string(), split_label(&rest[end + 1..]).1)
    } else {
        let (v, label) = split_label(d);
        (v.to_string(), label)
    };
    let kind = match ty {
        Some("length") => GenKind::Length,
        Some("int") => GenKind::Int,
        Some("number") => GenKind::Number,
        Some("bool") => GenKind::Bool,
        Some("angle") => GenKind::Angle,
        Some("percent") => GenKind::Percent,
        Some("material") => GenKind::Material,
        Some("choice") => {
            let opts: Vec<String> =
                args.unwrap_or_default().split('|').map(|o| o.trim().to_string()).filter(|o| !o.is_empty()).collect();
            if opts.is_empty() {
                return Err(format!("{key}: choice needs options: choice(a|b|c)"));
            }
            GenKind::Choice(opts)
        }
        Some(t) => return Err(format!("unknown parameter type `{t}` (length, int, number, bool, choice, angle, percent, material)")),
        None if dtext == "true" || dtext == "false" => GenKind::Bool,
        None if quoted => GenKind::Material,
        None => GenKind::Length,
    };
    let mut p = GenParam {
        key: key.to_string(),
        label: label.unwrap_or_else(|| key.replace('_', " ")),
        kind,
        default: GenValue::Bool(false),
        min: None,
        max: None,
        step: None,
    };
    match &p.kind {
        GenKind::Choice(_) => {}
        GenKind::Percent if args.is_none() => {
            p.min = Some(0.0);
            p.max = Some(100.0);
        }
        _ => {
            if let Some(a) = args {
                parse_range(&mut p, a)?;
            }
        }
    }
    let raw = match &p.kind {
        GenKind::Bool => match dtext.as_str() {
            "true" => GenValue::Bool(true),
            "false" => GenValue::Bool(false),
            _ => return Err(format!("{key}: bool default is true / false")),
        },
        _ => GenValue::Text(dtext),
    };
    p.default = p.normalize(&raw)?;
    Ok(p)
}

impl GenParam {
    /// A parameter from its `@param` header text: `key:type(min..max/step) = default "Label"`.
    pub fn parse(s: &str) -> Result<GenParam, String> {
        parse_param(s)
    }
}

impl GenFormula {
    /// A formula from its `@formula` header text: `key[:type] = expression "Label"`.
    pub fn parse(s: &str) -> Result<GenFormula, String> {
        parse_formula(s)
    }
}

impl Generator {
    /// Parse the header of `source`. A generator needs `@generator Name` and a `build` function.
    pub fn parse(source: &str) -> Result<Generator, String> {
        let mut g = Generator { name: String::new(), description: String::new(), params: Vec::new(), formulas: Vec::new(), checks: Vec::new(), source: source.to_string() };
        let mut tagged = false;
        for line in source.lines() {
            let t = line.trim();
            if t.is_empty() {
                continue;
            }
            let Some(c) = t.strip_prefix("//") else { break };
            let Some(tag) = c.trim().strip_prefix('@') else { continue };
            let (key, rest) = tag.split_once(char::is_whitespace).unwrap_or((tag, ""));
            let rest = rest.trim();
            match key {
                "generator" => {
                    tagged = true;
                    g.name = rest.to_string();
                }
                "description" => g.description = rest.to_string(),
                "param" => {
                    let p = parse_param(rest)?;
                    if g.params.iter().any(|x| x.key == p.key) {
                        return Err(format!("parameter `{}` twice", p.key));
                    }
                    g.params.push(p);
                }
                "formula" => {
                    let f = parse_formula(rest)?;
                    if g.param(&f.key).is_some() || g.formulas.iter().any(|x| x.key == f.key) {
                        return Err(format!("`{}` twice (formulas and parameters share names)", f.key));
                    }
                    g.formulas.push(f);
                }
                "check" => g.checks.push(parse_check(rest)?),
                _ => {}
            }
        }
        if !tagged {
            return Err("not a generator: the header needs `// @generator Name`".into());
        }
        if g.name.trim().is_empty() {
            return Err("empty @generator name".into());
        }
        if !source.contains("fn build") {
            return Err("a generator needs `fn build(p) { ... }`".into());
        }
        Ok(g)
    }

    pub fn param(&self, key: &str) -> Option<&GenParam> {
        self.params.iter().find(|p| p.key == key)
    }

    pub fn defaults(&self) -> GenValues {
        self.params.iter().map(|p| (p.key.clone(), p.default.clone())).collect()
    }

    /// `values` over the defaults, every value normalized; unknown keys are errors.
    pub fn normalize(&self, values: &GenValues) -> Result<GenValues, String> {
        if let Some(k) = values.keys().find(|k| self.param(k).is_none()) {
            let keys: Vec<&str> = self.params.iter().map(|p| p.key.as_str()).collect();
            return Err(format!("{}: no parameter `{k}` ({})", self.name, keys.join(", ")));
        }
        self.params
            .iter()
            .map(|p| Ok((p.key.clone(), match values.get(&p.key) {
                Some(v) => p.normalize(v)?,
                None => p.default.clone(),
            })))
            .collect()
    }

    /// The parameter driving local axis `k` when scaling (see [`SIZE_KEYS`]): a length.
    pub fn size_param(&self, k: usize) -> Option<&GenParam> {
        SIZE_KEYS[k].iter().find_map(|n| self.param(n).filter(|p| p.kind == GenKind::Length))
    }
}

impl Model {
    /// The generator values of the definition of instance `inst` in the context `path`.
    pub fn instance_generated(&self, path: &[InstanceId], inst: InstanceId) -> Option<&Generated> {
        let d = self.mesh_at(path)?.instances.get(inst)?.def;
        self.defs.get(d)?.generated.as_ref()
    }

    /// A definition built by `generator` with exactly `values` (the shared one).
    pub fn generated_def(&self, generator: &str, values: &GenValues) -> Option<DefId> {
        self.defs
            .iter()
            .find(|(_, d)| d.generated.as_ref().is_some_and(|g| g.generator == generator && g.values == *values))
            .map(|(id, _)| id)
    }

    /// Add (or replace, by name) a generator. Definitions built by an older source of the same
    /// generator keep their geometry until rebuilt.
    pub fn add_generator(&mut self, g: Generator) {
        self.generators.insert(g.name.clone(), g);
    }

    /// Remove generator `name` from the document. Its components stay as plain components
    /// (their definitions forget the generator). The number of placed instances they had, or
    /// None when there is no such generator.
    pub fn remove_generator(&mut self, name: &str) -> Option<usize> {
        self.generators.remove(name)?;
        let ds: Vec<DefId> = self.defs.iter().filter(|(_, d)| d.generated.as_ref().is_some_and(|g| g.generator == name)).map(|(id, _)| id).collect();
        let placed = ds.iter().map(|d| self.instance_count(*d)).sum();
        for d in ds {
            self.defs[d].generated = None;
        }
        Some(placed)
    }

    /// Placed instances of the components built by generator `name`.
    pub fn generator_uses(&self, name: &str) -> usize {
        self.defs
            .iter()
            .filter(|(_, d)| d.generated.as_ref().is_some_and(|g| g.generator == name))
            .map(|(id, _)| self.instance_count(id))
            .sum()
    }
}

/// Source-model id → this-model id maps for [`Model::graft_content`].
#[derive(Default)]
struct Graft {
    defs: HashMap<DefId, DefId>,
    materials: HashMap<MaterialId, MaterialId>,
    tags: HashMap<TagId, TagId>,
}

impl Graft {
    fn material(&mut self, src: &Model, dst: &mut Model, m: MaterialId) -> MaterialId {
        if let Some(x) = self.materials.get(&m) {
            return *x;
        }
        let Some(mat) = src.materials.get(m) else { return m };
        let id = match dst.materials.iter().find(|(_, x)| x.name == mat.name) {
            Some((id, _)) => id,
            None => dst.materials.insert(mat.clone()),
        };
        self.materials.insert(m, id);
        id
    }

    fn tag(&mut self, src: &Model, dst: &mut Model, t: TagId) -> TagId {
        if let Some(x) = self.tags.get(&t) {
            return *x;
        }
        let Some(tag) = src.tags.get(t) else { return t };
        let id = match dst.tags.iter().find(|(_, x)| x.name == tag.name) {
            Some((id, _)) => id,
            None => dst.tags.insert(tag.clone()),
        };
        self.tags.insert(t, id);
        id
    }

    /// `mesh` of `src` with its ids mapped into `dst` (nested definitions copied).
    fn mesh(&mut self, src: &Model, dst: &mut Model, mut mesh: Mesh) -> Mesh {
        let children: Vec<(InstanceId, DefId)> = mesh.instances.iter().map(|(i, x)| (i, x.def)).collect();
        for (i, cd) in children {
            let nd = self.def(src, dst, cd);
            let inst = &mut mesh.instances[i];
            inst.def = nd;
            inst.material = inst.material.map(|m| self.material(src, dst, m));
            inst.tag = inst.tag.map(|t| self.tag(src, dst, t));
        }
        for f in mesh.faces.values_mut() {
            f.front = f.front.map(|m| self.material(src, dst, m));
            f.back = f.back.map(|m| self.material(src, dst, m));
            f.tag = f.tag.map(|t| self.tag(src, dst, t));
        }
        for e in mesh.edges.values_mut() {
            e.tag = e.tag.map(|t| self.tag(src, dst, t));
        }
        mesh
    }

    fn def(&mut self, src: &Model, dst: &mut Model, d: DefId) -> DefId {
        if let Some(n) = self.defs.get(&d) {
            return *n;
        }
        let mut def = src.defs[d].clone();
        def.mesh = self.mesh(src, dst, std::mem::take(&mut def.mesh));
        // Parametric groups made by the script keep their base.
        if let Some(p) = def.params.as_mut() {
            p.base = self.def(src, dst, p.base);
            p.variant = p.variant.map(|v| self.def(src, dst, v));
        }
        if let Some(r) = def.rigged.as_mut() {
            r.master = self.def(src, dst, r.master);
        }
        let nd = dst.defs.insert(def);
        self.defs.insert(d, nd);
        nd
    }
}

impl Model {
    /// Definitions used inside `d` (nested, not `d` itself).
    fn nested_defs(&self, d: DefId, out: &mut HashSet<DefId>) {
        let Some(def) = self.defs.get(d) else { return };
        let kids = def
            .mesh
            .instances
            .values()
            .map(|i| i.def)
            .chain(def.params.iter().flat_map(|p| [Some(p.base), p.variant]).flatten())
            .chain(def.rigged.as_ref().map(|r| r.master));
        for k in kids.collect::<Vec<_>>() {
            if out.insert(k) {
                self.nested_defs(k, out);
            }
        }
    }

    /// Replace the content of definition `into` with the root content of `src` (the model a
    /// generator script built): nested definitions are copied, materials and tags matched by
    /// name (added when missing). Nested definitions of the old content nothing uses any more
    /// are removed.
    pub fn graft_content(&mut self, into: DefId, src: &Model) {
        let mut old = HashSet::new();
        self.nested_defs(into, &mut old);
        let mut g = Graft::default();
        let mesh = g.mesh(src, self, src.root_mesh().clone());
        if let Some(d) = self.defs.get_mut(into) {
            d.mesh = mesh;
        }
        self.remove_unused(old);
    }

    /// Remove the definitions of `cands` that nothing uses (repeatedly: their children too).
    pub fn remove_unused(&mut self, mut cands: HashSet<DefId>) {
        loop {
            let bases = self.param_bases();
            let gone: Vec<DefId> = cands
                .iter()
                .copied()
                .filter(|d| *d != self.root && self.defs.contains_key(*d) && self.instance_count(*d) == 0 && !bases.contains(d))
                .collect();
            if gone.is_empty() {
                return;
            }
            for d in gone {
                self.nested_defs(d, &mut cands);
                self.defs.remove(d);
                cands.remove(&d);
            }
        }
    }
}
