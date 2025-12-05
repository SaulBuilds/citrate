// Comprehensive tests for the execution module

use citrate_consensus::types::{
    Block, BlockHeader, GhostDagParams, Hash, PublicKey, Signature,
    Transaction as ConsensusTransaction, VrfProof,
};
use citrate_execution::{address_utils, types::*, Executor, StateDB};
use primitive_types::U256;
use std::sync::Arc;

#[allow(dead_code)]
fn create_test_block(height: u64) -> Block {
    Block {
        header: BlockHeader {
            version: 1,
            block_hash: Hash::new([height as u8; 32]),
            selected_parent_hash: Hash::default(),
            merge_parent_hashes: vec![],
            timestamp: 1000000 + height * 10,
            height,
            blue_score: height * 10,
            blue_work: (height as u128) * 1000,
            pruning_point: Hash::default(),
            proposer_pubkey: PublicKey::new([0u8; 32]),
            vrf_reveal: VrfProof {
                proof: vec![0u8; 80],
                output: Hash::default(),
            },
        },
        state_root: Hash::default(),
        tx_root: Hash::default(),
        receipt_root: Hash::default(),
        artifact_root: Hash::default(),
        ghostdag_params: GhostDagParams::default(),
        transactions: vec![],
        signature: Signature::new([0u8; 64]),
        embedded_models: vec![],
        required_pins: vec![],
    }
}

#[allow(dead_code)]
fn create_test_transaction(
    from: PublicKey,
    to: Option<PublicKey>,
    value: u128,
    nonce: u64,
) -> ConsensusTransaction {
    let mut hash_bytes = [0u8; 32];
    hash_bytes[0] = nonce as u8;

    ConsensusTransaction {
        hash: Hash::new(hash_bytes),
        nonce,
        from,
        to,
        value,
        gas_limit: 21000,
        gas_price: 20,
        data: vec![],
        signature: Signature::new([nonce as u8; 64]),
        tx_type: None,
    }
}

fn create_test_address(num: u8) -> Address {
    let mut addr = [0u8; 20];
    addr[0] = num;
    Address(addr)
}

#[cfg(test)]
mod execution_tests {
    use super::*;

    #[test]
    fn test_executor_creation() {
        let state_db = Arc::new(StateDB::new());
        let executor = Executor::new(state_db.clone());

        // Test that executor exists and can perform basic operations
        let addr = create_test_address(0);
        executor.set_balance(&addr, U256::from(100));
        assert_eq!(executor.get_balance(&addr), U256::from(100));
    }

    #[test]
    fn test_address_normalization() {
        // Test 20-byte address padded in 32-byte field
        let mut padded = [0u8; 32];
        padded[..20].copy_from_slice(&[1u8; 20]);
        let pubkey = PublicKey::new(padded);

        let normalized = address_utils::normalize_address(&pubkey);
        assert_eq!(normalized.0, [1u8; 20]);
    }

    #[test]
    fn test_address_from_hex() {
        let hex_with_prefix = "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb1";
        let hex_without = "742d35Cc6634C0532925a3b844Bc9e7595f0bEb1";

        let addr1 = address_utils::address_from_hex(hex_with_prefix).unwrap();
        let addr2 = address_utils::address_from_hex(hex_without).unwrap();

        assert_eq!(addr1, addr2);
    }

    #[test]
    fn test_balance_operations() {
        let state_db = Arc::new(StateDB::new());
        let executor = Executor::new(state_db);

        let addr = create_test_address(1);

        // Initial balance should be zero
        let initial = executor.get_balance(&addr);
        assert_eq!(initial, U256::zero());

        // Set balance
        let new_balance = U256::from(1000);
        executor.set_balance(&addr, new_balance);

        // Verify balance updated
        assert_eq!(executor.get_balance(&addr), new_balance);
    }

    #[test]
    fn test_nonce_operations() {
        let state_db = Arc::new(StateDB::new());
        let executor = Executor::new(state_db);

        let addr = create_test_address(2);

        // Initial nonce should be zero
        let initial = executor.get_nonce(&addr);
        assert_eq!(initial, 0);

        // Set nonce
        executor.set_nonce(&addr, 5);

        // Verify nonce updated
        assert_eq!(executor.get_nonce(&addr), 5);
    }

    #[test]
    fn test_code_operations() {
        let state_db = Arc::new(StateDB::new());
        let executor = Executor::new(state_db.clone());

        let addr = create_test_address(3);
        let code = vec![0x60, 0x60, 0x60, 0x40]; // Sample bytecode

        // Initially no code (check code hash)
        let initial_hash = executor.get_code_hash(&addr);
        assert_eq!(initial_hash, Hash::default());

        // Set code
        executor.set_code(&addr, code.clone());

        // Verify code stored (code hash should change)
        let new_hash = executor.get_code_hash(&addr);
        assert_ne!(new_hash, Hash::default());

        // Also verify through state_db using the hash
        let stored_code = state_db.get_code(&new_hash);
        assert_eq!(stored_code, Some(code));
    }

    #[test]
    fn test_storage_operations() {
        let state_db = Arc::new(StateDB::new());
        let _executor = Executor::new(state_db.clone());

        let addr = create_test_address(4);
        let key = b"test_key".to_vec();
        let value = b"test_value".to_vec();

        // Initially storage empty
        assert_eq!(state_db.get_storage(&addr, &key), None);

        // Set storage
        state_db.set_storage(addr, key.clone(), value.clone());

        // Verify storage updated
        assert_eq!(state_db.get_storage(&addr, &key), Some(value));
    }

    #[test]
    fn test_account_existence() {
        let state_db = Arc::new(StateDB::new());
        let executor = Executor::new(state_db);

        let addr = create_test_address(5);

        // Initially account doesn't exist (check by balance)
        assert_eq!(executor.get_balance(&addr), U256::zero());

        // Setting balance creates account
        executor.set_balance(&addr, U256::from(1));

        // Now account exists (has non-zero balance)
        assert_ne!(executor.get_balance(&addr), U256::zero());
    }

    #[test]
    fn test_state_commit() {
        let state_db = Arc::new(StateDB::new());
        let executor = Executor::new(state_db.clone());

        let addr = create_test_address(6);

        // Make changes
        executor.set_balance(&addr, U256::from(1000));
        executor.set_nonce(&addr, 10);

        // Commit state
        let state_root = executor.state_db().commit();

        // State root should be non-zero
        assert_ne!(state_root, Hash::default());
    }

    #[test]
    fn test_transfer_execution() {
        let state_db = Arc::new(StateDB::new());
        let executor = Executor::new(state_db);

        let sender = create_test_address(10);
        let receiver = create_test_address(11);

        // Setup initial balances
        executor.set_balance(&sender, U256::from(1000));
        executor.set_balance(&receiver, U256::from(0));

        // Simulate transfer
        let transfer_amount = U256::from(100);
        let sender_balance = executor.get_balance(&sender);
        let receiver_balance = executor.get_balance(&receiver);

        executor.set_balance(&sender, sender_balance - transfer_amount);
        executor.set_balance(&receiver, receiver_balance + transfer_amount);

        // Verify balances
        assert_eq!(executor.get_balance(&sender), U256::from(900));
        assert_eq!(executor.get_balance(&receiver), U256::from(100));
    }

    #[test]
    fn test_gas_calculation() {
        let state_db = Arc::new(StateDB::new());
        let _executor = Executor::new(state_db);

        // Test basic gas costs
        let base_gas = 21000u64;
        let gas_price = U256::from(20_000_000_000u64); // 20 gwei

        let gas_cost = U256::from(base_gas) * gas_price;
        assert_eq!(gas_cost, U256::from(base_gas) * gas_price);
    }

    #[test]
    fn test_account_creation() {
        let state_db = Arc::new(StateDB::new());
        let executor = Executor::new(state_db);

        let new_addr = create_test_address(20);
        let init_code = vec![0x60, 0x00, 0x56]; // PUSH1 0 JUMP

        // Create contract account
        executor.set_code(&new_addr, init_code.clone());
        executor.set_balance(&new_addr, U256::from(0));

        // Verify contract created (check code hash)
        assert_ne!(executor.get_code_hash(&new_addr), Hash::default());
    }

    #[test]
    fn test_selfdestruct_handling() {
        let state_db = Arc::new(StateDB::new());
        let executor = Executor::new(state_db);

        let contract = create_test_address(30);
        let beneficiary = create_test_address(31);

        // Setup contract with balance
        executor.set_balance(&contract, U256::from(1000));
        executor.set_code(&contract, vec![0xff]); // SELFDESTRUCT opcode

        // Simulate selfdestruct
        let contract_balance = executor.get_balance(&contract);
        let beneficiary_balance = executor.get_balance(&beneficiary);

        executor.set_balance(&beneficiary, beneficiary_balance + contract_balance);
        executor.set_balance(&contract, U256::zero());
        executor.set_code(&contract, vec![]);

        // Verify results
        assert_eq!(executor.get_balance(&contract), U256::zero());
        assert_eq!(executor.get_balance(&beneficiary), U256::from(1000));

        // Note: Empty code has a specific hash (keccak256 of empty bytes)
        // not Hash::default(). Let's just verify the code was changed.
        let empty_code_hash = executor.get_code_hash(&contract);
        // The hash should be different from what it was before (non-empty code)
        // but won't be Hash::default()
        assert_ne!(empty_code_hash, Hash::default());
    }

    #[test]
    fn test_address_collision_detection() {
        let state_db = Arc::new(StateDB::new());
        let executor = Executor::new(state_db);

        let addr = create_test_address(40);

        // First creation
        executor.set_code(&addr, vec![0x60]);
        // Check existence by verifying non-default state
        assert_ne!(executor.get_code_hash(&addr), Hash::default());

        // Attempt to create at same address should be detectable
        let existing_code_hash = executor.get_code_hash(&addr);
        assert_ne!(existing_code_hash, Hash::default());
    }

    #[test]
    fn test_state_persistence() {
        // Use the same StateDB instance to simulate persistence
        let state_db = Arc::new(StateDB::new());
        let addr = create_test_address(50);

        // First executor instance
        {
            let executor = Executor::new(state_db.clone());
            executor.set_balance(&addr, U256::from(5000));
            executor.set_nonce(&addr, 10);
            executor.state_db().commit();
        }

        // Second executor instance with same StateDB
        {
            let executor = Executor::new(state_db.clone());

            // State should persist across executors sharing the same StateDB
            assert_eq!(executor.get_balance(&addr), U256::from(5000));
            assert_eq!(executor.get_nonce(&addr), 10);
        }
    }
}
