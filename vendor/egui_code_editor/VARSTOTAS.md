Vendored egui_code_editor 0.4.2 (MIT, https://github.com/p4ymak/egui_code_editor).

Local change: `CursorState` is kept with `get_temp` / `insert_temp` instead of
`get_persisted` / `insert_persisted` (it has no serde derive, so the crate does not compile
when egui's `persistence` feature is on, as in Varstotas). Examples and dev-dependencies are
dropped from Cargo.toml. Drop this copy once upstream fixes it.
