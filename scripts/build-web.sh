#!/bin/sh
# The web build: target/web/ is a static site (index.html, varstotas.js, varstotas_bg.wasm).
# usage: scripts/build-web.sh [--dev]   — needs the wasm32-unknown-unknown target and
# wasm-bindgen-cli of the version in Cargo.lock; wasm-opt (binaryen) is used when installed.
set -e
cd "$(dirname "$0")/.."
profile=web
flag="--profile web"
if [ "$1" = "--dev" ]; then profile=debug; flag=; fi
cargo build -p varstotas_app --bin varstotas --target wasm32-unknown-unknown $flag
out=target/web
rm -rf "$out"
mkdir -p "$out"
wasm-bindgen --target web --no-typescript --out-dir "$out" --out-name varstotas "target/wasm32-unknown-unknown/$profile/varstotas.wasm"
if [ "$profile" = web ] && command -v wasm-opt >/dev/null; then
    wasm-opt -Oz --enable-bulk-memory --enable-nontrapping-float-to-int --enable-sign-ext --enable-mutable-globals --enable-reference-types --enable-multivalue --strip-debug --strip-producers -o "$out/varstotas_bg.wasm" "$out/varstotas_bg.wasm"
fi
cp web/index.html "$out/"
cp crates/varstotas_app/assets/icon.png "$out/"
ls -l "$out"
