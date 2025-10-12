// lattice-v3/core/execution/src/tensor/types.rs

use ndarray::{ArrayD, IxDyn};
use serde::de::{self};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Tensor representation for VM operations
#[derive(Debug, Clone)]
pub struct Tensor {
    pub data: ArrayD<f32>,
    pub shape: TensorShape,
    pub requires_grad: bool,
    pub grad: Option<Box<Tensor>>,
}

// Manual Serialize implementation for Tensor
impl Serialize for Tensor {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("Tensor", 4)?;

        // Serialize data as a flat vec
        let data_vec: Vec<f32> = self.data.iter().cloned().collect();
        state.serialize_field("data", &data_vec)?;
        state.serialize_field("shape", &self.shape)?;
        state.serialize_field("requires_grad", &self.requires_grad)?;
        state.serialize_field("grad", &self.grad)?;
        state.end()
    }
}

// Manual Deserialize implementation for Tensor
impl<'de> Deserialize<'de> for Tensor {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct TensorData {
            data: Vec<f32>,
            shape: TensorShape,
            requires_grad: bool,
            grad: Option<Box<Tensor>>,
        }

        let tensor_data = TensorData::deserialize(deserializer)?;
        let array = ArrayD::from_shape_vec(IxDyn(&tensor_data.shape.0), tensor_data.data)
            .map_err(de::Error::custom)?;

        Ok(Tensor {
            data: array,
            shape: tensor_data.shape,
            requires_grad: tensor_data.requires_grad,
            grad: tensor_data.grad,
        })
    }
}

impl Tensor {
    /// Create a new tensor from data
    pub fn new(data: Vec<f32>, shape: Vec<usize>) -> Result<Self, TensorError> {
        let total_elements: usize = shape.iter().product();
        if data.len() != total_elements {
            return Err(TensorError::ShapeMismatch {
                expected: total_elements,
                got: data.len(),
            });
        }

        let array = ArrayD::from_shape_vec(IxDyn(&shape), data)
            .map_err(|e| TensorError::InvalidShape(e.to_string()))?;

        Ok(Self {
            data: array,
            shape: TensorShape(shape),
            requires_grad: false,
            grad: None,
        })
    }

    /// Create a tensor of zeros
    pub fn zeros(shape: Vec<usize>) -> Self {
        let array = ArrayD::zeros(IxDyn(&shape));
        Self {
            data: array,
            shape: TensorShape(shape),
            requires_grad: false,
            grad: None,
        }
    }

    /// Create a tensor of ones
    pub fn ones(shape: Vec<usize>) -> Self {
        let array = ArrayD::ones(IxDyn(&shape));
        Self {
            data: array,
            shape: TensorShape(shape),
            requires_grad: false,
            grad: None,
        }
    }

    /// Create a random tensor
    pub fn random(shape: Vec<usize>) -> Self {
        use ndarray_rand::rand_distr::Uniform;
        use ndarray_rand::RandomExt;

        let array = ArrayD::random(IxDyn(&shape), Uniform::new(0., 1.));
        Self {
            data: array,
            shape: TensorShape(shape),
            requires_grad: false,
            grad: None,
        }
    }

    /// Enable gradient tracking
    pub fn require_grad(&mut self) {
        self.requires_grad = true;
    }

    /// Get tensor shape
    pub fn shape(&self) -> &[usize] {
        &self.shape.0
    }

    /// Get number of elements
    pub fn numel(&self) -> usize {
        self.data.len()
    }

    /// Reshape tensor
    pub fn reshape(&mut self, new_shape: Vec<usize>) -> Result<(), TensorError> {
        let total_elements: usize = new_shape.iter().product();
        if self.numel() != total_elements {
            return Err(TensorError::ShapeMismatch {
                expected: total_elements,
                got: self.numel(),
            });
        }

        self.data = self
            .data
            .clone()
            .into_shape(IxDyn(&new_shape))
            .map_err(|e| TensorError::InvalidShape(e.to_string()))?;
        self.shape = TensorShape(new_shape);
        Ok(())
    }

    /// Convert to bytes for storage
    pub fn to_bytes(&self) -> Vec<u8> {
        bincode::serialize(self).unwrap_or_default()
    }

    /// Create from bytes
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, TensorError> {
        bincode::deserialize(bytes).map_err(|e| TensorError::SerializationError(e.to_string()))
    }
}

/// Tensor shape wrapper
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TensorShape(pub Vec<usize>);

impl TensorShape {
    pub fn dims(&self) -> usize {
        self.0.len()
    }

    pub fn total_elements(&self) -> usize {
        self.0.iter().product()
    }

    pub fn is_compatible(&self, other: &TensorShape) -> bool {
        self.0 == other.0
    }

    pub fn broadcast_compatible(&self, other: &TensorShape) -> bool {
        if self.0.len() != other.0.len() {
            return false;
        }

        for (a, b) in self.0.iter().zip(&other.0) {
            if *a != *b && *a != 1 && *b != 1 {
                return false;
            }
        }
        true
    }
}

/// Tensor operation errors
#[derive(Debug, thiserror::Error)]
pub enum TensorError {
    #[error("Shape mismatch: expected {expected} elements, got {got}")]
    ShapeMismatch { expected: usize, got: usize },

    #[error("Invalid shape: {0}")]
    InvalidShape(String),

    #[error("Incompatible shapes for operation")]
    IncompatibleShapes,

    #[error("Division by zero")]
    DivisionByZero,

    #[error("Invalid axis: {0}")]
    InvalidAxis(usize),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Out of memory")]
    OutOfMemory,

    #[error("Unsupported operation")]
    UnsupportedOperation,
}
