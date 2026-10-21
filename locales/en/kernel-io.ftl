# Import / export (file formats) errors.

## dae.rs
kernel-io-not-a-collada-document = not a COLLADA document
kernel-io-nodes-nested-too-deep = nodes nested too deep

## dxf.rs
kernel-io-no-supported-entities-line-polyline-dface = no supported entities (LINE, POLYLINE, 3DFACE, CIRCLE, ARC)
kernel-io-not-a-dxf-file-no-section = not a DXF file (no SECTION)
kernel-io-not-a-dxf-file = not a DXF file

## fbx.rs
kernel-io-fbx-description = FBX: { $description }

## gltf.rs
kernel-io-index-out-of-range = index out of range
kernel-io-no-meshes-in-the-gltf-scene = no meshes in the glTF scene
kernel-io-node-hierarchy-too-deep = node hierarchy too deep
kernel-io-draco-e = Draco: { $e }
kernel-io-draco-primitive-without-bufferview = Draco primitive without bufferView
kernel-io-accessor-out-of-range = accessor out of range
kernel-io-unsupported-component-type = unsupported component type
kernel-io-unsupported-accessor-type = unsupported accessor type
kernel-io-accessor-without-count = accessor without count
kernel-io-bufferview-out-of-range = bufferView out of range
kernel-io-bufferview-without-bytelength = bufferView without byteLength
kernel-io-bad-buffer-index = bad buffer index
kernel-io-buffer-without-data = buffer without data
kernel-io-compressed-gltf-e-is-not-supported = compressed glTF ({ $e }) is not supported
kernel-io-only-gltf-is-supported = only glTF 2.0 is supported
kernel-io-external-file-uri-needs-a-path = external file { $uri } needs a path
kernel-io-bad-base-in-data-uri = bad base64 in data URI
kernel-io-not-a-gltf-file-e = not a glTF file: { $e }
kernel-io-glb-without-json = GLB without JSON
kernel-io-glb-json-e = GLB JSON: { $e }
kernel-io-truncated-glb-chunk = truncated GLB chunk

## meshes.rs
kernel-io-no-faces-in-the-file = no faces in the file

## native.rs
kernel-io-unsupported-version-version = unsupported version { $version }
kernel-io-not-a-varstotas-material-format-format = not a varstotas material (format = { $format })
kernel-io-missing-root-definition = missing root definition
kernel-io-not-a-varstotas-file-format-format = not a varstotas file (format = { $format })

## obj.rs
kernel-io-obj-line-ln-line = OBJ line { $ln }: { $line }

## ply.rs
kernel-io-no-faces-a-point-cloud-can = no faces (a point cloud can't be imported)
kernel-io-list-too-long = list too long
kernel-io-unknown-format-f = unknown format { $f }
kernel-io-property-before-element = property before element
kernel-io-bad-element-count = bad element count
kernel-io-no-end-header = no end_header
kernel-io-not-a-ply-file = not a PLY file
kernel-io-truncated-data = truncated data
kernel-io-unknown-type-s = unknown type { $s }

## points.rs
kernel-io-no-points-in-display = no points in { $display }

## stl.rs
kernel-io-stl-vertex-count-not-a-multiple = STL vertex count not a multiple of 3
kernel-io-bad-stl-vertex-line = bad STL vertex: { $line }
kernel-io-not-a-valid-stl = not a valid STL

## threemf.rs
kernel-io-mf-no-d-model-part = 3MF: no 3D model part
kernel-io-mf-no-object-id = 3MF: no object { $id }
kernel-io-mf-components-nested-too-deep = 3MF: components nested too deep
kernel-io-mf-no-part-path = 3MF: no part { $path }

## xml.rs
kernel-io-xml-no-root-element = XML: no root element
kernel-io-xml-unbalanced-tags = XML: unbalanced tags
kernel-io-xml-e = XML: { $e }

## zip.rs
kernel-io-name-zip-method-m-not-supported = { $name }: zip method { $m } not supported
kernel-io-truncated-zip = truncated zip
kernel-io-bad-zip-entry-name = bad zip entry { $name }
kernel-io-bad-zip-directory = bad zip directory

## Prefixes of the COLLADA / PLY readers
# $msg is the reader's own message
kernel-io-collada = COLLADA: { $msg }
kernel-io-ply = PLY: { $msg }

## IoError
# $why is the system error / the reader's message
kernel-io-error-io = io: { $why }
kernel-io-error-format = format: { $why }
