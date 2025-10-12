// lattice-v3/core/execution/src/inference/mod.rs

// Inference module - AI model execution infrastructure

pub mod metal_runtime;

#[cfg(target_os = "macos")]
pub mod coreml_bridge;

pub use metal_runtime::{
    MetalRuntime,
    MetalCapabilities,
    MetalModel,
    MetalModelFormat,
    AppleSiliconChip,
};

#[cfg(target_os = "macos")]
pub use coreml_bridge::{
    CoreMLModel,
    CoreMLInference,
    ModelMetadata,
};