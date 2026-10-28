#!/usr/bin/env bash
# Builds a Linux release for this machine's architecture and packs, in dist/:
#   Varstotas-<version>-linux-<arch>.tar.gz   the program, icons, .desktop, file type and
#                                             install.sh (installs into ~/.local)
#   Varstotas-<version>-linux-<arch>.AppImage one runnable file (appimagetool is downloaded)
#   Varstotas-<version>-linux-<arch>.deb      Debian / Ubuntu package (dpkg-deb and
#                                             dpkg-shlibdeps: apt install dpkg-dev)
#
# Usage: scripts/build-linux.sh [--no-appimage] [--no-deb]
# Needs pkg-config and the Wayland / xkbcommon headers (Debian / Ubuntu:
# apt install pkg-config libwayland-dev libxkbcommon-dev); the AppImage needs network access
# for appimagetool and its runtime.
# VARSTOTAS_VERSION overrides the version taken from Cargo (CI passes the release tag's).
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
TARGET_DIR="${CARGO_TARGET_DIR:-target}"

APPIMAGE=1
DEB=1
for arg in "$@"; do
  case "$arg" in
    --no-appimage) APPIMAGE=0 ;;
    --no-deb) DEB=0 ;;
    *) echo "Unknown option: $arg" >&2; exit 1 ;;
  esac
done

VERSION="${VARSTOTAS_VERSION:-}"
[[ -n "$VERSION" ]] || VERSION="$(cargo metadata --no-deps --format-version 1 \
  | python3 -c 'import json,sys; print(next(p["version"] for p in json.load(sys.stdin)["packages"] if p["name"]=="varstotas_app"))')"
ARCH="$(uname -m)"
echo "==> Varstotas $VERSION ($ARCH)"

DIST="$ROOT/dist"
NAME="Varstotas-$VERSION-linux-$ARCH"
APPIMAGETOOL_VERSION=1.9.1

echo "==> cargo build --release"
cargo build --release --locked -p varstotas_app --bin varstotas

WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT
mkdir -p "$DIST"

# Icons from the drawing script (stdlib Python, no image tools needed).
ICONS="$WORK/icons"
mkdir -p "$ICONS"
for s in 64 128 256; do
  python3 scripts/make_icon.py png "$ICONS/varstotas-$s.png" "$s"
done

# 1. tar.gz with install.sh.
STAGE="$WORK/$NAME"
mkdir -p "$STAGE/icons"
cp "$TARGET_DIR/release/varstotas" "$STAGE/"
strip "$STAGE/varstotas" || true
cp "$ICONS"/*.png "$STAGE/icons/"
cp packaging/linux/varstotas.desktop packaging/linux/varstotas-mime.xml packaging/linux/install.sh "$STAGE/"
chmod +x "$STAGE/install.sh"
cp LICENSE THIRD-PARTY-LICENSES.txt "$STAGE/"
cat >"$STAGE/README.txt" <<README
Varstotas $VERSION for Linux ($ARCH)

Run it in place:   ./varstotas
Install for you:   ./install.sh            (into ~/.local: menu entry, icon, .vrst files)
Remove:            ./install.sh --uninstall
For everyone:      sudo PREFIX=/usr/local ./install.sh
README
TAR="$DIST/$NAME.tar.gz"
rm -f "$TAR"
tar -C "$WORK" -czf "$TAR" "$NAME"
echo "==> $TAR"

# 2. AppImage.
if [[ $APPIMAGE == 1 ]]; then
  TOOL="$TARGET_DIR/tools/appimagetool-$APPIMAGETOOL_VERSION-$ARCH.AppImage"
  if [[ ! -x "$TOOL" ]]; then
    mkdir -p "$(dirname "$TOOL")"
    curl -fsSL -o "$TOOL" "https://github.com/AppImage/appimagetool/releases/download/$APPIMAGETOOL_VERSION/appimagetool-$ARCH.AppImage"
    chmod +x "$TOOL"
  fi
  APPDIR="$WORK/Varstotas.AppDir"
  mkdir -p "$APPDIR/usr/bin" "$APPDIR/usr/share/applications" "$APPDIR/usr/share/mime/packages"
  cp "$STAGE/varstotas" "$APPDIR/usr/bin/"
  for s in 64 128 256; do
    mkdir -p "$APPDIR/usr/share/icons/hicolor/${s}x${s}/apps"
    cp "$ICONS/varstotas-$s.png" "$APPDIR/usr/share/icons/hicolor/${s}x${s}/apps/varstotas.png"
  done
  cp packaging/linux/varstotas.desktop "$APPDIR/usr/share/applications/"
  cp packaging/linux/varstotas-mime.xml "$APPDIR/usr/share/mime/packages/varstotas.xml"
  cp packaging/linux/varstotas.desktop "$APPDIR/"
  cp "$ICONS/varstotas-256.png" "$APPDIR/varstotas.png"
  ln -s varstotas.png "$APPDIR/.DirIcon"
  cat >"$APPDIR/AppRun" <<'APPRUN'
#!/bin/sh
HERE="$(dirname "$(readlink -f "$0")")"
exec "$HERE/usr/bin/varstotas" "$@"
APPRUN
  chmod +x "$APPDIR/AppRun"
  OUT="$DIST/$NAME.AppImage"
  rm -f "$OUT"
  # no FUSE on CI runners and in containers: the tool runs extracted
  APPIMAGE_EXTRACT_AND_RUN=1 ARCH="$ARCH" "$TOOL" --no-appstream "$APPDIR" "$OUT"
  chmod +x "$OUT"
  echo "==> $OUT"
fi

# 3. .deb: the same files under /usr. The mime, desktop and icon caches are refreshed by
# dpkg triggers, so the package has no maintainer scripts.
if [[ $DEB == 1 ]]; then
  case "$ARCH" in
    x86_64) DEB_ARCH=amd64 ;;
    aarch64) DEB_ARCH=arm64 ;;
    *) echo "No Debian architecture for $ARCH" >&2; exit 1 ;;
  esac
  # the tree in debian/<package>, where dpkg-shlibdeps expects the files it analyses
  PKG="$WORK/deb/debian/varstotas"
  install -Dm755 "$STAGE/varstotas" "$PKG/usr/bin/varstotas"
  install -Dm644 packaging/linux/varstotas.desktop "$PKG/usr/share/applications/varstotas.desktop"
  install -Dm644 packaging/linux/varstotas-mime.xml "$PKG/usr/share/mime/packages/varstotas.xml"
  for s in 64 128 256; do
    install -Dm644 "$ICONS/varstotas-$s.png" "$PKG/usr/share/icons/hicolor/${s}x${s}/apps/varstotas.png"
  done
  DOC="$PKG/usr/share/doc/varstotas"
  mkdir -p "$DOC"
  { echo "Format: https://www.debian.org/doc/packaging-manuals/copyright-format/1.0/"
    echo "Upstream-Name: Varstotas"
    echo "Source: https://github.com/dirondin/varstotas"
    echo
    echo "Files: *"
    echo "License: MIT"
    sed 's/^$/./; s/^/ /' LICENSE
  } >"$DOC/copyright"
  gzip -9n -c THIRD-PARTY-LICENSES.txt >"$DOC/THIRD-PARTY-LICENSES.txt.gz"
  # linked libraries from dpkg-shlibdeps; the window system, keyboard and GPU libraries are
  # loaded at run time (dlopen), so they are listed by hand, as in packaging/aur/PKGBUILD
  printf 'Source: varstotas\n\nPackage: varstotas\nArchitecture: any\n' >"$WORK/deb/debian/control"
  SHLIBS="$(cd "$WORK/deb" && dpkg-shlibdeps -O debian/varstotas/usr/bin/varstotas | sed -n 's/^shlibs:Depends=//p')"
  mkdir -p "$PKG/DEBIAN"
  # Version: a pre-release 0.3.0-rc1 becomes 0.3.0~rc1, which sorts before 0.3.0
  cat >"$PKG/DEBIAN/control" <<CONTROL
Package: varstotas
Version: ${VERSION//-/\~}
Architecture: $DEB_ARCH
Maintainer: Michail Palubisak <dirondin@users.noreply.github.com>
Installed-Size: $(du -sk "$PKG/usr" | cut -f1)
Depends: $SHLIBS, libxkbcommon0, libwayland-client0, libvulkan1 | libegl1
Recommends: libwayland-cursor0, libwayland-egl1, libx11-6, libx11-xcb1, libxcursor1, libxi6, libxrandr2, mesa-vulkan-drivers, xdg-desktop-portal
Section: graphics
Priority: optional
Homepage: https://github.com/dirondin/varstotas
Description: 3D modelling for makers
 Varstotas is a free, open-source 3D modeller for makers and hobbyists. Plan
 furniture, workshop projects and 3D prints in real millimetres, then take the
 dimensions to the workshop or the model to a slicer (STL and 3MF export).
CONTROL
  OUT="$DIST/$NAME.deb"
  rm -f "$OUT"
  dpkg-deb --root-owner-group -Zxz --build "$PKG" "$OUT"
  echo "==> $OUT"
fi

echo "==> done"
ls -lh "$DIST"/"$NAME".*
