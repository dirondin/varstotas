// @name Texture Folders
// @name.ru Папки текстур
// @name.lt Tekstūrų aplankai
// @material_source
// @description Materials from texture sets on this computer: every subfolder of the folder is one material, its images sorted by their names (…_color / _diff, _normal / _nor_gl, _rough, _ao, _metal, _emission).
// @description.ru Материалы из наборов текстур на этом компьютере: каждая подпапка — один материал, изображения различаются по именам (…_color / _diff, _normal / _nor_gl, _rough, _ao, _metal, _emission).
// @description.lt Medžiagos iš tekstūrų rinkinių šiame kompiuteryje: kiekvienas poaplankis – viena medžiaga, paveikslėliai atskiriami pagal pavadinimus (…_color / _diff, _normal / _nor_gl, _rough, _ao, _metal, _emission).
// @param folder:text = "" "Folder"
// @param.ru folder "Папка"
// @param.lt folder "Aplankas"
// @param width = 1000 "Texture width"
// @param.ru width "Ширина текстуры"
// @param.lt width "Tekstūros plotis"

fn is_image(f) {
    let l = f.to_lower();
    l.ends_with(".png") || l.ends_with(".jpg") || l.ends_with(".jpeg") || l.ends_with(".tga") || l.ends_with(".bmp")
}

fn name_of(p) {
    let s = p;
    s.replace("\\", "/");
    let parts = s.split("/");
    parts[parts.len() - 1]
}

fn catalogue(params) {
    if params.folder == "" { throw "Set the folder (one subfolder per material) and press Load"; }
    if !file_exists(params.folder) { throw "No folder " + params.folder; }
    let out = [];
    for d in list_dirs(params.folder) {
        let images = [];
        for f in list_files(d, "") {
            if is_image(f) { images.push(f); }
        }
        if images.len() == 0 { continue; }
        // The thumbnail: the colour map when one is named so.
        let thumb = images[0];
        for f in images {
            let l = name_of(f).to_lower();
            if l.contains("color") || l.contains("colour") || l.contains("diff") || l.contains("albedo") { thumb = f; break; }
        }
        out.push(#{ id: d, name: name_of(d), category: "Local", width: params.width, thumb: thumb, license: "Local files" });
    }
    out
}

fn download(id, dir, params) {
    #{ name: name_of(id), folder: id, width: params.width }
}
