# Dialogs: save changes, autosave recovery, resize, export, 3D text, templates, About.

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
dialog-about-licence = Free software under the MIT licence.
dialog-about-licences = Licences…
