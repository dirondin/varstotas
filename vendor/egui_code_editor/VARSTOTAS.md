Vendored egui_code_editor 0.4.2 (MIT, https://github.com/p4ymak/egui_code_editor).

Local change: `CursorState` is kept with `get_temp` / `insert_temp` instead of
`get_persisted` / `insert_persisted` (it has no serde derive, so the crate does not compile
when egui's `persistence` feature is on, as in Varstotas). Examples and dev-dependencies are
dropped from Cargo.toml. Drop this copy once upstream fixes it.

Varstotas addition: `CodeEditor::with_doc_tags(true)` highlights `// @tag args` comment lines
(plugin / script headers): the tag like a keyword, then strings, numbers, `true` / `false`,
`:type` and names (`append_doc_tags` in `src/lib.rs`). `CodeEditor::with_marked_words(words, color)`
draws those identifiers in their own colour.
