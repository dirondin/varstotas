#!/usr/bin/env python3
"""Writes the Flatpak manifest from packaging/flatpak/ into OUT_DIR, next to it cargo-sources.json
(the crates of Cargo.lock, made by flatpak-builder-tools' flatpak-cargo-generator.py).

  scripts/flatpak-manifest.py OUT_DIR VERSION COMMIT   for Flathub: the release tag's sources
  scripts/flatpak-manifest.py OUT_DIR --local          a test build of this checkout

GENERATOR (environment): the path of flatpak-cargo-generator.py (needs aiohttp, PyYAML, tomlkit).
"""
import os
import re
import subprocess
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
APP_ID = "io.github.dirondin.Varstotas"
SOURCE = re.compile(r"      # SOURCE.*\n      - type: git\n(?:        .*\n)+")


def main():
    args = sys.argv[1:]
    if len(args) == 2 and args[1] == "--local":
        out, version, commit = args[0], None, None
    elif len(args) == 3:
        out, version, commit = args
        if not re.fullmatch(r"[0-9a-f]{40}", commit):
            sys.exit(f"COMMIT must be a full commit hash, not {commit}")
    else:
        sys.exit(__doc__.strip())
    generator = os.environ.get("GENERATOR")
    if not generator:
        sys.exit("GENERATOR is not set (the path of flatpak-cargo-generator.py)")
    with open(os.path.join(ROOT, "packaging", "flatpak", f"{APP_ID}.yml")) as f:
        text = f.read()
    if version is None:
        path = os.path.relpath(ROOT, os.path.abspath(out))
        text, n = SOURCE.subn(f"      - type: dir\n        path: {path}\n", text)
    else:
        git = (f"      - type: git\n        url: https://github.com/dirondin/varstotas.git\n"
               f"        tag: release-{version}\n        commit: {commit}\n")
        text, n = SOURCE.subn(git, text)
    if n != 1 or "@VERSION@" in text or "@COMMIT@" in text:
        sys.exit("the manifest's source block was not replaced")
    os.makedirs(out, exist_ok=True)
    manifest = os.path.join(out, f"{APP_ID}.yml")
    with open(manifest, "w") as f:
        f.write(text)
    subprocess.run([sys.executable, generator, os.path.join(ROOT, "Cargo.lock"),
                    "-o", os.path.join(out, "cargo-sources.json")], check=True)
    print(manifest)


if __name__ == "__main__":
    main()
