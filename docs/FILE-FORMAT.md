# Native document format

`.vrst` is UTF-8 JSON with an explicitly versioned envelope:

```json
{
  "format": "varstotas",
  "version": 4,
  "model": {
    "metadata": {
      "text": "A description, author notes or other free-form text.",
      "license": "A license name, URL or complete license text."
    }
  }
}
```

The example omits geometry and other required model fields. Both metadata fields are plain
text, support Unicode and multiple lines, and default to empty strings. An empty license
makes no licensing claim. Document metadata does not replace attribution attached to imported
components or materials.

Window → Document Info → Meta edits these fields and contains the document preview. Apply records one undoable document change; save the
document to write it to disk. Discard changes restores the current document values. Closing
Document Info without applying discards the draft. Metadata is also preserved by native autosave,
backup, Save As and file templates.

Authored normals preserve glTF/GLB smooth shading and hard seams. They survive native IO,
reversal and whole-face transforms (inverse transpose); partial deformation/topology edits
invalidate the authored values and use generated shading. Empty/missing arrays retain legacy
normal generation. Invalid lengths and non-finite/zero normals are rejected.

Per-face texture placement preserves rotation, independent U/V scale and shear, with a
planar origin. For `r = R(-angle) × (q - offset)`, texture coordinates in mm are
`(r.x / scale.x, (r.y + shear × r.x) / scale.y)`. Missing shear defaults to zero, preserving
older placements. Source atlas coordinates are fitted at the actual welded mesh vertices.
Older files cannot recover discarded UV shear without their original source assets.

Materials optionally store `stock`: what the material is bought as, a list of
`{ "kind": "sheet" | "board" | "linear", "name", "thickness", "length", "width", "lengths": [...], "grain", "price", "piece_prices": [...] }`
(mm; a sheet uses length × width, a board its width and the lengths on sale; linear stock is a
profile `name` cut to length like a board, its width × thickness only a limit, 0 = any; `price` is per
sheet or per metre of board, `piece_prices` when present prices boards per piece, one per length).
Offcuts kept for later projects live in the app's preferences, not in documents. The model optionally
stores `cut_list`: `{ "kerf", "trim", "min_offcut", "tolerance", "parts": { "<part name>": { "grain":
"Length" | "Width" | "Any", "exclude" } } }`. A sheet (track D) optionally stores `cut_diagrams`:
`[{ "min", "size", "page" }]`, a page of the cut list fitted into that rectangle. Both are omitted while empty or default, so older
readers and files are unaffected.

Scenes optionally store the independent `style.triangles` display checkbox (default false).

## Versions and compatibility

| Version | Schema |
| --- | --- |
| 1 | Original native model schema; metadata may be absent. |
| 2 | Document metadata (`model.metadata.text` and `model.metadata.license`). |
| 3 | Optional authored corner shading normals (`face.normals`, outer loop then holes). |
| 4 | Full affine per-face UV placement, including `face.uv.shear`. |

The writer emits version 4. The reader accepts versions 1–4, supplies empty metadata when
absent, and validates the model before rebuilding caches. A version-1 file is migrated in
memory; its original bytes remain unchanged until an explicit save. Normal backup settings
apply when saving it. Older application builds reject files newer than their supported version.

Missing, zero and future versions are rejected. The envelope's format/version is checked
before decoding the model schema so unsupported files produce a version error. Future schema
changes must add explicit reader migration/defaults and compatibility tests; breaking changes
increment the native version. The format version is independent of the application version.

Material files (`.vrstmat`, `format: "varstotas-material"`) retain their independent version 1.
