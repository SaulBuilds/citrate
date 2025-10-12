// Inference module - AI model execution infrastructure

pub mod metal_runtime;

#[cfg(target_os = "macos")]
pub mod coreml_bridge;

pub use metal_runtime::{
    AppleSiliconChip, MetalCapabilities, MetalModel, MetalModelFormat, MetalRuntime,
    ModelConfig, QuantizationType,
};

#[cfg(target_os = "macos")]
pub use coreml_bridge::{CoreMLInference, CoreMLModel, ModelMetadata};
