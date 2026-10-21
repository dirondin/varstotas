# Varstotas

A lightweight native classic-style 3D editor for macOS and Windows. Rust + egui + wgpu.

```bash
cargo run --release -p varstotas_app     # run
cargo test --workspace                 # tests
```

## Documents
- [Vision and UX principles](docs/00-VISION.md)
- [Features → plan and statuses](docs/01-FEATURES.md)
- [Architecture](docs/02-ARCHITECTURE.md)
- [Roadmap](docs/03-ROADMAP.md)
- [Team and process](docs/04-TEAM.md)
- [**Current status and progress**](docs/06-STATUS.md)
- [UX specification of the tools](docs/05-UX-SPEC.md)
- [M3 QA report](docs/qa/M3-report.md)

## Structure
```
crates/varstotas_kernel  — geometry kernel (sticky geometry, push/pull, offset, follow me, IO)
crates/varstotas_render  — wgpu renderer and camera
crates/varstotas_app     — application: UI, tools, inference, VCB
```

## Release builds
Artifacts go to `dist/` (not tracked by git).

```bash
scripts/build-macos.sh      # macOS universal (arm64 + x86_64)
scripts/build-windows.sh    # Windows x86_64, cross-built from macOS
```

- **macOS** — `scripts/build-macos.sh [--no-dmg]`: installs the `x86_64-apple-darwin` target if needed, builds release for both architectures, joins them with `lipo`, assembles `dist/Varstotas.app` (Info.plist: `dev.varstotas.app`, version from Cargo, macOS ≥ 11, Retina), builds the icon (`crates/varstotas_app/assets/icon.png`, drawn by `scripts/make_icon.py` → `iconutil` → `.icns`), signs ad-hoc (`codesign -s -`) and packs `Varstotas-<version>-macos-universal.zip` and `.dmg`. There is no Developer ID signature or notarization — on first launch: right click → "Open".
- **Windows** — `scripts/build-windows.sh`: installs the `x86_64-pc-windows-gnu` target and `mingw-w64` (via brew), builds `varstotas.exe` (GUI subsystem, no console; the icon and version resource are embedded via `crates/varstotas_app/build.rs` / `winresource` + `windres`) and puts it into `dist/varstotas-windows-x86_64.zip`. Requires Windows 10+ (UCRT); no external DLLs.
- Checking the build without a UI click: `VARSTOTAS_SCREENSHOT=shot.bmp dist/Varstotas.app/Contents/MacOS/varstotas` — captures the window and exits.
