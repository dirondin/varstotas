# Kernel errors and messages the UI shows (operations, curves, generators, parameters).
# Arguments are named in the comment above a message when not obvious.

## curves.rs
kernel-err-the-curve-bounds-faces-outside-its = the curve bounds faces outside its plane
kernel-err-other-edges-are-attached-to-the = other edges are attached to the curve
kernel-err-too-few-points = too few points
kernel-err-segments-must-be-min-max-curve = segments must be { $min }..{ $MAX_CURVE_SEGMENTS }
kernel-err-radius-must-be-positive = radius must be positive
kernel-err-not-an-arc-circle-or-polygon = not an arc, circle or polygon

## chamfer.rs
kernel-err-the-edges-at-a-corner-are = the edges at a corner are collinear
kernel-err-a-corner-of-the-edge-is = a corner of the edge is not simple (too many edges or faces)
kernel-err-the-chamfer-is-too-big-for = the chamfer is too big for the edge
kernel-err-the-chamfer-is-too-big-for-2 = the chamfer is too big for the neighbouring edges
kernel-err-an-end-lies-along-the-edge = an end lies along the edge
kernel-err-the-radius-must-be-positive = the radius must be positive
kernel-err-the-distance-must-be-positive = the distance must be positive
kernel-err-the-faces-are-coplanar = the faces are coplanar
kernel-err-broken-face-loop = broken face loop
kernel-err-the-edge-is-too-short = the edge is too short
kernel-err-the-edge-must-be-shared-by = the edge must be shared by exactly two faces

## follow_me.rs
kernel-err-path-reverses = path reverses
kernel-err-profile-is-parallel-to-the-path = profile is parallel to the path
kernel-err-path-is-not-connected = path is not connected
kernel-err-path-branches = path branches
kernel-err-empty-path = empty path

## mod.rs
kernel-err-nothing-to-flip = nothing to flip
kernel-err-flip-axis-is-zero = flip axis is zero

## push_pull.rs
kernel-err-through-hole-cap-vertex-missing = through-hole: cap vertex missing
kernel-err-push-pull-would-collapse-geometry = push/pull would collapse geometry
kernel-err-face-loop-without-edge = face loop without edge
kernel-err-distance-is-not-finite = distance is not finite

## params.rs
kernel-err-empty-base = empty base
kernel-err-size-must-be-positive = size must be positive
kernel-err-not-parametric = not parametric
kernel-err-empty-group = empty group
kernel-err-already-parametric = already parametric

## spline.rs
kernel-err-the-curve-would-cross-itself = the curve would cross itself
kernel-err-segments-must-be-max-curve-segments = segments must be 1..{ $MAX_CURVE_SEGMENTS }
kernel-err-not-a-curve-that-can-be = not a curve that can be edited

## stats.rs
kernel-err-an-empty-name = an empty name

## text3d.rs
kernel-err-no-outlines = no outlines

## generator.rs
kernel-gen-name-no-parameter-k-join = { $name }: no parameter `{ $k }` ({ $join })
kernel-gen-empty-generator-name = empty @generator name
kernel-gen-not-a-generator-the-header-needs = not a generator: the header needs `// @generator Name`
kernel-gen-key-twice-formulas-and-parameters-share = `{ $key }` twice (formulas and parameters share names)
kernel-gen-key-bool-default-is-true-false = { $key }: bool default is true / false
kernel-gen-unknown-parameter-type-t-length-int = unknown parameter type `{ $t }` (length, int, number, bool, choice, angle, percent, material)
kernel-gen-key-choice-needs-options-choice-a = { $key }: choice needs options: choice(a|b|c)
kernel-gen-bad-parameter-name-key = bad parameter name `{ $key }`
kernel-gen-check-failed-expr = check failed: { $expr }
kernel-gen-key-empty-formula = { $key }: empty formula
kernel-gen-key-a-formula-is-length-int = { $key }: a formula is length, int, number, bool, angle, percent or text, not `{ $t }`
kernel-gen-bad-formula-name-key = bad formula name `{ $key }`
kernel-gen-key-bad-number-s-in-the = { $key }: bad number `{ $s }` in the range
kernel-gen-key-not-a-name-s = { $key }: not a { $name }: { $s }
kernel-gen-key-s-is-not-one-of = { $key }: `{ $s }` is not one of { $join }

## model.rs
kernel-err-the-generator-s-script-can-t = the generator's script can't be copied
kernel-err-nothing-selected = nothing selected

## solid.rs
kernel-err-pick-at-least-two-solids = pick at least two solids
kernel-err-the-second-solid-has-a-degenerate = the second solid has a degenerate transform
kernel-err-the-solids-do-not-overlap = the solids do not overlap
kernel-err-pick-two-different-solids = pick two different solids
# $name is the group / component name
kernel-err-is-not-a-solid-every-edge = “{ $name }” is not a solid (every edge must bound exactly two faces, no nested groups)

## OpError
kernel-err-entity-not-found = entity not found
kernel-err-degenerate-geometry = degenerate geometry
# $why is the reason
kernel-err-operation-not-possible = operation not possible: { $why }
