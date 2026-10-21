# @name PLY Import (Python) (plugin)
# @name.ru Импорт PLY (Python) (плагин)
# @model_import ply
# @description An example model importer in Python (Varstotas reads PLY itself: File → Import → PLY…; this plugin stays as an example): Stanford PLY meshes (ASCII or binary) for File → Import and the 3D Library. The protocol: `plugin.py import_model`, the arguments ({path, params}) as JSON on stdin, a model spec as JSON on the last line of stdout ({meshes: [{name, vertices, faces}], unit}).
# @description.ru Пример импортёра моделей на Python (Varstotas читает PLY сам: Файл → Импорт → PLY…; плагин оставлен как пример): сетки Stanford PLY (ASCII или двоичные) для Файл → Импорт и 3D-библиотеки. Протокол: `plugin.py import_model`, аргументы ({path, params}) — JSON в stdin, описание модели — JSON в последней строке stdout ({meshes: [{name, vertices, faces}], unit}).
# @param unit:text = "m" "Unit of the file (m, cm, mm, in, ft)"
# @param.ru unit "Единицы файла (m, cm, mm, in, ft)"
import json
import os
import struct
import sys

TYPES = {
    "char": "b", "int8": "b", "uchar": "B", "uint8": "B", "short": "h", "int16": "h",
    "ushort": "H", "uint16": "H", "int": "i", "int32": "i", "uint": "I", "uint32": "I",
    "float": "f", "float32": "f", "double": "d", "float64": "d",
}


def read_ply(path):
    with open(path, "rb") as f:
        data = f.read()
    end = data.find(b"end_header")
    if not data.startswith(b"ply") or end < 0:
        raise ValueError("not a PLY file")
    body = data[data.index(b"\n", end) + 1:]
    elements, fmt = [], "ascii"
    for line in data[:end].decode("latin-1").splitlines():
        w = line.split()
        if not w:
            continue
        if w[0] == "format":
            fmt = w[1]
        elif w[0] == "element":
            elements.append((w[1], int(w[2]), []))
        elif w[0] == "property" and elements:
            elements[-1][2].append(w[1:])
    verts, faces = [], []
    if fmt == "ascii":
        nums = iter(body.split())
        for name, count, props in elements:
            for _ in range(count):
                row = {}
                for p in props:
                    if p[0] == "list":
                        n = int(next(nums))
                        row[p[-1]] = [int(next(nums)) for _ in range(n)]
                    else:
                        row[p[-1]] = float(next(nums))
                store(name, row, verts, faces)
    else:
        order = "<" if fmt == "binary_little_endian" else ">"
        at = 0

        def take(t):
            nonlocal at
            code = order + TYPES[t]
            v = struct.unpack_from(code, body, at)[0]
            at += struct.calcsize(code)
            return v

        for name, count, props in elements:
            for _ in range(count):
                row = {}
                for p in props:
                    if p[0] == "list":
                        n = take(p[1])
                        row[p[-1]] = [take(p[2]) for _ in range(n)]
                    else:
                        row[p[-1]] = take(p[0])
                store(name, row, verts, faces)
    return verts, faces


def store(name, row, verts, faces):
    if name == "vertex":
        verts.append([row.get("x", 0.0), row.get("y", 0.0), row.get("z", 0.0)])
    elif name == "face":
        faces.append(row.get("vertex_indices") or row.get("vertex_index") or [])


args = json.load(sys.stdin)
command = sys.argv[1] if len(sys.argv) > 1 else ""
if command != "import_model":
    sys.exit("unknown command " + command)
verts, faces = read_ply(args["path"])
name = os.path.splitext(os.path.basename(args["path"]))[0]
unit = args.get("params", {}).get("unit", "m") or "m"
print(json.dumps({"name": name, "unit": unit, "meshes": [{"name": name, "vertices": verts, "faces": faces}]}))
