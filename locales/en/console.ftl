# Script Console, Script Reference window and code editor.

## Script Console
console-title = Script Console
console-script-failed = script failed
# $error is the script's error text
console-script-error = Script error: { $error }
# $name is the script file name; this is the undo step name
console-script-undo = Script { $name }
# $path is a file path; $error is the system's message
console-cannot-read = Cannot read { $path }: { $error }
console-hint = Rhai scripts over the model. Try: box(ORIGIN, 1000, 600, 18)  — %CMD%+Enter runs
console-run = Run
console-run-hint = %CMD%+Enter
console-open = Open…
console-save = Save…
console-stop-python = Stop Python
console-clear = Clear Output
# $port is a TCP port number
console-bridge = Python / MCP bridge: 127.0.0.1:{ $port }
console-bridge-hint = import varstotas; m = varstotas.connect()  — see ~/.varstotas/python
console-filter-rhai = Rhai script

## Script Reference window
console-ref-title = Script Reference
console-search = Search
console-search-hint = function, property or word
console-fulltext = Full text: docs/SCRIPTING.md, docs/GENERATORS.md
console-copy-example = Copy the example
console-click-copy = Click to copy
console-topic-basics = Basics
console-topic-console = Console and plugins
console-topic-generators = Generators
console-topic-component = Component script
console-topic-functions = Functions
console-topic-properties = Properties
console-param-types = Parameter types
console-generators-note = Inside `build` the whole API works (see Functions). The script runs in an empty model holding the document's materials and tags; everything it builds becomes the component's content.
console-functions-intro = Every function of the API, by section. Click a signature to copy it. `x` is an object, an array of objects or a name of a child group.
console-properties-intro = After a dot: `g.name`, `f.normal.x`, `door.size.z`, `[1, 2].len()`. Properties marked assignable can be set: `g.name = "Crate"`, `g.hidden = true`.
console-kind-objects = Objects (faces, edges, groups, components)
console-kind-vectors = Vectors
console-kind-bounds = Bounds — bounds(x), x.bounds
console-kind-arrays = Arrays
console-kind-strings = Strings
console-nothing-found = Nothing found
console-members = Properties and methods
console-tags = Header tags
console-reference = ? Reference
console-reference-hint = Help → Script Reference: functions, properties, examples (F1 in the editor: the word at the cursor)


## Page: basics
console-ref-basics-1 = Scripts are written in Rhai — a small language with Rust / JavaScript-like syntax. Coordinates are world millimetres; lengths with units are strings: `length("1.2m")`.
console-ref-basics-2 = Language
console-ref-basics-4 = Strings: `"text"`, joined with `+`: `"w = " + w`. String methods `trim()`, `to_upper()` change the string in place and return nothing: `s.trim(); if s == "" {"{"} … {"}"}`.
console-ref-basics-5 = Constants: `ORIGIN`, `X`, `Y`, `Z` (unit axes). Vectors add, subtract and scale: `ORIGIN + X * 100`; `.x .y .z`, `length()`, `normalize()`, `dot(b)`, `cross(b)`, `distance(b)`.
console-ref-basics-6 = Objects
console-ref-basics-7 = Faces, edges, groups and components are references: `let g = box(ORIGIN, 100, 100, 100); g.name = "Crate";`. After operations that merge or cut geometry an old reference may become invalid — check `e.valid`.
console-ref-basics-8 = Anywhere an object is expected, a string names a group / component of the current context: `move("Door", vec3(0, 10, 0))`. `child("Knob", true)` searches nested groups too.
console-ref-basics-9 = Every object has `kind`, `bounds`, `size`, `min`, `max`, `center`, `hidden`; groups also `name`, `definition`, `material`, `tag` (see Properties).
console-ref-basics-10 = Context and undo
console-ref-basics-11 = New geometry goes into the current context: the group open when the script started, or the one entered with `enter(g)` (leave with `exit()`).
console-ref-basics-12 = A run is one undo step. A script that fails changes nothing; the error with its line is shown — and kept in Window → Log.
console-ref-basics-13 = `print(x)` writes to the output; the value of the last expression is printed too. Scripts stop after 30 s (generators 10 s).
## Page: console
console-ref-console-1 = Window → Script Console runs code against the open model. %CMD%+Enter runs, %CMD%+↑ / ↓ goes through earlier runs, Open… / Save… work with `.rhai` files. File → Run Script… runs a file.
console-ref-console-2 = Completion shows API functions while typing, properties after a dot (by the type on the left), and the signature of the function being called above the cursor. F1 opens this reference on the word at the cursor.
console-ref-console-3 = Header: dialogs and menu items
console-ref-console-4 = Lines `// @tag …` at the top turn a script into a plugin: Run first asks the parameters, points and objects. A file in `~/.varstotas/plugins` becomes an Extensions menu item.
console-ref-console-6 = Tags
## Page: generators
console-ref-generators-1 = A generator is a component with parameters: a script whose `build(p)` builds the component's content in its own axes (the origin is the insertion point). Each copy has its own values in Entity Info; copies with equal values share one definition.
console-ref-generators-2 = Window → Generator Editor… (or Edit Generator… on a tile): the source on the left, a live preview with parameter fields on the right; Apply updates the document, Save to Library writes the file.
console-ref-generators-4 = Header
console-ref-generators-5 = required; the generator's key in the document
console-ref-generators-6 = a value in `p`; every part of the range is optional
console-ref-generators-7 = a value computed from the parameters (read-only in Entity Info)
console-ref-generators-8 = values for which it is false are refused
console-ref-generators-9 = Parameter types
## Page: component
console-ref-component-1 = Window → Component Parameters… gives any component parameters: rows of values, formulas, motions of its parts (turn, slide, show, paint, set a part's parameter) and an optional script.
console-ref-component-2 = The script defines `update(p)`. It runs after the motions, inside the component, with the whole script API — the same functions as the console and generators. `p` holds the values and the formulas.
console-ref-component-4 = Finding parts
console-ref-component-5 = a group / component directly inside the component
console-ref-component-6 = searched in nested groups too (the nearest one wins)
console-ref-component-7 = inside a part
console-ref-component-8 = all parts (nested ones too)
console-ref-component-9 = the part's extents now (world)
console-ref-component-10 = hide, like hide(part)
console-ref-component-11 = How it works
console-ref-component-12 = The script changes a copy (the variant for these values), never the master: parts it changes are copied into the variant, parts it leaves alone stay shared. Copies with equal values share one variant.
console-ref-component-13 = Edits made inside a copy (double click) go back to the master without what the script did: what it added is dropped, moves / hiding / paint are undone.
console-ref-component-14 = Errors are shown when Apply is pressed, and kept in Window → Log.

## Code editor
# $sel is the selected completion's number, $n the number of completions
console-completion-count = { $sel } of { $n }
