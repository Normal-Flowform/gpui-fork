#![allow(clippy::disallowed_methods, reason = "build scripts are exempt")]

fn main() {
    println!("cargo::rustc-check-cfg=cfg(gles)");

    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();

    if target_os == "windows" {
        #[cfg(feature = "windows-manifest")]
        embed_resource();
    }
}

#[cfg(feature = "windows-manifest")]
fn embed_resource() {
    let crate_root = std::path::PathBuf::from(
        std::env::var_os("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR is set by Cargo"),
    );
    let manifest = crate_root.join("resources/windows/gpui.manifest.xml");
    let rc_file = crate_root.join("resources/windows/gpui.rc");
    println!("cargo:rerun-if-changed={}", manifest.display());
    println!("cargo:rerun-if-changed={}", rc_file.display());

    // `embed-resource` preprocesses the RC file into OUT_DIR. Resource
    // compilers then resolve quoted payload paths relative to that generated
    // file, not the crate, which breaks cross-builds and out-of-tree targets.
    let rc_template = std::fs::read_to_string(&rc_file).expect("gpui.rc is checked into the crate");
    let manifest_path = manifest.to_string_lossy().replace('\\', "/");
    let generated_rc = rc_template.replace(
        "resources/windows/gpui.manifest.xml",
        manifest_path.as_str(),
    );
    let generated_rc_path =
        std::path::PathBuf::from(std::env::var_os("OUT_DIR").expect("OUT_DIR is set by Cargo"))
            .join("gpui.rc");
    std::fs::write(&generated_rc_path, generated_rc).expect("write generated gpui.rc");

    embed_resource::compile(generated_rc_path, embed_resource::NONE)
        .manifest_required()
        .unwrap();
}
