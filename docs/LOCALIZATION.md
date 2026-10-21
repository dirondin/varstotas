# Localization

Every text the user reads comes from one file per language, `locales/<lang>.ftl`, in the
[Fluent](https://projectfluent.org/fluent/guide/) format; the code only holds keys. English
(`locales/en.ftl`) is the base: every key exists there, other languages fall back to it key by key.
The crate `varstotas_i18n` loads the files (embedded at build time) and looks texts up.

## In the code

```rust
use varstotas_i18n::tr;

ui.button(tr!("cancel"));
ui.label(tr!("library-items", count = items.len()));          // arguments: strings or numbers
doc.try_commit(&tr!("tool-slice-undo"), |m| …);                // undo names too
let what = tr!(if on { "tool-x-on" } else { "tool-x-off" });   // a computed key is fine
```

- `tr!` returns a `String`. A function that used to return `&'static str` for a display text
  returns `String` (`varstotas_i18n::tr(match self { A => "area-a", … })`).
- Keys are kebab case: `<area>-<thing>[-<detail>]`; the area is the first word (`tool-…`,
  `cmd-…`, `prefs-…`) and the section of the file where the key lives. Sections are comments:
  `## ======== Tools ========`, with `## …` subsections inside.
- Never build a sentence from pieces in the code: pass the varying parts as arguments, so a
  translation can reorder them. Numbers that choose a word form go as numbers
  (`count = n`), not preformatted.
- Lengths, angles and other values are formatted by the code (units, precision) and passed as
  strings.
- Display texts are not identifiers: preference values, QA spot names (`tray_lib::spot`),
  saved card names and the like stay fixed English ids; the label shown next to them is
  translated.
- Not translated: log messages (`log::…`), the bridge / MCP protocol and its errors, QA
  commands, GPU labels, file format names inside files, and document content (demo scenes,
  example materials, model names).

## Platform variants

A message attribute named after the platform replaces the value there (`macos`, `windows`,
`linux`); a `PLATFORM()` selector does the same inside a pattern:

```ftl
reveal-in-file-manager = Show in File Manager
    .macos = Reveal in Finder
    .windows = Show in Explorer

trash-hint = Deleted files go to the { PLATFORM() ->
        [macos] Trash
        [windows] Recycle Bin
       *[other] trash
    }.
```

Modifier names are markers, written the same in every language: `%COPY%` (Option on macOS,
Ctrl elsewhere), `%CMD%` (Cmd / Ctrl), `%ALT%` (Option / Alt), `%CMDK%` (`⌘` / `Ctrl+`, a
shortcut prefix, as in `%CMDK%S`).

## Plural forms and arguments

```ftl
library-items = { $count ->
        [one] { $count } item
       *[other] { $count } items
    }
```

Russian uses `[one]`, `[few]`, `[many]`, `*[other]`. A comment line `#` above a message tells
translators what its arguments are.

## Translators

- A language is a file `locales/<code>.ftl` (ISO 639-1: `ru`, `de`, …) with the keys of
  `locales/en.ftl` in the same sections, and a `language-name` message (its own name:
  `Deutsch`). Keys missing in a translation show in English.
- Without rebuilding: put `<code>.ftl` into `~/.varstotas/locales/`. It adds a language
  (it appears in Preferences → Interface → Language) or override single messages of a built-in
  one. Syntax errors go to Window → Log.
- `cargo test -p varstotas_i18n` checks the syntax of every file, that translations hold no keys
  English lacks, and that every key the sources use exists.
- `scripts/i18n_literals.py [paths]` lists literals that still look like UI text;
  `// i18n-ignore` on a line (or `// i18n-ignore-next` above it) marks a deliberate one.

## Plugins

A plugin's header texts get localized variants by a tag suffix — a language code, a platform
name, or both:

```rhai
// @name Stairs
// @name.ru Лестница
// @menu Construction/Stairs…
// @menu.ru Построение/Лестница…
// @description A straight flight of treads.
// @description.ru Прямой марш ступеней.
// @param width = 900 "Width"
// @param.ru width "Ширина"
// @pick 1 "Click the bottom front corner"
// @pick.ru "Щёлкните нижний передний угол"
// @menu.macos …                     (a platform variant)
```

`@name`, `@menu`, `@description`, `@param` (labels, by key), `@pick`, `@select` and `@notice`
take suffixes; the most specific variant for the UI language and platform wins, the plain tag
is the fallback. Plugins are re-read when the language changes. Texts a script prints or shows
can follow `lang()` (Rhai) / `varstotas.lang()` (Python) — `"en"`, `"ru"`, … — and `platform()`
(Rhai: `"macos"`, `"windows"`, `"linux"`).
