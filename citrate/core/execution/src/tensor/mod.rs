
// citrate/core/execution/src/tensor/mod.rs

// Tensor module
// Handles tensor operations and engine management

pub mod engine;
pub mod ops;
pub mod types;

pub use engine::TensorEngine;
pub use ops::TensorOps;
pub use types::{Tensor, TensorError, TensorShape};
