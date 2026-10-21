# Window → Preferences.

prefs-language = Language
# { $language } is the name of the system's language.
prefs-language-system = As the System ({ $language })
prefs-language-note = Translations of your own: put <code>.ftl files into ~/.varstotas/locales/<code>/ (see docs/LOCALIZATION.md); they also override single texts of a built-in language.

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

prefs-words-general = auto-save autosave minutes template new document units undo steps history
prefs-words-render = render image size samples bounces quality denoise programs ffmpeg video animation oidn open image denoise
prefs-words-drawing = snapping inference endpoint midpoint edge face axis grid step major increments move rotate scale angle
prefs-words-libraries = components materials folder examples in model tiles small large list
prefs-words-downloads = watch downloads folder browser
prefs-words-interface = theme light dark system appearance scale size zoom toolbar icons language translation
prefs-words-navigation = mouse trackpad orbit zoom pan speed invert wheel cursor middle button two fingers gesture
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
prefs-file-not-usable = { $path } (not usable)
prefs-file-found = { $path } (found)
prefs-choose = Choose…
prefs-default = Default
prefs-show-in-finder = Show in File Manager
    .macos = Show in Finder
    .windows = Show in Explorer

## General

prefs-autosave = Auto-save
prefs-autosave-every = Auto-save every
prefs-minutes = minutes
prefs-autosave-note = Unsaved changes are written to <name>.vrst~autosave next to the file.
prefs-new-document = New document
prefs-template-note = The template of File → New (units and scale); its view is set in the View section.
prefs-undo = Undo
prefs-keep = Keep
prefs-undo-steps = undo steps per document
prefs-undo-note = Each step keeps a copy of the model: big models with many steps take more memory.

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
prefs-render-note = File → New starts with these; Window → Render… changes the open document's.
prefs-programs = Programs
prefs-ffmpeg = ffmpeg (videos of File → Export → Animation)
prefs-ffmpeg-missing = not found: install it (brew install ffmpeg) or choose the program
prefs-oidn = Open Image Denoise library (the render's denoiser)
prefs-oidn-missing = not found: the built-in filter denoises (brew install open-image-denoise, or choose the library)

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
prefs-downloads-note = Also in Window → 3D Library… → Other Sites.

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
prefs-trackpad-note = Shift: Pan (Orbit when two fingers pan). Pinch and %CMD% + two fingers always zoom.

## View (defaults of a new document)

prefs-view-note = How a new document looks (File → New, New from Template). The View menu and the viewport's buttons change the current view.
prefs-camera = Camera
prefs-fov = Field of view
prefs-use-current-view = Use Current View
prefs-use-current-view-tip = Take the style, switches and camera of the active viewport
prefs-apply-current-view = Apply to Current View
prefs-apply-current-view-tip = Set the active viewport like a new document
