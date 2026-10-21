tool-select = Select
tool-eraser = Eraser
tool-paint = Paint Bucket
tool-texture-position = Texture Position
tool-line = Line
tool-arc = 2 Point Arc
tool-arc-center = Arc
tool-arc3 = 3 Point Arc
tool-pie = Pie
tool-bezier = Bezier Curve
tool-smooth-curve = Smooth Curve
tool-freehand = Freehand
tool-edit-curve = Edit Curve
tool-rectangle = Rectangle
tool-rotated-rectangle = Rotated Rectangle
tool-circle = Circle
tool-polygon = Polygon
tool-box = Box
tool-cylinder = Cylinder
tool-cone = Cone
tool-pyramid = Pyramid
tool-sphere = Sphere
tool-torus = Torus
tool-push-pull = Push/Pull
tool-move = Move
tool-rotate = Rotate
tool-scale = Scale
tool-offset = Offset
tool-chamfer = Chamfer
tool-fillet = Fillet
tool-solid-tools = Solid Tools
tool-follow-me = Follow Me
tool-tape = Tape Measure
tool-protractor = Protractor
tool-axes = Axes
tool-dimension = Dimensions
tool-text = Text
tool-section = Section Plane
tool-guide-plane = Guide Plane
tool-slice = Slice
tool-orbit = Orbit
tool-pan = Pan
tool-zoom = Zoom
tool-position-camera = Position Camera
tool-look-around = Look Around
tool-walk = Walk

tool-select-desc = Select entities to modify with other tools.
tool-eraser-desc = Erase edges, guides and other entities.
tool-paint-desc = Apply a material to faces and groups.
tool-texture-position-desc = Move, rotate and scale a face's texture with pins.
tool-line-desc = Draw edges. Closed planar loops become faces.
tool-arc-desc = Draw an arc from two end points and a bulge.
tool-arc3-desc = Draw an arc through three points.
tool-arc-center-desc = Draw an arc from its center and two points.
tool-pie-desc = Draw a closed pie-shaped face from a center and two points.
tool-bezier-desc = Draw a curve shaped by handles: click for a corner, press and drag for a smooth point.
tool-smooth-curve-desc = Draw a smooth curve through the points you click; closed, it makes a face.
tool-freehand-desc = Draw a curve freehand: press and drag; ended at its start, it makes a face.
tool-edit-curve-desc = Reshape a drawn curve: drag its points and handles, add or remove points.
tool-rectangle-desc = Draw a rectangular face from two corners.
tool-rotated-rectangle-desc = Draw a rectangle at any angle from an edge and a width.
tool-circle-desc = Draw a circle from center and radius.
tool-polygon-desc = Draw a regular polygon.
tool-box-desc = Draw a box: base corners, then the height. Its sizes stay editable.
tool-cylinder-desc = Draw a cylinder: centre, radius, height. Its sizes stay editable.
tool-cone-desc = Draw a cone: centre, radius, height. Its sizes stay editable.
tool-pyramid-desc = Draw a pyramid: base corners, then the height. Its sizes stay editable.
tool-sphere-desc = Draw a sphere resting on a face: centre, radius. Its sizes stay editable.
tool-torus-desc = Draw a torus lying on a face: centre, radius, tube. Its sizes stay editable.
tool-push-pull-desc = Push or pull a face to add or remove volume.
tool-move-desc = Move, stretch or copy entities.
tool-rotate-desc = Rotate, stretch or copy entities about an axis.
tool-scale-desc = Resize and stretch entities.
tool-offset-desc = Offset a face's edges inward or outward.
tool-chamfer-desc = Cut an edge between two faces with a flat bevel.
tool-fillet-desc = Round an edge between two faces.
tool-solid-tools-desc = Combine solid groups: outer shell, intersect, union, subtract, trim, split.
tool-follow-me-desc = Sweep a face along a path.
tool-tape-desc = Measure distances and create guides.
tool-protractor-desc = Measure angles and create angled guides.
tool-axes-desc = Move or reorient the drawing axes.
tool-dimension-desc = Place linear dimensions between points or on edges.
tool-text-desc = Add a text with a leader pointing at the model.
tool-section-desc = Cut the model with a section plane to see inside.
tool-guide-plane-desc = Add a work plane to draw on in the air; the axes and the grid follow it.
tool-slice-desc = Cut the selection or the model with a plane; optionally split it into two groups.
tool-orbit-desc = Rotate the camera around the model.
tool-pan-desc = Move the camera sideways.
tool-zoom-desc = Zoom the camera in or out.
tool-position-camera-desc = Stand at a point at eye height to look at the model.
tool-look-around-desc = Turn your head: pivot the view around the eye.
tool-walk-desc = Walk through the model at eye height.

solid-union = Union
solid-intersect = Intersect
solid-subtract = Subtract
solid-trim = Trim
solid-split = Split
solid-outer-shell = Outer Shell

## Slice

tool-slice-undo = Slice
tool-slice-undo-split = Slice and Split
tool-slice-nothing = Slice: the plane cuts nothing
tool-slice-status-first = Click a point on the cutting plane.
tool-slice-status-second = Click a second point. Double-click / Enter: plane of the face; arrows: normal along red / green / blue / edge.
tool-slice-status-third = Click a third point to cut.
# { $step } is one of the tool-slice-status-* texts; { $split } is "on" or "off".
tool-slice-status = { $step } Cuts the selection, or everything in the active context. %COPY%: { $split ->
        [on] Split ON: parts become two groups
       *[off] Split OFF: only edges are added
    }.

## Measurements box labels (shared)

tool-vcb-measurements = Measurements
tool-vcb-radius = Radius
tool-vcb-angle = Angle
tool-vcb-length = Length
tool-vcb-segments = Segments
tool-vcb-distance = Distance
tool-vcb-width-angle = Width, Angle
tool-vcb-dimensions = Dimensions
tool-vcb-height = Height
tool-vcb-sides = Sides
tool-vcb-bulge = Bulge
tool-vcb-origin = Origin
tool-vcb-point = Point
tool-vcb-eye-height = Eye Height
tool-vcb-height-offset = Height Offset
tool-vcb-fov = Field of View

## Arcs, rectangles

tool-undo-pie = Pie
tool-undo-arc = Arc
tool-undo-arc3 = 3 Point Arc
tool-undo-rotated-rectangle = Rotated Rectangle
# { $label } is the shape's name (an undo name).
tool-degenerate = { $label }: degenerate input
tool-cursor-angle = Angle
tool-arc-center-first = Select the center of the { $what } (a face orients it; arrows lock the plane). Type `{ $segs }s` for segments.
tool-arc-center-start = Select the start point (radius) or type a radius.
tool-arc-center-end = Move around the center and click to end the { $what }, or type an angle.
tool-arc-what-pie = pie
tool-arc-what-arc = arc
tool-arc3-first = Select the start point of the arc.
tool-arc3-second = Select a second point the arc passes through, or type a length.
tool-arc3-third = Select the end point, or type the arc's angle.
tool-rotrect-first = Select the first corner of the rectangle.
tool-rotrect-second = Select the second corner (the first edge), or type a length (`l,w`).
tool-rotrect-third = Move to set the width and angle and click, or type `width` or `width,angle`.

## Bezier / curves

tool-bezier-end = Enter or double-click finishes, a click on the first point closes it into a face
tool-bezier-freehand-start = Press and drag to draw; end at the start to close it into a face.
tool-bezier-freehand-draw = Drag to draw; release to finish.
tool-bezier-smooth-first = Click the first point of the curve.
# { $end } is the tool-bezier-end text.
tool-bezier-smooth-next = Click the next point; { $end }. Backspace removes the last point.
tool-bezier-handles-first = Click the first point of the curve; press and drag to pull out handles.
tool-bezier-handles-drag = Drag to shape the curve through this point; release to place it.
# { $end } is the tool-bezier-end text.
tool-bezier-handles-next = Click the next point (drag for handles); { $end }. Backspace removes the last point.

## Camera tools

tool-position-camera-status = Click to stand at a point, or drag from where to stand toward where to look. Type the eye height.
tool-look-around-status = Drag to look around. Type an eye height and press Enter.
tool-walk-status = Hold the button and move up/down to walk, left/right to turn (farther = faster). Shift = up/down and sideways, %CMD% = run. Arrow keys step.

## Fillet / Chamfer

tool-undo-fillet = Fillet
tool-undo-chamfer = Chamfer
# { $what } is the lower-case operation name, { $error } the reason.
tool-chamfer-cant = Can't { $what } this edge: { $error }
tool-chamfer-what-fillet = fillet
tool-chamfer-what-chamfer = chamfer
tool-fillet-sizing = Move away from the edge to set the radius, click to finish. Type a radius or `12s`; Esc cancels.
tool-chamfer-sizing = Move away from the edge to set the size, click to finish. Type a distance; Esc cancels.
tool-fillet-hover = Click an edge between two faces to round it (selected edges: all of them); double-click = last radius.
tool-chamfer-hover = Click an edge between two faces to bevel it (selected edges: all of them); double-click = last size.

## Tape measure, protractor, axes

tool-undo-resize-model = Resize Model
tool-undo-resize = Resize
tool-undo-create-guide = Create Guide
tool-undo-place-axes = Place Axes
tool-tape-guides-on = guides on
tool-tape-measure-only = measure only
# { $g } is tool-tape-guides-on or tool-tape-measure-only.
tool-tape-status-resize = Type a new length to resize the model, or select a start point ({ $g }).
tool-tape-status-start = Select a start point (an edge for a parallel guide). %COPY% toggles guides ({ $g }).
tool-tape-status-end = Select the end point or type a distance ({ $g }).
tool-protractor-center = Select the protractor center (arrows lock its axis).
tool-protractor-base = Select the base direction.
tool-protractor-angle = Select the angle (snaps every 15°) or type degrees.
tool-axes-origin = Select the new origin, or type x,y,z.
tool-axes-red = Select a point along the new red axis.
tool-axes-green = Select a point toward the new green axis.

## Curve editing, dimensions

tool-undo-add-curve-point = Add Curve Point
tool-undo-edit-curve = Edit Curve
tool-undo-remove-curve-point = Remove Curve Point
tool-undo-edit-segments = Edit Segments
tool-curve-edit-too-few = Edit Curve: too few points to remove one
tool-curve-edit-pick = Click a curve drawn with Bezier Curve, Smooth Curve or Freehand.
tool-curve-edit-drag = Drag to reshape; release to apply, Esc cancels.
tool-curve-edit-insert = Click to add a point here (the curve keeps its shape).
tool-curve-edit-idle = Drag points and handles; click the curve to add a point, Delete removes the selected point; double-click a point: corner / smooth. Enter finishes.
tool-undo-dimension = Dimension
tool-dimension-first = Select the start point of the dimension, or click an edge to dimension its length.
tool-dimension-second = Select the end point of the dimension.
tool-dimension-third = Move to offset the dimension line and click to place it.

## Line, rectangle, circle, polygon, arc

tool-undo-line = Line
tool-undo-rectangle = Rectangle
tool-undo-polygon = Polygon
tool-undo-circle = Circle
tool-line-first = Select start point. Arrow keys lock an axis.
tool-line-next = Select end point or type a length. Shift = lock inference, arrows = lock axis, Esc/double-click = finish.
tool-rectangle-first = Select first corner. Arrow keys pick the plane (→ red, ← green, ↑ blue normal).
tool-rectangle-second = Select opposite corner or type width,height. Esc = cancel.
tool-circle-what-polygon = polygon
tool-circle-what-circle = circle
tool-circle-for-sides = sides
tool-circle-for-segments = segments
# { $what } is tool-circle-what-*, { $kind } is tool-circle-for-*.
tool-circle-first = Select the { $what } center (face under cursor orients it; arrows lock the normal). Type `{ $segs }s` for { $kind }.
tool-circle-radius = Select a point on the radius or type a radius.
tool-arc-first = Select the start point of the arc. Start at the end of an edge to draw tangent to it.
tool-arc-end-tangent = Select the end point (double-click it for the cyan tangent arc) or type the chord length.
tool-arc-end = Select the end point or type the chord length.
tool-arc-bulge = Move to set the bulge and click, or type a bulge (`500r` for a radius). Type `12s` for segments.

## Eraser, Paint Bucket, Push/Pull, Offset, Follow Me

tool-undo-hide = Hide
tool-undo-unsoften = Unsoften
tool-undo-soften = Soften
tool-undo-erase = Erase
tool-eraser-status = Click or drag over edges, guides, dimensions or groups to erase. Shift = hide, %CMD% = soften, %CMD%+Shift = unsoften.
tool-paint-mods = Alt = sample, Ctrl = connected faces with the same material, Shift = all matching faces, Ctrl+Shift = matching connected faces
    .macos = Cmd = sample, Option = connected faces with the same material, Shift = all matching faces, Option+Shift = matching connected faces
tool-undo-paint = Paint
# { $mods } is tool-paint-mods.
tool-paint-status = Click a face or group to paint. { $mods }.
tool-paint-default-material = Default
tool-undo-push-pull = Push/Pull
# { $keep } is "on" or "off".
tool-push-pull-moving = Move to set the distance, click or type a value. %COPY% = keep original face ({ $keep ->
        [on] on
       *[off] off
    }).
tool-push-pull-idle = Click a face to push or pull it. Double-click repeats the last distance.
tool-undo-offset = Offset
tool-offset-moving = Move inward or outward and click, or type a distance.
tool-offset-idle = Click a face to offset its edges. Double-click repeats the last offset.
tool-undo-follow-me = Follow Me
tool-follow-me-profile = Click the profile face (pre-select path edges to sweep immediately).
tool-follow-me-path = Click or drag along path edges; double-click/Enter to finish. Click a face to sweep around its edge. Esc = cancel.

## Select

tool-select-mods = Ctrl = add, Shift = toggle, Ctrl+Shift = remove
    .macos = Option (or Cmd) = add, Shift = toggle, Shift+Option = remove
tool-undo-aim-light = Aim Light
tool-undo-active-cut = Active Cut
tool-select-aim = Aim the light: drop the handle on what it should shine at. Esc = cancel.
# { $mods } is tool-select-mods.
tool-select-box = Drag left→right for window selection, right→left for crossing selection. { $mods }.
tool-select-idle = Select objects. { $mods }. Drag to select multiple. Double-click a group to edit it.

## Move, Rotate, Scale, Paste

tool-undo-copy = Copy
tool-undo-move = Move
tool-undo-copy-array = Copy Array
tool-undo-rotate = Rotate
tool-undo-rotate-array = Rotate Array
tool-undo-scale = Scale
tool-undo-paste = Paste
tool-suffix-copy = (copy)
tool-suffix-on = (on)
# { $copy } is empty or " " + tool-suffix-copy.
tool-move-from = Pick the point to move from{ $copy }. Moves the selection, or the entity under the cursor. %COPY% = copy; after a copy type `5x` or `/5` for arrays.
tool-move-to = Pick the destination or type a distance{ $copy }. Shift = lock inference, arrows = lock axis.
tool-rotate-center = Select the rotation center (face orients the protractor; arrows lock its axis).
tool-rotate-zero = Select the zero direction.
# { $step } is tool-rotate-rounded or tool-rotate-snaps; { $on } is empty or " " + tool-suffix-on.
tool-rotate-angle = Select the angle ({ $step }) or type degrees. %COPY% = copy{ $on }; after a copy type `6x` for a radial array.
# { $step } is an angle.
tool-rotate-rounded = rounded to { $step }°
tool-rotate-snaps = snaps every 15°
tool-scale-uniformly = uniformly
tool-scale-two-axes = along two axes
tool-scale-one-axis = along one axis
tool-scale-about-centre = centre
tool-scale-about-opposite = opposite grip
# { $what } is tool-scale-uniformly / -two-axes / -one-axis, { $about } is tool-scale-about-*.
tool-scale-dragging = Move to scale { $what } about the { $about }; click or type a factor (per axis: `1.5; 2`). %CMD% = about centre.
tool-scale-select = Select entities (or hover one), then drag a grip.
tool-scale-idle = Drag a grip: corners scale uniformly, edge midpoints two axes, face centres one axis. %CMD% = about centre.
# { $factors } is the scale factor(s).
tool-scale-tooltip-uniform = Uniform scale { $factors }
tool-scale-tooltip = Scale { $factors }
# { $name } is the name of the component being placed.
tool-paste-place-named = Click to place { $name } (Esc cancels).
# { $count } is the number of entities.
tool-paste-place = Click to place the pasted { $count ->
        [one] { $count } entity
       *[other] { $count } entities
    } (Esc cancels).

## Section, solids, text, texture, workplane, misc

tool-undo-section-plane = Section Plane
tool-section-status = Click a face to place a section plane. It cuts away the side behind its arrows.

## Shapes (box, cylinder, cone, pyramid, sphere, torus)

tool-vcb-tube = Tube
# { $what } is the lower-case shape name.
tool-prim-box-first = Click the first corner of the { $what }'s base (a face under the cursor orients it; arrows set the plane).
tool-prim-round-first = Click the centre of the { $what }'s base (a face under the cursor orients it). Type `{ $segs }s` for segments.
tool-prim-box-second = Click the opposite corner, or type `width, depth` (`width, depth, height` places it at once).
tool-prim-sphere-radius = Move to set the radius and click, or type it.
tool-prim-torus-radius = Click the radius (to the middle of the tube), or type `radius, tube`.
tool-prim-round-radius = Click the radius, or type `radius` (`radius, height` places it at once).
tool-prim-torus-tube = Move to set the tube's radius and click, or type it.
tool-prim-height = Move up or down to set the height and click, or type it.

# { $name } is the solid operation's name.
tool-solid-not-solid = { $name }: not a solid (every edge must bound exactly two faces, no nested groups).
tool-solid-first = { $name }: click the first solid group or component.
tool-solid-next = { $name }: click the next solid to add it (Esc to start over).
tool-solid-cut = { $name }: click the solid to cut with the first one (Esc to start over).
tool-solid-second = { $name }: click the second solid (Esc to start over).
tool-undo-text = Text
tool-text-first = Click a point, edge or face to attach a text, or empty space for a text without a leader.
tool-text-place = Move to place the text and click. Edit the text in Entity Info.
tool-text3d-status = Click to place the 3D text (Esc cancels).
tool-undo-texture-position = Texture Position
tool-texture-pick = Click a textured face to position its texture.
tool-texture-move = Moving the texture. Esc cancels.
tool-texture-green = Rotating and scaling the texture about the red pin.
tool-texture-blue = Scaling the texture height.
tool-texture-idle = Drag the face or the red pin to move, the green pin to rotate/scale, the blue pin to stretch. Enter or click outside when done.

tool-undo-add-guide-plane = Add Guide Plane
tool-undo-edit-guide-plane = Edit Guide Plane
tool-workplane-origin = Click the plane's origin (or type x,y,z). Drag the active plane's handles: origin, normal tip, red spin handle.
tool-workplane-red = Click a point along the new red axis. Double-click / Enter: plane of the face; arrows: normal along red / green / blue / edge.
tool-workplane-third = Click a third point on the plane.
tool-workplane-drag-origin = Move the plane's origin.
tool-workplane-drag-normal = Tilt the plane: move its normal.
tool-workplane-drag-spin = Spin the plane around its normal.
tool-zoom-status = Drag up/down to zoom. Type a field of view and press Enter.
tool-orbit-status = Drag to orbit. Shift = pan.
tool-pan-status = Drag to pan.
# { $prompt } is the plugin's prompt.
tool-plugin-select = { $prompt } — click to add / remove, Enter to continue, Esc cancels.
# { $name } is the tool's name.
tool-not-implemented = { $name } tool is not implemented yet.

## Instructor panel (long help per tool; a line starting with • is a list item)

tool-select-instructor =
    Click an entity to select it.
    • Double-click a face: select it with its edges.
    • Triple-click: select everything connected.
    • Drag left→right: window selection (fully inside).
    • Drag right→left: crossing selection (touching).
    • Ctrl = add, Shift = toggle, Ctrl+Shift = remove.
    • Double-click a group to edit it; Esc to close it.
    • Delete erases the selection.
    .macos =
        Click an entity to select it.
        • Double-click a face: select it with its edges.
        • Triple-click: select everything connected.
        • Drag left→right: window selection (fully inside).
        • Drag right→left: crossing selection (touching).
        • Option (or Cmd) = add, Shift = toggle, Shift+Option = remove.
        • Double-click a group to edit it; Esc to close it.
        • Delete erases the selection.
tool-eraser-instructor = Click or drag over edges to erase them. Erasing an edge removes the faces it bounds.
tool-paint-instructor =
    Pick a material in the Materials panel, then click faces.
    • Alt: sample a material.
    • Ctrl: fill connected faces with the same material.
    • Shift: replace the material on all matching faces.
    • Ctrl+Shift: replace it on matching connected faces.
    .macos =
        Pick a material in the Materials panel, then click faces.
        • Cmd: sample a material.
        • Option: fill connected faces with the same material.
        • Shift: replace the material on all matching faces.
        • Option+Shift: replace it on matching connected faces.
tool-texture-position-instructor =
    Position the texture of a textured face (select it, or click it).
    • Drag the face or the red pin: move the texture.
    • Green pin: rotate and scale about the red pin.
    • Blue pin: stretch the texture height.
    • Esc cancels a drag; Enter or a click outside finishes.
tool-line-instructor =
    Click to start, click to end. Each end starts a new segment.
    Type a length and press Enter for precision.
    Shift locks the inference direction; arrow keys lock axes.
tool-arc-instructor =
    Click the start point, click the end point, then move to set the bulge and click.
    • Type a length for the chord, then a bulge (or `500r` for a radius); `12s` sets segments.
    • Starting at the end of an edge, a cyan arc is tangent to it.
    • Double-click to draw an arc tangent to the previous one.
tool-arc-center-instructor =
    Click the center, click the start point (radius), then move around and click the end.
    Type a radius after the first click, then an angle. `12s` sets segments.
tool-arc3-instructor =
    Click the start point, click a point the arc passes through, then click the end.
    Type a length after the first click, an angle after the second.
tool-bezier-instructor =
    Click points of the curve; press and drag at a point to pull out its handles (a plain click makes a corner).
    Enter or a double-click finishes an open curve (edges only); a click on the first point closes it into a face.
    Backspace removes the last point. Type a length for the next point toward the cursor; `12s` sets segments per span.
tool-edit-curve-instructor =
    Reshape a curve drawn with Bezier Curve, Smooth Curve or Freehand (double-click it with Select, or select it and pick Edit Curve).
    • Drag a point (square) to move it, a handle (circle) to bend the curve; a smooth point keeps its handles in line.
    • Click the curve between points to add a point; Delete removes the selected point.
    • Double-click a point: corner ↔ smooth. `12s` sets segments per span.
    • Enter, Esc or a click in empty space finishes.
tool-smooth-curve-instructor =
    Click points; the curve passes smoothly through them.
    Enter or a double-click finishes an open curve (edges only); a click on the first point closes it into a face.
    Backspace removes the last point. Type a length for the next point toward the cursor; `12s` sets segments per span.
tool-freehand-instructor =
    Press and drag to draw; release to finish. The stroke is smoothed.
    A stroke that ends at its start closes into a face; otherwise it stays edges.
tool-pie-instructor =
    Click the center, click the start point (radius), then click the end to make a closed pie face.
    Type a radius, then an angle.
tool-rectangle-instructor = Click the first corner, then the opposite corner. Type width,height to be exact.
tool-rotated-rectangle-instructor =
    Click the first corner, click the second (the first edge), then move to set the width and click.
    Type a length (or `l,w`) for the edge, then the width (or `w,angle`).
tool-circle-instructor = Click the center, then move and click to set the radius. Type `24s` before clicking to set segments.
tool-polygon-instructor = Click the center, then the radius. Type `6s` to set the number of sides.
tool-box-instructor =
    Click the first corner of the base, click the opposite corner, then move up or down and click to set the height.
    • A face under the cursor carries the base; arrows after the first click set its plane.
    • Type `width, depth`, then the height — or `width, depth, height` at once.
    • The result is a component: change its sizes in Entity Info; Explode it for plain geometry.
tool-cylinder-instructor =
    Click the centre of the base, click the radius, then move up or down and click to set the height.
    • Type the radius, then the height — or `radius, height` at once; `32s` sets segments.
    • The result is a component: change its sizes in Entity Info; Explode it for plain geometry.
tool-sphere-instructor =
    Click where the sphere rests, then move and click to set the radius (or type it).
    • `32s` sets segments. The result is a component: change its sizes in Entity Info.
tool-torus-instructor =
    Click the centre, click the radius (to the middle of the tube), then move and click to set the tube.
    • Type `radius, tube` at once; `32s` sets segments.
    • The result is a component: change its sizes in Entity Info.
tool-push-pull-instructor =
    Hover a face, click, move to set the distance, click again.
    %COPY% keeps the original face. Double-click repeats the last distance.
tool-move-instructor = Click a point to pick up the selection, click the destination. %COPY% makes a copy; type `5x` for an array.
tool-rotate-instructor = Click the center, click the zero direction, click the angle. Type an angle in degrees.
tool-scale-instructor = Select entities, then drag a grip. Type a scale factor.
tool-offset-instructor = Click a face, move inward or outward, click. Type a distance.
tool-chamfer-instructor =
    Click an edge between two faces (or one of the selected edges: all of them), move away from it to set the size, click again.
    • Type the distance while sizing; double-click uses the last size.
    • Typing right after a chamfer redoes it with the new size. Esc cancels.
tool-fillet-instructor =
    Click an edge between two faces (or one of the selected edges: all of them), move away from it to set the radius, click again.
    • Type the radius, `12s` for the number of segments, or both: `20,12s`.
    • Double-click uses the last radius; typing right after a fillet redoes it. Esc cancels.
tool-solid-tools-instructor =
    Tools → Solid Tools: pick an operation, then click the first solid group, then the second.
    • With solids selected the operation runs at once.
    • Subtract / Trim cut the second solid with the first (Trim keeps the first).
    • Split makes three pieces; Outer Shell / Union keep adding clicked solids.
    • A solid: every edge bounds exactly two faces, no nested groups.
tool-follow-me-instructor = Select a path, then click the profile face to sweep it.
tool-tape-instructor = Click two points to measure. Starting on an edge creates a parallel guide.
tool-protractor-instructor = Click the center, click the base direction, click the angle to create a guide.
tool-axes-instructor =
    Click the new origin, click a point along the red axis, then a point toward the green axis.
    Type `x,y,z` for the origin. Right-click the axes for Place / Move / Reset.
tool-dimension-instructor =
    Click a start point and an end point (or click an edge), then move to offset the dimension and click.
    Double-click a dimension's text with Select to edit it; empty text restores the measured length.
tool-text-instructor =
    Click a point, edge or face, move the text away and click again.
    • Default text: a point's coordinates, an edge's length, a face's area.
    • Click empty space for a text without a leader.
    • Edit the text in Entity Info; move it with Move or by moving the geometry.
tool-section-instructor =
    Click a face to place a section plane on it; it becomes the active cut.
    • The arrows point to the part that stays visible.
    • Select a plane by its border; move it with Move, delete it with Delete.
    • Right-click a plane: Active Cut, Reverse, Align View.
    • View → Section Planes / Section Cuts / Section Fill.
tool-guide-plane-instructor =
    Click three points: origin, a point along red, a point on the plane.
    • Double-click a face (or Enter after the first click): the plane of that face.
    • After the first click, arrows set the normal: → red, ← green, ↑ blue, ↓ last edge.
    • The new plane is active: points in the air land on it, axes and grid follow it.
    • Drag the active plane's handles: origin, normal tip (tilt), red handle (spin).
    • Window → Work Planes… to activate, hide, rename, delete or type angles.
tool-slice-instructor =
    Set the cutting plane like a guide plane: click three points.
    • Double-click a face (or Enter after the first click): the plane of that face.
    • After the first click, arrows set the normal: → red, ← green, ↑ blue, ↓ last edge.
    • Cuts the selection (faces, groups, components), or everything in the active context.
    • Edges are added along the cut; a component shared by other copies is made unique first.
    • %COPY% toggles Split (+ on the cursor): each cut body becomes two groups, closed where it was cut.
tool-orbit-instructor =
    Drag to orbit around the point under the cursor.
    Middle mouse button orbits in any tool; Shift+middle pans.
    Trackpad: two-finger scroll orbits, Shift+scroll pans, pinch zooms.
tool-pan-instructor = Drag to pan the view. Shift + middle mouse button pans in any tool.
tool-zoom-instructor =
    Drag up to zoom in, down to zoom out. The mouse wheel zooms toward the cursor in any tool.
    Type a field of view (e.g. 35) and press Enter. Shift+Z zooms to the whole model.
tool-position-camera-instructor =
    Click to stand at a point, looking the way the camera looks now.
    • Drag from where to stand toward where to look.
    • Type the eye height (default 1676 mm) before clicking.
    • Switches to Look Around afterwards.
tool-look-around-instructor = Drag to turn the view around the eye. Type an eye height and press Enter.
tool-walk-instructor =
    Drag up/down to walk forward/back, left/right to turn.
    • Shift: move up/down and sideways. %CMD%: run.
    • Arrow keys: step and turn. Type an eye height and press Enter.

# { $prompt } is the plugin's prompt, { $index } of { $total } the point being picked.
tool-plugin-pick = { $prompt } ({ $index } of { $total }; Esc cancels).
