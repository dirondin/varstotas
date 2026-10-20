#!/usr/bin/env bash
# Cross-compiles a Windows x86_64 release from macOS/Linux using the GNU (mingw-w64)
# toolchain and packs dist/varstotas-windows-x86_64.zip (varstotas.exe with embedded icon).
#
# Usage: scripts/build-windows.sh
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

TARGET=x86_64-pc-windows-gnu
PREFIX=x86_64-w64-mingw32
DIST="$ROOT/dist"
ZIP="$DIST/varstotas-windows-x86_64.zip"

# 1. Toolchain: rustup target + mingw-w64 (linker, windres).
rustup target list --installed | grep -qx "$TARGET" || rustup target add "$TARGET"
if ! command -v "$PREFIX-gcc" >/dev/null; then
  if command -v brew >/dev/null; then
    brew install mingw-w64
  else
    echo "error: $PREFIX-gcc not found; install mingw-w64" >&2
    exit 1
  fi
fi

# 2. Icon (.ico with PNG entries) for the resource embedded by crates/varstotas_app/build.rs.
WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT
echo "==> icon"
MASTER="$ROOT/crates/varstotas_app/assets/icon.png"
if command -v sips >/dev/null; then
  sips -z 256 256 "$MASTER" --out "$WORK/icon_256.png" >/dev/null
else
  python3 "$ROOT/scripts/make_icon.py" png "$WORK/icon_256.png" 256
fi
PNGS=()
for s in 16 24 32 48 64 128; do
  if command -v sips >/dev/null; then
    sips -z "$s" "$s" "$WORK/icon_256.png" --out "$WORK/icon_$s.png" >/dev/null
  else
    python3 "$ROOT/scripts/make_icon.py" png "$WORK/icon_$s.png" "$s"
  fi
  PNGS+=("$WORK/icon_$s.png")
done
PNGS+=("$WORK/icon_256.png")
python3 "$ROOT/scripts/make_icon.py" ico "$WORK/varstotas.ico" "${PNGS[@]}"

# 3. Build.
export CARGO_TARGET_X86_64_PC_WINDOWS_GNU_LINKER="$PREFIX-gcc"
export CC_x86_64_pc_windows_gnu="$PREFIX-gcc"
export AR_x86_64_pc_windows_gnu="$PREFIX-ar"
export VARSTOTAS_WIN_ICON="$WORK/varstotas.ico"
echo "==> cargo build --release --target $TARGET"
cargo build --release -p varstotas_app --bin varstotas --target "$TARGET"

# 4. Package.
EXE="target/$TARGET/release/varstotas.exe"
mkdir -p "$DIST"
rm -f "$ZIP"
STAGE="$WORK/stage"
mkdir -p "$STAGE"
cp "$EXE" "$STAGE/varstotas.exe"
cp "$ROOT/LICENSE" "$STAGE/LICENSE.txt"
cp "$ROOT/THIRD-PARTY-LICENSES.txt" "$STAGE/"
(cd "$STAGE" && zip -q -9 "$ZIP" varstotas.exe LICENSE.txt THIRD-PARTY-LICENSES.txt)

echo "==> done"
file "$EXE"
ls -lh "$EXE" "$ZIP"
