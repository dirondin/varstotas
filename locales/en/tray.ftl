# The side bar (tray): cards, Entity Info, Materials card, Tags, Outliner, Scenes, Shadows,
# Soften Edges, Instructor, and the shared parts of the libraries' tray frames (tray_lib).

## Cards (titles are ids in the preferences; these are the shown names)

tray-card-entity-info = Entity Info
tray-card-materials = Materials
tray-card-components = Components
tray-card-tags = Tags
tray-card-outliner = Outliner
tray-card-scenes = Scenes
tray-card-shadows = Shadows
tray-card-environment = Environment
tray-card-soften-edges = Soften Edges
tray-card-instructor = Instructor
tray-card-model-info-tip = Model Info: statistics of the document, Purge Unused
tray-cards-all-off = Every card is off: Window → Tray cards

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

## Entity Info

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
tray-params-make-hover = Size and copies set in Entity Info
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
tray-tex-position = Position…
tray-tex-position-hover = Texture Position tool (pins)
tray-tex-rotate = Rotate 90°
tray-tex-reset = Reset
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
tray-light-render-hover = Window → Render…
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

## Instructor

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
