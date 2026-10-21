# Window → Log and the status bar's log button.

log-level-error = error
log-level-warning = warning
log-title = Log
# $n is a count
log-errors = Errors { $n }
log-warnings = Warnings { $n }
log-filter = filter
log-clear = Clear
log-copy = Copy
log-copy-hint = Copy the entries shown
log-empty = No errors or warnings in this session.
log-no-match = Nothing matches the filters.
# $n is the number of errors not seen yet
log-badge-errors = ● { $n ->
        [one] { $n } error
       *[other] { $n } errors
    }
log-badge = Log
log-badge-hint = Window → Log: errors and warnings from everywhere (scripts, import / export, libraries …)
