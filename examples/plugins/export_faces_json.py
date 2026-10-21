# @name Faces as JSON (plugin)
# @name.ru Грани в JSON (плагин)
# @export json
# @description An example exporter (built in: File → Export → Faces (JSON)…). Every face of the model (world coordinates, mm) with its normal, area and material: an example exporter in Python.
# @description.ru Пример экспортёра на Python (встроен: Файл → Экспорт → Грани (JSON)…): каждая грань модели (мировые координаты, мм) с нормалью, площадью и материалом.
import json

import varstotas

m = varstotas.connect()
path = varstotas.params()["path"]
faces = m.rhai("""
    let out = [];
    for f in all_faces() {
        out.push(#{ vertices: f.vertices, normal: f.normal, area: f.area, material: f.material });
    }
    out
""")
data = [
    {"vertices": [list(v) for v in f["vertices"]], "normal": list(f["normal"]), "area": f["area"], "material": f["material"]}
    for f in faces
]
with open(path, "w", encoding="utf-8") as fh:
    json.dump({"units": "mm", "faces": data}, fh, indent=1)
print("Exported %d faces to %s" % (len(data), path))
