use super::types::{Tensor, TensorError};
use ndarray::{ArrayD, Axis, IxDyn};

/// Tensor operations implementation
pub struct TensorOps;

impl TensorOps {
    /// Element-wise addition
    pub fn add(a: &Tensor, b: &Tensor) -> Result<Tensor, TensorError> {
        if !a.shape.broadcast_compatible(&b.shape) {
            return Err(TensorError::IncompatibleShapes);
        }

        let result = &a.data + &b.data;
        Ok(Tensor {
            data: result,
            shape: a.shape.clone(),
            requires_grad: a.requires_grad || b.requires_grad,
            grad: None,
        })
    }

    /// Element-wise subtraction
    pub fn sub(a: &Tensor, b: &Tensor) -> Result<Tensor, TensorError> {
        if !a.shape.broadcast_compatible(&b.shape) {
            return Err(TensorError::IncompatibleShapes);
        }

        let result = &a.data - &b.data;
        Ok(Tensor {
            data: result,
            shape: a.shape.clone(),
            requires_grad: a.requires_grad || b.requires_grad,
            grad: None,
        })
    }

    /// Element-wise multiplication
    pub fn mul(a: &Tensor, b: &Tensor) -> Result<Tensor, TensorError> {
        if !a.shape.broadcast_compatible(&b.shape) {
            return Err(TensorError::IncompatibleShapes);
        }

        let result = &a.data * &b.data;
        Ok(Tensor {
            data: result,
            shape: a.shape.clone(),
            requires_grad: a.requires_grad || b.requires_grad,
            grad: None,
        })
    }

    /// Element-wise division
    pub fn div(a: &Tensor, b: &Tensor) -> Result<Tensor, TensorError> {
        if !a.shape.broadcast_compatible(&b.shape) {
            return Err(TensorError::IncompatibleShapes);
        }

        // Check for division by zero
        if b.data.iter().any(|&x| x == 0.0) {
            return Err(TensorError::DivisionByZero);
        }

        let result = &a.data / &b.data;
        Ok(Tensor {
            data: result,
            shape: a.shape.clone(),
            requires_grad: a.requires_grad || b.requires_grad,
            grad: None,
        })
    }

    /// Matrix multiplication
    pub fn matmul(a: &Tensor, b: &Tensor) -> Result<Tensor, TensorError> {
        // Check dimensions are compatible for matrix multiplication
        let a_shape = &a.shape.0;
        let b_shape = &b.shape.0;

        if a_shape.len() < 2 || b_shape.len() < 2 {
            return Err(TensorError::InvalidShape(
                "Tensors must have at least 2 dimensions for matmul".to_string(),
            ));
        }

        let a_cols = a_shape[a_shape.len() - 1];
        let b_rows = b_shape[b_shape.len() - 2];

        if a_cols != b_rows {
            return Err(TensorError::IncompatibleShapes);
        }

        // Perform matrix multiplication using ndarray's dot product
        // This is a simplified version - full implementation would handle batched matmul
        let a_2d = a
            .data
            .view()
            .into_dimensionality::<ndarray::Ix2>()
            .map_err(|_| TensorError::InvalidShape("Cannot convert to 2D".to_string()))?;
        let b_2d = b
            .data
            .view()
            .into_dimensionality::<ndarray::Ix2>()
            .map_err(|_| TensorError::InvalidShape("Cannot convert to 2D".to_string()))?;

        let result_2d = a_2d.dot(&b_2d);
        let result_shape = vec![a_shape[0], b_shape[1]];
        let result = ArrayD::from_shape_vec(IxDyn(&result_shape), result_2d.into_raw_vec())
            .map_err(|e| TensorError::InvalidShape(e.to_string()))?;

        Ok(Tensor {
            data: result,
            shape: super::types::TensorShape(result_shape),
            requires_grad: a.requires_grad || b.requires_grad,
            grad: None,
        })
    }

    /// Transpose tensor (swap last two dimensions)
    pub fn transpose(tensor: &Tensor) -> Result<Tensor, TensorError> {
        let shape = &tensor.shape.0;
        if shape.len() < 2 {
            return Err(TensorError::InvalidShape(
                "Tensor must have at least 2 dimensions".to_string(),
            ));
        }

        let mut axes: Vec<usize> = (0..shape.len()).collect();
        let n = axes.len();
        axes.swap(n - 2, n - 1);

        let transposed = tensor.data.view().permuted_axes(axes);
        let mut new_shape = shape.clone();
        new_shape.swap(n - 2, n - 1);

        Ok(Tensor {
            data: transposed.to_owned(),
            shape: super::types::TensorShape(new_shape),
            requires_grad: tensor.requires_grad,
            grad: None,
        })
    }

    /// Apply ReLU activation
    pub fn relu(tensor: &Tensor) -> Tensor {
        let result = tensor.data.mapv(|x| x.max(0.0));
        Tensor {
            data: result,
            shape: tensor.shape.clone(),
            requires_grad: tensor.requires_grad,
            grad: None,
        }
    }

    /// Apply Sigmoid activation
    pub fn sigmoid(tensor: &Tensor) -> Tensor {
        let result = tensor.data.mapv(|x| 1.0 / (1.0 + (-x).exp()));
        Tensor {
            data: result,
            shape: tensor.shape.clone(),
            requires_grad: tensor.requires_grad,
            grad: None,
        }
    }

    /// Apply Tanh activation
    pub fn tanh(tensor: &Tensor) -> Tensor {
        let result = tensor.data.mapv(|x| x.tanh());
        Tensor {
            data: result,
            shape: tensor.shape.clone(),
            requires_grad: tensor.requires_grad,
            grad: None,
        }
    }

    /// Apply Softmax activation along the last axis
    pub fn softmax(tensor: &Tensor) -> Result<Tensor, TensorError> {
        let axis = tensor.shape.0.len() - 1;

        // Compute exp(x - max) for numerical stability
        let max = tensor
            .data
            .fold_axis(Axis(axis), f32::NEG_INFINITY, |&a, &b| a.max(b));
        let exp_values = &tensor.data - &max.insert_axis(Axis(axis));
        let exp_values = exp_values.mapv(|x| x.exp());

        // Sum along axis
        let sum = exp_values.sum_axis(Axis(axis)).insert_axis(Axis(axis));
        let result = exp_values / sum;

        Ok(Tensor {
            data: result,
            shape: tensor.shape.clone(),
            requires_grad: tensor.requires_grad,
            grad: None,
        })
    }

    /// Sum all elements
    pub fn sum(tensor: &Tensor) -> f32 {
        tensor.data.sum()
    }

    /// Mean of all elements
    pub fn mean(tensor: &Tensor) -> f32 {
        tensor.data.mean().unwrap_or(0.0)
    }

    /// Max element
    pub fn max(tensor: &Tensor) -> f32 {
        tensor.data.fold(f32::NEG_INFINITY, |a, &b| a.max(b))
    }

    /// Min element
    pub fn min(tensor: &Tensor) -> f32 {
        tensor.data.fold(f32::INFINITY, |a, &b| a.min(b))
    }

    /// Compute L2 norm
    pub fn norm(tensor: &Tensor) -> f32 {
        let sum_squares = tensor.data.mapv(|x| x * x).sum();
        sum_squares.sqrt()
    }

    /// Convolution 2D operation (simplified)
    pub fn conv2d(
        input: &Tensor,
        kernel: &Tensor,
        stride: (usize, usize),
        padding: (usize, usize),
    ) -> Result<Tensor, TensorError> {
        // This is a simplified implementation
        // Full implementation would handle multiple channels, batches, etc.

        let input_shape = &input.shape.0;
        let kernel_shape = &kernel.shape.0;

        if input_shape.len() != 4 || kernel_shape.len() != 4 {
            return Err(TensorError::InvalidShape(
                "Conv2d expects 4D tensors (batch, channel, height, width)".to_string(),
            ));
        }

        // Calculate output dimensions
        let out_h = (input_shape[2] + 2 * padding.0 - kernel_shape[2]) / stride.0 + 1;
        let out_w = (input_shape[3] + 2 * padding.1 - kernel_shape[3]) / stride.1 + 1;

        let output_shape = vec![input_shape[0], kernel_shape[0], out_h, out_w];

        // Create output tensor (simplified - just zeros for now)
        // Full implementation would perform actual convolution
        Ok(Tensor::zeros(output_shape))
    }

    /// Max pooling 2D
    pub fn maxpool2d(
        input: &Tensor,
        kernel_size: (usize, usize),
        stride: (usize, usize),
    ) -> Result<Tensor, TensorError> {
        let input_shape = &input.shape.0;

        if input_shape.len() != 4 {
            return Err(TensorError::InvalidShape(
                "MaxPool2d expects 4D tensor".to_string(),
            ));
        }

        // Calculate output dimensions
        let out_h = (input_shape[2] - kernel_size.0) / stride.0 + 1;
        let out_w = (input_shape[3] - kernel_size.1) / stride.1 + 1;

        let output_shape = vec![input_shape[0], input_shape[1], out_h, out_w];

        // Create output tensor (simplified - just zeros for now)
        // Full implementation would perform actual max pooling
        Ok(Tensor::zeros(output_shape))
    }

    /// Batch normalization
    pub fn batch_norm(
        input: &Tensor,
        gamma: &Tensor,
        beta: &Tensor,
        eps: f32,
    ) -> Result<Tensor, TensorError> {
        // Compute mean and variance along batch dimension
        let mean = Self::mean(input);
        let variance = input
            .data
            .mapv(|x| (x - mean).powi(2))
            .mean()
            .unwrap_or(0.0);

        // Normalize
        let normalized = input.data.mapv(|x| (x - mean) / (variance + eps).sqrt());

        // Scale and shift
        let scaled = &normalized * &gamma.data + &beta.data;

        Ok(Tensor {
            data: scaled,
            shape: input.shape.clone(),
            requires_grad: input.requires_grad || gamma.requires_grad || beta.requires_grad,
            grad: None,
        })
    }

    /// Dropout (for training)
    pub fn dropout(tensor: &Tensor, p: f32, training: bool) -> Tensor {
        if !training || p == 0.0 {
            return tensor.clone();
        }

        use ndarray_rand::rand_distr::Uniform;
        use ndarray_rand::RandomExt;

        let mask = ArrayD::random(tensor.data.raw_dim(), Uniform::new(0.0, 1.0));
        let mask = mask.mapv(|x| if x > p { 1.0 / (1.0 - p) } else { 0.0 });

        Tensor {
            data: &tensor.data * &mask,
            shape: tensor.shape.clone(),
            requires_grad: tensor.requires_grad,
            grad: None,
        }
    }
}
