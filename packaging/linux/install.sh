#!/bin/sh
# Installs Varstotas for the current user (~/.local): the program, its menu entry and icon,
# and the .vrst file type. PREFIX=/usr/local sudo ./install.sh installs for everyone.
#   ./install.sh              install (or update)
#   ./install.sh --uninstall  remove
set -eu

HERE="$(cd "$(dirname "$0")" && pwd)"
PREFIX="${PREFIX:-$HOME/.local}"
BIN="$PREFIX/bin"
APPS="$PREFIX/share/applications"
ICONS="$PREFIX/share/icons/hicolor"
MIME="$PREFIX/share/mime"

refresh() {
  command -v update-mime-database >/dev/null && update-mime-database "$MIME" >/dev/null 2>&1 || true
  command -v update-desktop-database >/dev/null && update-desktop-database "$APPS" >/dev/null 2>&1 || true
  command -v gtk-update-icon-cache >/dev/null && gtk-update-icon-cache -q -t "$ICONS" >/dev/null 2>&1 || true
}

if [ "${1:-}" = "--uninstall" ]; then
  rm -f "$BIN/varstotas" "$APPS/varstotas.desktop" "$MIME/packages/varstotas.xml"
  for s in 64 128 256; do rm -f "$ICONS/${s}x${s}/apps/varstotas.png"; done
  refresh
  echo "Varstotas removed from $PREFIX"
  exit 0
fi

mkdir -p "$BIN" "$APPS" "$MIME/packages"
install -m 755 "$HERE/varstotas" "$BIN/varstotas"
for s in 64 128 256; do
  mkdir -p "$ICONS/${s}x${s}/apps"
  install -m 644 "$HERE/icons/varstotas-$s.png" "$ICONS/${s}x${s}/apps/varstotas.png"
done
sed "s|^Exec=varstotas|Exec=$BIN/varstotas|" "$HERE/varstotas.desktop" >"$APPS/varstotas.desktop"
chmod 644 "$APPS/varstotas.desktop"
install -m 644 "$HERE/varstotas-mime.xml" "$MIME/packages/varstotas.xml"
refresh
echo "Varstotas installed: $BIN/varstotas (menu: Graphics → Varstotas)"
case ":$PATH:" in *":$BIN:"*) ;; *) echo "Note: $BIN is not on PATH" ;; esac
