//! Stamps the build number and commit into the binary (`VARSTOTAS_BUILD`, `VARSTOTAS_COMMIT`)
//! and, on Windows, embeds the icon and version resource into `varstotas.exe`.
//! The build number is CI's run number (`GITHUB_RUN_NUMBER`, or `VARSTOTAS_BUILD` to override);
//! local builds have none and show as development builds. The icon is optional:
//! `scripts/build-windows.sh` generates one and passes its path via `VARSTOTAS_WIN_ICON`.
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    for var in ["VARSTOTAS_WIN_ICON", "VARSTOTAS_BUILD", "GITHUB_RUN_NUMBER", "GITHUB_SHA"] {
        println!("cargo:rerun-if-env-changed={var}");
    }
    let build = std::env::var("VARSTOTAS_BUILD")
        .or_else(|_| std::env::var("GITHUB_RUN_NUMBER"))
        .ok()
        .filter(|b| !b.is_empty() && b.bytes().all(|c| c.is_ascii_digit()))
        .unwrap_or_default();
    let commit = git(&["rev-parse", "--short=7", "HEAD"])
        .or_else(|| std::env::var("GITHUB_SHA").ok().map(|s| s.chars().take(7).collect()))
        .unwrap_or_default();
    // A new commit or checkout moves HEAD's reflog; rerun then so the hash stays current.
    if let Some(dir) = git(&["rev-parse", "--absolute-git-dir"]) {
        println!("cargo:rerun-if-changed={dir}/HEAD");
        println!("cargo:rerun-if-changed={dir}/logs/HEAD");
    }
    println!("cargo:rustc-env=VARSTOTAS_BUILD={build}");
    println!("cargo:rustc-env=VARSTOTAS_COMMIT={commit}");

    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }
    let mut res = winresource::WindowsResource::new();
    res.set("ProductName", "Varstotas")
        .set("FileDescription", "Varstotas — 3D modeler")
        .set("CompanyName", "Varstotas")
        .set("LegalCopyright", "MIT License");
    if let Ok(number) = build.parse::<u64>() {
        // major.minor.patch.build, 16 bits each
        let part = |v: &str| std::env::var(v).ok().and_then(|s| s.parse::<u64>().ok()).unwrap_or(0) & 0xffff;
        let version = part("CARGO_PKG_VERSION_MAJOR") << 48
            | part("CARGO_PKG_VERSION_MINOR") << 32
            | part("CARGO_PKG_VERSION_PATCH") << 16
            | number & 0xffff;
        res.set_version_info(winresource::VersionInfo::FILEVERSION, version)
            .set_version_info(winresource::VersionInfo::PRODUCTVERSION, version);
    }
    if let Ok(icon) = std::env::var("VARSTOTAS_WIN_ICON") {
        println!("cargo:rerun-if-changed={icon}");
        res.set_icon(&icon);
    }
    if let Err(e) = res.compile() {
        // Missing windres/rc must not break the build — the exe just has no icon.
        println!("cargo:warning=windows resource not embedded: {e}");
    }
}

fn git(args: &[&str]) -> Option<String> {
    let out = Command::new("git").args(args).output().ok()?;
    let text = String::from_utf8(out.stdout).ok()?.trim().to_string();
    (out.status.success() && !text.is_empty()).then_some(text)
}
