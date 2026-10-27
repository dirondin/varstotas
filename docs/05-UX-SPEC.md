# UX specification (behaviour)

## Window layout
```
┌ Menu: File Edit View Camera Draw Tools Window Help ───────────────────────┐
│ Top toolbar: Select Eraser Paint | Line Arc Rect Circle Polygon |         │
│   3D solids | Move Rotate Scale PushPull Offset FollowMe … | Tape … |     │
│   Orbit Pan Zoom ZoomExtents | Snapping (views and styles — in viewport)  │
├──────────────────────────────────────────────────────────┬────────────────┤
│                                                          │ Side Bar:      │
│                    3D view (sky/ground,                  │ ▸ Properties  │
│                    RGB axes, scale figure)               │ ▸ Materials    │
│                                                          │ ▸ Tags         │
│                                                          │ ▸ Outliner     │
│                                                          │ ▸ Scenes       │
│                                                          │ ▸ Tool Guide   │
├──────────────────────────────────────────────────────────┴────────────────┤
│ ⓘ Tool hint / modifiers                         │ Length: [ 2500 mm  ]   │
└───────────────────────────────────────────────────────────────────────────┘
```
Light theme, soft colours: sky `#A8C8E8→#E8F0F8`, ground `#C8C0B0` (transparent when looking from below), face front `#FFFFFF`, back `#A4B0C8`, edges `#000000`, selection `#0050FF`.

## Hotkeys
Space Select · L Line · A Arc · R Rectangle · C Circle · E Eraser · B Paint · P Push/Pull · M Move · Q Rotate · S Scale · F Offset · T Tape · O Orbit · H Pan · Z Zoom · Shift+Z Zoom Extents · Shift+S snapping on/off · G Make Group/Component · Ctrl/Cmd+Z/Y undo/redo · Delete — delete the selection · Esc — cancel the step · Ctrl/Cmd+A — select all · Ctrl/Cmd+S/O/N.
Middle button — Orbit, Shift+middle — Pan, wheel — Zoom to cursor (in any tool). On a Mac trackpad: two fingers — orbit, Shift+two fingers — pan, pinch — zoom.

## Measurements box
- While a tool is active, digits typed on the keyboard **without clicking into the box** go into the box. Enter — apply.
- The box label changes with the tool: Length, Dimensions, Radius, Sides, Distance, Angle, Scale, Offset.
- Values: `2500` (in model units), `250cm`, `2.5m`, `10'`, `6"`, `1'6"`; a pair `300;200` or `300,200` (if the decimal separator is a dot).
- After the operation finishes, new input edits the last operation (line length, push/pull depth, radius…).
- Arrays after a Move/Rotate copy: `5x` (5 more copies in the same step), `/5` (5 copies within the interval).
- Circle/polygon: `24s` — number of segments/sides.

## Inference (snapping)
Priority: Endpoint > Midpoint/Center > Intersection > On Edge > axis/parallel/perpendicular directions > On Face > free point.
Capture radius: 12 px (points), 8 px (edges). A tooltip hint next to the cursor (with a ~150 ms delay).
| Type | Marker | Colour |
|---|---|---|
| Endpoint | circle ● | green `#00C000` |
| Midpoint | circle ● | cyan `#00C0FF` |
| On Edge | square ■ | red `#FF0000` |
| On Face | diamond ◆ | blue `#0000FF` |
| Center | circle ◯ | green |
| Intersection | cross ✕ | red |
| On Red/Green/Blue Axis | line | red/green/blue |
| Parallel/Perpendicular to Edge | line | magenta `#FF00FF` |
| From Point | dashed line | axis colour |
- **Shift** holds the current direction/plane snap.
- **Arrows:** → lock the red axis, ← the green, ↑ the blue, ↓ parallel/perpendicular to the edge under the cursor.
- Hovering over a point (~0.5 s) makes it a "reference": dashed axes are built from it (From Point).

## Tools (steps)
- **Line:** click A → [move: rubber line + Measurements box Length] → click B (B becomes the start of the next) … Esc/double click — finish. A closed flat loop → face.
- **Rectangle:** click corner 1 → move (plane: the face under the cursor, otherwise the ground/axis plane by view direction) → click corner 2. Measurements box `w,h`. Inference of a square ("Square") and of the golden ratio.
- **Circle/Polygon:** click the centre (orientation — by the face under the cursor, otherwise horizontal) → click the radius. Measurements box: before the first click — `Ns` segments; after — the radius. Default 24 segments / 6 sides.
- **Arc (2pt):** click the start → click the end → move the bulge → click. Snaps "Half circle", "Tangent".
- **Push/Pull:** hovering highlights a face → click → move (distance along the normal) → click. Measurements box Distance. Double click — repeat the previous distance. Ctrl/Option — keep the original face. Pushing in through an object to the opposite parallel face — a through hole.
- **Move:** if there is a selection — move it; otherwise — the object under the cursor (highlighted). Click the grab point → click the destination. Ctrl/Option — copy. Moving vertices/edges/faces stretches the connected geometry. Arrows lock the axis.
- **Rotate:** click — centre (the protractor is oriented by the face / by an arrow axis) → click — zero direction → click — angle (snap step 15°). Measurements box Angle.
- **Offset:** click a face → move inward/outward → click. Measurements box Distance. Double click — repeat.
- **Eraser:** click/drag over edges; deleting an edge deletes the adjacent faces.
- **Tape Measure:** click → click: shows the distance; with Ctrl (on by default) from an edge — a parallel guide line, from a point — a guide point.
- **Paint Bucket:** click a face — the material from the Materials panel; Alt/Option — eyedropper; Ctrl — all connected faces; Shift — all faces with the same material.
- **Select:** click — an entity; double click on a face — the face + its edges; triple — everything connected; a box left-to-right — only fully enclosed, right-to-left — also crossing. Ctrl — add, Shift — toggle, Ctrl+Shift — remove. Double click on a group — enter editing; Esc / click outside — exit.

## Status bar
Always shows the current step: "Select start point." → "Select end point or enter value." and modifiers: "Shift = lock inference, Ctrl = toggle copy".
