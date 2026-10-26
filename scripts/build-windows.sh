#!/usr/bin/env bash
# Builds a Windows x86_64 release and packs, in dist/:
#   Varstotas-<version>-windows-x86_64.zip        portable: varstotas.exe (embedded icon) + licences
#   Varstotas-<version>-windows-x86_64-setup.exe  installer (--native only; Inno Setup,
#                                                 packaging/windows/varstotas.iss)
#
# Usage: scripts/build-windows.sh [--native]
#   default   cross-compiles from macOS / Linux with the GNU (mingw-w64) toolchain
#   --native  builds on Windows with MSVC (Git Bash; CI); Inno Setup 6 is installed with
#             Chocolatey when missing
# VARSTOTAS_VERSION overrides the version taken from Cargo (CI passes the release tag's).
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
TARGET_DIR="${CARGO_TARGET_DIR:-target}"

NATIVE=0
for arg in "$@"; do
  case "$arg" in
    --native) NATIVE=1 ;;
    *) echo "Unknown option: $arg" >&2; exit 1 ;;
  esac
done

PY="$(command -v python3 || command -v python)"
VERSION="${VARSTOTAS_VERSION:-}"
[[ -n "$VERSION" ]] || VERSION="$(cargo metadata --no-deps --format-version 1 \
  | "$PY" -c 'import json,sys; print(next(p["version"] for p in json.load(sys.stdin)["packages"] if p["name"]=="varstotas_app"))')"
echo "==> Varstotas $VERSION"

if [[ $NATIVE == 1 ]]; then
  TARGET=x86_64-pc-windows-msvc
else
  TARGET=x86_64-pc-windows-gnu
  PREFIX=x86_64-w64-mingw32
fi
DIST="$ROOT/dist"
ZIP="$DIST/Varstotas-$VERSION-windows-x86_64.zip"
SETUP="Varstotas-$VERSION-windows-x86_64-setup"

# 1. Toolchain: rustup target (+ mingw-w64 for the linker and windres when cross-compiling).
rustup target list --installed | grep -qx "$TARGET" || rustup target add "$TARGET"
if [[ $NATIVE == 0 ]] && ! command -v "$PREFIX-gcc" >/dev/null; then
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
PNGS=()
for s in 16 24 32 48 64 128 256; do
  if command -v sips >/dev/null; then
    sips -z "$s" "$s" "$MASTER" --out "$WORK/icon_$s.png" >/dev/null
  else
    "$PY" "$ROOT/scripts/make_icon.py" png "$WORK/icon_$s.png" "$s"
  fi
  PNGS+=("$WORK/icon_$s.png")
done
"$PY" "$ROOT/scripts/make_icon.py" ico "$WORK/varstotas.ico" "${PNGS[@]}"

# 3. Build.
if [[ $NATIVE == 0 ]]; then
  export CARGO_TARGET_X86_64_PC_WINDOWS_GNU_LINKER="$PREFIX-gcc"
  export CC_x86_64_pc_windows_gnu="$PREFIX-gcc"
  export AR_x86_64_pc_windows_gnu="$PREFIX-ar"
fi
ICO="$WORK/varstotas.ico"
# Git Bash: a Windows path for the native build script
command -v cygpath >/dev/null && ICO="$(cygpath -m "$ICO")"
export VARSTOTAS_WIN_ICON="$ICO"
echo "==> cargo build --release --target $TARGET"
cargo build --release --locked -p varstotas_app --bin varstotas --target "$TARGET"

# 4. Package.
EXE="$TARGET_DIR/$TARGET/release/varstotas.exe"
mkdir -p "$DIST"
rm -f "$ZIP"
STAGE="$WORK/stage"
mkdir -p "$STAGE"
cp "$EXE" "$STAGE/varstotas.exe"
cp "$ROOT/LICENSE" "$STAGE/LICENSE.txt"
cp "$ROOT/THIRD-PARTY-LICENSES.txt" "$STAGE/"
FILES=(varstotas.exe LICENSE.txt THIRD-PARTY-LICENSES.txt)
if command -v zip >/dev/null; then
  (cd "$STAGE" && zip -q -9 "$ZIP" "${FILES[@]}")
else
  (cd "$STAGE" && 7z a -tzip -mx=9 "$ZIP" "${FILES[@]}" >/dev/null)
fi

# 5. Installer.
if [[ $NATIVE == 1 ]]; then
  win() { cygpath -w "$1"; }
  ISCC="$(command -v iscc || true)"
  for d in "/c/Program Files (x86)/Inno Setup 6" "/c/Program Files/Inno Setup 6"; do
    [[ -z "$ISCC" && -x "$d/ISCC.exe" ]] && ISCC="$d/ISCC.exe"
  done
  if [[ -z "$ISCC" ]]; then
    echo "==> installing Inno Setup"
    choco install innosetup -y --no-progress >/dev/null
    ISCC="/c/Program Files (x86)/Inno Setup 6/ISCC.exe"
  fi
  # Windows file versions are numeric: 1.2.3 + CI build 42 → 1.2.3.42 (no build → .0; anything else → 0.0.0.0)
  BUILD_NUMBER="${VARSTOTAS_BUILD:-${GITHUB_RUN_NUMBER:-0}}"
  NUM="$(sed -nE "s/^([0-9]+)\\.([0-9]+)\\.([0-9]+).*/\\1.\\2.\\3.$BUILD_NUMBER/p" <<<"$VERSION")"
  echo "==> installer"
  # MSYS_NO_PATHCONV: Git Bash would turn the /D… options into paths ("more than one script filename")
  MSYS_NO_PATHCONV=1 MSYS2_ARG_CONV_EXCL='*' "$ISCC" -Q "/DAppVersion=$VERSION" "/DNumVersion=${NUM:-0.0.0.0}" \
    "/DSourceDir=$(win "$STAGE")" "/DIconFile=$(win "$WORK/varstotas.ico")" \
    "/DOutputDir=$(win "$DIST")" "/DOutputBase=$SETUP" "$(win "$ROOT/packaging/windows/varstotas.iss")"
fi

echo "==> done"
command -v file >/dev/null && file "$EXE"
ls -lh "$EXE" "$DIST"/Varstotas-"$VERSION"-windows-*
