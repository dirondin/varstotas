#!/usr/bin/env python3
"""Programmatic Varstotas app icon (stdlib only).

Draws a blue rounded square with an isometric "push/pull" box and writes:
  make_icon.py png  <out.png> [size]     -- master PNG (default 1024)
  make_icon.py ico  <out.ico> <png>...   -- pack PNG files into a Windows .ico
"""
import struct
import sys
import zlib

# Isometric box (unit coords, y down).
T, UL, UR, C = (0.5, 0.22), (0.24, 0.37), (0.76, 0.37), (0.5, 0.52)
LL, LR, B = (0.24, 0.69), (0.76, 0.69), (0.5, 0.84)
FACES = [
    ((T, UR, C, UL), (0xF7, 0xF9, 0xFC)),  # top
    ((UL, C, B, LL), (0xCF, 0xD8, 0xE3)),  # left
    ((C, UR, LR, B), (0x9C, 0xAB, 0xBD)),  # right
]
EDGES = [(T, UR), (UR, LR), (LR, B), (B, LL), (LL, UL), (UL, T), (UL, C), (C, UR), (C, B)]
EDGE_W = 0.011
EDGE_COL = (0x1F, 0x29, 0x37)
BG_TOP, BG_BOT = (0x4A, 0x9A, 0xF0), (0x1D, 0x58, 0xB5)
INSET, RADIUS = 0.085, 0.19


def in_poly(x, y, poly):
    inside = False
    n = len(poly)
    for i in range(n):
        x1, y1 = poly[i]
        x2, y2 = poly[(i + 1) % n]
        if (y1 > y) != (y2 > y) and x < (x2 - x1) * (y - y1) / (y2 - y1) + x1:
            inside = not inside
    return inside


def seg_dist(x, y, a, b):
    ax, ay = a
    dx, dy = b[0] - ax, b[1] - ay
    t = max(0.0, min(1.0, ((x - ax) * dx + (y - ay) * dy) / (dx * dx + dy * dy)))
    px, py = ax + t * dx - x, ay + t * dy - y
    return (px * px + py * py) ** 0.5


def in_round_rect(x, y):
    lo, hi = INSET, 1.0 - INSET
    if x < lo or x > hi or y < lo or y > hi:
        return False
    cx = min(max(x, lo + RADIUS), hi - RADIUS)
    cy = min(max(y, lo + RADIUS), hi - RADIUS)
    return (x - cx) ** 2 + (y - cy) ** 2 <= RADIUS * RADIUS


def sample(x, y):
    """Returns RGBA for one sample point."""
    if not in_round_rect(x, y):
        return (0, 0, 0, 0)
    if 0.2 < x < 0.8 and 0.18 < y < 0.88:
        if any(seg_dist(x, y, a, b) < EDGE_W for a, b in EDGES):
            return EDGE_COL + (255,)
        for poly, col in FACES:
            if in_poly(x, y, poly):
                return col + (255,)
    t = (y - INSET) / (1 - 2 * INSET)
    return tuple(round(BG_TOP[i] + (BG_BOT[i] - BG_TOP[i]) * t) for i in range(3)) + (255,)


def render(size, ss=2):
    rows = []
    offs = [(i + 0.5) / ss for i in range(ss)]
    n = ss * ss
    for py in range(size):
        row = bytearray([0])  # PNG filter: none
        for px in range(size):
            acc = [0, 0, 0, 0]
            for oy in offs:
                for ox in offs:
                    r, g, b, a = sample((px + ox) / size, (py + oy) / size)
                    acc[0] += r * a
                    acc[1] += g * a
                    acc[2] += b * a
                    acc[3] += a
            a = acc[3]
            if a == 0:
                row += b"\0\0\0\0"
            else:
                row += bytes((acc[0] // a, acc[1] // a, acc[2] // a, a // n))
        rows.append(bytes(row))
    return b"".join(rows)


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
