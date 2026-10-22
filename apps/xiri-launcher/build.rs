//! Windows only: embed the app icon and version info (VERSIONINFO) into `xiri-launcher.exe`.
//!
//! On every other target this does nothing. A missing resource compiler is a warning, so a
//! cross-compile from macOS or Linux still links, unless `XIRI_LAUNCHER_REQUIRE_WINRES=1` (set
//! by the release packaging) turns it into an error.

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=../../assets/app-icon/xiri-launcher.ico");
    println!("cargo:rerun-if-env-changed=XIRI_LAUNCHER_REQUIRE_WINRES");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }
    let mut res = winresource::WindowsResource::new();
    res.set_icon("../../assets/app-icon/xiri-launcher.ico")
        .set("ProductName", "Xiri Launcher")
        .set("ProductVersion", env!("CARGO_PKG_VERSION"))
        .set("FileVersion", env!("CARGO_PKG_VERSION"))
        .set("FileDescription", "Xiri Launcher")
        .set("CompanyName", "sugyua")
        .set("LegalCopyright", "Copyright (c) the Xiri Launcher authors. MIT OR Apache-2.0.")
        .set("OriginalFilename", "xiri-launcher.exe")
        .set("InternalName", "xiri-launcher");
    if let Err(e) = res.compile() {
        if std::env::var_os("XIRI_LAUNCHER_REQUIRE_WINRES").is_some() {
            println!("cargo:warning=embedding Windows resources failed: {e}");
            std::process::exit(1);
        }
        println!("cargo:warning=xiri-launcher.exe built without icon/version resources: {e}");
    }
}
