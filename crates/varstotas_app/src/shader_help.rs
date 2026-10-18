//! Help for the shader editor (WGSL custom surfaces, docs/SHADERS.md): keywords and types for
//! the highlighting, the fields of `Surface` after `s.` / `o.`, the `@param`s after `p.`,
//! swizzles after a vector field, and signatures of the helpers and WGSL built-ins for the
//! call hint.

pub const KEYWORDS: &[&str] = &[
    "fn", "let", "var", "const", "return", "if", "else", "for", "while", "loop", "break", "continue", "switch", "case",
    "default", "struct", "true", "false", "discard", "continuing",
];

pub const TYPES: &[&str] = &[
    "f32", "i32", "u32", "bool", "vec2", "vec3", "vec4", "vec2f", "vec3f", "vec4f", "vec2i", "vec3i", "vec2u", "mat2x2", "mat3x3",
    "mat4x4", "array", "Surface", "Params",
];

/// (name, signature, description): the helpers every shader gets and the common WGSL built-ins.
pub const FNS: &[(&str, &str, &str)] = &[
    ("surface", "surface(s: Surface, p: Params) -> Surface", "the shader: return the changed surface"),
    ("hash3", "hash3(p: vec3<f32>) -> f32", "pseudo-random value 0..1 per point"),
    ("noise", "noise(p: vec3<f32>) -> f32", "smooth value noise 0..1"),
    ("fbm", "fbm(p: vec3<f32>, octaves: i32) -> f32", "layered noise 0..1 (clouds, marble, warping)"),
    ("srgb", "srgb(c: vec3<f32>) -> vec3<f32>", "a colour picked in sRGB (0..1) → linear"),
    ("hsv", "hsv(h: f32, sat: f32, v: f32) -> vec3<f32>", "hue / saturation / value (0..1) → linear colour"),
    ("tangent_of", "tangent_of(n: vec3<f32>) -> vec3<f32>", "a unit vector across n"),
    ("bump", "bump(n: vec3<f32>, dh: vec2<f32>) -> vec3<f32>", "n tilted by the slope of a height field"),
    ("abs", "abs(x)", "absolute value"),
    ("min", "min(a, b)", "the smaller"),
    ("max", "max(a, b)", "the larger"),
    ("clamp", "clamp(x, lo, hi)", "x limited to lo..hi"),
    ("saturate", "saturate(x)", "x limited to 0..1"),
    ("mix", "mix(a, b, t)", "a + (b − a)·t"),
    ("step", "step(edge, x)", "0 below edge, 1 from it"),
    ("smoothstep", "smoothstep(lo, hi, x)", "smooth 0→1 between lo and hi"),
    ("fract", "fract(x)", "x − floor(x)"),
    ("floor", "floor(x)", "round down"),
    ("ceil", "ceil(x)", "round up"),
    ("round", "round(x)", "nearest whole number"),
    ("sign", "sign(x)", "−1, 0 or 1"),
    ("sqrt", "sqrt(x)", "square root"),
    ("pow", "pow(x, y)", "x to the power y"),
    ("exp", "exp(x)", "e to the power x"),
    ("log", "log(x)", "natural logarithm"),
    ("sin", "sin(x)", "sine (radians)"),
    ("cos", "cos(x)", "cosine (radians)"),
    ("tan", "tan(x)", "tangent (radians)"),
    ("atan2", "atan2(y, x)", "angle of (x, y), radians"),
    ("length", "length(v)", "vector length"),
    ("distance", "distance(a, b)", "length(a − b)"),
    ("normalize", "normalize(v)", "unit vector along v"),
    ("dot", "dot(a, b)", "dot product"),
    ("cross", "cross(a, b)", "cross product of two vec3"),
    ("reflect", "reflect(i, n)", "i mirrored about the normal n"),
    ("select", "select(f, t, cond)", "t when cond, else f"),
    ("vec2", "vec2<f32>(x, y)", "a 2-vector"),
    ("vec3", "vec3<f32>(x, y, z)", "a 3-vector"),
    ("vec4", "vec4<f32>(x, y, z, w)", "a 4-vector"),
];

/// The fields of `Surface`: (name, type, description).
pub const SURFACE: &[(&str, &str, &str)] = &[
    ("base", "vec3<f32>", "linear base colour"),
    ("alpha", "f32", "opacity"),
    ("metallic", "f32", "0 = dielectric, 1 = metal"),
    ("roughness", "f32", "0 = mirror, 1 = matte"),
    ("transmission", "f32", "glass: 1 = light passes through"),
    ("emission", "vec3<f32>", "light given off, linear, strength included"),
    ("normal", "vec3<f32>", "world normal, unit, facing the viewer"),
    ("pos", "vec3<f32>", "model position, mm (z = up)"),
    ("uv", "vec2<f32>", "planar texture coordinates, mm"),
    ("view", "vec3<f32>", "unit direction toward the eye"),
    ("time", "f32", "seconds"),
];

const SWIZZLE2: &[&str] = &["x", "y", "xy", "yx"];
const SWIZZLE3: &[&str] = &["x", "y", "z", "xy", "xz", "yz", "xyz", "zyx", "r", "g", "b", "rgb"];

/// The `@param` tag: (tag, syntax, description).
pub const PARAM_TAG: (&str, &str, &str) = ("param", "// @param name = default (min..max) \"Label\"", "a slider of the material editor, read as p.name (at most 8)");

/// A row of the completion list: (name, label, text inserted after the typed prefix, detail).
pub type Row = (String, String, String, String);

fn is_ident(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

/// Names of `surface`'s two parameters (`s`, `p` by default).
fn surface_args(src: &str) -> (String, String) {
    let (mut s, mut p) = ("s".to_string(), "p".to_string());
    if let Some(i) = src.find("fn surface(") {
        let rest = &src[i + "fn surface(".len()..];
        let args: Vec<&str> = rest.split(')').next().unwrap_or("").split(',').collect();
        let name = |a: &str| a.split(':').next().unwrap_or("").trim().to_string();
        if let Some(a) = args.first().map(|a| name(a)).filter(|n| !n.is_empty()) {
            s = a;
        }
        if let Some(a) = args.get(1).map(|a| name(a)).filter(|n| !n.is_empty()) {
            p = a;
        }
    }
    (s, p)
}

/// Variables holding a `Surface`: the first argument, `var o = s;`, `x: Surface`.
fn surface_vars(src: &str) -> Vec<String> {
    let (s, _) = surface_args(src);
    let mut vars = vec![s];
    for line in src.lines() {
        let t = line.trim();
        for kw in ["var ", "let "] {
            if let Some(rest) = t.strip_prefix(kw)
                && let Some((name, val)) = rest.split_once('=')
            {
                let name = name.split(':').next().unwrap_or("").trim();
                let val = val.trim().trim_end_matches(';').trim();
                if vars.iter().any(|v| v == val) || rest.split_once('=').is_some_and(|(n, _)| n.contains("Surface")) {
                    vars.push(name.to_string());
                }
            }
        }
    }
    vars
}

/// The cursor (char index) is in a `//` comment.
fn in_comment(chars: &[char]) -> bool {
    let start = chars.iter().rposition(|c| *c == '\n').map_or(0, |p| p + 1);
    chars[start..].windows(2).any(|w| w == ['/', '/'])
}

/// Completion rows at `cursor`: `// @` → param; after `p.` the parameters; after a Surface
/// variable the fields; after a vector field its swizzles. With the typed prefix and its start.
pub fn items_at(src: &str, cursor: usize) -> Option<(Vec<Row>, String, usize)> {
    let chars: Vec<char> = src.chars().take(cursor).collect();
    let mut i = chars.len();
    while i > 0 && is_ident(chars[i - 1]) {
        i -= 1;
    }
    let prefix: String = chars[i..].iter().collect();
    let rest = |name: &str| name.get(prefix.len()..).unwrap_or_default().to_string();
    // `// @pa|`
    let line_start = chars.iter().rposition(|c| *c == '\n').map_or(0, |p| p + 1);
    let line: String = chars[line_start..].iter().collect();
    if let Some(after) = line.trim_start().strip_prefix("//").map(str::trim_start).and_then(|b| b.strip_prefix('@'))
        && after.chars().all(is_ident)
    {
        let rows = if PARAM_TAG.0.starts_with(after) {
            vec![(PARAM_TAG.0.to_string(), format!("@{}", PARAM_TAG.0), format!("{} ", rest(PARAM_TAG.0)), format!("{}  — {}", PARAM_TAG.1, PARAM_TAG.2))]
        } else {
            Vec::new()
        };
        return Some((rows, prefix, i));
    }
    if i == 0 || chars[i - 1] != '.' || in_comment(&chars[..i]) {
        return None;
    }
    // The expression before the dot: `name` or `name.field`.
    let mut j = i - 1;
    let end = j;
    while j > 0 && is_ident(chars[j - 1]) {
        j -= 1;
    }
    let word: String = chars[j..end].iter().collect();
    let owner: Option<String> = (j > 1 && chars[j - 1] == '.').then(|| {
        let e = j - 1;
        let mut k = e;
        while k > 0 && is_ident(chars[k - 1]) {
            k -= 1;
        }
        chars[k..e].iter().collect()
    });
    let full: String = src.to_string();
    let (_, pname) = surface_args(&full);
    let svars = surface_vars(&full);
    let rows: Vec<Row> = match owner {
        // `s.pos.` → swizzles
        Some(o) if svars.contains(&o) => {
            let ty = SURFACE.iter().find(|f| f.0 == word).map(|f| f.1)?;
            let sw = if ty.starts_with("vec2") { SWIZZLE2 } else if ty.starts_with("vec3") { SWIZZLE3 } else { return None };
            sw.iter().filter(|s| s.starts_with(prefix.as_str())).map(|s| (s.to_string(), s.to_string(), rest(s), format!("of {word}: {ty}"))).collect()
        }
        Some(_) => return None,
        None if word == pname => varstotas_kernel::shader_params(&full)
            .into_iter()
            .filter(|q| q.name.starts_with(prefix.as_str()))
            .map(|q| {
                let detail = format!(": f32  — {} ({}..{}, default {})", q.label, q.min, q.max, q.default);
                (q.name.clone(), q.name.clone(), rest(&q.name), detail)
            })
            .collect(),
        None if svars.contains(&word) => SURFACE
            .iter()
            .filter(|f| f.0.starts_with(prefix.as_str()))
            .map(|f| (f.0.to_string(), f.0.to_string(), rest(f.0), format!(": {}  — {}", f.1, f.2)))
            .collect(),
        None => return None,
    };
    Some((rows, prefix, i))
}

/// The signature and description of function `name`.
pub fn fn_doc(name: &str) -> Option<(&'static str, &'static str)> {
    FNS.iter().find(|f| f.0 == name).map(|f| (f.1, f.2))
}

/// The cursor (char index) is on a `// @param` line.
pub fn on_param_line(src: &str, cursor: usize) -> bool {
    let chars: Vec<char> = src.chars().collect();
    let cursor = cursor.min(chars.len());
    let start = chars[..cursor].iter().rposition(|c| *c == '\n').map_or(0, |p| p + 1);
    let end = chars[cursor..].iter().position(|c| *c == '\n').map_or(chars.len(), |p| cursor + p);
    let line: String = chars[start..end].iter().collect();
    line.trim_start().strip_prefix("//").is_some_and(|b| b.trim_start().starts_with("@param"))
}

#[cfg(test)]
mod tests {
    use super::*;

    const SRC: &str = "// @param size = 200 (10..2000) \"Square size (mm)\"\n// @param contrast = 0.5\nfn surface(s: Surface, p: Params) -> Surface {\n    var o = s;\n    ";

    fn names(src: &str) -> Vec<String> {
        items_at(src, src.chars().count()).map(|(r, _, _)| r.into_iter().map(|r| r.0).collect()).unwrap_or_default()
    }

    #[test]
    fn completion_knows_surfaces_params_and_swizzles() {
        assert!(names(&format!("{SRC}o.ro")) == ["roughness"]);
        assert_eq!(names(&format!("{SRC}s.")).len(), SURFACE.len());
        assert_eq!(names(&format!("{SRC}p.")), ["size", "contrast"]);
        assert_eq!(names(&format!("{SRC}let c = p.co")), ["contrast"]);
        assert!(names(&format!("{SRC}s.pos.x")).contains(&"xyz".to_string()));
        assert!(names(&format!("{SRC}s.uv.")).contains(&"xy".to_string()));
        assert!(names(&format!("{SRC}q.")).is_empty());
        assert_eq!(names("// @pa"), ["param"]);
        // in a comment: nothing
        assert!(names(&format!("{SRC}// s.")).is_empty());
        assert_eq!(fn_doc("fbm").map(|d| d.0), Some("fbm(p: vec3<f32>, octaves: i32) -> f32"));
        assert!(on_param_line(SRC, 5));
        assert!(!on_param_line(SRC, SRC.len()));
    }
}
