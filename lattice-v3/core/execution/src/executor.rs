use crate::state::StateDB;
use crate::types::{
    Address, ExecutionError, GasSchedule, Log, TransactionReceipt, TransactionType,
    ModelId, ModelState, ModelMetadata, AccessPolicy, TrainingJob, JobId, JobStatus,
};
use lattice_consensus::types::{Block, Transaction, Hash, VrfProof};
use primitive_types::U256;
use std::sync::Arc;
use tracing::{debug, info, warn, error};

/// Execution context for a transaction
pub struct ExecutionContext {
    pub block_number: u64,
    pub block_hash: Hash,
    pub timestamp: u64,
    pub gas_limit: u64,
    pub gas_used: u64,
    pub gas_price: u64,
    pub origin: Address,
    pub logs: Vec<Log>,
    pub output: Vec<u8>,
}

impl ExecutionContext {
    pub fn new(block: &Block, tx: &Transaction) -> Self {
        Self {
            block_number: block.header.height,
            block_hash: block.hash(),
            timestamp: block.header.timestamp,
            gas_limit: tx.gas_limit,
            gas_used: 0,
            gas_price: tx.gas_price,
            origin: Address::from_public_key(&tx.from),
            logs: Vec::new(),
            output: Vec::new(),
        }
    }
    
    /// Consume gas
    pub fn use_gas(&mut self, amount: u64) -> Result<(), ExecutionError> {
        if self.gas_used + amount > self.gas_limit {
            return Err(ExecutionError::OutOfGas);
        }
        self.gas_used += amount;
        Ok(())
    }
    
    /// Add log
    pub fn add_log(&mut self, log: Log) {
        self.logs.push(log);
    }
}

/// Transaction executor
pub struct Executor {
    state_db: Arc<StateDB>,
    state_store: Option<Arc<dyn StateStoreTrait>>,
    gas_schedule: GasSchedule,
}

/// Trait for state storage to avoid circular dependency
pub trait StateStoreTrait: Send + Sync {
    fn put_account(&self, address: &Address, account: &crate::types::AccountState) -> anyhow::Result<()>;
    fn get_account(&self, address: &Address) -> anyhow::Result<Option<crate::types::AccountState>>;
    fn put_code(&self, code_hash: &Hash, code: &[u8]) -> anyhow::Result<()>;
}

impl Executor {
    pub fn new(state_db: Arc<StateDB>) -> Self {
        Self {
            state_db,
            state_store: None,
            gas_schedule: GasSchedule::default(),
        }
    }
    
    pub fn with_storage<S: StateStoreTrait + 'static>(state_db: Arc<StateDB>, state_store: Option<Arc<S>>) -> Self {
        Self {
            state_db,
            state_store: state_store.map(|s| s as Arc<dyn StateStoreTrait>),
            gas_schedule: GasSchedule::default(),
        }
    }
    
    /// Get reference to state database
    pub fn state_db(&self) -> &Arc<StateDB> {
        &self.state_db
    }
    
    /// Get account balance
    pub fn get_balance(&self, address: &Address) -> U256 {
        // Try to load from storage first if available
        if let Some(store) = &self.state_store {
            if let Ok(Some(account)) = store.get_account(address) {
                // Update in-memory state
                self.state_db.accounts.set_account(*address, account.clone());
                return account.balance;
            }
        }
        
        self.state_db.accounts.get_balance(address)
    }
    
    /// Get account nonce
    pub fn get_nonce(&self, address: &Address) -> u64 {
        self.state_db.accounts.get_nonce(address)
    }
    
    /// Get contract code hash
    pub fn get_code_hash(&self, address: &Address) -> Hash {
        self.state_db.accounts.get_code_hash(address)
    }
    
    /// Set account balance
    pub fn set_balance(&self, address: &Address, balance: U256) {
        self.state_db.accounts.set_balance(*address, balance);
        
        // Persist to storage if available
        if let Some(store) = &self.state_store {
            let account = self.state_db.accounts.get_account(address);
            if let Err(e) = store.put_account(address, &account) {
                error!("Failed to persist account balance: {}", e);
            }
        }
    }
    
    /// Set account nonce
    pub fn set_nonce(&self, address: &Address, nonce: u64) {
        self.state_db.accounts.set_nonce(*address, nonce);
        
        // Persist to storage if available
        if let Some(store) = &self.state_store {
            let account = self.state_db.accounts.get_account(address);
            if let Err(e) = store.put_account(address, &account) {
                error!("Failed to persist account nonce: {}", e);
            }
        }
    }
    
    /// Set contract code
    pub fn set_code(&self, address: &Address, code: Vec<u8>) {
        let code_hash = self.state_db.set_code(*address, code.clone());
        self.state_db.accounts.set_code_hash(*address, code_hash);
        
        // Persist to storage if available
        if let Some(store) = &self.state_store {
            if let Err(e) = store.put_code(&code_hash, &code) {
                error!("Failed to persist contract code: {}", e);
            }
            
            let account = self.state_db.accounts.get_account(address);
            if let Err(e) = store.put_account(address, &account) {
                error!("Failed to persist account code hash: {}", e);
            }
        }
    }
    
    /// Calculate state root
    pub fn calculate_state_root(&self) -> Hash {
        self.state_db.calculate_state_root()
    }
    
    /// Execute a transaction
    pub async fn execute_transaction(
        &self,
        block: &Block,
        tx: &Transaction,
    ) -> Result<TransactionReceipt, ExecutionError> {
        let mut context = ExecutionContext::new(block, tx);
        let from = Address::from_public_key(&tx.from);
        
        // Create snapshot for potential rollback
        let snapshot = self.state_db.snapshot();
        
        // Validate and update nonce
        self.state_db.accounts.check_and_increment_nonce(&from, tx.nonce)?;
        
        // Check balance for gas
        let gas_cost = U256::from(tx.gas_limit) * U256::from(tx.gas_price);
        let balance = self.state_db.accounts.get_balance(&from);
        if balance < gas_cost + U256::from(tx.value) {
            self.state_db.restore(snapshot);
            return Err(ExecutionError::InsufficientBalance {
                need: gas_cost + U256::from(tx.value),
                have: balance,
            });
        }
        
        // Deduct gas cost upfront
        self.state_db.accounts.set_balance(from, balance - gas_cost);
        
        // Parse and execute transaction type
        let tx_type = self.parse_transaction_type(tx)?;
        let result = self.execute_transaction_type(tx_type, &mut context, from).await;
        
        // Handle execution result
        let status = match result {
            Ok(()) => {
                // Refund unused gas
                let refund = U256::from(tx.gas_limit - context.gas_used) * U256::from(tx.gas_price);
                let balance = self.state_db.accounts.get_balance(&from);
                self.state_db.accounts.set_balance(from, balance + refund);
                true
            }
            Err(e) => {
                warn!("Transaction execution failed: {}", e);
                // Rollback state changes but keep gas consumed
                self.state_db.restore(snapshot);
                self.state_db.accounts.check_and_increment_nonce(&from, tx.nonce)?;
                self.state_db.accounts.set_balance(from, balance - gas_cost);
                false
            }
        };
        
        // Create receipt
        let receipt = TransactionReceipt {
            tx_hash: tx.hash,
            block_hash: block.hash(),
            block_number: block.header.height,
            from,
            to: tx.to.map(|pk| crate::address_utils::normalize_address(&pk)),
            gas_used: context.gas_used,
            status,
            logs: context.logs,
            output: context.output,
        };
        
        info!("Transaction {} executed: status={}, gas_used={}", 
              tx.hash, status, context.gas_used);
        
        Ok(receipt)
    }
    
    /// Parse transaction data into type
    fn parse_transaction_type(&self, tx: &Transaction) -> Result<TransactionType, ExecutionError> {
        // Simple parsing based on transaction data
        // In production, this would use proper ABI encoding/decoding
        
        if tx.data.is_empty() {
            // Simple transfer
            // Use proper address normalization to handle both formats
            let to = tx.to
                .map(|pk| crate::address_utils::normalize_address(&pk))
                .ok_or(ExecutionError::InvalidInput)?;
            
            Ok(TransactionType::Transfer {
                to,
                value: U256::from(tx.value),
            })
        } else if tx.to.is_none() {
            // Contract deployment
            Ok(TransactionType::Deploy {
                code: tx.data.clone(),
                init_data: vec![],
            })
        } else {
            // Contract call or special operation
            let to = Address::from_public_key(&tx.to.unwrap());
            
            // Check first 4 bytes for function selector
            if tx.data.len() >= 4 {
                match &tx.data[0..4] {
                    [0x01, 0x00, 0x00, 0x00] => {
                        // Register model
                        self.parse_register_model(&tx.data[4..])
                    }
                    [0x02, 0x00, 0x00, 0x00] => {
                        // Inference request
                        self.parse_inference_request(&tx.data[4..])
                    }
                    _ => {
                        // Generic call
                        Ok(TransactionType::Call {
                            to,
                            data: tx.data.clone(),
                            value: U256::from(tx.value),
                        })
                    }
                }
            } else {
                Ok(TransactionType::Call {
                    to,
                    data: tx.data.clone(),
                    value: U256::from(tx.value),
                })
            }
        }
    }
    
    /// Parse register model transaction
    fn parse_register_model(&self, data: &[u8]) -> Result<TransactionType, ExecutionError> {
        // Simplified parsing - in production would use proper encoding
        if data.len() < 32 {
            return Err(ExecutionError::InvalidInput);
        }
        
        let model_hash = Hash::new(data[0..32].try_into().unwrap());
        
        Ok(TransactionType::RegisterModel {
            model_hash,
            metadata: ModelMetadata {
                name: "Model".to_string(),
                version: "1.0".to_string(),
                description: "AI Model".to_string(),
                framework: "PyTorch".to_string(),
                input_shape: vec![1, 224, 224, 3],
                output_shape: vec![1, 1000],
                size_bytes: 1_000_000,
                created_at: 0,
            },
            access_policy: AccessPolicy::Public,
        })
    }
    
    /// Parse inference request
    fn parse_inference_request(&self, data: &[u8]) -> Result<TransactionType, ExecutionError> {
        if data.len() < 32 {
            return Err(ExecutionError::InvalidInput);
        }
        
        let model_id = ModelId(Hash::new(data[0..32].try_into().unwrap()));
        
        Ok(TransactionType::InferenceRequest {
            model_id,
            input_data: data[32..].to_vec(),
            max_gas: 1_000_000,
        })
    }
    
    /// Execute transaction type
    async fn execute_transaction_type(
        &self,
        tx_type: TransactionType,
        context: &mut ExecutionContext,
        from: Address,
    ) -> Result<(), ExecutionError> {
        match tx_type {
            TransactionType::Transfer { to, value } => {
                self.execute_transfer(from, to, value, context).await
            }
            
            TransactionType::Deploy { code, init_data } => {
                self.execute_deploy(from, code, init_data, context).await
            }
            
            TransactionType::Call { to, data, value } => {
                self.execute_call(from, to, data, value, context).await
            }
            
            TransactionType::RegisterModel { model_hash, metadata, access_policy } => {
                self.execute_register_model(from, model_hash, metadata, access_policy, context).await
            }
            
            TransactionType::UpdateModel { model_id, new_version, changelog } => {
                self.execute_update_model(from, model_id, new_version, changelog, context).await
            }
            
            TransactionType::InferenceRequest { model_id, input_data, max_gas } => {
                self.execute_inference(from, model_id, input_data, max_gas, context).await
            }
            
            TransactionType::SubmitGradient { job_id, gradient_data, proof } => {
                self.execute_submit_gradient(from, job_id, gradient_data, proof, context).await
            }
        }
    }
    
    /// Execute transfer
    async fn execute_transfer(
        &self,
        from: Address,
        to: Address,
        value: U256,
        context: &mut ExecutionContext,
    ) -> Result<(), ExecutionError> {
        context.use_gas(self.gas_schedule.transfer)?;
        
        // Create recipient account if not exists
        self.state_db.accounts.create_account_if_not_exists(to);
        
        // Transfer value
        self.state_db.accounts.transfer(&from, &to, value)?;
        
        // Add transfer log
        context.add_log(Log {
            address: to,
            topics: vec![Hash::new(*b"Transfer000000000000000000000000")],
            data: {
                let mut bytes = [0u8; 32];
                value.to_big_endian(&mut bytes);
                bytes.to_vec()
            },
        });
        
        debug!("Transfer: {} -> {} : {}", from, to, value);
        Ok(())
    }
    
    /// Execute contract deployment
    async fn execute_deploy(
        &self,
        from: Address,
        code: Vec<u8>,
        _init_data: Vec<u8>,
        context: &mut ExecutionContext,
    ) -> Result<(), ExecutionError> {
        context.use_gas(self.gas_schedule.create)?;
        
        // Calculate contract address (simplified)
        let nonce = self.state_db.accounts.get_nonce(&from);
        use sha3::{Digest, Keccak256};
        let mut hasher = Keccak256::default();
        hasher.update(&from.0);
        hasher.update(&nonce.to_be_bytes());
        let hash = hasher.finalize();
        
        let mut contract_addr = [0u8; 20];
        contract_addr.copy_from_slice(&hash[12..32]);
        let contract_address = Address(contract_addr);
        
        // Create contract account
        self.state_db.accounts.create_account_if_not_exists(contract_address);
        
        // Store code
        let code_hash = self.state_db.set_code(contract_address, code);
        
        // Set contract address in output
        context.output = contract_address.0.to_vec();
        
        // Add deployment log
        context.add_log(Log {
            address: contract_address,
            topics: vec![Hash::new(*b"ContractDeployed0000000000000000")],
            data: vec![], // Hash bytes not directly accessible
        });
        
        info!("Contract deployed at: {}", contract_address);
        Ok(())
    }
    
    /// Execute contract call with AI opcode support
    async fn execute_call(
        &self,
        from: Address,
        to: Address,
        data: Vec<u8>,
        value: U256,
        context: &mut ExecutionContext,
    ) -> Result<(), ExecutionError> {
        context.use_gas(self.gas_schedule.call)?;
        
        // Transfer value if any
        if value > U256::zero() {
            self.state_db.accounts.transfer(&from, &to, value)?;
        }
        
        // Execute contract code with VM
        if let Some(code) = self.state_db.get_code(&self.state_db.accounts.get_code_hash(&to)) {
            // Check for AI opcodes in the bytecode
            let ai_result = self.scan_and_execute_ai_opcodes(&code, &data, context).await?;
            
            if ai_result.is_some() {
                // AI operation was executed
                context.output = ai_result.unwrap();
            } else {
                // Standard EVM execution
                debug!("Executing contract at {} with {} bytes of code", to, code.len());
                
                // For now, just consume some gas based on code size
                let execution_gas = (code.len() as u64 / 32) * self.gas_schedule.sload;
                context.use_gas(execution_gas)?;
            }
            
            // Add execution log
            context.add_log(Log {
                address: to,
                topics: vec![
                    Hash::new(*b"ContractExecuted0000000000000000"),
                ],
                data: data.clone(),
            });
        }
        
        Ok(())
    }
    
    /// Scan bytecode for AI opcodes and execute them
    async fn scan_and_execute_ai_opcodes(
        &self,
        code: &[u8],
        input: &[u8],
        context: &mut ExecutionContext,
    ) -> Result<Option<Vec<u8>>, ExecutionError> {
        // AI opcode definitions
        const TENSOR_OP: u8 = 0xf0;
        const MODEL_LOAD: u8 = 0xf1;
        const MODEL_EXEC: u8 = 0xf2;
        const ZK_PROVE: u8 = 0xf3;
        const ZK_VERIFY: u8 = 0xf4;
        
        for (i, &byte) in code.iter().enumerate() {
            match byte {
                TENSOR_OP => {
                    debug!("Executing TENSOR_OP at position {}", i);
                    context.use_gas(self.gas_schedule.tensor_op)?;
                    return Ok(Some(self.execute_tensor_operation(input, context).await?));
                }
                MODEL_LOAD => {
                    debug!("Executing MODEL_LOAD at position {}", i);
                    context.use_gas(self.gas_schedule.model_load)?;
                    return Ok(Some(self.execute_model_load(input, context).await?));
                }
                MODEL_EXEC => {
                    debug!("Executing MODEL_EXEC at position {}", i);
                    context.use_gas(self.gas_schedule.model_exec)?;
                    return Ok(Some(self.execute_model_execution(input, context).await?));
                }
                ZK_PROVE => {
                    debug!("Executing ZK_PROVE at position {}", i);
                    context.use_gas(self.gas_schedule.zk_prove)?;
                    return Ok(Some(self.execute_zk_prove(input, context).await?));
                }
                ZK_VERIFY => {
                    debug!("Executing ZK_VERIFY at position {}", i);
                    context.use_gas(self.gas_schedule.zk_verify)?;
                    return Ok(Some(self.execute_zk_verify(input, context).await?));
                }
                _ => continue,
            }
        }
        
        Ok(None)
    }
    
    /// Execute tensor operation
    async fn execute_tensor_operation(
        &self,
        input: &[u8],
        context: &mut ExecutionContext,
    ) -> Result<Vec<u8>, ExecutionError> {
        // Parse tensor operation from input
        if input.len() < 8 {
            return Err(ExecutionError::InvalidInput);
        }
        
        // Simulate tensor operation
        let op_type = input[0];
        let dimensions = u32::from_le_bytes([input[1], input[2], input[3], input[4]]);
        
        // Gas cost based on tensor dimensions
        let tensor_gas = dimensions as u64 * 100;
        context.use_gas(tensor_gas)?;
        
        info!("Tensor operation: type={}, dimensions={}", op_type, dimensions);
        
        // Return simulated result
        Ok(vec![0xf0, op_type, 0x01, 0x00])
    }
    
    /// Execute model loading
    async fn execute_model_load(
        &self,
        input: &[u8],
        context: &mut ExecutionContext,
    ) -> Result<Vec<u8>, ExecutionError> {
        if input.len() < 32 {
            return Err(ExecutionError::InvalidInput);
        }
        
        let model_hash = Hash::new(input[0..32].try_into().unwrap());
        let model_id = ModelId(model_hash);
        
        // Check if model exists
        let model = self.state_db.get_model(&model_id)
            .ok_or(ExecutionError::ModelNotFound(model_id))?;
        
        // Gas based on model size
        let load_gas = (model.metadata.size_bytes / 1024) as u64;
        context.use_gas(load_gas)?;
        
        info!("Model loaded: {:?}", model_id);
        
        // Return model handle
        Ok(model_hash.as_bytes().to_vec())
    }
    
    /// Execute model inference
    async fn execute_model_execution(
        &self,
        input: &[u8],
        context: &mut ExecutionContext,
    ) -> Result<Vec<u8>, ExecutionError> {
        if input.len() < 32 {
            return Err(ExecutionError::InvalidInput);
        }
        
        let model_hash = Hash::new(input[0..32].try_into().unwrap());
        let model_id = ModelId(model_hash);
        let inference_data = &input[32..];
        
        // Execute inference
        self.execute_inference(
            context.origin,
            model_id,
            inference_data.to_vec(),
            context.gas_limit - context.gas_used,
            context,
        ).await?;
        
        Ok(context.output.clone())
    }
    
    /// Execute ZK proof generation
    async fn execute_zk_prove(
        &self,
        input: &[u8],
        context: &mut ExecutionContext,
    ) -> Result<Vec<u8>, ExecutionError> {
        // Parse proof parameters
        if input.is_empty() {
            return Err(ExecutionError::InvalidInput);
        }
        
        // Simulate proof generation
        let proof_size = input.len().min(1024);
        let proof_gas = proof_size as u64 * 1000;
        context.use_gas(proof_gas)?;
        
        info!("ZK proof generated for {} bytes of input", input.len());
        
        // Return simulated proof
        Ok(vec![0xf3; 64])
    }
    
    /// Execute ZK proof verification
    async fn execute_zk_verify(
        &self,
        input: &[u8],
        context: &mut ExecutionContext,
    ) -> Result<Vec<u8>, ExecutionError> {
        // Parse proof and public inputs
        if input.len() < 64 {
            return Err(ExecutionError::InvalidInput);
        }
        
        let proof = &input[0..64];
        let public_inputs = &input[64..];
        
        // Simulate verification
        let verify_gas = 5000 + (public_inputs.len() as u64 * 10);
        context.use_gas(verify_gas)?;
        
        // Check if proof is valid (simplified)
        let is_valid = proof.iter().all(|&b| b == 0xf3);
        
        info!("ZK proof verification: valid={}", is_valid);
        
        // Return verification result
        Ok(vec![if is_valid { 0x01 } else { 0x00 }])
    }
    
    /// Execute model registration
    async fn execute_register_model(
        &self,
        from: Address,
        model_hash: Hash,
        mut metadata: ModelMetadata,
        access_policy: AccessPolicy,
        context: &mut ExecutionContext,
    ) -> Result<(), ExecutionError> {
        context.use_gas(self.gas_schedule.model_register)?;
        
        let model_id = ModelId(model_hash);
        metadata.created_at = context.timestamp;
        
        let model_state = ModelState {
            owner: from,
            model_hash,
            version: 1,
            metadata,
            access_policy,
            usage_stats: Default::default(),
        };
        
        self.state_db.register_model(model_id, model_state)?;
        
        // Add registration log
        context.add_log(Log {
            address: from,
            topics: vec![
                Hash::new(*b"ModelRegistered00000000000000000"),
                model_hash,
            ],
            data: vec![],
        });
        
        info!("Model registered: {:?} by {}", model_id, from);
        Ok(())
    }
    
    /// Execute model update
    async fn execute_update_model(
        &self,
        from: Address,
        model_id: ModelId,
        new_version: Hash,
        _changelog: String,
        context: &mut ExecutionContext,
    ) -> Result<(), ExecutionError> {
        context.use_gas(self.gas_schedule.model_update)?;
        
        let mut model = self.state_db.get_model(&model_id)
            .ok_or(ExecutionError::ModelNotFound(model_id))?;
        
        // Check ownership
        if model.owner != from {
            return Err(ExecutionError::AccessDenied);
        }
        
        // Update model
        model.model_hash = new_version;
        model.version += 1;
        
        self.state_db.update_model(model_id, model)?;
        
        info!("Model updated: {:?} to version {}", model_id, new_version);
        Ok(())
    }
    
    /// Execute inference request
    async fn execute_inference(
        &self,
        from: Address,
        model_id: ModelId,
        input_data: Vec<u8>,
        max_gas: u64,
        context: &mut ExecutionContext,
    ) -> Result<(), ExecutionError> {
        // Base gas cost
        context.use_gas(self.gas_schedule.inference_base)?;
        
        // Additional gas per MB of input
        let input_mb = (input_data.len() / 1_048_576) as u64;
        context.use_gas(self.gas_schedule.inference_per_mb * input_mb)?;
        
        // Check gas limit
        if context.gas_used > max_gas {
            return Err(ExecutionError::OutOfGas);
        }
        
        let mut model = self.state_db.get_model(&model_id)
            .ok_or(ExecutionError::ModelNotFound(model_id))?;
        
        // Check access policy
        match &model.access_policy {
            AccessPolicy::Public => {},
            AccessPolicy::Private if model.owner == from => {},
            AccessPolicy::Restricted(allowed) if allowed.contains(&from) => {},
            AccessPolicy::PayPerUse { fee } => {
                // Transfer fee to model owner
                self.state_db.accounts.transfer(&from, &model.owner, *fee)?;
                model.usage_stats.total_fees_earned += *fee;
            }
            _ => return Err(ExecutionError::AccessDenied),
        }
        
        // Update usage stats
        model.usage_stats.total_inferences += 1;
        model.usage_stats.total_gas_used += context.gas_used;
        model.usage_stats.last_used = context.timestamp;
        
        self.state_db.update_model(model_id, model)?;
        
        // Simulate inference output
        context.output = vec![0x01, 0x02, 0x03, 0x04];
        
        info!("Inference executed: model={:?}, from={}", model_id, from);
        Ok(())
    }
    
    /// Execute gradient submission
    async fn execute_submit_gradient(
        &self,
        from: Address,
        job_id: JobId,
        _gradient_data: Vec<u8>,
        _proof: Vec<u8>,
        context: &mut ExecutionContext,
    ) -> Result<(), ExecutionError> {
        context.use_gas(self.gas_schedule.training_submit)?;
        
        let mut job = self.state_db.get_training_job(&job_id)
            .ok_or(ExecutionError::Reverted("Job not found".to_string()))?;
        
        // Check job status
        if job.status != JobStatus::Active {
            return Err(ExecutionError::Reverted("Job not active".to_string()));
        }
        
        // Add participant if not already
        if !job.participants.contains(&from) {
            job.participants.push(from);
        }
        
        job.gradients_submitted += 1;
        
        // Check if job complete
        if job.gradients_submitted >= job.gradients_required {
            job.status = JobStatus::Completed;
            job.completed_at = Some(context.timestamp);
            
            // Distribute rewards
            let reward_per_participant = job.reward_pool / U256::from(job.participants.len());
            for participant in &job.participants {
                let balance = self.state_db.accounts.get_balance(participant);
                self.state_db.accounts.set_balance(*participant, balance + reward_per_participant);
            }
        }
        
        self.state_db.update_training_job(job_id, job)?;
        
        info!("Gradient submitted: job={:?}, from={}", job_id, from);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lattice_consensus::types::{BlockHeader, PublicKey, Signature};
    
    fn create_test_block() -> Block {
        Block {
            header: BlockHeader {
                version: 1,
                block_hash: Hash::default(),
                selected_parent_hash: Hash::default(),
                merge_parent_hashes: vec![],
                timestamp: 1000000,
                height: 100,
                blue_score: 0,
                blue_work: 0,
                pruning_point: Hash::default(),
                proposer_pubkey: PublicKey::new([0; 32]),
                vrf_reveal: VrfProof {
                    proof: vec![],
                    output: Hash::default(),
                },
            },
            state_root: Hash::default(),
            tx_root: Hash::default(),
            receipt_root: Hash::default(),
            artifact_root: Hash::default(),
            ghostdag_params: Default::default(),
            transactions: vec![],
            signature: Signature::new([0; 64]),
        }
    }
    
    fn create_test_tx(from: PublicKey, to: Option<PublicKey>, value: u64, nonce: u64) -> Transaction {
        Transaction {
            hash: Hash::new([1; 32]),
            nonce,
            from,
            to,
            value: value as u128,
            gas_limit: 100000,
            gas_price: 1000000000,
            data: vec![],
            signature: Signature::new([0; 64]),
            tx_type: None,
        }
    }
    
    #[tokio::test]
    async fn test_transfer_execution() {
        let state_db = Arc::new(StateDB::new());
        let executor = Executor::new(state_db.clone());
        
        let alice = PublicKey::new([1; 32]);
        let bob = PublicKey::new([2; 32]);
        let alice_addr = Address::from_public_key(&alice);
        let bob_addr = Address::from_public_key(&bob);
        
        // Setup alice with balance (needs enough for gas + transfer value)
        state_db.accounts.set_balance(alice_addr, U256::from(1_000_000_000_000_000u128));
        
        let block = create_test_block();
        let tx = create_test_tx(alice, Some(bob), 1000, 0);
        
        let receipt = executor.execute_transaction(&block, &tx).await.unwrap();
        
        assert!(receipt.status);
        assert_eq!(state_db.accounts.get_balance(&bob_addr), U256::from(1000));
    }
}
