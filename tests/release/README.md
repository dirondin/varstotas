# Testing a release

How to test a release candidate on every system before it is published. The release build
(`release.yml`, started by a `release-*` tag) leaves its files in the run's `release-files`
artifact: the packages, `SHA256SUMS.txt` and this kit in `release-tests/`. The kit installs or
unpacks each package, starts it and drives it through UI scripts; what is left for a person is
in [By hand](#by-hand).

## Automated: the package tests

1. Download `release-files` from the Release build run (Actions → Release build → the run →
   Artifacts) and unzip it. It holds the packages, `SHA256SUMS.txt` and `release-tests/`.
2. On each system run the kit from that folder:

   | System | Command | Packages tested |
   |---|---|---|
   | macOS | `bash release-tests/run.sh` | `.dmg` (full), `.zip` |
   | Linux | `bash release-tests/run.sh` | `.AppImage` (full), `.tar.gz` with `install.sh`, `.deb` contents |
   | Linux, Debian or Ubuntu | `bash release-tests/run.sh --deb` | also installs the `.deb` with `sudo apt-get`, tests it, removes it |
   | Windows | double-click `release-tests\run.cmd` | installer (full: install, Start menu, `.vrst` type, uninstall), `.zip` |
   | Windows Sandbox | double-click `release-tests\sandbox.cmd` | the same in a clean Windows; `sandbox.cmd -NoGpu` without a graphics card |

   Only the packages for the computer's processor are tested (`run.cmd -AllCpu` adds the x64
   ones on Windows on Arm).
3. The run ends with **PASS** or **FAIL** and writes `test-results/<time>-<system>-<cpu>/`
   beside the packages: `report.html` (results, system and graphics card, window images),
   `report.md`, `logs/`, `shots/`. The quick set takes about a minute per package, the full
   one about 3 minutes.

What each package goes through:

- its checksum against `SHA256SUMS.txt`;
- unpacked or installed the way a user does it (the `.dmg` mounted and the app copied out, the
  installer run silently into a temporary folder, `install.sh` into a temporary prefix);
- its packaging: macOS `Info.plist` version, both processors in the binary, the signature;
  Windows Apps entry, Start menu entry, `.vrst` file type and a clean uninstall; Linux menu
  entry, icons and file type from `install.sh`, the `.deb`'s control fields and contents;
- `varstotas --version` is this release;
- one normal start: graphics chosen as for a user (with the fallbacks), the window drawn;
- the UI scripts in `scripts.txt`: the `quick` ones on every package (a model, export and
  import, What's New with this version), the `full` ones on the main package of each system
  (modelling, sections, 3D text, 3D printing, sheets, unfolding, the realistic style and the
  path tracer, physics, the cut list, joinery, assembly, doors, fittings, Preferences).

The app runs with its home and data folder in a temporary folder, so the settings, libraries
and recent files of the computer are not touched. The installer is not run on a computer where
Varstotas is installed (it would update and then remove that copy): use Windows Sandbox there.
The `.deb` is installed only with `--deb`, and not over an installed one.

Options: `--full` / `-Full` runs the full set on every package, `--quick` / `-Quick` only the
quick one, `--only NAME` / `-Only NAME` picks packages by file name, `--keep` / `-Keep` keeps the
temporary folders. From the repository the kit runs as `tests/release/run.sh <packages folder>`
(or `tests\release\run.cmd <packages folder>`).

Notes:

- The app's window opens and works by itself for a few minutes: leave the mouse and keyboard
  alone while it does.
- The UI scripts click where things are in a 1400×900 window: on a smaller screen (the report
  says so) their checks can fail without a fault in the app.
- macOS: the copy under test has its quarantine flag removed, so the Gatekeeper prompt is not
  covered (see below).
- Linux without a desktop session (a server, a container) needs `xvfb-run`; the AppImage runs
  with `APPIMAGE_EXTRACT_AND_RUN=1` where FUSE is missing (the report says so).
- Windows Sandbox needs Windows 10/11 Pro or Enterprise with the Windows Sandbox feature on.

## By hand

What the scripts cannot see. For each system, on the installed package:

- **First start as downloaded**: macOS Gatekeeper (Privacy & Security → Open Anyway), Windows
  SmartScreen (More info → Run anyway), the AppImage's offer to add itself to the menu.
- **Desktop integration**: the app in Launchpad / Start menu / the applications menu with its
  icon; a `.vrst` file opens in Varstotas by double-click; the Dock / taskbar icon.
- **Real input**: orbit, pan and zoom with a mouse and with a trackpad (the right drag on
  laptops), drawing with inference, a model's file opened and saved through the system dialogs.
- **What's New and About**: this version's lines, the version and build.
- **The new features of the release**: walk through the CHANGELOG.md section of the version.
- **The web version**: the `web/` site of the run's `web-site` artifact
  (`python3 -m http.server -d web 8765`, then <http://127.0.0.1:8765/> in Chrome).

Then publish: Actions → Publish release, run from the tag.
