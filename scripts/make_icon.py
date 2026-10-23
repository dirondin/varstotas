#!/usr/bin/env python3
"""Programmatic Varstotas app icon (stdlib only).

A complete four-legged workbench seen orthographically from an oblique above-front angle. The screen's
vertical centre divides the textured wooden view on the left from the translucent X-ray
schematic view on the right. No model axes; a continuous soft gradient fills the rounded square.

  make_icon.py png  <out.png> [size]     -- master PNG (default 1024; takes a while)
  make_icon.py ico  <out.ico> <png>...   -- pack PNG files into a Windows .ico

The master is committed as crates/varstotas_app/assets/icon.png (window / Dock icon and the
source of the bundles' .icns / .ico); regenerate it after changing this script.
"""
import math
import struct
import sys
import zlib

# ---- scene (model units, Z up) -------------------------------------------------------------

# Boxes: (min, max, grain axis 0/1/2).
RAIL_Z = 10.0 / 3.0  # two thirds of the leg height down from the tabletop
BOXES = [
    ((0.0, 0.0, 10.0), (18.0, 10.0, 10.65), 0),  # complete tabletop
    *[((x, y, 0.0), (x + 1.0, y + 1.0, 10.0), 2)
      for x in (1.3, 15.7) for y in (1.3, 7.7)],  # four slimmer legs, same centres
    *[((2.3, y, RAIL_Z - 0.5), (15.7, y + 0.6, RAIL_Z + 0.5), 0)
      for y in (1.5, 7.9)],  # long lower rails, joined to the inner leg faces
    *[((x, 2.3, RAIL_Z - 0.5), (x + 0.6, 7.7, RAIL_Z + 0.5), 1)
      for x in (1.5, 15.9)],  # short lower rails, at exactly the same height
]

EYE_DIR = (1.63, -2.0, 1.175)  # orthographic tabletop slopes matched to the app screenshot
TARGET = (9.0, 5.0, 5.325)
SPLIT_X = 0.5  # screen space, independent of model orientation
LIGHT = (-0.45, -0.75, 0.95)

# Split by projected screen position, rather than a diagonal plane in model space.
def is_wood(p):
    return project(p)[0] < SPLIT_X


WOOD_LIGHT, WOOD_DARK = (0.86, 0.66, 0.42), (0.62, 0.40, 0.20)
SCHEMATIC = (0.72, 0.81, 0.94)
SCHEMATIC_ALPHA = 0.28
EDGE_W = 0.0030  # icon units
EDGE_COL = (0.05, 0.06, 0.08)

BG_TOP, BG_BOTTOM = (0.92, 0.94, 0.96), (0.72, 0.77, 0.83)
INSET, RADIUS = 0.085, 0.19


# ---- small vector helpers --------------------------------------------------------------------

def sub(a, b):
    return (a[0] - b[0], a[1] - b[1], a[2] - b[2])


def dot(a, b):
    return a[0] * b[0] + a[1] * b[1] + a[2] * b[2]


def cross(a, b):
    return (a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0])


def norm(a):
    l = math.sqrt(dot(a, a))
    return (a[0] / l, a[1] / l, a[2] / l)


FWD = norm(EYE_DIR)  # towards the viewer
RIGHT = norm(cross((0.0, 0.0, 1.0), FWD))
UP = cross(FWD, RIGHT)
LIGHT_N = norm(LIGHT)


def camera_project(p):
    d = sub(p, TARGET)
    return dot(d, RIGHT), dot(d, UP), -dot(d, FWD)


# Fit the entire bench inside the rounded square with room around all four legs.
CORNERS = [tuple((lo, hi)[side][axis] for axis, side in enumerate(sides))
           for lo, hi, _ in BOXES
           for sides in ((x, y, z) for x in (0, 1) for y in (0, 1) for z in (0, 1))]
PX = [camera_project(p)[0] for p in CORNERS]
PY = [camera_project(p)[1] for p in CORNERS]
SCALE = min(0.78 / (max(PX) - min(PX)), 0.78 / (max(PY) - min(PY)))
OFFSET = (0.5 - (min(PX) + max(PX)) * SCALE / 2,
          0.540 + (min(PY) + max(PY)) * SCALE / 2)


def project(p):
    """Model point -> (x, y, depth) in icon units (y down, larger depth = farther)."""
    x, y, depth = camera_project(p)
    return (OFFSET[0] + x * SCALE, OFFSET[1] - y * SCALE, depth)


# ---- procedural wood -------------------------------------------------------------------------

def hash3(i, j, k):
    h = (i * 374761393 + j * 668265263 + k * 2147483647) & 0xFFFFFFFF
    h = ((h ^ (h >> 13)) * 1274126177) & 0xFFFFFFFF
    return ((h ^ (h >> 16)) & 0xFFFF) / 65535.0


def vnoise(x, y, z):
    i, j, k = math.floor(x), math.floor(y), math.floor(z)
    fx, fy, fz = x - i, y - j, z - k
    fx, fy, fz = fx * fx * (3 - 2 * fx), fy * fy * (3 - 2 * fy), fz * fz * (3 - 2 * fz)

    def lerp(a, b, t):
        return a + (b - a) * t

    c = [[[hash3(i + a, j + b, k + c) for c in (0, 1)] for b in (0, 1)] for a in (0, 1)]
    return lerp(
        lerp(lerp(c[0][0][0], c[0][0][1], fz), lerp(c[0][1][0], c[0][1][1], fz), fy),
        lerp(lerp(c[1][0][0], c[1][0][1], fz), lerp(c[1][1][0], c[1][1][1], fz), fy),
        fx,
    )


def wood(p, axis):
    """Long grain along `axis`: rings across it, stretched noise along it."""
    a = p[axis]
    u, v = [p[i] for i in range(3) if i != axis]
    warp = vnoise(a * 0.35, u * 1.3, v * 1.3) * 1.6
    rings = 0.5 + 0.5 * math.sin((u * 0.9 + v * 0.55 + warp) * 5.0)
    rings = rings ** 3
    fine = vnoise(a * 0.6, u * 14.0, v * 14.0)
    t = min(1.0, max(0.0, 0.25 + 0.6 * rings + 0.25 * (fine - 0.5)))
    return tuple(WOOD_LIGHT[i] + (WOOD_DARK[i] - WOOD_LIGHT[i]) * t for i in range(3))


# ---- geometry --------------------------------------------------------------------------------

def box_faces(lo, hi):
    """Quads (4 corners, outward normal) of an axis-aligned box."""
    x0, y0, z0 = lo
    x1, y1, z1 = hi
    return [
        (((x0, y0, z0), (x1, y0, z0), (x1, y0, z1), (x0, y0, z1)), (0, -1, 0)),
        (((x1, y1, z0), (x0, y1, z0), (x0, y1, z1), (x1, y1, z1)), (0, 1, 0)),
        (((x0, y1, z0), (x0, y0, z0), (x0, y0, z1), (x0, y1, z1)), (-1, 0, 0)),
        (((x1, y0, z0), (x1, y1, z0), (x1, y1, z1), (x1, y0, z1)), (1, 0, 0)),
        (((x0, y0, z1), (x1, y0, z1), (x1, y1, z1), (x0, y1, z1)), (0, 0, 1)),
        (((x0, y1, z0), (x1, y1, z0), (x1, y0, z0), (x0, y0, z0)), (0, 0, -1)),
    ]


def box_edges(lo, hi):
    c = [(x, y, z) for x in (lo[0], hi[0]) for y in (lo[1], hi[1]) for z in (lo[2], hi[2])]
    return [(c[a], c[b]) for a in range(8) for b in range(a + 1, 8)
            if sum(c[a][i] != c[b][i] for i in range(3)) == 1]


# ---- rasterizer ------------------------------------------------------------------------------

class Canvas:
    def __init__(self, size):
        self.size = size
        self.surfaces = [None] * (size * size)  # nearest fill (depth, rgb, alpha)
        self.layers = {}  # edge samples, including X-ray edges on the right

    def frag(self, i, depth, rgb, alpha):
        # Use one translucent shell for a clear tint; draw the X-ray edges separately.
        if alpha <= 0:
            return
        surface = self.surfaces[i]
        if surface is None or depth < surface[0]:
            self.surfaces[i] = (depth, rgb, alpha)

    def triangle(self, tri, shade):
        """`tri`: three (model point, projected point); `shade(p) -> (rgb, alpha)`."""
        n = self.size
        (pa, a), (pb, b), (pc, c) = tri
        ax, ay, bx, by, cx, cy = a[0] * n, a[1] * n, b[0] * n, b[1] * n, c[0] * n, c[1] * n
        area = (bx - ax) * (cy - ay) - (by - ay) * (cx - ax)
        if abs(area) < 1e-9:
            return
        x0, x1 = max(0, int(min(ax, bx, cx))), min(n - 1, int(max(ax, bx, cx)) + 1)
        y0, y1 = max(0, int(min(ay, by, cy))), min(n - 1, int(max(ay, by, cy)) + 1)
        for py in range(y0, y1 + 1):
            sy = py + 0.5
            for px in range(x0, x1 + 1):
                sx = px + 0.5
                w0 = ((bx - sx) * (cy - sy) - (by - sy) * (cx - sx)) / area
                w1 = ((cx - sx) * (ay - sy) - (cy - sy) * (ax - sx)) / area
                w2 = 1.0 - w0 - w1
                if w0 < 0 or w1 < 0 or w2 < 0:
                    continue
                # Orthographic projection interpolates model position and depth linearly.
                p = tuple(w0 * pa[k] + w1 * pb[k] + w2 * pc[k] for k in range(3))
                depth = w0 * a[2] + w1 * b[2] + w2 * c[2]
                rgb, alpha = shade(p)
                self.frag(py * n + px, depth, rgb, alpha)

    def segment(self, pa, pb, width, color_at, bias, front_visible=True):
        """Anti-aliased line from model `pa` to `pb`; `color_at(p) -> (rgb, alpha)`."""
        n = self.size
        a, b = project(pa), project(pb)
        ax, ay, bx, by = a[0] * n, a[1] * n, b[0] * n, b[1] * n
        hw = width * n / 2
        dx, dy = bx - ax, by - ay
        ll = dx * dx + dy * dy
        if ll < 1e-12:
            return
        x0, x1 = max(0, int(min(ax, bx) - hw - 1)), min(n - 1, int(max(ax, bx) + hw + 1))
        y0, y1 = max(0, int(min(ay, by) - hw - 1)), min(n - 1, int(max(ay, by) + hw + 1))
        for py in range(y0, y1 + 1):
            sy = py + 0.5
            for px in range(x0, x1 + 1):
                sx = px + 0.5
                t = ((sx - ax) * dx + (sy - ay) * dy) / ll
                if t < 0 or t > 1:
                    continue
                qx, qy = ax + t * dx - sx, ay + t * dy - sy
                d = math.sqrt(qx * qx + qy * qy)
                cov = min(1.0, hw + 0.5 - d)
                if cov <= 0:
                    continue
                p = tuple(pa[k] + (pb[k] - pa[k]) * t for k in range(3))
                rgb, alpha = color_at(p)
                depth = a[2] + (b[2] - a[2]) * t - bias
                i = py * n + px
                o = self.surfaces[i]
                hidden = not front_visible or (o is not None and depth > o[0])
                if sx / n < SPLIT_X:
                    if hidden:
                        continue
                elif hidden:
                    alpha *= 0.45
                self.layers.setdefault(i, []).append((depth, rgb, alpha * cov))


def background(x, y):
    t = max(0.0, min(1.0, (y - INSET) / (1.0 - 2.0 * INSET)))
    t = t * t * (3.0 - 2.0 * t)
    return tuple(BG_TOP[i] + (BG_BOTTOM[i] - BG_TOP[i]) * t for i in range(3))


def in_round_rect(x, y):
    lo, hi = INSET, 1.0 - INSET
    if x < lo or x > hi or y < lo or y > hi:
        return False
    cx = min(max(x, lo + RADIUS), hi - RADIUS)
    cy = min(max(y, lo + RADIUS), hi - RADIUS)
    return (x - cx) ** 2 + (y - cy) ** 2 <= RADIUS * RADIUS


def lit(rgb, normal, ambient):
    k = ambient + (1 - ambient) * max(0.0, dot(normal, LIGHT_N))
    return tuple(min(1.0, c * k) for c in rgb)


def draw_dimensions(cv):
    """X/Y tabletop dimensions and Z front-right leg height, as in the app."""
    z = BOXES[0][1][2]
    corners = ((0.0, 0.0, z), (0.0, 10.0, z), (18.0, 10.0, z))
    tips = [(*p[:2], z + 1.4) for p in corners]
    ink = lambda p: (EDGE_COL, 1.0)
    overlay_bias = 1e6  # CAD annotations stay readable over translucent geometry.
    for p in corners:
        cv.segment((*p[:2], z + 0.12), (*p[:2], z + 1.65), 0.0014, ink, overlay_bias)

    def arrow(tip, towards):
        direction = sub(towards, tip)
        dx, dy = dot(direction, RIGHT), -dot(direction, UP)
        length = math.hypot(dx, dy)
        along = 0.012 / (length * SCALE)
        base = tuple(tip[k] + direction[k] * along for k in range(3))
        side = tuple((-dy * RIGHT[k] - dx * UP[k]) / length * 0.0035 / SCALE for k in range(3))
        wings = [tuple(base[k] + sign * side[k] for k in range(3)) for sign in (-1, 1)]
        pts = [(p, (*project(p)[:2], -overlay_bias)) for p in (tip, *wings)]
        cv.triangle(pts, ink)

    def label(letter, a, b, offset):
        # Simple vector lettering stays crisp at every icon size, without a font dependency.
        glyphs = {
            "X": [((-0.5, -0.5), (0.5, 0.5)), ((-0.5, 0.5), (0.5, -0.5))],
            "Y": [((-0.5, -0.5), (0.0, 0.0), (0.5, -0.5)), ((0.0, 0.0), (0.0, 0.5))],
            "Z": [((-0.5, -0.5), (0.5, -0.5), (-0.5, 0.5), (0.5, 0.5))],
        }
        mid = tuple((a[k] + b[k]) / 2 for k in range(3))
        for stroke in glyphs[letter]:
            pts = [tuple(mid[k] + RIGHT[k] * (x * 0.018 + offset[0]) / SCALE
                         - UP[k] * (y * 0.026 + offset[1]) / SCALE for k in range(3)) for x, y in stroke]
            for p, q in zip(pts, pts[1:]):
                cv.segment(p, q, 0.0065, ink, overlay_bias)

    for letter, (a, b) in zip(("Y", "X"), zip(tips, tips[1:])):
        cv.segment(a, b, 0.0017, ink, overlay_bias)
        arrow(a, b)
        arrow(b, a)
        label(letter, a, b, (0.0, -0.023))

    # Offset the height dimension along the tabletop's short axis, beside the right leg.
    lo, hi, _ = BOXES[3]  # front-right leg
    a, b = (hi[0], hi[1] + 1.7, lo[2]), (hi[0], hi[1] + 1.7, hi[2])
    for height in (lo[2], hi[2]):
        cv.segment((hi[0], hi[1] + 0.12, height), (hi[0], hi[1] + 1.95, height), 0.0014, ink, overlay_bias)
    cv.segment(a, b, 0.0017, ink, overlay_bias)
    arrow(a, b)
    arrow(b, a)
    label("Z", a, b, (0.022, 0.10))  # keep the letter clear of the lower rails


def draw(n):
    cv = Canvas(n)
    for lo, hi, grain in BOXES:
        for quad, nrm in box_faces(lo, hi):
            if dot(nrm, FWD) <= 0:
                continue  # nearest outward shell supplies the fill; X-ray edges follow

            def shade(p, nrm=nrm, grain=grain):
                if is_wood(p):
                    return lit(wood(p, grain), nrm, 0.55), 1.0
                return lit(SCHEMATIC, nrm, 0.6), SCHEMATIC_ALPHA

            pts = [(q, project(q)) for q in quad]
            cv.triangle((pts[0], pts[1], pts[2]), shade)
            cv.triangle((pts[0], pts[2], pts[3]), shade)
    # Wood hides rear geometry; the schematic half also shows hidden edges, more faintly.
    for lo, hi, _ in BOXES:
        for a, b in box_edges(lo, hi):
            normals = [tuple((-1.0 if a[k] == lo[k] else 1.0) if i == k else 0.0 for i in range(3))
                       for k in range(3) if a[k] == b[k]]
            front_visible = any(dot(nrm, FWD) > 0 for nrm in normals)
            cv.segment(a, b, EDGE_W, lambda p: (EDGE_COL, 1.0 if is_wood(p) else 0.75), 0.05, front_visible)
    draw_dimensions(cv)
    return cv


def render(size, ss=2):
    n = size * ss
    cv = draw(n)
    out = bytearray()
    for py in range(size):
        out.append(0)  # PNG filter: none
        for px in range(size):
            acc = [0.0, 0.0, 0.0, 0.0]
            for oy in range(ss):
                for ox in range(ss):
                    sx, sy = px * ss + ox, py * ss + oy
                    x, y = (sx + 0.5) / n, (sy + 0.5) / n
                    if not in_round_rect(x, y):
                        continue
                    i = sy * n + sx
                    o = cv.surfaces[i]
                    col = background(x, y)
                    if o:
                        col = tuple(col[k] + (o[1][k] - col[k]) * o[2] for k in range(3))
                    for d, rgb, a in sorted(cv.layers.get(i, ()), key=lambda f: -f[0]):
                        if x >= SPLIT_X or o is None or d <= o[0]:
                            col = tuple(col[k] + (rgb[k] - col[k]) * a for k in range(3))
                    for k in range(3):
                        acc[k] += col[k]
                    acc[3] += 1
            m = ss * ss
            if acc[3] == 0:
                out += b"\0\0\0\0"
            else:
                c = acc[3]
                out += bytes((round(acc[0] / c * 255), round(acc[1] / c * 255), round(acc[2] / c * 255), round(c / m * 255)))
    return bytes(out)


def write_png(path, size, raw):
    def chunk(tag, data):
        return struct.pack(">I", len(data)) + tag + data + struct.pack(">I", zlib.crc32(tag + data))

    ihdr = struct.pack(">IIBBBBB", size, size, 8, 6, 0, 0, 0)
    with open(path, "wb") as f:
        f.write(b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", ihdr) + chunk(b"IDAT", zlib.compress(raw, 9)) + chunk(b"IEND", b""))


def write_ico(path, pngs):
    blobs = [open(p, "rb").read() for p in pngs]
    header = struct.pack("<HHH", 0, 1, len(blobs))
    offset = 6 + 16 * len(blobs)
    entries, data = b"", b""
    for blob in blobs:
        w, h = struct.unpack(">II", blob[16:24])
        entries += struct.pack("<BBBBHHII", w % 256, h % 256, 0, 0, 1, 32, len(blob), offset + len(data))
        data += blob
    with open(path, "wb") as f:
        f.write(header + entries + data)


if __name__ == "__main__":
    mode = sys.argv[1] if len(sys.argv) > 1 else ""
    if mode == "png" and len(sys.argv) >= 3:
        size = int(sys.argv[3]) if len(sys.argv) > 3 else 1024
        write_png(sys.argv[2], size, render(size))
    elif mode == "ico" and len(sys.argv) >= 4:
        write_ico(sys.argv[2], sys.argv[3:])
    else:
        sys.exit(__doc__)
