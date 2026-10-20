fn main() {
    build_info();
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    if target_os != "windows" {
        return;
    }

    let manifest_dir =
        std::path::PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR"));
    let icon_dir = manifest_dir.join("../../assets/icons");
    let rc = manifest_dir.join("resources/windows/ferrostage.rc");
    println!("cargo:rerun-if-changed={}", rc.display());
    println!(
        "cargo:rerun-if-changed={}",
        icon_dir.join("ferrostage.ico").display()
    );

    embed_resource::compile(rc, embed_resource::ParamsIncludeDirs([icon_dir]))
        .manifest_optional()
        .expect("compile ferrostage.exe icon resource");
}

/// Settings > About metadata. Rerun when workspace sources change, so the
/// time describes the sources in this binary.
fn build_info() {
    let manifest_dir =
        std::path::PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR"));
    let root = manifest_dir.join("../..");
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-env-changed=SOURCE_DATE_EPOCH");
    for path in ["crates", "assets", "Cargo.toml", "Cargo.lock"] {
        let path = root.join(path);
        if path.exists() {
            println!("cargo:rerun-if-changed={}", path.display());
        }
    }
    let epoch = std::env::var("SOURCE_DATE_EPOCH")
        .ok()
        .and_then(|value| value.parse::<i64>().ok())
        .unwrap_or_else(|| {
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_or(0, |elapsed| elapsed.as_secs() as i64)
        });
    println!("cargo:rustc-env=FERROSTAGE_BUILD_EPOCH={epoch}");
}
