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
    m.set_stock("Birch", {"kind": "sheet", "length": 2440, "width": 1220, "thickness": t, "grain": True, "price": 85})
    m.set_stock("Back", {"kind": "sheet", "length": 2440, "width": 1220, "thickness": 4, "price": 18})
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

    # Joinery: the top and bottom in rabbets cut into the sides, the shelves on dowels
    side_l, side_r = m.child(shelf, "Side L"), m.child(shelf, "Side R")
    names = ["Bottom", "Top"] + ["Shelf %d" % i for i in range(1, shelves + 1)]
    for n in names:
        kind = "housing" if n in ("Bottom", "Top") else "dowels"
        m.joint(side_l, m.child(shelf, n), kind)
        m.joint(side_r, m.child(shelf, n), kind)
    # What else to buy: the dowels priced by name, glue and the back's nails by hand
    m.cut_part("Dowel Ø8 × 40", {"price": 0.05})
    m.cut_hardware([{"name": "Wood glue 250 ml", "count": 1, "price": 6},
                    {"name": "Nail 1.6 × 25", "count": 30, "price": 0.02}])
    # Assembly steps (Window -> Assembly): the case is built lying on its left side
    m.set_assembly_steps([
        {"parts": side_l, "note": "Lay the left side on a flat surface, inside up."},
        {"parts": [m.child(shelf, n) for n in names],
         "note": "Glue the rabbets and the dowels, then set the bottom, the top and the shelves into the left side."},
        {"parts": side_r, "note": "Put the right side on, the dowels in their holes, and tap it down evenly. Clamp until the glue sets."},
        {"parts": m.child(shelf, "Back"), "note": "Check the diagonals are equal, then nail or screw the back on."},
    ])
    m.select(shelf)

size = m.bounds(shelf)["size"]
print("Bookshelf %g x %g x %g, %d parts, %d joints" % (size.x, size.y, size.z, len(m.entities(shelf)), len(m.joints())))
