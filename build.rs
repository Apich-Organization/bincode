fn main() {
    println!("cargo:rustc-check-cfg=cfg(is_nightly)");

    let rustc = std::env::var_os("RUSTC").unwrap_or_else(|| "rustc".into());
    let output = std::process::Command::new(rustc).arg("-vV").output().ok();

    if let Some(output) = output {
        let stdout = String::from_utf8_lossy(&output.stdout);
        if stdout.contains("nightly") {
            println!("cargo:rustc-cfg=is_nightly");
        }
    }

    // Add cdylib and staticlib crate types for no-std (embedded/FFI) builds.
    // These are not included by default so that `cargo test --no-default-features`
    // works without requiring a panic handler or -Zbuild-std.
    if std::env::var("CARGO_FEATURE_NO_STD").is_ok() {
        println!("cargo:rustc-crate-type=cdylib");
        println!("cargo:rustc-crate-type=staticlib");
    }
}
