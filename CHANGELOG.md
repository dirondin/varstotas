# Release notes

What each version brought, in a few lines: the app shows this file in Help → What's New, and
a release's notes on GitHub start with its section. A section is `## <version> — <date>`
followed by `- ` lines; a Release build fails when its version has no section here.

## 0.2.0 — 2026-10-28

- Builds for Windows on Arm and Linux aarch64, and a .deb package
- Works on more computers: the graphics API is chosen at start and can be set in Preferences, with OpenGL and software fallbacks for virtual machines
- The Windows app no longer needs the Visual C++ runtime
- Animation exported as a GIF, with no ffmpeg to install
- Exports that write files beside the model (OBJ, DAE) ask for the file name first
- Everything Varstotas keeps for itself is in one data folder
- The AppImage offers to add itself to the applications menu
- Renamed panels and tools: Properties, Document Info, Tool Guide, Sweep

## 0.1.0 — 2026-10-26

- First public release for macOS, Windows and Linux
- Direct modelling with inference snapping and exact typed values
- Groups, components, tags, scenes, sections, dimensions and text
- Furniture making: cut list and cutting diagrams, joinery, hardware, assembly steps and an instructions booklet
- Preparing models for 3D printing, unfolding, sheets with scaled views, simple physics
- PBR materials, a realistic real-time style and a GPU path tracer
- Material and component libraries, online models and materials, parametric generators
- Rhai scripts and plugins, a Python bridge and an MCP server
- Import and export of glTF, OBJ, STL, PLY, DAE, 3MF, DXF and more
- English, Russian, Lithuanian and Belarusian
