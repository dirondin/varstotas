// @name Model Folders
// @model_source
// @description 3D models on this computer: every model file in the folder and its subfolders (glb, gltf, obj, stl, dxf, varstotas, and what importer plugins read); an image with the same name is its thumbnail, the subfolder its category.
// @param folder:text = "" "Folder"
// @param unit:text = "" "Unit (empty: the file's own)"

fn name_of(p) {
    let s = p;
    s.replace("\\", "/");
    let parts = s.split("/");
    parts[parts.len() - 1]
}

fn stem_of(p) {
    let n = name_of(p);
    let i = n.index_of(".");
    if i > 0 { n.sub_string(0, i) } else { n }
}

fn catalogue(params) {
    if params.folder == "" { throw "Set the folder of your models and press Load"; }
    if !file_exists(params.folder) { throw "No folder " + params.folder; }
    let exts = ["glb", "gltf", "obj", "stl", "dxf", "vrst", "ply"];
    let out = [];
    for f in list_files(params.folder, "") {
        let l = f.to_lower();
        let ok = false;
        for e in exts {
            if l.ends_with("." + e) { ok = true; }
        }
        if !ok { continue; }
        let dir = f.sub_string(0, f.len() - name_of(f).len());
        let thumb = "";
        for e in [".png", ".jpg", ".jpeg"] {
            if file_exists(dir + stem_of(f) + e) { thumb = dir + stem_of(f) + e; break; }
        }
        let cat = name_of(dir.sub_string(0, dir.len() - 1));
        out.push(#{ id: f, name: stem_of(f), category: cat, thumb: thumb, license: "Local files" });
    }
    out
}

fn download(id, dir, params) {
    let spec = #{ file: id, name: stem_of(id) };
    if params.unit != "" { spec.unit = params.unit; }
    spec
}
