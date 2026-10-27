#!/usr/bin/env python3
"""Fills the package manager templates in packaging/ for one release.

Writes into OUT_DIR:
  scoop/varstotas.json        Scoop bucket manifest (Windows .zip, x86_64 and arm64)
  homebrew/Casks/varstotas.rb Homebrew tap cask (macOS .dmg)
  aur/PKGBUILD                AUR varstotas-bin (Linux .tar.gz, x86_64 and aarch64)
The hashes come from the release's SHA256SUMS.txt.
usage: scripts/package-manifests.py VERSION SHA256SUMS.txt OUT_DIR
"""
import os
import re
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
FILES = {  # placeholder → release file (after "Varstotas-<version>-")
    "@SHA256_WINDOWS_ZIP@": "windows-x86_64.zip",
    "@SHA256_WINDOWS_ARM_ZIP@": "windows-aarch64.zip",
    "@SHA256_MACOS_DMG@": "macos-universal.dmg",
    "@SHA256_LINUX_TAR@": "linux-x86_64.tar.gz",
    "@SHA256_LINUX_ARM_TAR@": "linux-aarch64.tar.gz",
}
TEMPLATES = {  # template in packaging/ → output path
    "scoop/varstotas.json": "scoop/varstotas.json",
    "homebrew/varstotas.rb": "homebrew/Casks/varstotas.rb",
    "aur/PKGBUILD": "aur/PKGBUILD",
}


def main():
    if len(sys.argv) != 4:
        sys.exit(__doc__.strip().splitlines()[-1])
    version, sums, out = sys.argv[1:]
    hashes = {}
    with open(sums) as f:
        for line in f:
            digest, name = line.split(maxsplit=1)
            hashes[name.strip().lstrip("*")] = digest
    values = {"@VERSION@": version}
    for key, suffix in FILES.items():
        name = f"Varstotas-{version}-{suffix}"
        if name not in hashes:
            sys.exit(f"{name} is not in {sums}")
        values[key] = hashes[name]
    for src, dst in TEMPLATES.items():
        with open(os.path.join(ROOT, "packaging", src)) as f:
            text = f.read()
        for key, value in values.items():
            text = text.replace(key, value)
        left = re.findall(r"@[A-Z0-9_]+@", text)
        if left:
            sys.exit(f"unfilled placeholders in {src}: {left}")
        path = os.path.join(out, dst)
        os.makedirs(os.path.dirname(path), exist_ok=True)
        with open(path, "w") as f:
            f.write(text)
        print(path)


if __name__ == "__main__":
    main()
