fn main() {
    // Skia's ICU unicode module uses Windows registry APIs on Windows,
    // which live in advapi32.lib.
    #[cfg(target_os = "windows")]
    println!("cargo:rustc-link-lib=advapi32");
}
