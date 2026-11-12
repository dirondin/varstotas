#!/usr/bin/env bash
# Package tests of a release on macOS and Linux: every package for this system and processor
# (macOS .dmg and .zip; Linux .AppImage, .tar.gz and, with --deb, the .deb) is checked against
# SHA256SUMS.txt, unpacked or installed into a temporary folder, its version checked, started
# once as a user would (graphics chosen as on a normal start, a screenshot taken) and driven
# through the UI scripts of scripts.txt. Writes test-results/<time>-<system>-<cpu>/report.md
# and report.html beside the packages and ends with PASS or FAIL (exit code 0 or 1).
#
# usage: run.sh [options] [packages folder]   (default: the folder this kit is in, or the current one)
#   --full        the full script set on every package (default: only on the main package)
#   --quick       the quick set only
#   --only NAME   only packages whose file name contains NAME (repeatable)
#   --deb         Linux: also install the .deb with sudo apt-get, test /usr/bin/varstotas, remove it
#   --keep        keep the temporary folders (unpacked apps, script folders)
# Nothing outside the temporary folders is touched (HOME and the data folder point there), except
# the .deb with --deb. Needs a desktop session (on Linux without one: xvfb-run).
set -u

here=$(cd "$(dirname "$0")" && pwd)
# the kit (release-tests/ next to the packages) or the repository (tests/release)
if [[ -d $here/tests/ui/scripts ]]; then kit=$here; else kit=$(cd "$here/../.." && pwd); fi
set_full=0 set_quick=0 deb=0 keep=0 only=() pkgdir=
while (( $# )); do
  case $1 in
    --full) set_full=1 ;;
    --quick) set_quick=1 ;;
    --deb) deb=1 ;;
    --keep) keep=1 ;;
    --only) shift; only+=("$1") ;;
    -h|--help) sed -n '2,17p' "$0" | sed 's/^# \{0,1\}//'; exit 0 ;;
    *) pkgdir=$1 ;;
  esac
  shift
done
if [[ -z $pkgdir ]]; then
  if [[ -f $here/../SHA256SUMS.txt ]]; then pkgdir=$here/..; else pkgdir=.; fi
fi
pkgdir=$(cd "$pkgdir" && pwd) || exit 1

case $(uname -s) in
  Darwin) os=macos ;;
  Linux) os=linux ;;
  *) echo "run.sh: macOS and Linux only (Windows: run.ps1)"; exit 1 ;;
esac
case $(uname -m) in
  x86_64|amd64) cpu=x86_64 ;;
  arm64|aarch64) cpu=aarch64 ;;
  *) cpu=$(uname -m) ;;
esac

out=$pkgdir/test-results/$(date +%Y%m%d-%H%M%S)-$os-$cpu
mkdir -p "$out/logs" "$out/shots"
tmp=$(mktemp -d "${TMPDIR:-/tmp}/varstotas-release.XXXXXX") || exit 1
rows=$out/.rows   # variant<TAB>check<TAB>ok|FAIL|skip|info<TAB>detail
: > "$rows"
started=$(date +%s)

if [[ -t 1 ]]; then green=$'\e[32m' red=$'\e[31m' dim=$'\e[2m' bold=$'\e[1m' off=$'\e[0m'; else green= red= dim= bold= off=; fi

# record a result row and print it
row() {
  local variant=$1 check=$2 result=$3 detail=${4:-}
  printf '%s\t%s\t%s\t%s\n' "$variant" "$check" "$result" "${detail//$'\t'/ }" >> "$rows"
  local c=$dim
  [[ $result == ok ]] && c=$green
  [[ $result == FAIL ]] && c=$red
  printf '  %s%-4s%s %-22s %s\n' "$c" "$result" "$off" "$check" "${detail:0:150}"
}

# run with a time limit (no `timeout` on macOS): exit code, 124 when killed
limited() {
  local secs=$1; shift
  "$@" & local pid=$!
  ( sleep "$secs"; kill -TERM $pid 2>/dev/null; sleep 5; kill -KILL $pid 2>/dev/null ) >/dev/null 2>&1 &
  local watch=$!
  wait $pid; local code=$?
  if kill -0 $watch 2>/dev/null; then kill $watch 2>/dev/null; wait $watch 2>/dev/null; else code=124; fi
  return $code
}

# a display for the app on Linux
wrap=()
if [[ $os == linux && -z ${DISPLAY:-} && -z ${WAYLAND_DISPLAY:-} ]]; then
  if command -v xvfb-run >/dev/null; then wrap=(xvfb-run -a -s "-screen 0 1920x1080x24"); else echo "no display (DISPLAY / WAYLAND_DISPLAY) and no xvfb-run"; exit 1; fi
fi

# a fresh home and data folder for one start
fresh_home() {
  local h=$tmp/home-$1
  rm -rf "$h"; mkdir -p "$h/.varstotas"
  echo "$h"
}

app_env() {
  local h=$1
  echo HOME="$h" XDG_CONFIG_HOME="$h/.config" XDG_DATA_HOME="$h/.local/share" XDG_CACHE_HOME="$h/.cache" \
    VARSTOTAS_DATA_DIR="$h/.varstotas" VARSTOTAS_LANG=en RUST_LOG=info
}

# --- the release ------------------------------------------------------------------------------
sums=$pkgdir/SHA256SUMS.txt
version=
for f in "$pkgdir"/Varstotas-*; do
  [[ -e $f ]] || continue
  b=$(basename "$f")
  version=$(echo "$b" | sed -nE 's/^Varstotas-(.*)-(macos|linux|windows)-.*/\1/p')
  [[ -n $version ]] && break
done
if [[ -z $version ]]; then echo "no Varstotas-* packages in $pkgdir"; exit 1; fi
base=${version%%-*}

echo "${bold}Varstotas $version — package tests on $os $cpu${off}"
echo "packages: $pkgdir"
echo "results:  $out"

{
  echo "system: $(uname -srm)"
  if [[ $os == macos ]]; then
    echo "macOS: $(sw_vers -productVersion) ($(sw_vers -buildVersion))"
    system_profiler SPDisplaysDataType 2>/dev/null | grep -E 'Chipset Model|Metal' | sed 's/^ */gpu: /'
  else
    [[ -r /etc/os-release ]] && echo "distribution: $(. /etc/os-release; echo "$PRETTY_NAME")"
    echo "session: ${XDG_SESSION_TYPE:-?} ${XDG_CURRENT_DESKTOP:-} ${wrap[*]:+(xvfb)}"
    command -v lspci >/dev/null && lspci 2>/dev/null | grep -iE 'vga|3d|display' | sed 's/^/gpu: /'
  fi
} > "$out/system.txt"
sed 's/^/  /' "$out/system.txt"

echo
echo "${bold}checksums${off}"
if [[ ! -f $sums ]]; then
  row release SHA256SUMS FAIL "no SHA256SUMS.txt"
else
  if command -v sha256sum >/dev/null; then sha=(sha256sum); else sha=(shasum -a 256); fi
  n=0 bad=0 missing=0
  while read -r hash name; do
    [[ -z $name ]] && continue
    name=${name#\*}
    if [[ ! -f $pkgdir/$name ]]; then missing=$((missing + 1)); continue; fi
    n=$((n + 1))
    got=$("${sha[@]}" "$pkgdir/$name" | cut -d' ' -f1)
    [[ $got == "$hash" ]] || { bad=$((bad + 1)); row release "checksum" FAIL "$name"; }
  done < "$sums"
  (( bad == 0 )) && row release SHA256SUMS ok "$n files match$( (( missing )) && echo ", $missing listed but not here")"
fi

# --- one app ----------------------------------------------------------------------------------
script_list() {   # the script names for a set (quick or full)
  local set=$1
  while read -r kind name; do
    [[ $kind == '#'* || -z $kind ]] && continue
    if [[ $kind == quick || ( $set == full && $kind == full ) ]]; then echo "$name"; fi
  done < "$here/scripts.txt"
}

# test_app VARIANT BIN SET: version, start, scripts
test_app() {
  local v=$1 bin=$2 set=$3 h code log
  local vout
  vout=$(${wrap[@]+"${wrap[@]}"} "$bin" --version 2>&1 | tail -1)
  if [[ $vout == "Varstotas $version ("* ]]; then row "$v" "--version" ok "$vout"; else row "$v" "--version" FAIL "${vout:-no output}"; fi

  # a normal start (graphics probed and chosen as for a user), a screenshot after ~40 frames
  h=$(fresh_home "$v-start")
  log=$out/logs/$v-start.log
  local shot=$out/shots/$v-start.bmp
  limited 120 env $(app_env "$h") VARSTOTAS_SCREENSHOT="$shot" ${wrap[@]+"${wrap[@]}"} "$bin" > "$log" 2>&1
  code=$?
  local gfx
  gfx=$(grep -o 'graphics in use: .*' "$log" | tail -1 | sed 's/graphics in use: //')
  if (( code == 0 )) && [[ -s $shot ]] && ! grep -q panicked "$log"; then
    row "$v" "start" ok "${gfx:-graphics not logged}"
  else
    row "$v" "start" FAIL "exit $code$( [[ -s $shot ]] || echo ', no window image'); $(grep -E 'ERROR|panicked' "$log" | head -1 | cut -c1-160)"
  fi
  if [[ -s $shot ]]; then
    local size w hh
    size=$(od -An -t d4 -j 18 -N 8 "$shot")
    read -r w hh <<< "$size"
    hh=${hh#-}
    if (( w < 1400 || hh < 900 )); then row "$v" window info "${w}x${hh}: smaller than the 1400x900 the UI scripts are written for (a small screen), so their clicks may miss"; fi
  fi

  # the UI scripts, each in its own folder with a fresh home
  local n s work ok bad
  for n in $(script_list "$set"); do
    s=$kit/tests/ui/scripts/$n.txt
    [[ -f $s ]] || { row "$v" "script $n" FAIL "missing $s"; continue; }
    h=$(fresh_home "$v-$n")
    [[ -d $kit/tests/ui/fixtures/$n/components ]] && cp -R "$kit/tests/ui/fixtures/$n/components" "$h/.varstotas/"
    work=$tmp/work-$v-$n
    rm -rf "$work"; mkdir -p "$work/models" "$out/shots/$v"
    ln -s "$out/shots/$v" "$work/shots"
    ln -s "$h" "$work/home"
    log=$out/logs/$v-$n.log
    local t0=$(date +%s)
    (cd "$work" && limited 400 env $(app_env "$h") VARSTOTAS_FOREGROUND=1 VARSTOTAS_SCRIPT="$s" ${wrap[@]+"${wrap[@]}"} "$bin") > "$log" 2>&1
    code=$?
    bad=$(grep -cE 'expect FAIL|panicked' "$log")
    ok=$(grep -c 'expect ok' "$log")
    if (( code == 0 && bad == 0 )); then
      row "$v" "script $n" ok "$ok checks, $(( $(date +%s) - t0 ))s"
    else
      row "$v" "script $n" FAIL "exit $code, $bad failed, $ok ok: $(grep -E 'expect FAIL|panicked' "$log" | head -1 | sed -E 's/.*(expect FAIL|panicked)/\1/' | cut -c1-140)"
    fi
  done
}

pick() {   # packages of this system and processor matching --only
  local f b o
  for f in "$pkgdir"/Varstotas-"$version"-"$os"-*; do
    [[ -f $f ]] || continue
    b=$(basename "$f")
    [[ $os == macos || $b == *"-$cpu"* ]] || continue
    if (( ${#only[@]} )); then
      local hit=0
      for o in "${only[@]}"; do [[ $b == *"$o"* ]] && hit=1; done
      (( hit )) || continue
    fi
    echo "$f"
  done
}

set_for() {   # the script set for a package: full on the main one
  if (( set_quick )); then echo quick; elif (( set_full )) || [[ $1 == *.dmg || $1 == *.AppImage ]]; then echo full; else echo quick; fi
}

# --- macOS ------------------------------------------------------------------------------------
check_mac_app() {   # VARIANT APP
  local v=$1 app=$2
  local plist=$app/Contents/Info.plist bin=$app/Contents/MacOS/varstotas
  local short build
  short=$(/usr/libexec/PlistBuddy -c 'Print :CFBundleShortVersionString' "$plist" 2>/dev/null)
  build=$(/usr/libexec/PlistBuddy -c 'Print :CFBundleVersion' "$plist" 2>/dev/null)
  if [[ $short == "$base" ]]; then row "$v" "Info.plist" ok "$short (build $build)"; else row "$v" "Info.plist" FAIL "version '$short', expected $base"; fi
  local archs
  archs=$(lipo -archs "$bin" 2>&1)
  if [[ $archs == *x86_64* && $archs == *arm64* ]]; then row "$v" "universal" ok "$archs"; else row "$v" "universal" FAIL "$archs"; fi
  if codesign --verify --deep --strict "$app" 2>"$out/logs/$v-codesign.log"; then
    row "$v" "signature" ok "$(codesign -dv "$app" 2>&1 | grep -E '^Signature=' | cut -d= -f2) (no Developer ID: Gatekeeper asks on first start)"
  else
    row "$v" "signature" FAIL "$(head -1 "$out/logs/$v-codesign.log")"
  fi
  # a downloaded copy carries the quarantine flag: Gatekeeper would stop a scripted start
  xattr -dr com.apple.quarantine "$app" 2>/dev/null
}

test_mac_zip() {
  local f=$1 v=zip dir=$tmp/zip
  echo; echo "${bold}$(basename "$f")${off}"
  mkdir -p "$dir"
  if ! ditto -x -k "$f" "$dir" 2>"$out/logs/$v-unpack.log" || [[ ! -d $dir/Varstotas.app ]]; then row $v unpack FAIL "$(head -1 "$out/logs/$v-unpack.log")"; return; fi
  row $v unpack ok "Varstotas.app"
  check_mac_app $v "$dir/Varstotas.app"
  test_app $v "$dir/Varstotas.app/Contents/MacOS/varstotas" "$(set_for "$f")"
}

test_mac_dmg() {
  local f=$1 v=dmg mnt=$tmp/dmg-mount dir=$tmp/dmg-app
  echo; echo "${bold}$(basename "$f")${off}"
  mkdir -p "$mnt" "$dir"
  if ! hdiutil attach -nobrowse -readonly -noautoopen -mountpoint "$mnt" "$f" > "$out/logs/$v-attach.log" 2>&1; then row $v mount FAIL "$(tail -1 "$out/logs/$v-attach.log")"; return; fi
  local items
  items=$(ls "$mnt" | tr '\n' ' ')
  if [[ -d $mnt/Varstotas.app && -L $mnt/Applications ]]; then row $v "disk image" ok "$items"; else row $v "disk image" FAIL "has: $items"; fi
  # installed as a user does: the app dragged to a folder
  ditto "$mnt/Varstotas.app" "$dir/Varstotas.app"
  hdiutil detach "$mnt" -quiet || hdiutil detach "$mnt" -force -quiet
  check_mac_app $v "$dir/Varstotas.app"
  test_app $v "$dir/Varstotas.app/Contents/MacOS/varstotas" "$(set_for "$f")"
}

# --- Linux ------------------------------------------------------------------------------------
test_linux_tar() {
  local f=$1 v=tar dir=$tmp/tar
  echo; echo "${bold}$(basename "$f")${off}"
  mkdir -p "$dir"
  if ! tar xzf "$f" -C "$dir" 2>"$out/logs/$v-unpack.log"; then row $v unpack FAIL "$(head -1 "$out/logs/$v-unpack.log")"; return; fi
  local top
  top=$(ls "$dir" | head -1)
  local app=$dir/$top
  [[ -x $app/varstotas ]] && row $v unpack ok "$top/" || { row $v unpack FAIL "no $top/varstotas"; return; }
  # install.sh into a temporary prefix, then --uninstall
  local prefix=$tmp/tar-prefix h
  h=$(fresh_home tar-install)
  if env HOME="$h" PREFIX="$prefix" sh "$app/install.sh" > "$out/logs/$v-install.log" 2>&1 \
    && [[ -x $prefix/bin/varstotas && -f $prefix/share/applications/varstotas.desktop && -f $prefix/share/mime/packages/varstotas.xml ]]; then
    row $v "install.sh" ok "bin, menu entry, icons, .vrst type"
  else
    row $v "install.sh" FAIL "$(tail -1 "$out/logs/$v-install.log")"
  fi
  if env HOME="$h" PREFIX="$prefix" sh "$app/install.sh" --uninstall >> "$out/logs/$v-install.log" 2>&1 && [[ ! -e $prefix/bin/varstotas ]]; then
    row $v "install.sh --uninstall" ok ""
  else
    row $v "install.sh --uninstall" FAIL "$(tail -1 "$out/logs/$v-install.log")"
  fi
  test_app $v "$app/varstotas" "$(set_for "$f")"
}

test_linux_appimage() {
  local f=$1 v=AppImage bin=$tmp/Varstotas.AppImage
  echo; echo "${bold}$(basename "$f")${off}"
  cp "$f" "$bin" && chmod +x "$bin"
  if ! "$bin" --version > "$out/logs/$v-fuse.log" 2>&1; then
    # no FUSE (containers, some minimal systems): the AppImage unpacks itself instead
    export APPIMAGE_EXTRACT_AND_RUN=1
    row $v FUSE info "not available ($(grep -m1 -iE 'fuse|error' "$out/logs/$v-fuse.log" | cut -c1-80)); runs with APPIMAGE_EXTRACT_AND_RUN=1"
  else
    row $v FUSE ok "mounts"
  fi
  test_app $v "$bin" "$(set_for "$f")"
  unset APPIMAGE_EXTRACT_AND_RUN
}

test_linux_deb() {
  local f=$1 v=deb
  echo; echo "${bold}$(basename "$f")${off}"
  local pkg ver arch want_arch=amd64
  [[ $cpu == aarch64 ]] && want_arch=arm64
  pkg=$(dpkg-deb -f "$f" Package 2>/dev/null) ver=$(dpkg-deb -f "$f" Version 2>/dev/null) arch=$(dpkg-deb -f "$f" Architecture 2>/dev/null)
  if [[ -z $pkg ]]; then
    row $v control skip "no dpkg-deb (not a Debian or Ubuntu system)"
    return
  fi
  if [[ $pkg == varstotas && $arch == "$want_arch" ]]; then row $v control ok "$pkg $ver $arch"; else row $v control FAIL "$pkg $ver $arch"; fi
  local files
  files=$(dpkg-deb -c "$f" | awk '{print $6}')
  local missing=
  for p in ./usr/bin/varstotas ./usr/share/applications/varstotas.desktop; do
    grep -qxF "$p" <<< "$files" || missing+=" $p"
  done
  [[ -z $missing ]] && row $v contents ok "binary, menu entry" || row $v contents FAIL "missing:$missing"
  if (( ! deb )); then
    row $v install skip "run with --deb to install it with sudo"
    return
  fi
  if dpkg -s varstotas >/dev/null 2>&1; then
    row $v install skip "varstotas is installed on this system already (left as it is)"
    return
  fi
  if sudo apt-get install -y "$f" > "$out/logs/$v-install.log" 2>&1 && [[ -x /usr/bin/varstotas ]]; then
    row $v install ok "apt-get install"
    test_app $v /usr/bin/varstotas "$(set_for "$f")"
    if sudo apt-get remove -y varstotas > "$out/logs/$v-remove.log" 2>&1 && [[ ! -e /usr/bin/varstotas ]]; then row $v remove ok ""; else row $v remove FAIL "$(tail -1 "$out/logs/$v-remove.log")"; fi
  else
    row $v install FAIL "$(tail -1 "$out/logs/$v-install.log")"
  fi
}

# --- run --------------------------------------------------------------------------------------
pkgs=$(pick)
[[ -z $pkgs ]] && row release packages FAIL "no packages for $os $cpu"
while read -r f; do
  [[ -z $f ]] && continue
  case $f in
    *.zip) [[ $os == macos ]] && test_mac_zip "$f" ;;
    *.dmg) test_mac_dmg "$f" ;;
    *.tar.gz) test_linux_tar "$f" ;;
    *.AppImage) test_linux_appimage "$f" ;;
    *.deb) test_linux_deb "$f" ;;
  esac
done <<< "$pkgs"

# --- report -----------------------------------------------------------------------------------
fails=$(awk -F'\t' '$3=="FAIL"' "$rows" | wc -l | tr -d ' ')
oks=$(awk -F'\t' '$3=="ok"' "$rows" | wc -l | tr -d ' ')
mins=$(( ($(date +%s) - started + 59) / 60 ))
verdict=PASS; (( fails )) && verdict=FAIL
html_esc() { sed -e 's/&/\&amp;/g' -e 's/</\&lt;/g' -e 's/>/\&gt;/g'; }
{
  echo "# Varstotas $version — $os $cpu: $verdict"
  echo
  echo "$(date '+%Y-%m-%d %H:%M'), $mins min, $oks passed, $fails failed."
  echo
  echo '```'; cat "$out/system.txt"; echo '```'
  echo
  echo "| Package | Check | Result | Detail |"
  echo "|---|---|---|---|"
  awk -F'\t' '{ gsub(/\|/, "\\|", $4); printf "| %s | %s | %s | %s |\n", $1, $2, ($3=="FAIL" ? "**FAIL**" : $3), $4 }' "$rows"
  echo
  echo "Logs: logs/, window images: shots/."
} > "$out/report.md"
{
  echo "<!doctype html><meta charset=utf-8><title>Varstotas $version $os $cpu: $verdict</title>"
  echo "<style>body{font:14px system-ui,sans-serif;margin:16px;color:#222;background:#fff}td,th{padding:3px 8px;border-bottom:1px solid #ddd;text-align:left;vertical-align:top}"
  echo ".ok{color:#18794e}.FAIL{color:#c00;font-weight:bold}.skip,.info{color:#888}h1 .PASS{color:#18794e}img{max-width:320px;border:1px solid #ccc;margin:4px}pre{background:#f4f4f4;padding:8px}</style>"
  echo "<h1>Varstotas $version — $os $cpu: <span class=$verdict>$verdict</span></h1>"
  echo "<p>$(date '+%Y-%m-%d %H:%M'), $mins min, $oks passed, $fails failed.</p><pre>$(html_esc < "$out/system.txt")</pre><table><tr><th>Package<th>Check<th>Result<th>Detail"
  while IFS=$'\t' read -r a b c d; do
    printf '<tr><td>%s<td>%s<td class=%s>%s<td>%s\n' "$a" "$b" "$c" "$c" "$(printf '%s' "$d" | html_esc)"
  done < "$rows"
  echo "</table><h2>Window images</h2>"
  (cd "$out" && find shots -name '*.bmp' | sort) | while read -r s; do echo "<a href=\"$s\"><img src=\"$s\" title=\"$s\" loading=lazy></a>"; done
} > "$out/report.html"
rm -f "$rows"
(( keep )) && echo "temporary files: $tmp" || rm -rf "$tmp"

echo
if [[ $verdict == PASS ]]; then echo "${green}${bold}PASS${off} — $oks checks, $mins min"; else echo "${red}${bold}FAIL${off} — $fails failed, $oks passed, $mins min"; fi
echo "report: $out/report.html"
[[ $verdict == PASS ]]
