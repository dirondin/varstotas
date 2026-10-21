# Sites without an API in the 3D Library: Search on, Other Sites, Watch Downloads, the Import
# Download window, dropped files and links.

## Messages
# $path is a file.
downloads-no-models = { $path }: no model files in it
downloads-no-address = { $path }: no web address in it
downloads-not-model = { $path }: not a model file (glb, gltf, fbx, dae, obj, 3mf, ply, stl, dxf, varstotas, zip)
# $site is the site, $title the model.
downloads-waiting-site = Waiting for a file from { $site }: download it in the browser
downloads-waiting-file = Waiting for the file of { $title } ({ $site }): download it in the browser
downloads-no-library = No components library folder
# $name is the file, $folder the library folder, $n the number of models.
downloads-imported = Imported { $name } to Components → { $folder }
downloads-imported-many = Imported { $n } models of { $name } to Components → { $folder }
downloads-import-failed = Import { $name }: { $error }
downloads-err-no-file-name = no file name
downloads-err-no-model-chosen = no model chosen
# $label is a licence, $names the first models under it, $more " +N" or empty, $note what it asks.
downloads-licence-part = { $label } ({ $names }{ $more }) — { $note }
# $list is the licence-part texts.
downloads-licences = Licences: { $list }. File → Export → Attributions lists the credits.

## Search on, Other Sites

downloads-search-on = Search on:
# $site is the site, $query the search.
downloads-search-hover = Open { $site }'s search for «{ $query }» in the browser
downloads-edit = Edit…
downloads-edit-hover = Edit the sites (~/.varstotas/sources.toml); the list is read again
downloads-other = Other Sites
# $waiting is the number of waiting downloads.
downloads-other-waiting = Other Sites — waiting for { $waiting } download(s)
downloads-other-text = Download from any site in your browser: Varstotas doesn't visit the sites itself. It takes the file from your Downloads folder and keeps the page as the credit.
downloads-watch = Watch Downloads
downloads-watch-hover = Offer every new model file or archive of the Downloads folder for import (always on while a page waits)
# $dir is the watched folder.
downloads-watching = watching { $dir }
downloads-off = off
downloads-import-file = Import File…
downloads-import-file-hover = Import a model file or an archive you downloaded
downloads-models-filter = Models
# $title is the model, $site the site, $licence its licence.
downloads-waiting-card = Waiting for a file from this page: { $title } ({ $site }, { $licence })
downloads-page = Page
# $file is a file address.
downloads-browser-file = Download in the browser: { $file }
downloads-stop = Stop waiting

## Import Download window

downloads-import-title = Import Download
downloads-importing = Importing…
# $name is the file, $mb its size in megabytes.
downloads-file-size = { $name }  ({ $mb } MB)
downloads-credit-from-page = ✔ The credit is from the page you sent
downloads-duplicate = ⚠ The same file was imported before
downloads-field-title = Title
downloads-field-author = Author
downloads-field-site = Site
downloads-licence = Licence
downloads-licence-hint = as the page says
downloads-place = Place it in the model
# $on models are chosen of $n.
downloads-models-count = { $on } of { $n } models (each becomes a component)
downloads-all = All
downloads-none = None
# $folder is the library folder of the site.
downloads-destination = Components → { $folder }
downloads-ignore = Ignore
# $n is the number of other offers.
downloads-more-waiting = { $n } more waiting
