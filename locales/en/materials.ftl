# The Materials library (side bar card), online materials, PBR import.

## Library card

mat-plus-hover = New material from an image, import material files
mat-online-hover = Online Materials: free PBR materials (Poly Haven, ambientCG, plugins)
mat-new-from-texture = New from Texture…
mat-new-from-texture-hover = A material from an image file (PNG, JPEG)
mat-import = Import…
mat-import-hover = Add material files (.vrstmat, plugin formats) to the document
mat-choose-folder = Choose Folder…
mat-default-folder = Default Folder
mat-default-folder-hover = Back to ~/.varstotas/materials
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

mat-online-title = Online Materials
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
