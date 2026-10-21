//! Localization of everything the user reads: Fluent messages (`locales/<lang>.ftl`) looked
//! up by key. See docs/LOCALIZATION.md.
//!
//! - `tr!("menu-file-open")`, `tr!("status-faces", count = n)` — the text in the current
//!   language; a key missing there falls back to English, a key missing everywhere comes back as
//!   the key itself (and is remembered in [`missing_keys`]).
//! - Platform variants: a message's attribute named after the platform (`.macos`, `.windows`,
//!   `.linux`) replaces its value there; inside a pattern `{ PLATFORM() -> [macos] … *[other] … }`
//!   selects per platform too.
//! - The locales of the repository are embedded at build time; `.ftl` files in the user's
//!   folder (`~/.varstotas/locales/<lang>.ftl`) add languages or override single messages.
//! - Modifier markers in texts are named for the platform: `%COPY%` (Option on macOS, Ctrl
//!   elsewhere), `%CMD%` (Cmd / Ctrl), `%ALT%` (Option / Alt), `%CMDK%` (`⌘` / `Ctrl+`, a
//!   shortcut prefix) — see [`platform_keys`].
//! - Plugins localize their header texts with suffixed tags, picked by [`pick_variant`].

use fluent_bundle::concurrent::FluentBundle;
use fluent_bundle::{FluentArgs, FluentResource};
use std::borrow::Cow;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::{LazyLock, Mutex};
use unic_langid::LanguageIdentifier;

pub use fluent_bundle::FluentValue;

mod embedded {
    include!(concat!(env!("OUT_DIR"), "/embedded.rs"));
}

/// The language every key exists in; the fallback of the others.
pub const BASE: &str = "en";

/// The platform's name as used by message attributes and `PLATFORM()`.
pub fn platform() -> &'static str {
    if cfg!(target_os = "macos") {
        "macos"
    } else if cfg!(target_os = "windows") {
        "windows"
    } else {
        "linux"
    }
}

fn key_name(marker: &str, mac: bool) -> &'static str {
    match (marker, mac) {
        ("COPY", true) | ("ALT", true) => "Option",
        ("COPY", false) | ("CMD", false) => "Ctrl",
        ("CMD", true) => "Cmd",
        ("ALT", false) => "Alt",
        ("CMDK", true) => "⌘",
        ("CMDK", false) => "Ctrl+",
        _ => "",
    }
}

/// `s` with its modifier markers (`%COPY%`, `%CMD%`, `%ALT%`, `%CMDK%`) named for this platform.
pub fn platform_keys(s: &str) -> Cow<'_, str> {
    platform_keys_for(s, cfg!(target_os = "macos"))
}

fn platform_keys_for(s: &str, mac: bool) -> Cow<'_, str> {
    if !s.contains('%') {
        return Cow::Borrowed(s);
    }
    let mut out = s.to_string();
    for m in ["CMDK", "COPY", "CMD", "ALT"] {
        out = out.replace(&format!("%{m}%"), key_name(m, mac));
    }
    Cow::Owned(out)
}

type Bundle = FluentBundle<FluentResource>;

struct State {
    lang: String,
    /// The current language first, then [`BASE`] (when different).
    bundles: Vec<Bundle>,
    cache: HashMap<String, String>,
    missing: HashSet<String>,
    errors: Vec<String>,
    user_dir: Option<PathBuf>,
    generation: u64,
    /// Bundles of other languages asked for by [`with_language`] (tests).
    others: HashMap<String, Vec<Bundle>>,
}

thread_local! {
    /// [`with_language`]'s language on this thread.
    static OVERRIDE: std::cell::RefCell<Option<String>> = const { std::cell::RefCell::new(None) };
}

fn override_lang() -> Option<String> {
    OVERRIDE.with(|o| o.borrow().clone())
}

/// Runs `f` with the texts of `lang` on this thread only (tests: other threads keep theirs).
pub fn with_language<R>(lang: &str, f: impl FnOnce() -> R) -> R {
    let prev = OVERRIDE.with(|o| o.replace(Some(lang.to_string())));
    let r = f();
    OVERRIDE.with(|o| *o.borrow_mut() = prev);
    r
}

static STATE: LazyLock<Mutex<State>> = LazyLock::new(|| {
    let mut s = State { lang: String::new(), bundles: Vec::new(), cache: HashMap::new(), missing: HashSet::new(), errors: Vec::new(), user_dir: None, generation: 0, others: HashMap::new() };
    s.load(BASE);
    Mutex::new(s)
});

fn state() -> std::sync::MutexGuard<'static, State> {
    STATE.lock().unwrap_or_else(|e| e.into_inner())
}

/// Sources of a language: the embedded files, then the user's (which override).
fn sources(lang: &str, user_dir: Option<&Path>) -> Vec<(String, String)> {
    let mut out: Vec<(String, String)> = embedded::EMBEDDED
        .iter()
        .filter(|(l, _, _)| *l == lang)
        .map(|(_, f, text)| (f.to_string(), text.to_string()))
        .collect();
    if let Some(dir) = user_dir {
        let p = dir.join(format!("{lang}.ftl"));
        if let Ok(text) = std::fs::read_to_string(&p) {
            out.push((p.display().to_string(), text));
        }
    }
    out
}

fn bundle(lang: &str, user_dir: Option<&Path>, errors: &mut Vec<String>) -> Bundle {
    let id: LanguageIdentifier = lang.parse().unwrap_or_default();
    let mut b = Bundle::new_concurrent(vec![id]);
    // Unicode isolation marks around arguments would show as boxes in the UI.
    b.set_use_isolating(false);
    let _ = b.add_function("PLATFORM", |_, _| FluentValue::from(platform()));
    for (name, text) in sources(lang, user_dir) {
        let res = match FluentResource::try_new(text) {
            Ok(r) => r,
            Err((r, errs)) => {
                errors.extend(errs.iter().map(|e| format!("{name}: {e:?}")));
                r
            }
        };
        b.add_resource_overriding(res);
    }
    b
}

impl State {
    fn chain(&self, lang: &str, errors: &mut Vec<String>) -> Vec<Bundle> {
        let mut bundles = vec![bundle(lang, self.user_dir.as_deref(), errors)];
        if lang != BASE {
            bundles.push(bundle(BASE, self.user_dir.as_deref(), errors));
        }
        bundles
    }

    fn load(&mut self, lang: &str) {
        let mut errors = Vec::new();
        let bundles = self.chain(lang, &mut errors);
        self.others.clear();
        self.lang = lang.to_string();
        self.bundles = bundles;
        self.errors = errors;
        self.cache.clear();
        self.missing.clear();
        self.generation += 1;
    }

    fn format(&mut self, key: &str, args: Option<&FluentArgs>) -> String {
        let attr = platform();
        let bundles = match override_lang().filter(|l| *l != self.lang) {
            Some(l) => {
                if !self.others.contains_key(&l) {
                    let chain = self.chain(&l, &mut Vec::new());
                    self.others.insert(l.clone(), chain);
                }
                &self.others[&l]
            }
            None => &self.bundles,
        };
        for b in bundles {
            let Some(msg) = b.get_message(key) else { continue };
            let Some(pattern) = msg.get_attribute(attr).map(|a| a.value()).or(msg.value()) else { continue };
            let mut errs = Vec::new();
            let text = b.format_pattern(pattern, args, &mut errs);
            return platform_keys(&text).into_owned();
        }
        self.missing.insert(key.to_string());
        key.to_string()
    }
}

/// The text of `key` in the current language.
pub fn tr(key: &str) -> String {
    let mut s = state();
    if override_lang().is_some_and(|l| l != s.lang) {
        return s.format(key, None);
    }
    if let Some(t) = s.cache.get(key) {
        return t.clone();
    }
    let t = s.format(key, None);
    s.cache.insert(key.to_string(), t.clone());
    t
}

/// The text of `key` with named arguments (`{ $name }` in the pattern).
pub fn tr_args(key: &str, args: &[(&str, FluentValue<'_>)]) -> String {
    let mut fa = FluentArgs::new();
    for (k, v) in args {
        fa.set(*k, v.clone());
    }
    state().format(key, Some(&fa))
}

/// `tr!("key")` / `tr!("key", name = value, …)`: values are strings or numbers
/// (numbers select plural forms: `{ $count -> [one] … *[other] … }`).
#[macro_export]
macro_rules! tr {
    ($key:expr) => {
        $crate::tr($key)
    };
    ($key:expr, $($name:ident = $value:expr),+ $(,)?) => {
        $crate::tr_args($key, &[$((stringify!($name), $crate::IntoArg::into_arg($value))),+])
    };
}

/// A value usable as a message argument.
pub trait IntoArg<'a> {
    fn into_arg(self) -> FluentValue<'a>;
}

impl<'a> IntoArg<'a> for &'a str {
    fn into_arg(self) -> FluentValue<'a> {
        FluentValue::from(self)
    }
}
impl<'a> IntoArg<'a> for &'a String {
    fn into_arg(self) -> FluentValue<'a> {
        FluentValue::from(self.as_str())
    }
}
impl<'a> IntoArg<'a> for String {
    fn into_arg(self) -> FluentValue<'a> {
        FluentValue::from(self)
    }
}
impl<'a> IntoArg<'a> for Cow<'a, str> {
    fn into_arg(self) -> FluentValue<'a> {
        FluentValue::from(self)
    }
}
macro_rules! num_arg {
    ($($t:ty),*) => {$(
        impl<'a> IntoArg<'a> for $t {
            fn into_arg(self) -> FluentValue<'a> {
                FluentValue::from(self)
            }
        }
    )*};
}
num_arg!(i8, i16, i32, i64, isize, u8, u16, u32, u64, usize, f32, f64);

/// Whether `key` exists (in the current language or the base one).
pub fn has(key: &str) -> bool {
    state().bundles.iter().any(|b| b.has_message(key))
}

/// The current language code (`en`, `ru`, …).
pub fn language() -> String {
    override_lang().unwrap_or_else(|| state().lang.clone())
}

/// Changes since the start: bumped on every language (re)load, so caches of texts can notice.
pub fn generation() -> u64 {
    state().generation
}

/// Keys asked for but found nowhere since the last (re)load.
pub fn missing_keys() -> Vec<String> {
    let mut v: Vec<String> = state().missing.iter().cloned().collect();
    v.sort();
    v
}

/// Syntax errors of the loaded `.ftl` files (shown in the log).
pub fn load_errors() -> Vec<String> {
    state().errors.clone()
}

/// The folder of the user's own translations (`<dir>/<lang>.ftl`); reloads the language.
pub fn set_user_dir(dir: Option<PathBuf>) {
    let mut s = state();
    s.user_dir = dir;
    let lang = s.lang.clone();
    s.load(&lang);
}

/// The language the system asks for, when there is a translation for it.
pub fn system_language() -> String {
    let available = languages();
    sys_locale::get_locales()
        .filter_map(|l| l.split(['-', '_']).next().map(str::to_lowercase))
        .find(|l| available.iter().any(|(c, _)| c == l))
        .unwrap_or_else(|| BASE.to_string())
}

/// Switches the language: `None` / `"system"` = the system's; unknown codes fall back to
/// English. The `VARSTOTAS_LANG` environment variable wins over both (QA runs in English).
pub fn set_language(code: Option<&str>) {
    let env = std::env::var("VARSTOTAS_LANG").ok().filter(|s| !s.is_empty());
    let want = match env.as_deref().or(code) {
        None | Some("system") => system_language(),
        Some(c) => c.to_string(),
    };
    let lang = if languages().iter().any(|(c, _)| *c == want) { want } else { BASE.to_string() };
    let mut s = state();
    if s.lang != lang {
        s.load(&lang);
    }
}

/// Available languages: (code, its own name from the `language-name` message), English first.
pub fn languages() -> Vec<(String, String)> {
    let user_dir = state().user_dir.clone();
    let mut codes: Vec<String> = embedded::EMBEDDED.iter().map(|(l, _, _)| l.to_string()).collect();
    if let Some(dir) = &user_dir {
        for e in std::fs::read_dir(dir).into_iter().flatten().flatten() {
            let p = e.path();
            if p.extension().is_some_and(|x| x == "ftl") {
                if let Some(stem) = p.file_stem() {
                    codes.push(stem.to_string_lossy().into_owned());
                }
            }
        }
    }
    codes.sort();
    codes.dedup();
    codes.sort_by_key(|c| c != BASE);
    codes
        .into_iter()
        .map(|c| {
            let mut errs = Vec::new();
            let b = bundle(&c, user_dir.as_deref(), &mut errs);
            let name = b
                .get_message("language-name")
                .and_then(|m| m.value())
                .map(|p| b.format_pattern(p, None, &mut Vec::new()).into_owned())
                .unwrap_or_else(|| c.clone());
            (c, name)
        })
        .collect()
}

/// Picks one of a plugin's localized variants of a text. `variants` maps a suffix to the text:
/// `""` is the plain tag, otherwise dot-separated parts that are language codes or platform
/// names (`ru`, `macos`, `ru.windows`). A variant whose parts all match wins over less specific
/// ones (a language counts more than the platform); the plain text is the fallback.
pub fn pick_variant(variants: &BTreeMap<String, String>) -> Option<&str> {
    let lang = language();
    let plat = platform();
    let score = |suffix: &str| -> Option<u32> {
        if suffix.is_empty() {
            return Some(0);
        }
        let mut s = 0;
        for part in suffix.split('.') {
            if part.eq_ignore_ascii_case(&lang) {
                s += 2;
            } else if part.eq_ignore_ascii_case(plat) {
                s += 1;
            } else {
                return None;
            }
        }
        Some(s)
    };
    variants
        .iter()
        .filter_map(|(k, v)| score(k).map(|s| (s, v)))
        .max_by_key(|(s, _)| *s)
        .map(|(_, v)| v.as_str())
        .or_else(|| variants.values().next().map(String::as_str))
}

/// Every message key of the embedded files of `lang` (tests and tools).
pub fn embedded_keys(lang: &str) -> BTreeMap<String, String> {
    let mut keys = BTreeMap::new();
    for (l, file, text) in embedded::EMBEDDED {
        if *l != lang {
            continue;
        }
        if let Ok(res) = fluent_syntax::parser::parse(*text) {
            for e in res.body {
                if let fluent_syntax::ast::Entry::Message(m) = e {
                    keys.insert(m.id.name.to_string(), file.to_string());
                }
            }
        }
    }
    keys
}

/// Syntax errors of the embedded files (tests).
pub fn embedded_errors() -> Vec<String> {
    let mut out = Vec::new();
    for (_, file, text) in embedded::EMBEDDED {
        if let Err((_, errs)) = fluent_syntax::parser::parse(*text) {
            out.extend(errs.iter().map(|e| format!("{file}: {e:?}")));
        }
    }
    out
}

/// The languages embedded at build time.
pub fn embedded_languages() -> Vec<&'static str> {
    let mut v: Vec<&str> = embedded::EMBEDDED.iter().map(|(l, _, _)| *l).collect();
    v.dedup();
    v
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn key_names_follow_the_platform() {
        let s = "%COPY% = copy, %CMD%+Enter runs, %ALT% = sample, `%CMDK%A` selects all";
        assert_eq!(platform_keys_for(s, true), "Option = copy, Cmd+Enter runs, Option = sample, `⌘A` selects all");
        assert_eq!(platform_keys_for(s, false), "Ctrl = copy, Ctrl+Enter runs, Alt = sample, `Ctrl+A` selects all");
        assert_eq!(platform_keys_for("50% off", false), "50% off");
    }
}
