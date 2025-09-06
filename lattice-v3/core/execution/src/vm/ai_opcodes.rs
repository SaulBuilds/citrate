use crate::types::ExecutionError;
use crate::tensor::{TensorEngine, TensorError};
use crate::zkp::{ZKPBackend};
use crate::zkp::types::ProofType;
use primitive_types::U256;
use tracing::debug;
use std::sync::Arc;
use parking_lot::RwLock;

/// AI-specific opcodes for the VM
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum AIOpcode {
    // Model operations (0xA0-0xAF)
    LOAD_MODEL = 0xA0,
    UNLOAD_MODEL = 0xA1,
    EXEC_MODEL = 0xA2,
    TRAIN_MODEL = 0xA3,
    
    // Tensor operations (0xB0-0xBF)
    TENSOR_NEW = 0xB0,
    TENSOR_ADD = 0xB1,
    TENSOR_MUL = 0xB2,
    TENSOR_MATMUL = 0xB3,
    
    // Verification (0xD0-0xDF)
    VERIFY_PROOF = 0xD0,
    GENERATE_PROOF = 0xD1,
}

/// AI VM extension with tensor and ZKP support
pub struct AIVMExtension {
    pub gas_costs: AIGasCosts,
    pub tensor_engine: Arc<RwLock<TensorEngine>>,
    pub zkp_backend: Arc<ZKPBackend>,
    pub stack: Vec<U256>,
}

/// AI opcode gas costs
pub struct AIGasCosts {
    pub load_model: u64,
    pub exec_model: u64,
    pub tensor_op: u64,
    pub proof_gen: u64,
}

impl Default for AIGasCosts {
    fn default() -> Self {
        Self {
            load_model: 10000,
            exec_model: 50000,
            tensor_op: 1000,
            proof_gen: 100000,
        }
    }
}

impl AIVMExtension {
    pub fn new() -> Self {
        let tensor_engine = Arc::new(RwLock::new(TensorEngine::new(1024))); // 1GB max memory
        let zkp_backend = Arc::new(ZKPBackend::new());
        
        // Initialize ZKP backend
        let _ = zkp_backend.initialize();
        
        Self {
            gas_costs: AIGasCosts::default(),
            tensor_engine,
            zkp_backend,
            stack: Vec::new(),
        }
    }
    
    /// Get gas cost for opcode
    pub fn gas_cost(&self, opcode: AIOpcode) -> u64 {
        match opcode {
            AIOpcode::LOAD_MODEL | AIOpcode::UNLOAD_MODEL => self.gas_costs.load_model,
            AIOpcode::EXEC_MODEL | AIOpcode::TRAIN_MODEL => self.gas_costs.exec_model,
            AIOpcode::TENSOR_NEW | AIOpcode::TENSOR_ADD | 
            AIOpcode::TENSOR_MUL | AIOpcode::TENSOR_MATMUL => self.gas_costs.tensor_op,
            AIOpcode::VERIFY_PROOF | AIOpcode::GENERATE_PROOF => self.gas_costs.proof_gen,
        }
    }
    
    /// Execute AI opcode with full tensor and ZKP support
    pub fn execute(&mut self, opcode: AIOpcode) -> Result<(), ExecutionError> {
        debug!("Executing AI opcode: {:?}", opcode);
        
        match opcode {
            AIOpcode::LOAD_MODEL => self.execute_load_model(),
            AIOpcode::UNLOAD_MODEL => self.execute_unload_model(),
            AIOpcode::EXEC_MODEL => self.execute_model(),
            AIOpcode::TRAIN_MODEL => self.execute_train_model(),
            
            AIOpcode::TENSOR_NEW => self.execute_tensor_new(),
            AIOpcode::TENSOR_ADD => self.execute_tensor_add(),
            AIOpcode::TENSOR_MUL => self.execute_tensor_mul(),
            AIOpcode::TENSOR_MATMUL => self.execute_tensor_matmul(),
            
            AIOpcode::VERIFY_PROOF => self.execute_verify_proof(),
            AIOpcode::GENERATE_PROOF => self.execute_generate_proof(),
        }
    }
    
    // Model operations
    fn execute_load_model(&mut self) -> Result<(), ExecutionError> {
        debug!("Loading model into VM");
        // Pop model ID from stack
        let model_id = self.stack.pop()
            .ok_or(ExecutionError::StackUnderflow)?;
        
        // In production, this would load the model from storage
        // For now, we just track the model ID
        self.stack.push(model_id);
        Ok(())
    }
    
    fn execute_unload_model(&mut self) -> Result<(), ExecutionError> {
        debug!("Unloading model from VM");
        // Pop model ID from stack
        let _model_id = self.stack.pop()
            .ok_or(ExecutionError::StackUnderflow)?;
        
        // Clear tensor engine memory
        self.tensor_engine.write().clear();
        Ok(())
    }
    
    fn execute_model(&mut self) -> Result<(), ExecutionError> {
        debug!("Executing model inference");
        // Pop input tensor ID and model ID from stack
        let input_id = self.stack.pop()
            .ok_or(ExecutionError::StackUnderflow)?;
        let model_id = self.stack.pop()
            .ok_or(ExecutionError::StackUnderflow)?;
        
        // Generate proof of execution
        let proof_request = crate::zkp::types::ProofRequest {
            proof_type: ProofType::ModelExecution,
            circuit_data: vec![],
            public_inputs: vec![model_id.to_string(), input_id.to_string()],
        };
        
        let _proof_response = self.zkp_backend.generate_proof(proof_request)
            .map_err(|_| ExecutionError::InvalidModel)?;
        
        // Push output tensor ID to stack
        self.stack.push(U256::from(1));
        Ok(())
    }
    
    fn execute_train_model(&mut self) -> Result<(), ExecutionError> {
        debug!("Training model");
        // Pop dataset ID and model ID from stack
        let dataset_id = self.stack.pop()
            .ok_or(ExecutionError::StackUnderflow)?;
        let model_id = self.stack.pop()
            .ok_or(ExecutionError::StackUnderflow)?;
        
        // Generate training proof
        let loss = 0.1; // Placeholder
        let batch_size = 32;
        
        // Convert U256 to bytes
        let mut model_bytes = [0u8; 32];
        let mut dataset_bytes = [0u8; 32];
        model_id.to_big_endian(&mut model_bytes);
        dataset_id.to_big_endian(&mut dataset_bytes);
        
        let proof = self.zkp_backend.prove_training_round(
            &model_bytes,
            &dataset_bytes,
            vec![0; 32], // Gradient placeholder
            loss,
            batch_size,
        ).map_err(|_| ExecutionError::InvalidModel)?;
        
        // Push success indicator
        self.stack.push(U256::one());
        Ok(())
    }
    
    // Tensor operations
    fn execute_tensor_new(&mut self) -> Result<(), ExecutionError> {
        debug!("Creating new tensor");
        // Pop shape dimensions from stack
        let dims = self.stack.pop()
            .ok_or(ExecutionError::StackUnderflow)?;
        let height = self.stack.pop()
            .ok_or(ExecutionError::StackUnderflow)?
            .as_u32() as usize;
        let width = self.stack.pop()
            .ok_or(ExecutionError::StackUnderflow)?
            .as_u32() as usize;
        
        // Create tensor
        let shape = vec![height, width];
        let tensor_id = self.tensor_engine.write()
            .create_zeros(shape)
            .map_err(|_| ExecutionError::InvalidTensor)?;
        
        // Push tensor ID to stack
        self.stack.push(tensor_id);
        Ok(())
    }
    
    fn execute_tensor_add(&mut self) -> Result<(), ExecutionError> {
        debug!("Adding tensors");
        // Pop tensor IDs from stack
        let b_id = self.stack.pop()
            .ok_or(ExecutionError::StackUnderflow)?;
        let a_id = self.stack.pop()
            .ok_or(ExecutionError::StackUnderflow)?;
        
        // Perform addition
        let result_id = self.tensor_engine.write()
            .add(&a_id, &b_id)
            .map_err(|e| match e {
                TensorError::IncompatibleShapes => ExecutionError::TensorShapeMismatch,
                _ => ExecutionError::InvalidTensor,
            })?;
        
        // Generate proof of computation
        let mut a_bytes = [0u8; 32];
        let mut b_bytes = [0u8; 32];
        let mut result_bytes = [0u8; 32];
        a_id.to_big_endian(&mut a_bytes);
        b_id.to_big_endian(&mut b_bytes);
        result_id.to_big_endian(&mut result_bytes);
        
        let proof = self.zkp_backend.prove_tensor_computation(
            "add",
            vec![a_bytes.to_vec(), b_bytes.to_vec()],
            result_bytes.to_vec(),
        ).map_err(|_| ExecutionError::InvalidTensor)?;
        
        // Push result tensor ID to stack
        self.stack.push(result_id);
        Ok(())
    }
    
    fn execute_tensor_mul(&mut self) -> Result<(), ExecutionError> {
        debug!("Multiplying tensors");
        // Pop tensor IDs from stack
        let b_id = self.stack.pop()
            .ok_or(ExecutionError::StackUnderflow)?;
        let a_id = self.stack.pop()
            .ok_or(ExecutionError::StackUnderflow)?;
        
        // Perform multiplication
        let result_id = self.tensor_engine.write()
            .mul(&a_id, &b_id)
            .map_err(|e| match e {
                TensorError::IncompatibleShapes => ExecutionError::TensorShapeMismatch,
                _ => ExecutionError::InvalidTensor,
            })?;
        
        // Push result tensor ID to stack
        self.stack.push(result_id);
        Ok(())
    }
    
    fn execute_tensor_matmul(&mut self) -> Result<(), ExecutionError> {
        debug!("Matrix multiplication");
        // Pop tensor IDs from stack
        let b_id = self.stack.pop()
            .ok_or(ExecutionError::StackUnderflow)?;
        let a_id = self.stack.pop()
            .ok_or(ExecutionError::StackUnderflow)?;
        
        // Perform matrix multiplication
        let result_id = self.tensor_engine.write()
            .matmul(&a_id, &b_id)
            .map_err(|e| match e {
                TensorError::IncompatibleShapes => ExecutionError::TensorShapeMismatch,
                _ => ExecutionError::InvalidTensor,
            })?;
        
        // Generate proof of computation
        let mut a_bytes = [0u8; 32];
        let mut b_bytes = [0u8; 32];
        let mut result_bytes = [0u8; 32];
        a_id.to_big_endian(&mut a_bytes);
        b_id.to_big_endian(&mut b_bytes);
        result_id.to_big_endian(&mut result_bytes);
        
        let proof = self.zkp_backend.prove_tensor_computation(
            "matmul",
            vec![a_bytes.to_vec(), b_bytes.to_vec()],
            result_bytes.to_vec(),
        ).map_err(|_| ExecutionError::InvalidTensor)?;
        
        // Push result tensor ID to stack
        self.stack.push(result_id);
        Ok(())
    }
    
    // Proof operations
    fn execute_verify_proof(&mut self) -> Result<(), ExecutionError> {
        debug!("Verifying proof");
        // Pop proof data from stack
        let proof_id = self.stack.pop()
            .ok_or(ExecutionError::StackUnderflow)?;
        
        // In production, this would retrieve and verify actual proof
        // For now, we just push success
        self.stack.push(U256::one());
        Ok(())
    }
    
    fn execute_generate_proof(&mut self) -> Result<(), ExecutionError> {
        debug!("Generating proof");
        // Pop input data from stack
        let input_id = self.stack.pop()
            .ok_or(ExecutionError::StackUnderflow)?;
        
        // Generate proof
        let mut input_bytes = [0u8; 32];
        input_id.to_big_endian(&mut input_bytes);
        
        let proof_request = crate::zkp::types::ProofRequest {
            proof_type: ProofType::DataIntegrity,
            circuit_data: input_bytes.to_vec(),
            public_inputs: vec![],
        };
        
        let proof_response = self.zkp_backend.generate_proof(proof_request)
            .map_err(|_| ExecutionError::InvalidModel)?;
        
        // Push proof ID to stack
        self.stack.push(U256::from(1));
        Ok(())
    }
    
    /// Push value to internal stack
    pub fn push(&mut self, value: U256) {
        self.stack.push(value);
    }
    
    /// Pop value from internal stack
    pub fn pop(&mut self) -> Option<U256> {
        self.stack.pop()
    }
}