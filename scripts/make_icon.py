#!/usr/bin/env python3
"""Programmatic Varstotas app icon (stdlib only).

The corner of a workbench (top, leg, aprons, stretchers) seen from above-front, cut in half
by a vertical plane: the left half is wood, the right half translucent bluish faces, all with
black edges. Behind it the model axes run from a centre point; the background is the editor's
sky / ground gradient in a rounded square.

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
BOXES = [
    ((0.0, 0.0, 9.0), (9.5, 9.5, 10.8), 0),  # top
    ((0.5, 0.5, 0.0), (2.5, 2.5, 9.0), 2),  # leg
    ((2.5, 0.9, 7.0), (9.5, 1.9, 9.0), 0),  # apron along X
    ((0.9, 2.5, 7.0), (1.9, 9.5, 9.0), 1),  # apron along Y
    ((2.5, 1.0, 1.4), (9.5, 2.0, 2.8), 0),  # stretcher along X
    ((1.0, 2.5, 1.4), (2.0, 9.5, 2.8), 1),  # stretcher along Y
]
# Axes from this point (behind the bench, on the floor) in +X / +Y / +Z.
AXIS_ORIGIN = (6.0, 6.0, 0.0)
AXIS_LEN = 40.0
AXIS_COLORS = [(0.85, 0.0, 0.0), (0.0, 0.62, 0.0), (0.0, 0.0, 0.9)]

EYE_DIR = (-1.0, -0.82, 0.78)  # from the scene towards the viewer
TARGET = (4.6, 4.6, 5.0)
SCALE = 0.046  # model units -> icon units
OFFSET = (0.5, 0.52)  # where TARGET lands on the icon
LIGHT = (-0.45, -0.75, 0.95)

# Half on the viewer's left (x < y side of this plane) is wood, the rest translucent.
def is_wood(p):
    return p[0] - p[1] < 0.0


WOOD_LIGHT, WOOD_DARK = (0.86, 0.66, 0.42), (0.62, 0.40, 0.20)
GLASS = (0.36, 0.56, 0.95)
GLASS_ALPHA = 0.42
EDGE_W = 0.0062  # icon units
EDGE_COL = (0.05, 0.06, 0.08)

SKY_TOP, SKY_HORIZON, GROUND = (0.50, 0.70, 0.92), (0.91, 0.94, 0.97), (0.78, 0.75, 0.69)
GROUND_FAR = (0.62, 0.58, 0.52)
HORIZON = 0.58
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


def project(p):
    """Model point -> (x, y, depth) in icon units (y down, larger depth = farther)."""
    d = sub(p, TARGET)
    return (OFFSET[0] + dot(d, RIGHT) * SCALE, OFFSET[1] - dot(d, UP) * SCALE, -dot(d, FWD))


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
        self.opaque = [None] * (size * size)  # (depth, rgb)
        self.layers = {}  # pixel -> [(depth, rgb, alpha)]

    def frag(self, i, depth, rgb, alpha):
        if alpha >= 1.0:
            o = self.opaque[i]
            if o is None or depth < o[0]:
                self.opaque[i] = (depth, rgb)
        else:
            self.layers.setdefault(i, []).append((depth, rgb, alpha))

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
                # orthographic: model point and depth interpolate linearly
                p = tuple(w0 * pa[k] + w1 * pb[k] + w2 * pc[k] for k in range(3))
                depth = w0 * a[2] + w1 * b[2] + w2 * c[2]
                rgb, alpha = shade(p)
                self.frag(py * n + px, depth, rgb, alpha)

    def segment(self, pa, pb, width, color_at, bias):
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
                o = self.opaque[i]
                if o is not None and depth > o[0]:
                    continue
                self.layers.setdefault(i, []).append((depth, rgb, alpha * cov))


def background(x, y):
    if y < HORIZON:
        t = (y - INSET) / (HORIZON - INSET)
        t = max(0.0, min(1.0, t)) ** 1.3
        return tuple(SKY_TOP[i] + (SKY_HORIZON[i] - SKY_TOP[i]) * t for i in range(3))
    t = max(0.0, min(1.0, (y - HORIZON) / (1.0 - INSET - HORIZON)))
    return tuple(GROUND_FAR[i] + (GROUND[i] - GROUND_FAR[i]) * t ** 0.7 for i in range(3))


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


def draw(n):
    cv = Canvas(n)
    for lo, hi, grain in BOXES:
        for quad, nrm in box_faces(lo, hi):
            back = dot(nrm, FWD) <= 0

            def shade(p, nrm=nrm, grain=grain, back=back):
                if is_wood(p):
                    if back:
                        return (0, 0, 0), 0.0
                    return lit(wood(p, grain), nrm, 0.55), 1.0
                g = lit(GLASS, nrm if not back else tuple(-c for c in nrm), 0.6)
                return g, GLASS_ALPHA * (0.55 if back else 1.0)

            pts = [(q, project(q)) for q in quad]
            cv.triangle((pts[0], pts[1], pts[2]), shade)
            cv.triangle((pts[0], pts[2], pts[3]), shade)
    # edges: wood edges only where visible (depth test), glass edges all of them
    for lo, hi, _ in BOXES:
        for a, b in box_edges(lo, hi):
            cv.segment(a, b, EDGE_W, lambda p: (EDGE_COL, 1.0 if is_wood(p) else 0.85), 0.05)
    # axes behind the bench: they only show where nothing opaque covers them
    for axis, col in enumerate(AXIS_COLORS):
        end = list(AXIS_ORIGIN)
        end[axis] += AXIS_LEN
        cv.segment(AXIS_ORIGIN, tuple(end), EDGE_W * 1.15, lambda p, col=col: (col, 0.95), 0.0)
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
                    o = cv.opaque[i]
                    col = o[1] if o else background(x, y)
                    for d, rgb, a in sorted(cv.layers.get(i, ()), key=lambda f: -f[0]):
                        if o is None or d <= o[0]:
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
