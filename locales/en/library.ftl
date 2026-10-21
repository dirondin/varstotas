# The Components library card: sections, tiles, dialogs, context menus, messages.

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
library-online-hover = 3D Library: free models from the web (Poly Haven, Smithsonian, NASA, Kenney, plugins)
library-save-selection = Save Selection to Library…
library-select-first = Select something first
library-new-generator = New Generator…
library-new-generator-hover = A component built by a script with parameters (Window → Generator Editor)
library-choose-folder = Choose Folder…
library-default-folder = Default Folder
library-default-folder-hover = Back to ~/.varstotas/components
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
