#!/bin/sh
# Assembles the package-test kit (what Release build puts in its release-files artifact as
# release-tests/, beside the packages): the runners, scripts.txt and the UI scripts with the
# models and fixtures they use, laid out as in the repository (tests/ui/...).
# usage: tests/release/kit.sh <destination folder>
set -eu
here=$(cd "$(dirname "$0")" && pwd)
root=$(cd "$here/../.." && pwd)
dest=$1
mkdir -p "$dest/tests/ui"
cp "$here"/run.sh "$here"/run.ps1 "$here"/run.cmd "$here"/sandbox.ps1 "$here"/sandbox.cmd "$here"/scripts.txt "$here"/README.md "$dest/"
chmod +x "$dest/run.sh"
cp -R "$root/tests/ui/scripts" "$root/tests/ui/models" "$root/tests/ui/fixtures" "$root/tests/ui/data" "$dest/tests/ui/"
