# 3D Printing

Open **Window → Check Model…**. This single entry opens the resizable 3D printing preparation window
for the complete planned workflow: **Check, Repair, Analysis, Printer & bed, Split & connectors, Print export**. Commands
are also available in shortcut/toolbar settings. Tools inspect the common model surface, so
these operations apply to native geometry and every imported format.

The check controls (tolerance, issue visibility, Run Check and Copy Report) belong to the
**Check** tab. **Analysis** and **Printer & bed** have their own refresh actions; the other tabs
show only their own tools.

## Check and repair

Choose **Visible model** or **Selection**. Individual faces can be inspected; mutations work
on whole parts. Check reports world-space millimetres, area, signed shell volume, open and
non-manifold edges, winding, duplicate/degenerate triangles, inverted shells/cavities and loose
edges. Clicking a category isolates its viewport highlight.

Repair welds coincident vertices, removes duplicate/degenerate faces and corrects winding and
shell orientation. Closing small planar holes is opt-in (initial maximum bounding-box diagonal:
10 mm). Ambiguous/nested boundaries and flat sheets are left for manual repair. Locks, hidden
geometry, procedural ancestors, shared copies and valid corner shading are protected. Repair
is one Undo step; a no-op does not dirty the document.

## Intersections, thickness and overhangs

**Analysis** adds BVH-accelerated triangle intersection detection, including coplanar overlap.
Shared edges and mere vertex contact are excluded; actual crossings at shared vertices are
still found. The configurable candidate budget is explicit: reaching it marks the result
incomplete. Intersections are highlighted in magenta.

Thickness casts inward rays at up to four barycentric positions per triangle, distributed over
the model under a configurable sample budget. It reports coverage, the minimum measured value,
below-threshold samples and unknown exits. The threshold can be set to twice the configured
nozzle diameter. Wall heatmaps use red below threshold, green above and grey when no sample on
the triangle found a valid exit. Unsampled triangles retain their normal appearance. This is
normal-ray sampling, not an exact global minimum-distance or medial-axis calculation: tiny
features between samples can be missed. Open, inconsistent or intersecting surfaces are
explicitly marked unreliable.

Downward faces under the configurable angle from downward vertical are highlighted orange;
contact at Z=0 is excluded. This is a geometric overhang estimate, not support generation.

## Printer profile and placement

A persistent printer profile stores X/Y/Z build dimensions, edge margin, part spacing, nozzle
size and permission to rotate parts by 90°. The viewport can show the build volume, and
**Frame print volume** fits it in view. The report counts parts outside the usable volume,
parts off the bed and potential bounding-box overlaps.

- **Drop to bed** moves each part to Z=0.
- **Center on bed** centres the combined scope in X/Y without changing scale.
- **Arrange parts** uses a shelf arrangement of bounding boxes, with spacing/margins and
  optional 90° rotation. Failure leaves the document unchanged. It uses one bed and can reject
  irregular shapes that would fit with manual packing.
- **Lay selected face on bed** rotates the enclosing part so that its selected outward face
  points down, then drops and centres it. Recessed faces that would put material below the
  bed are rejected. This leaves the part-editing context and selects the moved whole part.

Placement changes transforms rather than dimensions. Locks/procedural objects are respected,
and every accepted mutation is undoable. Hidden faces mixed with loose root geometry require
manual grouping before placement.

## Split and connectors

Select one whole group/component. Set an X/Y/Z or custom plane normal and its world-space
signed offset. A translucent plane previews the cut. **Cut and cap** clips triangles with
interpolated UVs and normals and closes the cut, including holes through hollow parts. Source
materials are retained. Closed, consistently oriented, duplicate-free input is required;
intersection checking must complete without crossings. Invalid cuts leave the source intact.

Optional connectors produce sockets in both halves plus separate cylindrical registration
dowels. Count, diameter, length, radial/axial clearance and surrounding wall are configurable.
Candidates are spread along the longer extent of the cut. Radial/depth probes reject positions
without sufficient material; overlapping sockets are rejected. Clearance still requires
calibration for the printer/material. This is automatic placement on a single cut, not a
manual arbitrary connector editor.

**Split to printer dimensions** cuts the selected part into an axis-aligned grid sized to the
usable printer volume (up to 256 parts). It caps the resulting sections and creates no
connectors. Both split modes keep assembly coordinates: arrange the resulting parts afterward,
or select subsets for separate beds. Coincident caps at assembly positions may be reported as
coincident surfaces until the halves are separated.

## Export and slicer

**Print export** uses the same Visible/Selection scope. It preserves world coordinates and
millimetres, rather than resetting each part to the origin. 3MF retains separate objects,
base colours, names, model description, licence and source credits. It does not include PBR
textures. STL is triangle geometry only. Export writes a snapshot atomically and does not
modify the native document. The existing File exporters remain available separately.

3MF Core metadata follows the standard keys `Title`, `Description` and `LicenseTerms`
([Core specification §3.4.1](https://github.com/3MFConsortium/spec_core/blob/master/3MF%20Core%20Specification.md)).
3MF vertices retain full coordinate precision; capped tessellation preserves collinear contour
subdivisions to avoid exported T-junctions.

Export, then **Open in Slicer**: the file opens in the system's default app for `.3mf` / `.stl`
(make the slicer the default for them). Varstotas runs no program path of its own, so the same
works in a sandbox (Flatpak, through the OpenURI portal). Model, selection
or analysis/profile changes invalidate the handoff until a fresh export. The slicer is never
launched automatically. Layer inspection, toolpaths and G-code remain the slicer's job.

## Background work and validation

Check/analysis, repair, cutting and export run on snapshots. Document identity, model version,
scope/selection, tolerance and analysis/profile settings must match before results are applied.
Stale results cannot replace newer edits. Old overlays are hidden until rechecked. Exporting a
snapshot that becomes stale may finish writing its file, but cannot enable the current-model
slicer handoff.

Kernel regressions cover world scaling, sample coverage, overhang bed contact, shared-vertex
crossings, budgets, material/scope export, 3MF XML/precision, bed placement and rollback, selected
face orientation, capped diagonal/cavity cuts, sockets/dowels, shared nested instances, locks,
no-op placement and automatic bed-sized cuts. The pre-existing repair tests cover hidden and
procedural geometry, shared copies, winding/cavities and conservative hole closing.

Native scripted QA: `tests/ui/scripts/printing.txt`, `printing_holes.txt`, `printing_all.txt` and
`printing_autocut.txt`. Screenshots and logs are under `target/ui-tests/shots/printing-*` and
`target/printing-*-native.log`. Test and build results are recorded in `target/printing-all-workspace.log`
and `target/printing-all-build.log`. The local signed debug bundle is `target/Varstotas.app`.
The user's production NASA/Sofa library assets are not rewritten by printing QA.

Final validation: **636 workspace tests passed, 2 ignored**. Eleven extended printing kernel
cases and the literal slicer-argument regression passed (that regression was later retired with
the configured slicer path). Clippy completed with the existing
warnings and no new printing warnings. The complete Russian native workflow passed 9 checks;
automatic bed splitting passed 3 checks. A controlled fake slicer received exactly one file
argument and was not launched again after Undo invalidated the export (now the harness records
the file handed to the default app instead). The exported four-object
3MF was independently parsed as XML: every object's triangle edges were paired, with millimetre
units. English and Russian screenshots were inspected.

A read-only analysis of the actual 88,174-face NASA asset also completed: 20,000 thickness
samples, 6,953 unknown exits and 1,245 below-threshold samples. Its intersection check correctly
stopped at the configured candidate budget and reported incomplete coverage; it did not claim
the defective overlapping source was ready to print. Native document dirty state remained false
(`target/printing-nasa-all-native.log`).
