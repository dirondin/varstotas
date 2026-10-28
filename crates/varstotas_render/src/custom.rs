//! Custom WGSL surfaces of materials (docs/SHADERS.md): each source defines
//! `fn surface(s: Surface, p: Params) -> Surface`; its top-level names get a per-shader suffix
//! so several shaders can live in one module, `Params` becomes a struct of its `@param`
//! values, and a generated `custom_surface` dispatches on the material's shader id.

use crate::wgpu::naga;

/// The scene shader (scene passes and the effects passes) without custom surfaces.
pub(crate) const BASE: &str = concat!(include_str!("shader.wgsl"), "\n", include_str!("post.wgsl"), "\n", include_str!("pathtrace.wgsl"));

/// A material's surface shader for `Renderer::set_custom_shaders`.
#[derive(Clone, Debug, PartialEq)]
pub struct CustomSurface {
    pub source: String,
    /// Names of its parameters in order (the fields of `Params`, at most 8).
    pub params: Vec<String>,
}

fn is_ident_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}

/// Names declared at the top of `src` by `fn`, `struct`, `const` or `alias` (also local
/// constants: renaming them too is harmless).
fn declared_names(src: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let code: String = src.lines().map(|l| l.split("//").next().unwrap_or("")).collect::<Vec<_>>().join("\n");
    let words: Vec<&str> = code.split(|c: char| !is_ident_char(c)).filter(|w| !w.is_empty()).collect();
    for w in words.windows(2) {
        if matches!(w[0], "fn" | "struct" | "const" | "alias") && !out.iter().any(|n| n == w[1]) {
            out.push(w[1].to_string());
        }
    }
    out
}

/// Replace whole-word `names` (and `Params`) in `src` with `<name>_c<i>`; comments stay.
fn rename(src: &str, names: &[String], i: usize) -> String {
    let mut out = String::with_capacity(src.len() + 64);
    let mut word = String::new();
    let flush = |word: &mut String, out: &mut String| {
        if !word.is_empty() {
            // i18n-ignore-next
            if word == "Params" || names.iter().any(|n| n == word.as_str()) {
                out.push_str(&format!("{word}_c{i}"));
            } else {
                out.push_str(word);
            }
            word.clear();
        }
    };
    for line in src.split_inclusive('\n') {
        let (code, comment) = match line.find("//") {
            Some(k) => line.split_at(k),
            None => (line, ""),
        };
        for c in code.chars() {
            if is_ident_char(c) {
                word.push(c);
            } else {
                flush(&mut word, &mut out);
                out.push(c);
            }
        }
        flush(&mut word, &mut out);
        out.push_str(comment);
    }
    out
}

/// The full module: `BASE`, the shaders (`Some` = included; the index + 1 is the material's
/// shader id), and the dispatch. Also returns the first line of each included shader.
pub(crate) fn compose(shaders: &[Option<&CustomSurface>]) -> (String, Vec<usize>) {
    let mut src = String::from(BASE);
    src.push_str("\n// ---- custom surfaces ----\n");
    let mut starts = vec![0; shaders.len()];
    let mut cases = String::new();
    for (i, sh) in shaders.iter().enumerate() {
        let Some(sh) = sh else { continue };
        let fields: Vec<String> = sh.params.iter().take(8).map(|n| format!("{n}: f32")).collect();
        let fields = if fields.is_empty() { "_unused: f32".to_string() } else { fields.join(", ") };
        src.push_str(&format!("struct Params_c{i} {{ {fields} }}\n"));
        starts[i] = src.lines().count() + 1;
        src.push_str(&rename(&sh.source, &declared_names(&sh.source), i));
        src.push('\n');
        let n = sh.params.len().clamp(1, 8);
        let args: Vec<&str> = ["a.x", "a.y", "a.z", "a.w", "b.x", "b.y", "b.z", "b.w"][..n].to_vec();
        cases.push_str(&format!("        case {}u: {{ return surface_c{i}(s, Params_c{i}({})); }}\n", i + 1, args.join(", ")));
    }
    src.push_str(&format!(
        "fn custom_surface(id: u32, s: Surface, a: vec4<f32>, b: vec4<f32>) -> Surface {{\n    switch id {{\n{cases}        default: {{ return s; }}\n    }}\n}}\n"
    ));
    (src, starts)
}

/// Parse and validate a module; the error names the line (1-based) of the composed source.
pub(crate) fn validate(src: &str) -> Result<(), (Option<usize>, String)> {
    let module = naga::front::wgsl::parse_str(src).map_err(|e| (e.location(src).map(|l| l.line_number as usize), e.message().to_string()))?;
    let mut v = naga::valid::Validator::new(naga::valid::ValidationFlags::all(), naga::valid::Capabilities::empty());
    v.validate(&module).map_err(|e| {
        let line = e.spans().next().map(|(span, _)| span.location(src).line_number as usize);
        let mut msg = e.as_inner().to_string();
        let mut cause: Option<&dyn std::error::Error> = std::error::Error::source(e.as_inner());
        while let Some(c) = cause {
            msg = format!("{msg}: {c}");
            cause = c.source();
        }
        (line, msg)
    })?;
    Ok(())
}

/// Check one shader alone: `None` = fine, else a message with its own line number.
pub fn check(sh: &CustomSurface) -> Option<String> {
    if !declared_names(&sh.source).iter().any(|n| n == "surface") {
        return Some(varstotas_i18n::tr!("render-shader-define-surface"));
    }
    let (src, starts) = compose(&[Some(sh)]);
    match validate(&src) {
        Ok(()) => None,
        Err((line, msg)) => {
            let msg = msg.replace("_c0", "");
            Some(match line {
                Some(l) if l >= starts[0] && l < starts[0] + sh.source.lines().count() => varstotas_i18n::tr!("render-shader-line", line = l - starts[0] + 1, msg = msg),
                _ => msg,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const WOOD: &str = "// @param scale = 4\nfn ring(r: f32) -> f32 { return fract(r); }\nfn surface(s: Surface, p: Params) -> Surface {\n    var o = s;\n    o.base = o.base * ring(length(s.pos.xy) / p.scale); // fn fake(\n    return o;\n}\n";

    #[test]
    fn names_are_suffixed() {
        let names = declared_names(WOOD);
        assert_eq!(names, vec!["ring".to_string(), "surface".to_string()]);
        let r = rename(WOOD, &names, 3);
        assert!(r.contains("fn ring_c3(") && r.contains("fn surface_c3(s: Surface, p: Params_c3)"));
        assert!(r.contains("ring_c3(length") && r.contains("// fn fake("));
    }

    #[test]
    fn shaders_compose_and_report_errors() {
        let ok = CustomSurface { source: WOOD.into(), params: vec!["scale".into()] };
        assert_eq!(check(&ok), None);
        let other = CustomSurface { source: WOOD.replace("4", "2"), params: vec!["scale".into()] };
        let (src, _) = compose(&[Some(&ok), None, Some(&other)]);
        assert!(validate(&src).is_ok(), "{:?}", validate(&src));
        assert!(src.contains("case 3u"));
        let bad = CustomSurface { source: "fn surface(s: Surface, p: Params) -> Surface {\n    return s.nope;\n}\n".into(), params: vec![] };
        let e = check(&bad).unwrap();
        assert!(e.starts_with("line 2:"), "{e}");
        let missing = CustomSurface { source: "fn other() {}".into(), params: vec![] };
        assert!(check(&missing).unwrap().contains("fn surface"));
    }
}

/// The OpenGL fallback (no Vulkan driver: some VMs, a Flatpak whose Mesa does not drive the GPU)
/// translates every entry point to GLSL ES 3.10 the way wgpu does (one entry point, unused parts
/// dropped); features GLSL ES lacks (`@interpolate(linear)`, depth `textureLoad`) fail there.
/// The GL backend (naga's GLSL writer) is not built on Apple platforms.
#[cfg(all(test, not(target_vendor = "apple")))]
mod gles_tests {
    use super::{compose, naga};

    #[test]
    fn every_entry_point_translates_to_glsl_es() {
        let (src, _) = compose(&[]);
        let module = naga::front::wgsl::parse_str(&src).expect("parse");
        let info = naga::valid::Validator::new(naga::valid::ValidationFlags::all(), naga::valid::Capabilities::all())
            .validate(&module)
            .expect("validate");
        let options = naga::back::glsl::Options::default(); // GLSL ES 3.10
        let mut failed = Vec::new();
        for ep in &module.entry_points {
            let (m, i) = naga::back::pipeline_constants::process_overrides(&module, &info, Some((ep.stage, &ep.name)), &Default::default())
                .expect("overrides");
            let pipeline = naga::back::glsl::PipelineOptions { shader_stage: ep.stage, entry_point: ep.name.clone(), multiview: None };
            let mut out = String::new();
            let r = naga::back::glsl::Writer::new(&mut out, &m, &i, &options, &pipeline, Default::default()).and_then(|mut w| w.write());
            if let Err(e) = r {
                failed.push(format!("{}: {e}", ep.name));
            }
        }
        assert!(failed.is_empty(), "{failed:#?}");
    }
}
