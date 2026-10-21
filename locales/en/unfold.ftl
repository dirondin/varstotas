unfold-paper-fit = Fit to pieces
unfold-paper-a4 = A4
unfold-paper-a3 = A3
unfold-paper-a2 = A2

## Unfold window and export

unfold-title = Unfold
unfold-selection = Selection
unfold-everything = Everything in the context
unfold-nothing = Unfold: nothing to unfold (select faces or groups)
# $name is the document name, $scale like "1:10".
unfold-footer = { $name } - unfold { $scale }
# $n is the sheet number out of $total.
unfold-footer-sheet = { $name } - unfold { $scale } - sheet { $n } of { $total }
# $oversize is the number of pieces larger than the sheet (0 = none).
unfold-done = Unfolded { $faces } faces into { $pieces } pieces on { $sheets } sheets{ $oversize ->
        [0] {""}
       *[other] {" "}({ $oversize } larger than the sheet)
    }: { $path }
unfold-export-failed = Export failed: { $error }
# $what is "Selection" or "Everything in the context".
unfold-no-faces = { $what }: no faces. Select faces, groups or components.
unfold-summary = { $what }: { $faces } faces → { $pieces } pieces on { $sheets } sheet(s)
unfold-oversize = { $n } piece(s) larger than the sheet: choose a larger paper or scale.
unfold-scale = Scale 1:
unfold-paper = Paper
unfold-landscape = Landscape
unfold-glue-tabs = Glue tabs
unfold-edge-numbers = Edge numbers
unfold-soft-folds = Folds on soft edges
unfold-sheet-of = Sheet { $n } of { $total }
unfold-legend-cut = ━ cut
unfold-legend-valley = ╍ valley
unfold-legend-mountain = ━·━ mountain
unfold-export = Export:
