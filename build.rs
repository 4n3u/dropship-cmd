use std::path::Path;

fn main() {
    if !cfg!(target_os = "windows") {
        return;
    }

    let icon_ico = Path::new(env!("CARGO_MANIFEST_DIR")).join("icon.ico");
    println!("cargo:rerun-if-changed={}", icon_ico.display());

    let mut resource = winres::WindowsResource::new();
    resource.set_icon(icon_ico.to_string_lossy().as_ref());
    resource
        .compile()
        .expect("failed to compile Windows resources");
}
