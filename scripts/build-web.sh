#!/bin/sh
# The web build: target/web/ is the static site: the home page (web/index.html, site.css, site.js,
# i18n/, img/ and the README pictures from docs/images) and the app in app/ (web/app/: index.html,
# manifest.webmanifest, icons/, sw.js — the offline worker, filled in here —,
# varstotas.js, varstotas_bg.wasm).
# usage: scripts/build-web.sh [--dev] [--beta | --site]
#   --dev   a debug build
#   --beta  the beta in target/web-beta/ (the app only: put it at beta/app/): Varstotas Beta, its
#           preferences and data folder apart from the release's (VARSTOTAS_BETA)
#   --site  only the home page (the app built before is kept)
# The home page's buttons give WEB_VERSION and BETA_VERSION, by default the version in Cargo.toml
# (the site of a release gives the release's for the app and master's for the beta: pages.yml).
# The app needs the wasm32-unknown-unknown target and wasm-bindgen-cli of the version in Cargo.lock; wasm-opt (binaryen) is used when installed.
set -e
cd "$(dirname "$0")/.."
profile=web
flag="--profile web"
if [ "$1" = "--dev" ]; then profile=debug; flag=; shift; fi
out=target/web
beta=
if [ "$1" = "--beta" ]; then beta=1; out=target/web-beta; fi
cargo_version=$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -1)
site() {
    rm -rf "$out/i18n" "$out/img"
    mkdir -p "$out/img"
    cp -R web/site.css web/site.js web/i18n "$out/"
    # the versions on the buttons
    sed -e "s/__WEB_VERSION__/${WEB_VERSION:-$cargo_version}/" -e "s/__BETA_VERSION__/${BETA_VERSION:-$cargo_version}/" web/index.html > "$out/index.html"
    cp web/img/* "$out/img/"
    (cd docs/images && cp workbench.jpg desk-caddy.jpg gallery.jpg cut-diagram.png assembly.gif assembly-booklet.png "../../$out/img/")
}
if [ "$1" = "--site" ]; then site; ls -l "$out"; exit; fi
if [ -n "$beta" ]; then
    VARSTOTAS_BETA=1 cargo build -p varstotas_app --bin varstotas --target wasm32-unknown-unknown $flag
else
    cargo build -p varstotas_app --bin varstotas --target wasm32-unknown-unknown $flag
fi
rm -rf "$out"
mkdir -p "$out/app"
wasm-bindgen --target web --no-typescript --out-dir "$out/app" --out-name varstotas "target/wasm32-unknown-unknown/$profile/varstotas.wasm"
if [ "$profile" = web ] && command -v wasm-opt >/dev/null; then
    wasm-opt -Oz --enable-bulk-memory --enable-nontrapping-float-to-int --enable-sign-ext --enable-mutable-globals --enable-reference-types --enable-multivalue --strip-debug --strip-producers -o "$out/app/varstotas_bg.wasm" "$out/app/varstotas_bg.wasm"
fi
cp -R web/app/index.html web/app/manifest.webmanifest web/app/icons "$out/app/"
if [ -n "$beta" ]; then
    # its own name on the page and as an installed app
    sed -i.bak -e 's|<title>Varstotas</title>|<title>Varstotas Beta</title>|' -e 's|apple-mobile-web-app-title" content="Varstotas"|apple-mobile-web-app-title" content="Varstotas Beta"|' "$out/app/index.html"
    sed -i.bak -e 's|"name": "Varstotas"|"name": "Varstotas Beta"|' -e 's|"short_name": "Varstotas"|"short_name": "Varstotas Beta"|' "$out/app/manifest.webmanifest"
    rm "$out/app/"*.bak
fi
scripts/web-worker.sh "$out/app"
if [ -z "$beta" ]; then site; fi
ls -l "$out"
