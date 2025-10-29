// citrate/core/execution/src/revm_adapter.rs

use crate::state::StateDB;
use crate::types::{Address, ExecutionError};
use primitive_types::U256;
use revm::{
    primitives::{
        AccountInfo, Address as RevmAddress, Bytecode, Bytes, ExecutionResult, Output,
        TransactTo, TxEnv, B256, U256 as RevmU256, SpecId, KECCAK_EMPTY,
    },
    Database, DatabaseCommit, Evm,
};
use std::sync::Arc;
use tracing::{debug, info};

/// Adapter to make StateDB compatible with revm's Database trait
pub struct StateDBAdapter {
    state_db: Arc<StateDB>,
}

impl StateDBAdapter {
    pub fn new(state_db: Arc<StateDB>) -> Self {
        Self { state_db }
    }
}

impl Database for StateDBAdapter {
    type Error = ExecutionError;

    fn basic(&mut self, address: RevmAddress) -> Result<Option<AccountInfo>, Self::Error> {
        let addr = Address(address.0 .0);
        let balance = self.state_db.accounts.get_balance(&addr);
        let nonce = self.state_db.accounts.get_nonce(&addr);
        let code_hash = self.state_db.accounts.get_code_hash(&addr);

        // Convert to revm types
        let balance_revm = RevmU256::from_limbs(balance.0);

        // Check if code_hash is default (all zeros) - if so, use KECCAK_EMPTY
        // This is critical for EIP-3607 check - revm rejects transactions from accounts with code
        let code_hash_b256 = if code_hash.as_bytes().iter().all(|&b| b == 0) {
            KECCAK_EMPTY  // Proper empty code hash
        } else {
            B256::from_slice(code_hash.as_bytes())
        };

        Ok(Some(AccountInfo {
            balance: balance_revm,
            nonce: nonce,
            code_hash: code_hash_b256,
            code: None, // Lazy load code
        }))
    }

    fn code_by_hash(&mut self, code_hash: B256) -> Result<Bytecode, Self::Error> {
        let hash = citrate_consensus::types::Hash::new(code_hash.0);
        let code = self
            .state_db
            .get_code(&hash)
            .unwrap_or_default();

        Ok(Bytecode::new_raw(Bytes::from(code)))
    }

    fn storage(&mut self, address: RevmAddress, index: RevmU256) -> Result<RevmU256, Self::Error> {
        let addr = Address(address.0 .0);
        let key_bytes: [u8; 32] = index.to_be_bytes();

        // Get storage value as bytes
        let value_bytes = self
            .state_db
            .get_storage(&addr, &key_bytes)
            .unwrap_or_else(|| vec![0u8; 32]);

        // Pad to 32 bytes if needed
        let mut padded = [0u8; 32];
        let len = value_bytes.len().min(32);
        padded[32 - len..].copy_from_slice(&value_bytes[value_bytes.len() - len..]);

        Ok(RevmU256::from_be_bytes(padded))
    }

    fn block_hash(&mut self, _number: RevmU256) -> Result<B256, Self::Error> {
        // Simplified: return zero hash
        Ok(B256::ZERO)
    }
}

impl DatabaseCommit for StateDBAdapter {
    fn commit(&mut self, changes: revm::primitives::HashMap<RevmAddress, revm::primitives::Account>) {
        for (address, account) in changes {
            let addr = Address(address.0 .0);

            // Update balance
            let balance = U256::from_big_endian(&account.info.balance.to_be_bytes::<32>());
            self.state_db.accounts.set_balance(addr, balance);

            // Update nonce
            self.state_db.accounts.set_nonce(addr, account.info.nonce);

            // Update storage
            for (key, value) in account.storage {
                let key_bytes = key.to_be_bytes::<32>();
                let value_bytes = value.present_value.to_be_bytes::<32>();
                self.state_db.set_storage(addr, key_bytes.to_vec(), value_bytes.to_vec());
            }

            // Update code if changed
            if account.info.code.is_some() {
                let code = account.info.code.unwrap();
                let code_bytes = code.bytes().to_vec();
                if !code_bytes.is_empty() {
                    self.state_db.set_code(addr, code_bytes);
                }
            }
        }
    }
}

/// Execute contract creation using revm
pub fn execute_contract_create(
    state_db: Arc<StateDB>,
    deployer: Address,
    init_code: Vec<u8>,
    value: U256,
    gas_limit: u64,
    gas_price: U256,
    chain_id: u64,
    block_number: u64,
    block_timestamp: u64,
) -> Result<(Address, Vec<u8>, u64), ExecutionError> {
    debug!("Executing contract creation with revm");
    debug!("  Deployer: {}", deployer);
    debug!("  Init code size: {} bytes", init_code.len());
    debug!("  Gas limit: {}", gas_limit);

    // Create database adapter
    let mut db = StateDBAdapter::new(state_db.clone());

    // Build EVM with transaction
    // Use LONDON spec for EVM compatibility
    // LONDON includes all necessary opcodes for Solidity 0.8.x
    let mut evm = Evm::builder()
        .with_db(&mut db)
        .modify_cfg_env(|cfg| {
            cfg.chain_id = chain_id;
        })
        .with_spec_id(SpecId::SHANGHAI)
        .modify_tx_env(|tx| {
            tx.caller = RevmAddress::from_slice(&deployer.0);
            tx.transact_to = TransactTo::Create;
            tx.data = Bytes::from(init_code);
            tx.value = RevmU256::from_limbs(value.0);
            tx.gas_limit = gas_limit;
            tx.gas_price = RevmU256::from_limbs(gas_price.0);
            tx.chain_id = Some(chain_id);
        })
        .modify_block_env(|block| {
            block.number = RevmU256::from(block_number);
            block.timestamp = RevmU256::from(block_timestamp);
        })
        .build();

    // Execute transaction
    let result = evm.transact_commit().map_err(|e| {
        ExecutionError::Reverted(format!("revm execution failed: {:?}", e))
    })?;

    match result {
        ExecutionResult::Success {
            output,
            gas_used,
            ..
        } => {
            match output {
                Output::Create(runtime_code, Some(contract_address)) => {
                    let addr = Address(contract_address.0 .0);
                    let code = runtime_code.to_vec();

                    info!(
                        "Contract deployed successfully at {} with {} bytes of runtime code",
                        addr,
                        code.len()
                    );

                    Ok((addr, code, gas_used))
                }
                Output::Create(_, None) => {
                    Err(ExecutionError::Reverted(
                        "Contract creation failed: no address returned".to_string(),
                    ))
                }
                _ => Err(ExecutionError::Reverted(
                    "Unexpected output type for contract creation".to_string(),
                )),
            }
        }
        ExecutionResult::Revert { gas_used, output } => {
            let reason = if !output.is_empty() {
                format!("0x{}", hex::encode(&output))
            } else {
                "Unknown reason".to_string()
            };
            Err(ExecutionError::Reverted(format!(
                "Contract creation reverted: {} (gas used: {})",
                reason, gas_used
            )))
        }
        ExecutionResult::Halt { reason, gas_used } => Err(ExecutionError::Reverted(format!(
            "Contract creation halted: {:?} (gas used: {})",
            reason, gas_used
        ))),
    }
}

/// Execute contract call using revm
pub fn execute_contract_call(
    state_db: Arc<StateDB>,
    caller: Address,
    contract: Address,
    calldata: Vec<u8>,
    value: U256,
    gas_limit: u64,
    gas_price: U256,
    chain_id: u64,
    block_number: u64,
    block_timestamp: u64,
) -> Result<(Vec<u8>, u64), ExecutionError> {
    debug!("Executing contract call with revm");
    debug!("  Caller: {}", caller);
    debug!("  Contract: {}", contract);
    debug!("  Calldata size: {} bytes", calldata.len());

    // Create database adapter
    let mut db = StateDBAdapter::new(state_db);

    // Build EVM with transaction
    // Use LONDON spec for EVM compatibility
    let mut evm = Evm::builder()
        .with_db(&mut db)
        .modify_cfg_env(|cfg| {
            cfg.chain_id = chain_id;
        })
        .with_spec_id(SpecId::SHANGHAI)
        .modify_tx_env(|tx| {
            tx.caller = RevmAddress::from_slice(&caller.0);
            tx.transact_to = TransactTo::Call(RevmAddress::from_slice(&contract.0));
            tx.data = Bytes::from(calldata);
            tx.value = RevmU256::from_limbs(value.0);
            tx.gas_limit = gas_limit;
            tx.gas_price = RevmU256::from_limbs(gas_price.0);
            tx.chain_id = Some(chain_id);
        })
        .modify_block_env(|block| {
            block.number = RevmU256::from(block_number);
            block.timestamp = RevmU256::from(block_timestamp);
        })
        .build();

    // Execute transaction
    let result = evm.transact_commit().map_err(|e| {
        ExecutionError::Reverted(format!("revm execution failed: {:?}", e))
    })?;

    match result {
        ExecutionResult::Success {
            output, gas_used, ..
        } => match output {
            Output::Call(return_data) => Ok((return_data.to_vec(), gas_used)),
            _ => Err(ExecutionError::Reverted(
                "Unexpected output type for contract call".to_string(),
            )),
        },
        ExecutionResult::Revert { gas_used, output } => {
            let reason = if !output.is_empty() {
                format!("0x{}", hex::encode(&output))
            } else {
                "Unknown reason".to_string()
            };
            Err(ExecutionError::Reverted(format!(
                "Contract call reverted: {} (gas used: {})",
                reason, gas_used
            )))
        }
        ExecutionResult::Halt { reason, gas_used } => Err(ExecutionError::Reverted(format!(
            "Contract call halted: {:?} (gas used: {})",
            reason, gas_used
        ))),
    }
}
