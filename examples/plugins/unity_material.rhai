// @name Unity Material
// @name.ru Материал Unity
// @material_import mat
// @description Unity materials (.mat, Standard / URP / HDRP Lit): base colour and map, normal map, occlusion, emission, metallic and smoothness. The textures are found by their GUIDs in the .meta files of the project's Assets folder (PNG, JPEG, TGA, BMP).
// @description.ru Материалы Unity (.mat, Standard / URP / HDRP Lit): базовый цвет и карта, карта нормалей, затенение, свечение, металличность и гладкость. Текстуры ищутся по GUID в файлах .meta папки Assets проекта (PNG, JPEG, TGA, BMP).
// @param width = 1000 "Texture width"
// @param.ru width "Ширина текстуры"

fn trimmed(s) {
    let t = s;
    t.trim();
    t
}

// "guid: 1234, type: 3}" → the text after `name: ` up to `,` or `}`.
fn field(s, name) {
    let i = s.index_of(name + ": ");
    if i < 0 { return ""; }
    let rest = s.sub_string(i + name.len() + 2);
    let end = rest.index_of(",");
    if end < 0 { end = rest.index_of("}"); }
    if end >= 0 { rest = rest.sub_string(0, end); }
    trimmed(rest)
}

fn number(s) {
    try { parse_float(trimmed(s)) } catch { 0.0 }
}

// Folder of a path ("/" separators).
fn parent(p) {
    let parts = p.split("/");
    parts.pop();
    parts.reduce(|a, b| if a == () { b } else { a + "/" + b })
}

fn first(m, keys) {
    for k in keys {
        if k in m { return m[k]; }
    }
    ()
}

fn import_material(path, params) {
    let file = path;
    file.replace("\\", "/");
    let name = "";
    let section = "";
    let current = "";
    let tex = #{};
    let floats = #{};
    let colors = #{};
    for raw in read_text(path).split("\n") {
        let l = trimmed(raw);
        if l.starts_with("m_Name:") {
            if name == "" { name = trimmed(l.sub_string(7)); }
            continue;
        }
        if l == "m_TexEnvs:" || l == "m_Floats:" || l == "m_Colors:" || l == "m_Ints:" {
            section = l;
            continue;
        }
        if l.starts_with("- ") {
            let kv = l.sub_string(2);
            let i = kv.index_of(":");
            if i < 0 { continue; }
            let key = kv.sub_string(0, i);
            let val = kv.sub_string(i + 1);
            if section == "m_TexEnvs:" {
                current = key;
            } else if section == "m_Floats:" || section == "m_Ints:" {
                floats[key] = number(val);
            } else if section == "m_Colors:" {
                colors[key] = [number(field(val, "r")), number(field(val, "g")), number(field(val, "b")), number(field(val, "a"))];
            }
            continue;
        }
        if section == "m_TexEnvs:" && current != "" && l.starts_with("m_Texture:") {
            let g = field(l, "guid");
            if g != "" { tex[current] = g; }
        }
    }
    if name == "" { name = "Unity Material"; }

    // GUID → texture file, from the .meta files of the Assets folder (or the folder above).
    let root = parent(file);
    let up = root;
    while up != () && up != "" {
        if up.ends_with("/Assets") || up == "Assets" { root = up; break; }
        up = parent(up);
    }
    if !root.ends_with("/Assets") { root = parent(root) ?? root; }
    let wanted = tex.values();
    let files = #{};
    if wanted.len() > 0 {
        for meta in list_files(root, "meta") {
            let text = read_text(meta);
            for g in wanted {
                if text.contains("guid: " + g) {
                    files[g] = meta.sub_string(0, meta.len() - 5);
                }
            }
        }
    }
    let maps = #{};
    let slots = [
        ["color", ["_BaseMap", "_MainTex", "_BaseColorMap"]],
        ["normal", ["_BumpMap", "_NormalMap"]],
        ["ao", ["_OcclusionMap"]],
        ["emission", ["_EmissionMap", "_EmissiveColorMap"]],
    ];
    for s in slots {
        let g = first(tex, s[1]);
        if g == () || !(g in files) { continue; }
        let f = files[g];
        let low = f.to_lower();
        if low.ends_with(".png") || low.ends_with(".jpg") || low.ends_with(".jpeg") || low.ends_with(".tga") || low.ends_with(".bmp") {
            maps[s[0]] = f;
        } else {
            print("Skipped " + f + " (not PNG, JPEG, TGA or BMP)");
        }
    }

    let spec = #{ name: name, width: params.width, maps: maps };
    let c = first(colors, ["_BaseColor", "_Color"]);
    if c != () {
        if !("color" in maps) { spec.color = [c[0], c[1], c[2]]; }
        let surface = first(floats, ["_Surface", "_Mode"]);
        if surface != () && surface >= 1.0 { spec.opacity = c[3]; }
    }
    let metal = first(floats, ["_Metallic"]);
    if metal != () { spec.metallic = metal; }
    let smooth = first(floats, ["_Smoothness", "_Glossiness"]);
    if smooth != () { spec.roughness = 1.0 - smooth; }
    let ns = first(floats, ["_BumpScale", "_NormalScale"]);
    if ns != () { spec.normal_scale = ns; }
    let e = first(colors, ["_EmissionColor", "_EmissiveColor"]);
    if e != () && (e[0] > 0.0 || e[1] > 0.0 || e[2] > 0.0) {
        let k = max(e[0], max(e[1], e[2]));
        spec.emission = [e[0] / k, e[1] / k, e[2] / k];
        spec.emission_strength = k;
    }
    spec
}
