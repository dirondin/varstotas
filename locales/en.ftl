# Varstotas UI texts: English (the base language: every key is here).
# Sections follow the areas of the keys (the first word of a key). See docs/LOCALIZATION.md.

## ======== Common ========

language-name = English

# The system file manager, for "Show in …" / "Open … Folder" items.
file-manager = file manager
    .macos = Finder
    .windows = Explorer

# Show a file in the system file manager.
reveal-in-file-manager = Show in File Manager
    .macos = Reveal in Finder
    .windows = Show in Explorer

ok = OK
cancel = Cancel
close = Close
apply = Apply
delete = Delete

## ======== Application ========

app-edit-dimension-text = Edit Dimension Text
app-dimension-text-hint = Empty restores the measured length; `<>` stands for it.
# { $step } is the grid step with its unit.
app-grid-step = Grid step { $step }
# { $input } is what was typed.
app-invalid-grid-step = Invalid grid step: { $input }
app-invalid-value = Invalid value: { $input }
app-undo-move-axes = Move Axes
app-no-credits = No downloaded components with credits in this model
# { $name } is the undo name.
app-flash-undo = Undo { $name }
app-nothing-to-undo = Nothing to undo
app-flash-redo = Redo { $name }
app-nothing-to-redo = Nothing to redo
app-select-to-group = Select something to group first
app-undo-make-group = Make Group
app-undo-make-component = Make Component
app-select-to-explode = Select a group or component to explode
app-undo-explode = Explode
app-undo-hide = Hide
app-undo-unhide-all = Unhide All
# { $name } is the tool's name.
app-not-implemented = { $name } is not implemented yet
app-undo-delete = Delete
app-cant-move-into-itself = Can't move a group or component into itself
app-undo-move-out-of-group = Move Out of Group
app-undo-move-into-group = Move into Group
app-undo-add-scene = Add Scene
app-undo-update-scene = Update Scene
app-undo-delete-scene = Delete Scene
app-undo-move-scene = Move Scene
app-undo-cut = Cut
app-no-guides = There are no guides
app-snapping-on = Snapping on
app-snapping-off = Snapping off
app-select-one-component = Select one component
app-undo-make-unique = Make Unique
# { $count } is the number of plugins.
app-plugins-loaded = { $count } plugins loaded
app-select-one-group-or-component = Select one group or component
app-undo-component-parameters = Component Parameters
app-undo-make-parametric = Make Parametric
app-undo-active-cut = Active Cut
app-undo-reverse-section = Reverse Section
app-undo-soften-edges = Soften Edges
app-select-faces-to-reverse = Select faces to reverse
app-undo-reverse-faces = Reverse Faces
app-select-face-to-orient = Select a face to orient its neighbours like it
app-undo-orient-faces = Orient Faces
app-faces-agree = All connected faces already agree
# { $count } is the number of faces.
app-faces-reversed = { $count ->
        [one] { $count } face reversed
       *[other] { $count } faces reversed
    }
app-move-axes-hint = Offset along red, green, blue and an optional rotation about blue: `dx, dy, dz[, angle]`.
app-undo-reset-axes = Reset Axes
app-select-to-flip = Select something to flip
app-undo-flip-red = Flip Along Red
app-undo-flip-green = Flip Along Green
app-undo-flip-blue = Flip Along Blue
app-select-to-intersect = Select faces, groups or components to intersect
app-undo-intersect-model = Intersect With Model
app-undo-intersect-selection = Intersect With Selection
app-undo-intersect-context = Intersect With Context
app-no-intersections = No intersections found
# { $count } is the number of edges.
app-intersections-created = { $count ->
        [one] { $count } intersection edge created
       *[other] { $count } intersection edges created
    }
app-nothing-to-copy = Nothing selected to copy
# { $error } is the error text.
app-copy-failed = Copy failed: { $error }
app-text-no-characters = The text has no visible characters
app-clipboard-empty = The clipboard is empty
# { $factor } is the scale factor.
app-resized = Resized ×{ $factor }
app-scene-update = Update
app-scene-add = Add
app-scene-delete = Delete
app-scene-move-left = Move Left
app-scene-move-right = Move Right
app-pause-animation = Pause Animation
app-play-animation = Play Animation
# { $path } is a file path, { $error } the error text.
app-cant-use = Can't use { $path }: { $error }
app-undo-add-material = Add Material
app-dropped-material = Material { $name } added: click a face to paint it
app-no-scenes = No scenes to play. Add scenes in the Scenes panel.
# { $path } is a file path.
app-opened = Opened { $path }
app-could-not-open = Could not open { $path }: { $error }
app-saved = Saved { $path }
app-save-failed = Save failed: { $error }
# { $warning } is a warning text.
app-exported-warning = Exported { $path }. { $warning }
app-exported = Exported { $path }
app-export-failed = Export failed: { $error }
app-write-denied-windows = Windows denied access. If the folder is Documents, Pictures or Desktop, Defender's Controlled folder access may be blocking Varstotas: Windows Security → Virus & threat protection → Ransomware protection → Allow an app through Controlled folder access.
# { $format } is the format name.
app-undo-import = Import { $format }
app-imported = Imported { $path }
app-autosave-failed = Autosave to { $path } failed: { $error }
# { $name } is a file name.
app-recovered = Recovered { $name } from autosave
app-could-not-recover = Could not recover { $name }: { $error }

## File dialog filters

app-filter-image = Image
app-filter-script = Script
app-filter-model = Varstotas model

## ======== Menus ========

menu-intersect-faces = Intersect Faces
menu-flip-along = Flip Along
menu-axes-place = Place
menu-axes-move = Move…
menu-axes-reset = Reset
menu-active-cut = Active Cut
menu-reverse = Reverse
menu-align-view = Align View
menu-unfold = Unfold…
menu-texture = Texture
menu-texture-position = Position
menu-uv-editor = UV Editor…
menu-mark-seam = Mark as Seam
menu-clear-seam = Clear Seam
menu-file = File
menu-new-from-template = New from Template
menu-open-recent = Open Recent
menu-preview-image = Preview Image
menu-preview-image-hint = The picture the Components library shows for this file (else it draws the model)
menu-import = Import
menu-export = Export
menu-unfold-export = Unfold (SVG / PDF / DXF)…
menu-edit = Edit
menu-view = View
menu-face-style = Face Style
menu-shaded = Shaded
menu-shaded-textures = Shaded with Textures
menu-monochrome = Monochrome
menu-hidden-line = Hidden Line
menu-wireframe = Wireframe
menu-realistic = Realistic
menu-x-ray = X-Ray
menu-edge-style = Edge Style
menu-edges = Edges
menu-edges-hint = Edges in this face style (Realistic has its own switch)
menu-profiles = Profiles
menu-guides = Guides
menu-axes = Axes
menu-ground = Ground
menu-sky = Sky
menu-sky-hint = The sky gradient behind the model; off = a plain light background
menu-grid = Grid
menu-shadows = Shadows
menu-section-planes = Section Planes
menu-section-cuts = Section Cuts
menu-section-fill = Section Fill
menu-viewports = Viewports
menu-viewports-one = One
menu-viewports-two = Two (side by side)
menu-viewports-four = Four (2 × 2)
menu-presets = Presets
menu-animation = Animation
menu-pause = Pause
menu-play = Play
menu-next-scene = Next Scene
menu-previous-scene = Previous Scene
menu-add-scene = Add Scene
menu-camera = Camera
menu-standard-views = Standard Views
menu-parallel-projection = Parallel Projection
menu-perspective = Perspective
menu-draw = Draw
menu-shapes = Shapes
menu-tools = Tools
menu-solid-tools = Solid Tools
menu-snapping = Snapping
menu-extensions = Plugins
menu-window = Window
menu-side-bar = Side Bar
menu-sheets = Sheets…
menu-help = Help
menu-user-guide = User Guide
menu-user-guide-hint = How to use Varstotas: tools, menus, files, keys (F1 over the model: the active tool)
menu-examples = Examples
menu-examples-hint = Open the repository examples in your browser
menu-script-reference = Script Reference
menu-script-reference-hint = Scripts, generators and component scripts: functions, properties, examples
menu-licences = Licences…
menu-licences-hint = Varstotas's licence, the open-source components, file formats and online libraries with their terms
menu-about = About Varstotas
menu-increments = Increments
menu-all-on = All On
menu-all-off = All Off
menu-show-grid = Show Grid
menu-increments-move = Move
menu-increments-rotate = Rotate
menu-increments-scale = Scale
menu-erase = Erase
# { $name } is the undo name of the last change.
menu-undo-named = Undo { $name }
menu-redo-named = Redo { $name }

## ======== Commands ========

cmd-new = New
cmd-open = Open…
cmd-save = Save
cmd-save-as = Save As…
cmd-preview-show = Show…
cmd-preview-from-file = From Image File…
cmd-preview-from-view = From Current View
cmd-preview-remove = Remove
cmd-import-obj = OBJ…
cmd-import-stl = STL…
cmd-import-dxf = DXF…
cmd-import-gltf = glTF / GLB…
cmd-mf = 3MF…
cmd-import-dae = COLLADA (DAE)…
cmd-import-fbx = FBX…
cmd-import-ply = PLY…
cmd-import-csv = Points (CSV)…
cmd-export-image = 2D Graphic (PNG)…
cmd-export-animation = Animation (GIF / PNG)…
cmd-export-glb = glTF (GLB)…
cmd-export-drawing = 2D Drawing (SVG / PDF)…
cmd-sheets = Sheets (PDF / SVG)…
cmd-export-report = Report: Parts and Areas (CSV)…
cmd-export-faces-json = Faces (JSON)…
cmd-export-attributions = Attributions (credits of downloaded models)…
cmd-run-script = Run Script…
cmd-quit = Quit
cmd-undo = Undo
cmd-redo = Redo
cmd-cut = Cut
cmd-copy = Copy
cmd-paste = Paste
cmd-paste-in-place = Paste in Place
cmd-delete = Delete
cmd-purge-unused = Purge Unused
cmd-delete-guides = Delete Guides
cmd-select-all = Select All
cmd-select-none = Select None
cmd-hide = Hide
cmd-unhide-all = Unhide All
cmd-make-group = Make Group
cmd-make-component = Make Component…
cmd-make-unique = Make Unique
cmd-make-parametric = Make Parametric
cmd-explode = Explode
cmd-close-group = Close Group
cmd-intersect-model = With Model
cmd-intersect-selection = With Selection
cmd-intersect-context = With Context
cmd-reverse-faces = Reverse Faces
cmd-orient-faces = Orient Faces
cmd-flip-red = Red Direction
cmd-flip-green = Green Direction
cmd-flip-blue = Blue Direction
cmd-unfold = Unfold…
cmd-physics = Physics…
cmd-text-3d = 3D Text…
cmd-library-3d = 3D Online Library…
cmd-materials-online = Materials Online Library…
cmd-zoom-extents = Zoom Extents
cmd-snapping = Snapping
cmd-show-grid = Show Grid
cmd-side-bar = Side Bar
cmd-render = Render…
cmd-work-planes = Work Planes…
cmd-model-info = Document Info…
cmd-component-params = Component Parameters…
cmd-generator-editor = Generator Editor…
cmd-console = Script Console
cmd-log = Log
cmd-preferences = Preferences…

## Descriptions (one line for the toolbar tooltip)

cmd-desc-flip = Mirror the selection along this axis.
cmd-desc-snap = This snap on / off.
cmd-desc-new = Start a new document.
cmd-desc-open = Open a Varstotas file.
cmd-desc-save = Save the document.
cmd-desc-save-as = Save the document under a new name.
cmd-desc-preview-show = The picture the Components library shows for this file.
cmd-desc-preview-from-file = Use an image file as the document's preview.
cmd-desc-preview-from-view = Use the current view as the document's preview.
cmd-desc-preview-remove = Drop the preview (the library draws the model).
cmd-desc-run-script = Run a Rhai script file on the model.
cmd-desc-quit = Close Varstotas.
cmd-desc-cut = Copy the selection and erase it.
cmd-desc-copy = Copy the selection.
cmd-desc-paste = Place the copied geometry with the mouse.
cmd-desc-paste-in-place = Paste where the geometry was.
cmd-desc-delete = Erase the selection.
cmd-desc-purge-unused = Drop materials and components the model no longer uses.
cmd-desc-delete-guides = Erase every guide line and guide point.
cmd-desc-select-all = Select everything in the active context.
cmd-desc-select-none = Clear the selection.
cmd-desc-hide = Hide the selection.
cmd-desc-unhide-all = Show everything hidden.
cmd-desc-make-group = Wrap the selection in a group.
cmd-desc-make-component = Turn the selection into a component.
cmd-desc-make-unique = Give the selected component its own definition: editing it no longer changes the other copies.
cmd-desc-make-parametric = Give the selected group parameters (Component Parameters).
cmd-desc-explode = Unpack the selected groups and components.
cmd-desc-close-group = Leave the group being edited.
cmd-desc-reverse-faces = Swap the front and back of the selected faces.
cmd-desc-orient-faces = Turn the faces next to the selected one the same way.
cmd-desc-unfold = Lay the selected faces flat (SVG / PDF / DXF).
cmd-desc-physics = Drop the selected groups and components onto the model under gravity, or let a face fall as cloth.
cmd-desc-text-3d = Add text as 3D geometry (a group).
cmd-desc-zoom-extents = Fit the whole model in the view.
cmd-desc-snapping = Snapping on / off.
cmd-desc-show-grid = The grid on the ground on / off.
cmd-desc-side-bar = The side bar with the cards (Properties, Materials …) shown or hidden.
cmd-desc-render = Path-traced pictures of the model (Window → Render).
cmd-desc-work-planes = Saved drawing planes.
cmd-desc-model-info = Statistics of the document, Purge Unused.
cmd-desc-component-params = Parameters of the selected component: values that turn, slide, show or paint its parts.
cmd-desc-library-3d = Free models from online libraries.
cmd-desc-materials-online = Free PBR materials from online libraries.
cmd-desc-uv-editor = Lay out the texture coordinates of the selected faces.
cmd-desc-generator-editor = Write and run generator scripts.
cmd-desc-console = Run Rhai commands on the model.
cmd-desc-log = Errors and warnings from everywhere: scripts, import / export, libraries, downloads …
cmd-desc-preferences = Settings of Varstotas.
# { $name } is "Undo" or "Redo".
cmd-desc-undo-redo = { $name } the last change.
cmd-desc-intersect-model = Add edges where the selected faces cut the model.
cmd-desc-intersect-selection = Add edges where the selected faces cut each other.
cmd-desc-intersect-context = Add edges where the selected faces cut the group being edited.
cmd-desc-solid-outer-shell = Keep the outer shell of two selected solids.
cmd-desc-solid-intersect = Keep the shared volume of two selected solids.
cmd-desc-solid-union = Join two selected solids.
cmd-desc-solid-subtract = Cut the first out of the second of two selected solids.
cmd-desc-solid-trim = Trim the second by the first of two selected solids.
cmd-desc-solid-split = Split into pieces two selected solids.
# { $card } is the name of the card.
cmd-desc-card = The { $card } card of the side bar shown or hidden.
# { $name } is the command's name.
cmd-desc-default = { $name }.

cmd-name-preview-show = Preview Image: Show

cmd-name-preview-from-file = Preview Image from File

cmd-name-preview-from-view = Preview Image from View

cmd-name-preview-remove = Remove Preview Image

cmd-name-intersect-model = Intersect with Model

cmd-name-intersect-selection = Intersect with Selection

cmd-name-intersect-context = Intersect with Context

cmd-name-flip-red = Flip Along Red

cmd-name-flip-green = Flip Along Green

cmd-name-flip-blue = Flip Along Blue

# { $name } is the format's name, as in the menu.
cmd-name-import = Import { $name }
cmd-name-export = Export { $name }
# { $name } is the solid operation's name.
cmd-name-solid = Solid { $name }
# { $name } is the inference's name.
cmd-name-snap = Snap: { $name }
# { $card } is the name of the card.
cmd-name-card = Side Bar: { $card }

## ======== Tools ========

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
tool-follow-me = Sweep
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

## Eraser, Paint Bucket, Push/Pull, Offset, Sweep

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
tool-undo-follow-me = Sweep
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
tool-text-place = Move to place the text and click. Edit the text in Properties.
tool-text3d-status = Click to place the 3D text (Esc cancels).
tool-undo-texture-position = Texture Position
tool-texture-pick = Click a textured face to position its texture.
tool-texture-move = Moving the texture. Esc cancels.
tool-texture-green = Rotating and scaling the texture about the red pin.
tool-texture-blue = Scaling the texture height.
tool-texture-idle = Drag the face or the red pin to move, the green pin to rotate/scale, the blue pin to stretch. Enter or click outside when done.
tool-texture-idle-many = { $count } faces: the texture continues from the picked face. Shift+click adds or removes a face, Alt+click continues it onto another.
tool-texture-cant-continue = The texture can't continue onto that face (it is perpendicular).
tool-texture-vcb = Tile

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

## Tool Guide panel (long help per tool; a line starting with • is a list item)

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
    • Pins snap to vertices and edges; a double-click on an edge aligns the texture to it.
    • Shift+click adds a face, Alt+click continues the texture onto it (round corners).
    • Type the tile size on the face (`800` or `800;400`) or an angle (`30°`).
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
    • The result is a component: change its sizes in Properties; Explode it for plain geometry.
tool-cylinder-instructor =
    Click the centre of the base, click the radius, then move up or down and click to set the height.
    • Type the radius, then the height — or `radius, height` at once; `32s` sets segments.
    • The result is a component: change its sizes in Properties; Explode it for plain geometry.
tool-sphere-instructor =
    Click where the sphere rests, then move and click to set the radius (or type it).
    • `32s` sets segments. The result is a component: change its sizes in Properties.
tool-torus-instructor =
    Click the centre, click the radius (to the middle of the tube), then move and click to set the tube.
    • Type `radius, tube` at once; `32s` sets segments.
    • The result is a component: change its sizes in Properties.
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
    • Edit the text in Properties; move it with Move or by moving the geometry.
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
    • Window → Work Planes to activate, hide, rename, delete or type angles.
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

## ======== Snapping and inference ========

snap-endpoint = Endpoints
snap-midpoint = Midpoints
snap-center = Centers
snap-intersection = Intersections
snap-origin = Origin
snap-on-edge = On Edge
snap-on-face = On Face
snap-on-guide = On Guide Lines
snap-guide-point = Guide Points
snap-on-red-axis = Red Axis
snap-on-green-axis = Green Axis
snap-on-blue-axis = Blue Axis
snap-parallel-to-edge = Parallel to Edge
snap-perpendicular-to-edge = Perpendicular to Edge
snap-from-point = From Point
snap-tangent = Tangent at Vertex
snap-square = Square
snap-golden-section = Golden Section
snap-half-circle = Half Circle
snap-grid-point = Grid Points
snap-on-grid-line = Grid Lines

inference-endpoint = Endpoint
inference-midpoint = Midpoint
inference-center = Center
inference-intersection = Intersection
inference-on-edge = On Edge
inference-on-red-axis = On Red Axis
inference-on-green-axis = On Green Axis
inference-on-blue-axis = On Blue Axis
inference-parallel-to-edge = Parallel to Edge
inference-perpendicular-to-edge = Perpendicular to Edge
inference-from-point = From Point
inference-on-face = On Face
inference-guide-point = Guide Point
inference-on-guide = On Line
inference-origin = Origin
inference-square = Square
inference-golden-section = Golden Section
inference-half-circle = Half Circle
inference-tangent = Tangent at Vertex
inference-grid-point = Grid Point
inference-on-grid-line = On Grid Line

## Snapping menu groups

snap-group-points = Points
snap-group-on-objects = On Objects
snap-group-directions = Directions
snap-group-shapes = Shapes
snap-group-grid = Grid

## ======== Editing ========

edit-nothing-selected = nothing selected
edit-nothing-to-copy = nothing to copy
edit-component-file-empty = the component file is empty
edit-internal-error = internal error
edit-untitled = Untitled
# { $error } is the error text.
edit-parameters-error = Parameters: { $error }
edit-undo-erase = Erase
edit-undo-delete-guides = Delete Guides

# { $label } is the undo name of the operation, { $error } the reason.
edit-failed = { $label } failed: { $error }
edit-op-error = { $label }: { $error }

## ======== Views ========

view-iso = Iso
view-top = Top
view-bottom = Bottom
view-front = Front
view-back = Back
view-left = Left
view-right = Right

## View HUD (the viewport's drop-downs)

view-style-shaded = Shaded
view-style-textured = Textured
view-style-textured-menu = Shaded with Textures
view-style-monochrome = Monochrome
view-style-hidden-line = Hidden Line
view-style-wireframe = Wireframe
view-style-realistic = Realistic
# $label is the style name.
view-style-xray-suffix = { $label } · X-Ray
view-parallel = Parallel
view-perspective = Perspective
# $view is a standard view name, $projection is Parallel / Perspective.
view-label = { $view } · { $projection }
view-parallel-projection = Parallel Projection
view-fov = Field of View
view-fov-tip = The vertical angle the perspective camera sees (also: type it in the Zoom tool)
view-zoom-extents = Zoom Extents
view-viewports = Viewports
view-panes-1 = One view
view-panes-2 = Two views side by side
view-panes-4 = Four views (2 × 2)
view-presets = Viewport Presets
# $label is the current view, e.g. "Top · Parallel".
view-hud-view-tip = View: { $label } — standard views, perspective or parallel projection, field of view, viewports
# $label is the current style.
view-hud-style-tip = Style: { $label } — how faces and edges are drawn (also View → Face Style)
view-xray = X-Ray
view-edges = Edges
view-profiles = Profiles
view-show-axes = Axes
view-show-ground = Ground
view-show-sky = Sky
view-show-grid = Grid
view-show-guides = Guides
view-show-shadows = Shadows
view-show-section-planes = Section Planes
view-show-section-cuts = Section Cuts
view-show-section-fill = Section Fill
view-show-model-header = The model (every view)

## Viewport presets

view-preset-front-iso = Front + Iso
view-preset-top-iso = Top + Iso
view-preset-front-right = Front + Right
view-preset-front-left-top-iso = Front, Left, Top + Iso
view-preset-top-front-right-iso = Top, Front, Right + Iso
view-preset-front-back-left-right = Front, Back, Left, Right

## Viewport messages

view-nothing-rendered = nothing rendered
view-no-renderer = the 3D renderer is not available
# $error is the reason.
view-renderer-unavailable-yet = 3D renderer not available yet ({ $error })
view-renderer-unavailable = 3D renderer not available (wgpu backend required)
view-group = Group
# $path is the chain of group names, e.g. "Table › Leg".
view-editing = Editing: Model › { $path }   (Esc to close)
view-hud-show-tip = Show: axes, ground, sky, grid in this view; guides, shadows, section planes / cuts / fill in every view (also the View menu)

## ======== Toolbar ========

toolbar-size-small = Small
toolbar-size-medium = Medium
toolbar-size-large = Large

## Reserve headings

toolbar-group-draw = Draw
toolbar-group-tools = Tools
toolbar-group-camera = Camera
toolbar-group-file = File
toolbar-group-edit = Edit
toolbar-group-snapping = Snapping
toolbar-group-window = Window

## Buttons

toolbar-snapping-menu = Snapping Menu
toolbar-separator = Separator
toolbar-snapping = Snapping
# $state is "off", "some" or "on".
toolbar-snapping-tip = Choose which points and directions the cursor snaps to. { $state ->
        [off] Snapping is off.
        [some] Some snaps are off.
       *[on] All snaps are on.
    }

## ======== Status bar ========

status-grid-step = Grid Step
status-measurements-tip = Measurements: just type a value and press Enter (e.g. 2500, 2.5m, 300,200, 24s)

## ======== Work planes ========

workplane-title = Work Planes
workplane-none = No work planes. Use Tools → Guide Plane to add one.
workplane-active-tip = Active: draw on this plane
workplane-visible = Visible
workplane-deactivate = Deactivate
workplane-no-active-hint = Without an active plane the drawing axes are used.
# $name is the plane's name.
workplane-world = { $name } (world)
workplane-origin = Origin
workplane-angles = Tilt / azimuth / spin
workplane-tilt-tip = Tilt: the normal's angle from blue
workplane-azimuth-tip = Azimuth: direction of the tilt, from red
workplane-spin-tip = Spin around the normal
workplane-undo-delete = Delete Work Plane
workplane-undo-edit = Edit Work Planes

## ======== Layout sheets and vector export ========

layout-credits-none = Credits: no downloaded models
# $lines is one line per downloaded model.
layout-credits = Credits:
    { $lines }
layout-col-name = Name
layout-col-qty = Qty
layout-col-size = Size
layout-col-material = Material
layout-no-parts = No groups or components
layout-scale-as-shown = As shown
# $n is the scale denominator.
layout-scale-label = Scale 1:{ $n }
layout-tb-author = Author
layout-tb-date = Date
layout-tb-scale = Scale
layout-tb-sheet = Sheet
# $n is the sheet number, $total the count of sheets.
layout-tb-sheet-of = { $n } of { $total }

## ======== Preferences ========

prefs-language = Language
# { $language } is the name of the system's language.
prefs-language-system = As the System ({ $language })
prefs-language-note = Translations of your own: put <code>.ftl into { DATA_DIR() }/locales/ (see docs/LOCALIZATION.md); it also overrides single texts of a built-in language.

prefs-section-general = General
prefs-section-drawing = Drawing
prefs-section-libraries = Libraries
prefs-section-downloads = Downloads
prefs-section-view = View
prefs-section-navigation = Navigation
prefs-section-interface = Interface
prefs-section-shortcuts = Shortcuts
prefs-section-toolbar = Toolbar
prefs-section-render = Render

## Search words of the sections (besides their names; a translation lists its own)

prefs-words-general = auto-save autosave minutes template new document units undo steps history backup previous version bridge python mcp ai agent scripts
prefs-words-render = render image size samples bounces quality denoise programs gif animation oidn open image denoise
prefs-words-drawing = snapping inference endpoint midpoint edge face axis grid step major increments move rotate scale angle
prefs-words-libraries = components materials folder examples in model tiles small large list
prefs-words-downloads = watch downloads folder browser
prefs-words-interface = theme light dark system appearance scale size zoom toolbar icons language translation startup welcome start screen
prefs-words-navigation = mouse trackpad orbit zoom pan speed invert wheel cursor middle button two fingers gesture pinch twist roll multitouch
prefs-words-shortcuts = keyboard shortcuts hotkeys keys
prefs-words-toolbar = toolbar buttons order reserve separator customize tools
prefs-words-view = new document style shaded textures monochrome hidden line wireframe realistic edges profiles axes ground sky grid shadows perspective parallel projection fov field of view camera

## Window

prefs-title = Preferences
prefs-search = Search
prefs-nothing-found = Nothing found
prefs-reset = Reset
prefs-reset-tip = This section's settings back to the defaults
# $section is the section's name.
prefs-defaults-flash = Preferences → { $section }: defaults

## Rows with a folder or a program

# $path is the folder.
prefs-folder-default = { $path } (default)
prefs-file-found = { $path } (found)
prefs-choose = Choose…
prefs-default = Default
prefs-show-in-finder = Show in File Manager
    .macos = Show in Finder
    .windows = Show in Explorer

## General

prefs-data-folder = Data folder
prefs-data-folder-note = Plugins, libraries, templates, your own translations and the bridge for Python scripts and AI agents (bridge.json, python/) are kept here.
prefs-autosave = Auto-save
prefs-autosave-every = Auto-save every
prefs-minutes = minutes
prefs-autosave-note = Unsaved changes are written to { DATA_DIR() }/autosave, not next to the file.
prefs-new-document = New document
prefs-template-note = The template of File → New (units and scale); its view is set in the View section.
prefs-undo = Undo
prefs-keep = Keep
prefs-undo-steps = undo steps per document
prefs-undo-note = Each step keeps a copy of the model: big models with many steps take more memory.
prefs-gpu = Graphics API
prefs-gpu-auto = Automatic
prefs-gpu-restart = Restart now
prefs-gpu-in-use = In use: { $api }
prefs-gpu-fell-back = { $api } did not start last time; Automatic chose instead.
prefs-gpu-note = Automatic tries each API at start in a separate process and takes the first that works (a graphics card before a software renderer); an API that once failed to draw is skipped until the next version. A new choice applies on restart; the document comes along, unsaved changes too (not their undo history).
prefs-gpu-env = Set by the environment variable WGPU_BACKEND.
prefs-gpu-restart-failed = Could not restart: { $error }
prefs-appimage-menu = Applications menu (AppImage)
prefs-appimage-add = Add to the applications menu
prefs-appimage-remove = Remove from the applications menu
prefs-appimage-note = The menu entry, icon and .vrst file type of this AppImage, for you (in ~/.local/share). After moving the AppImage file, start it once: the entry follows it.
appimage-offer-title = Add Varstotas to the applications menu?
appimage-offer-text = This AppImage is not in the applications menu, so the dock shows a generic icon and .vrst files do not open with Varstotas. Add the menu entry, the icon and the .vrst file type for you (in ~/.local/share)? Preferences → General can remove them again.
appimage-offer-add = Add to menu
appimage-offer-no = No, thanks
appimage-added = Varstotas is in the applications menu
appimage-removed = Varstotas is removed from the applications menu
appimage-failed = Could not change the applications menu entry: { $error }
prefs-bridge = Scripts and AI agents
prefs-bridge-external = Let scripts and AI agents outside Varstotas connect (MCP)
prefs-bridge-note = Off: only scripts started from Varstotas (File → Run Script…, plugins) reach the bridge. On: { DATA_DIR() }/bridge.json holds the local address and token for other programs, such as an MCP server for an AI assistant. The bridge listens on 127.0.0.1 only.

## Render

prefs-new-documents = New documents
prefs-match-view = Match the view
prefs-match-view-tip = The height follows the viewport's shape
prefs-denoise = Denoise when the render stops
prefs-denoise-ai = With Open Image Denoise when installed
prefs-use-current-doc = Use Current Document's
prefs-use-current-doc-tip = Take the render settings of the open document
prefs-apply-current-doc = Apply to Current Document
prefs-apply-current-doc-tip = The open document gets these settings (Undo puts them back)
prefs-render-note = File → New starts with these; Window → Render changes the open document's.
prefs-programs = Programs
prefs-oidn = Open Image Denoise library (the render's denoiser)
prefs-oidn-missing = not found: the built-in filter denoises (install Open Image Denoise: brew install open-image-denoise, or your system's package)

## Drawing

prefs-grid = Grid
prefs-show-grid = Show grid
prefs-grid-step = step
prefs-grid-major = major every
prefs-grid-note = Zooming out multiplies the step by the major count. With Select active, typing a length sets the step.
prefs-snapping = Snapping and increments
prefs-snapping-note = The same switches as the Snapping menu.

## Libraries

prefs-components = Components
prefs-library-folder = Library folder
prefs-components-folder-tip = Components (.vrst) and generators (.rhai) of the Components card
prefs-show-examples = Show Examples
prefs-show-examples-tip = The bundled example generators
prefs-show-in-model = Show In Model
prefs-show-in-model-tip = The open document's components and generators
prefs-materials = Materials
prefs-materials-folder-tip = Materials (.vrstmat) of the Materials card; subfolders are categories
prefs-show-mat-examples-tip = The materials bundled with Varstotas (the Material Gallery demo)
prefs-tiles = Tiles
prefs-tiles-note = Both cards; also in their … menus.

## Downloads

prefs-watch-downloads = Watch Downloads
prefs-watch-offer = Offer new model files for import
prefs-watch-offer-tip = Every new model file or archive in the folder is offered for import (always on while a page of Other Sites waits)
prefs-watched-folder = Watched folder
prefs-watched-folder-tip = Where your browser saves files
prefs-downloads-note = Also in Window → 3D Online Library → Other Sites.

## Toolbar

prefs-toolbar-note = The buttons of the toolbar, in order; the others wait in reserve (they stay in the menus).
prefs-toolbar-on = On the toolbar
prefs-toolbar-reserve = Reserve
prefs-toolbar-to-reserve = To the reserve
prefs-toolbar-down = Down
prefs-toolbar-up = Up
prefs-toolbar-to-end = To the end of the toolbar

## Interface

prefs-theme = Theme
prefs-theme-light = Light
prefs-theme-dark = Dark
prefs-theme-system = As the System
prefs-theme-note = The viewport keeps its own colours (sky, ground); sheets and unfold patterns stay white paper.
prefs-scale = Scale
prefs-scale-note = Menus, panels and windows; %CMDK%+ / %CMDK%− / %CMDK%0 change it too.
prefs-toolbar-icons = Toolbar icons

## Shortcuts

prefs-shortcuts-note = Click a shortcut, then press the new keys: Esc cancels, Backspace removes the shortcut. A key another command had is taken from it.
# $command is the command's name.
prefs-shortcut-removed = { $command }: no shortcut
# $keys is the pressed shortcut, $why the reason.
prefs-shortcut-reserved = { $keys } can't be a shortcut: { $why }
# $other is the command that had the shortcut before.
prefs-shortcut-taken = { $command }: { $keys } (taken from { $other })
prefs-shortcut-set = { $command }: { $keys }
prefs-shortcuts-tools = Tools
prefs-press-keys = Press keys…
prefs-default-none = Default: none
# $keys is the list of default shortcuts.
prefs-default-keys = Default: { $keys }

## Navigation

prefs-speed = Speed
prefs-nav-orbit = Orbit
prefs-nav-pan = Pan
prefs-nav-zoom = Zoom
prefs-invert-wheel = Invert the wheel
prefs-invert-wheel-tip = Wheel away from you zooms out
prefs-zoom-cursor = Zoom toward the cursor
prefs-zoom-cursor-tip = Off: toward the middle of the view
prefs-mouse = Mouse
prefs-middle-drag = Middle button drag:
prefs-shift-other = Shift gives the other one.
prefs-trackpad = Trackpad
prefs-two-fingers = Two fingers:
prefs-trackpad-note = Shift swaps orbit and pan. Pinch and %CMD% + two fingers zoom independently of the mouse wheel.

## View (defaults of a new document)

prefs-view-note = How a new document looks (File → New, New from Template). The View menu and the viewport's buttons change the current view.
prefs-camera = Camera
prefs-fov = Field of view
prefs-use-current-view = Use Current View
prefs-use-current-view-tip = Take the style, switches and camera of the active viewport
prefs-apply-current-view = Apply to Current View
prefs-apply-current-view-tip = Set the active viewport like a new document

## ======== Shortcuts ========

key-new = New
key-open = Open…
key-save = Save
key-save-as = Save As…
key-quit = Quit
key-undo = Undo
key-redo = Redo
key-select-all = Select All
key-select-none = Select None
key-make-group = Make Group
key-zoom-extents = Zoom Extents
key-toggle-snapping = Snapping On / Off
key-next-scene = Next Scene
key-prev-scene = Previous Scene

## Key names and reserved keys (Preferences → Shortcuts)

key-name-space = Space
key-name-pgdn = PgDn
key-name-pgup = PgUp
key-reserved-tools = this key is used by every tool
key-reserved-arrows = arrows lock the inference axes
key-reserved-f1 = F1 opens the guide
key-reserved-digits = digits type into Measurements
key-reserved-measurements = this key types into Measurements
key-reserved-clipboard = Cut / Copy / Paste

## ======== Templates ========

template-architectural-mm = Architectural — Millimeters
template-architectural-m = Architectural — Meters
template-woodworking-mm = Woodworking — Millimeters
template-imperial = Imperial — Inches

template-architectural-mm-desc = Buildings and interiors, lengths in millimeters.
template-architectural-m-desc = Buildings and sites, lengths in meters.
template-woodworking-mm-desc = Furniture and small objects: a close-up view, lengths in millimeters.
template-imperial-desc = Lengths in inches.

## ======== Side bar ========

## Cards (titles are ids in the preferences; these are the shown names)

tray-card-entity-info = Properties
tray-card-materials = Materials
tray-card-material-editor = Material Editor
tray-card-components = Components
tray-card-tags = Tags
tray-card-outliner = Outliner
tray-card-scenes = Scenes
tray-card-shadows = Shadows
tray-card-environment = Environment
tray-card-soften-edges = Soften Edges
tray-card-instructor = Tool Guide
tray-card-model-info-tip = Document Info: statistics of the document, Purge Unused
tray-cards-all-off = Every card is off: Window → Side Bar

tray-untagged = Untagged
tray-default = Default
tray-rename = Rename
tray-invalid-value = Invalid value
# $error is the reason.
tray-kernel-not-ready = (kernel not ready: { $error })
tray-through = Through

## Shadows

tray-shadows-show = Show shadows
tray-shadows-time = Time
tray-shadows-date = Date
tray-shadows-light = Light
tray-shadows-dark = Dark
tray-shadows-display = Display
tray-shadows-on-faces = On faces
tray-shadows-on-ground = On ground
tray-shadows-location = Location
tray-shadows-latitude = Latitude
tray-shadows-longitude = Longitude
tray-shadows-utc-offset = UTC offset
tray-shadows-north-angle = North angle
# $alt is the sun's altitude in degrees.
tray-sun-above = Sun { $alt }° above the horizon
tray-sun-below = Sun below the horizon

## Soften Edges

tray-soften-allow-normals = Allow normals between
tray-soften-smooth-normals = Smooth normals
tray-soften-coplanar = Soften coplanar
tray-soften-select = Select edges or faces
# $n is the number of edges.
tray-soften-edges-count = { $n } edges

## Properties

tray-info-invalid-context = Invalid context
tray-info-no-selection = No Selection
tray-info-units = Units
tray-unit-millimeters = Millimeters
tray-unit-centimeters = Centimeters
tray-unit-meters = Meters
tray-unit-inches = Inches
tray-unit-feet = Feet
tray-info-faces = Faces
tray-info-edges = Edges
tray-info-groups = Groups
tray-info-components = Components
tray-info-face = Face
tray-info-area = Area
tray-info-front = Front
tray-info-back = Back
tray-info-tag = Tag
tray-info-edge = Edge
tray-info-length = Length
tray-info-soft = Soft
tray-info-smooth = Smooth
tray-info-group = Group
tray-info-component = Component
tray-info-parametric-group = Parametric Group
# $kind is Group, Component or Parametric Group.
tray-info-solid = Solid { $kind }
tray-info-volume = Volume
tray-info-instance = Instance
tray-info-name-hint = name
tray-info-definition = Definition
tray-info-material = Material
tray-info-source = Source
tray-info-licence-unknown = licence unknown
# $line is the credit line of the model.
tray-info-source-hover =
    { $line }
    File → Export → Attributions lists the credits of the model
# $count is the number of selected items.
tray-info-n-guides = { $count } Guides
tray-info-n-dimensions = { $count } Dimensions
tray-info-n-entities = { $count } Entities
tray-info-contains = Contains
tray-count-faces = { $n ->
        [one] { $n } Face
       *[other] { $n } Faces
    }
tray-count-edges = { $n ->
        [one] { $n } Edge
       *[other] { $n } Edges
    }
tray-count-groups = { $n ->
        [one] { $n } Group
       *[other] { $n } Groups
    }
tray-count-components = { $n ->
        [one] { $n } Component
       *[other] { $n } Components
    }
tray-count-guides = { $n ->
        [one] { $n } Guide
       *[other] { $n } Guides
    }
tray-count-dimensions = { $n ->
        [one] { $n } Dimension
       *[other] { $n } Dimensions
    }

## Undo step names

tray-undo-rename = Rename
tray-undo-soften-edge = Soften Edge
tray-undo-unsoften-edge = Unsoften Edge
tray-undo-set-tag = Set Tag
tray-undo-texture-position = Texture Position
tray-undo-edit-material = Edit Material
tray-undo-edit-segments = Edit Segments
tray-undo-edit-radius = Edit Radius
tray-undo-edit-text = Edit Text
tray-undo-edit-dimension-text = Edit Dimension Text
tray-undo-rename-section = Rename Section
tray-undo-delete-tag = Delete Tag
tray-undo-rename-tag = Rename Tag
tray-undo-add-tag = Add Tag
tray-undo-scene-properties = Scene Properties
tray-undo-rename-scene = Rename Scene

## Parametric group

tray-params-make = Make Parametric
tray-params-make-hover = Size and copies set in Properties
tray-params-title = Parameters
tray-params-length = Length (X)
tray-params-width = Width (Y)
tray-params-height = Height (Z)
tray-params-gap = Gap
tray-params-copies = Copies
tray-params-along = Along
tray-params-resize = Resize
tray-params-stretch = Stretch
tray-params-stretch-hover = Move the far half: ends keep their size
tray-params-scale = Scale

## Texture of a face

tray-tex-title = Texture
tray-tex-offset-x = Offset X
tray-tex-offset-y = Offset Y
tray-tex-rotation = Rotation
tray-tex-scale-x = Scale X
tray-tex-scale-y = Scale Y
tray-tex-reset = Reset
tray-tex-tile-w = Tile width
tray-tex-tile-h = Tile height
uv-title = UV Editor
uv-undo = UV Edit
uv-undo-mark-seams = Mark Seams
uv-undo-clear-seams = Clear Seams
uv-seams-marked = { $count } edges marked as seams.
uv-seams-cleared = { $count } seams cleared.
uv-no-edges = Select edges first (together with the faces).
uv-nothing = Nothing to change.
uv-empty = Select faces with a texture, or paint some, to edit their texture layout.
uv-unfold-hover = Lay each island flat with true lengths, cut at its seams
uv-planar = Planar
uv-box = Box
uv-cylinder = Cylinder
uv-sphere = Sphere
uv-seams-mark-hover = Mark the selected edges as seams
uv-seams-clear-hover = Clear the seams of the selected edges (else all)
uv-seams-hard-hover = Mark every hard (not soft) edge between the faces as a seam
uv-rotate-ccw = Rotate 90° anticlockwise
uv-rotate = Rotate 90° clockwise
uv-flip-u = Flip left ↔ right
uv-flip-v = Flip top ↕ bottom
uv-fit-hover = Scale each island to fill one tile
uv-pack-hover = Lay the islands out in one tile at one scale
uv-align-hover = Turn each island so its longest edge runs straight
uv-reset-hover = Clear the texture placement of the islands (back to the plain mapping)
uv-tool-select = Select: click an island (a face in face mode), Shift+click adds or removes, drag a box over empty space; drag a picked one to move it
uv-tool-move = Move the picked islands or faces (the gizmo's red / green arrow: along one axis)
uv-tool-rotate = Rotate the picked islands or faces about the pivot
uv-tool-scale = Scale the picked islands or faces about the pivot (the gizmo's red / green square: one axis only)
uv-mode-island = Pick whole islands
uv-mode-face = Pick single faces
uv-select-all = Pick everything
uv-frame = Frame all the islands
uv-project-hover = Project the texture: planar, box, cylinder or sphere
uv-reset-rotation = Reset the rotation: turn the islands back so the texture runs straight (the layout keeps its shape)
uv-reset-scale = Reset the scale: the texture back to its own size (the layout keeps its shape)
uv-hint-select = Click / box: pick · click again: the one below · Shift: add or remove · drag a picked one: move · wheel: zoom · right drag: pan
uv-hint-move = Drag: move the picked ones · gizmo arrows: one axis only · snaps to corners and the tile grid (Alt: no snapping; Shift: steps on / off) · type dx; dy
uv-hint-rotate = Drag: turn about the pivot · drag the pivot's centre to move it, double-click it to centre it · Shift: angle steps on / off · type an angle
uv-hint-scale = Drag: scale about the pivot · red / green square: one axis, orange: both · drag the pivot's centre to move it, double-click to centre it · Shift: steps on / off · type k or kx; ky
uv-value-move = Move
uv-value-rotate = Angle
uv-value-scale = Scale
uv-value-hover = An exact value, Enter applies it; right after a drag it replaces that drag
uv-checker-hover = Show a checker instead of the textures (here and in the 3D view) to see stretching
uv-density = Texel density
uv-px-per-m = px/m
uv-summary = { $faces } faces · { $islands } islands · { $selected } selected
uv-material = Material: { $name }
uv-map-hover = Which map of the material is shown behind the layout
uv-no-texture = No texture: the faces and the current material have no image
uv-tile-hover = Repeat the texture over the whole canvas (else one tile only)
tray-tex-width = Texture width
tray-tex-height = Texture height
tray-tex-keep-aspect = Keep aspect ratio

## Curves

tray-curve = Curve
tray-curve-closed = Closed Curve
tray-curve-points = Points
# $count is the number of points, $smooth how many of them are smooth.
tray-curve-points-value = { $count } ({ $smooth } smooth)
tray-curve-segments = Segments
tray-curve-segments-hover = Edges per curved span between two points
tray-curve-edit = Edit Curve
tray-curve-edit-hover = Drag its points and handles (or double-click the curve)
tray-curve-arc = Arc
tray-curve-circle = Circle
tray-curve-polygon = Polygon
tray-curve-radius = Radius
tray-curve-sides = Sides
tray-curve-circumference = Circumference
tray-curve-arc-length = Arc Length
tray-curve-angle = Angle

## Dimension, light, section plane, guides

tray-dim-text = Text
tray-dim-linear = Linear Dimension
tray-light-title = Light
tray-light-render = Render…
tray-light-render-hover = Window → Render
tray-light-hint = Move / Rotate move and turn it; drag the square handle to aim it.
tray-section-title = Section Plane
tray-section-name = Name
tray-section-active-cut = Active Cut
tray-section-normal = Normal
tray-section-reverse = Reverse
tray-section-align-view = Align View
tray-guide-line = Guide Line
tray-guide-direction = Direction
tray-guide-point = Guide Point
tray-guide-position = Position

## Materials card

tray-mat-current = Current paint material (B)
tray-mat-custom = Custom
tray-mat-use = Use

## Tags

tray-tags-add = Add tag
tray-tags-header = Visible · Name
tray-tags-untagged-hover = Toggle visibility of untagged top-level geometry, groups and components (their contents belong to them)
tray-tags-toggle = Toggle visibility
tray-tags-rename-hover = Double-click to rename; right-click for more
tray-tags-delete-hover = Delete the tag (what it tags becomes Untagged)
# $n is the number of the new tag.
tray-tag-new-name = Tag { $n }
# $name is the tag, $uses how many items were tagged with it.
tray-tag-deleted = Deleted tag “{ $name }”
tray-tag-deleted-uses = Deleted tag “{ $name }”: { $uses } item(s) are Untagged now

## Outliner

tray-outliner-empty = No groups or components yet.
tray-outliner-hide-show = Hide / show
tray-outliner-zoom = Zoom To
tray-outliner-zoom-hover = Select it and frame it in the view (double-click)
tray-outliner-edit-group = Edit Group
tray-outliner-edit-component = Edit Component
tray-outliner-unhide = Unhide
tray-outliner-hide = Hide
tray-outliner-explode = Explode
tray-outliner-make-unique = Make Unique
tray-outliner-make-unique-hover = Its own definition: editing it no longer changes the other copies
tray-outliner-move-out = Move Out of Group
tray-outliner-move-out-hover = Into the group / component around this one (or drag the row)

## Scenes

tray-scenes-add = Add scene from current view
tray-scenes-update = Update selected scene to current view
tray-scenes-delete = Delete selected scene
tray-scenes-up = Move scene up
tray-scenes-down = Move scene down
tray-scenes-play = Play
tray-scenes-pause = Pause
tray-scenes-play-hover = Play the scene animation (View → Animation)
tray-scenes-empty = No scenes. Click + to save the current view.
tray-scenes-click-hover = Click to go to this scene, double-click to rename
tray-scenes-include = Include in animation
tray-scenes-animation = Animation
tray-scenes-transitions = Scene transitions
tray-scenes-transition = Transition
tray-scenes-delay = Scene delay

## Tool Guide

# $key is the shortcut.
tray-instructor-shortcut = Shortcut: { $key }
tray-instructor-more = More in the User Guide (F1)
tray-instructor-not-implemented = This tool is not implemented yet.

## Library cards, shared frame (tray_lib)

tray-tiles-small = Small Tiles
tray-tiles-large = Large Tiles
tray-tiles-list = List
tray-lib-search = Search
tray-lib-refresh = Refresh: read the library folder again
tray-lib-more = Library folder, what to show, tile size
tray-lib-section-show = Show the section
tray-lib-section-fold = Fold the section
# $unused is the number of unused items.
tray-purge-materials = Purge Unused: remove the { $unused } materials nothing in the model uses
tray-purge-materials-none = Purge Unused: no unused materials
tray-purge-components = Purge Unused: remove the { $unused } components nothing in the model uses
tray-purge-components-none = Purge Unused: no unused components
tray-lib-fav-add = Add to Favourites
tray-lib-fav-remove = Remove from Favourites
tray-lib-move-to-folder = Move to Folder
tray-my-library = My Library
# $path is the file that is in the way.
tray-lib-exists = { $path } already exists
tray-folder-new = New Folder…
tray-folder-rename = Rename Folder…
tray-folder-delete = Delete Folder…
tray-folder-delete-files = Delete Folder and Its Files…

## ======== Components library ========

## Sections

library-examples = Examples
library-examples-hover = Generators bundled with Varstotas (not files)
library-in-model = In Model
library-in-model-hover = Components and generators of the open document (Save to Library makes a file)
library-favourites = Favourites
library-favourites-hover = Right-click a library tile → Add to Favourites
library-recent = Recent
library-recent-hover = Library components placed last
library-hide-examples = Hide the bundled examples (… → Show Examples brings them back)
library-hide-in-model = Hide the document's components (… → Show In Model brings them back)
# $path is the folder.
library-folder-reveal-hover =
    { $path }
    Click to show in the Finder
    .windows =
    { $path }
    Click to show in Explorer
library-show-in-finder = Show in Finder
    .windows = Show in Explorer
    .linux = Show in File Manager

## Top row and the … menu

library-plus-hover = Save the selection to the library, new generator
library-online-hover = 3D Online Library: free models from the web (Poly Haven, Smithsonian, NASA, Kenney, plugins)
library-save-selection = Save Selection to Library…
library-select-first = Select something first
library-new-generator = New Generator…
library-new-generator-hover = A component built by a script with parameters (Window → Generator Editor)
library-choose-folder = Choose Folder…
library-default-folder = Default Folder
library-default-folder-hover = Back to { DATA_DIR() }/components
library-purge = Purge Unused Components
library-purge-hover = Remove the components (and group definitions) nothing in the model uses
library-show-examples = Show Examples
library-show-examples-hover = The bundled example generators
library-show-in-model = Show In Model
library-show-in-model-hover = The open document's components and generators
library-empty = No components yet: select something and use + → Save Selection to Library.
library-nothing-found = Nothing found
library-image-filter = Image
library-name = Name
library-save = Save
# $n is the number of items.
library-unreadable-files = { $n } file(s) could not be read
library-unreadable-generators = { $n } generator(s) could not be read

## Tiles and their context menus

# $name is the component, $note an extra line (ends with a line break, or empty), $x $y $z the size.
library-tile-hover =
    { $name }
    { $note }{ $x } × { $y } × { $z }
    Click to place
# $uses is how many copies are placed.
library-in-document = In this document: { $uses } placed
library-place = Place
library-select-instances = Select Instances
library-preview-from-view = Set Preview from View
library-preview-from-view-file-hover = The viewport as it is now becomes the picture (saved in the file): frame the model first
library-preview-from-view-def-hover = The viewport as it is now becomes the tile's picture (saved with the document): frame the component first
library-preview-image = Set Preview Image…
library-preview-image-file-hover = A picture shown here instead of the drawn model (saved in the file)
library-preview-image-def-hover = A picture shown on the tile instead of the rendered model
library-rename-ellipsis = Rename…
library-delete-ellipsis = Delete…
library-save-to-library = Save to Library
library-save-to-library-hover = Write it to the library folder, for any document
library-parameters = Parameters…
library-parameters-hover = Window → Component Parameters: values that move, show or paint its parts
library-delete-from-document = Delete from Document…
library-delete-from-document-hover = The component and all its placed copies

## Preview pictures

library-undo-set-preview = Set Preview Image
library-undo-remove-preview = Remove Preview Image
library-undo-set-def-preview = Set Component Preview
library-undo-remove-def-preview = Remove Component Preview
library-preview-set-file = Preview image set (saved with the file)
library-preview-removed = Preview image removed
# $name is the component.
library-preview-set-def = Preview image of { $name } set (saved with the document)
library-preview-removed-def = Preview image of { $name } removed
# $path is the image file, $error the reason.
library-preview-error-file = Preview image: { $path }: { $error }
library-preview-error = Preview image: { $error }

## Saving, placing, moving

# $path is a file, $name a component, $error the reason.
library-exists = { $path } already exists
library-save-error = Save to Library: { $error }
library-saved = Saved to the library: { $path }
library-save-failed = Save to Library failed: { $error }
library-place-failed = Place { $name }: { $error }
library-no-copies = No copies of this component at the top level of the model
library-move-failed = Move to Folder: { $error }

## Folder dialogs

library-folder-new-title = New Folder
library-folder-rename-title = Rename Folder
library-folder-name = Folder name
library-create = Create
library-rename = Rename
# $heading is the dialog's title, $name the folder, $error the reason.
library-folder-done = { $heading }: { $name }
library-folder-failed = { $heading }: { $error }
library-folder-delete-title = Delete Folder?
# $name is the folder.
library-folder-delete-text-materials = The folder “{ $name }” will be deleted from the materials library:
library-folder-delete-text-components = The folder “{ $name }” will be deleted from the components library:
library-folder-empty = The folder is empty.
# $files is a number of files, $dirs a number of subfolders.
library-folder-what = { $files } file(s)
library-folder-what-dirs = { $files } file(s) in { $dirs } subfolder(s)
# $what is the library-folder-what text.
library-folder-not-empty = ⚠ The folder is not empty: { $what } will be deleted with it.
# $n is how many files are not listed.
library-folder-more = … and { $n } more
library-undone-copies-stay = This cannot be undone. Copies already placed in documents stay.
library-folder-delete-with = Delete with { $files } file(s)
library-folder-deleted = Deleted folder { $name }
library-folder-deleted-files = Deleted folder { $name } and { $files } file(s)
library-folder-delete-failed = Delete Folder: { $error }

## Delete dialogs

library-delete-title = Delete from the library?
library-delete-text = “{ $name }” will be deleted from the library folder:
library-delete-doc-title = Delete from the document?
library-delete-gen-text = The generator “{ $name }” will be removed from this document.
# $uses is the number of placed components.
library-delete-gen-uses = { $uses } placed component(s) stay as plain components, without parameters.
library-undo-brings-back = Edit → Undo brings it back.
library-undo-brings-them-back = Edit → Undo brings them back.
library-undo-delete-generator = Delete Generator { $name }
# $n is the number of components left without parameters.
library-generator-removed-uses = Removed the generator { $name }: { $n } component(s) stay without parameters
library-generator-removed = Removed the generator { $name }
library-deleted = Deleted from the library: { $name }
library-delete-failed = Delete { $path }: { $error }
library-undo-delete-component = Delete Component { $name }
# $n is the number of removed copies.
library-component-deleted-copies = Deleted { $name } and { $n } placed copy(ies)
library-component-deleted = Deleted { $name } from the document
library-delete-def-text = “{ $name }” is placed { $uses } time(s). The component and all its copies will be removed.

## Rename dialog

library-rename-material = Rename Material
library-rename-component = Rename Component
# $file is the library file.
library-rename-file-too = The library file { $file } is renamed too.
library-undo-rename-component = Rename Component { $name }
library-renamed = Renamed to { $name }
library-rename-failed = Rename { $path }: { $error }

## ======== Materials ========

## Library card

mat-plus-hover = New material from an image, import material files
mat-online-hover = Materials Online Library: free PBR materials (Poly Haven, ambientCG, plugins)
mat-new-from-texture = New from Texture…
mat-new-from-texture-hover = A material from an image file (PNG, JPEG)
mat-new-procedural = New Procedural…
mat-new-procedural-hover = A seamless texture drawn from a pattern: bricks, tiles, planks, noise or checker
proc-title = Procedural Material
proc-bricks = Bricks
proc-tiles = Tiles
proc-planks = Planks
proc-noise = Noise
proc-checker = Checker
proc-main = Colour
proc-joint = Joints
proc-second = Second colour
proc-count = Pieces across × up
proc-scale = Scale
proc-joint-width = Joint width
proc-variation = Variation
proc-seed = Variant
proc-another = Another
proc-width = Tile width
proc-name = Name
proc-hint = The tile repeats seamlessly; the preview shows 2 × 2 tiles.
proc-create = Create Material
proc-plugin-hover = A pattern from a plugin (@texture_generator)
proc-generating = Generating…
proc-created = Material { $name } created: click faces to paint
mat-import = Import…
mat-import-hover = Add material files (.vrstmat, plugin formats) to the document
mat-choose-folder = Choose Folder…
mat-default-folder = Default Folder
mat-default-folder-hover = Back to { DATA_DIR() }/materials
mat-purge = Purge Unused Materials
mat-purge-hover = Remove the materials nothing in the model uses
mat-show-examples-hover = The materials bundled with Varstotas (the Material Gallery demo)
mat-in-model-hover = The materials of the open document
mat-favourites-hover = Right-click a library material → Add to Favourites
mat-recent-hover = Library materials used last
mat-examples-hover = Materials bundled with Varstotas (the Material Gallery demo)
mat-empty = Empty: right-click a model material → Save to Library
mat-use = Use
mat-use-hover = Paint with it (it is added to the model when something is painted)
mat-export = Export…
mat-save-hover = Write it to the materials library folder, for any document
mat-copy-hover = Copy it to your materials library
mat-delete-model-hover = Remove it from the model: what it paints gets the default material (Undo brings it back)
mat-replace = Replace with Current Material
mat-replace-hover = Repaint everything painted with it with the current paint material and remove it from the model (Undo brings it back)
mat-undo-replace = Replace { $name } with { $with }
mat-replaced = { $name } → { $with }: { $n } repainted
mat-file-filter = Varstotas material
mat-all-filter = All materials

## Tile description (hover)

mat-kind-metal = metal
mat-kind-glass = glass
mat-kind-emissive = emissive
mat-kind-texture = texture
# $name is the shader preset.
mat-kind-shader = shader { $name }
mat-kind-shader-custom = custom
# $value is the roughness, 0 to 1.
mat-kind-roughness = roughness { $value }

## Messages

mat-undo-add = Add Material { $name }
mat-undo-delete = Delete Material { $name }
mat-undo-rename = Rename Material { $name }
mat-saved = Saved to the materials library: { $name }
mat-save-failed = Save material failed: { $error }
mat-exported = Exported { $path }
mat-export-failed = Export material failed: { $error }
# $names are the imported materials, $plugin the importer.
mat-imported-with = Imported { $names } with { $plugin }
mat-import-empty = { $plugin }: no materials in { $path }
mat-import-failed-with = Can't import { $path } with { $plugin }: { $error }
mat-imported = Imported material { $name }
mat-import-failed = Can't import { $path }: { $error }
# $n is the number of painted items put back to the default material.
mat-deleted-uses = Deleted material { $name }, { $n } use(s) back to the default (Undo brings it back)
mat-deleted = Deleted material { $name } (Undo brings it back)
mat-delete-failed = Can't delete material { $name }
mat-rename-exists = There is a material “{ $name }” already
mat-file-exists = { $path } exists already
mat-delete-title = Delete from the materials library?
mat-delete-undone = This cannot be undone. Documents using it keep their copy.
mat-deleted-from-library = Deleted from the materials library: { $name }

## Online materials (window and catalogues)

mat-online-title = Materials Online Library
mat-online-plugins-hint =
    Plugins with @material_source
    are listed here
mat-online-search = Search: brick, oak, concrete…
mat-online-description = Free CC0 photo scans (colour, normal, roughness, occlusion, metal maps at real-world size). Downloads go to the materials library.
mat-online-click-hint = Click a material to see it; double-click downloads it and paints with it.
# $width is a length.
mat-online-wide = { $width } wide
mat-online-use-hover = Already in the library: paint with it
mat-online-download-use = Download & Use
mat-online-download-use-hover = Save it to the library, add it to the model and paint
mat-online-download-hover = Save it to the materials library
# $source is the source's name.
mat-online-loading = Loading the { $source } catalogue…
mat-online-load-failed = Can't load the catalogue: { $error }
# $n is the number of shown materials.
mat-online-count = { $n } materials
mat-online-load-hover = Ask the plugin for its materials with these values
mat-online-downloaded = Downloaded { $name } to the materials library
mat-terms-polyhaven = Powered by Poly Haven (polyhaven.com). The textures are CC0: free for any use, also commercial; no credit needed.
mat-terms-ambientcg = ambientCG materials are CC0: free for any use, also commercial; credit optional.
mat-err-not-catalogue = not a catalogue
mat-err-no-colour-map = no colour map

## Material Editor
mated-window-title = Material Editor
mated-none = No material: pick a swatch or a library material, or take one from the selection.
mated-not-in-model = Not in the model yet: it is added when you paint with it or edit it.
mated-used = { $count ->
        [one] Used on { $count } face or object
       *[other] Used on { $count } faces and objects
    }
mated-pick = Pick from selection
mated-pick-tip = Eyedropper: make the material of the selected face or object current
mated-pick-none = Select a painted face or object to pick its material
mated-picked = Current material: { $name }
mated-select-tip = Select the faces and objects in this context painted with this material
mated-selected = { $count ->
        [one] Selected { $count } entity
       *[other] Selected { $count } entities
    }
mated-window-tip = Open the editor in a window
mated-basic = Basic
mated-color = Colour
mated-opacity = Opacity
mated-texture = Texture
mated-no-texture = No texture
mated-look = Look
mated-hue = Hue
mated-saturation = Saturation
mated-lightness = Lightness
mated-cutoff = Alpha cut-off
mated-colorize = Colorize
mated-colorize-tip = Tint the image with the material colour, keeping its brightness
mated-decal = Decal
mated-decal-tip = Show one tile without repeating: around it the material colour shows
mated-world = World mapping
mated-world-tip = Coordinates from the world position (box projection): ignores Texture Position, patterns continue across faces
mated-look-reset = Reset look
mated-undo-look = Texture look
mated-undo-color = Edit material colour
mated-textures = Textures
mated-slot-base = Base texture
mated-slot-empty = Empty
mated-view-tip = View large
mated-downscale = Downscale
mated-downscale-tip = Large image: the renderer samples at most 1024 px. Re-encode at { $side } px to keep the document small
mated-load-tip = Load an image…
mated-replace-tip = Replace with another image… (the tile size is kept)
mated-clear-tip = Remove this texture
mated-export-tip = Save the image as a file…
mated-edit-tip = Edit in the default image editor for the file: saving there updates the material
mated-map-set = Load map set from folder…
mated-map-set-tip = Fill the slots from a folder by file-name suffixes: _color / _albedo / _diff, _normal (OpenGL preferred), _rough, _metal, _ao, _emission
mated-watching = { $count ->
        [one] Watching { $count } texture open in the image editor
       *[other] Watching { $count } textures open in the image editor
    }
mated-view-title = { $material }: { $slot }
mated-view-tiled = Repeat 3 × 3
mated-undo-texture = Load Texture
mated-undo-map-set = Load Map Set
mated-undo-clear = Clear Texture
mated-undo-downscale = Downscale Texture
mated-undo-reload = Reload Texture
mated-no-maps = No images with map suffixes in { $path }
mated-maps-loaded = { $count ->
        [one] Loaded { $count } map
       *[other] Loaded { $count } maps
    }
mated-exported = Saved { $path }
mated-editing = Editing { $file }: save it to update the material
mated-cant-open-editor = Can't open the image editor: { $error }
mated-reloaded = { $slot } reloaded from the image editor

## ======== 3D Online Library ========

model-lib-licence-any = Any licence
model-lib-licence-commercial = Commercial use OK
model-lib-licence-cc0 = CC0 only
model-lib-licence-known = Known licence

## Window and filters

model-lib-title = 3D Online Library
model-lib-source = Source
model-lib-source-count = { $label } ({ $n })
model-lib-plugins-hint =
    Plugins with @model_source
    are listed here
model-lib-search-enter = Search, then Enter: chair, skull…
model-lib-search = Search: chair, rock, plant…
model-lib-all-categories = All categories
# $description is the source's description.
model-lib-downloads-go = { $description } Downloads go to the components library.
model-lib-from-url = From URL
model-lib-import = Import
model-lib-import-hover = Download the file into the components library and place it
model-lib-click-hint = Click a model to see it; double-click downloads it and places it.
model-lib-type-search = Type a search and press Enter.
model-lib-loading = Loading { $source } models…
model-lib-load-failed = Can't load the models: { $error }
model-lib-retry = Retry
# $n is the number of shown models.
model-lib-count = { $n } models
model-lib-show = Show
model-lib-format-any = Any format
model-lib-size-any = Any size
model-lib-size-10k = ≤ 10k triangles
model-lib-size-50k = ≤ 50k triangles
model-lib-size-100k = ≤ 100k triangles
model-lib-clear = Clear
model-lib-reload = Reload
model-lib-load = Load
model-lib-choose-with = choose with …
model-lib-choose-it = Choose it
model-lib-plugin-apply-hover = Use these values (then search)
model-lib-plugin-load-hover = Ask the plugin for its models with these values

## Details of a model

# $n is a number of triangles.
model-lib-triangles = { $n } triangles
# $value is the size in megabytes.
model-lib-megabytes = { $value } MB
model-lib-heavy = ⚠ A heavy model: it may slow the view down.
model-lib-downloading = Downloading…
# $n is the number of models of the pack in the library.
model-lib-in-components = ✔ { $n } models in Components
model-lib-show-in-components = Show in Components
model-lib-download-browser = Download in Browser
model-lib-download-browser-hover = Open the pack's download in your browser; the archive is imported from Downloads with its credit
model-lib-download-again = Download Again
model-lib-download-pack = Download Pack
model-lib-download-pack-hover = Download the pack: each model becomes a component of the library
model-lib-place-hover = Already in the components library: place it in the model
model-lib-download-place = Download & Place
model-lib-download-place-hover = Save it to the components library and place it in the model
model-lib-download = Download
model-lib-download-hover = Save it to the components library
model-lib-web-page = Web Page

## Messages

# $name is the model.
model-lib-cancelled = Download of { $name } cancelled
model-lib-fav-added = ☆ { $name } added to Favourites
model-lib-fav-removed = { $name } removed from Favourites
model-lib-paste-url = Paste the address of a model file (https://…/model.glb)
model-lib-browser-download = Download { $name } in the browser: Varstotas imports the archive from Downloads
# $n is the number of models, $folder the library folder, $pack the pack's folder.
model-lib-downloaded-pack = Downloaded { $n } models to Components → { $folder } → { $pack }
model-lib-downloaded = Downloaded { $name } ({ $faces } faces) to Components → { $folder }
model-lib-not-in-library = { $path } is not in the components library
model-lib-download-failed = Download failed: { $error }
model-lib-import-failed = Import { $path }: { $error }
model-lib-undo-import = Import { $name }
model-lib-imported = Imported { $path }

## Sources (online_models)

model-lib-source-favourites = Favourites
model-lib-source-downloaded = Downloaded
model-lib-terms-polyhaven = Powered by Poly Haven (polyhaven.com). The models are CC0: free for any use, also commercial; no credit needed.
model-lib-terms-smithsonian = Most objects are CC0, but some are marked «Usage conditions apply» (non-commercial only). Check the object's page before commercial use; trademarks and other rights in the objects themselves are not covered.
model-lib-terms-nasa = NASA media are generally not copyrighted, but the NASA insignia and logos on models are protected, and nothing may suggest NASA endorses your product.
model-lib-terms-kenney = Kenney's packs are CC0: free for any use, also commercial; credit optional. Kenney's logo is not part of it.
model-lib-terms-quaternius = Quaternius' packs are CC0: free for any use, also commercial; credit optional.
model-lib-terms-saved = Each model keeps the licence it came with (its details; File → Export → Attributions). Varstotas warns when a placed or exported model's licence limits its use.
model-lib-desc-polyhaven = Free CC0 models (glTF, 1K textures) at their real size.
model-lib-desc-smithsonian = Smithsonian Open Access 3D: scans of museum objects. Type a search and press Enter.
model-lib-desc-nasa = NASA 3D Resources: spacecraft, rovers, planets (GLB).
model-lib-desc-kenney = Kenney's game asset packs (CC0): a pack downloads whole, each model becomes a component in Components → Kenney → <pack>.
model-lib-desc-quaternius = Quaternius' low-poly packs (CC0). They are downloaded in your browser (Google Drive): Varstotas opens the page and imports the archive from Downloads.
model-lib-desc-favourites = Models you marked ☆ (click ☆ in a model's details).
model-lib-desc-history = Everything downloaded or imported from Downloads, newest first. Type to search.

## Errors of catalogues and imports

model-lib-err-not-catalogue = not a catalogue
model-lib-err-not-search = not a search answer
model-lib-err-kenney-packs = no packs found on kenney.nl (the page changed?)
model-lib-err-quaternius-packs = no packs found on quaternius.com (the page changed?)
model-lib-err-plugin-array = the plugin must return an array of models
model-lib-err-no-download = no download on the pack's page
model-lib-err-quaternius-browser = Quaternius packs are downloaded in the browser: use Download in Browser
model-lib-err-importer-depth = importer plugins call each other too deep
model-lib-err-empty-answer = an empty answer
model-lib-err-plugin-nothing = the plugin returned nothing
model-lib-err-spec-object = a model spec must be an object
model-lib-err-spec-empty = the model spec has no file, url, zip or meshes
model-lib-err-archive-no-model = no model file (glb, gltf, fbx, dae, obj, 3mf, ply, stl, dxf) in the archive
# $k is the number of the mesh in the answer.
model-lib-err-mesh-vertices = mesh { $k }: no vertices
model-lib-err-mesh-faces = mesh { $k }: no faces
model-lib-err-no-faces = no faces in the meshes
model-lib-err-model-empty = the model is empty
model-lib-err-no-size = the model has no size
model-lib-err-no-home = no home folder
model-lib-err-archive-none = no model files in the archive
model-lib-err-none-read = no models could be read

## ======== Downloads ========

## Messages
# $path is a file.
downloads-no-models = { $path }: no model files in it
downloads-no-address = { $path }: no web address in it
downloads-not-model = { $path }: not a model file (glb, gltf, fbx, dae, obj, 3mf, ply, stl, dxf, varstotas, zip)
# $site is the site, $title the model.
downloads-waiting-site = Waiting for a file from { $site }: download it in the browser
downloads-waiting-file = Waiting for the file of { $title } ({ $site }): download it in the browser
downloads-no-library = No components library folder
# $name is the file, $folder the library folder, $n the number of models.
downloads-imported = Imported { $name } to Components → { $folder }
downloads-imported-many = Imported { $n } models of { $name } to Components → { $folder }
downloads-import-failed = Import { $name }: { $error }
downloads-err-no-file-name = no file name
downloads-err-no-model-chosen = no model chosen
# $label is a licence, $names the first models under it, $more " +N" or empty, $note what it asks.
downloads-licence-part = { $label } ({ $names }{ $more }) — { $note }
# $list is the licence-part texts.
downloads-licences = Licences: { $list }. File → Export → Attributions lists the credits.

## Search on, Other Sites

downloads-search-on = Search on:
# $site is the site, $query the search.
downloads-search-hover = Open { $site }'s search for «{ $query }» in the browser
downloads-edit = Edit…
downloads-edit-hover = Edit the sites ({ DATA_DIR() }/sources.toml); the list is read again
downloads-other = Other Sites
# $waiting is the number of waiting downloads.
downloads-other-waiting = Other Sites — waiting for { $waiting } download(s)
downloads-other-text = Download from any site in your browser: Varstotas doesn't visit the sites itself. It takes the file from your Downloads folder and keeps the page as the credit.
downloads-watch = Watch Downloads
downloads-watch-hover = Offer every new model file or archive of the Downloads folder for import (always on while a page waits)
# $dir is the watched folder.
downloads-watching = watching { $dir }
downloads-off = off
downloads-import-file = Import File…
downloads-import-file-hover = Import a model file or an archive you downloaded
downloads-models-filter = Models
# $title is the model, $site the site, $licence its licence.
downloads-waiting-card = Waiting for a file from this page: { $title } ({ $site }, { $licence })
downloads-page = Page
# $file is a file address.
downloads-browser-file = Download in the browser: { $file }
downloads-stop = Stop waiting

## Import Download window

downloads-import-title = Import Download
downloads-importing = Importing…
# $name is the file, $mb its size in megabytes.
downloads-file-size = { $name }  ({ $mb } MB)
downloads-credit-from-page = ✔ The credit is from the page you sent
downloads-duplicate = ⚠ The same file was imported before
downloads-field-title = Title
downloads-field-author = Author
downloads-field-site = Site
downloads-licence = Licence
downloads-licence-hint = as the page says
downloads-place = Place it in the model
# $on models are chosen of $n.
downloads-models-count = { $on } of { $n } models (each becomes a component)
downloads-all = All
downloads-none = None
# $folder is the library folder of the site.
downloads-destination = Components → { $folder }
downloads-ignore = Ignore
# $n is the number of other offers.
downloads-more-waiting = { $n } more waiting

## ======== Render ========

render-dof-view = As view
render-dof-off = Off
render-dof-on = On

## Quality presets and image sizes (the ids stay English)

render-preset-draft = Draft
render-preset-preview = Preview
render-preset-final = Final
render-preset-best = Best
render-size-hd = HD 1280 × 720
render-size-full-hd = Full HD 1920 × 1080
render-size-qhd = QHD 2560 × 1440
render-size-4k = 4K 3840 × 2160
render-size-square = Square 1080
render-size-portrait = Portrait 1080 × 1350

## Months (sun date)

render-month-jan = Jan
render-month-feb = Feb
render-month-mar = Mar
render-month-apr = Apr
render-month-may = May
render-month-jun = Jun
render-month-jul = Jul
render-month-aug = Aug
render-month-sep = Sep
render-month-oct = Oct
render-month-nov = Nov
render-month-dec = Dec

## Export and shader errors

render-export-empty = image size must be at least 1 x 1 pixels
# $w and $h are the requested size, $max the limit per side.
render-export-too-large = image size { $w } x { $h } is too large (at most { $max } pixels per side)
render-export-gpu = GPU error while exporting the image: { $error }
render-shader-clash = does not compile with the other shaders: { $error }

## Render window

render-title = Render
render-current-view = Render the current view
render-stop = Stop
render-continue = Continue
render-more-samples = More Samples
render-more-samples-tip = Double the samples
render-save-png = Save PNG…
render-save-exr = Save EXR…
render-save-exr-tip = Linear light, 32-bit float
render-all-scenes = Render All Scenes…
# $error is the reason.
render-failed = Render failed: { $error }
render-error = Render: { $error }
# $n is the number of scenes, $folder where they were saved.
render-scenes-done = Rendered { $n } scene(s) to { $folder }
# $written scenes are saved, scene $next is being rendered.
render-scenes-progress = Rendering scenes: { $written } saved, scene { $next } in progress
# $path is the saved file.
render-saved = Saved { $path }
# $secs is a number with one decimal.
render-status-running = { $samples } / { $max } samples · { $secs } s
# $denoised is "yes" or "no".
render-status-done = Done: { $samples } samples in { $secs } s{ $denoised ->
        [yes] , denoised
       *[no] {""}
    }
render-status-stopped = Stopped at { $samples } samples · { $secs } s
render-undo-lights = Edit Lights
render-undo-settings = Render Settings

## Lights

render-lights = Lights ({ $n })
render-add-point = + Point
render-add-spot = + Spot
render-add-area = + Area
render-on = On
render-light-point = point
render-light-spot = spot
render-light-area = area
render-light-name = Name
render-light-type = Type
render-light-position = Position
render-light-to-centre = To view centre
render-light-to-centre-tip = Move it 2 m above the camera target
render-light-to-camera = To camera
render-light-aim-at-target = Aim at target
render-light-aim-at-target-tip = Shine at the view's target
render-light-aim = Aim
render-colour = Colour
render-light-power = Power
render-light-radius = Radius
render-light-radius-tip = Bulb size: softer shadows
render-light-cone = Cone
render-light-cone-angle = Cone angle
render-light-soft = Soft
render-light-soft-edge = Soft edge
render-size = Size

## Settings

render-sec-image = Image
render-sec-sampling = Sampling
render-sec-camera = Camera
render-sec-light = Light
render-sec-colour = Colour
render-quality = Quality
# $samples and $bounces are numbers.
render-quality-tip = { $samples } samples, { $bounces } bounces
render-match-view = Match view
render-match-view-tip = The view's proportions at this height
render-size-custom = Custom
render-transparent = Transparent background
render-transparent-tip = The sky and the ground behind the model are transparent in the PNG (they still light it)
render-samples = Samples
render-time-limit = Time limit
render-time-limit-tip = 0 = until the samples are done
render-bounces = Bounces
render-clamp = Clamp
render-clamp-tip = Brightest indirect light of a sample: lower kills fireflies, higher keeps bright bounce light; 0 = off
render-denoiser = Denoiser
render-denoiser-off = Off
render-denoiser-builtin = Built-in
render-denoiser-builtin-tip = Edge-aware filter
render-denoiser-ai = AI (Open Image Denoise)
render-denoiser-install = Install Open Image Denoise: brew install open-image-denoise
render-dof = Depth of field
render-dof-tip = As view: the Effects' depth of field when it is on
# $mm is the equivalent focal length.
render-lens = { $mm } mm lens
render-lens-tip = The view's field of view on a 35 mm camera
render-aperture = Aperture
render-aperture-tip = Smaller f-number = shallower focus
render-focus = Focus
render-focus-target = Target
render-focus-target-tip = Focus on the view's target (orbit centre)
render-sun = Sun
render-sun-tip = Sun strength (its direction: Shadows)
render-sky = Sky
render-sky-tip = Environment light
render-rotation = Rotation
render-rotation-tip = Turns the sky / HDRI
render-exposure = Exposure
render-env-exposure-tip = The environment's exposure (view and render)
render-exposure-tip = Added to the environment's (render only)
render-tone-curve = Tone curve
# $name is the environment's tone curve.
render-tone-environment = Environment: { $name }
render-white-balance = White balance
render-white-balance-tip = 6500 K = neutral; higher is warmer
render-tint = Tint
render-tint-tip = Green (−) .. magenta (+)
render-saturation = Saturation
render-contrast = Contrast
render-vignette = Vignette
render-reset = Reset
render-reset-tip = Back to the default colour settings
render-shader-define-surface = define fn surface(s: Surface, p: Params) -> Surface
# $line is the line in the user's shader code, $msg the compiler's message.
render-shader-line = line { $line }: { $msg }

## ======== Realistic style ========

realistic-undo-material = Edit Material
# $path is a file, $error the reason.
realistic-cant-use = Can't use { $path }: { $error }
realistic-cant-read = Can't read { $path }: { $error }
realistic-used-by = Used by the Realistic style
realistic-load = Load…
realistic-clear = Clear
realistic-reset = Reset
realistic-preset = Preset

## Environment

realistic-background = Background
realistic-sky-sky = Sky
realistic-sky-color = Color
realistic-sky-hdri = HDRI
realistic-sky-physical = Physical sky
realistic-sky-physical-tip = Daylight for the sun's position in the Shadows card
realistic-haze = Haze
realistic-haze-tip = Turbidity: 2 = clear blue sky, 10 = hazy; the sun's time and date are in the Shadows card
realistic-hdri-none = none (sky)
realistic-hdri-tip = An equirectangular panorama (.hdr)
realistic-rotation = Rotation
realistic-intensity = Intensity
realistic-sun = Sun
realistic-sun-tip = Sun strength; its direction comes from the Shadows card when shadows are on
realistic-exposure = Exposure
realistic-tone-mapping = Tone mapping
realistic-tonemap-aces = ACES (filmic)
realistic-tonemap-agx = AgX (soft)
realistic-tonemap-none = None
realistic-show-env = Show the environment behind the model
realistic-effects = Effects

## Physical properties

realistic-physical = Physical (Realistic)
realistic-preset-matte = Matte
realistic-preset-satin = Satin
realistic-preset-glossy = Glossy
realistic-preset-metal = Metal
realistic-preset-chrome = Chrome
realistic-preset-glass = Glass
realistic-preset-light = Light
realistic-metallic = Metallic
realistic-metallic-tip = 0 = paint, plastic, wood; 1 = bare metal
realistic-roughness = Roughness
realistic-roughness-tip = 0 = mirror-like, 1 = matte
realistic-glass = Glass
realistic-glass-tip = How much light passes through
realistic-ior = IOR
realistic-ior-tip = Index of refraction (glass 1.5, water 1.33)
realistic-emission = Emission
realistic-emission-tip = Light the surface gives off: colour and strength
realistic-coat-sheen = Coat, Sheen, Edges
realistic-clearcoat = Clear coat
realistic-clearcoat-tip = A glossy varnish layer: lacquered wood, car paint, glazed tiles
realistic-coat-rough = Coat rough
realistic-coat-rough-tip = Roughness of the varnish (0 = glossy)
realistic-sheen = Sheen
realistic-sheen-tip = A soft glow at grazing angles: velvet, cloth, upholstery
realistic-sheen-tint = Sheen tint
realistic-sheen-tint-tip = 0 = white sheen, 1 = the colour of the material
realistic-anisotropy = Anisotropy
realistic-anisotropy-tip = Stretched highlights along the texture (+) or across it (−): brushed metal
realistic-anti-tiling = Anti-tiling
realistic-anti-tiling-tip = Breaks up the visible repetition of the textures
realistic-bevel = Bevel, mm
realistic-bevel-tip = Rounds the edges of faces in the render (path tracing only)
realistic-shader = Shader
realistic-shader-tip = A WGSL surface program (docs/SHADERS.md): procedural patterns, waves, glow
realistic-shader-none = None
realistic-shader-custom = Custom
realistic-shader-edit = Edit…
realistic-shader-edit-tip = Write or change the shader
realistic-map-normal = Normal
realistic-map-roughness = Roughness
realistic-map-metallic = Metallic
realistic-map-occlusion = Occlusion
realistic-map-emission = Emission
realistic-normal-strength = Normal strength
realistic-remove-physical = Remove physical properties

## Shader editor

# $material is the material's name.
realistic-shader-window = Shader — { $material }
realistic-start-from = Start from…
realistic-start-from-tip = Replace the code with this shader
realistic-checker-new = Checker (new)
realistic-reference = Reference
realistic-reference-tip = Surface fields, helpers and built-ins
realistic-compiles = Compiles; the material uses it
realistic-ref-surface = Surface (s, o)
realistic-ref-parameters = Parameters
realistic-ref-functions = Functions

## Effects

realistic-fx-off = Off
realistic-fx-fast = Fast
realistic-fx-balanced = Balanced
realistic-fx-quality = Quality
realistic-fx-soft-shadows = Soft shadows
realistic-fx-soft-shadows-tip = Shadows blur away from what casts them
realistic-fx-sun-size = Sun size
realistic-fx-ao = Ambient occlusion
realistic-fx-ao-tip = Darker creases and contact shadows
realistic-fx-radius = Radius, mm
realistic-fx-strength = Strength
realistic-fx-reflections = Reflections
realistic-fx-reflections-tip = The model mirrored in glossy surfaces (screen space)
realistic-fx-bloom = Bloom
realistic-fx-bloom-tip = Bright light glows
realistic-fx-threshold = Threshold
realistic-fx-dof = Depth of field
realistic-fx-dof-tip = Blur in front of and behind the focus distance
realistic-fx-blur = Blur
realistic-fx-focus = Focus, mm
realistic-fx-focus-tip = 0 = the point at the centre of the view
realistic-fx-fog = Fog
realistic-fx-fog-tip = Haze growing with distance
realistic-fx-fog-distance = Distance, mm
realistic-fx-fog-height = Height, mm
realistic-fx-fog-height-tip = 0 = even fog; else it thins out over this height
realistic-vignette = Vignette
realistic-style = Style
realistic-stylize-photo = Photo
realistic-stylize-pencil = Pencil
realistic-stylize-watercolor = Watercolor
realistic-stylize-technical = Technical
realistic-fx-clay = Clay
realistic-fx-clay-tip = Every material as plain clay (a massing model)
realistic-fx-high-quality = High quality
realistic-fx-high-quality-tip = More samples for soft shadows, occlusion and reflections

## ======== Animation ========

anim-title = Export Animation
anim-no-scenes = no scenes in the animation
anim-no-scenes-flash = No scenes in the animation
# $error is the reason.
anim-error = Animation: { $error }
# $what is a file path, or "N frames in <folder>".
anim-saved = Saved { $what }
anim-render-dropped = the render was dropped
# $n is the frame count, $dir the folder with the PNG frames.
anim-frames-in = { $n } frames in { $dir }
anim-cancelled = cancelled
anim-animate = Animate
anim-scenes = Scenes
anim-add-scenes = Add scenes (with “Include in animation”) first
anim-orbit = Orbit around the view
anim-length = Length
anim-turns-suffix = {" "}turns
anim-holds-hint = Holds and transitions: Scenes card → Animation
anim-size = Size
anim-fps = Frame rate
anim-frames = Frames
anim-as-view = As the view
anim-path-traced = Path traced
anim-output = Output
anim-gif = Animated GIF
anim-png-sequence = PNG sequence
# $done of $total frames.
anim-frame-progress = Frame { $done } / { $total }
anim-encoding = Writing the GIF…
anim-export = Export…

## ======== Unfold ========

unfold-paper-fit = Fit to pieces
unfold-paper-a4 = A4
unfold-paper-a3 = A3
unfold-paper-a2 = A2

## Unfold window and export

unfold-title = Unfold
unfold-selection = Selection
unfold-everything = Everything in the context
unfold-nothing = Unfold: nothing to unfold (select faces or groups)
# $name is the document name, $scale like "1:10".
unfold-footer = { $name } - unfold { $scale }
# $n is the sheet number out of $total.
unfold-footer-sheet = { $name } - unfold { $scale } - sheet { $n } of { $total }
# $oversize is the number of pieces larger than the sheet (0 = none).
unfold-done = Unfolded { $faces } faces into { $pieces } pieces on { $sheets } sheets{ $oversize ->
        [0] {""}
       *[other] {" "}({ $oversize } larger than the sheet)
    }: { $path }
unfold-export-failed = Export failed: { $error }
# $what is "Selection" or "Everything in the context".
unfold-no-faces = { $what }: no faces. Select faces, groups or components.
unfold-summary = { $what }: { $faces } faces → { $pieces } pieces on { $sheets } sheet(s)
unfold-oversize = { $n } piece(s) larger than the sheet: choose a larger paper or scale.
unfold-scale = Scale 1:
unfold-paper = Paper
unfold-landscape = Landscape
unfold-glue-tabs = Glue tabs
unfold-edge-numbers = Edge numbers
unfold-soft-folds = Folds on soft edges
unfold-sheet-of = Sheet { $n } of { $total }
unfold-legend-cut = ━ cut
unfold-legend-valley = ╍ valley
unfold-legend-mountain = ━·━ mountain
unfold-export = Export:

## ======== Shader editor ========

## Bundled presets
shader-preset-wood = Wood
shader-preset-water = Water
shader-preset-reeded-glass = Reeded Glass
shader-preset-glow = Glow

## Completion and call hints (descriptions of the helpers, WGSL built-ins and Surface fields)
shader-fn-surface = the shader: return the changed surface
shader-fn-hash3 = pseudo-random value 0..1 per point
shader-fn-noise = smooth value noise 0..1
shader-fn-fbm = layered noise 0..1 (clouds, marble, warping)
shader-fn-srgb = a colour picked in sRGB (0..1) → linear
shader-fn-hsv = hue / saturation / value (0..1) → linear colour
shader-fn-tangent_of = a unit vector across n
shader-fn-bump = n tilted by the slope of a height field
shader-fn-abs = absolute value
shader-fn-min = the smaller
shader-fn-max = the larger
shader-fn-clamp = x limited to lo..hi
shader-fn-saturate = x limited to 0..1
shader-fn-mix = a + (b − a)·t
shader-fn-step = 0 below edge, 1 from it
shader-fn-smoothstep = smooth 0→1 between lo and hi
shader-fn-fract = x − floor(x)
shader-fn-floor = round down
shader-fn-ceil = round up
shader-fn-round = nearest whole number
shader-fn-sign = −1, 0 or 1
shader-fn-sqrt = square root
shader-fn-pow = x to the power y
shader-fn-exp = e to the power x
shader-fn-log = natural logarithm
shader-fn-sin = sine (radians)
shader-fn-cos = cosine (radians)
shader-fn-tan = tangent (radians)
shader-fn-atan2 = angle of (x, y), radians
shader-fn-length = vector length
shader-fn-distance = length(a − b)
shader-fn-normalize = unit vector along v
shader-fn-dot = dot product
shader-fn-cross = cross product of two vec3
shader-fn-reflect = i mirrored about the normal n
shader-fn-select = t when cond, else f
shader-fn-vec2 = a 2-vector
shader-fn-vec3 = a 3-vector
shader-fn-vec4 = a 4-vector
shader-surface-base = linear base colour
shader-surface-alpha = opacity
shader-surface-metallic = 0 = dielectric, 1 = metal
shader-surface-roughness = 0 = mirror, 1 = matte
shader-surface-transmission = glass: 1 = light passes through
shader-surface-emission = light given off, linear, strength included
shader-surface-normal = world normal, unit, facing the viewer
shader-surface-pos = model position, mm (z = up)
shader-surface-uv = planar texture coordinates, mm
shader-surface-view = unit direction toward the eye
shader-surface-time = seconds
shader-param-tag = a slider of the material editor, read as p.name (at most 8)

## ======== Dialogs ========

## Autosave recovery
dialog-untitled = Untitled
dialog-age-recently = recently
dialog-age-under-minute = less than a minute ago
# $n is a number
dialog-age-minutes = { $n ->
        [one] { $n } minute ago
       *[other] { $n } minutes ago
    }
dialog-age-hours = { $n ->
        [one] { $n } hour ago
       *[other] { $n } hours ago
    }
dialog-age-days = { $n ->
        [one] { $n } day ago
       *[other] { $n } days ago
    }

## Save changes
dialog-save-title = Save changes?
# $name is the document name
dialog-save-question = Do you want to save the changes you made to “{ $name }”?
dialog-save-warning = Your changes will be lost if you don't save them.
dialog-dont-save = Don't Save
dialog-save = Save

## Recover autosaved model
dialog-recover-title = Recover autosaved model?
# $name is a file name; $age is dialog-age-* text ("5 minutes ago")
dialog-recover-copy = Varstotas found an autosaved copy of “{ $name }” from { $age } that is newer than the saved file.
dialog-recover-untitled = Varstotas found an autosaved untitled model from { $age }.
dialog-recover-note = It may contain changes that were not saved, e.g. after a crash.
dialog-discard = Discard
dialog-discard-hint = Delete the autosaved copy
dialog-recover = Recover
dialog-not-now = Not Now
dialog-not-now-hint = Keep the autosaved copy for later

## Resize
dialog-resize-title = Resize?
dialog-resize-model = Do you want to resize the model?
dialog-resize-group = Do you want to resize the active group or component?
# $factor is the scale factor, e.g. 1.2500
dialog-resize-scaled = Everything in it is scaled by { $factor } about its origin.
dialog-resize-keys = Enter = Yes · Esc = No
dialog-yes = Yes
dialog-no = No

## Value prompt
dialog-invalid-value = Invalid value

## Common labels
dialog-width = Width
dialog-height = Height

## Export 2D Graphic
dialog-export-title = Export 2D Graphic
dialog-export-aspect = Use view aspect ratio
dialog-export-viewport = Viewport size
# $w and $h are pixel counts
dialog-export-viewport-hint = { $w } × { $h } px (screen pixels)
dialog-export-double = Double the image size
dialog-export-transparent = Transparent background
dialog-export-axes = Axes
dialog-export-guides = Guides
dialog-export-dims = Dimensions
dialog-export-note = The selection and tool previews are never exported.
dialog-export = Export…
dialog-filter-png = PNG image

## Export 2D Drawing
dialog-drawing-title = Export 2D Drawing
dialog-drawing-intro = The current view as lines: SVG or PDF.
dialog-drawing-scale = To scale 1:
dialog-drawing-perspective = Perspective view: fitted to an A4 page. Camera → Parallel Projection for a scale.
dialog-drawing-fitted = Fitted to an A4 page.
dialog-drawing-profiles = Profiles (thick outlines)
dialog-drawing-hidden = Hidden lines (dashed)
dialog-drawing-dims = Dimensions and text
dialog-drawing-sections = Section cut
dialog-drawing-pdf = PDF…
dialog-drawing-svg = SVG…

## Place 3D Text
dialog-text3d-title = Place 3D Text
dialog-text3d-font = Font
dialog-text3d-filled = Filled
dialog-text3d-extruded = Extruded
dialog-text3d-hint = Enter starts a new line, %CMD%+Enter places the text.
dialog-text3d-place = Place

## Template chooser
dialog-template-title = Choose a template
dialog-template-intro = New models start with these units. You can change the default in Preferences.

## About
dialog-about-tagline = A light, native 3D modeler for drawing and building.
# $version is the app version
dialog-about-version = Version { $version }
app-start-failed-title = Varstotas could not start
app-start-failed = Varstotas could not open its window:

    { $error }

    The 3D view needs Vulkan, DirectX 12, Metal or OpenGL 3.3. Update the graphics driver; in a virtual machine, turn on 3D acceleration. To try another graphics API, set the environment variable WGPU_BACKEND to gl, vulkan or dx12.
dialog-about-build = Build { $build } · { $commit }
dialog-about-dev-build = Development build · { $commit }
dialog-about-licence = Free software under the MIT licence.
dialog-about-licences = Licences…

## ======== Sheets ========

sheets-title = Sheets
sheets-custom = Custom
sheets-none-yet = No sheets yet.
sheets-none-first = No sheet: add one first (Window → Sheets)
sheets-nothing-to-export = Sheets: nothing to export (add a sheet first)
# $i is the sheet's number, $n the number of sheets
sheets-exported-one = Exported sheet { $i } of { $n }: { $path }
sheets-exported-all = Exported { $n } sheet(s): { $path }
sheets-export-failed = Export failed: { $error }
sheets-add-first = Add a sheet to start

## Undo steps
sheets-undo-add = Add Sheet
sheets-undo-edit = Edit Sheet
sheets-undo-duplicate = Duplicate Sheet
sheets-undo-delete = Delete Sheet

## Sheet list and properties
sheets-add = + Sheet
sheets-duplicate = Duplicate
sheets-name = Name
sheets-paper = Paper
sheets-landscape = Landscape
sheets-title-block = Title block
sheets-tb-title = Title
sheets-tb-author = Author
sheets-tb-date = Date

## Adding items
sheets-add-viewport = + Viewport
sheets-add-viewport-hint = The current view of the model
sheets-add-text = + Text
sheets-add-credits = + Credits
sheets-add-credits-hint = The credits of the downloaded models (title, author, site, licence): a text with {"{credits}"}, filled when drawn
sheets-remove = Remove
sheets-add-views = + Standard Views
sheets-add-views-hint = Front, top and left at a standard scale with overall dimensions, and an iso view
sheets-add-table = + Parts Table
sheets-add-table-hint = Groups and components with sizes

## Selected item
# $n is the item's number
sheets-viewport-n = Viewport { $n }
sheets-text-n = Text { $n }
sheets-table-n = Parts table { $n }
sheets-shows = Shows
sheets-captured = Captured view
sheets-update-view = Update to Current View
sheets-scale = Scale
sheets-scale-hint = Scale needs Parallel Projection
sheets-position = Position
sheets-size = Size
sheets-height = Height
sheets-text-height = Text height
sheets-profiles = Profiles
sheets-hidden = Hidden lines
sheets-dims = Dimensions
sheets-sections = Sections
sheets-frame = Frame
sheets-caption = Caption
sheets-overall = Overall dimensions
sheets-hint = Click an item in the preview to edit it; drag to move, drag the corner to resize.

## Footer
# $paper is a paper name; the others are counts
sheets-summary = { $paper } — { $viewports } viewport(s), { $scaled } to scale, { $texts } text(s)
sheets-svg-hint = This sheet
sheets-pdf-hint = All sheets, a page each
sheets-export = Export:

## ======== Component parameters ========

rig-title = Component Parameters
rig-select-hint = Select a component (not a group) and choose Window → Component Parameters, or Edit Parameters… in Properties.
rig-state-has = — with parameters
rig-state-none = — no parameters yet: Apply adds them
rig-no-parts = The component has no parts to move: group its pieces inside it (a door, a drawer…) — open it and use Make Group.

## Properties block
# $name is the component's name; $n the number of copies
rig-info-copies = { $name } — { $n } copies
rig-no-params = No parameters yet
rig-reset = Reset
rig-reset-hint = Back to the default values
rig-edit = Edit Parameters…
rig-edit-hint = Window → Component Parameters: parameters, formulas, parts' motions, script
# $n is the number of motions
rig-info-summary = Component with parameters ({ $n } motion(s))

## Undo steps and messages
rig-undo-reset = Reset Parameters
rig-undo-edit = Edit Parameters
rig-undo-add = Add Parameters
rig-undo-remove = Remove Parameters
rig-select-rigged = Select components with parameters
rig-select-component = Select a component to give it parameters
rig-err-group = Groups can't have parameters: make it a component first (Edit → Make Component)
rig-err-generated = This component is built by a generator or a parametric group
rig-applied = Parameters applied
rig-removed = Parameters removed
rig-err-place-copy = Place a copy of the component to pick in the model
rig-pick-axis = Click two points on the axis of rotation
rig-pick-slide = Click two points: the direction of the slide
rig-err-same-points = The two points are the same

## Parameters table
rig-parameters = Parameters
rig-parameters-hint = The values each copy has (Properties shows them)
rig-col-name = Name
rig-col-label = Label
rig-col-type = Type
rig-col-min = Min
rig-col-max = Max
rig-col-step = Step
rig-col-default = Default
rig-col-options = Options
rig-as-drawn = (as drawn)
rig-remove-param = Remove the parameter
rig-add-param = + Parameter

## Formulas
rig-formulas = Formulas
rig-formulas-hint = Values computed from the parameters (Rhai expressions), usable by the motions and the script
rig-add-formula = + Formula

## Motions
rig-motions = Motions
rig-motions-hint = A part (a group / component directly inside) turned, slid, shown, painted or given a parameter value (Set parameter: a part that has parameters itself — a generated component such as a Draw → Shapes cylinder, or a component with parameters) by a parameter. Turn / slide: the parameter's range maps onto From..To (degrees / length); without a range, or with an expression, the value is the motion itself
rig-motion-rotate = Rotate
rig-motion-slide = Slide
rig-motion-show = Show / hide
rig-motion-paint = Paint
rig-motion-set = Set parameter
rig-parameter = (parameter)
rig-part-no-params = The part has no parameters
rig-part-param-hint = The part's own parameter set to the value
rig-by = by
rig-expression = (expression)
rig-from = from
rig-to = to
rig-remove-motion = Remove the motion
rig-axis-through = Axis through
rig-along = along
rig-direction = Direction
rig-pick = Pick…
rig-pick-hint = Click two points in the model
rig-axes-note = mm, the component's axes
rig-add-motion = + Motion

## Script
rig-script = Script
rig-script-hint = Optional Rhai code run after the motions inside the component: fn update(p) {"{"} … {"}"} with the whole script API (rotate, move, paint, show / hide, set_params, box…); child(name) / child(name, true) finds a part by name (deeply), part.min / max / size / bounds — its extents; p — the values and formulas
rig-add-script = + Script

## Buttons
rig-add = Add Parameters
rig-revert = Revert
rig-remove = Remove Parameters
rig-remove-hint = The copies show the component as drawn again

## ======== Generators ========

## Generator editor
gen-editor-title = Generator Editor
gen-new = New
gen-new-hint = Start from a template
gen-open = Open…
gen-filter = Generator
gen-save = Save to Library
gen-save-hint = Write the generator to { DATA_DIR() }/generators (it shows in the Components tray)
gen-apply-hint = Update this generator of the document: all its components are rebuilt (one undo step)
gen-apply-doc = Apply to Document
gen-apply-doc-hint = Use this source in the document: its components are rebuilt (one undo step)
gen-place = Place
gen-place-hint = Place a component with the preview values
# $name is the generator's name in the document
gen-in-model = In Model: { $name }
gen-not-saved = not saved
gen-build-failed = Build failed
gen-drag-turn = Drag to turn
gen-defaults = Defaults
gen-no-home = No home directory
# $path is a file path; $error the system's message
gen-saved = Saved to { $path }
gen-cannot-save = Cannot save { $path }: { $error }
gen-name-taken = The document already has a generator named { $name }
gen-undo-update = Update Generator { $name }
# $old and $name are generator names
gen-renamed-applied = { $old } → { $name }: applied to the document
gen-same-source = { $name }: the document already uses this source
gen-applied = { $name }: applied to the document

## Generator list and tiles
# $label is a parameter label
gen-not-length = { $label }: not a length
gen-formulas = Formulas
gen-origin-bundled = Bundled example (Edit → Save to Library makes your copy)
gen-origin-document = In this document
gen-tile-error = Error: { $error }
# $note is the description and origin lines
gen-tile-tip =
    Generator — parameters in Properties
    { $note }
gen-edit = Edit Generator…
gen-show-in-finder = Show in File Manager
    .macos = Show in Finder
    .windows = Show in Explorer
gen-delete = Delete…
gen-error = Generator: { $error }

## Properties block
# $name is the generator's name; $n the number of components
gen-info-components = { $name } — { $n } components
gen-info-generator = Generator: { $name }
gen-reset-hint = Back to the generator's defaults
gen-unique-generated = A copy of its generator for this component ("Shelf 2" under In Model): editing that script changes only it
gen-unique-hint = Give this component its own definition: editing it no longer changes the other copies
gen-unique-already = This copy is already unique: nothing else shares its definition or generator
gen-unique-select = Select one component
gen-make-unique = Make Unique
gen-mixed = The selected components differ; a change sets them all
gen-select-generated = Select generated components

## ======== Document Info ========

model-info-title = Document Info
model-info-scene = In the scene
model-info-scene-hint = What is drawn: the content of every placed copy counted
model-info-file = In the file
model-info-file-hint = What the document stores: each definition once, used or not
model-info-libraries = Libraries
# $n is a number, already formatted
model-info-files = { $n } file(s)
# $used, $all and $unused are formatted numbers
model-info-used-of = { $used } of { $all } ({ $unused } unused)

## Rows
model-info-row-polygons = Polygons (faces)
model-info-row-triangles = Triangles
model-info-row-edges = Edges
model-info-row-groups-placed = Groups placed
model-info-row-components-placed = Components placed
model-info-row-components = Components
model-info-row-groups = Groups
model-info-row-materials = Materials
model-info-row-textures = Textures
model-info-row-tags = Tags
model-info-row-scenes = Scenes
model-info-row-sheets = Layout sheets
model-info-row-generators = Generators
model-info-row-lights = Lights
model-info-row-guides = Guides / dimensions
model-info-row-file-size = File size (saved)
model-info-row-lib-components = Components library
model-info-row-lib-materials = Materials library
model-info-preview-image = Preview image
model-info-none = none
model-info-show = Show…
model-info-show-hint = File → Preview Image → Show…

## Preview Image window
model-info-preview-title = Preview Image
model-info-no-preview = No preview image — the Components library draws the model
model-info-preview-note = Saved with the file; the Components library shows it for this file.
model-info-from-file = From Image File…
model-info-from-view = From Current View

## Purge Unused
model-info-purge = Purge Unused
model-info-purge-note = Removes what nothing in the scene uses (Edit → Undo brings it back).
# $n is the number of unused definitions / materials
model-info-purge-defs = Components and groups ({ $n })
model-info-purge-mats = Materials ({ $n })
model-info-nothing-to-purge = Nothing unused to purge
model-info-purge-failed = Purge failed
# $n is a count
model-info-purged-components = { $n } component(s)
model-info-purged-groups = { $n } group(s)
model-info-purged-materials = { $n } material(s)
# $parts is a comma-separated list of the three texts above
model-info-purged = Purged { $parts }

## ======== Plugins ========

## Plugin header errors (shown to plugin authors)
# $label is a parameter label; $value is the text entered
ext-param-invalid = { $label }: not a valid value: { $value }
# $text is the @param line
ext-param-needs-default = @param needs `name = default`: { $text }
ext-param-bad-name = bad parameter name `{ $key }`
ext-param-unclosed-quote = unclosed quote: { $text }
ext-param-unknown-type = unknown parameter type `{ $ty }` (length, int, number, text, bool)
ext-pick-needs-number = @pick needs a number of points: { $text }
ext-pick-range = @pick: 1 to 16 points
ext-pick-default = Click { $n } point(s)
ext-select-default = Select objects
ext-empty-name = empty @name

## Material plugins
ext-mat-plugin-failed = the plugin failed
ext-mat-no-answer = the plugin printed no answer
# $error is the JSON parser's message; $answer is the start of the plugin's answer
ext-mat-not-json = its answer is not JSON ({ $error }): { $answer }
ext-mat-catalogue-array = catalogue() must return an array of materials
ext-mat-no-home = no home folder
ext-mat-default-name = Material
ext-mat-import-nothing = import_material() returned nothing
ext-tex-bad-size = Inline image: "size" must be one side or [width, height] of 1..8192 pixels
ext-tex-no-pixels = Inline image: give its pixels as "rgb", "rgba" or "grey" (an array of 0..255, base64 or a blob)
ext-tex-pixel-count = Inline image: { $want } bytes expected, { $got } given
ext-mat-spec-object = a material spec must be an object

## Plugins menu and plugin dialogs
ext-script-undo = Script
# $error is the plugin header parser's message
ext-header-error = header: { $error }
ext-no-plugins = No plugins
# $n is the number of plugins
ext-failed-to-load = { $n } plugin(s) failed to load…
ext-reload = Reload Plugins
ext-open-folder = Open Plugins Folder
ext-script-gone = the script is gone
# $name is the plugin's or script's name (log lines)
ext-run-script = script { $name }
ext-run-plugin = plugin { $name }
ext-plugin-failed = plugin failed
ext-done = { $name } done
ext-exited = { $name } exited with { $status }
ext-python-no-bridge = Python scripts need the bridge (disabled by VARSTOTAS_NO_BRIDGE)
ext-python-busy = A Python script is already running
ext-python-cant-start = Cannot start { $python }: { $error } (set VARSTOTAS_PYTHON)
ext-pick-points = Pick Points
ext-select-objects = Select Objects
ext-run = Run

## ======== Script Console and Script Reference ========

## Script Console
console-title = Script Console
console-script-failed = script failed
# $error is the script's error text
console-script-error = Script error: { $error }
# $name is the script file name; this is the undo step name
console-script-undo = Script { $name }
# $path is a file path; $error is the system's message
console-cannot-read = Cannot read { $path }: { $error }
console-hint = Rhai scripts over the model. Try: box(ORIGIN, 1000, 600, 18)  — %CMD%+Enter runs
console-run = Run
console-run-hint = %CMD%+Enter
console-open = Open…
console-save = Save…
console-stop-python = Stop Python
console-clear = Clear Output
# $port is a TCP port number
console-bridge = Python / MCP bridge: 127.0.0.1:{ $port }
console-bridge-hint = import varstotas; m = varstotas.connect()  — see { DATA_DIR() }/python
console-filter-rhai = Rhai script

## Script Reference window
console-ref-title = Script Reference
console-search = Search
console-search-hint = function, property or word
console-fulltext = Full text: docs/SCRIPTING.md, docs/GENERATORS.md
console-copy-example = Copy the example
console-click-copy = Click to copy
console-topic-basics = Basics
console-topic-console = Console and plugins
console-topic-generators = Generators
console-topic-component = Component script
console-topic-functions = Functions
console-topic-properties = Properties
console-param-types = Parameter types
console-generators-note = Inside `build` the whole API works (see Functions). The script runs in an empty model holding the document's materials and tags; everything it builds becomes the component's content.
console-functions-intro = Every function of the API, by section. Click a signature to copy it. `x` is an object, an array of objects or a name of a child group.
console-properties-intro = After a dot: `g.name`, `f.normal.x`, `door.size.z`, `[1, 2].len()`. Properties marked assignable can be set: `g.name = "Crate"`, `g.hidden = true`.
console-kind-objects = Objects (faces, edges, groups, components)
console-kind-vectors = Vectors
console-kind-bounds = Bounds — bounds(x), x.bounds
console-kind-arrays = Arrays
console-kind-strings = Strings
console-nothing-found = Nothing found
console-members = Properties and methods
console-tags = Header tags
console-reference = ? Reference
console-reference-hint = Help → Script Reference: functions, properties, examples (F1 in the editor: the word at the cursor)

## Page: basics
console-ref-basics-1 = Scripts are written in Rhai — a small language with Rust / JavaScript-like syntax. Coordinates are world millimetres; lengths with units are strings: `length("1.2m")`.
console-ref-basics-2 = Language
console-ref-basics-4 = Strings: `"text"`, joined with `+`: `"w = " + w`. String methods `trim()`, `to_upper()` change the string in place and return nothing: `s.trim(); if s == "" {"{"} … {"}"}`.
console-ref-basics-5 = Constants: `ORIGIN`, `X`, `Y`, `Z` (unit axes). Vectors add, subtract and scale: `ORIGIN + X * 100`; `.x .y .z`, `length()`, `normalize()`, `dot(b)`, `cross(b)`, `distance(b)`.
console-ref-basics-6 = Objects
console-ref-basics-7 = Faces, edges, groups and components are references: `let g = box(ORIGIN, 100, 100, 100); g.name = "Crate";`. After operations that merge or cut geometry an old reference may become invalid — check `e.valid`.
console-ref-basics-8 = Anywhere an object is expected, a string names a group / component of the current context: `move("Door", vec3(0, 10, 0))`. `child("Knob", true)` searches nested groups too.
console-ref-basics-9 = Every object has `kind`, `bounds`, `size`, `min`, `max`, `center`, `hidden`; groups also `name`, `definition`, `material`, `tag` (see Properties).
console-ref-basics-10 = Context and undo
console-ref-basics-11 = New geometry goes into the current context: the group open when the script started, or the one entered with `enter(g)` (leave with `exit()`).
console-ref-basics-12 = A run is one undo step. A script that fails changes nothing; the error with its line is shown — and kept in Window → Log.
console-ref-basics-13 = `print(x)` writes to the output; the value of the last expression is printed too. Scripts stop after 30 s (generators 10 s).
## Page: console
console-ref-console-1 = Window → Script Console runs code against the open model. %CMD%+Enter runs, %CMD%+↑ / ↓ goes through earlier runs, Open… / Save… work with `.rhai` files. File → Run Script… runs a file.
console-ref-console-2 = Completion shows API functions while typing, properties after a dot (by the type on the left), and the signature of the function being called above the cursor. F1 opens this reference on the word at the cursor.
console-ref-console-3 = Header: dialogs and menu items
console-ref-console-4 = Lines `// @tag …` at the top turn a script into a plugin: Run first asks the parameters, points and objects. A file in `{ DATA_DIR() }/plugins` becomes a Plugins menu item.
console-ref-console-6 = Tags
## Page: generators
console-ref-generators-1 = A generator is a component with parameters: a script whose `build(p)` builds the component's content in its own axes (the origin is the insertion point). Each copy has its own values in Properties; copies with equal values share one definition.
console-ref-generators-2 = Window → Generator Editor (or Edit Generator… on a tile): the source on the left, a live preview with parameter fields on the right; Apply updates the document, Save to Library writes the file.
console-ref-generators-4 = Header
console-ref-generators-5 = required; the generator's key in the document
console-ref-generators-6 = a value in `p`; every part of the range is optional
console-ref-generators-7 = a value computed from the parameters (read-only in Properties)
console-ref-generators-8 = values for which it is false are refused
console-ref-generators-9 = Parameter types
## Page: component
console-ref-component-1 = Window → Component Parameters gives any component parameters: rows of values, formulas, motions of its parts (turn, slide, show, paint, set a part's parameter) and an optional script.
console-ref-component-2 = The script defines `update(p)`. It runs after the motions, inside the component, with the whole script API — the same functions as the console and generators. `p` holds the values and the formulas.
console-ref-component-4 = Finding parts
console-ref-component-5 = a group / component directly inside the component
console-ref-component-6 = searched in nested groups too (the nearest one wins)
console-ref-component-7 = inside a part
console-ref-component-8 = all parts (nested ones too)
console-ref-component-9 = the part's extents now (world)
console-ref-component-10 = hide, like hide(part)
console-ref-component-11 = How it works
console-ref-component-12 = The script changes a copy (the variant for these values), never the master: parts it changes are copied into the variant, parts it leaves alone stay shared. Copies with equal values share one variant.
console-ref-component-13 = Edits made inside a copy (double click) go back to the master without what the script did: what it added is dropped, moves / hiding / paint are undone.
console-ref-component-14 = Errors are shown when Apply is pressed, and kept in Window → Log.

## Code editor
# $sel is the selected completion's number, $n the number of completions
console-completion-count = { $sel } of { $n }

## ======== Log ========

log-level-error = error
log-level-warning = warning
log-title = Log
# $n is a count
log-errors = Errors { $n }
log-warnings = Warnings { $n }
log-filter = filter
log-clear = Clear
log-copy = Copy
log-copy-hint = Copy the entries shown
log-empty = No errors or warnings in this session.
log-no-match = Nothing matches the filters.
# $n is the number of errors not seen yet
log-badge-errors = ● { $n ->
        [one] { $n } error
       *[other] { $n } errors
    }
log-badge = Log
log-badge-hint = Window → Log: errors and warnings from everywhere (scripts, import / export, libraries …)

## ======== Licences ========

licences-tab-varstotas = Varstotas
licences-tab-components = Open Source
licences-tab-formats = File Formats
licences-tab-libraries = Online Libraries

licence-cc0 = CC0
licence-cc-by = CC-BY
licence-cc-by-sa = CC-BY-SA
licence-cc-by-nc = CC-BY-NC
licence-cc-by-nd = CC-BY-ND
licence-standard = Standard / personal use
licence-unknown = Unknown

licence-cc-by-nc-note = non-commercial: don't sell what you make with it
licence-cc-by-nd-note = no derivatives: don't share a changed copy
licence-cc-by-sa-note = share-alike: share changed copies under the same licence
licence-standard-note = the site's own licence (royalty-free / personal use / usage conditions): often no commercial use or no sharing of the model file itself — check it
licence-unknown-note = licence unknown: for your own use only

# Licences window: labels and the explanations of formats and online libraries (the licence texts themselves are not translated).

## Window
licences-title = Licences
licences-count = { $n } components
licences-filter = Filter: name or licence
licences-save = Save as Text…
licences-save-hint = All components and their licence texts in one file
licences-saved = Saved { $path }
licences-pick = Pick a component to see its licence.
licences-all-text-title = Varstotas uses the following open-source components.
licences-applies-to = Applies to: { $users }

## Varstotas tab
licences-own = Varstotas is free software under the MIT licence: you may use it for personal and commercial work, copy, change and share it.
licences-programs = Programs used when you have them
licences-yours = Your models are yours. Models and materials downloaded from online libraries keep their own licences (Online Libraries; File → Export → Attributions).

## Programs


licence-program-oidn-use = AI denoiser of the renderer
licence-program-oidn-licence = Apache-2.0 — not shipped, loaded from your installation

licence-program-python-use = Python plugins and scripts
licence-program-python-licence = PSF licence — not shipped, your installation

## File formats tab
licence-formats-col-format = Format
licence-formats-col-varstotas = Varstotas
licence-formats-col-how = How
licence-formats-col-note = The format

licence-format-varstotas-what = open, save
licence-format-varstotas-how = Varstotas's own format
licence-format-varstotas-note = MIT, like Varstotas

licence-format-gltf-what = import, export
licence-format-gltf-how = own reader / writer; Draco meshes via draco-core (Apache-2.0)
licence-format-gltf-note = Khronos open standard, royalty-free

licence-format-obj-what = import, export
licence-format-obj-how = own reader / writer
licence-format-obj-note = open, public specification

licence-format-stl-what = import, export
licence-format-stl-how = own reader / writer
licence-format-stl-note = open, public specification

licence-format-ply-what = import, export
licence-format-ply-how = own reader / writer
licence-format-ply-note = open, public specification

licence-format-3mf-what = import, export
licence-format-3mf-how = own reader / writer (zip + XML)
licence-format-3mf-note = 3MF Consortium specification, royalty-free

licence-format-collada-what = import, export
licence-format-collada-how = own reader / writer (images as files next to it)
licence-format-collada-note = Khronos open standard, royalty-free

licence-format-fbx-what = import
licence-format-fbx-how = ufbx (MIT / Unlicense), an independent reader
licence-format-fbx-note = Autodesk's format; FBX is an Autodesk trademark, Varstotas is not affiliated

licence-format-dxf-what = import, export
licence-format-dxf-how = own reader / writer
licence-format-dxf-note = Autodesk's openly published format; DXF is an Autodesk trademark

licence-format-svg-pdf-what = export (drawings, unfolds, sheets)
licence-format-svg-pdf-how = own writers
licence-format-svg-pdf-note = W3C / ISO open standards

licence-format-images-what = textures, images, environments
licence-format-images-how = image crate (MIT / Apache-2.0)
licence-format-images-note = open formats

licence-format-openexr-what = render export
licence-format-openexr-how = own writer
licence-format-openexr-note = Academy Software Foundation, BSD-3-Clause

licence-format-csv-json-what = points import, faces export (plugins)
licence-format-csv-json-how = own code
licence-format-csv-json-note = open formats

licence-format-gif-what = animation export
licence-format-gif-how = image crate (MIT / Apache-2.0)
licence-format-gif-note = open format

## Online libraries tab
licence-libs-intro = Every downloaded model keeps its source, author and licence; Varstotas warns when a placed or exported model's licence limits its use (non-commercial, no derivatives, share-alike, a site's own licence, unknown).
licence-libs-col-library = Library
licence-libs-col-assets = Assets
licence-libs-col-licence = Licence
licence-libs-col-terms = Terms

licence-lib-poly-haven-what = models, materials
licence-lib-poly-haven-licence = CC0
licence-lib-poly-haven-terms = Free public API, also for commercial apps; asks to show «Powered by Poly Haven» (done above its assets) and a unique User-Agent (Varstotas).

licence-lib-ambientcg-what = materials
licence-lib-ambientcg-licence = CC0
licence-lib-ambientcg-terms = Free API and downloads; credit appreciated, not required.

licence-lib-smithsonian-what = models
licence-lib-smithsonian-licence = mostly CC0; some «Usage conditions apply» (non-commercial)
licence-lib-smithsonian-terms = The API does not say which: a model's page has to be checked. Varstotas marks these models as «check the usage» and warns when they are placed or exported.

licence-lib-nasa-what = models
licence-lib-nasa-licence = not copyrighted (US public domain)
licence-lib-nasa-terms = NASA insignia and logos are protected; no use may suggest NASA endorsement.

licence-lib-kenney-what = model packs
licence-lib-kenney-licence = CC0
licence-lib-kenney-terms = Read from the public asset pages of kenney.nl; Kenney's logo is not CC0.

licence-lib-quaternius-what = model packs
licence-lib-quaternius-licence = CC0
licence-lib-quaternius-terms = Downloaded by you in the browser.

licence-lib-search-name = Search on sites (Printables, Thingiverse, Sketchfab, …)

licence-lib-search-what = models
licence-lib-search-licence = each model its own
licence-lib-search-terms = Varstotas only opens the site's search in your browser; you download yourself and set the licence in the import card.

licence-lib-plugins-name = Plugins

licence-lib-plugins-what = models, materials
licence-lib-plugins-licence = per plugin
licence-lib-plugins-terms = A source plugin shows its terms (its @notice line) above its assets; e.g. Sketchfab needs your own account token and a credit to the author and Sketchfab.

## ======== User Guide ========

help-topic-start = Getting started
help-topic-navigation = Navigating the view
help-topic-selecting = Selecting
help-topic-drawing = Drawing
help-topic-shapes = 3D shapes
help-topic-modifying = Modifying
help-topic-inference = Inference and snapping
help-topic-measurements = Measurements box
help-topic-groups = Groups and components
help-topic-library = Components library
help-topic-materials = Materials and textures
help-topic-organizing = Tags, Outliner, Properties
help-topic-construction = Guides, dimensions, sections
help-topic-styles = Styles and rendering
help-topic-scenes = Scenes, sheets, drawings
help-topic-files = Files, import and export
help-topic-scripts = Scripts and plugins
help-topic-shortcuts = Keyboard and mouse
help-topic-problems = When something goes wrong
## start
help-start-1 = Varstotas is a 3D modeller: you draw edges, closed edges make faces, faces are pushed and pulled into solids. Everything sticks together until you group it.
help-start-2 = The window
help-start-3 = Menu bar
help-start-4 = File, Edit, View, Camera, Draw, Tools, Plugins, Window, Help
help-start-5 = Toolbar
help-start-6 = the tools (a tooltip names each one and its key), navigation and Snapping; views and styles are in the viewport's top-right corner
help-start-7 = Viewport
help-start-8 = the model; the red, green and blue lines are the axes, the ground is under them
help-start-9 = Side Bar
help-start-10 = cards on the right: Properties, Materials, Components, Tags, Outliner, Scenes, Shadows, Environment, Soften Edges, Tool Guide (Window → Side Bar turns cards on and off)
help-start-11 = Status bar
help-start-12 = what the active tool expects, messages and errors, the Log button, the Measurements box
help-start-13 = A first model
help-start-14 = 1. Press `R` (Rectangle), click a corner on the ground, click the opposite corner — or type `1000,600` and Enter.
help-start-15 = 2. Press `P` (Push/Pull), click the rectangle, move up and click — or type `750` and Enter.
help-start-16 = 3. Press `Space` (Select), triple-click the box to select it all, press `G` to make a group.
help-start-17 = 4. Press `B` (Paint Bucket), pick a material in the Materials card, click faces.
help-start-18 = 5. `Shift+Z` fits the model in the view; File → Save writes a `.vrst` file.
help-start-19 = Help while working
help-start-20 = The Tool Guide card explains the active tool; F1 over the model opens this guide on it. Finished models to look at (a furnished house, a material gallery) are in the folder { DATA_DIR() }/examples: File → Open.
## navigation
help-navigation-1 = Mouse and trackpad
help-navigation-2 = Middle button drag
help-navigation-3 = orbit around the point under the cursor
help-navigation-4 = Shift + middle drag
help-navigation-5 = pan
help-navigation-6 = Wheel
help-navigation-7 = zoom toward the cursor
help-navigation-8 = Trackpad: two-finger swipe
help-navigation-9 = orbit; with Shift — pan
help-navigation-10 = Trackpad: pinch, or %CMD% + swipe
help-navigation-11 = zoom
help-navigation-12 = These work in every tool, so you never need to leave a drawing operation to look around.
help-navigation-13 = Window → Preferences → Navigation changes them: orbit and zoom speed, an inverted wheel, zoom toward the middle of the view instead of the cursor, the middle button panning (Shift orbits), two fingers panning or zooming.
help-navigation-14 = Views
help-navigation-15 = Camera → Standard Views (and the view drop-down in the viewport's corner): Iso, Top, Bottom, Front, Back, Left, Right. Camera → Parallel Projection / Perspective. `Shift+Z` — Zoom Extents: the whole model in view.
help-navigation-16 = Saved views are scenes — see Scenes, sheets, drawings.
help-navigation-17 = The viewport's own controls
help-navigation-18 = Three icon drop-downs in the viewport's top-right corner hold everything about how it shows the model (hover one to see what it is set to):
help-navigation-19 = View
help-navigation-20 = standard views, Perspective / Parallel Projection, Field of View, Zoom Extents, Viewports 1 / 2 / 4
help-navigation-21 = Style
help-navigation-22 = Shaded … Realistic; X-Ray, Edges, Profiles
help-navigation-23 = Show (the eye)
help-navigation-24 = axes, ground, sky (off = a plain light background), grid of this view; guides, shadows, section planes / cuts / fill of the model (every view)
help-navigation-25 = Switches keep the menu open; a click outside closes it.
help-navigation-26 = Several viewports
help-navigation-27 = View → Viewports (or the view drop-down) shows 1, 2 (side by side) or 4 (2 × 2) views of the model. Each has its own camera, style and view switches; new ones start from the top, the front and the right, in parallel projection. A click in a view makes it the active one (blue frame): menus, toolbars and tools act on it, and the click itself already works there.
help-navigation-28 = Presets (View → Viewports, or Viewport Presets in the view drop-down) set up common layouts at once, e.g. Front + Iso or Front, Left, Top + Iso: the flat views in parallel projection, the iso view in perspective.
## selecting
help-selecting-1 = Edit → Select All (`%CMDK%A`), Select None (`%CMDK%T`). Delete or Backspace erases the selection; Edit → Hide hides it, Unhide All brings everything back.
help-selecting-2 = Right-click the model for the context menu: Erase, Hide, Make Group, Make Component…, Explode, Intersect Faces, Reverse / Orient Faces, Flip Along, Texture → Position, Unfold…, and the commands of section planes and axes.
help-selecting-3 = The Outliner card shows the groups and components as a tree: a click there selects, a double click zooms to the object.
## drawing
help-drawing-1 = Drawing tools work on the face under the cursor, or on the ground / the active work plane. Closed loops of edges become faces; a line across a face splits it.
help-drawing-2 = While a tool runs you can type a value at any time (no need to click the Measurements box) and press Enter — see Measurements box.
## shapes
help-shapes-1 = Draw → Shapes builds a whole solid in one operation. The result is a generated component: select it and change its sizes and segments in Properties; Explode turns it into plain geometry.
## modifying
help-modifying-1 = Move, Rotate and Scale work on the selection, or on what you click when nothing is selected. Moving a vertex, edge or face stretches the geometry around it.
help-modifying-2 = Copy
help-modifying-3 = Move / Rotate with %COPY% makes a copy
help-modifying-4 = Arrays
help-modifying-5 = after a copy type `5x` (five copies at the same step) or `/5` (five copies between)
help-modifying-6 = Edit menu and context menu
help-modifying-7 = Intersect Faces
help-modifying-8 = with the model, the selection or the context: edges where faces cross
help-modifying-9 = Slice
help-modifying-10 = a plane cuts the selection or the context; with %COPY% (Split) each cut body becomes two closed groups
help-modifying-11 = Reverse / Orient Faces
help-modifying-12 = turn the front side (white) outward
help-modifying-13 = Flip Along
help-modifying-14 = mirror the selection along the red, green or blue direction
help-modifying-15 = Soften Edges card
help-modifying-16 = soften / smooth edges by angle
help-modifying-17 = Window → Unfold
help-modifying-18 = flatten faces or groups into a cutting pattern (SVG / PDF / DXF)
## inference
help-inference-1 = While you draw, the cursor snaps to points and directions of the model; a marker and a tooltip say what it found.
help-inference-2 = Endpoint, Midpoint
help-inference-3 = green / cyan dot
help-inference-4 = On Edge, On Face
help-inference-5 = red square / blue diamond
help-inference-6 = Center, Intersection
help-inference-7 = of circles and arcs, of edges
help-inference-8 = On Red / Green / Blue Axis
help-inference-9 = a line in the axis' colour
help-inference-10 = Parallel / Perpendicular
help-inference-11 = magenta line along an edge
help-inference-12 = From Point
help-inference-13 = a dotted line from a point along an axis
help-inference-14 = Locking
help-inference-15 = Hold `Shift` to lock the direction found; the arrow keys lock an axis: → red, ← green, ↑ blue, ↓ parallel / perpendicular to the last edge.
help-inference-16 = Snapping menu
help-inference-17 = The magnet button (and the Snapping menu) turns each inference on or off; `Shift+S` turns snapping off and on altogether. Increments round free moves (10 mm), angles (15°) and scale factors — snapping to geometry and typed values win over them.
help-inference-18 = View → Grid shows a grid with its own snapping; the step is in Window → Preferences → Drawing, and with Select active typing a length sets it.
help-inference-19 = Work planes
help-inference-20 = The Guide Plane tool (Tools menu) and Window → Work Planes make a plane to draw on in the air: points in empty space land on it, the axes and the grid follow it.
## measurements
help-measurements-1 = The Measurements box at the right of the status bar shows the current length, angle or size. Just type while a tool runs and press Enter; Backspace corrects, Esc clears.
help-measurements-2 = 1200, 1.2m, 120cm, 4', 1'6"
help-measurements-3 = lengths: the model unit, or a unit you name
help-measurements-4 = 1000,600
help-measurements-5 = two values: a rectangle's sides, a box's base
help-measurements-6 = 45
help-measurements-7 = an angle (Rotate, Protractor, arcs)
help-measurements-8 = 24s
help-measurements-9 = segments of a circle, arc or shape; sides of a polygon
help-measurements-10 = 5x, /5
help-measurements-11 = copies in a row after a Move / Rotate copy
help-measurements-12 = 2, 0.5
help-measurements-13 = a scale factor
help-measurements-14 = Typing again right after an operation changes that operation (a different length, more copies).
help-measurements-15 = The model unit comes from the template (File → New from Template); `,` and `.` both work as the decimal sign.
## groups
help-groups-1 = Geometry sticks: edges and faces that touch merge and split each other. To keep a part separate, make it a group or a component.
help-groups-2 = Make Group (G)
help-groups-3 = a group: one object; copies are independent
help-groups-4 = Make Component…
help-groups-5 = a component: copies share one definition — editing one edits all
help-groups-6 = Double-click
help-groups-7 = enter a group / component to edit it (the rest dims); Esc or Edit → Close Group leaves
help-groups-8 = Explode
help-groups-9 = back to loose geometry
help-groups-10 = Make Unique (right-click, Properties)
help-groups-11 = a copy of a component gets its own definition ("Box 2"): editing it no longer changes the other copies; a generated one gets a copy of its generator ("Shelf 2" under In Model)
help-groups-12 = Place from the library again
help-groups-13 = the copy in the model is reused while it is unchanged; after you edited it the library's original comes in as a new definition ("Chair 2")
help-groups-14 = Components with parameters
help-groups-15 = Edit → Make Parametric gives a component sizes X / Y / Z (stretch or scale) and a row of copies in Properties.
help-groups-16 = Window → Component Parameters gives any component parameters: values and formulas that turn, slide, show or paint its parts, an optional script; every copy has its own values (sliders in Properties).
help-groups-17 = Generators are components built by a script with typed parameters (Window → Generator Editor). The Draw → Shapes solids are generators too.
help-groups-18 = Component script in the Script Reference
help-groups-19 = Generators in the Script Reference
## library
help-library-1 = The Components and Materials cards look the same: a top row (Search, Refresh, + to add, the cloud button for the online library, … for the library folder, what to show and the tile size: Small / Large / List), then the sections In Model (the open document), Favourites, Recent, My Library with its folders, and Examples. A section folds with its arrow; a folder's name opens it in the Finder.
help-library-2 = Right-click a tile → Add to Favourites
help-library-3 = the ☆ tile also shows in Favourites at the top (Remove from Favourites takes it out); Recent lists what you placed or picked to paint with lately
help-library-4 = Right-click a folder name
help-library-5 = New Folder…; for a subfolder also Rename Folder… and Delete Folder (only when it is empty)
help-library-6 = Right-click a tile → Move to Folder
help-library-7 = move the file into another folder of the library (favourites follow it)
help-library-8 = Drag a tile into the viewport
help-library-9 = a component is placed where you drop it; a material paints the face under the cursor
help-library-10 = Components lists your library (`{ DATA_DIR() }/components`, or a folder you choose with …), the generators, the bundled Examples and the components of the open document.
help-library-11 = Click a tile
help-library-12 = place it: the copy follows the cursor, click to put it down
help-library-13 = + → Save Selection to Library…
help-library-14 = write the selection as a new `.vrst` file
help-library-15 = Right-click a tile
help-library-16 = Place, Set Preview from View, Set Preview Image…, Remove Preview Image, Rename…, Show in Finder, Delete…; for a document component also Select Instances, Save to Library, Parameters…, Delete from Document…
help-library-17 = Pictures
help-library-18 = A tile shows the file's picture when it has one, else a rendered thumbnail. Set Preview from View takes the viewport as it is now — frame the component first. A component placed from the library takes the file's picture with it; File → Preview Image sets the picture of the open document.
help-library-19 = 3D Online Library
help-library-20 = The cloud button in the Components card (or Window → 3D Online Library) downloads models: Poly Haven, Smithsonian, NASA, Kenney packs, by URL, or from plugin sources. A downloaded model becomes a component in the library with its credit (Properties → Source, File → Export → Attributions). Filters by licence, format and size; favourites and the history of downloads.
help-library-21 = Each source shows its terms above its models (a plugin's @notice line). A model whose licence limits its use (non-commercial, no derivatives, share-alike, a site's own licence, unknown) gets a warning when it is placed and when the document is exported. Help → Licences… lists Varstotas's licence, the open-source components, file formats and the online libraries with their terms.
## materials
help-materials-1 = The Materials card shows the materials of the model (In Model), your library (`{ DATA_DIR() }/materials`, or a folder you choose with …) and Examples; right-click a tile for Use, Rename…, Save to Library, Show in Finder, Export…, Delete…. Picking a library material only arms the Paint Bucket: the material joins the model (In Model) with the first face painted. Delete on an In Model tile removes the material from the model, its faces go back to the default (Edit → Undo brings it back). Edit a material there: colour, opacity, a texture image and its size, physical properties for the Realistic style (metal, roughness, glass, emission, maps, coat, sheen).
help-materials-2 = Save to Library / Export… / + → Import…
help-materials-3 = share a material between documents (`.vrstmat`)
help-materials-4 = Cloud button (Materials card)
help-materials-5 = download PBR scans from Poly Haven and ambientCG, or from plugin sources
help-materials-6 = Shader
help-materials-7 = a material can have its own WGSL shader with parameters (the shader editor has its own reference)
help-materials-8 = Faces have a front (white) and a back (grey-blue) side; painting a group paints the faces inside it that have no material of their own.
## organizing
help-organizing-1 = Properties
help-organizing-2 = properties of the selection: area, length, volume of solids, tag, material, name, parameters of components and generators
help-organizing-3 = Tags
help-organizing-4 = visibility, colour and name of tags (layers); deleting a tag moves its content to Untagged
help-organizing-5 = Outliner
help-organizing-6 = the tree of groups and components: Zoom To, Edit, Hide, Explode, Move Out of Group, Delete; drag rows to move objects between groups
help-organizing-7 = Hide / Unhide All
help-organizing-8 = Window → Document Info
help-organizing-9 = statistics of the document (faces, triangles, components, materials, textures …) and Purge Unused
## construction
help-construction-1 = Guides are selected, moved and deleted like geometry; Edit → Delete Guides removes them all, View → Guides hides them. Tools → 3D Text… makes text as geometry.
help-construction-2 = View → Section Planes / Section Cuts / Section Fill control how sections are shown; the active cut is saved in scenes.
## styles
help-styles-1 = View menu (and the viewport's Style drop-down)
help-styles-2 = Shaded, Shaded with Textures, Monochrome
help-styles-3 = faces in material colours, with textures, in one colour
help-styles-4 = Hidden Line, Wireframe
help-styles-5 = only the edges (hidden ones removed or not)
help-styles-6 = Realistic
help-styles-7 = physical materials, lighting from the environment, sun, effects — the Environment card sets sky, HDRI, exposure and effects
help-styles-8 = X-Ray, Edges, Profiles
help-styles-9 = see through faces; edges and thick outlines on and off
help-styles-10 = Shadows
help-styles-11 = sun shadows; the time and date are in the Shadows card
help-styles-12 = Rendering
help-styles-13 = Window → Render renders the view with a path tracer on the GPU: glass, reflections, lights, soft shadows; quality presets, frame size, depth of field, denoising (when Intel OIDN is installed), colour grading; PNG / EXR, or every scene at once.
help-styles-14 = Lights (point, spot, area) are placed from the Render window; they are selected by clicking their marker and moved and rotated like other objects.
help-styles-15 = File → Export → 2D Graphic (PNG) saves the view as it is; File → Export → Animation… makes an animated GIF or PNG frames of the scenes or a fly-around.
## scenes
help-scenes-1 = Scenes card
help-scenes-2 = saved views with their style, visible tags and section; tabs above the viewport switch them
help-scenes-3 = View → Animation
help-scenes-4 = Play the scenes; PgDn / PgUp — next / previous scene
help-scenes-5 = Window → Sheets
help-scenes-6 = layout pages: viewports of scenes at a scale 1:N, texts, a title block, standard views with dimensions, a parts table; multi-page PDF
help-scenes-7 = File → Export → 2D Drawing
help-scenes-8 = the current view as a vector SVG / PDF with profiles, hidden lines, sections and dimensions
help-scenes-9 = File → Export → Report
help-scenes-10 = parts with sizes for cutting, areas by material (CSV)
## files
help-files-1 = Documents are `.vrst` files (textures and pictures are saved inside). File → New from Template picks the units; Window → Preferences sets the default template and the auto-save interval.
help-files-2 = Window → Preferences has sections on the left and a search: General (auto-save, the template of File → New, how many undo steps are kept), Interface (light, dark or system theme, the scale of menus and panels, toolbar icon size), Drawing (grid, snapping and increments — the same switches as the Snapping menu), Libraries (the components and materials folders, Examples, tile size), Downloads (the watched folder), Navigation (mouse and trackpad), Shortcuts (click a shortcut and press the new keys; Backspace removes it, ↺ puts the default back), Render (the render settings of new documents; where the Open Image Denoise library is) and View (how a new document looks: style, edges, axes, ground, sky, grid, shadows, projection, field of view; Use Current View / Apply to Current View). Changes apply at once; Reset puts a section back to the defaults.
help-files-3 = Auto-save writes unsaved changes to `{ DATA_DIR() }/autosave`; after a crash Varstotas offers to recover them.
help-files-4 = Import
help-files-5 = OBJ, STL, DXF, glTF / GLB, 3MF, COLLADA (DAE), FBX, PLY, points from CSV, and formats added by plugins; you can also drop files on the window
help-files-6 = Export
help-files-7 = OBJ, STL, DXF, glTF (GLB), 3MF, COLLADA (DAE), PLY, faces as JSON, PNG, SVG / PDF drawings, sheets, unfold patterns, animation, report, attributions
help-files-8 = Edit → Cut / Copy / Paste / Paste in Place
help-files-9 = Paste in Place keeps the copied position
help-files-10 = File → Preview Image
help-files-11 = the picture the library shows for this file
## scripts
help-scripts-1 = Window → Script Console runs Rhai scripts on the model: build, change, select, measure. File → Run Script… runs a `.rhai` (or `.py`) file. A run is one undo step.
help-scripts-2 = Plugins are scripts in `{ DATA_DIR() }/plugins`: they become items of the Plugins menu, with dialogs for parameters and picking points; they can add importers, exporters and material / model sources.
help-scripts-3 = Python and AI agents can drive Varstotas through the local bridge (Python client, MCP server).
help-scripts-4 = Open the Script Reference
help-scripts-5 = Console and plugins
help-scripts-6 = All functions
## shortcuts
help-shortcuts-1 = Tool keys work when no text field has the focus. Letters typed during an operation go to the Measurements box.
## problems
help-problems-1 = Window → Log
help-problems-2 = every error and warning of the session — scripts, import / export, libraries, downloads; a red counter in the status bar means new errors
help-problems-3 = Edit → Undo
help-problems-4 = every operation can be undone; a failed operation changes nothing
help-problems-5 = Window → Document Info → Purge Unused
help-problems-6 = remove unused components, materials and tags to make the file smaller
help-problems-7 = Faces look grey-blue
help-problems-8 = you see their back: select them, right-click → Reverse Faces or Orient Faces
help-problems-9 = Cannot draw where expected
help-problems-10 = check Shift / arrow locks, the active work plane (Window → Work Planes) and whether you are inside a group (Esc leaves it)
help-problems-11 = Something is missing
help-problems-12 = Edit → Unhide All, the Tags card (hidden tags), View → Section Planes

## Window, search, shortcuts
help-window-title = User Guide
help-search = Search
help-search-hint = tool, command or word
help-nothing-found = Nothing found
help-in-the-guide = In the guide
help-tools = Tools
help-commands = Commands
help-use = Use
help-use-hint = Pick this tool
help-shortcuts-note = Change them in Window → Preferences → Shortcuts. Mouse and trackpad: see Navigating the view. Selection modifiers: see Selecting.
help-key-cut-copy-paste-keys = %CMDK%X / %CMDK%C / %CMDK%V
help-key-cut-copy-paste = Cut / Copy / Paste
help-key-delete = Delete, Backspace
help-key-delete-what = erase the selection
help-key-esc = Esc
help-key-esc-what = cancel the operation; leave the group being edited
help-key-shift = Shift (hold)
help-key-shift-what = lock the inference direction
help-key-arrows = → ← ↑ ↓
help-key-arrows-what = lock the red / green / blue axis, parallel to the last edge
help-key-enter = Enter
help-key-enter-what = apply the value typed in the Measurements box
help-key-f1 = F1
help-key-f1-what = this guide on the active tool; in a script editor — the Script Reference

## ======== Scripting ========

## Functions
script-fn-vec3 = A point / vector (an array [x, y, z] works too).
script-fn-line = Edges from a to b.
script-fn-polyline = Edges through points; closed and planar → a face.
script-fn-face = A face through the points.
script-fn-rect = A rectangular face in the XY plane.
script-fn-circle = A circle face (or edges).
script-fn-polygon = A regular polygon face.
script-fn-arc = Arc edges around center.
script-fn-box = A new box group (origin = min corner).
script-fn-cylinder = A new cylinder group (bottom at center).
script-fn-cone = A new cone group (base at center, apex h above it).
script-fn-pyramid = A new pyramid group: a w × d base from origin, the apex h above its middle.
script-fn-sphere = A new sphere group around center.
script-fn-torus = A new torus group around center: r to the middle of the tube, tube = its radius.
script-fn-pushpull = Push / pull a face; returns the cap face.
script-fn-offset = Offset a face's loop (d < 0 inward); returns the new face.
script-fn-follow-me = Sweep a profile face along a path.
script-fn-erase = Erase an entity or an array of them.
script-fn-move = Move entities by a vector.
script-fn-copy = Copy entities by a vector; returns the copies.
script-fn-rotate = Rotate entities about an axis.
script-fn-scale = Scale entities about a point.
script-fn-group = Group entities; returns the group.
script-fn-component = Make a component; returns its instance.
script-fn-explode = Explode a group / component.
script-fn-make-unique = Give a component instance its own definition.
script-fn-union = Solid union of two solid groups.
script-fn-subtract = Solid a minus b.
script-fn-intersect = Solid intersection.
script-fn-trim = a minus b, b stays.
script-fn-split = Split two solids into three.
script-fn-outer-shell = Outer shell of solids.
script-fn-reverse = Reverse faces.
script-fn-soften = Soften edges up to the fold angle.
script-fn-material = Create / recolour a material; returns its name.
script-fn-paint = Paint faces / groups (() removes).
script-fn-tag = Assign a tag (created when missing).
script-fn-make-parametric = Make a group parametric.
script-fn-set-params = Set a generated component's values, or a parametric group's size / copies.
script-fn-params = Values of a generated component / parameters of a parametric group, or ().
script-fn-reset-params = A generated component back to its generator's defaults.
script-fn-generators = Names of the document's generators.
script-fn-add-generator = Add / update a generator from its source; returns its name.
script-fn-generate = Place a generated component at a point.
script-fn-dimension = A linear dimension between points.
script-fn-note = A text note with a leader.
script-fn-enter = Build inside a group from now on.
script-fn-exit = Leave the group entered last.
script-fn-faces = Faces of the current context.
script-fn-edges = Edges of the current context.
script-fn-groups = Groups / components of the current context.
script-fn-all = Everything in the current context.
script-fn-entities = Everything inside a group.
script-fn-child = The group / component named name in the current context (or inside g); deep = search nested groups too. () when none.
script-fn-children = Groups / components of the current context (or inside g); deep = nested ones too.
script-fn-show = Show (on = false: hide) entities.
script-fn-hide = Hide entities.
script-fn-selection = The selection.
script-fn-select = Set the selection (() clears).
script-fn-bounds = #{ "{" } min, max, size, center { "}" } in world coordinates.
script-fn-model-bounds = Bounds of the whole model.
script-fn-length = Parse a length with units ("1.2m") → mm.
script-fn-units = The model's length unit.
script-fn-materials = Material names.
script-fn-all-faces = Every face, nested groups included.
script-fn-read-text = Read a text file.
script-fn-lang = The UI language code ("en", "ru", …): pick the texts a plugin prints or shows.
script-fn-platform = "macos", "windows" or "linux".
script-fn-write-text = Write a text file.
script-fn-file-exists = A file or folder is there.
script-fn-list-files = Files under dir (subfolders too) with the extension ("" = any), sorted.
script-fn-list-dirs = The folders in dir, sorted.
script-fn-parse-json = JSON text → maps, arrays, numbers, strings.
script-fn-fetch-text = Download a web page / API answer as text (HTTP(S)); headers: #{ "{" } "X-Auth-Token": key { "}" }.
script-fn-fetch-file = Download url into the file path.
script-fn-url-encode = Percent-encode text for a URL (a search query).
script-fn-print = Write to the console log.
script-fn-len = Length of an array / string.
script-fn-range = Integers from..to (to excluded), for loops.
script-fn-min = The smaller number.
script-fn-max = The larger number.
script-fn-clamp = x kept within lo..hi.
script-fn-ceil = Round up to a whole number (as a float).
script-fn-noise = Smooth value noise in 0..1 at x, y (one cell = 1). With period > 0 it repeats every period cells (seamless tiles); seed picks the pattern.
script-fn-fbm = Fractal noise in 0..1: octaves of noise, each twice as fine and half as strong; period and seed as in noise.

## Sections
script-group-points-and-vectors = Points and vectors
script-group-drawing = Drawing
script-group-solids = Solids
script-group-editing = Editing
script-group-groups-and-components = Groups and components
script-group-finding-objects = Finding objects
script-group-visibility-materials-tags = Visibility, materials, tags
script-group-solid-tools = Solid tools
script-group-parameters-and-generators = Parameters and generators
script-group-dimensions-and-notes = Dimensions and notes
script-group-files-and-network = Files and network
script-group-output-and-numbers = Output and numbers

## Members of entities, vectors, arrays, strings
script-member-kind = "face", "edge", "group", "component" …
script-member-valid = the entity still exists
script-member-area = face area, mm²
script-member-normal = face normal (world)
script-member-edges = edges of the face
script-member-faces = faces of the edge / neighbours of the face
script-member-length = edge length, mm
script-member-soft = the edge is soft
script-member-center = centre of the bounds (world)
script-member-vertices = points of the face / edge (world)
script-member-name = group / component name (assignable)
script-member-hidden = the entity is hidden (assignable)
script-member-bounds = #{ "{" } min, max, size, center { "}" } (world)
script-member-size = size of the bounds along x, y, z
script-member-min = minimum corner of the bounds (world)
script-member-max = maximum corner of the bounds (world)
script-member-definition = component definition name
script-member-material = material name or ()
script-member-tag = tag name or ()
script-member-x = x coordinate (assignable)
script-member-y = y coordinate (assignable)
script-member-z = z coordinate (assignable)
script-member-length-2 = vector length
script-member-normalize = unit vector
script-member-dot = dot product
script-member-cross = cross product
script-member-distance = distance to b
script-member-min-2 = minimum corner
script-member-max-2 = maximum corner
script-member-size-2 = size along x, y, z
script-member-center-2 = centre
script-member-len = number of items
script-member-is-empty = no items
script-member-push = append an item
script-member-pop = remove and return the last item
script-member-insert = insert an item
script-member-remove = remove and return an item
script-member-contains = the array / string contains the value
script-member-index-of = index of the value or -1
script-member-filter = items where the closure is true
script-member-map = a new array of the closure results
script-member-reduce = fold the items
script-member-len-2 = number of characters
script-member-to-upper = upper case (in place)
script-member-to-lower = lower case (in place)
script-member-trim = strip spaces (in place)
script-member-split = split into an array
script-member-starts-with = the string starts with text
script-member-to-string = as text

## Header tags of plugins
script-tag-name = name in the menu and the dialog title
script-tag-menu = path under Plugins
script-tag-description = tooltip and text of the dialog
script-tag-param = a value asked in the dialog before the run
script-tag-pick = N points clicked before the run → points
script-tag-select = objects selected before the run → objects
script-tag-import = a File → Import entry; the file in path
script-tag-export = a File → Export entry; the file in path
script-tag-generator = generator: makes this file a generator (see docs/GENERATORS.md)
script-tag-formula = generator: a value computed from the parameters, in p
script-tag-check = generator: values for which it is false are refused

## Parameter types (@param key:type)
script-param-type-length = length with units ("1.2m"), mm in the script — the default for numbers
script-param-type-int = whole number
script-param-type-number = fractional number
script-param-type-text = text — the default for a quoted value
script-param-type-bool = check box — the default for true / false
script-param-type-choice = generator: one of choice(a|b|c)
script-param-type-angle = generator: degrees, a slider
script-param-type-percent = generator: 0–100, a slider
script-param-type-material = generator: a material name of the document

script-type-face = face
script-type-edge = edge
script-type-group = group
script-type-vector = vector
script-type-num = number
script-type-bool = bool
script-type-str = string
script-type-bounds = bounds
script-type-faces = array of faces
script-type-edges = array of edges
script-type-groups = array of groups
script-type-vecs = array of points
script-type-array = array
script-type-any = entity / vector

## Parameter types, without the "generator: " lead (Script Reference, generator topic)
script-param-type-choice-plain = one of choice(a|b|c)
script-param-type-angle-plain = degrees, a slider
script-param-type-percent-plain = 0–100, a slider
script-param-type-material-plain = a material name of the document

# Errors a script user sees (API, generators, parameter rigs)

## api.rs
script-err-k-expected-a-number-bool-or = { $k }: expected a number, bool or string, got { $type_name }
script-err-set-params-unknown-key-k-x = set_params: unknown key { $k } (x, y, z, count, axis, gap, stretch)
script-err-set-params-stretch-is-true-false = set_params: stretch is true / false
script-err-set-params-not-parametric-call-make = set_params: not parametric (call make_parametric first)
script-err-paint-no-material-n-create-it = paint: no material { $n } (create it with material(name, colour))
script-err-paint-expected-a-material-name-got = paint: expected a material name, got { $t }
script-err-show-hide-groups-and-components-only = show / hide: groups and components only
script-err-outer-shell-solids-must-be-in = outer_shell: solids must be in one context
script-err-group-entities-must-be-in-one = group: entities must be in one context
script-err-scale-zero-factor = scale: zero factor
script-err-rotate-zero-axis = rotate: zero axis
script-err-follow-me-the-path-must-be = follow_me: the path must be in the face's context
script-err-note-not-added = note: not added
script-err-dimension-not-added = dimension: not added
script-err-what-sizes-must-be-positive-a = { $what }: sizes must be positive (a torus tube thinner than its radius), segments 3..1000
script-err-cylinder-no-face = cylinder: no face
script-err-cylinder-radius-and-height-must-be = cylinder: radius and height must be non-zero, segments 3..1000
script-err-box-sizes-must-be-non-zero = box: sizes must be non-zero
script-err-circle-radius-must-be-positive-segments = circle: radius must be positive, segments 3..1000
script-err-face-points-are-not-a-valid = face: points are not a valid planar polygon
script-err-soft-not-an-edge = soft: not an edge
script-err-faces-not-an-edge-or-face = faces: not an edge or face
script-err-length-not-an-edge = length: not an edge
script-err-time-limit-of-secs-s-exceeded = time limit of { $secs }s exceeded
script-err-both-solids-must-be-in-the = both solids must be in the same context
script-err-expected-a-colour-rrggbb-or-r = expected a colour: "#RRGGBB" or [r, g, b]
script-err-bad-colour-s-use-rrggbb = bad colour { $s } (use #RRGGBB)
script-err-bad-colour-s = bad colour { $s }
script-err-curve-not-added = curve not added
script-err-degenerate-curve = degenerate curve
script-err-expected-edges-got-e = expected edges, got { $e }
script-err-expected-a-group-or-component-got = expected a group or component, got { $e }
script-err-expected-a-face-got-e = expected a face, got { $e }
script-err-entity-no-longer-exists-merged-split = entity no longer exists (merged, split or erased)
script-err-context-no-longer-exists = context no longer exists
script-err-expected-one-entity-got-len = expected one entity, got { $len }
script-err-expected-an-entity-or-an-array = expected an entity or an array of entities, got { $type_name }
script-err-expected-an-entity-got-did-the = expected an entity, got () (did the operation fail?)
script-err-no-group-component-named-name-here = no group / component named `{ $name }` here
script-err-expected-an-array-of-points = expected an array of points
script-err-expected-a-point-vec-or-x = expected a point (vec3 or [x, y, z]), got { $type_name }
script-err-expected-a-number-got-type-name = expected a number, got { $type_name }
script-err-not-a-length-s = not a length: { $s }
script-err-no-model = no model

## generator.rs
script-err-no-such-instance = no such instance
script-err-not-a-generated-component = not a generated component
script-err-no-such-context = no such context
script-err-no-generator-name-in-the-document = no generator `{ $name }` in the document
script-err-name-generators-nested-too-deep = { $name }: generators nested too deep
script-err-name-check-expr-is-not-true = { $name }: check `{ $expr }` is not true / false
script-err-expected-a-number-got-t = expected a number, got { $t }

## json.rs
script-err-bad-entity-id-s = bad entity id { $s }

## rig.rs
script-err-part-has-no-parameter-key = `{ $part }` has no parameter `{ $key }`
script-err-part-has-no-parameters = `{ $part }` has no parameters
script-err-no-part-part-in-the-component = no part `{ $part }` in the component
script-err-part-no-parameter-k-it-has = { $part }: no parameter `{ $k }` (it has { $keys })
script-err-part-the-part-has-no-parameters = { $part }: the part has no parameters (only generated components and components with parameters have)
script-err-part-show-hide-needs-a-yes = { $part }: show / hide needs a yes / no or number parameter
script-err-part-choose-the-part-s-parameter = { $part }: choose the part's parameter
script-err-part-the-direction-is-zero = { $part }: the direction is zero
script-err-part-the-axis-has-no-direction = { $part }: the axis has no direction
script-err-part-choose-a-parameter-or-write = { $part }: choose a parameter or write an expression
script-err-a-joint-without-a-part = a joint without a part
script-err-k-twice-parameters-and-formulas-share = `{ $k }` twice (parameters and formulas share names)
script-err-no-such-component = no such component
script-err-this-component-is-built-by-a = this component is built by a generator or a parametric group
script-err-groups-can-t-have-parameters-make = groups can't have parameters: make it a component first
script-err-not-a-component-with-parameters = not a component with parameters
script-err-part-the-part-has-no-parameters-2 = { $part }: the part has no parameters
script-err-part-no-such-part = { $part }: no such part
script-err-what-the-value-is-not-a = { $what }: the value is not a number, flag or text
script-err-what-the-value-is-not-a-2 = { $what }: the value is not a material name
script-err-what-the-value-is-not-true = { $what }: the value is not true / false
script-err-what-the-value-is-not-a-3 = { $what }: the value is not a number

## ======== Kernel messages ========

## curves.rs
kernel-err-the-curve-bounds-faces-outside-its = the curve bounds faces outside its plane
kernel-err-other-edges-are-attached-to-the = other edges are attached to the curve
kernel-err-too-few-points = too few points
kernel-err-segments-must-be-min-max-curve = segments must be { $min }..{ $MAX_CURVE_SEGMENTS }
kernel-err-radius-must-be-positive = radius must be positive
kernel-err-not-an-arc-circle-or-polygon = not an arc, circle or polygon

## chamfer.rs
kernel-err-the-edges-at-a-corner-are = the edges at a corner are collinear
kernel-err-a-corner-of-the-edge-is = a corner of the edge is not simple (too many edges or faces)
kernel-err-the-chamfer-is-too-big-for = the chamfer is too big for the edge
kernel-err-the-chamfer-is-too-big-for-2 = the chamfer is too big for the neighbouring edges
kernel-err-an-end-lies-along-the-edge = an end lies along the edge
kernel-err-the-radius-must-be-positive = the radius must be positive
kernel-err-the-distance-must-be-positive = the distance must be positive
kernel-err-the-faces-are-coplanar = the faces are coplanar
kernel-err-broken-face-loop = broken face loop
kernel-err-the-edge-is-too-short = the edge is too short
kernel-err-the-edge-must-be-shared-by = the edge must be shared by exactly two faces

## follow_me.rs
kernel-err-path-reverses = path reverses
kernel-err-profile-is-parallel-to-the-path = profile is parallel to the path
kernel-err-path-is-not-connected = path is not connected
kernel-err-path-branches = path branches
kernel-err-empty-path = empty path

## mod.rs
kernel-err-nothing-to-flip = nothing to flip
kernel-err-flip-axis-is-zero = flip axis is zero

## push_pull.rs
kernel-err-through-hole-cap-vertex-missing = through-hole: cap vertex missing
kernel-err-push-pull-would-collapse-geometry = push/pull would collapse geometry
kernel-err-face-loop-without-edge = face loop without edge
kernel-err-distance-is-not-finite = distance is not finite

## params.rs
kernel-err-empty-base = empty base
kernel-err-size-must-be-positive = size must be positive
kernel-err-not-parametric = not parametric
kernel-err-empty-group = empty group
kernel-err-already-parametric = already parametric

## spline.rs
kernel-err-the-curve-would-cross-itself = the curve would cross itself
kernel-err-segments-must-be-max-curve-segments = segments must be 1..{ $MAX_CURVE_SEGMENTS }
kernel-err-not-a-curve-that-can-be = not a curve that can be edited

## stats.rs
kernel-err-an-empty-name = an empty name

## text3d.rs
kernel-err-no-outlines = no outlines

## generator.rs
kernel-gen-name-no-parameter-k-join = { $name }: no parameter `{ $k }` ({ $join })
kernel-gen-empty-generator-name = empty @generator name
kernel-gen-not-a-generator-the-header-needs = not a generator: the header needs `// @generator Name`
kernel-gen-key-twice-formulas-and-parameters-share = `{ $key }` twice (formulas and parameters share names)
kernel-gen-key-bool-default-is-true-false = { $key }: bool default is true / false
kernel-gen-unknown-parameter-type-t-length-int = unknown parameter type `{ $t }` (length, int, number, bool, choice, angle, percent, material)
kernel-gen-key-choice-needs-options-choice-a = { $key }: choice needs options: choice(a|b|c)
kernel-gen-bad-parameter-name-key = bad parameter name `{ $key }`
kernel-gen-check-failed-expr = check failed: { $expr }
kernel-gen-key-empty-formula = { $key }: empty formula
kernel-gen-key-a-formula-is-length-int = { $key }: a formula is length, int, number, bool, angle, percent or text, not `{ $t }`
kernel-gen-bad-formula-name-key = bad formula name `{ $key }`
kernel-gen-key-bad-number-s-in-the = { $key }: bad number `{ $s }` in the range
kernel-gen-key-not-a-name-s = { $key }: not a { $name }: { $s }
kernel-gen-key-s-is-not-one-of = { $key }: `{ $s }` is not one of { $join }

## model.rs
kernel-err-the-generator-s-script-can-t = the generator's script can't be copied
kernel-err-nothing-selected = nothing selected

## solid.rs
kernel-err-pick-at-least-two-solids = pick at least two solids
kernel-err-the-second-solid-has-a-degenerate = the second solid has a degenerate transform
kernel-err-the-solids-do-not-overlap = the solids do not overlap
kernel-err-pick-two-different-solids = pick two different solids
# $name is the group / component name
kernel-err-is-not-a-solid-every-edge = “{ $name }” is not a solid (every edge must bound exactly two faces, no nested groups)

## OpError
kernel-err-entity-not-found = entity not found
kernel-err-degenerate-geometry = degenerate geometry
# $why is the reason
kernel-err-operation-not-possible = operation not possible: { $why }

## ======== Import and export ========

## dae.rs
kernel-io-not-a-collada-document = not a COLLADA document
kernel-io-nodes-nested-too-deep = nodes nested too deep

## dxf.rs
kernel-io-no-supported-entities-line-polyline-dface = no supported entities (LINE, POLYLINE, 3DFACE, CIRCLE, ARC)
kernel-io-not-a-dxf-file-no-section = not a DXF file (no SECTION)
kernel-io-not-a-dxf-file = not a DXF file

## fbx.rs
kernel-io-fbx-description = FBX: { $description }

## gltf.rs
kernel-io-index-out-of-range = index out of range
kernel-io-no-meshes-in-the-gltf-scene = no meshes in the glTF scene
kernel-io-node-hierarchy-too-deep = node hierarchy too deep
kernel-io-draco-e = Draco: { $e }
kernel-io-draco-primitive-without-bufferview = Draco primitive without bufferView
kernel-io-accessor-out-of-range = accessor out of range
kernel-io-unsupported-component-type = unsupported component type
kernel-io-unsupported-accessor-type = unsupported accessor type
kernel-io-accessor-without-count = accessor without count
kernel-io-bufferview-out-of-range = bufferView out of range
kernel-io-bufferview-without-bytelength = bufferView without byteLength
kernel-io-bad-buffer-index = bad buffer index
kernel-io-buffer-without-data = buffer without data
kernel-io-compressed-gltf-e-is-not-supported = compressed glTF ({ $e }) is not supported
kernel-io-only-gltf-is-supported = only glTF 2.0 is supported
kernel-io-external-file-uri-needs-a-path = external file { $uri } needs a path
kernel-io-bad-base-in-data-uri = bad base64 in data URI
kernel-io-not-a-gltf-file-e = not a glTF file: { $e }
kernel-io-glb-without-json = GLB without JSON
kernel-io-glb-json-e = GLB JSON: { $e }
kernel-io-truncated-glb-chunk = truncated GLB chunk

## meshes.rs
kernel-io-no-faces-in-the-file = no faces in the file

## native.rs
kernel-io-unsupported-version-version = unsupported version { $version }
kernel-io-not-a-varstotas-material-format-format = not a varstotas material (format = { $format })
kernel-io-missing-root-definition = missing root definition
kernel-io-not-a-varstotas-file-format-format = not a varstotas file (format = { $format })

## obj.rs
kernel-io-obj-line-ln-line = OBJ line { $ln }: { $line }

## ply.rs
kernel-io-no-faces-a-point-cloud-can = no faces (a point cloud can't be imported)
kernel-io-list-too-long = list too long
kernel-io-unknown-format-f = unknown format { $f }
kernel-io-property-before-element = property before element
kernel-io-bad-element-count = bad element count
kernel-io-no-end-header = no end_header
kernel-io-not-a-ply-file = not a PLY file
kernel-io-truncated-data = truncated data
kernel-io-unknown-type-s = unknown type { $s }

## points.rs
kernel-io-no-points-in-display = no points in { $display }

## stl.rs
kernel-io-stl-vertex-count-not-a-multiple = STL vertex count not a multiple of 3
kernel-io-bad-stl-vertex-line = bad STL vertex: { $line }
kernel-io-not-a-valid-stl = not a valid STL

## threemf.rs
kernel-io-mf-no-d-model-part = 3MF: no 3D model part
kernel-io-mf-no-object-id = 3MF: no object { $id }
kernel-io-mf-components-nested-too-deep = 3MF: components nested too deep
kernel-io-mf-no-part-path = 3MF: no part { $path }

## xml.rs
kernel-io-xml-no-root-element = XML: no root element
kernel-io-xml-unbalanced-tags = XML: unbalanced tags
kernel-io-xml-e = XML: { $e }

## zip.rs
kernel-io-name-zip-method-m-not-supported = { $name }: zip method { $m } not supported
kernel-io-truncated-zip = truncated zip
kernel-io-bad-zip-entry-name = bad zip entry { $name }
kernel-io-bad-zip-directory = bad zip directory

## Prefixes of the COLLADA / PLY readers
# $msg is the reader's own message
kernel-io-collada = COLLADA: { $msg }
kernel-io-ply = PLY: { $msg }

## IoError
# $why is the system error / the reader's message
kernel-io-error-io = io: { $why }
kernel-io-error-format = format: { $why }

## File templates
template-files-heading = From template files
template-files-empty = No .vrst templates in this folder
template-open-folder = Open Templates Folder…
template-folder-error = Could not read templates: { $error }
template-created = Created from template: { $path }
template-load-failed = Could not load template { $path }: { $error }
template-folder-label = Templates folder
template-folder-tip = Place .vrst models here to use them as starting points for new documents.

app-export-replace-title = Replace export files?
app-export-replace-files =
    Export will replace these material or texture files:
    { $files }
    Continue?
app-export-folder = Choose the folder for { $file } and its files (materials, textures)
app-export-replace-all =
    Export will replace these files:
    { $files }
    Continue?
app-import-missing-title = Files next to the model are missing
app-import-missing =
    { $file } refers to files next to it that Varstotas could not read:
    { $files }
    Choose the folder the model is in to load them? (An app in a sandbox, such as the Flatpak, sees only the file picked in the dialog.)
app-import-not-in-folder = { $file } is not in { $folder }

## Object and tag locking
cmd-lock = Lock
cmd-unlock = Unlock
cmd-unlock-all = Unlock All
cmd-desc-lock = Lock selected objects against selection and editing
cmd-desc-unlock-all = Unlock all objects and tags
edit-locked = This operation would change locked objects. Unlock them first.
script-member-locked = Effective editing lock (read-only; includes tag and ancestor locks).

## Startup screen and feedback
start-menu = Start Screen
start-subtitle = Create a model, continue your work, or explore the examples.
start-create = New model
start-new = Create
start-open = Open a document…
start-guide = Quick guide
start-recent = Recent documents
start-no-recent = Your recently opened documents will appear here.
start-basics-title = Getting started
start-basics = Space: Select · L: Line · R: Rectangle. Type exact dimensions in Measurements. Use the mouse wheel to zoom and the middle button to orbit.
start-show = Show at startup
start-continue = Continue
report-menu = Report a Problem…
report-intro = Review the report, then open a draft in your browser. Sign in to GitHub to submit it yourself.
report-access = A private repository requires access. Model files and screenshots are not attached automatically.
report-title = Summary
report-description = What happened?
report-description-hint = Steps to reproduce, expected result, and what actually happened.
report-errors = Include the last 10 warnings and errors
report-review = Review the exact report below before sharing it. Logs may contain file names or other details.
report-copy = Copy report
report-github = Open GitHub draft
report-paste = The full report was copied. Paste it into the GitHub issue body.

cli-usage =
    Usage: Varstotas [--open] <document.vrst>
    Varstotas -- <document.vrst>
    Varstotas --help | --version
cli-one-document = Specify one document
cli-unknown-option = Unknown option: { $option }
cli-open-path = --open requires a document path

prefs-save-backups = Keep a backup of the previous saved version
prefs-save-backups-note = Explicit saves keep the previous version in { DATA_DIR() }/backups as <name> (…).bak.vrst. Turning this off leaves existing backups untouched.

model-info-metadata = Document metadata
model-info-metadata-text = Text / description
model-info-license = License
model-info-license-hint = License name, URL or full text (optional)
model-info-metadata-apply = Apply
model-info-metadata-reset = Discard changes
model-info-format-version = Save format version: { $version }
model-info-metadata-edit = Edit document metadata

model-info-tab-info = Info
model-info-tab-meta = Meta
model-info-tab-cleanup = Cleanup

download-progress-unknown = Downloaded { $received } / total unknown

# Practical step-by-step walkthrough.
tutorial-title = Tutorial
tutorial-step = Step { $current } / { $total }
tutorial-try = Try it
tutorial-tools = Activate a tool
tutorial-details = More about this step
tutorial-practice-note = Keep this window beside your model while you practise.
tutorial-back = Back
tutorial-restart = Start over
tutorial-finish = Finish
tutorial-next = Next
tutorial-0-title = Meet Varstotas
tutorial-0-body = Build models from edges and faces, then organise them into reusable components. Varstotas combines precise drawing, editable 3D shapes, materials, scenes and drawing sheets. This walkthrough takes you from a blank document to a model ready to share.
tutorial-0-task = Choose File → New. We will build a simple tabletop, explore its tools and prepare the file for sharing. Use Next at your own pace.
tutorial-1-title = Look around your model
tutorial-1-body = Drag with the middle mouse button to orbit; hold Shift to pan. The wheel zooms toward the cursor. Trackpad navigation and speeds can be adjusted in Preferences → Navigation. Camera views and projection are also available in the viewport corner.
tutorial-1-task = Try orbiting and panning. Switch between Top and Iso in Camera → Standard Views, then use Zoom Extents to frame the model.
tutorial-2-title = Draw a precise base
tutorial-2-body = Edges form faces when a closed outline is planar. Rectangle and Circle create a face directly. The Measurements box accepts exact sizes while a tool is active; tooltips and the Tool Guide card explain the current stage.
tutorial-2-task = Activate Rectangle. Click a first corner, move the pointer and type 1400mm,800mm into Measurements, then press Enter. You have a tabletop outline.
tutorial-3-title = Give it volume
tutorial-3-body = Push/Pull turns a face into a solid. Draw → Shapes creates a box, cylinder, cone, pyramid, sphere or torus in one operation. These shapes are components with editable generator parameters in Properties.
tutorial-3-task = Use Push/Pull on the rectangle and enter 40 for its thickness. Try Box nearby: choose the first corner, opposite base corner, then height.
tutorial-4-title = Let points snap into place
tutorial-4-body = The cursor identifies endpoints, midpoints, axes and grid points. Coloured inference lines guide the direction. Arrow keys lock axes; Shift locks the current inference. The magnet menu controls individual snap types.
tutorial-4-task = Show the grid and activate Line. Hover a corner, then follow an axis to another grid line. Repeat with Box and notice snapping at the base and height stages. Escape cancels an unfinished operation.
tutorial-5-title = Edit and reuse
tutorial-5-body = Move, Rotate and Scale reshape a selection; Undo lets you experiment. A group isolates geometry from nearby edges. Components share one definition, so editing one copy updates its siblings.
tutorial-5-task = Select the tabletop and make a group from Edit. Move it, undo the move, and double-click the group to edit inside. Close the editing context before continuing.
tutorial-6-title = Keep a model organised
tutorial-6-body = Outliner shows nested groups and components. Tags control visibility and editing locks. Properties exposes the current selection and its parameters. Hidden and locked objects are separate states.
tutorial-6-task = Give the tabletop a name in Properties. Create a tag in Tags, assign it and toggle visibility. Try the lock, then unlock it again before painting.
tutorial-7-title = Add materials and choose a style
tutorial-7-body = Paint assigns colours and textures; Texture Position adjusts their placement. Materials Online Library downloads reusable materials. Shaded with Textures is useful while modelling; Monochrome exposes front/back faces, and Realistic uses physical material properties and lighting.
tutorial-7-task = Choose a material and paint a face. Compare Shaded with Textures, Monochrome and Realistic in the style menu. Try X-Ray to see hidden edges.
tutorial-8-title = Present the result
tutorial-8-body = Scenes remember views and model display settings. Dimensions annotate the model, section planes reveal the inside, and sheets arrange views on paper. Render creates a finished image; animation can move between scenes.
tutorial-8-task = Frame an Iso view and add a scene. Make a second scene from Top. Switch between them, then try Dimension or Section on your model.
tutorial-9-title = Save and share
tutorial-9-body = Native .vrst files preserve editable geometry, materials, scenes and metadata. Document Info → Meta stores a description, license and preview. Import/export connects to OBJ, STL, GLB and other formats; scripts and plugins extend the tools.
tutorial-9-task = Save the model. Add a description and license in Document Info → Meta and save again. Explore Help → Examples for downloadable projects, or repeat a step to practise further.

app-reversed-faces = Reversed { $count } face(s). Use Monochrome to inspect front/back orientation.

download-progress-previews = Downloading previews: { $count }

prefs-trackpad-enabled = Enable trackpad / smooth-scroll navigation
prefs-gesture-speed = Sensitivity
prefs-invert-swipe = Invert two-finger swipe
prefs-pinch-enabled = Pinch to zoom
prefs-invert-pinch = Invert pinch
prefs-twist = Two-finger twist
prefs-twist-off = Off
prefs-twist-orbit = Orbit around vertical axis
prefs-twist-roll = Roll view
prefs-invert-twist = Invert twist
prefs-trackpad-platform = Native pinch and twist depend on OS and driver support. Smooth-scroll devices also use these settings.

view-triangles = Triangles
view-triangles-hint = Show every tessellation edge over the current style, including soft edges and polygon diagonals.

# 3D Printing: basic surface checks and conservative repair.
menu-3d-printing = 3D Printing
print-title = 3D printing preparation
print-check-command = Check Model…
print-repair-command = Repair Mesh…
print-check-description = Inspect the selected objects or visible model for basic print-surface defects.
print-repair-description = Repair mesh seams, duplicates, orientation and optional small planar holes.
print-scope = Scope:
print-visible = Visible model
print-selection = Selection
print-tab-check = Print Check
print-tab-repair = Repair
print-working = Working on a model snapshot…
print-worker-failed = The check could not finish. Run it again.
print-result-stale = The model or selection changed. The repair was not applied; run the check again.
print-repair-undo = Repair for 3D printing
print-stale = This report is out of date. Run the check again.
print-repair-weld = Join coincident vertices
print-repair-degenerate = Remove degenerate faces
print-repair-duplicates = Remove duplicate faces
print-repair-orient = Make face winding consistent
print-repair-shells = Orient closed shells, preserving nested cavities
print-repair-holes = Close small planar holes
print-hole-size = Maximum hole size:
print-repair-note = Repairs can be undone. Locked, hidden and procedural geometry is preserved. Complex or nested hole boundaries are left for manual repair.
print-repair-whole-objects = For repair, select whole groups/components or switch to Visible model.
print-repair-run = Repair and recheck
print-repaired = Joined vertices: { $welded }; removed faces: { $removed }; reversed faces: { $reversed }; closed holes: { $holes }; skipped objects: { $skipped }.
print-empty = No visible faces in this scope. Select an object or choose Visible model.
print-counts = Faces: { $faces } · triangles: { $triangles } · shells: { $shells } ({ $closed } closed)
print-size = Size: { $x } × { $y } × { $z } mm
print-area = Surface area: { $area } mm²
print-volume = Enclosed volume: { $volume } mm³
print-found-problems = Surface issues found
print-basic-pass = Basic surface checks passed
print-limitations = Wall thickness is sampled. See Analysis and Printer tabs for intersections, overhangs and build-volume checks. Verify sliced layers before printing.
print-tolerance = Vertex matching tolerance:
print-highlight = Highlight issues in the viewport
print-check-run = Run check
print-copy = Copy report
print-open-edges = Open boundary edges
print-nonmanifold = Edges with more than two faces
print-winding = Inconsistent edge winding
print-degenerate = Degenerate triangles / empty faces
print-duplicates = Duplicate triangles
print-inverted = Inverted closed shells
print-wire = Loose edges (not printable)
print-export-visible = Export visible model

# Printing preparation tools
print-analysis = Analysis
print-bed = Printer & bed
print-split = Split & connectors
print-export = Print export
print-export-stale = The model or settings changed. Export again before opening the slicer.
print-cut-undo = Split for printing
print-place-undo = Place on print bed
print-placement-done = Placement updated. Run Check to refresh the report.
print-cut-select = Select one whole group/component to cut.
print-cut-done = Created { $parts } parts, including { $dowels } separate dowels. Arrange them on the bed.
print-export-extension = Choose .3mf or .stl.
print-export-done = Exported { $parts } parts to { $path }
print-slicer-opened = The exported file was opened in the default app for it.
print-min-wall = Minimum wall thickness
print-two-nozzles = Use two nozzle widths
print-sample-budget = Maximum wall samples
print-pair-budget = Maximum intersection candidates
print-overhang-angle = Overhang angle from downward vertical
print-map = Surface map
print-map-none = None
print-map-wall = Wall thickness
print-map-overhang = Overhangs
print-intersection-count = Intersecting triangle pairs: { $n }; candidates checked: { $pairs }
print-intersection-incomplete = Candidate limit reached: intersection analysis is incomplete.
print-wall-coverage = Wall samples: { $samples } / { $possible }; triangles sampled: { $triangles } / { $total }
print-wall-result = Minimum measured: { $minimum } mm; thin samples: { $thin }; unknown: { $unknown }
print-overhang-result = Overhang area: { $area } mm² ({ $n } triangles)
print-wall-unreliable = Open, overlapping or inconsistently oriented surfaces make wall measurements unreliable.
print-wall-estimate = Thickness uses inward rays at up to four points per triangle. Small details between samples can be missed. Grey means no valid exit was found; red means below the threshold. Overhangs exclude bed contact. Verify the sliced layers before printing.
print-profile-name = Printer profile
print-bed-width = Bed width X
print-bed-depth = Bed depth Y
print-bed-height = Build height Z
print-bed-margin = Bed margin
print-bed-gap = Part spacing
print-nozzle = Nozzle diameter
print-arrange-rotate = Allow 90° rotation when arranging
print-bed-show = Show print volume in the viewport
print-drop = Drop to bed
print-center = Center on bed
print-arrange = Arrange parts
print-lay-face = Lay selected face on bed
print-placement-note = Placement keeps scale and uses whole parts. Arrangement uses bounding boxes and may reject shapes that could fit with manual packing. Overlaps below are bounding-box warnings.
print-bed-result = Parts: { $parts }; outside: { $outside }; off bed: { $floating }; bounding-box overlaps: { $overlaps }
print-fits = Fits
print-outside = Outside print volume
print-cut-normal = Cut plane normal
print-cut-offset = Plane offset from world origin
print-connectors = Add sockets and separate dowels
print-dowel-count = Number of dowels
print-dowel-diameter = Dowel diameter
print-dowel-length = Dowel length
print-dowel-clearance = Radial and axial clearance
print-dowel-wall = Minimum surrounding wall
print-dowel-note = Cuts require closed, consistently oriented parts. Sockets are distributed along the cut and checked for surrounding material. Separate dowels and both halves remain at assembly positions; arrange them afterward. Calibrate clearance for your printer.
print-cut-run = Cut and cap
print-cut-to-bed = Split to printer dimensions
print-cut-to-bed-note = Automatic cuts use the usable X/Y/Z printer dimensions, up to 256 parts. This mode creates capped parts without connectors and keeps assembly positions.
print-export-scope-note = Export uses the selected scope above and preserves world coordinates in millimetres. 3MF keeps separate objects; STL contains triangles.
print-export-materials-note = 3MF includes base colours, names, model description, licence and source credits. PBR textures are not exported. STL contains geometry only.
print-export-3mf = Export 3MF…
print-export-stl = Export STL…
print-slicer-open = Open in Slicer
print-slicer-note = Opens the export in the system's default app for .3mf / .stl: make your slicer the default for these files. Changes to the model require a fresh export.
print-export-missing = The exported file no longer exists. Export it again.

# Printing operation errors
print-error-1 = Invalid parent transform
print-error-2 = No visible printable faces in this scope
print-error-3 = Invalid printer dimensions or margin
print-error-4 = Locked or procedural objects cannot be changed
print-error-5 = The part contains locked geometry
print-error-6 = Unhide all part geometry before splitting
print-error-7 = The part contains locked edges
print-error-8 = The part contains locked objects
print-error-9 = Select whole parts
print-error-10 = Select whole groups/components for placement and cutting
print-error-11 = Group loose geometry before placement when other root faces are hidden
print-error-12 = No visible parts
print-error-13 = Part has no surface
print-error-14 = Singular parent transform
print-error-15 = Invalid printer dimensions
print-error-16 = Empty parts
print-error-17 = Parts do not fit on one bed at their original scale; choose fewer parts or split them
print-error-18 = Degenerate face
print-error-19 = The chosen face is recessed; it cannot lie on the bed without geometry below it
print-error-20 = Ambiguous cut boundary; repair the part before splitting
print-error-21 = Open cut boundary
print-error-22 = Invalid cut boundary
print-error-23 = Cut hole has no outer contour
print-error-24 = The capped cut is not a closed manifold; repair the source mesh first
print-error-25 = The plane does not divide this part
print-error-26 = Invalid cut plane
print-error-27 = Choose exactly one whole part to split
print-error-28 = Repair open, duplicate or inconsistent surfaces before cutting a printable part
print-error-29 = Cut requires a complete intersection check with no intersecting surfaces
print-error-30 = Choose one whole part
print-error-31 = More than 256 bed-sized parts would be required; use a larger profile or cut in stages
print-error-32 = Invalid dowel dimensions, count or clearance
print-error-33 = No cap for connector placement
print-error-34 = Connectors are too close; reduce their count or diameter
print-error-35 = A connector would leave the part or has insufficient wall/depth. Reduce size/count, change the plane or use a thicker cut region
print-error-36 = Invalid socket
print-error-37 = Socket subtraction failed
print-error-38 = Invalid dowel
print-bed-frame = Frame print volume
print-part-name = Print part
print-socket-name = Socket cutter
print-dowel-name = Registration dowel

print-go-check = Go to Print Check
print-analysis-run = Run analysis
print-bed-refresh = Refresh bed check

# Window → Physics
physics-title = Physics
physics-drop = Drop
physics-drop-hint = The selected groups and components fall and come to rest on everything else that is visible.
physics-shape = Shape
physics-shape-hull = Hull
physics-shape-hull-hint = The convex hull of each object: fast and steady, hollows are filled in.
physics-shape-pieces = Pieces
physics-shape-pieces-hint = Convex pieces of each object: a bowl can hold things, slower to prepare.
physics-friction = Friction
physics-bounce = Bounce
physics-gravity = Gravity
physics-time = Time limit
physics-floor = Floor at Z = 0
physics-no-bodies = Select groups or components to drop.
physics-bodies = Objects to drop: { $n }
physics-run = Run
physics-running = Simulating… { $time } s
physics-done = At rest after { $time } s
physics-stopped = Stopped at { $time } s
physics-fell = Objects with nothing under them: { $n } (they stay where they were)
physics-apply = Apply
physics-again = Run again
physics-stop = Stop
physics-cancel = Cancel
physics-interrupted = The model was changed: the run stopped.
physics-failed = Simulation failed: { $error }
physics-undo = Drop
physics-applied = Objects moved: { $n }
physics-cloth = Cloth
physics-cloth-hint = The selected face (or a group holding one face) falls as cloth and drapes over everything else that is visible. It becomes a group of smooth triangles.
physics-cloth-no-face = Select one face, or a group holding one face.
physics-cloth-too-small = The face is smaller than the spacing.
physics-cloth-too-fine = { $n } points: raise the spacing.
physics-cloth-particles = Cloth points: { $n }
physics-cloth-spacing = Spacing
physics-cloth-spacing-hint = The distance between the cloth's points: smaller makes finer folds and a slower run.
physics-cloth-stiffness = Stiffness
physics-cloth-stiffness-hint = How much the cloth resists stretching.
physics-cloth-bending = Bending
physics-cloth-bending-hint = How much the cloth resists folding: low for silk, high for leather.
physics-cloth-thickness = Thickness
physics-cloth-thickness-hint = The gap the cloth keeps from what it lies on (at least a tenth of the spacing).
physics-cloth-self = Does not pass through itself
physics-cloth-self-hint = Folds collide with each other (slower).
physics-cloth-undo = Cloth
physics-cloth-applied = The cloth is a new group.
