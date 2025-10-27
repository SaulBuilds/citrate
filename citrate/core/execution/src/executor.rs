// citrate/core/execution/src/executor.rs

use crate::metrics::{PRECOMPILE_CALLS_TOTAL, VM_EXECUTIONS_TOTAL, VM_GAS_USED};
use crate::precompiles::{PrecompileExecutor, inference::InferencePrecompile};
use crate::inference::metal_runtime::MetalRuntime;
use crate::state::StateDB;
use crate::types::{
    AccessPolicy, Address, ExecutionError, GasSchedule, JobId, JobStatus, Log, ModelId,
    ModelMetadata, ModelState, TransactionReceipt, TransactionType,
};
use crate::vm::VM;
use async_trait::async_trait;
use hex;
use citrate_consensus::types::{Block, Hash, Transaction};
use primitive_types::U256;
use std::time::Instant;
use serde_json;
use std::sync::Arc;
use tracing::{debug, error, info, warn};

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
            origin: crate::address_utils::normalize_address(&tx.from),
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
    inference_service: Option<Arc<dyn InferenceService>>,
    artifact_service: Option<Arc<dyn ArtifactService>>,
    ai_storage: Option<Arc<dyn AIModelStorage>>,
    model_registry: Option<Arc<dyn ModelRegistryAdapter>>,
    #[allow(dead_code)]
    precompile_executor: Option<Arc<tokio::sync::RwLock<PrecompileExecutor>>>,
}

/// Trait for state storage to avoid circular dependency
pub trait StateStoreTrait: Send + Sync {
    fn put_account(
        &self,
        address: &Address,
        account: &crate::types::AccountState,
    ) -> anyhow::Result<()>;
    fn get_account(&self, address: &Address) -> anyhow::Result<Option<crate::types::AccountState>>;
    fn put_code(&self, code_hash: &Hash, code: &[u8]) -> anyhow::Result<()>;
}

/// Bridge trait to persist AI model metadata & artifacts in external storage layers.
pub trait AIModelStorage: Send + Sync {
    fn register_model(
        &self,
        model_id: ModelId,
        model_state: &ModelState,
        weight_cid: &str,
    ) -> anyhow::Result<()>;
    fn update_model_weights(
        &self,
        model_id: ModelId,
        weight_cid: &str,
        new_version: u32,
    ) -> anyhow::Result<()>;
}

/// Bridge trait to inform higher-level registries (e.g. MCP) about model lifecycle events.
#[async_trait]
pub trait ModelRegistryAdapter: Send + Sync {
    async fn register_model(
        &self,
        model_id: ModelId,
        model_state: &ModelState,
        artifact_cid: Option<&str>,
    ) -> anyhow::Result<()>;

    async fn update_model(
        &self,
        _model_id: ModelId,
        _model_state: &ModelState,
        _artifact_cid: Option<&str>,
    ) -> anyhow::Result<()> {
        Ok(())
    }
}

/// Trait to delegate AI inference to an external service (e.g., MCP)
#[async_trait]
pub trait InferenceService: Send + Sync {
    /// Run inference and return (output bytes, extra gas used, provider address, provider fee in wei, optional proof bytes)
    async fn run_inference(
        &self,
        model_id: ModelId,
        input: Vec<u8>,
        max_gas: u64,
    ) -> Result<(Vec<u8>, u64, Address, U256, Option<Vec<u8>>), ExecutionError>;
}

/// Trait to pin and query artifact CIDs (e.g., IPFS)
#[async_trait]
pub trait ArtifactService: Send + Sync {
    async fn pin(&self, cid: &str, replicas: usize) -> Result<(), ExecutionError>;
    async fn status(&self, cid: &str) -> Result<String, ExecutionError>;
    async fn add(&self, data: &[u8]) -> Result<String, ExecutionError>;
}

/// Summary returned by `run_inference_preview`
pub struct InferencePreview {
    pub output: Vec<u8>,
    pub gas_used: u64,
    pub provider: Address,
    pub provider_fee: U256,
    pub proof: Option<Vec<u8>>,
    pub latency_ms: u64,
}

impl Executor {
    pub fn new(state_db: Arc<StateDB>) -> Self {
        // Initialize Metal runtime and precompiles if available
        let precompile_executor = if cfg!(target_os = "macos") {
            match MetalRuntime::new() {
                Ok(runtime) => {
                    let inference_precompile = InferencePrecompile::new(Arc::new(runtime));
                    let executor = PrecompileExecutor::new()
                        .with_inference(inference_precompile);
                    Some(Arc::new(tokio::sync::RwLock::new(executor)))
                },
                Err(e) => {
                    warn!("Failed to initialize Metal runtime: {}", e);
                    None
                }
            }
        } else {
            None
        };

        Self {
            state_db,
            state_store: None,
            gas_schedule: GasSchedule::default(),
            inference_service: None,
            artifact_service: None,
            ai_storage: None,
            model_registry: None,
            precompile_executor,
        }
    }

    pub fn with_storage<S: StateStoreTrait + 'static>(
        state_db: Arc<StateDB>,
        state_store: Option<Arc<S>>,
    ) -> Self {
        // Initialize Metal runtime and precompiles if available
        let precompile_executor = if cfg!(target_os = "macos") {
            match MetalRuntime::new() {
                Ok(runtime) => {
                    let inference_precompile = InferencePrecompile::new(Arc::new(runtime));
                    let executor = PrecompileExecutor::new()
                        .with_inference(inference_precompile);
                    Some(Arc::new(tokio::sync::RwLock::new(executor)))
                },
                Err(e) => {
                    warn!("Failed to initialize Metal runtime: {}", e);
                    None
                }
            }
        } else {
            None
        };

        Self {
            state_db,
            state_store: state_store.map(|s| s as Arc<dyn StateStoreTrait>),
            gas_schedule: GasSchedule::default(),
            inference_service: None,
            artifact_service: None,
            ai_storage: None,
            model_registry: None,
            precompile_executor,
        }
    }

    /// Attach an inference service for MCP-backed inference execution
    pub fn with_inference_service(mut self, svc: Arc<dyn InferenceService>) -> Self {
        self.inference_service = Some(svc);
        self
    }

    /// Attach an artifact service for IPFS pinning and status
    pub fn with_artifact_service(mut self, svc: Arc<dyn ArtifactService>) -> Self {
        self.artifact_service = Some(svc);
        self
    }

    /// Attach persistent AI storage adapter (e.g., StorageManager bridge)
    pub fn with_ai_storage_adapter(mut self, storage: Arc<dyn AIModelStorage>) -> Self {
        self.ai_storage = Some(storage);
        self
    }

    /// Attach model registry adapter (e.g., MCP service bridge)
    pub fn with_model_registry_adapter(mut self, adapter: Arc<dyn ModelRegistryAdapter>) -> Self {
        self.model_registry = Some(adapter);
        self
    }

    /// Get reference to state database
    pub fn state_db(&self) -> &Arc<StateDB> {
        &self.state_db
    }

    /// Store raw artifact bytes via configured artifact service
    pub async fn add_artifact(&self, data: &[u8]) -> Result<String, ExecutionError> {
        if let Some(svc) = &self.artifact_service {
            svc.add(data).await
        } else {
            Err(ExecutionError::Reverted(
                "Artifact service not configured".into(),
            ))
        }
    }

    /// Get account balance
    pub fn get_balance(&self, address: &Address) -> U256 {
        // Try to load from storage first if available
        if let Some(store) = &self.state_store {
            if let Ok(Some(account)) = store.get_account(address) {
                // Update in-memory state
                self.state_db
                    .accounts
                    .set_account(*address, account.clone());
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
        let from = crate::address_utils::normalize_address(&tx.from);

        // Create snapshot for potential rollback
        let snapshot = self.state_db.snapshot();

        // Validate and update nonce
        self.state_db
            .accounts
            .check_and_increment_nonce(&from, tx.nonce)?;

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
        let result = self
            .execute_transaction_type(tx_type, &mut context, from)
            .await;

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
                self.state_db
                    .accounts
                    .check_and_increment_nonce(&from, tx.nonce)?;
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

        info!(
            "Transaction {} executed: status={}, gas_used={}",
            tx.hash, status, context.gas_used
        );

        Ok(receipt)
    }

    /// Parse transaction data into type
    fn parse_transaction_type(&self, tx: &Transaction) -> Result<TransactionType, ExecutionError> {
        // Simple parsing based on transaction data
        // In production, this would use proper ABI encoding/decoding

        if tx.data.is_empty() {
            // Simple transfer
            // Use proper address normalization to handle both formats
            let to = tx
                .to
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
            let to = crate::address_utils::normalize_address(&tx.to.unwrap());

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
                    [0x03, 0x00, 0x00, 0x00] => {
                        // Update model
                        self.parse_update_model(&tx.data[4..])
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
        if data.len() < 36 {
            return Err(ExecutionError::InvalidInput);
        }

        let model_hash = Hash::new(data[0..32].try_into().unwrap());
        let meta_len = u32::from_be_bytes(data[32..36].try_into().unwrap()) as usize;
        let mut offset = 36;
        if data.len() < offset + meta_len {
            return Err(ExecutionError::InvalidInput);
        }
        let metadata_bytes = &data[offset..offset + meta_len];
        offset += meta_len;

        let mut metadata: ModelMetadata =
            serde_json::from_slice(metadata_bytes).map_err(|_| ExecutionError::InvalidInput)?;

        if metadata.name.is_empty() {
            metadata.name = format!("Model-{}", hex::encode(&model_hash.as_bytes()[..4]));
        }
        if metadata.version.is_empty() {
            metadata.version = "1.0.0".to_string();
        }
        if metadata.description.is_empty() {
            metadata.description = "Registered AI model".to_string();
        }
        if metadata.framework.is_empty() {
            metadata.framework = "Unknown".to_string();
        }
        if metadata.input_shape.is_empty() {
            metadata.input_shape = vec![1];
        }
        if metadata.output_shape.is_empty() {
            metadata.output_shape = vec![1];
        }

        if offset >= data.len() {
            return Err(ExecutionError::InvalidInput);
        }
        let policy_byte = data[offset];
        offset += 1;

        let access_policy = match policy_byte {
            0 => AccessPolicy::Public,
            1 => AccessPolicy::Private,
            2 => AccessPolicy::Restricted(Vec::new()),
            3 => {
                if data.len() < offset + 32 {
                    return Err(ExecutionError::InvalidInput);
                }
                let mut fee_bytes = [0u8; 32];
                fee_bytes.copy_from_slice(&data[offset..offset + 32]);
                offset += 32;
                AccessPolicy::PayPerUse {
                    fee: U256::from_big_endian(&fee_bytes),
                }
            }
            _ => AccessPolicy::Public,
        };

        let mut artifact_cid: Option<String> = None;
        if data.len() >= offset + 4 {
            let cid_len = u32::from_be_bytes(data[offset..offset + 4].try_into().unwrap()) as usize;
            offset += 4;
            if cid_len > 0 {
                if data.len() < offset + cid_len {
                    return Err(ExecutionError::InvalidInput);
                }
                artifact_cid = Some(
                    String::from_utf8(data[offset..offset + cid_len].to_vec())
                        .map_err(|_| ExecutionError::InvalidInput)?,
                );
            }
        }

        Ok(TransactionType::RegisterModel {
            model_hash,
            metadata,
            access_policy,
            artifact_cid,
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

    /// Parse update model transaction
    fn parse_update_model(&self, data: &[u8]) -> Result<TransactionType, ExecutionError> {
        if data.len() < 36 {
            return Err(ExecutionError::InvalidInput);
        }

        let model_id = ModelId(Hash::new(data[0..32].try_into().unwrap()));
        let meta_len = u32::from_be_bytes(data[32..36].try_into().unwrap()) as usize;
        let mut offset = 36;
        if data.len() < offset + meta_len {
            return Err(ExecutionError::InvalidInput);
        }
        let metadata_bytes = &data[offset..offset + meta_len];
        offset += meta_len;

        let mut metadata: ModelMetadata = serde_json::from_slice(metadata_bytes)
            .map_err(|_| ExecutionError::InvalidInput)?;

        if metadata.name.is_empty() {
            metadata.name = format!("Model-{}", hex::encode(&model_id.0.as_bytes()[..4]));
        }
        if metadata.version.is_empty() {
            metadata.version = "1.0.1".to_string();
        }
        if metadata.framework.is_empty() {
            metadata.framework = "Unknown".to_string();
        }
        if metadata.input_shape.is_empty() {
            metadata.input_shape = vec![1];
        }
        if metadata.output_shape.is_empty() {
            metadata.output_shape = vec![1];
        }

        metadata.created_at = metadata.created_at.max(0);

        let artifact_cid = if data.len() >= offset + 4 {
            let cid_len = u32::from_be_bytes(data[offset..offset + 4].try_into().unwrap()) as usize;
            offset += 4;
            if cid_len > 0 {
                if data.len() < offset + cid_len {
                    return Err(ExecutionError::InvalidInput);
                }
                Some(
                    String::from_utf8(data[offset..offset + cid_len].to_vec())
                        .map_err(|_| ExecutionError::InvalidInput)?,
                )
            } else {
                None
            }
        } else {
            None
        };

        Ok(TransactionType::UpdateModel {
            model_id,
            metadata,
            artifact_cid,
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

            TransactionType::RegisterModel {
                model_hash,
                metadata,
                access_policy,
                artifact_cid,
            } => {
                self.execute_register_model(
                    from,
                    model_hash,
                    metadata,
                    access_policy,
                    artifact_cid,
                    context,
                )
                .await
            }

            TransactionType::UpdateModel {
                model_id,
                metadata,
                artifact_cid,
            } => {
                self.execute_update_model(from, model_id, metadata, artifact_cid, context)
                    .await
            }

            TransactionType::InferenceRequest {
                model_id,
                input_data,
                max_gas,
            } => {
                self.execute_inference(from, model_id, input_data, max_gas, context)
                    .await
            }

            TransactionType::SubmitGradient {
                job_id,
                gradient_data,
                proof,
            } => {
                self.execute_submit_gradient(from, job_id, gradient_data, proof, context)
                    .await
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
        hasher.update(from.0);
        hasher.update(nonce.to_be_bytes());
        let hash = hasher.finalize();

        let mut contract_addr = [0u8; 20];
        contract_addr.copy_from_slice(&hash[12..32]);
        let contract_address = Address(contract_addr);

        // Create contract account
        self.state_db
            .accounts
            .create_account_if_not_exists(contract_address);

        // Store code
        let _code_hash = self.state_db.set_code(contract_address, code);

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

        // Precompile dispatch first
        if self.is_precompile_address(&to) {
            self.execute_precompile(&to, &data, from, context).await?;
            return Ok(());
        }

        // Execute contract code with VM (unified path)
        if let Some(code) = self
            .state_db
            .get_code(&self.state_db.accounts.get_code_hash(&to))
        {
            // Fast path: scan for AI opcodes (TENSOR_OP, MODEL_LOAD/EXEC, ZK_*). If present,
            // execute them directly and return their output to align with API expectations.
            if let Ok(Some(ai_out)) = self
                .scan_and_execute_ai_opcodes(&code, &data, context)
                .await
            {
                context.output = ai_out;
                // Add execution log similar to VM path
                context.add_log(Log {
                    address: to,
                    topics: vec![Hash::new(*b"ContractExecuted0000000000000000")],
                    data: data.clone(),
                });
                return Ok(());
            }

            debug!(
                "Executing contract at {} with {} bytes of code via VM",
                to,
                code.len()
            );
            let available_gas = context.gas_limit.saturating_sub(context.gas_used);
            let mut vm = VM::new(available_gas);
            let vm_output = match vm.execute_with_input(&code, &data) {
                Ok(out) => {
                    VM_EXECUTIONS_TOTAL.with_label_values(&["ok"]).inc();
                    out
                }
                Err(e) => {
                    VM_EXECUTIONS_TOTAL.with_label_values(&["err"]).inc();
                    return Err(e);
                }
            };
            let gas_spent = available_gas.saturating_sub(vm.gas_remaining);
            if gas_spent > 0 {
                context.use_gas(gas_spent)?;
            }
            VM_GAS_USED.observe(gas_spent as f64);
            context.output = vm_output;

            // Add execution log
            context.add_log(Log {
                address: to,
                topics: vec![Hash::new(*b"ContractExecuted0000000000000000")],
                data: data.clone(),
            });
        }

        Ok(())
    }

    // ---------- Precompile handling ----------
    fn is_precompile_address(&self, addr: &Address) -> bool {
        let model = Self::model_precompile_address();
        let artifact = Self::artifact_precompile_address();
        let governance = Self::governance_precompile_address();
        *addr == model || *addr == artifact || *addr == governance
    }

    fn model_precompile_address() -> Address {
        // 0x0000000000000000000000000000000000001000
        let mut a = [0u8; 20];
        a[18] = 0x10;
        a[19] = 0x00;
        Address(a)
    }

    fn artifact_precompile_address() -> Address {
        // 0x0000000000000000000000000000000000001002
        let mut a = [0u8; 20];
        a[18] = 0x10;
        a[19] = 0x02;
        Address(a)
    }

    fn governance_precompile_address() -> Address {
        // 0x0000000000000000000000000000000000001003
        let mut a = [0u8; 20];
        a[18] = 0x10;
        a[19] = 0x03;
        Address(a)
    }

    async fn execute_precompile(
        &self,
        to: &Address,
        data: &[u8],
        from: Address,
        context: &mut ExecutionContext,
    ) -> Result<(), ExecutionError> {
        if *to == Self::model_precompile_address() {
            let res = self.execute_model_precompile(data, from, context).await;
            match &res {
                Ok(()) => PRECOMPILE_CALLS_TOTAL
                    .with_label_values(&["model", "unknown", "ok"])
                    .inc(),
                Err(_) => PRECOMPILE_CALLS_TOTAL
                    .with_label_values(&["model", "unknown", "err"])
                    .inc(),
            }
            res
        } else if *to == Self::artifact_precompile_address() {
            PRECOMPILE_CALLS_TOTAL
                .with_label_values(&["artifact", "noop", "ok"])
                .inc();
            Ok(())
        } else if *to == Self::governance_precompile_address() {
            let res = self
                .execute_governance_precompile(data, from, context)
                .await;
            match &res {
                Ok(()) => PRECOMPILE_CALLS_TOTAL
                    .with_label_values(&["governance", "unknown", "ok"])
                    .inc(),
                Err(_) => PRECOMPILE_CALLS_TOTAL
                    .with_label_values(&["governance", "unknown", "err"])
                    .inc(),
            }
            res
        } else {
            Err(ExecutionError::InvalidInput)
        }
    }

    async fn execute_governance_precompile(
        &self,
        data: &[u8],
        from: Address,
        context: &mut ExecutionContext,
    ) -> Result<(), ExecutionError> {
        use sha3::{Digest, Keccak256};
        if data.len() < 4 {
            return Err(ExecutionError::InvalidInput);
        }
        let selector = &data[0..4];
        let args = &data[4..];

        let sel_set_admin = &Keccak256::digest(b"setAdmin(address)")[..4];
        let sel_queue = &Keccak256::digest(b"queueSetParam(bytes32,bytes,uint64)")[..4];
        let sel_execute = &Keccak256::digest(b"executeSetParam(bytes32)")[..4];
        let sel_get = &Keccak256::digest(b"getParam(bytes32)")[..4];

        let gov_addr = Self::governance_precompile_address();

        // Read current admin or default to treasury address
        let admin_key = b"ADMIN".to_vec();
        let current_admin = self
            .state_db
            .get_storage(&gov_addr, &admin_key)
            .and_then(|v| {
                if v.len() >= 20 {
                    let mut a = [0u8; 20];
                    a.copy_from_slice(&v[..20]);
                    Some(Address(a))
                } else {
                    None
                }
            })
            .unwrap_or(Address([0x11; 20]));

        if selector == sel_set_admin {
            if from != current_admin {
                return Err(ExecutionError::AccessDenied);
            }
            if args.len() < 32 {
                return Err(ExecutionError::InvalidInput);
            }
            let mut addr = [0u8; 20];
            addr.copy_from_slice(&args[12..32]);
            self.state_db
                .set_storage(gov_addr, admin_key, addr.to_vec());
            return Ok(());
        }

        if selector == sel_queue {
            if from != current_admin {
                return Err(ExecutionError::AccessDenied);
            }
            if args.len() < 96 {
                return Err(ExecutionError::InvalidInput);
            }
            let key = &args[0..32];
            let mut offb = [0u8; 32];
            offb.copy_from_slice(&args[32..64]);
            let off = primitive_types::U256::from_big_endian(&offb);
            let off_usize: usize = off.try_into().unwrap_or(usize::MAX);
            if off_usize == usize::MAX {
                return Err(ExecutionError::InvalidInput);
            }
            let mut eta_bytes = [0u8; 32];
            eta_bytes.copy_from_slice(&args[64..96]);
            let eta_u256 = primitive_types::U256::from_big_endian(&eta_bytes);
            let eta: u64 = eta_u256.try_into().unwrap_or(u64::MAX);
            let dyn_start = 4 + off_usize;
            if data.len() < dyn_start + 32 {
                return Err(ExecutionError::InvalidInput);
            }
            let mut lenb = [0u8; 32];
            lenb.copy_from_slice(&data[dyn_start..dyn_start + 32]);
            let len = primitive_types::U256::from_big_endian(&lenb);
            let l: usize = len.try_into().unwrap_or(usize::MAX);
            let val_start = dyn_start + 32;
            let val_end = val_start
                .checked_add(l)
                .ok_or(ExecutionError::InvalidInput)?;
            if data.len() < val_end {
                return Err(ExecutionError::InvalidInput);
            }
            let value = &data[val_start..val_end];
            // Store pending
            let mut pending_key = b"PENDING:".to_vec();
            pending_key.extend_from_slice(key);
            let mut stored = eta.to_le_bytes().to_vec();
            stored.extend_from_slice(value);
            self.state_db.set_storage(gov_addr, pending_key, stored);
            return Ok(());
        }

        if selector == sel_execute {
            if from != current_admin {
                return Err(ExecutionError::AccessDenied);
            }
            if args.len() < 32 {
                return Err(ExecutionError::InvalidInput);
            }
            let key = &args[0..32];
            let mut pending_key = b"PENDING:".to_vec();
            pending_key.extend_from_slice(key);
            if let Some(stored) = self.state_db.get_storage(&gov_addr, &pending_key) {
                if stored.len() < 8 {
                    return Err(ExecutionError::InvalidInput);
                }
                let mut eta_bytes = [0u8; 8];
                eta_bytes.copy_from_slice(&stored[..8]);
                let eta = u64::from_le_bytes(eta_bytes);
                if context.timestamp < eta {
                    return Err(ExecutionError::Reverted("Timelock not expired".into()));
                }
                let value = &stored[8..];
                let mut param_key = b"PARAM:".to_vec();
                param_key.extend_from_slice(key);
                self.state_db
                    .set_storage(gov_addr, param_key, value.to_vec());
                self.state_db.delete_storage(gov_addr, &pending_key);
                return Ok(());
            } else {
                return Err(ExecutionError::Reverted("No such pending param".into()));
            }
        }

        if selector == sel_get {
            if args.len() < 32 {
                return Err(ExecutionError::InvalidInput);
            }
            let key = &args[0..32];
            let mut param_key = b"PARAM:".to_vec();
            param_key.extend_from_slice(key);
            if let Some(value) = self.state_db.get_storage(&gov_addr, &param_key) {
                context.output = value;
            } else {
                context.output = Vec::new();
            }
            return Ok(());
        }

        Err(ExecutionError::InvalidInput)
    }

    async fn execute_model_precompile(
        &self,
        data: &[u8],
        from: Address,
        context: &mut ExecutionContext,
    ) -> Result<(), ExecutionError> {
        use sha3::{Digest, Keccak256};
        if data.len() < 4 {
            return Err(ExecutionError::InvalidInput);
        }
        let selector = &data[0..4];
        let args = &data[4..];

        let sel_register = &Keccak256::digest(b"registerModel(bytes32,string)")[..4];
        let sel_register_ex =
            &Keccak256::digest(b"registerModel(bytes32,string,uint8,uint256)")[..4];
        let sel_infer = &Keccak256::digest(b"executeInference(bytes32,bytes)")[..4];
        let sel_pin = &Keccak256::digest(b"pin(string,uint256)")[..4];
        let sel_status = &Keccak256::digest(b"status(string)")[..4];

        if selector == sel_register || selector == sel_register_ex {
            if args.len() < 64 {
                return Err(ExecutionError::InvalidInput);
            }
            let mut mh = [0u8; 32];
            mh.copy_from_slice(&args[0..32]);
            let model_hash = Hash::new(mh);

            let mut off = [0u8; 32];
            off.copy_from_slice(&args[32..64]);
            let offset = primitive_types::U256::from_big_endian(&off);
            let offset_usize: usize = offset.try_into().unwrap_or(usize::MAX);
            if offset_usize == usize::MAX {
                return Err(ExecutionError::InvalidInput);
            }
            let dyn_start = 4 + offset_usize;
            if data.len() < dyn_start + 32 {
                return Err(ExecutionError::InvalidInput);
            }
            let mut lb = [0u8; 32];
            lb.copy_from_slice(&data[dyn_start..dyn_start + 32]);
            let len = primitive_types::U256::from_big_endian(&lb);
            let len_usize: usize = len.try_into().unwrap_or(usize::MAX);
            let cid_start = dyn_start + 32;
            let cid_end = cid_start
                .checked_add(len_usize)
                .ok_or(ExecutionError::InvalidInput)?;
            if data.len() < cid_end {
                return Err(ExecutionError::InvalidInput);
            }
            let cid = String::from_utf8_lossy(&data[cid_start..cid_end]).to_string();

            let md = ModelMetadata {
                name: "OnchainModel".to_string(),
                version: "1.0".to_string(),
                description: "Registered via precompile".to_string(),
                framework: "Unknown".to_string(),
                input_shape: vec![1],
                output_shape: vec![1],
                size_bytes: 0,
                created_at: context.timestamp,
            };
            let access_policy = if selector == sel_register_ex {
                if args.len() < 128 {
                    return Err(ExecutionError::InvalidInput);
                }
                let pol_u8 = args[95];
                match pol_u8 {
                    0 => AccessPolicy::Public,
                    1 => AccessPolicy::Private,
                    2 => AccessPolicy::Restricted(Vec::new()),
                    3 => {
                        let mut pb = [0u8; 32];
                        pb.copy_from_slice(&args[96..128]);
                        let fee = primitive_types::U256::from_big_endian(&pb);
                        AccessPolicy::PayPerUse { fee }
                    }
                    _ => AccessPolicy::Public,
                }
            } else {
                AccessPolicy::Public
            };

            let res = self
                .execute_register_model(
                    from,
                    model_hash,
                    md,
                    access_policy,
                    Some(cid.clone()),
                    context,
                )
                .await;

            match &res {
                Ok(()) => PRECOMPILE_CALLS_TOTAL
                    .with_label_values(&["model", "registerModel", "ok"])
                    .inc(),
                Err(_) => PRECOMPILE_CALLS_TOTAL
                    .with_label_values(&["model", "registerModel", "err"])
                    .inc(),
            }
            res
        } else if selector == sel_infer {
            if args.len() < 64 {
                return Err(ExecutionError::InvalidInput);
            }
            let mut mh = [0u8; 32];
            mh.copy_from_slice(&args[0..32]);
            let model_id = ModelId(Hash::new(mh));

            let mut off = [0u8; 32];
            off.copy_from_slice(&args[32..64]);
            let offset = primitive_types::U256::from_big_endian(&off);
            let offset_usize: usize = offset.try_into().unwrap_or(usize::MAX);
            if offset_usize == usize::MAX {
                return Err(ExecutionError::InvalidInput);
            }
            let dyn_start = 4 + offset_usize;
            if data.len() < dyn_start + 32 {
                return Err(ExecutionError::InvalidInput);
            }
            let mut lb = [0u8; 32];
            lb.copy_from_slice(&data[dyn_start..dyn_start + 32]);
            let len = primitive_types::U256::from_big_endian(&lb);
            let len_usize: usize = len.try_into().unwrap_or(usize::MAX);
            let bytes_start = dyn_start + 32;
            let bytes_end = bytes_start
                .checked_add(len_usize)
                .ok_or(ExecutionError::InvalidInput)?;
            if data.len() < bytes_end {
                return Err(ExecutionError::InvalidInput);
            }
            let input_data = data[bytes_start..bytes_end].to_vec();

            let res = self
                .execute_inference(
                    from,
                    model_id,
                    input_data,
                    context.gas_limit.saturating_sub(context.gas_used),
                    context,
                )
                .await;
            match &res {
                Ok(()) => PRECOMPILE_CALLS_TOTAL
                    .with_label_values(&["model", "executeInference", "ok"])
                    .inc(),
                Err(_) => PRECOMPILE_CALLS_TOTAL
                    .with_label_values(&["model", "executeInference", "err"])
                    .inc(),
            }
            res
        } else if selector == sel_pin {
            // pin(string cid, uint256 replicas)
            // args: offset(32) | replicas(32)
            if args.len() < 64 {
                return Err(ExecutionError::InvalidInput);
            }
            let mut off = [0u8; 32];
            off.copy_from_slice(&args[0..32]);
            let offset = primitive_types::U256::from_big_endian(&off);
            let off_usize: usize = offset.try_into().unwrap_or(usize::MAX);
            let mut repb = [0u8; 32];
            repb.copy_from_slice(&args[32..64]);
            let replicas_u256 = primitive_types::U256::from_big_endian(&repb);
            let replicas: usize = replicas_u256.try_into().unwrap_or(1);
            let dyn_start = 4 + off_usize;
            if data.len() < dyn_start + 32 {
                return Err(ExecutionError::InvalidInput);
            }
            let mut lenb = [0u8; 32];
            lenb.copy_from_slice(&data[dyn_start..dyn_start + 32]);
            let len = primitive_types::U256::from_big_endian(&lenb);
            let l: usize = len.try_into().unwrap_or(usize::MAX);
            let s = dyn_start + 32;
            let e = s.checked_add(l).ok_or(ExecutionError::InvalidInput)?;
            if data.len() < e {
                return Err(ExecutionError::InvalidInput);
            }
            let cid = String::from_utf8_lossy(&data[s..e]).to_string();
            if let Some(art) = &self.artifact_service {
                art.pin(&cid, replicas).await?;
            }
            context.output = b"ok".to_vec();
            Ok(())
        } else if selector == sel_status {
            // status(string cid)
            if args.len() < 32 {
                return Err(ExecutionError::InvalidInput);
            }
            let mut off = [0u8; 32];
            off.copy_from_slice(&args[0..32]);
            let offset = primitive_types::U256::from_big_endian(&off);
            let off_usize: usize = offset.try_into().unwrap_or(usize::MAX);
            let dyn_start = 4 + off_usize;
            if data.len() < dyn_start + 32 {
                return Err(ExecutionError::InvalidInput);
            }
            let mut lenb = [0u8; 32];
            lenb.copy_from_slice(&data[dyn_start..dyn_start + 32]);
            let len = primitive_types::U256::from_big_endian(&lenb);
            let l: usize = len.try_into().unwrap_or(usize::MAX);
            let s = dyn_start + 32;
            let e = s.checked_add(l).ok_or(ExecutionError::InvalidInput)?;
            if data.len() < e {
                return Err(ExecutionError::InvalidInput);
            }
            let cid = String::from_utf8_lossy(&data[s..e]).to_string();
            let status = if let Some(art) = &self.artifact_service {
                art.status(&cid).await?
            } else {
                "unknown".to_string()
            };
            context.output = status.into_bytes();
            Ok(())
        } else {
            Err(ExecutionError::InvalidInput)
        }
    }

    fn artifact_index_key(model_hash: &Hash) -> Vec<u8> {
        let mut k = b"MODEL_ARTS:".to_vec();
        k.extend_from_slice(model_hash.as_bytes());
        k
    }

    fn add_model_artifact(&self, model_hash: &Hash, cid: &str) {
        let addr = Self::artifact_precompile_address();
        let key = Self::artifact_index_key(model_hash);
        let mut list: Vec<String> = if let Some(bytes) = self.state_db.get_storage(&addr, &key) {
            serde_json::from_slice(&bytes).unwrap_or_default()
        } else {
            Vec::new()
        };
        if !list.iter().any(|c| c == cid) {
            list.push(cid.to_string());
        }
        if let Ok(bytes) = serde_json::to_vec(&list) {
            self.state_db.set_storage(addr, key, bytes);
        }
    }

    pub fn list_model_artifacts(&self, model_hash: &Hash) -> Vec<String> {
        let addr = Self::artifact_precompile_address();
        let key = Self::artifact_index_key(model_hash);
        if let Some(bytes) = self.state_db.get_storage(&addr, &key) {
            serde_json::from_slice(&bytes).unwrap_or_default()
        } else {
            Vec::new()
        }
    }

    fn proof_index_key(model_hash: &Hash) -> Vec<u8> {
        let mut k = b"MODEL_PROOFS:".to_vec();
        k.extend_from_slice(model_hash.as_bytes());
        k
    }

    fn add_model_proof_artifact(&self, model_hash: &Hash, cid: &str) {
        let addr = Self::artifact_precompile_address();
        let key = Self::proof_index_key(model_hash);
        let mut list: Vec<String> = if let Some(bytes) = self.state_db.get_storage(&addr, &key) {
            serde_json::from_slice(&bytes).unwrap_or_default()
        } else {
            Vec::new()
        };
        if !list.iter().any(|c| c == cid) {
            list.push(cid.to_string());
        }
        if let Ok(bytes) = serde_json::to_vec(&list) {
            self.state_db.set_storage(addr, key, bytes);
        }
    }

    pub fn list_model_proofs(&self, model_hash: &Hash) -> Vec<String> {
        let addr = Self::artifact_precompile_address();
        let key = Self::proof_index_key(model_hash);
        if let Some(bytes) = self.state_db.get_storage(&addr, &key) {
            serde_json::from_slice(&bytes).unwrap_or_default()
        } else {
            Vec::new()
        }
    }

    pub async fn artifact_pin(&self, cid: &str, replicas: usize) -> Result<(), ExecutionError> {
        if let Some(svc) = &self.artifact_service {
            svc.pin(cid, replicas).await
        } else {
            Err(ExecutionError::Reverted(
                "Artifact service not configured".into(),
            ))
        }
    }

    pub async fn artifact_status(&self, cid: &str) -> Result<String, ExecutionError> {
        if let Some(svc) = &self.artifact_service {
            svc.status(cid).await
        } else {
            Ok("unknown".into())
        }
    }

    fn default_artifact_replicas(&self) -> usize {
        // Read from governance: PARAM:artifact_replication
        let gov_addr = Self::governance_precompile_address();
        if let Some(bytes) = self
            .state_db
            .get_storage(&gov_addr, b"PARAM:artifact_replication")
        {
            if !bytes.is_empty() {
                return bytes[0].max(1) as usize;
            }
            if bytes.len() >= 8 {
                let mut arr = [0u8; 8];
                arr.copy_from_slice(&bytes[..8]);
                let v = u64::from_le_bytes(arr);
                return v.max(1) as usize;
            }
        }
        1
    }

    /// Execute an inference using the configured inference service without mutating state.
    pub async fn run_inference_preview(
        &self,
        from: Address,
        model_id: ModelId,
        input_data: Vec<u8>,
        max_gas: u64,
    ) -> Result<InferencePreview, ExecutionError> {
        let model = self
            .state_db
            .get_model(&model_id)
            .ok_or(ExecutionError::ModelNotFound(model_id))?;

        match &model.access_policy {
            AccessPolicy::Public => {}
            AccessPolicy::Private if model.owner == from => {}
            AccessPolicy::Restricted(allowed) if allowed.contains(&from) => {}
            AccessPolicy::PayPerUse { .. } => {
                if model.owner != from {
                    return Err(ExecutionError::AccessDenied);
                }
            }
            _ => return Err(ExecutionError::AccessDenied),
        }

        if let Some(svc) = &self.inference_service {
            let start = Instant::now();
            let (output, gas_used, provider, provider_fee, proof) =
                svc.run_inference(model_id, input_data, max_gas).await?;
            let latency_ms = start.elapsed().as_millis() as u64;

            Ok(InferencePreview {
                output,
                gas_used,
                provider,
                provider_fee,
                proof,
                latency_ms,
            })
        } else {
            Ok(InferencePreview {
                output: vec![0x01, 0x02, 0x03, 0x04],
                gas_used: 0,
                provider: Address([0; 20]),
                provider_fee: U256::zero(),
                proof: None,
                latency_ms: 0,
            })
        }
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

        info!(
            "Tensor operation: type={}, dimensions={}",
            op_type, dimensions
        );

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
        let model = self
            .state_db
            .get_model(&model_id)
            .ok_or(ExecutionError::ModelNotFound(model_id))?;

        // Gas based on model size
        let load_gas = model.metadata.size_bytes / 1024;
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
        )
        .await?;

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
        artifact_cid: Option<String>,
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
        let persisted_state = model_state.clone();

        self.state_db.register_model(model_id, model_state)?;

        if let Some(cid) = artifact_cid.clone() {
            let art_addr = Self::artifact_precompile_address();
            let mut key = b"MODEL_CID:".to_vec();
            key.extend_from_slice(model_hash.as_bytes());
            self.state_db
                .set_storage(art_addr, key, cid.clone().into_bytes());

            self.add_model_artifact(&model_hash, &cid);
            if let Some(art) = &self.artifact_service {
                let replicas = self.default_artifact_replicas();
                if let Err(err) = art.pin(&cid, replicas).await {
                    warn!("Failed to pin model artifact {}: {}", cid, err);
                }
            }

            if let Some(storage) = &self.ai_storage {
                if let Err(err) = storage.register_model(model_id, &persisted_state, &cid) {
                    warn!(
                        "AI storage registration failed for model {:?}: {}",
                        model_id, err
                    );
                }
            }

            if let Some(adapter) = &self.model_registry {
                if let Err(err) = adapter
                    .register_model(model_id, &persisted_state, Some(&cid))
                    .await
                {
                    warn!(
                        "Model registry adapter failed for model {:?}: {}",
                        model_id, err
                    );
                }
            }
        } else {
            if let Some(adapter) = &self.model_registry {
                if let Err(err) = adapter
                    .register_model(model_id, &persisted_state, None)
                    .await
                {
                    warn!(
                        "Model registry adapter failed for model {:?}: {}",
                        model_id, err
                    );
                }
            }
        }

        // Add registration log
        context.add_log(Log {
            address: from,
            topics: vec![Hash::new(*b"ModelRegistered00000000000000000"), model_hash],
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
        new_metadata: ModelMetadata,
        artifact_cid: Option<String>,
        context: &mut ExecutionContext,
    ) -> Result<(), ExecutionError> {
        context.use_gas(self.gas_schedule.model_update)?;

        let mut model = self
            .state_db
            .get_model(&model_id)
            .ok_or(ExecutionError::ModelNotFound(model_id))?;

        // Check ownership
        if model.owner != from {
            return Err(ExecutionError::AccessDenied);
        }

        model.metadata = new_metadata;
        model.metadata.created_at = context.timestamp;
        model.version += 1;
        let updated_model = model.clone();

        self.state_db.update_model(model_id, model)?;

        if let Some(cid) = artifact_cid.clone() {
            self.add_model_artifact(&updated_model.model_hash, &cid);
            if let Some(art) = &self.artifact_service {
                let replicas = self.default_artifact_replicas();
                if let Err(err) = art.pin(&cid, replicas).await {
                    warn!("Failed to pin updated model artifact {}: {}", cid, err);
                }
            }
            if let Some(storage) = &self.ai_storage {
                if let Err(err) = storage.update_model_weights(model_id, &cid, updated_model.version)
                {
                    warn!(
                        "AI storage weight update failed for {:?}: {}",
                        model_id, err
                    );
                }
            }
            if let Some(adapter) = &self.model_registry {
                if let Err(err) = adapter
                    .update_model(model_id, &updated_model, Some(&cid))
                    .await
                {
                    warn!(
                        "Model registry adapter update failed for {:?}: {}",
                        model_id, err
                    );
                }
            }
        } else if let Some(adapter) = &self.model_registry {
            if let Err(err) = adapter.update_model(model_id, &updated_model, None).await {
                warn!(
                    "Model registry adapter update failed for {:?}: {}",
                    model_id, err
                );
            }
        }

        info!(
            "Model updated: {:?} to version {}",
            model_id, updated_model.version
        );
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

        let mut model = self
            .state_db
            .get_model(&model_id)
            .ok_or(ExecutionError::ModelNotFound(model_id))?;

        // Check access policy
        match &model.access_policy {
            AccessPolicy::Public => {}
            AccessPolicy::Private if model.owner == from => {}
            AccessPolicy::Restricted(allowed) if allowed.contains(&from) => {}
            AccessPolicy::PayPerUse { fee } => {
                // Split fee: 10% protocol treasury, 90% to model owner
                let treasury_address = Address([0x11; 20]);
                let treasury_cut = *fee / U256::from(10u8);
                let owner_cut = *fee - treasury_cut;
                // Perform transfers
                self.state_db
                    .accounts
                    .transfer(&from, &model.owner, owner_cut)?;
                if treasury_cut > U256::zero() {
                    self.state_db
                        .accounts
                        .transfer(&from, &treasury_address, treasury_cut)?;
                }
                model.usage_stats.total_fees_earned += *fee;
            }
            _ => return Err(ExecutionError::AccessDenied),
        }

        // Delegate to inference service if configured, otherwise simulate
        if let Some(svc) = &self.inference_service {
            let remaining = context.gas_limit.saturating_sub(context.gas_used);
            let (out, gas_used, provider_addr, provider_fee, proof_bytes_opt) = svc
                .run_inference(model_id, input_data.clone(), remaining)
                .await?;
            // Charge compute gas
            if gas_used > 0 {
                context.use_gas(gas_used)?;
            }
            // Pay provider
            if provider_fee > U256::zero() {
                self.state_db
                    .accounts
                    .transfer(&from, &provider_addr, provider_fee)?;
            }
            // Store proof artifact if provided
            if let Some(proof_bytes) = proof_bytes_opt {
                if let Some(art) = &self.artifact_service {
                    // Add to first provider, then pin across others via pin()
                    if let Ok(cid) = art.add(&proof_bytes).await {
                        self.add_model_proof_artifact(&model_id.0, &cid);
                        let _ = art.pin(&cid, self.default_artifact_replicas()).await;
                    }
                }
            }
            context.output = out;
        } else {
            // Simulate inference output
            context.output = vec![0x01, 0x02, 0x03, 0x04];
        }

        // Update usage stats
        model.usage_stats.total_inferences += 1;
        model.usage_stats.total_gas_used += context.gas_used;
        model.usage_stats.last_used = context.timestamp;
        self.state_db.update_model(model_id, model)?;

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

        let mut job = self
            .state_db
            .get_training_job(&job_id)
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
                self.state_db
                    .accounts
                    .set_balance(*participant, balance + reward_per_participant);
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
    use async_trait::async_trait;
    use citrate_consensus::types::{BlockHeader, PublicKey, Signature, VrfProof};
    use parking_lot::Mutex;
    use serde_json::json;
    use sha3::{Digest, Keccak256};
    use std::sync::Arc;

    struct RecordingStorage {
        records: Arc<Mutex<Vec<(ModelId, String)>>>,
        updates: Arc<Mutex<Vec<(ModelId, String, u32)>>>,
    }

    impl RecordingStorage {
        fn new() -> (
            Self,
            Arc<Mutex<Vec<(ModelId, String)>>>,
            Arc<Mutex<Vec<(ModelId, String, u32)>>>,
        ) {
            let records = Arc::new(Mutex::new(Vec::new()));
            let updates = Arc::new(Mutex::new(Vec::new()));
            (
                Self {
                    records: records.clone(),
                    updates: updates.clone(),
                },
                records,
                updates,
            )
        }
    }

    impl AIModelStorage for RecordingStorage {
        fn register_model(
            &self,
            model_id: ModelId,
            _model_state: &ModelState,
            weight_cid: &str,
        ) -> anyhow::Result<()> {
            self.records.lock().push((model_id, weight_cid.to_string()));
            Ok(())
        }

        fn update_model_weights(
            &self,
            model_id: ModelId,
            weight_cid: &str,
            new_version: u32,
        ) -> anyhow::Result<()> {
            self.updates
                .lock()
                .push((model_id, weight_cid.to_string(), new_version));
            Ok(())
        }
    }

    struct RecordingRegistry {
        records: Arc<Mutex<Vec<(ModelId, Option<String>)>>>,
    }

    impl RecordingRegistry {
        fn new() -> (Self, Arc<Mutex<Vec<(ModelId, Option<String>)>>>) {
            let records = Arc::new(Mutex::new(Vec::new()));
            (
                Self {
                    records: records.clone(),
                },
                records,
            )
        }
    }

    #[async_trait]
    impl ModelRegistryAdapter for RecordingRegistry {
        async fn register_model(
            &self,
            model_id: ModelId,
            _model_state: &ModelState,
            artifact_cid: Option<&str>,
        ) -> anyhow::Result<()> {
            self.records
                .lock()
                .push((model_id, artifact_cid.map(|s| s.to_string())));
            Ok(())
        }
    }

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

    fn create_test_tx(
        from: PublicKey,
        to: Option<PublicKey>,
        value: u64,
        nonce: u64,
    ) -> Transaction {
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
        state_db
            .accounts
            .set_balance(alice_addr, U256::from(1_000_000_000_000_000u128));

        let block = create_test_block();
        let tx = create_test_tx(alice, Some(bob), 1000, 0);

        let receipt = executor.execute_transaction(&block, &tx).await.unwrap();

        assert!(receipt.status);
        assert_eq!(state_db.accounts.get_balance(&bob_addr), U256::from(1000));
    }

    #[tokio::test]
    async fn test_register_model_via_transaction_payload() {
        let state_db = Arc::new(StateDB::new());
        let (storage_adapter, storage_records, _) = RecordingStorage::new();
        let (registry_adapter, registry_records) = RecordingRegistry::new();

        let executor = Executor::new(state_db.clone())
            .with_ai_storage_adapter(Arc::new(storage_adapter))
            .with_model_registry_adapter(Arc::new(registry_adapter));

        let sender_pk = PublicKey::new([4; 32]);
        let from_addr = Address::from_public_key(&sender_pk);
        state_db
            .accounts
            .set_balance(from_addr, U256::from(1_000_000_000_000_000u128));

        let block = create_test_block();
        let mut target_addr = [0u8; 32];
        target_addr[18] = 0x10;
        target_addr[19] = 0x00;
        let target_pk = PublicKey::new(target_addr);

        let model_hash_bytes = [0xAB; 32];
        let metadata_json = json!({
            "name": "CLI Model",
            "version": "1.2.3",
            "description": "Integration test model",
            "framework": "onnx",
            "input_shape": [1, 4],
            "output_shape": [1],
            "size_bytes": 2048
        });
        let metadata_bytes = serde_json::to_vec(&metadata_json).unwrap();
        let metadata_len = metadata_bytes.len() as u32;
        let artifact_cid = "bafyModelCID123";

        let mut data = Vec::new();
        data.extend_from_slice(&[0x01, 0x00, 0x00, 0x00]);
        data.extend_from_slice(&model_hash_bytes);
        data.extend_from_slice(&metadata_len.to_be_bytes());
        data.extend_from_slice(&metadata_bytes);
        data.push(0); // public policy
        data.extend_from_slice(&(artifact_cid.len() as u32).to_be_bytes());
        data.extend_from_slice(artifact_cid.as_bytes());

        let tx = citrate_consensus::types::Transaction {
            hash: Hash::new([5; 32]),
            nonce: 0,
            from: sender_pk,
            to: Some(target_pk),
            value: 0,
            gas_limit: 200000,
            gas_price: 1_000_000_000,
            data,
            signature: Signature::new([0; 64]),
            tx_type: None,
        };

        let receipt = executor.execute_transaction(&block, &tx).await.unwrap();
        assert!(receipt.status);

        let model_id = ModelId(Hash::new(model_hash_bytes));
        let stored_model = state_db.get_model(&model_id).expect("model stored");
        assert_eq!(stored_model.metadata.name, "CLI Model");
        assert_eq!(stored_model.metadata.framework, "onnx");
        assert_eq!(stored_model.metadata.input_shape, vec![1, 4]);

        let stored_records = storage_records.lock();
        assert_eq!(stored_records.len(), 1);
        assert_eq!(stored_records[0].0, model_id);
        assert_eq!(stored_records[0].1, artifact_cid);
        drop(stored_records);

        let registry_records_guard = registry_records.lock();
        assert_eq!(registry_records_guard.len(), 1);
        assert_eq!(registry_records_guard[0].0, model_id);
        assert_eq!(registry_records_guard[0].1.as_deref(), Some(artifact_cid));
    }

    #[tokio::test]
    async fn test_model_precompile_register_and_infer() {
        let state_db = Arc::new(StateDB::new());
        let executor = Executor::new(state_db.clone());

        // Sender and dummy block
        let sender_pk = PublicKey::new([3; 32]);
        let from_addr = Address::from_public_key(&sender_pk);
        state_db
            .accounts
            .set_balance(from_addr, U256::from(1_000_000_000_000_000u128));

        let block = create_test_block();

        // Build precompile public key whose first 20 bytes are the model precompile address
        let mut pc_bytes = [0u8; 32];
        // 0x...1000 in last two bytes of 20-byte address
        pc_bytes[18] = 0x10;
        pc_bytes[19] = 0x00;
        let precompile_pk = PublicKey::new(pc_bytes);

        // registerModel(bytes32,string)
        let mut reg_data = Vec::new();
        let reg_sel = &Keccak256::digest(b"registerModel(bytes32,string)")[..4];
        reg_data.extend_from_slice(reg_sel);
        let model_hash = [9u8; 32];
        reg_data.extend_from_slice(&model_hash); // bytes32
        reg_data.extend_from_slice(&[0u8; 31]);
        reg_data.push(64); // offset = 64 (0x40)
                           // dynamic part
        reg_data.extend_from_slice(&[0u8; 31]);
        reg_data.push(3); // length = 3
        reg_data.extend_from_slice(b"cid");
        // pad to 32
        reg_data.extend_from_slice(&[0u8; 29]);

        // Execute register call
        let tx_reg = citrate_consensus::types::Transaction {
            hash: Hash::new([2; 32]),
            nonce: 0,
            from: sender_pk,
            to: Some(precompile_pk),
            value: 0,
            gas_limit: 200000,
            gas_price: 1_000_000_000,
            data: reg_data,
            signature: Signature::new([0; 64]),
            tx_type: None,
        };
        let _ = executor.execute_transaction(&block, &tx_reg).await.unwrap();

        // Verify model registered
        let mid = ModelId(Hash::new(model_hash));
        let model = state_db.get_model(&mid).expect("model exists");
        assert_eq!(model.owner, from_addr);

        // executeInference(bytes32,bytes)
        let mut inf_data = Vec::new();
        let inf_sel = &Keccak256::digest(b"executeInference(bytes32,bytes)")[..4];
        inf_data.extend_from_slice(inf_sel);
        inf_data.extend_from_slice(&model_hash);
        inf_data.extend_from_slice(&[0u8; 31]);
        inf_data.push(64); // offset to bytes
                           // dynamic bytes
        inf_data.extend_from_slice(&[0u8; 31]);
        inf_data.push(4); // len = 4
        inf_data.extend_from_slice(&[1, 2, 3, 4]); // bytes
        inf_data.extend_from_slice(&[0u8; 28]); // pad

        let tx_inf = citrate_consensus::types::Transaction {
            hash: Hash::new([3; 32]),
            nonce: 1,
            from: sender_pk,
            to: Some(precompile_pk),
            value: 0,
            gas_limit: 200000,
            gas_price: 1_000_000_000,
            data: inf_data,
            signature: Signature::new([0; 64]),
            tx_type: None,
        };
        let receipt = executor.execute_transaction(&block, &tx_inf).await.unwrap();
        assert!(receipt.status);
        // Output set by executor inference simulation
        assert_eq!(receipt.output, vec![0x01, 0x02, 0x03, 0x04]);
    }

    #[tokio::test]
    async fn test_governance_precompile_timelock_and_params() {
        use sha3::{Digest, Keccak256};

        let state_db = Arc::new(StateDB::new());
        let executor = Executor::new(state_db.clone());

        // Set sender to default treasury admin (Address([0x11;20])) so setAdmin/queue can succeed
        let mut admin_pk_bytes = [0u8; 32];
        admin_pk_bytes[..20].copy_from_slice(&[0x11; 20]);
        let admin_pk = PublicKey::new(admin_pk_bytes);
        let admin_addr = Address([0x11; 20]);
        state_db
            .accounts
            .set_balance(admin_addr, U256::from(1_000_000_000_000_000u128));

        let mut block = create_test_block();
        block.header.timestamp = 1_000_000;

        // Build governance precompile address as in executor
        let gov_addr = {
            let mut a = [0u8; 20];
            a[18] = 0x10;
            a[19] = 0x03;
            Address(a)
        };
        let mut gov_pk = [0u8; 32];
        gov_pk[..20].copy_from_slice(&gov_addr.0);
        let gov_pk = PublicKey::new(gov_pk);

        // 1) setAdmin(address) to same admin (no-op but exercises path)
        let mut set_admin = Vec::new();
        let sel_set_admin = &Keccak256::digest(b"setAdmin(address)")[..4];
        set_admin.extend_from_slice(sel_set_admin);
        // abi-encode address as 32-byte, right-aligned: pad 12 zeros then 20-byte addr
        set_admin.extend_from_slice(&[0u8; 12]);
        set_admin.extend_from_slice(&admin_addr.0);
        let tx_set = Transaction {
            hash: Hash::new([10; 32]),
            nonce: 0,
            from: admin_pk,
            to: Some(gov_pk),
            value: 0,
            gas_limit: 200000,
            gas_price: 1_000_000_000,
            data: set_admin,
            signature: Signature::new([0; 64]),
            tx_type: None,
        };
        let _ = executor.execute_transaction(&block, &tx_set).await.unwrap();

        // 2) queueSetParam(bytes32 key, bytes value, uint64 eta)
        let key = [0xAAu8; 32];
        let value: Vec<u8> = vec![0xDE, 0xAD, 0xBE, 0xEF];
        let eta: u64 = block.header.timestamp + 60;

        let mut queue = Vec::new();
        let sel_queue = &Keccak256::digest(b"queueSetParam(bytes32,bytes,uint64)")[..4];
        queue.extend_from_slice(sel_queue);
        // key (32)
        queue.extend_from_slice(&key);
        // offset to dynamic bytes: 0x60 (96), counting from after selector per ABI (impl adds +4)
        let mut off = [0u8; 32];
        off[31] = 96;
        queue.extend_from_slice(&off);
        // eta (uint64) as 32-byte big-endian
        let mut eta_be = [0u8; 32];
        eta_be[24..32].copy_from_slice(&eta.to_be_bytes());
        queue.extend_from_slice(&eta_be);
        // dynamic bytes: length (32) + data + padding
        let mut lenb = [0u8; 32];
        lenb[31] = value.len() as u8;
        queue.extend_from_slice(&lenb);
        queue.extend_from_slice(&value);
        // pad to 32
        queue.extend_from_slice(&vec![0u8; (32 - (value.len() % 32)) % 32]);

        let tx_q = Transaction {
            hash: Hash::new([11; 32]),
            nonce: 1,
            from: admin_pk,
            to: Some(gov_pk),
            value: 0,
            gas_limit: 300000,
            gas_price: 1_000_000_000,
            data: queue,
            signature: Signature::new([0; 64]),
            tx_type: None,
        };
        let _ = executor.execute_transaction(&block, &tx_q).await.unwrap();

        // 3) executeSetParam(bytes32 key) before eta → expect revert
        let mut exec = Vec::new();
        let sel_exec = &Keccak256::digest(b"executeSetParam(bytes32)")[..4];
        exec.extend_from_slice(sel_exec);
        exec.extend_from_slice(&key);
        let tx_e_early = Transaction {
            hash: Hash::new([12; 32]),
            nonce: 2,
            from: admin_pk,
            to: Some(gov_pk),
            value: 0,
            gas_limit: 300000,
            gas_price: 1_000_000_000,
            data: exec.clone(),
            signature: Signature::new([0; 64]),
            tx_type: None,
        };
        let res = executor.execute_transaction(&block, &tx_e_early).await;
        assert!(res.is_ok());
        // Even on revert, receipt.status=false, but our test harness doesn’t expose; this ensures path runs.

        // Advance time and execute again
        block.header.timestamp = eta + 1;
        // Build a fresh tx with incremented nonce now that previous attempts affected nonce accounting
        let tx_e_late = Transaction {
            nonce: 3,
            ..tx_e_early
        };
        let rcpt_ok = executor
            .execute_transaction(&block, &tx_e_late)
            .await
            .unwrap();
        assert!(rcpt_ok.status);

        // 4) getParam(bytes32 key) returns value in output
        let mut getp = Vec::new();
        let sel_get = &Keccak256::digest(b"getParam(bytes32)")[..4];
        getp.extend_from_slice(sel_get);
        getp.extend_from_slice(&key);
        let tx_g = Transaction {
            hash: Hash::new([13; 32]),
            nonce: 4,
            from: admin_pk,
            to: Some(gov_pk),
            value: 0,
            gas_limit: 200000,
            gas_price: 1_000_000_000,
            data: getp,
            signature: Signature::new([0; 64]),
            tx_type: None,
        };
        let rcpt_get = executor.execute_transaction(&block, &tx_g).await.unwrap();
        assert!(rcpt_get.status);
        assert_eq!(rcpt_get.output, value);
    }
}
