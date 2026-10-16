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
    f("set_params", &["set_params(g, #{ x, y, z, count, axis, gap, stretch })"], "Set parametric size / copies."),
    f("params", &["params(g)"], "Parameters of a parametric group, or ()."),
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

/// Methods and properties after a dot (entities, vectors, arrays).
pub const MEMBERS: &[(&str, &str)] = &[
    ("kind", "entity kind: face, edge, vertex, group, component, parametric"),
    ("valid", "the entity still exists"),
    ("area", "face area (mm²)"),
    ("length", "edge length; v.length() for vectors"),
    ("normal", "face normal"),
    ("center", "centre point"),
    ("vertices", "points of a face / edge"),
    ("edges", "edges of a face"),
    ("faces", "faces of an edge / neighbours of a face"),
    ("name", "group / component name (assignable)"),
    ("definition", "component definition name"),
    ("material", "material name"),
    ("tag", "tag name"),
    ("soft", "edge is soft"),
    ("x", "vector component"),
    ("y", "vector component"),
    ("z", "vector component"),
    ("normalize", "v.normalize()"),
    ("dot", "a.dot(b)"),
    ("cross", "a.cross(b)"),
    ("distance", "a.distance(b)"),
    ("push", "array.push(value)"),
    ("len", "array.len()"),
];

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
