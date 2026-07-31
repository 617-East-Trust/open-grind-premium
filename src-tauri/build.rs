fn main() {
    // Emit 16 KB ELF PT_LOAD alignment for Android 15+ compatibility.
    // Added because Android 15+ on 16KB-page-size devices (e.g. Pixel 9
    // Pro XL) refuses to install or launch apps with unaligned .so files.
    // These linker flags only apply when targeting Android (clang).
    let target = std::env::var("TARGET").unwrap_or_default();
    if target.contains("android") {
        println!("cargo:rustc-link-arg=-Wl,-z,max-page-size=16384");
    }
    tauri_build::build()
}
