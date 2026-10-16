//! Signatures and one-line descriptions of the script API, for the editor's completion and
//! parameter hints (the full reference is `docs/SCRIPTING.md`).

/// One API function: its overloads (`name(arg, arg)`) and what it does.
#[derive(Clone, Copy, Debug)]
pub struct FnDoc {
    pub name: &'static str,
    pub sigs: &'static [&'static str],
    pub doc: &'static str,
}

const fn f(name: &'static str, sigs: &'static [&'static str], doc: &'static str) -> FnDoc {
    FnDoc { name, sigs, doc }
}

/// Every API function (the names of [`crate::api_names`]) plus the Rhai built-ins scripts use most.
pub const FNS: &[FnDoc] = &[
    f("vec3", &["vec3(x, y, z)", "vec3(x, y)"], "A point / vector (an array [x, y, z] works too)."),
    f("line", &["line(a, b)"], "Edges from a to b."),
    f("polyline", &["polyline(points)", "polyline(points, closed)"], "Edges through points; closed and planar → a face."),
    f("face", &["face(points)"], "A face through the points."),
    f("rect", &["rect(origin, w, d)"], "A rectangular face in the XY plane."),
    f(
        "circle",
        &["circle(center, r)", "circle(center, r, segments)", "circle(center, r, segments, normal)"],
        "A circle face (or edges).",
    ),
    f("polygon", &["polygon(center, r, sides)", "polygon(center, r, sides, normal)"], "A regular polygon face."),
    f(
        "arc",
        &["arc(center, start_point, degrees)", "arc(center, start_point, degrees, segments)", "arc(center, start_point, degrees, segments, normal)"],
        "Arc edges around center.",
    ),
    f("box", &["box(origin, sx, sy, sz)"], "A new box group (origin = min corner)."),
    f("cylinder", &["cylinder(center, r, h)", "cylinder(center, r, h, segments)"], "A new cylinder group (bottom at center)."),
    f("pushpull", &["pushpull(face, distance)"], "Push / pull a face; returns the cap face."),
    f("offset", &["offset(face, distance)"], "Offset a face's loop (d < 0 inward); returns the new face."),
    f("follow_me", &["follow_me(face, edges)"], "Sweep a profile face along a path."),
    f("erase", &["erase(x)"], "Erase an entity or an array of them."),
    f("move", &["move(x, vector)"], "Move entities by a vector."),
    f("copy", &["copy(x, vector)"], "Copy entities by a vector; returns the copies."),
    f("rotate", &["rotate(x, center, axis, degrees)"], "Rotate entities about an axis."),
    f("scale", &["scale(x, center, k)", "scale(x, center, vec3(kx, ky, kz))"], "Scale entities about a point."),
    f("group", &["group(x)"], "Group entities; returns the group."),
    f("component", &["component(x, name)"], "Make a component; returns its instance."),
    f("explode", &["explode(g)"], "Explode a group / component."),
    f("make_unique", &["make_unique(g)"], "Give a component instance its own definition."),
    f("union", &["union(a, b)"], "Solid union of two solid groups."),
    f("subtract", &["subtract(a, b)"], "Solid a minus b."),
    f("intersect", &["intersect(a, b)"], "Solid intersection."),
    f("trim", &["trim(a, b)"], "a minus b, b stays."),
    f("split", &["split(a, b)"], "Split two solids into three."),
    f("outer_shell", &["outer_shell(groups)"], "Outer shell of solids."),
    f("reverse", &["reverse(faces)"], "Reverse faces."),
    f("soften", &["soften(x, degrees)"], "Soften edges up to the fold angle."),
    f("material", &["material(name, \"#RRGGBB\")", "material(name, [r, g, b])"], "Create / recolour a material; returns its name."),
    f("paint", &["paint(x, material)", "paint(x, ())"], "Paint faces / groups (() removes)."),
    f("tag", &["tag(x, name)"], "Assign a tag (created when missing)."),
    f("make_parametric", &["make_parametric(g)"], "Make a group parametric."),
    f(
        "set_params",
        &["set_params(g, #{ key: value, ... })", "set_params(g, #{ x, y, z, count, axis, gap, stretch })"],
        "Set a generated component's values, or a parametric group's size / copies.",
    ),
    f("params", &["params(g)"], "Values of a generated component / parameters of a parametric group, or ()."),
    f("reset_params", &["reset_params(g)"], "A generated component back to its generator's defaults."),
    f("generators", &["generators()"], "Names of the document's generators."),
    f("add_generator", &["add_generator(source)"], "Add / update a generator from its source; returns its name."),
    f("generate", &["generate(name, at)", "generate(name, at, #{ key: value, ... })"], "Place a generated component at a point."),
    f("dimension", &["dimension(a, b, offset)"], "A linear dimension between points."),
    f("note", &["note(at, offset, text)"], "A text note with a leader."),
    f("enter", &["enter(g)"], "Build inside a group from now on."),
    f("exit", &["exit()"], "Leave the group entered last."),
    f("faces", &["faces()"], "Faces of the current context."),
    f("edges", &["edges()"], "Edges of the current context."),
    f("groups", &["groups()"], "Groups / components of the current context."),
    f("all", &["all()"], "Everything in the current context."),
    f("entities", &["entities(g)"], "Everything inside a group."),
    f("selection", &["selection()"], "The selection."),
    f("select", &["select(x)", "select(())"], "Set the selection (() clears)."),
    f("bounds", &["bounds(x)"], "#{ min, max, size, center } in world coordinates."),
    f("model_bounds", &["model_bounds()"], "Bounds of the whole model."),
    f("length", &["length(text)"], "Parse a length with units (\"1.2m\") → mm."),
    f("units", &["units()"], "The model's length unit."),
    f("materials", &["materials()"], "Material names."),
    f("all_faces", &["all_faces()"], "Every face, nested groups included."),
    f("read_text", &["read_text(path)"], "Read a text file."),
    f("write_text", &["write_text(path, text)"], "Write a text file."),
    f("print", &["print(value)"], "Write to the console log."),
    f("len", &["len(array)"], "Length of an array / string."),
    f("range", &["range(from, to)", "range(from, to, step)"], "Integers from..to (to excluded), for loops."),
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
    prop("kind", "\"face\", \"edge\", \"group\", \"component\" …", ENT, Str),
    prop("valid", "the entity still exists", ENT, Bool),
    prop("area", "face area, mm²", &[Face], Num),
    prop("normal", "face normal (world)", &[Face], Vector),
    prop("edges", "edges of the face", &[Face], Edges),
    prop("faces", "faces of the edge / neighbours of the face", &[Face, Edge], Faces),
    prop("length", "edge length, mm", &[Edge], Num),
    prop("soft", "the edge is soft", &[Edge], Bool),
    prop("center", "centre of the bounds (world)", &[Face, Edge, Group], Vector),
    prop("vertices", "points of the face / edge (world)", &[Face, Edge], Vecs),
    prop("name", "group / component name (assignable)", &[Group], Str),
    prop("definition", "component definition name", &[Group], Str),
    prop("material", "material name or ()", &[Face, Group], Str),
    prop("tag", "tag name or ()", ENT, Str),
    prop("x", "x coordinate (assignable)", &[Vector], Num),
    prop("y", "y coordinate (assignable)", &[Vector], Num),
    prop("z", "z coordinate (assignable)", &[Vector], Num),
    meth("length", "length()", "vector length", &[Vector], Num),
    meth("normalize", "normalize()", "unit vector", &[Vector], Vector),
    meth("dot", "dot(b)", "dot product", &[Vector], Num),
    meth("cross", "cross(b)", "cross product", &[Vector], Vector),
    meth("distance", "distance(b)", "distance to b", &[Vector], Num),
    prop("min", "minimum corner", &[Bounds], Vector),
    prop("max", "maximum corner", &[Bounds], Vector),
    prop("size", "size along x, y, z", &[Bounds], Vector),
    prop("center", "centre", &[Bounds], Vector),
    meth("len", "len()", "number of items", ARR, Num),
    meth("is_empty", "is_empty()", "no items", ARR, Bool),
    meth("push", "push(value)", "append an item", ARR, Any),
    meth("pop", "pop()", "remove and return the last item", ARR, Any),
    meth("insert", "insert(index, value)", "insert an item", ARR, Any),
    meth("remove", "remove(index)", "remove and return an item", ARR, Any),
    meth("contains", "contains(value)", "the array / string contains the value", &[Faces, Edges, Groups, Vecs, Array, Str], Bool),
    meth("index_of", "index_of(value)", "index of the value or -1", ARR, Num),
    meth("filter", "filter(|x| condition)", "items where the closure is true", ARR, Any),
    meth("map", "map(|x| value)", "a new array of the closure results", ARR, Array),
    meth("reduce", "reduce(|sum, x| sum + x, start)", "fold the items", ARR, Any),
    meth("len", "len()", "number of characters", &[Str], Num),
    meth("to_upper", "to_upper()", "upper case (in place)", &[Str], Any),
    meth("to_lower", "to_lower()", "lower case (in place)", &[Str], Any),
    meth("trim", "trim()", "strip spaces (in place)", &[Str], Any),
    meth("split", "split(separator)", "split into an array", &[Str], Array),
    meth("starts_with", "starts_with(text)", "the string starts with text", &[Str], Bool),
    meth("to_string", "to_string()", "as text", &[Face, Edge, Group, Vector, Num], Str),
];

/// What a function returns.
fn fn_ty(name: &str) -> Ty {
    match name {
        "face" | "rect" | "circle" | "polygon" | "pushpull" | "offset" => Face,
        "box" | "cylinder" | "group" | "component" | "union" | "subtract" | "intersect" | "trim" | "outer_shell" => Group,
        "line" | "arc" | "edges" => Edges,
        "faces" | "all_faces" => Faces,
        "groups" | "split" => Groups,
        "all" | "entities" | "selection" | "copy" | "materials" | "range" => Array,
        "vec3" => Vector,
        "bounds" | "model_bounds" => Bounds,
        "material" | "units" | "read_text" => Str,
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
    pub fn label(self) -> &'static str {
        match self {
            Face => "face",
            Edge => "edge",
            Group => "group",
            Vector => "vector",
            Num => "number",
            Bool => "bool",
            Str => "string",
            Bounds => "bounds",
            Faces => "array of faces",
            Edges => "array of edges",
            Groups => "array of groups",
            Vecs => "array of points",
            Array => "array",
            Any => "entity / vector",
        }
    }
}

/// Header tags of plugins and console scripts (`// @tag …`, see `docs/PLUGINS.md`): name,
/// syntax, what it does.
pub const TAGS: &[(&str, &str, &str)] = &[
    ("name", "@name Title", "name in the menu and the dialog title"),
    ("menu", "@menu Folder / Item", "path under Extensions"),
    ("description", "@description Text", "tooltip and text of the dialog"),
    ("param", "@param key[:type] = default \"Label\"", "a value asked in the dialog before the run"),
    ("pick", "@pick N \"Prompt\"", "N points clicked before the run → points"),
    ("select", "@select \"Prompt\"", "objects selected before the run → objects"),
    ("import", "@import ext, ext", "a File → Import entry; the file in path"),
    ("export", "@export ext", "a File → Export entry; the file in path"),
];

/// Types of `@param key:type`.
pub const PARAM_TYPES: &[(&str, &str)] = &[
    ("length", "length with units (\"1.2m\"), mm in the script — the default for numbers"),
    ("int", "whole number"),
    ("number", "fractional number"),
    ("text", "text — the default for a quoted value"),
    ("bool", "check box — the default for true / false"),
];

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
        assert_eq!(at("let f = rect(ORIGIN, 1, 2);\nf."), Some((Ty::Face, "".into())));
        assert_eq!(at("let f = rect(ORIGIN, 1, 2);\nf.normal."), Some((Ty::Vector, "".into())));
        assert_eq!(at("for e in edges() { e."), Some((Ty::Edge, "".into())));
        assert_eq!(at("for i in 0..3 { i."), Some((Ty::Num, "".into())));
        assert_eq!(at("faces()[0].ar"), Some((Ty::Face, "ar".into())));
        assert_eq!(at("box(ORIGIN, 1, 2, 3)."), Some((Ty::Group, "".into())));
        assert_eq!(at("bounds(g).size.x"), Some((Ty::Vector, "x".into())));
        assert_eq!(at("points[0]."), Some((Ty::Vector, "".into())));
        assert_eq!(at("let p = ORIGIN; p."), Some((Ty::Vector, "".into())));
        assert_eq!(at("unknown."), Some((Ty::Any, "".into())));
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
        assert_eq!(at("let a = 1;\n  //@"), Some((TagCtx::Tag, "".into())));
        assert_eq!(at("// @param w:in"), Some((TagCtx::ParamType, "in".into())));
        assert_eq!(at("// @param w : "), Some((TagCtx::ParamType, "".into())));
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
