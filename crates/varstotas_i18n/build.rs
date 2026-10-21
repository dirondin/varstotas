//! Embeds every `locales/<lang>.ftl` of the repository into the binary.

use std::fmt::Write;
use std::path::PathBuf;

fn main() {
    let root = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap()).join("../../locales");
    let root = root.canonicalize().expect("locales/ folder");
    println!("cargo:rerun-if-changed={}", root.display());
    let mut files: Vec<PathBuf> = std::fs::read_dir(&root).unwrap().flatten().map(|e| e.path()).filter(|p| p.extension().is_some_and(|e| e == "ftl")).collect();
    files.sort();
    let mut out = String::from("pub static EMBEDDED: &[(&str, &str, &str)] = &[\n");
    for p in files {
        println!("cargo:rerun-if-changed={}", p.display());
        let lang = p.file_stem().unwrap().to_string_lossy().into_owned();
        let name = p.file_name().unwrap().to_string_lossy().into_owned();
        writeln!(out, "    ({lang:?}, {name:?}, include_str!({:?})),", p.display().to_string()).unwrap();
    }
    out.push_str("];\n");
    std::fs::write(PathBuf::from(std::env::var("OUT_DIR").unwrap()).join("embedded.rs"), out).unwrap();
}
