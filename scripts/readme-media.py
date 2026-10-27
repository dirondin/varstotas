#!/usr/bin/env python3
"""Regenerates the README media and the GitHub social preview from the running app.

Drives a debug build with UI scripts (`record` writes window frames), then encodes with
ffmpeg / ImageMagick:
  docs/images/push-pull.gif, house-orbit.gif, gallery.jpg, workbench.jpg, desk-caddy.jpg
  .github/social-preview.png
Needs macOS (like tests/ui), ffmpeg, ImageMagick 7 and curl. The app window must stay uncovered.
usage: scripts/readme-media.py
"""
import glob
import os
import shutil
import subprocess

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
OUT = os.path.join(ROOT, "target", "readme-media")
IMAGES = os.path.join(ROOT, "docs", "images")
APP = os.path.join(ROOT, "target", "debug", "varstotas")


def run(*cmd):
    subprocess.run(cmd, check=True)


def app_script(name, lines):
    path = os.path.join(OUT, f"{name}.txt")
    with open(path, "w") as f:
        f.write("\n".join(lines) + "\n")
    env = dict(os.environ, HOME=os.path.join(OUT, "home"), RUST_LOG="info", VARSTOTAS_LANG="en",
               VARSTOTAS_QA_EXAMPLES=os.path.join(ROOT, "examples"), VARSTOTAS_SCRIPT=path)
    with open(os.path.join(OUT, f"{name}.log"), "w") as log:
        subprocess.run([APP], env=env, stdout=log, stderr=subprocess.STDOUT, check=True, timeout=600)


def frames(name):
    d = os.path.join(OUT, f"frames_{name}")
    shutil.rmtree(d, ignore_errors=True)
    return d


def pushpull_script():
    """A rectangle pulled into a box, a smaller rectangle on top pushed in, then an orbit."""
    out = []

    def glide(a, b, n):
        for i in range(1, n + 1):
            t = i / n
            t = t * t * (3 - 2 * t)  # ease in / out
            p = [a[k] + (b[k] - a[k]) * t for k in range(3)]
            out.append(f"rawp move {p[0]:.0f} {p[1]:.0f} {p[2]:.0f}")

    x, y, h, d = 4600, 3500, 2500, 1000
    cx, cy = x / 2, y / 2
    out += ["action new", "wait 20", "view iso", "wait 20", "rawp move -1500 -1500 0",
            f"record {frames('pushpull')} 2", "wait 6", "tool rectangle"]
    glide((-1500, -1500, 0), (0, 0, 0), 10)
    out.append("rawp click 0 0 0")
    glide((0, 0, 0), (x, y, 0), 26)
    out += [f"rawp click {x} {y} 0", "wait 8", "tool pushpull"]
    glide((x, y, 0), (cx, cy, 0), 10)
    out.append(f"rawp click {cx} {cy} 0")
    glide((cx, cy, 0), (cx, cy, h), 26)
    out += [f"rawp click {cx} {cy} {h}", "wait 8", "tool rectangle"]
    glide((cx, cy, h), (d, d, h), 10)
    out.append(f"rawp click {d} {d} {h}")
    glide((d, d, h), (x - d, y - d, h), 22)
    out += [f"rawp click {x - d} {y - d} {h}", "wait 8", "tool pushpull"]
    glide((x - d, y - d, h), (cx, cy, h), 10)
    out.append(f"rawp click {cx} {cy} {h}")
    glide((cx, cy, h), (cx, cy, h - 2000), 22)
    out += [f"rawp click {cx} {cy} {h - 2000}", "wait 6", "tool select", f"rawp move {x + 3000} -3000 0", "wait 6"]
    out += ["orbitpx 6 0"] * 28
    out += ["wait 16", "record off", "quit"]
    return out


def gif(src_pattern, dst, vf, palettegen, paletteuse):
    """Frames → GIF with its own palette (flat UI: dithered; textured render: no dither, smaller)."""
    run("ffmpeg", "-v", "error", "-y", "-framerate", "50", "-i", src_pattern, "-vf",
        f"{vf},split[a][b];[a]palettegen={palettegen}[p];[b][p]paletteuse={paletteuse}",
        "-loop", "0", os.path.join(IMAGES, dst))


def main():
    os.makedirs(os.path.join(OUT, "home"), exist_ok=True)
    run("cargo", "build", "-q", "-p", "varstotas_app", "--manifest-path", os.path.join(ROOT, "Cargo.toml"))

    app_script("pushpull", pushpull_script())
    gif(os.path.join(OUT, "frames_pushpull", "frame_%05d.bmp"), "push-pull.gif", "fps=25,scale=960:-1:flags=lanczos",
        "max_colors=128:stats_mode=diff", "dither=bayer:bayer_scale=5:diff_mode=rectangle")

    # the house: an orbit played forth and back (a seamless loop)
    orbit = frames("orbit")
    app_script("house", ["action demo_house", "wait 60", "action scene_0", "wait 420", "movepx 520 390", "wait 4",
                         f"record {orbit} 2"] + ["orbitpx -3 0"] * 46 + ["record off", "quit"])
    seq = frames("orbit_seq")
    os.makedirs(seq)
    count = len(os.listdir(orbit))
    order = list(range(count)) + list(range(count - 2, 0, -1))
    for n, i in enumerate(order):
        os.link(os.path.join(orbit, f"frame_{i:05d}.bmp"), os.path.join(seq, f"f_{n:05d}.bmp"))
    gif(os.path.join(seq, "f_%05d.bmp"), "house-orbit.gif",
        "fps=12,crop=1108:666:0:197,scale=640:-1:flags=lanczos", "max_colors=96", "dither=none")
    # the material gallery in its Studio scene (Realistic style, given time to settle)
    gallery = frames("gallery")
    app_script("gallery", ["open " + os.path.join(ROOT, "examples/demos/material_gallery.vrst"), "wait 60",
                           "action scene_0", "wait 420", f"record {gallery} 2", "wait 1", "record off", "quit"])
    run("magick", os.path.join(gallery, "frame_00000.bmp"), "-quality", "86", os.path.join(IMAGES, "gallery.jpg"))
    # hobby projects with their main dimensions: stills, then the workbench shifted right for the social preview
    # (one recorded frame each: `screenshot` can miss while the demo's thumbnails render)
    shots = {name: frames(name) for name in MAKER}
    bench = frames("workbench_preview")
    lines = []
    for name, dims in MAKER.items():
        view, zoom = VIEW.get(name, ([], 0.9))
        lines += [f"action demo {name}", "wait 300", dimension_script(dims), "wait 10", *view, f"zoompx {zoom} 520 420",
                  "wait 20", f"record {shots[name]} 2", "wait 1", "record off"]
    lines += ["action demo workbench", "wait 300", dimension_script(MAKER["workbench"]), "wait 10", "zoompx 0.75 520 420",
              "wait 4", "rawp move 800 400 450", "panpx 150 -40", "wait 20", f"record {bench} 2", "wait 1", "record off",
              "quit"]
    app_script("maker", lines)
    for name, d in shots.items():
        run("magick", os.path.join(d, "frame_00000.bmp"), "-quality", "86",
            os.path.join(IMAGES, f"{name.replace('_', '-')}.jpg"))
    social_preview(os.path.join(bench, "frame_00000.bmp"))


# demo → dimensions (from, to, offset vector) in world millimetres, on the sides facing the demo's camera
MAKER = {
    "workbench": [((0, 0, 895), (1600, 0, 895), (0, 0, 250)), ((1600, 0, 895), (1600, 800, 895), (0, 0, 250)),
                  ((1600, 800, 0), (1600, 800, 895), (250, 0, 0))],
    "desk_caddy": [((0, 0, 0), (140, 0, 0), (0, -25, 0)), ((140, 0, 0), (140, 90, 0), (25, 0, 0)),
                   ((0, 0, 0), (0, 0, 80), (-25, 0, 0))],
}
# demos saved without a camera: the view to take, and the zoom after it
VIEW = {"desk_caddy": (["action console_close", "view iso", "zoom_extents"], 1.15)}


def dimension_script(dims):
    """A UI-script line adding the dimensions through the Rhai `dimension(a, b, offset)`."""
    return "action script " + "; ".join(f"dimension({list(a)}, {list(b)}, {list(o)})" for a, b, o in dims) + ";"


def social_preview(render):
    """1280×640: the dimensioned workbench full-bleed, darkened to the left under the icon, name and
    tagline (Ubuntu Light, egui's bundled font, Ubuntu Font Licence); text inside the 80 px safe area."""
    font = sorted(glob.glob(os.path.expanduser("~/.cargo/registry/src/*/epaint_default_fonts-*/fonts/Ubuntu-Light.ttf")))[-1]
    # the icon without its transparent margin, 120 px, centred on the height of the title's capital V
    icon = os.path.join(OUT, "icon120.png")
    run("magick", os.path.join(ROOT, "crates/varstotas_app/assets/icon.png"), "-trim", "+repage", "-resize", "120x120", icon)
    run("magick", render, "-resize", "1400x900", "-crop", "1110x555+0+180", "+repage", "-resize", "1280x640^",
        "-gravity", "center", "-extent", "1280x640",
        "(", "-size", "640x1280", "gradient:rgba(12,18,28,0.92)-rgba(12,18,28,0)", "-rotate", "90", "-flop",
        "-gravity", "west", "-background", "none", "-extent", "1280x640", ")", "-gravity", "center", "-composite",
        icon, "-gravity", "northwest", "-geometry", "+88+175", "-composite",
        "-font", font, "-fill", "white", "-pointsize", "96", "-annotate", "+232+178", "Varstotas",
        "-pointsize", "36", "-annotate", "+88+330", "3D modelling for makers",
        "-pointsize", "28", "-fill", "rgba(255,255,255,0.85)",
        "-annotate", "+88+392", "Furniture, workshop projects, 3D prints",
        "-annotate", "+88+434", "Free & open source · macOS · Windows · Linux",
        "-strip", "PNG24:" + os.path.join(ROOT, ".github", "social-preview.png"))


if __name__ == "__main__":
    main()
