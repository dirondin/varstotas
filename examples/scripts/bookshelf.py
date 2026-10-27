"""Bookshelf from Python: the same model as bookshelf.rhai, built through the bridge.

Run it from Varstotas (File -> Run Script...) or from a terminal while Varstotas is open:
    PYTHONPATH=python python3 examples/scripts/bookshelf.py      (from the repository root)
"""
import varstotas
from varstotas import vec3

w, h, d, t = 800, 1800, 300, 18   # outer width, height, depth, board thickness (mm)
shelves = 4

m = varstotas.connect()
with m.operation("Bookshelf"):     # one undo step
    m.material("Birch", "#E3C99A")
    m.material("Back", "#B8B2A6")
    parts = []
    for name, origin, size in [
        ("Side L", vec3(0, 0, 0), (t, d, h)),
        ("Side R", vec3(w - t, 0, 0), (t, d, h)),
        ("Bottom", vec3(t, 0, 0), (w - 2 * t, d, t)),
        ("Top", vec3(t, 0, h - t), (w - 2 * t, d, t)),
    ]:
        b = m.box(origin, *size)
        b.name = name
        parts.append(b)
    step = (h - t) / (shelves + 1)
    for i in range(1, shelves + 1):
        s = m.box(vec3(t, 0, i * step), w - 2 * t, d - 10, t)
        s.name = "Shelf %d" % i
        parts.append(s)
    m.paint(parts, "Birch")
    back = m.box(vec3(0, d, 0), w, 4, h)
    back.name = "Back"
    m.paint(back, "Back")
    shelf = m.group(parts + [back])
    shelf.name = "Bookshelf"
    m.select(shelf)

size = m.bounds(shelf)["size"]
print("Bookshelf %g x %g x %g, %d parts" % (size.x, size.y, size.z, len(m.entities(shelf))))
