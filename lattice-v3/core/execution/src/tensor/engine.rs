use super::ops::TensorOps;
use super::types::{Tensor, TensorError};
use primitive_types::U256;
use std::collections::HashMap;

/// Tensor execution engine for the VM
pub struct TensorEngine {
    tensors: HashMap<U256, Tensor>,
    next_id: U256,
    max_memory: usize,
    current_memory: usize,
}

impl TensorEngine {
    pub fn new(max_memory_mb: usize) -> Self {
        Self {
            tensors: HashMap::new(),
            next_id: U256::one(),
            max_memory: max_memory_mb * 1024 * 1024,
            current_memory: 0,
        }
    }

    /// Allocate a new tensor
    pub fn create_tensor(
        &mut self,
        data: Vec<f32>,
        shape: Vec<usize>,
    ) -> Result<U256, TensorError> {
        let tensor = Tensor::new(data, shape)?;
        let memory_size = tensor.numel() * std::mem::size_of::<f32>();

        if self.current_memory + memory_size > self.max_memory {
            return Err(TensorError::OutOfMemory);
        }

        let id = self.next_id;
        self.tensors.insert(id, tensor);
        self.next_id += U256::one();
        self.current_memory += memory_size;

        Ok(id)
    }

    /// Create tensor with zeros
    pub fn create_zeros(&mut self, shape: Vec<usize>) -> Result<U256, TensorError> {
        let tensor = Tensor::zeros(shape);
        let memory_size = tensor.numel() * std::mem::size_of::<f32>();

        if self.current_memory + memory_size > self.max_memory {
            return Err(TensorError::OutOfMemory);
        }

        let id = self.next_id;
        self.tensors.insert(id, tensor);
        self.next_id += U256::one();
        self.current_memory += memory_size;

        Ok(id)
    }

    /// Create tensor with ones
    pub fn create_ones(&mut self, shape: Vec<usize>) -> Result<U256, TensorError> {
        let tensor = Tensor::ones(shape);
        let memory_size = tensor.numel() * std::mem::size_of::<f32>();

        if self.current_memory + memory_size > self.max_memory {
            return Err(TensorError::OutOfMemory);
        }

        let id = self.next_id;
        self.tensors.insert(id, tensor);
        self.next_id += U256::one();
        self.current_memory += memory_size;

        Ok(id)
    }

    /// Create random tensor
    pub fn create_random(&mut self, shape: Vec<usize>) -> Result<U256, TensorError> {
        let tensor = Tensor::random(shape);
        let memory_size = tensor.numel() * std::mem::size_of::<f32>();

        if self.current_memory + memory_size > self.max_memory {
            return Err(TensorError::OutOfMemory);
        }

        let id = self.next_id;
        self.tensors.insert(id, tensor);
        self.next_id += U256::one();
        self.current_memory += memory_size;

        Ok(id)
    }

    /// Get tensor by ID
    pub fn get_tensor(&self, id: &U256) -> Option<&Tensor> {
        self.tensors.get(id)
    }

    /// Get mutable tensor by ID
    pub fn get_tensor_mut(&mut self, id: &U256) -> Option<&mut Tensor> {
        self.tensors.get_mut(id)
    }

    /// Delete tensor
    pub fn delete_tensor(&mut self, id: &U256) -> Result<(), TensorError> {
        if let Some(tensor) = self.tensors.remove(id) {
            let memory_size = tensor.numel() * std::mem::size_of::<f32>();
            self.current_memory = self.current_memory.saturating_sub(memory_size);
            Ok(())
        } else {
            Err(TensorError::InvalidShape("Tensor not found".to_string()))
        }
    }

    /// Perform element-wise addition
    pub fn add(&mut self, a_id: &U256, b_id: &U256) -> Result<U256, TensorError> {
        let a = self
            .tensors
            .get(a_id)
            .ok_or_else(|| TensorError::InvalidShape("Tensor A not found".to_string()))?;
        let b = self
            .tensors
            .get(b_id)
            .ok_or_else(|| TensorError::InvalidShape("Tensor B not found".to_string()))?;

        let result = TensorOps::add(a, b)?;
        let memory_size = result.numel() * std::mem::size_of::<f32>();

        if self.current_memory + memory_size > self.max_memory {
            return Err(TensorError::OutOfMemory);
        }

        let id = self.next_id;
        self.tensors.insert(id, result);
        self.next_id += U256::one();
        self.current_memory += memory_size;

        Ok(id)
    }

    /// Perform element-wise subtraction
    pub fn sub(&mut self, a_id: &U256, b_id: &U256) -> Result<U256, TensorError> {
        let a = self
            .tensors
            .get(a_id)
            .ok_or_else(|| TensorError::InvalidShape("Tensor A not found".to_string()))?;
        let b = self
            .tensors
            .get(b_id)
            .ok_or_else(|| TensorError::InvalidShape("Tensor B not found".to_string()))?;

        let result = TensorOps::sub(a, b)?;
        let memory_size = result.numel() * std::mem::size_of::<f32>();

        if self.current_memory + memory_size > self.max_memory {
            return Err(TensorError::OutOfMemory);
        }

        let id = self.next_id;
        self.tensors.insert(id, result);
        self.next_id += U256::one();
        self.current_memory += memory_size;

        Ok(id)
    }

    /// Perform element-wise multiplication
    pub fn mul(&mut self, a_id: &U256, b_id: &U256) -> Result<U256, TensorError> {
        let a = self
            .tensors
            .get(a_id)
            .ok_or_else(|| TensorError::InvalidShape("Tensor A not found".to_string()))?;
        let b = self
            .tensors
            .get(b_id)
            .ok_or_else(|| TensorError::InvalidShape("Tensor B not found".to_string()))?;

        let result = TensorOps::mul(a, b)?;
        let memory_size = result.numel() * std::mem::size_of::<f32>();

        if self.current_memory + memory_size > self.max_memory {
            return Err(TensorError::OutOfMemory);
        }

        let id = self.next_id;
        self.tensors.insert(id, result);
        self.next_id += U256::one();
        self.current_memory += memory_size;

        Ok(id)
    }

    /// Perform matrix multiplication
    pub fn matmul(&mut self, a_id: &U256, b_id: &U256) -> Result<U256, TensorError> {
        let a = self
            .tensors
            .get(a_id)
            .ok_or_else(|| TensorError::InvalidShape("Tensor A not found".to_string()))?;
        let b = self
            .tensors
            .get(b_id)
            .ok_or_else(|| TensorError::InvalidShape("Tensor B not found".to_string()))?;

        let result = TensorOps::matmul(a, b)?;
        let memory_size = result.numel() * std::mem::size_of::<f32>();

        if self.current_memory + memory_size > self.max_memory {
            return Err(TensorError::OutOfMemory);
        }

        let id = self.next_id;
        self.tensors.insert(id, result);
        self.next_id += U256::one();
        self.current_memory += memory_size;

        Ok(id)
    }

    /// Apply ReLU activation
    pub fn relu(&mut self, tensor_id: &U256) -> Result<U256, TensorError> {
        let tensor = self
            .tensors
            .get(tensor_id)
            .ok_or_else(|| TensorError::InvalidShape("Tensor not found".to_string()))?;

        let result = TensorOps::relu(tensor);
        let memory_size = result.numel() * std::mem::size_of::<f32>();

        if self.current_memory + memory_size > self.max_memory {
            return Err(TensorError::OutOfMemory);
        }

        let id = self.next_id;
        self.tensors.insert(id, result);
        self.next_id += U256::one();
        self.current_memory += memory_size;

        Ok(id)
    }

    /// Apply Sigmoid activation
    pub fn sigmoid(&mut self, tensor_id: &U256) -> Result<U256, TensorError> {
        let tensor = self
            .tensors
            .get(tensor_id)
            .ok_or_else(|| TensorError::InvalidShape("Tensor not found".to_string()))?;

        let result = TensorOps::sigmoid(tensor);
        let memory_size = result.numel() * std::mem::size_of::<f32>();

        if self.current_memory + memory_size > self.max_memory {
            return Err(TensorError::OutOfMemory);
        }

        let id = self.next_id;
        self.tensors.insert(id, result);
        self.next_id += U256::one();
        self.current_memory += memory_size;

        Ok(id)
    }

    /// Apply Softmax activation
    pub fn softmax(&mut self, tensor_id: &U256) -> Result<U256, TensorError> {
        let tensor = self
            .tensors
            .get(tensor_id)
            .ok_or_else(|| TensorError::InvalidShape("Tensor not found".to_string()))?;

        let result = TensorOps::softmax(tensor)?;
        let memory_size = result.numel() * std::mem::size_of::<f32>();

        if self.current_memory + memory_size > self.max_memory {
            return Err(TensorError::OutOfMemory);
        }

        let id = self.next_id;
        self.tensors.insert(id, result);
        self.next_id += U256::one();
        self.current_memory += memory_size;

        Ok(id)
    }

    /// Transpose tensor
    pub fn transpose(&mut self, tensor_id: &U256) -> Result<U256, TensorError> {
        let tensor = self
            .tensors
            .get(tensor_id)
            .ok_or_else(|| TensorError::InvalidShape("Tensor not found".to_string()))?;

        let result = TensorOps::transpose(tensor)?;
        let memory_size = result.numel() * std::mem::size_of::<f32>();

        if self.current_memory + memory_size > self.max_memory {
            return Err(TensorError::OutOfMemory);
        }

        let id = self.next_id;
        self.tensors.insert(id, result);
        self.next_id += U256::one();
        self.current_memory += memory_size;

        Ok(id)
    }

    /// Reshape tensor
    pub fn reshape(&mut self, tensor_id: &U256, new_shape: Vec<usize>) -> Result<(), TensorError> {
        let tensor = self
            .tensors
            .get_mut(tensor_id)
            .ok_or_else(|| TensorError::InvalidShape("Tensor not found".to_string()))?;

        tensor.reshape(new_shape)
    }

    /// Get tensor shape
    pub fn get_shape(&self, tensor_id: &U256) -> Option<Vec<usize>> {
        self.tensors.get(tensor_id).map(|t| t.shape.0.clone())
    }

    /// Get tensor data as bytes
    pub fn get_tensor_bytes(&self, tensor_id: &U256) -> Option<Vec<u8>> {
        self.tensors.get(tensor_id).map(|t| t.to_bytes())
    }

    /// Load tensor from bytes
    pub fn load_tensor_bytes(&mut self, bytes: &[u8]) -> Result<U256, TensorError> {
        let tensor = Tensor::from_bytes(bytes)?;
        let memory_size = tensor.numel() * std::mem::size_of::<f32>();

        if self.current_memory + memory_size > self.max_memory {
            return Err(TensorError::OutOfMemory);
        }

        let id = self.next_id;
        self.tensors.insert(id, tensor);
        self.next_id += U256::one();
        self.current_memory += memory_size;

        Ok(id)
    }

    /// Clear all tensors
    pub fn clear(&mut self) {
        self.tensors.clear();
        self.current_memory = 0;
    }

    /// Get memory usage
    pub fn memory_usage(&self) -> (usize, usize) {
        (self.current_memory, self.max_memory)
    }
}
