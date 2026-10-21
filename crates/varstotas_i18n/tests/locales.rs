//! The embedded locales: syntax, keys of the translations against English, lookup rules.

use std::collections::BTreeMap;
use varstotas_i18n::*;

#[test]
fn locales_parse() {
    let errs = embedded_errors();
    assert!(errs.is_empty(), "{errs:#?}");
}

#[test]
fn translations_have_no_stale_keys() {
    let en = embedded_keys(BASE);
    for lang in embedded_languages() {
        let stale: Vec<String> = embedded_keys(lang).into_keys().filter(|k| !en.contains_key(k)).collect();
        assert!(stale.is_empty(), "{lang}: keys not in English: {stale:?}");
    }
}

#[test]
fn lookup_platform_fallback_and_plugins() {
    set_language(Some("en"));
    assert_eq!(language(), "en");
    let reveal = tr!("reveal-in-file-manager");
    match platform() {
        "macos" => assert_eq!(reveal, "Reveal in Finder"),
        "windows" => assert_eq!(reveal, "Show in Explorer"),
        _ => assert_eq!(reveal, "Show in File Manager"),
    }
    assert_eq!(tr!("no-such-key-anywhere"), "no-such-key-anywhere");
    assert!(missing_keys().contains(&"no-such-key-anywhere".to_string()));
    assert!(languages().iter().any(|(c, n)| c == "ru" && n == "Русский"));

    set_language(Some("ru"));
    assert_eq!(language(), "ru");
    assert_eq!(tr!("cancel"), "Отмена");
    set_language(Some("xx"));
    assert_eq!(language(), "en");

    let v: BTreeMap<String, String> =
        [("", "Stairs"), ("ru", "Лестница"), ("ru.macos", "Лестница (Mac)"), ("macos", "Stairs (Mac)")].into_iter().map(|(a, b)| (a.to_string(), b.to_string())).collect();
    let mac = platform() == "macos";
    assert_eq!(pick_variant(&v), Some(if mac { "Stairs (Mac)" } else { "Stairs" }));
    with_language("ru", || assert_eq!(pick_variant(&v), Some(if mac { "Лестница (Mac)" } else { "Лестница" })));
    with_language("ru", || assert_eq!(tr!("prefs-language-system", language = "Русский"), "Как в системе (Русский)"));
    assert_eq!(tr!("cancel"), "Cancel");
}

/// Every key-like literal of the sources (`"<area>-…"` with the area of an English key) is
/// an English key: catches typos and keys forgotten in the .ftl files.
#[test]
fn keys_used_in_the_sources_exist() {
    let en = embedded_keys(BASE);
    let areas: std::collections::BTreeSet<&str> = en.keys().filter_map(|k| k.split('-').next()).collect();
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut missing = Vec::new();
    let mut stack = vec![root.join("crates")];
    while let Some(d) = stack.pop() {
        for e in std::fs::read_dir(&d).unwrap().flatten() {
            let p = e.path();
            if p.is_dir() {
                if p.file_name().is_some_and(|n| n != "target") {
                    stack.push(p);
                }
                continue;
            }
            if p.extension().is_none_or(|x| x != "rs") || !p.to_string_lossy().contains("/src/") {
                continue;
            }
            let text = std::fs::read_to_string(&p).unwrap();
            for (n, line) in text.lines().enumerate() {
                if line.trim_start().starts_with("//") {
                    continue;
                }
                for lit in line.split('"').skip(1).step_by(2) {
                    let key_like = lit.len() > 2
                        && lit.contains('-')
                        && lit.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
                        && !lit.starts_with('-')
                        && !lit.ends_with('-');
                    if key_like && areas.contains(lit.split('-').next().unwrap()) && !en.contains_key(lit) {
                        missing.push(format!("{}:{}: {lit}", p.display(), n + 1));
                    }
                }
            }
        }
    }
    assert!(missing.is_empty(), "keys not in locales/en: {missing:#?}");
}
