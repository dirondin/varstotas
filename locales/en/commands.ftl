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
cmd-export-animation = Animation (Video / PNG)…
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
cmd-d = 3D Text…
cmd-zoom-extents = Zoom Extents
cmd-snapping = Snapping
cmd-show-grid = Show Grid
cmd-side-bar = Side Bar
cmd-render = Render…
cmd-work-planes = Work Planes…
cmd-model-info = Model Info…
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
cmd-desc-text-3d = Add text as 3D geometry (a group).
cmd-desc-zoom-extents = Fit the whole model in the view.
cmd-desc-snapping = Snapping on / off.
cmd-desc-show-grid = The grid on the ground on / off.
cmd-desc-side-bar = The side bar with the cards (Entity Info, Materials …) shown or hidden.
cmd-desc-render = Path-traced pictures of the model (Window → Render).
cmd-desc-work-planes = Saved drawing planes.
cmd-desc-model-info = Statistics of the document, Purge Unused.
cmd-desc-component-params = Parameters of the selected component: values that turn, slide, show or paint its parts.
cmd-desc-library-3d = Free models from online libraries.
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
