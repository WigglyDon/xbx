fn main() {
    println!("cargo:rerun-if-env-changed=XBX_NATIVE_LIB_DIR");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("android") {
        let directory = std::env::var("XBX_NATIVE_LIB_DIR")
            .expect("set XBX_NATIVE_LIB_DIR to the source-built xemu/SDL3 library directory");
        println!("cargo:rustc-link-search=native={directory}");
        println!("cargo:rustc-link-search=native={directory}/_deps/sdl3-build");
    }
}
