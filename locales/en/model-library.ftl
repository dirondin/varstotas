model-lib-licence-any = Any licence
model-lib-licence-commercial = Commercial use OK
model-lib-licence-cc0 = CC0 only
model-lib-licence-known = Known licence

## Window and filters

model-lib-title = 3D Library
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
