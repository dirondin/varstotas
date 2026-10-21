//! Embeds every `locales/<lang>/*.ftl` of the repository into the binary.

use std::fmt::Write;
use std::path::PathBuf;

fn main() {
    let root = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap()).join("../../locales");
    let root = root.canonicalize().expect("locales/ folder");
    println!("cargo:rerun-if-changed={}", root.display());
    let mut files = Vec::new();
    for lang in std::fs::read_dir(&root).unwrap().flatten() {
        if !lang.path().is_dir() {
            continue;
        }
        println!("cargo:rerun-if-changed={}", lang.path().display());
        for f in std::fs::read_dir(lang.path()).unwrap().flatten() {
            let p = f.path();
            if p.extension().is_some_and(|e| e == "ftl") {
                files.push((lang.file_name().to_string_lossy().into_owned(), p));
            }
        }
    }
    files.sort();
    let mut out = String::from("pub static EMBEDDED: &[(&str, &str, &str)] = &[\n");
    for (lang, p) in files {
        let name = p.file_name().unwrap().to_string_lossy().into_owned();
        writeln!(out, "    ({lang:?}, {name:?}, include_str!({:?})),", p.display().to_string()).unwrap();
    }
    out.push_str("];\n");
    std::fs::write(PathBuf::from(std::env::var("OUT_DIR").unwrap()).join("embedded.rs"), out).unwrap();
}
