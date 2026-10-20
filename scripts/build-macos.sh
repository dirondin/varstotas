#!/usr/bin/env bash
# Builds a universal (arm64 + x86_64) macOS release bundle: dist/Varstotas.app,
# plus dist/Varstotas-<version>-macos-universal.zip and .dmg.
#
# Usage: scripts/build-macos.sh [--no-dmg]
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

MAKE_DMG=1
[[ "${1:-}" == "--no-dmg" ]] && MAKE_DMG=0

TARGETS=(aarch64-apple-darwin x86_64-apple-darwin)
BIN=varstotas
APP_NAME=Varstotas
BUNDLE_ID=dev.vrst.app
MIN_MACOS=11.0
DIST="$ROOT/dist"
APP="$DIST/$APP_NAME.app"

VERSION="$(cargo metadata --no-deps --format-version 1 \
  | python3 -c 'import json,sys; print(next(p["version"] for p in json.load(sys.stdin)["packages"] if p["name"]=="varstotas_app"))')"
echo "==> Varstotas $VERSION"

# 1. Toolchain targets.
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

# 3. Bundle skeleton.
rm -rf "$APP"
mkdir -p "$APP/Contents/MacOS" "$APP/Contents/Resources"
SLICES=()
for t in "${TARGETS[@]}"; do SLICES+=("target/$t/release/$BIN"); done
lipo -create -output "$APP/Contents/MacOS/$BIN" "${SLICES[@]}"
chmod +x "$APP/Contents/MacOS/$BIN"

# 4. Icon (.icns via iconutil from a programmatically drawn PNG).
WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT
echo "==> icon"
python3 "$ROOT/scripts/make_icon.py" png "$WORK/icon_1024.png" 1024
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
  <key>CFBundleVersion</key><string>$VERSION</string>
  <key>CFBundleShortVersionString</key><string>$VERSION</string>
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
  hdiutil create -volname "$APP_NAME" -srcfolder "$STAGE" -ov -format UDZO "$DMG" >/dev/null
fi

echo "==> done"
lipo -info "$APP/Contents/MacOS/$BIN"
ls -lh "$DIST"
