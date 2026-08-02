//! Embeds the Windows icon and version resource into the native executable.
//!
//! Windows host building a Windows target only: the wasm build has no
//! resource section, and a non-Windows host has no RC toolchain. A resource
//! failure downgrades to a warning — the exe still runs, it just ships
//! without its badge.

fn main() {
    println!("cargo:rerun-if-changed=assets/icon.ico");
    #[cfg(windows)]
    embed_windows_resources();
}

#[cfg(windows)]
fn embed_windows_resources() {
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }
    let mut res = winresource::WindowsResource::new();
    res.set_icon("assets/icon.ico");
    res.set("ProductName", "Nightmare Shift");
    res.set(
        "FileDescription",
        "Nightmare Shift - a horror taxi roguelite",
    );
    res.set("OriginalFilename", "nightmare_shift.exe");
    res.set("LegalCopyright", "(c) WebHatchery");
    if let Err(error) = res.compile() {
        println!("cargo:warning=resource embedding skipped: {error}");
    }
}
