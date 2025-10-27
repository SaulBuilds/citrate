// citrate/core/execution/src/vm/evm_integration.rs

use crate::types::{Address, ExecutionError};
use crate::vm::evm_opcodes::{EVMContext, EVMExecutor, EVMState};
use crate::state::StateDB;
use primitive_types::U256;
use std::sync::Arc;
use tracing::debug;

/// Integration layer between EVM opcodes and Citrate execution environment
pub struct EVMIntegration {
    executor: EVMExecutor,
    state_db: Arc<StateDB>,
}

impl EVMIntegration {
    pub fn new(state_db: Arc<StateDB>) -> Self {
        Self {
            executor: EVMExecutor::new(),
            state_db,
        }
    }

    /// Execute EVM bytecode with full context integration
    pub fn execute(
        &mut self,
        code: &[u8],
        input_data: &[u8],
        caller: Address,
        contract_address: Address,
        value: U256,
        gas_limit: u64,
        gas_price: U256,
        origin: Address,
        block_number: u64,
        block_timestamp: u64,
        block_hash: [u8; 32],
        coinbase: [u8; 20],
        chain_id: u64,
        base_fee: U256,
    ) -> Result<(Vec<u8>, u64), ExecutionError> {
        // Create dynamic context with closures for state access
        let state_db = self.state_db.clone();
        let state_db_balance = state_db.clone();
        let state_db_code = state_db.clone();
        let state_db_code_size = state_db.clone();
        let state_db_code_hash = state_db.clone();

        let context = EVMContext {
            block_number,
            block_timestamp,
            block_hash,
            coinbase,
            prevrandao: U256::from(block_timestamp), // Simplified
            gas_limit,
            chain_id,
            base_fee,
            blob_base_fee: U256::zero(),
            origin: origin.0,
            caller: caller.0,
            call_value: value,
            gas_price,
            calldata: input_data.to_vec(),
            address: contract_address.0,
            code: code.to_vec(),
            get_balance: Box::new(move |addr: &[u8; 20]| {
                let address = Address(*addr);
                state_db_balance.accounts.get_balance(&address)
            }),
            get_code_size: Box::new(move |addr: &[u8; 20]| {
                let address = Address(*addr);
                let code_hash = state_db_code_size.accounts.get_code_hash(&address);
                if let Some(code) = state_db_code_size.get_code(&code_hash) {
                    code.len()
                } else {
                    0
                }
            }),
            get_code_hash: Box::new(move |addr: &[u8; 20]| {
                let address = Address(*addr);
                let code_hash = state_db_code_hash.accounts.get_code_hash(&address);
                code_hash.as_bytes().clone()
            }),
            get_code: Box::new(move |addr: &[u8; 20]| {
                let address = Address(*addr);
                let code_hash = state_db_code.accounts.get_code_hash(&address);
                state_db_code.get_code(&code_hash).unwrap_or_default()
            }),
        };

        // Create execution state
        let mut state = EVMState::new(gas_limit);

        // Execute bytecode instruction by instruction
        while state.pc < context.code.len() && !state.stopped {
            let opcode = context.code[state.pc];
            debug!("Executing opcode 0x{:02x} at PC {}", opcode, state.pc);

            let original_pc = state.pc;

            // Execute the opcode
            self.executor.execute_opcode(opcode, &mut state, &context)?;

            // Advance PC unless jump occurred (PC would have changed)
            if state.pc == original_pc {
                state.pc += 1;
            }

            // Check for execution completion
            if state.stopped {
                break;
            }
        }

        let gas_used = gas_limit.saturating_sub(state.gas_remaining);

        if state.reverted {
            return Err(ExecutionError::Reverted("EVM execution reverted".to_string()));
        }

        Ok((state.return_data, gas_used))
    }

    /// Get the underlying state database
    pub fn state_db(&self) -> &Arc<StateDB> {
        &self.state_db
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::StateDB;

    #[test]
    fn test_evm_integration_simple_execution() {
        let state_db = Arc::new(StateDB::new());
        let mut integration = EVMIntegration::new(state_db.clone());

        // Simple bytecode: PUSH1 0x42 RETURN
        let code = vec![0x60, 0x42, 0x60, 0x00, 0x52, 0x60, 0x20, 0x60, 0x00, 0xf3];

        let caller = Address([1u8; 20]);
        let contract = Address([2u8; 20]);

        let result = integration.execute(
            &code,
            &[],
            caller,
            contract,
            U256::zero(),
            100000,
            U256::from(1000000000u64),
            caller,
            100,
            1000000,
            [0u8; 32],
            [0u8; 20],
            1,
            U256::from(1000000000u64),
        );

        assert!(result.is_ok());
        let (output, gas_used) = result.unwrap();
        assert!(!output.is_empty());
        assert!(gas_used > 0);
    }

    #[test]
    fn test_evm_integration_with_balance_lookup() {
        let state_db = Arc::new(StateDB::new());
        let mut integration = EVMIntegration::new(state_db.clone());

        let test_addr = Address([3u8; 20]);
        let test_balance = U256::from(12345u64);

        // Set up test balance
        state_db.accounts.set_balance(test_addr, test_balance);

        // Bytecode: PUSH20 address BALANCE PUSH1 0x00 MSTORE PUSH1 0x20 PUSH1 0x00 RETURN
        let mut code = vec![0x73]; // PUSH20
        code.extend_from_slice(&test_addr.0);
        code.extend_from_slice(&[0x31]); // BALANCE
        code.extend_from_slice(&[0x60, 0x00, 0x52]); // PUSH1 0x00 MSTORE
        code.extend_from_slice(&[0x60, 0x20, 0x60, 0x00, 0xf3]); // PUSH1 0x20 PUSH1 0x00 RETURN

        let caller = Address([1u8; 20]);
        let contract = Address([2u8; 20]);

        let result = integration.execute(
            &code,
            &[],
            caller,
            contract,
            U256::zero(),
            100000,
            U256::from(1000000000u64),
            caller,
            100,
            1000000,
            [0u8; 32],
            [0u8; 20],
            1,
            U256::from(1000000000u64),
        );

        assert!(result.is_ok());
        let (output, _) = result.unwrap();

        // Check if balance was correctly retrieved and returned
        let mut expected = [0u8; 32];
        test_balance.to_big_endian(&mut expected);
        assert_eq!(output, expected);
    }
}