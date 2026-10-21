# Varstotas — product vision

**Varstotas** is a lightweight native 3D editor (macOS / Windows) of the classic type: you draw lines and get surfaces, you pull a surface and get a volume. No construction trees, sketches or constraints: "pencil and paper in 3D".

## Who it is for
- Architects, interior designers, furniture makers, makers — quick concepts and simple models.
- Newcomers to 3D: the tool must be understandable within 10 minutes.

## UX principles (not up for discussion)
1. **Few tools, each does one thing.** Line, Rectangle, Circle, Push/Pull, Move, Rotate….
2. **Inference everywhere.** The cursor itself "sticks" to endpoints, midpoints, edges, faces, axes (red/green/blue), parallels and perpendiculars. Coloured hints + a pop-up label ("Endpoint", "On Red Axis").
3. **Precision without dialogs.** Measurements box (VCB): after a click you just type `2500` or `300,200` and press Enter. The last operation can be corrected by typing again.
4. **Sticky geometry.** Edges closed into a flat loop automatically form a face; a line across a face splits it in two. Groups/components isolate geometry.
5. **Always a hint on what to do next.** The status bar describes the tool's current step and modifiers (Shift — lock the direction, Ctrl/Option — copy, arrows — lock the axis).
6. **Instant response.** 60+ FPS on scenes of tens of thousands of faces, startup < 1 s, binary < 30 MB.
7. **Unlimited Undo/Redo** for every operation.

## What we do NOT do (at the start)
- Compatibility with other programs' closed formats — only our own `.vrst` format + OBJ/STL export/import.
- Ruby API / extension store, a separate layout program, photorealistic rendering, cloud.

## MVP success criteria
A user without instructions draws a house: floor rectangle → Push/Pull walls → roof lines → Move the ridge vertex → windows via Offset + Push/Pull, paints with materials, saves and exports to OBJ.
