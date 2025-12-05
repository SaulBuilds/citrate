//! Build script for Citrate Core GUI
//!
//! This script runs at compile time and sets up:
//! - Tauri build configuration
//! - Dev mode detection via environment variables
//! - Build metadata for runtime use

use std::env;
use std::time::{SystemTime, UNIX_EPOCH};

fn main() {
    // Standard Tauri build
    tauri_build::build();

    // Detect dev mode from environment
    let dev_mode = env::var("CITRATE_DEV_MODE")
        .map(|v| v == "true" || v == "1")
        .unwrap_or(false)
        || env::var("PROFILE")
            .map(|p| p == "debug")
            .unwrap_or(false);

    // Set cfg flag for conditional compilation
    if dev_mode {
        println!("cargo:rustc-cfg=dev_mode");
    }

    // Expose build timestamp (Unix epoch seconds)
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    println!("cargo:rustc-env=BUILD_TIMESTAMP={}", timestamp);

    // Rerun if environment changes
    println!("cargo:rerun-if-env-changed=CITRATE_DEV_MODE");
    println!("cargo:rerun-if-env-changed=PROFILE");
}
