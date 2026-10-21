# Script Reference and editor hints: function, member, tag and type descriptions, section titles.
# The names and signatures (read_text(path)) are code and stay in the sources.

## Functions
script-fn-vec3 = A point / vector (an array [x, y, z] works too).
script-fn-line = Edges from a to b.
script-fn-polyline = Edges through points; closed and planar → a face.
script-fn-face = A face through the points.
script-fn-rect = A rectangular face in the XY plane.
script-fn-circle = A circle face (or edges).
script-fn-polygon = A regular polygon face.
script-fn-arc = Arc edges around center.
script-fn-box = A new box group (origin = min corner).
script-fn-cylinder = A new cylinder group (bottom at center).
script-fn-cone = A new cone group (base at center, apex h above it).
script-fn-pyramid = A new pyramid group: a w × d base from origin, the apex h above its middle.
script-fn-sphere = A new sphere group around center.
script-fn-torus = A new torus group around center: r to the middle of the tube, tube = its radius.
script-fn-pushpull = Push / pull a face; returns the cap face.
script-fn-offset = Offset a face's loop (d < 0 inward); returns the new face.
script-fn-follow-me = Sweep a profile face along a path.
script-fn-erase = Erase an entity or an array of them.
script-fn-move = Move entities by a vector.
script-fn-copy = Copy entities by a vector; returns the copies.
script-fn-rotate = Rotate entities about an axis.
script-fn-scale = Scale entities about a point.
script-fn-group = Group entities; returns the group.
script-fn-component = Make a component; returns its instance.
script-fn-explode = Explode a group / component.
script-fn-make-unique = Give a component instance its own definition.
script-fn-union = Solid union of two solid groups.
script-fn-subtract = Solid a minus b.
script-fn-intersect = Solid intersection.
script-fn-trim = a minus b, b stays.
script-fn-split = Split two solids into three.
script-fn-outer-shell = Outer shell of solids.
script-fn-reverse = Reverse faces.
script-fn-soften = Soften edges up to the fold angle.
script-fn-material = Create / recolour a material; returns its name.
script-fn-paint = Paint faces / groups (() removes).
script-fn-tag = Assign a tag (created when missing).
script-fn-make-parametric = Make a group parametric.
script-fn-set-params = Set a generated component's values, or a parametric group's size / copies.
script-fn-params = Values of a generated component / parameters of a parametric group, or ().
script-fn-reset-params = A generated component back to its generator's defaults.
script-fn-generators = Names of the document's generators.
script-fn-add-generator = Add / update a generator from its source; returns its name.
script-fn-generate = Place a generated component at a point.
script-fn-dimension = A linear dimension between points.
script-fn-note = A text note with a leader.
script-fn-enter = Build inside a group from now on.
script-fn-exit = Leave the group entered last.
script-fn-faces = Faces of the current context.
script-fn-edges = Edges of the current context.
script-fn-groups = Groups / components of the current context.
script-fn-all = Everything in the current context.
script-fn-entities = Everything inside a group.
script-fn-child = The group / component named name in the current context (or inside g); deep = search nested groups too. () when none.
script-fn-children = Groups / components of the current context (or inside g); deep = nested ones too.
script-fn-show = Show (on = false: hide) entities.
script-fn-hide = Hide entities.
script-fn-selection = The selection.
script-fn-select = Set the selection (() clears).
script-fn-bounds = #{ "{" } min, max, size, center { "}" } in world coordinates.
script-fn-model-bounds = Bounds of the whole model.
script-fn-length = Parse a length with units ("1.2m") → mm.
script-fn-units = The model's length unit.
script-fn-materials = Material names.
script-fn-all-faces = Every face, nested groups included.
script-fn-read-text = Read a text file.
script-fn-lang = The UI language code ("en", "ru", …): pick the texts a plugin prints or shows.
script-fn-platform = "macos", "windows" or "linux".
script-fn-write-text = Write a text file.
script-fn-file-exists = A file or folder is there.
script-fn-list-files = Files under dir (subfolders too) with the extension ("" = any), sorted.
script-fn-list-dirs = The folders in dir, sorted.
script-fn-parse-json = JSON text → maps, arrays, numbers, strings.
script-fn-fetch-text = Download a web page / API answer as text (system curl); headers: #{ "{" } "X-Auth-Token": key { "}" }.
script-fn-fetch-file = Download url into the file path.
script-fn-url-encode = Percent-encode text for a URL (a search query).
script-fn-print = Write to the console log.
script-fn-len = Length of an array / string.
script-fn-range = Integers from..to (to excluded), for loops.
script-fn-min = The smaller number.
script-fn-max = The larger number.
script-fn-clamp = x kept within lo..hi.
script-fn-ceil = Round up to a whole number (as a float).

## Sections
script-group-points-and-vectors = Points and vectors
script-group-drawing = Drawing
script-group-solids = Solids
script-group-editing = Editing
script-group-groups-and-components = Groups and components
script-group-finding-objects = Finding objects
script-group-visibility-materials-tags = Visibility, materials, tags
script-group-solid-tools = Solid tools
script-group-parameters-and-generators = Parameters and generators
script-group-dimensions-and-notes = Dimensions and notes
script-group-files-and-network = Files and network
script-group-output-and-numbers = Output and numbers

## Members of entities, vectors, arrays, strings
script-member-kind = "face", "edge", "group", "component" …
script-member-valid = the entity still exists
script-member-area = face area, mm²
script-member-normal = face normal (world)
script-member-edges = edges of the face
script-member-faces = faces of the edge / neighbours of the face
script-member-length = edge length, mm
script-member-soft = the edge is soft
script-member-center = centre of the bounds (world)
script-member-vertices = points of the face / edge (world)
script-member-name = group / component name (assignable)
script-member-hidden = the entity is hidden (assignable)
script-member-bounds = #{ "{" } min, max, size, center { "}" } (world)
script-member-size = size of the bounds along x, y, z
script-member-min = minimum corner of the bounds (world)
script-member-max = maximum corner of the bounds (world)
script-member-definition = component definition name
script-member-material = material name or ()
script-member-tag = tag name or ()
script-member-x = x coordinate (assignable)
script-member-y = y coordinate (assignable)
script-member-z = z coordinate (assignable)
script-member-length-2 = vector length
script-member-normalize = unit vector
script-member-dot = dot product
script-member-cross = cross product
script-member-distance = distance to b
script-member-min-2 = minimum corner
script-member-max-2 = maximum corner
script-member-size-2 = size along x, y, z
script-member-center-2 = centre
script-member-len = number of items
script-member-is-empty = no items
script-member-push = append an item
script-member-pop = remove and return the last item
script-member-insert = insert an item
script-member-remove = remove and return an item
script-member-contains = the array / string contains the value
script-member-index-of = index of the value or -1
script-member-filter = items where the closure is true
script-member-map = a new array of the closure results
script-member-reduce = fold the items
script-member-len-2 = number of characters
script-member-to-upper = upper case (in place)
script-member-to-lower = lower case (in place)
script-member-trim = strip spaces (in place)
script-member-split = split into an array
script-member-starts-with = the string starts with text
script-member-to-string = as text

## Header tags of plugins
script-tag-name = name in the menu and the dialog title
script-tag-menu = path under Extensions
script-tag-description = tooltip and text of the dialog
script-tag-param = a value asked in the dialog before the run
script-tag-pick = N points clicked before the run → points
script-tag-select = objects selected before the run → objects
script-tag-import = a File → Import entry; the file in path
script-tag-export = a File → Export entry; the file in path
script-tag-generator = generator: makes this file a generator (see docs/GENERATORS.md)
script-tag-formula = generator: a value computed from the parameters, in p
script-tag-check = generator: values for which it is false are refused

## Parameter types (@param key:type)
script-param-type-length = length with units ("1.2m"), mm in the script — the default for numbers
script-param-type-int = whole number
script-param-type-number = fractional number
script-param-type-text = text — the default for a quoted value
script-param-type-bool = check box — the default for true / false
script-param-type-choice = generator: one of choice(a|b|c)
script-param-type-angle = generator: degrees, a slider
script-param-type-percent = generator: 0–100, a slider
script-param-type-material = generator: a material name of the document

script-type-face = face
script-type-edge = edge
script-type-group = group
script-type-vector = vector
script-type-num = number
script-type-bool = bool
script-type-str = string
script-type-bounds = bounds
script-type-faces = array of faces
script-type-edges = array of edges
script-type-groups = array of groups
script-type-vecs = array of points
script-type-array = array
script-type-any = entity / vector

## Parameter types, without the "generator: " lead (Script Reference, generator topic)
script-param-type-choice-plain = one of choice(a|b|c)
script-param-type-angle-plain = degrees, a slider
script-param-type-percent-plain = 0–100, a slider
script-param-type-material-plain = a material name of the document

# Errors a script user sees (API, generators, parameter rigs)

## api.rs
script-err-k-expected-a-number-bool-or = { $k }: expected a number, bool or string, got { $type_name }
script-err-set-params-unknown-key-k-x = set_params: unknown key { $k } (x, y, z, count, axis, gap, stretch)
script-err-set-params-stretch-is-true-false = set_params: stretch is true / false
script-err-set-params-not-parametric-call-make = set_params: not parametric (call make_parametric first)
script-err-paint-no-material-n-create-it = paint: no material { $n } (create it with material(name, colour))
script-err-paint-expected-a-material-name-got = paint: expected a material name, got { $t }
script-err-show-hide-groups-and-components-only = show / hide: groups and components only
script-err-outer-shell-solids-must-be-in = outer_shell: solids must be in one context
script-err-group-entities-must-be-in-one = group: entities must be in one context
script-err-scale-zero-factor = scale: zero factor
script-err-rotate-zero-axis = rotate: zero axis
script-err-follow-me-the-path-must-be = follow_me: the path must be in the face's context
script-err-note-not-added = note: not added
script-err-dimension-not-added = dimension: not added
script-err-what-sizes-must-be-positive-a = { $what }: sizes must be positive (a torus tube thinner than its radius), segments 3..1000
script-err-cylinder-no-face = cylinder: no face
script-err-cylinder-radius-and-height-must-be = cylinder: radius and height must be non-zero, segments 3..1000
script-err-box-sizes-must-be-non-zero = box: sizes must be non-zero
script-err-circle-radius-must-be-positive-segments = circle: radius must be positive, segments 3..1000
script-err-face-points-are-not-a-valid = face: points are not a valid planar polygon
script-err-soft-not-an-edge = soft: not an edge
script-err-faces-not-an-edge-or-face = faces: not an edge or face
script-err-length-not-an-edge = length: not an edge
script-err-time-limit-of-secs-s-exceeded = time limit of { $secs }s exceeded
script-err-both-solids-must-be-in-the = both solids must be in the same context
script-err-expected-a-colour-rrggbb-or-r = expected a colour: "#RRGGBB" or [r, g, b]
script-err-bad-colour-s-use-rrggbb = bad colour { $s } (use #RRGGBB)
script-err-bad-colour-s = bad colour { $s }
script-err-curve-not-added = curve not added
script-err-degenerate-curve = degenerate curve
script-err-expected-edges-got-e = expected edges, got { $e }
script-err-expected-a-group-or-component-got = expected a group or component, got { $e }
script-err-expected-a-face-got-e = expected a face, got { $e }
script-err-entity-no-longer-exists-merged-split = entity no longer exists (merged, split or erased)
script-err-context-no-longer-exists = context no longer exists
script-err-expected-one-entity-got-len = expected one entity, got { $len }
script-err-expected-an-entity-or-an-array = expected an entity or an array of entities, got { $type_name }
script-err-expected-an-entity-got-did-the = expected an entity, got () (did the operation fail?)
script-err-no-group-component-named-name-here = no group / component named `{ $name }` here
script-err-expected-an-array-of-points = expected an array of points
script-err-expected-a-point-vec-or-x = expected a point (vec3 or [x, y, z]), got { $type_name }
script-err-expected-a-number-got-type-name = expected a number, got { $type_name }
script-err-not-a-length-s = not a length: { $s }
script-err-no-model = no model

## generator.rs
script-err-no-such-instance = no such instance
script-err-not-a-generated-component = not a generated component
script-err-no-such-context = no such context
script-err-no-generator-name-in-the-document = no generator `{ $name }` in the document
script-err-name-generators-nested-too-deep = { $name }: generators nested too deep
script-err-name-check-expr-is-not-true = { $name }: check `{ $expr }` is not true / false
script-err-expected-a-number-got-t = expected a number, got { $t }

## json.rs
script-err-bad-entity-id-s = bad entity id { $s }

## rig.rs
script-err-part-has-no-parameter-key = `{ $part }` has no parameter `{ $key }`
script-err-part-has-no-parameters = `{ $part }` has no parameters
script-err-no-part-part-in-the-component = no part `{ $part }` in the component
script-err-part-no-parameter-k-it-has = { $part }: no parameter `{ $k }` (it has { $keys })
script-err-part-the-part-has-no-parameters = { $part }: the part has no parameters (only generated components and components with parameters have)
script-err-part-show-hide-needs-a-yes = { $part }: show / hide needs a yes / no or number parameter
script-err-part-choose-the-part-s-parameter = { $part }: choose the part's parameter
script-err-part-the-direction-is-zero = { $part }: the direction is zero
script-err-part-the-axis-has-no-direction = { $part }: the axis has no direction
script-err-part-choose-a-parameter-or-write = { $part }: choose a parameter or write an expression
script-err-a-joint-without-a-part = a joint without a part
script-err-k-twice-parameters-and-formulas-share = `{ $k }` twice (parameters and formulas share names)
script-err-no-such-component = no such component
script-err-this-component-is-built-by-a = this component is built by a generator or a parametric group
script-err-groups-can-t-have-parameters-make = groups can't have parameters: make it a component first
script-err-not-a-component-with-parameters = not a component with parameters
script-err-part-the-part-has-no-parameters-2 = { $part }: the part has no parameters
script-err-part-no-such-part = { $part }: no such part
script-err-what-the-value-is-not-a = { $what }: the value is not a number, flag or text
script-err-what-the-value-is-not-a-2 = { $what }: the value is not a material name
script-err-what-the-value-is-not-true = { $what }: the value is not true / false
script-err-what-the-value-is-not-a-3 = { $what }: the value is not a number
