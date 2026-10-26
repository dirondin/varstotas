#!/usr/bin/env bash
# Builds a universal (arm64 + x86_64) macOS release bundle: dist/Varstotas.app,
# plus dist/Varstotas-<version>-macos-universal.zip and .dmg.
#
# Usage: scripts/build-macos.sh [--no-dmg] [--debug]
# --debug builds a local target/Varstotas.app without universal slices or archives.
# VARSTOTAS_VERSION overrides the version taken from Cargo (CI passes the release tag's).
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
TARGET_DIR="${CARGO_TARGET_DIR:-target}"

MAKE_DMG=1
DEBUG_BUILD=0
for arg in "$@"; do
  case "$arg" in
    --no-dmg) MAKE_DMG=0 ;;
    --debug) DEBUG_BUILD=1; MAKE_DMG=0 ;;
    *) echo "Unknown option: $arg" >&2; exit 1 ;;
  esac
done

TARGETS=(aarch64-apple-darwin x86_64-apple-darwin)
BIN=varstotas
APP_NAME=Varstotas
BUNDLE_ID=dev.varstotas.app
MIN_MACOS=11.0
DIST="$ROOT/dist"
APP="$DIST/$APP_NAME.app"
[[ "$DEBUG_BUILD" == 1 ]] && APP="$ROOT/target/$APP_NAME.app"

VERSION="${VARSTOTAS_VERSION:-}"
[[ -n "$VERSION" ]] || VERSION="$(cargo metadata --no-deps --format-version 1 \
  | python3 -c 'import json,sys; print(next(p["version"] for p in json.load(sys.stdin)["packages"] if p["name"]=="varstotas_app"))')"
echo "==> Varstotas $VERSION"
# Bundle versions are numeric: 0.2.0-rc.1 → 0.2.0
NUM_VERSION="$(sed -nE 's/^([0-9]+(\.[0-9]+){0,2}).*/\1/p' <<<"$VERSION")"
# CFBundleVersion is the CI build number (the same one shown in About)
BUILD_NUMBER="${VARSTOTAS_BUILD:-${GITHUB_RUN_NUMBER:-}}"

# 1–2. A local debug build, or release builds for both architectures.
if [[ "$DEBUG_BUILD" == 1 ]]; then
  cargo build --offline -p varstotas_app --bin "$BIN"
else
  installed="$(rustup target list --installed)"
  for t in "${TARGETS[@]}"; do
    grep -qx "$t" <<<"$installed" || rustup target add "$t"
  done

  # 2. Release builds per arch.
  export MACOSX_DEPLOYMENT_TARGET="$MIN_MACOS"
  for t in "${TARGETS[@]}"; do
    echo "==> cargo build --release --target $t"
    cargo build --release -p varstotas_app --bin "$BIN" --target "$t"
  done
fi

# 3. Bundle skeleton.
rm -rf "$APP"
mkdir -p "$APP/Contents/MacOS" "$APP/Contents/Resources"
if [[ "$DEBUG_BUILD" == 1 ]]; then
  cp "$TARGET_DIR/debug/$BIN" "$APP/Contents/MacOS/$BIN"
else
  SLICES=()
  for t in "${TARGETS[@]}"; do SLICES+=("$TARGET_DIR/$t/release/$BIN"); done
  lipo -create -output "$APP/Contents/MacOS/$BIN" "${SLICES[@]}"
fi
chmod +x "$APP/Contents/MacOS/$BIN"

# 4. Icon (.icns via iconutil from the master PNG drawn by scripts/make_icon.py).
WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT
echo "==> icon"
cp "$ROOT/crates/varstotas_app/assets/icon.png" "$WORK/icon_1024.png"
ICONSET="$WORK/Varstotas.iconset"
mkdir -p "$ICONSET"
for s in 16 32 128 256 512; do
  sips -z "$s" "$s" "$WORK/icon_1024.png" --out "$ICONSET/icon_${s}x${s}.png" >/dev/null
  d=$((s * 2))
  sips -z "$d" "$d" "$WORK/icon_1024.png" --out "$ICONSET/icon_${s}x${s}@2x.png" >/dev/null
done
iconutil -c icns "$ICONSET" -o "$APP/Contents/Resources/Varstotas.icns"
cp "$ROOT/LICENSE" "$ROOT/THIRD-PARTY-LICENSES.txt" "$APP/Contents/Resources/"

# 5. Info.plist.
cat >"$APP/Contents/Info.plist" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleName</key><string>$APP_NAME</string>
  <key>CFBundleDisplayName</key><string>$APP_NAME</string>
  <key>CFBundleIdentifier</key><string>$BUNDLE_ID</string>
  <key>CFBundleExecutable</key><string>$BIN</string>
  <key>CFBundleIconFile</key><string>Varstotas</string>
  <key>CFBundlePackageType</key><string>APPL</string>
  <key>CFBundleInfoDictionaryVersion</key><string>6.0</string>
  <key>CFBundleVersion</key><string>${BUILD_NUMBER:-${NUM_VERSION:-0.0.0}}</string>
  <key>CFBundleShortVersionString</key><string>${NUM_VERSION:-0.0.0}</string>
  <key>LSMinimumSystemVersion</key><string>$MIN_MACOS</string>
  <key>LSApplicationCategoryType</key><string>public.app-category.graphics-design</string>
  <key>NSHighResolutionCapable</key><true/>
  <key>NSSupportsAutomaticGraphicsSwitching</key><true/>
  <key>NSPrincipalClass</key><string>NSApplication</string>
</dict>
</plist>
EOF
plutil -lint "$APP/Contents/Info.plist" >/dev/null

# 6. Ad-hoc signature (no Developer ID; Gatekeeper will still ask on first open).
codesign --force --deep -s - "$APP"
codesign --verify --deep --strict "$APP"

if [[ "$DEBUG_BUILD" == 1 ]]; then
  echo "==> done: $APP"
  exit 0
fi

# 7. Archives.
ZIP="$DIST/$APP_NAME-$VERSION-macos-universal.zip"
rm -f "$ZIP"
ditto -c -k --keepParent "$APP" "$ZIP"
if [[ $MAKE_DMG == 1 ]]; then
  DMG="$DIST/$APP_NAME-$VERSION-macos-universal.dmg"
  rm -f "$DMG"
  STAGE="$WORK/dmg"
  mkdir -p "$STAGE"
  cp -R "$APP" "$STAGE/"
  cp "$ROOT/LICENSE" "$ROOT/THIRD-PARTY-LICENSES.txt" "$STAGE/"
  ln -s /Applications "$STAGE/Applications"
  # An explicit size: hdiutil's own estimate is sometimes too small on CI runners ("No space left
  # on device"); and it now and then fails with "Resource busy": try again.
  SIZE_MB=$(( $(du -sm "$STAGE" | cut -f1) * 3 / 2 + 32 ))
  for attempt in 1 2 3; do
    hdiutil create -volname "$APP_NAME" -srcfolder "$STAGE" -fs HFS+ -size "${SIZE_MB}m" -ov -format UDZO "$DMG" >/dev/null && break
    [[ $attempt == 3 ]] && exit 1
    sleep 5
  done
fi

echo "==> done"
lipo -info "$APP/Contents/MacOS/$BIN"
ls -lh "$DIST"
