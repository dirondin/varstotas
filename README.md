# Varstotas

A lightweight native classic-style 3D editor for macOS and Windows. Rust + egui + wgpu.

```bash
cargo run --release -p varstotas_app     # run
cargo test --workspace                 # tests
qa/run_all.sh [name …]                 # UI regression scripts (explicit repository fixtures)
```

UI languages: English, Russian, Lithuanian, Belarusian (Preferences → Interface → Language).

## Documents
- [Vision and UX principles](docs/00-VISION.md)
- [Features → plan and statuses](docs/01-FEATURES.md)
- [Architecture](docs/02-ARCHITECTURE.md)
- [Code map](docs/CODEMAP.md)
- [Roadmap](docs/03-ROADMAP.md)
- [**Current status and progress**](docs/06-STATUS.md)
- [UX specification of the tools](docs/05-UX-SPEC.md)
- [Scripting (Rhai, Python, bridge)](docs/SCRIPTING.md) · [Generators](docs/GENERATORS.md) · [Plugins](docs/PLUGINS.md) · [Shaders](docs/SHADERS.md)
- [Localization](docs/LOCALIZATION.md)

## Structure
```
crates/varstotas_kernel  — geometry kernel (sticky geometry, push/pull, offset, follow me, IO)
crates/varstotas_render  — wgpu renderer, camera, path tracer
crates/varstotas_script  — Rhai scripting API over the kernel
crates/varstotas_i18n    — localization (Fluent)
crates/varstotas_app     — application: UI, tools, inference, VCB, plugins
locales/                 — UI texts, one .ftl file per language
python/                  — Python client of the local bridge and the MCP server
examples/                — example plugins, scripts, generators and the demo models (.vrst)
qa/                      — UI regression scripts, fixtures, models
vendor/                  — egui_code_editor with a local fix
```

## Examples
Demo scenes, scripts, plugins and optional generators live only in `examples/` in this repository.
They are not embedded in the application or installed into `~/.varstotas`.
**Help → Examples** opens the repository examples in a browser. Download what you need:
open `.vrst` demos with File → Open, run scripts with File → Run Script, copy selected plugins
to `~/.varstotas/plugins/`, and generators to your components library.
Previously installed files are left untouched. Built-in Draw → Shapes tools use their own
resources in `crates/varstotas_app/assets/primitives/`.

## Document templates
File → New from Template lists the built-in unit presets and `.vrst` files from
`~/.varstotas/templates/`. Open Templates Folder opens that directory in Finder / Explorer.
Preferences → General → Templates folder lets you choose another directory or restore the default.
A file template creates an untitled, unsaved copy with its model settings; Save prompts for a new
file and never overwrites the template automatically. Files added to the directory appear the next
time the menu is opened. Existing unsaved work is protected by the usual Save / Don't Save / Cancel prompt.

## Release builds
Artifacts go to `dist/` (not tracked by git).

```bash
scripts/build-macos.sh      # macOS universal (arm64 + x86_64)
scripts/build-windows.sh    # Windows x86_64, cross-built from macOS
```

- **macOS** — `scripts/build-macos.sh [--no-dmg]`: installs the `x86_64-apple-darwin` target if needed, builds release for both architectures, joins them with `lipo`, assembles `dist/Varstotas.app` (Info.plist: `dev.varstotas.app`, version from Cargo, macOS ≥ 11, Retina), builds the icon (`crates/varstotas_app/assets/icon.png`, drawn by `scripts/make_icon.py` → `iconutil` → `.icns`), signs ad-hoc (`codesign -s -`) and packs `Varstotas-<version>-macos-universal.zip` and `.dmg`. There is no Developer ID signature or notarization — on first launch: right click → "Open".
- **Windows** — `scripts/build-windows.sh`: installs the `x86_64-pc-windows-gnu` target and `mingw-w64` (via brew), builds `varstotas.exe` (GUI subsystem, no console; the icon and version resource are embedded via `crates/varstotas_app/build.rs` / `winresource` + `windres`) and puts it into `dist/varstotas-windows-x86_64.zip`. Requires Windows 10+ (UCRT); no external DLLs.
- Checking the build without a UI click: `VARSTOTAS_SCREENSHOT=shot.bmp dist/Varstotas.app/Contents/MacOS/varstotas` — captures the window and exits.
