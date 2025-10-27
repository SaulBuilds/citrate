// citrate/core/execution/build.rs

// Build script for CoreML bridge
use std::env;

fn main() {
    // Only compile CoreML bridge on macOS
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();

    if target_os == "macos" {
        // Compile Objective-C bridge
        cc::Build::new()
            .file("src/inference/coreml_bridge.m")
            .flag("-fobjc-arc")
            .flag("-fmodules")
            .compile("coreml_bridge");

        // Link to Apple frameworks
        println!("cargo:rustc-link-lib=framework=CoreML");
        println!("cargo:rustc-link-lib=framework=Foundation");
        println!("cargo:rustc-link-lib=framework=Metal");
        println!("cargo:rustc-link-lib=framework=MetalPerformanceShaders");
        println!("cargo:rustc-link-lib=framework=Accelerate");
    }
}