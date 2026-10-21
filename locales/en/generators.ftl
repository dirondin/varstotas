# Generators: the generator editor and the generator tiles / Entity Info block.

## Generator editor
gen-editor-title = Generator Editor
gen-new = New
gen-new-hint = Start from a template
gen-open = Open…
gen-filter = Generator
gen-save = Save to Library
gen-save-hint = Write the generator to ~/.varstotas/generators (it shows in the Components tray)
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
    Generator — parameters in Entity Info
    { $note }
gen-edit = Edit Generator…
gen-show-in-finder = Show in File Manager
    .macos = Show in Finder
    .windows = Show in Explorer
gen-delete = Delete…
gen-error = Generator: { $error }

## Entity Info block
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
