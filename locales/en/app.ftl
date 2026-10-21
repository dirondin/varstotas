# The application shell: flash messages, confirmations, window titles, errors.

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
app-no-scenes = No scenes to play. Add scenes in the Scenes panel.
app-demo-house = Demo: Country House — see the Scenes card and Window → Render…
app-demo-gallery = Demo: Material Gallery — see the Scenes card and Window → Render…
# { $path } is a file path.
app-opened = Opened { $path }
app-could-not-open = Could not open { $path }: { $error }
app-saved = Saved { $path }
app-save-failed = Save failed: { $error }
# { $warning } is a warning text.
app-exported-warning = Exported { $path }. { $warning }
app-exported = Exported { $path }
app-export-failed = Export failed: { $error }
# { $format } is the format name.
app-undo-import = Import { $format }
app-imported = Imported { $path }
app-undo-insert-demo = Insert Demo Geometry
app-autosave-failed = Autosave to { $path } failed: { $error }
# { $name } is a file name.
app-recovered = Recovered { $name } from autosave
app-could-not-recover = Could not recover { $name }: { $error }

## File dialog filters

app-filter-image = Image
app-filter-script = Script
app-filter-model = Varstotas model
