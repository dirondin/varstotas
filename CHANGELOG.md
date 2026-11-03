# Release notes

What each version brought, in a few lines: the app shows this file in Help → What's New, and
a release's notes on GitHub start with its section. A section is `## <version> — <date>`
followed by `- ` lines; a Release build fails when its version has no section here.

## 0.3.0 — 2026-11-04

- Web version: Varstotas in a browser with WebGPU
- Furniture making: cut list with stock, offcuts, edge banding, labels, prices and inches
- Joinery cut into the parts with the fasteners drawn, and warnings for likely mistakes
- Furniture hardware: hinges, slides, gas struts, handles, legs, casters and your own fittings
- Doors, drawers, backs and plinths
- Assembly: exploded view, steps and sub-assemblies, an instructions booklet
- Drilling maps (PDF)
- Orbit with a right drag on laptops without a middle button
- What's New in Help

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
- Preparing models for 3D printing, unfolding, sheets with scaled views, simple physics
- PBR materials, a realistic real-time style and a GPU path tracer
- Material and component libraries, online models and materials, parametric generators
- Rhai scripts and plugins, a Python bridge and an MCP server
- Import and export of glTF, OBJ, STL, PLY, DAE, 3MF, DXF and more
- English, Russian, Lithuanian and Belarusian
