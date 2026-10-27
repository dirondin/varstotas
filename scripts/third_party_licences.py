#!/usr/bin/env python3
"""Collect the licences of the crates Varstotas ships (Help → Licences…, the bundles).

usage: python3 scripts/third_party_licences.py
writes crates/varstotas_app/licences/third_party.json (embedded in the app) and
THIRD-PARTY-LICENSES.txt (copied next to the app by the build scripts).

The crates are those `cargo tree` lists for varstotas_app on every target that is released (macOS, Windows, Linux)
(normal and build dependencies). Each crate's licence files (LICENSE*, LICENCE*, COPYING*,
NOTICE*, UNLICENSE*, and the font licences of epaint_default_fonts) are taken from its
source; a crate without such files gets the standard text of its licence, with its authors
as the copyright holders for MIT-style licences.
"""
import json
import os
import re
import subprocess
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
TARGETS = [
    "aarch64-apple-darwin", "x86_64-apple-darwin",
    "x86_64-pc-windows-msvc", "aarch64-pc-windows-msvc",
    "x86_64-unknown-linux-gnu", "aarch64-unknown-linux-gnu",
]
NAMES = re.compile(r"^(licen[cs]e|copying|notice|unlicense|ofl|ufl)|license\.txt$|-license\.txt$", re.I)

MIT = """Permission is hereby granted, free of charge, to any person obtaining a copy of this
software and associated documentation files (the "Software"), to deal in the Software
without restriction, including without limitation the rights to use, copy, modify, merge,
publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons
to whom the Software is furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all copies or
substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED,
INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR
PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE
FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR
OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
DEALINGS IN THE SOFTWARE.
"""


def run(cmd):
    return subprocess.run(cmd, cwd=ROOT, check=True, capture_output=True, text=True).stdout


def shipped():
    out = set()
    for t in TARGETS:
        tree = run(["cargo", "tree", "-p", "varstotas_app", "-e", "normal,build", "--target", t, "--prefix", "none", "-f", "{p}"])
        for line in tree.splitlines():
            parts = line.split()
            if len(parts) >= 2 and parts[1].startswith("v"):
                out.add((parts[0], parts[1][1:]))
    return out


def licence_files(d):
    found = []
    for sub in ["", "fonts", "LICENSES", "licenses"]:
        p = os.path.join(d, sub)
        if not os.path.isdir(p):
            continue
        for f in sorted(os.listdir(p)):
            fp = os.path.join(p, f)
            if os.path.isfile(fp) and NAMES.search(f) and not f.endswith((".rs", ".toml")):
                found.append(fp)
    return found


def main():
    meta = json.loads(run(["cargo", "metadata", "--format-version", "1"]))
    by_id = {(p["name"], p["version"]): p for p in meta["packages"]}
    want = sorted(shipped(), key=lambda x: (x[0].lower(), x[1]))
    texts, index, crates = [], {}, []
    apache = None
    for name, ver in want:
        p = by_id.get((name, ver))
        if p is None or p["name"].startswith("varstotas"):
            continue
        for f in licence_files(os.path.dirname(p["manifest_path"])):
            if "APACHE" in os.path.basename(f).upper():
                apache = apache or open(f, encoding="utf-8", errors="replace").read().strip()
    for name, ver in want:
        p = by_id.get((name, ver))
        if p is None or p["name"].startswith("varstotas"):
            continue
        lic = p.get("license") or ("see " + os.path.basename(p["license_file"]) if p.get("license_file") else "unknown")
        files = licence_files(os.path.dirname(p["manifest_path"]))
        bodies = []
        for f in files:
            t = open(f, encoding="utf-8", errors="replace").read().strip()
            if t:
                bodies.append((os.path.basename(f), t))
        if not bodies:
            holders = ", ".join(a.split("<")[0].strip() for a in p.get("authors", [])) or f"the {name} authors"
            if "MIT" in lic:
                bodies.append(("MIT", f"Copyright (c) {holders}\n\n{MIT}"))
            if "Apache-2.0" in lic and apache:
                bodies.append(("Apache-2.0", apache))
            if not bodies:
                bodies.append((lic, f"Licensed under {lic}. Copyright (c) {holders}."))
        ids = []
        for fname, t in bodies:
            if t not in index:
                index[t] = len(texts)
                texts.append(t)
            ids.append([fname, index[t]])
        crates.append({"name": name, "version": ver, "license": lic, "repository": p.get("repository") or "", "texts": ids})
    out_dir = os.path.join(ROOT, "crates", "varstotas_app", "licences")
    os.makedirs(out_dir, exist_ok=True)
    with open(os.path.join(out_dir, "third_party.json"), "w", encoding="utf-8") as f:
        json.dump({"crates": crates, "texts": texts}, f, ensure_ascii=False, separators=(",", ":"))
    with open(os.path.join(ROOT, "THIRD-PARTY-LICENSES.txt"), "w", encoding="utf-8") as f:
        f.write("Varstotas uses the following open-source components.\n\n")
        for c in crates:
            repo = f"  {c['repository']}" if c["repository"] else ""
            f.write(f"{c['name']} {c['version']} — {c['license']}{repo}\n")
        f.write("\n\nThe licence texts, each with the components it applies to.\n\n")
        for i, t in enumerate(texts):
            users = sorted({f"{c['name']} {c['version']}" for c in crates if any(j == i for _, j in c["texts"])})
            f.write("=" * 78 + "\n")
            f.write("Applies to: " + ", ".join(users) + "\n\n")
            f.write(t + "\n\n")
    print(f"{len(crates)} crates, {len(texts)} distinct licence texts", file=sys.stderr)


if __name__ == "__main__":
    main()
