#!/bin/sh
# The web build: target/web/ is the static site: the home page (web/index.html, site.css, site.js,
# i18n/, img/ and the README pictures from docs/images) and the app in app/ (web/app/: index.html,
# manifest.webmanifest, icons/, sw.js — the offline worker, filled in here —,
# varstotas.js, varstotas_bg.wasm).
# usage: scripts/build-web.sh [--dev | --site]   — --site copies only the home page (the app built
# before is kept). The app needs the wasm32-unknown-unknown target and wasm-bindgen-cli of the version in Cargo.lock; wasm-opt (binaryen) is used when installed.
set -e
cd "$(dirname "$0")/.."
profile=web
flag="--profile web"
if [ "$1" = "--dev" ]; then profile=debug; flag=; fi
out=target/web
site() {
    rm -rf "$out/i18n" "$out/img"
    mkdir -p "$out/img"
    cp -R web/index.html web/site.css web/site.js web/i18n "$out/"
    cp web/img/* "$out/img/"
    (cd docs/images && cp workbench.jpg desk-caddy.jpg gallery.jpg cut-diagram.png assembly.gif assembly-booklet.png "../../$out/img/")
}
if [ "$1" = "--site" ]; then site; ls -l "$out"; exit; fi
cargo build -p varstotas_app --bin varstotas --target wasm32-unknown-unknown $flag
rm -rf "$out"
mkdir -p "$out/app"
wasm-bindgen --target web --no-typescript --out-dir "$out/app" --out-name varstotas "target/wasm32-unknown-unknown/$profile/varstotas.wasm"
if [ "$profile" = web ] && command -v wasm-opt >/dev/null; then
    wasm-opt -Oz --enable-bulk-memory --enable-nontrapping-float-to-int --enable-sign-ext --enable-mutable-globals --enable-reference-types --enable-multivalue --strip-debug --strip-producers -o "$out/app/varstotas_bg.wasm" "$out/app/varstotas_bg.wasm"
fi
cp -R web/app/index.html web/app/manifest.webmanifest web/app/icons "$out/app/"
# the offline worker (a PWA): the app's files to cache and a hash of them, so a new build is new
files=$(cd "$out/app" && find . -type f | sort)
version=$(cd "$out/app" && echo "$files" | xargs cat | { sha256sum 2>/dev/null || shasum -a 256; } | cut -c1-16)
list=$(echo "$files" | sed 's/.*/"&"/' | tr '\n' ',' | sed 's/,$//')
sed -e "s/__VERSION__/$version/" -e "s|__FILES__|[\"./\",$list]|" web/app/sw.js > "$out/app/sw.js"
site
ls -l "$out"
