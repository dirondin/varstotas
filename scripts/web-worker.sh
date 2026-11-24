#!/bin/sh
# The web app's offline worker (a PWA): writes <app dir>/sw.js from web/app/sw.js with the app's
# files to cache and a hash of them, so a new build is new. Run by scripts/build-web.sh, and by
# pages.yml over a release's app built before (its worker then keeps to its own caches too).
# usage: scripts/web-worker.sh <app dir>
set -e
root="$(cd "$(dirname "$0")/.." && pwd)"
app="$1"
rm -f "$app/sw.js"
files=$(cd "$app" && find . -type f | sort)
version=$(cd "$app" && echo "$files" | xargs cat | { sha256sum 2>/dev/null || shasum -a 256; } | cut -c1-16)
list=$(echo "$files" | sed 's/.*/"&"/' | tr '\n' ',' | sed 's/,$//')
sed -e "s/__VERSION__/$version/" -e "s|__FILES__|[\"./\",$list]|" "$root/web/app/sw.js" > "$app/sw.js"
