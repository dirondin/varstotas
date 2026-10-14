//! Embeds the Windows icon and version resource into `varstotas.exe`.
//! No-op for every other target. The icon is optional: `scripts/build-windows.sh`
//! generates one and passes its path via `VARSTOTAS_WIN_ICON`.

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-env-changed=VARSTOTAS_WIN_ICON");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }
    let mut res = winresource::WindowsResource::new();
    res.set("ProductName", "Varstotas")
        .set("FileDescription", "Varstotas — 3D modeler")
        .set("CompanyName", "Varstotas")
        .set("LegalCopyright", "MIT License");
    if let Ok(icon) = std::env::var("VARSTOTAS_WIN_ICON") {
        println!("cargo:rerun-if-changed={icon}");
        res.set_icon(&icon);
    }
    if let Err(e) = res.compile() {
        // Missing windres/rc must not break the build — the exe just has no icon.
        println!("cargo:warning=windows resource not embedded: {e}");
    }
}
