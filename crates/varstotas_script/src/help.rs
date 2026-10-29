//! Signatures and one-line descriptions of the script API, for the editor's completion and
//! parameter hints (the full reference is `docs/SCRIPTING.md`).

/// One API function: its overloads (`name(arg, arg)`) and what it does.
#[derive(Clone, Copy, Debug)]
pub struct FnDoc {
    pub name: &'static str,
    pub sigs: &'static [&'static str],
    pub doc: &'static str,
}

impl FnDoc {
    /// The description in the UI language (`doc` is its key).
    pub fn text(&self) -> String {
        varstotas_i18n::tr(self.doc)
    }
}

const fn f(name: &'static str, sigs: &'static [&'static str], doc: &'static str) -> FnDoc {
    FnDoc { name, sigs, doc }
}

/// Every API function (the names of [`crate::api_names`]) plus the Rhai built-ins scripts use most.
pub const FNS: &[FnDoc] = &[
    f("vec3", &["vec3(x, y, z)", "vec3(x, y)"], "script-fn-vec3"),
    f("line", &["line(a, b)"], "script-fn-line"),
    f("polyline", &["polyline(points)", "polyline(points, closed)"], "script-fn-polyline"),
    f("face", &["face(points)"], "script-fn-face"),
    f("rect", &["rect(origin, w, d)"], "script-fn-rect"),
    f("circle", &["circle(center, r)", "circle(center, r, segments)", "circle(center, r, segments, normal)"], "script-fn-circle",
    ),
    f("polygon", &["polygon(center, r, sides)", "polygon(center, r, sides, normal)"], "script-fn-polygon"),
    f("arc", &["arc(center, start_point, degrees)", "arc(center, start_point, degrees, segments)", "arc(center, start_point, degrees, segments, normal)"], "script-fn-arc",
    ),
    f("box", &["box(origin, sx, sy, sz)"], "script-fn-box"),
    f("cylinder", &["cylinder(center, r, h)", "cylinder(center, r, h, segments)"], "script-fn-cylinder"),
    f("cone", &["cone(center, r, h)", "cone(center, r, h, segments)"], "script-fn-cone"),
    f("pyramid", &["pyramid(origin, w, d, h)"], "script-fn-pyramid"),
    f("sphere", &["sphere(center, r)", "sphere(center, r, segments)"], "script-fn-sphere"),
    f("torus", &["torus(center, r, tube)", "torus(center, r, tube, segments, sides)"], "script-fn-torus"),
    f("pushpull", &["pushpull(face, distance)"], "script-fn-pushpull"),
    f("offset", &["offset(face, distance)"], "script-fn-offset"),
    f("follow_me", &["follow_me(face, edges)"], "script-fn-follow-me"),
    f("erase", &["erase(x)"], "script-fn-erase"),
    f("move", &["move(x, vector)"], "script-fn-move"),
    f("copy", &["copy(x, vector)"], "script-fn-copy"),
    f("rotate", &["rotate(x, center, axis, degrees)"], "script-fn-rotate"),
    f("scale", &["scale(x, center, k)", "scale(x, center, vec3(kx, ky, kz))"], "script-fn-scale"),
    f("group", &["group(x)"], "script-fn-group"),
    f("component", &["component(x, name)"], "script-fn-component"),
    f("explode", &["explode(g)"], "script-fn-explode"),
    f("make_unique", &["make_unique(g)"], "script-fn-make-unique"),
    f("union", &["union(a, b)"], "script-fn-union"),
    f("subtract", &["subtract(a, b)"], "script-fn-subtract"),
    f("intersect", &["intersect(a, b)"], "script-fn-intersect"),
    f("trim", &["trim(a, b)"], "script-fn-trim"),
    f("split", &["split(a, b)"], "script-fn-split"),
    f("outer_shell", &["outer_shell(groups)"], "script-fn-outer-shell"),
    f("reverse", &["reverse(faces)"], "script-fn-reverse"),
    f("soften", &["soften(x, degrees)"], "script-fn-soften"),
    f("material", &["material(name, \"#RRGGBB\")", "material(name, [r, g, b])"], "script-fn-material"),
    f("paint", &["paint(x, material)", "paint(x, ())"], "script-fn-paint"),
    f("tag", &["tag(x, name)"], "script-fn-tag"),
    f("make_parametric", &["make_parametric(g)"], "script-fn-make-parametric"),
    f("set_params", &["set_params(g, #{ key: value, ... })", "set_params(g, #{ x, y, z, count, axis, gap, stretch })"], "script-fn-set-params",
    ),
    f("params", &["params(g)"], "script-fn-params"),
    f("reset_params", &["reset_params(g)"], "script-fn-reset-params"),
    f("generators", &["generators()"], "script-fn-generators"),
    f("add_generator", &["add_generator(source)"], "script-fn-add-generator"),
    f("generate", &["generate(name, at)", "generate(name, at, #{ key: value, ... })"], "script-fn-generate"),
    f("dimension", &["dimension(a, b, offset)"], "script-fn-dimension"),
    f("note", &["note(at, offset, text)"], "script-fn-note"),
    f("enter", &["enter(g)"], "script-fn-enter"),
    f("exit", &["exit()"], "script-fn-exit"),
    f("faces", &["faces()"], "script-fn-faces"),
    f("edges", &["edges()"], "script-fn-edges"),
    f("groups", &["groups()"], "script-fn-groups"),
    f("all", &["all()"], "script-fn-all"),
    f("entities", &["entities(g)"], "script-fn-entities"),
    f("child", &["child(name)", "child(name, deep)", "child(g, name)", "child(g, name, deep)"], "script-fn-child",
    ),
    f("children", &["children()", "children(deep)", "children(g)", "children(g, deep)"], "script-fn-children",
    ),
    f("show", &["show(x)", "show(x, on)"], "script-fn-show"),
    f("hide", &["hide(x)"], "script-fn-hide"),
    f("selection", &["selection()"], "script-fn-selection"),
    f("select", &["select(x)", "select(())"], "script-fn-select"),
    f("bounds", &["bounds(x)"], "script-fn-bounds"),
    f("model_bounds", &["model_bounds()"], "script-fn-model-bounds"),
    f("length", &["length(text)"], "script-fn-length"),
    f("units", &["units()"], "script-fn-units"),
    f("materials", &["materials()"], "script-fn-materials"),
    f("cut_list", &["cut_list()"], "script-fn-cut-list"),
    f("set_stock", &["set_stock(material, #{ kind: \"sheet\", length: 2440, width: 1220, thickness: 18, grain: true, price: 60 })", "set_stock(material, #{ kind: \"board\", width: 95, thickness: 45, lengths: [3000, 6000], prices: [9, 16] })", "set_stock(material, #{ kind: \"linear\", name: \"Ø 20 rod\", width: 20, thickness: 20, lengths: [1000, 2000], price: 3 })", "set_stock(material, [stock, …])", "set_stock(material, ())"], "script-fn-set-stock"),
    f("cut_settings", &["cut_settings(#{ kerf: 3, trim: 10, min_offcut: 100, band: 1 })"], "script-fn-cut-settings"),
    f("cut_part", &["cut_part(name, #{ grain: \"any\", bands: [true, true, false, false] })", "cut_part(\"Hinge\", #{ hardware: true, price: 2.5 })"], "script-fn-cut-part"),
    f("cut_hardware", &["cut_hardware([#{ name: \"Screw 4 × 16\", count: 24, price: 0.05 }])"], "script-fn-cut-hardware"),
    f("joint", &["joint(\"Side\", \"Shelf\", \"dowels\")", "joint(side, top, \"dovetail\", #{ count: 5, pin: 8, angle: 9.5 })"], "script-fn-joint"),
    f("joints", &["joints()"], "script-fn-joints"),
    f("set_joint", &["set_joint(0, #{ count: 4, diameter: 10 })", "set_joint(0, #{ kind: \"tenon\", through: true })"], "script-fn-set-joint"),
    f("remove_joint", &["remove_joint(0)"], "script-fn-remove-joint"),
    f("all_faces", &["all_faces()"], "script-fn-all-faces"),
    f("read_text", &["read_text(path)"], "script-fn-read-text"),
    f("lang", &["lang()"], "script-fn-lang"),
    f("platform", &["platform()"], "script-fn-platform"),
    f("write_text", &["write_text(path, text)"], "script-fn-write-text"),
    f("file_exists", &["file_exists(path)"], "script-fn-file-exists"),
    f("list_files", &["list_files(dir, ext)"], "script-fn-list-files"),
    f("list_dirs", &["list_dirs(dir)"], "script-fn-list-dirs"),
    f("parse_json", &["parse_json(text)"], "script-fn-parse-json"),
    f("fetch_text", &["fetch_text(url)", "fetch_text(url, headers)"], "script-fn-fetch-text"),
    f("fetch_file", &["fetch_file(url, path)", "fetch_file(url, path, headers)"], "script-fn-fetch-file"),
    f("url_encode", &["url_encode(text)"], "script-fn-url-encode"),
    f("print", &["print(value)"], "script-fn-print"),
    f("len", &["len(array)"], "script-fn-len"),
    f("range", &["range(from, to)", "range(from, to, step)"], "script-fn-range"),
    f("min", &["min(a, b)"], "script-fn-min"),
    f("max", &["max(a, b)"], "script-fn-max"),
    f("clamp", &["clamp(x, lo, hi)"], "script-fn-clamp"),
    f("ceil", &["ceil(x)"], "script-fn-ceil"),
    f("noise", &["noise(x, y)", "noise(x, y, period, seed)"], "script-fn-noise"),
    f("fbm", &["fbm(x, y, period, octaves, seed)"], "script-fn-fbm"),
];

/// Sections of the in-app reference: a title and the [`FNS`] names in it (every function is
/// in one).
pub const GROUPS: &[(&str, &[&str])] = &[
    ("script-group-points-and-vectors", &["vec3", "length", "units"]),
    ("script-group-drawing", &["line", "polyline", "face", "rect", "circle", "polygon", "arc"]),
    ("script-group-solids", &["box", "cylinder", "cone", "pyramid", "sphere", "torus"]),
    ("script-group-editing", &["pushpull", "offset", "follow_me", "erase", "move", "copy", "rotate", "scale", "reverse", "soften"]),
    ("script-group-groups-and-components", &["group", "component", "explode", "make_unique", "enter", "exit"]),
    ("script-group-finding-objects", &["child", "children", "faces", "edges", "groups", "all", "entities", "all_faces", "selection", "select", "bounds", "model_bounds"]),
    ("script-group-visibility-materials-tags", &["show", "hide", "material", "paint", "materials", "tag"]),
    ("script-group-cut-list", &["cut_list", "set_stock", "cut_settings", "cut_part", "cut_hardware"]),
    ("script-group-joinery", &["joint", "joints", "set_joint", "remove_joint"]),
    ("script-group-solid-tools", &["union", "subtract", "intersect", "trim", "split", "outer_shell"]),
    ("script-group-parameters-and-generators", &["make_parametric", "set_params", "params", "reset_params", "generators", "add_generator", "generate"]),
    ("script-group-dimensions-and-notes", &["dimension", "note"]),
    ("script-group-files-and-network", &["read_text", "write_text", "file_exists", "list_files", "list_dirs", "parse_json", "fetch_text", "fetch_file", "url_encode"]),
    ("script-group-output-and-numbers", &["print", "lang", "platform", "len", "range", "min", "max", "clamp", "ceil", "noise", "fbm"]),
];

/// What a script expression is, as far as the editor can tell (for completion after a dot).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ty {
    Face,
    Edge,
    Group,
    Vector,
    Num,
    Bool,
    Str,
    /// `bounds()`: `#{ min, max, size, center }`.
    Bounds,
    Faces,
    Edges,
    Groups,
    Vecs,
    Array,
    Any,
}

impl Ty {
    /// Element type of an array.
    fn elem(self) -> Ty {
        match self {
            Ty::Faces => Ty::Face,
            Ty::Edges => Ty::Edge,
            Ty::Groups => Ty::Group,
            Ty::Vecs => Ty::Vector,
            _ => Ty::Any,
        }
    }
}

/// A property (`f.area`) or method (`v.dot(b)`, `sig` has the parentheses) after a dot.
#[derive(Clone, Copy, Debug)]
pub struct Member {
    pub name: &'static str,
    /// `None` for a property, else the call (`dot(b)`).
    pub sig: Option<&'static str>,
    pub doc: &'static str,
    pub on: &'static [Ty],
    pub ty: Ty,
}

impl Member {
    /// The description in the UI language (`doc` is its key).
    pub fn text(&self) -> String {
        varstotas_i18n::tr(self.doc)
    }
}

const fn prop(name: &'static str, doc: &'static str, on: &'static [Ty], ty: Ty) -> Member {
    Member { name, sig: None, doc, on, ty }
}

const fn meth(name: &'static str, sig: &'static str, doc: &'static str, on: &'static [Ty], ty: Ty) -> Member {
    Member { name, sig: Some(sig), doc, on, ty }
}

use Ty::*;
const ENT: &[Ty] = &[Face, Edge, Group];
const ARR: &[Ty] = &[Faces, Edges, Groups, Vecs, Array];

/// Properties and methods of entities, vectors, arrays, strings and bounds.
pub const MEMBERS: &[Member] = &[
    prop("kind", "script-member-kind", ENT, Str),
    prop("valid", "script-member-valid", ENT, Bool),
    prop("area", "script-member-area", &[Face], Num),
    prop("normal", "script-member-normal", &[Face], Vector),
    prop("edges", "script-member-edges", &[Face], Edges),
    prop("faces", "script-member-faces", &[Face, Edge], Faces),
    prop("length", "script-member-length", &[Edge], Num),
    prop("soft", "script-member-soft", &[Edge], Bool),
    prop("center", "script-member-center", &[Face, Edge, Group], Vector),
    prop("vertices", "script-member-vertices", &[Face, Edge], Vecs),
    prop("name", "script-member-name", &[Group], Str),
    prop("locked", "script-member-locked", ENT, Bool),
    prop("hidden", "script-member-hidden", ENT, Bool),
    prop("bounds", "script-member-bounds", ENT, Bounds),
    prop("size", "script-member-size", ENT, Vector),
    prop("min", "script-member-min", ENT, Vector),
    prop("max", "script-member-max", ENT, Vector),
    prop("definition", "script-member-definition", &[Group], Str),
    prop("material", "script-member-material", &[Face, Group], Str),
    prop("tag", "script-member-tag", ENT, Str),
    prop("x", "script-member-x", &[Vector], Num),
    prop("y", "script-member-y", &[Vector], Num),
    prop("z", "script-member-z", &[Vector], Num),
    meth("length", "length()", "script-member-length-2", &[Vector], Num),
    meth("normalize", "normalize()", "script-member-normalize", &[Vector], Vector),
    meth("dot", "dot(b)", "script-member-dot", &[Vector], Num),
    meth("cross", "cross(b)", "script-member-cross", &[Vector], Vector),
    meth("distance", "distance(b)", "script-member-distance", &[Vector], Num),
    prop("min", "script-member-min-2", &[Bounds], Vector),
    prop("max", "script-member-max-2", &[Bounds], Vector),
    prop("size", "script-member-size-2", &[Bounds], Vector),
    prop("center", "script-member-center-2", &[Bounds], Vector),
    meth("len", "len()", "script-member-len", ARR, Num),
    meth("is_empty", "is_empty()", "script-member-is-empty", ARR, Bool),
    meth("push", "push(value)", "script-member-push", ARR, Any),
    meth("pop", "pop()", "script-member-pop", ARR, Any),
    meth("insert", "insert(index, value)", "script-member-insert", ARR, Any),
    meth("remove", "remove(index)", "script-member-remove", ARR, Any),
    meth("contains", "contains(value)", "script-member-contains", &[Faces, Edges, Groups, Vecs, Array, Str], Bool),
    meth("index_of", "index_of(value)", "script-member-index-of", ARR, Num),
    meth("filter", "filter(|x| condition)", "script-member-filter", ARR, Any),
    meth("map", "map(|x| value)", "script-member-map", ARR, Array),
    meth("reduce", "reduce(|sum, x| sum + x, start)", "script-member-reduce", ARR, Any),
    meth("len", "len()", "script-member-len-2", &[Str], Num),
    meth("to_upper", "to_upper()", "script-member-to-upper", &[Str], Any),
    meth("to_lower", "to_lower()", "script-member-to-lower", &[Str], Any),
    meth("trim", "trim()", "script-member-trim", &[Str], Any),
    meth("split", "split(separator)", "script-member-split", &[Str], Array),
    meth("starts_with", "starts_with(text)", "script-member-starts-with", &[Str], Bool),
    meth("to_string", "to_string()", "script-member-to-string", &[Face, Edge, Group, Vector, Num], Str),
];

/// What a function returns.
fn fn_ty(name: &str) -> Ty {
    match name {
        "face" | "rect" | "circle" | "polygon" | "pushpull" | "offset" => Face,
        "box" | "cylinder" | "cone" | "pyramid" | "sphere" | "torus" | "child" | "group" | "component" | "union" | "subtract" | "intersect" | "trim" | "outer_shell" => Group,
        "line" | "arc" | "edges" => Edges,
        "faces" | "all_faces" => Faces,
        "groups" | "children" | "split" => Groups,
        "all" | "entities" | "selection" | "copy" | "materials" | "range" | "list_files" | "list_dirs" => Array,
        "vec3" => Vector,
        "bounds" | "model_bounds" => Bounds,
        "material" | "units" | "read_text" | "fetch_text" | "lang" | "platform" => Str,
        "length" | "len" => Num,
        _ => Any,
    }
}

/// Members that apply to `ty` (all entity and vector members for an unknown type).
pub fn members_for(ty: Ty) -> Vec<&'static Member> {
    let any = [Face, Edge, Group, Vector];
    let mut out: Vec<&Member> = Vec::new();
    for m in MEMBERS {
        let fits = if ty == Any { m.on.iter().any(|t| any.contains(t)) } else { m.on.contains(&ty) };
        if fits && !out.iter().any(|o| o.name == m.name) {
            out.push(m);
        }
    }
    out
}

fn is_ident(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

/// The completion context after a dot: the receiver's type, the typed part of the member name
/// and its start (char indices into `src`; `cursor` is a char index).
pub fn member_context(src: &str, cursor: usize) -> Option<(Ty, String, usize)> {
    let chars: Vec<char> = src.chars().take(cursor).collect();
    let mut i = chars.len();
    while i > 0 && is_ident(chars[i - 1]) {
        i -= 1;
    }
    let prefix: String = chars[i..].iter().collect();
    if i == 0 || chars[i - 1] != '.' || in_comment_or_string(&chars[..i]) {
        return None;
    }
    let ty = expr_ty(&chars, i - 1, 0)?;
    Some((ty, prefix, i))
}

/// The cursor is inside a `//` comment or a string.
fn in_comment_or_string(chars: &[char]) -> bool {
    let line_start = chars.iter().rposition(|c| *c == '\n').map_or(0, |p| p + 1);
    let line = &chars[line_start..];
    let mut quote: Option<char> = None;
    for (k, c) in line.iter().enumerate() {
        match quote {
            Some(q) if *c == q => quote = None,
            Some(_) => {}
            None if *c == '"' || *c == '`' => quote = Some(*c),
            None if *c == '/' && line.get(k + 1) == Some(&'/') => return true,
            None => {}
        }
    }
    quote.is_some()
}

/// Type of the expression ending just before `end` (the dot): `name`, `name(…)`, `x[…]`,
/// `x.member`, a constant.
fn expr_ty(chars: &[char], end: usize, depth: usize) -> Option<Ty> {
    if depth > 8 {
        return Some(Any);
    }
    let mut j = end;
    while j > 0 && chars[j - 1] == ' ' {
        j -= 1;
    }
    if j == 0 {
        return None;
    }
    let last = chars[j - 1];
    // `x[i]`: an item of the array.
    if last == ']' || last == ')' {
        let open_c = if last == ']' { '[' } else { '(' };
        let mut d = 0;
        let mut k = j;
        while k > 0 {
            k -= 1;
            if chars[k] == last {
                d += 1;
            } else if chars[k] == open_c {
                d -= 1;
                if d == 0 {
                    break;
                }
            }
        }
        if last == ']' {
            return Some(if k > 0 && (is_ident(chars[k - 1]) || matches!(chars[k - 1], ')' | ']')) { expr_ty(chars, k, depth + 1)?.elem() } else { Array });
        }
        let mut s = k;
        while s > 0 && is_ident(chars[s - 1]) {
            s -= 1;
        }
        let name: String = chars[s..k].iter().collect();
        if name.is_empty() {
            return Some(Any);
        }
        if s > 0 && chars[s - 1] == '.' {
            return Some(member_ty(expr_ty(chars, s - 1, depth + 1)?, &name));
        }
        return Some(fn_ty(&name));
    }
    if !is_ident(last) {
        return None;
    }
    let mut s = j;
    while s > 0 && is_ident(chars[s - 1]) {
        s -= 1;
    }
    let name: String = chars[s..j].iter().collect();
    if name.starts_with(|c: char| c.is_ascii_digit()) {
        return None; // `1.5`
    }
    if s > 0 && chars[s - 1] == '.' {
        return Some(member_ty(expr_ty(chars, s - 1, depth + 1)?, &name));
    }
    if CONSTANTS.contains(&name.as_str()) {
        return Some(Vector);
    }
    Some(var_ty(chars, &name, s, depth))
}

fn member_ty(recv: Ty, name: &str) -> Ty {
    let ms = members_for(recv);
    ms.iter().find(|m| m.name == name).map_or(Any, |m| m.ty)
}

/// Type of a variable from its last `let name = …` / `for name in …` before `before`.
fn var_ty(chars: &[char], name: &str, before: usize, depth: usize) -> Ty {
    let text: String = chars[..before].iter().collect();
    if let Some((_, t)) = header_vars(&text).into_iter().find(|(n, _)| n == name) {
        return t;
    }
    match name {
        "points" => return Vecs,
        "objects" => return Array,
        _ => {}
    }
    let mut best: Option<(usize, Ty)> = None;
    for kw in ["let", "const", "for"] {
        let pat = format!("{kw} {name}");
        let mut from = 0;
        while let Some(p) = text[from..].find(&pat) {
            let at = from + p;
            from = at + pat.len();
            let before_ok = at == 0 || !text[..at].chars().next_back().is_some_and(is_ident);
            let rest = &text[at + pat.len()..];
            if !before_ok || rest.starts_with(is_ident) {
                continue;
            }
            let rest = rest.trim_start();
            let expr = if kw == "for" { rest.strip_prefix("in") } else { rest.strip_prefix('=') };
            let Some(expr) = expr else { continue };
            let e: Vec<char> = expr.trim_start().chars().take_while(|c| *c != ';' && *c != '{' && *c != '\n').collect();
            let e_str: String = e.iter().collect();
            let e_trim = e_str.trim_end();
            let ty = if e_trim.starts_with('[') {
                Array
            } else if e_trim.starts_with('"') || e_trim.starts_with('`') {
                Str
            } else if e_trim.starts_with(|c: char| c.is_ascii_digit() || c == '-') && !e_trim.contains("..") {
                Num
            } else {
                let ec: Vec<char> = e_trim.chars().collect();
                expr_ty(&ec, ec.len(), depth + 1).unwrap_or(Any)
            };
            let ty = if kw == "for" { if e_trim.contains("..") { Num } else { ty.elem() } } else { ty };
            if best.is_none_or(|(b, _)| at > b) {
                best = Some((at, ty));
            }
        }
    }
    best.map_or(Any, |(_, t)| t)
}

impl Ty {
    pub fn label(self) -> String {
        varstotas_i18n::tr(match self {
            Face => "script-type-face",
            Edge => "script-type-edge",
            Group => "script-type-group",
            Vector => "script-type-vector",
            Num => "script-type-num",
            Bool => "script-type-bool",
            Str => "script-type-str",
            Bounds => "script-type-bounds",
            Faces => "script-type-faces",
            Edges => "script-type-edges",
            Groups => "script-type-groups",
            Vecs => "script-type-vecs",
            Array => "script-type-array",
            Any => "script-type-any",
        })
    }
}

/// Header tags of plugins and console scripts (`// @tag …`, see `docs/PLUGINS.md`): name,
/// syntax, what it does.
pub const TAGS: &[(&str, &str, &str)] = &[
    ("name", "@name Title", "script-tag-name"), // i18n-ignore
    ("menu", "@menu Folder / Item", "script-tag-menu"), // i18n-ignore
    ("description", "@description Text", "script-tag-description"), // i18n-ignore
    ("param", "@param key[:type] = default \"Label\"", "script-tag-param"), // i18n-ignore
    ("pick", "@pick N \"Prompt\"", "script-tag-pick"), // i18n-ignore
    ("select", "@select \"Prompt\"", "script-tag-select"), // i18n-ignore
    ("import", "@import ext, ext", "script-tag-import"), // i18n-ignore
    ("export", "@export ext", "script-tag-export"), // i18n-ignore
    ("generator", "@generator Name", "script-tag-generator"), // i18n-ignore
    ("formula", "@formula key[:type] = expression \"Label\"", "script-tag-formula"), // i18n-ignore
    ("check", "@check expression \"Message\"", "script-tag-check"), // i18n-ignore
];

/// Types of `@param key:type`.
pub const PARAM_TYPES: &[(&str, &str)] = &[
    ("length", "script-param-type-length"),
    ("int", "script-param-type-int"),
    ("number", "script-param-type-number"),
    ("text", "script-param-type-text"),
    ("bool", "script-param-type-bool"),
    ("choice", "script-param-type-choice"),
    ("angle", "script-param-type-angle"),
    ("percent", "script-param-type-percent"),
    ("material", "script-param-type-material"),
];

/// `@param` types that only generators understand.
pub fn is_generator_type(name: &str) -> bool {
    matches!(name, "choice" | "angle" | "percent" | "material")
}

/// The description of a `@param` type (`key` from [`PARAM_TYPES`]); `plain` drops the
/// "generator: " lead of a generator-only type.
pub fn param_type_text(key: &str, plain: bool) -> String {
    let plain_key = format!("{key}-plain");
    if plain && varstotas_i18n::has(&plain_key) {
        varstotas_i18n::tr(&plain_key)
    } else {
        varstotas_i18n::tr(key)
    }
}

/// Completion in a header comment line.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TagCtx {
    /// `// @pa|`: the tag name.
    Tag,
    /// `// @param w:in|`: the parameter type.
    ParamType,
}

/// The header-tag context at `cursor` (char index): what is completed, the typed prefix and
/// its start.
pub fn tag_context(src: &str, cursor: usize) -> Option<(TagCtx, String, usize)> {
    let chars: std::vec::Vec<char> = src.chars().take(cursor).collect();
    let line_start = chars.iter().rposition(|c| *c == '\n').map_or(0, |p| p + 1);
    let line: String = chars[line_start..].iter().collect();
    let body = line.trim_start().strip_prefix("//")?.trim_start();
    let after = body.strip_prefix('@')?;
    let word_len = after.find(|c: char| !(c.is_alphanumeric() || c == '_')).unwrap_or(after.len());
    if word_len == after.len() {
        return Some((TagCtx::Tag, after.to_string(), cursor - after.chars().count()));
    }
    let rest = after.strip_prefix("param")?;
    let rest = rest.strip_prefix(char::is_whitespace)?.trim_start();
    let (key, ty) = rest.split_once(':')?;
    let key = key.trim_end();
    if key.is_empty() || !key.chars().all(|c| c.is_alphanumeric() || c == '_') {
        return None;
    }
    let ty = ty.trim_start();
    if !ty.chars().all(|c| c.is_alphanumeric()) {
        return None;
    }
    Some((TagCtx::ParamType, ty.to_string(), cursor - ty.chars().count()))
}

/// Variables a script gets from its header (the comment lines at the top): `@param` keys,
/// `points` (`@pick`), `objects` (`@select`), `path` (`@import` / `@export`), with types.
pub fn header_vars(src: &str) -> std::vec::Vec<(String, Ty)> {
    let mut out = std::vec::Vec::new();
    for line in src.lines() {
        let t = line.trim();
        if t.is_empty() {
            continue;
        }
        let Some(c) = t.strip_prefix("//") else { break };
        let Some(tag) = c.trim().strip_prefix('@') else { continue };
        let (key, rest) = tag.split_once(char::is_whitespace).unwrap_or((tag, ""));
        match key {
            "param" => {
                let Some((decl, default)) = rest.split_once('=') else { continue };
                let (name, ty) = decl.split_once(':').map_or((decl, None), |(n, t)| (n, Some(t.trim())));
                let name = name.trim();
                if name.is_empty() || !name.chars().all(|c| c.is_alphanumeric() || c == '_') {
                    continue;
                }
                let d = default.trim();
                let ty = match ty {
                    Some("text") => Str,
                    Some("bool") => Bool,
                    Some(_) => Num,
                    None if d.starts_with("true") || d.starts_with("false") => Bool,
                    None if d.starts_with('"') => Str,
                    None => Num,
                };
                out.push((name.to_string(), ty));
            }
            "pick" => out.push(("points".into(), Vecs)),
            "select" => out.push(("objects".into(), Array)),
            "import" | "export" if !out.iter().any(|(n, _)| n == "path") => out.push(("path".into(), Str)),
            _ => {}
        }
    }
    out
}

/// The tag of the header line around `cursor`, for the syntax hint.
pub fn tag_at(src: &str, cursor: usize) -> Option<&'static (&'static str, &'static str, &'static str)> {
    let chars: std::vec::Vec<char> = src.chars().collect();
    let cursor = cursor.min(chars.len());
    let start = chars[..cursor].iter().rposition(|c| *c == '\n').map_or(0, |p| p + 1);
    let end = chars[cursor..].iter().position(|c| *c == '\n').map_or(chars.len(), |p| cursor + p);
    let line: String = chars[start..end].iter().collect();
    let after = line.trim_start().strip_prefix("//")?.trim_start().strip_prefix('@')?;
    let word = after.split(|c: char| !(c.is_alphanumeric() || c == '_')).next()?;
    TAGS.iter().find(|t| t.0 == word)
}

pub const KEYWORDS: &[&str] = &[
    "let", "const", "fn", "if", "else", "for", "in", "while", "loop", "break", "continue", "return", "true", "false", "switch",
    "do", "until", "throw", "try", "catch",
];

pub const CONSTANTS: &[&str] = &["ORIGIN", "X", "Y", "Z"];

/// The documentation of a function name.
pub fn fn_doc(name: &str) -> Option<&'static FnDoc> {
    FNS.iter().find(|d| d.name == name)
}

/// Argument names of a signature (`"box(origin, sx, sy, sz)"` → `["origin", "sx", "sy", "sz"]`),
/// split at top-level commas.
pub fn sig_args(sig: &str) -> Vec<&str> {
    let Some(open) = sig.find('(') else { return Vec::new() };
    let inner = &sig[open + 1..sig.rfind(')').unwrap_or(sig.len())];
    let mut out = Vec::new();
    let (mut depth, mut start) = (0i32, 0usize);
    for (i, c) in inner.char_indices() {
        match c {
            '(' | '[' | '{' => depth += 1,
            ')' | ']' | '}' => depth -= 1,
            ',' if depth == 0 => {
                out.push(inner[start..i].trim());
                start = i + 1;
            }
            _ => {}
        }
    }
    if !inner.trim().is_empty() {
        out.push(inner[start..].trim());
    }
    out
}

/// The call the cursor is in: the function name before the innermost unclosed `(` and the
/// index of the argument at `cursor` (a char index into `src`). `member` is true for a method
/// call (`a.dot(`). Strings and comments are skipped.
pub fn call_context(src: &str, cursor: usize) -> Option<(String, usize, bool)> {
    let chars: Vec<char> = src.chars().take(cursor).collect();
    // Open brackets: (name before a '(' with its member flag, argument index).
    let mut stack: Vec<(Option<(String, bool)>, usize)> = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        let next = chars.get(i + 1).copied();
        match c {
            '/' if next == Some('/') => {
                while i < chars.len() && chars[i] != '\n' {
                    i += 1;
                }
            }
            '/' if next == Some('*') => {
                i += 2;
                while i < chars.len() && !(chars[i] == '*' && chars.get(i + 1) == Some(&'/')) {
                    i += 1;
                }
                i += 1;
            }
            '"' | '`' | '\'' => {
                i += 1;
                while i < chars.len() && chars[i] != c {
                    if chars[i] == '\\' {
                        i += 1;
                    }
                    i += 1;
                }
            }
            '(' => {
                let mut j = i;
                while j > 0 && chars[j - 1].is_whitespace() {
                    j -= 1;
                }
                let end = j;
                while j > 0 && (chars[j - 1].is_alphanumeric() || chars[j - 1] == '_') {
                    j -= 1;
                }
                let name: String = chars[j..end].iter().collect();
                let member = j > 0 && chars[j - 1] == '.';
                let named = !name.is_empty() && !name.starts_with(|c: char| c.is_ascii_digit()) && !KEYWORDS.contains(&name.as_str());
                stack.push((named.then_some((name, member)), 0));
            }
            '[' | '{' => stack.push((None, 0)),
            ')' | ']' | '}' => {
                stack.pop();
            }
            ',' => {
                if let Some(top) = stack.last_mut() {
                    top.1 += 1;
                }
            }
            _ => {}
        }
        i += 1;
    }
    let (name, arg) = stack.pop()?;
    let (name, member) = name?;
    Some((name, arg, member))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_function_is_in_one_group() {
        for d in FNS {
            let n = GROUPS.iter().filter(|(_, names)| names.contains(&d.name)).count();
            assert_eq!(n, 1, "{} is in {n} groups", d.name);
        }
        for (_, names) in GROUPS {
            for n in *names {
                assert!(fn_doc(n).is_some(), "{n} is not documented");
            }
        }
    }

    #[test]
    fn every_api_function_is_documented() {
        for n in crate::api_names() {
            let d = fn_doc(n).unwrap_or_else(|| panic!("no help for {n}"));
            assert!(d.sigs.iter().all(|s| s.starts_with(&format!("{n}("))), "{n}");
        }
        assert_eq!(sig_args("box(origin, sx, sy, sz)"), ["origin", "sx", "sy", "sz"]);
        assert_eq!(sig_args("set_params(g, #{ x, y })"), ["g", "#{ x, y }"]);
        assert!(sig_args("exit()").is_empty());
    }

    #[test]
    fn member_completion_by_type() {
        let at = |s: &str| member_context(s, s.chars().count()).map(|(t, p, _)| (t, p));
        assert_eq!(at("let g = box(ORIGIN, 1, 2, 3);\ng.na"), Some((Ty::Group, "na".into())));
        assert_eq!(at("let f = rect(ORIGIN, 1, 2);\nf."), Some((Ty::Face, String::new())));
        assert_eq!(at("let f = rect(ORIGIN, 1, 2);\nf.normal."), Some((Ty::Vector, String::new())));
        assert_eq!(at("for e in edges() { e."), Some((Ty::Edge, String::new())));
        assert_eq!(at("for i in 0..3 { i."), Some((Ty::Num, String::new())));
        assert_eq!(at("faces()[0].ar"), Some((Ty::Face, "ar".into())));
        assert_eq!(at("box(ORIGIN, 1, 2, 3)."), Some((Ty::Group, String::new())));
        assert_eq!(at("bounds(g).size.x"), Some((Ty::Vector, "x".into())));
        assert_eq!(at("points[0]."), Some((Ty::Vector, String::new())));
        assert_eq!(at("let p = ORIGIN; p."), Some((Ty::Vector, String::new())));
        assert_eq!(at("unknown."), Some((Ty::Any, String::new())));
        assert_eq!(at("let x = 1.5"), None);
        assert_eq!(at("// g."), None);
        assert_eq!(at("print(\"a.b"), None);
        let names: std::vec::Vec<_> = members_for(Ty::Group).iter().map(|m| m.name).collect();
        assert!(names.contains(&"name") && !names.contains(&"area"));
    }

    #[test]
    fn header_variables() {
        let src = "// @name X\n// @param w = 100 \"W\"\n// @param n:int = 3\n// @param label = \"a\"\n// @param on = true\n// @pick 2\n// @select\nlet a = 1; // @param late = 1\n";
        let v = header_vars(src);
        let names: std::vec::Vec<_> = v.iter().map(|(n, t)| (n.as_str(), *t)).collect();
        assert_eq!(names, [("w", Ty::Num), ("n", Ty::Num), ("label", Ty::Str), ("on", Ty::Bool), ("points", Ty::Vecs), ("objects", Ty::Array)]);
        let s = format!("{src}label.");
        assert_eq!(member_context(&s, s.chars().count()).map(|c| c.0), Some(Ty::Str));
    }

    #[test]
    fn header_tag_completion() {
        let at = |s: &str| tag_context(s, s.chars().count()).map(|(t, p, _)| (t, p));
        assert_eq!(at("// @pa"), Some((TagCtx::Tag, "pa".into())));
        assert_eq!(at("let a = 1;\n  //@"), Some((TagCtx::Tag, String::new())));
        assert_eq!(at("// @param w:in"), Some((TagCtx::ParamType, "in".into())));
        assert_eq!(at("// @param w : "), Some((TagCtx::ParamType, String::new())));
        assert_eq!(at("// @param w = 1"), None);
        assert_eq!(at("// @pick 2"), None);
        assert_eq!(at("let x = a@"), None);
        assert_eq!(tag_at("// @pick 2 \"x\"", 3).map(|t| t.0), Some("pick"));
        assert_eq!(tag_at("// plain", 3), None);
    }

    #[test]
    fn call_context_at_the_cursor() {
        let at = |s: &str| call_context(s, s.chars().count());
        assert_eq!(at("box(ORIGIN, 10, "), Some(("box".into(), 2, false)));
        assert_eq!(at("box(vec3(1, 2"), Some(("vec3".into(), 1, false)));
        assert_eq!(at("box(vec3(1, 2), [1, 2, 3], "), Some(("box".into(), 2, false)));
        assert_eq!(at("rotate(x, \"a, (b\", "), Some(("rotate".into(), 2, false)));
        assert_eq!(at("// f(\nline(a"), Some(("line".into(), 0, false)));
        assert_eq!(at("a.dot(b"), Some(("dot".into(), 0, true)));
        assert_eq!(at("box(1, 2, 3, 4)"), None);
        assert_eq!(at("if (a"), None);
        assert_eq!(at("let p = [1, 2"), None);
    }
}
