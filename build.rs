fn main() {
    let icon = format!("{}/dieselrogue.ico", env!("CARGO_MANIFEST_DIR"));
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=dieselrogue.ico");

    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows")
        && std::path::Path::new(&icon).exists()
    {
        let mut res = winres::WindowsResource::new();
        res.set_icon(&icon);
        res.compile().expect("failed to compile Windows resources");
    }
}
