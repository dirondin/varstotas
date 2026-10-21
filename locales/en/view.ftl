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
