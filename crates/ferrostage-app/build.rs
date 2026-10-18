fn main() {
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
