// lattice-v3/core/execution/src/vm/evm_tests.rs

#[cfg(test)]
mod tests {
    use crate::state::StateDB;
    use crate::types::Address;
    use crate::vm::EVMIntegration;
    use primitive_types::U256;
    use std::sync::Arc;

    #[test]
    fn test_evm_arithmetic_operations() {
        let state_db = Arc::new(StateDB::new());
        let mut integration = EVMIntegration::new(state_db);

        // Test ADD: PUSH1 5 PUSH1 3 ADD PUSH1 0 MSTORE PUSH1 32 PUSH1 0 RETURN
        let code = vec![
            0x60, 0x05, // PUSH1 5
            0x60, 0x03, // PUSH1 3
            0x01,       // ADD
            0x60, 0x00, // PUSH1 0
            0x52,       // MSTORE
            0x60, 0x20, // PUSH1 32
            0x60, 0x00, // PUSH1 0
            0xf3,       // RETURN
        ];

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

        // Check that the result is 8 (5 + 3)
        let mut expected = [0u8; 32];
        U256::from(8).to_big_endian(&mut expected);
        assert_eq!(output, expected);
        assert!(gas_used > 0);
    }

    #[test]
    fn test_evm_memory_operations() {
        let state_db = Arc::new(StateDB::new());
        let mut integration = EVMIntegration::new(state_db);

        // Test MSTORE/MLOAD: PUSH1 0x42 PUSH1 0 MSTORE PUSH1 0 MLOAD PUSH1 0 MSTORE PUSH1 32 PUSH1 0 RETURN
        let code = vec![
            0x60, 0x42, // PUSH1 0x42
            0x60, 0x00, // PUSH1 0
            0x52,       // MSTORE
            0x60, 0x00, // PUSH1 0
            0x51,       // MLOAD
            0x60, 0x00, // PUSH1 0
            0x52,       // MSTORE
            0x60, 0x20, // PUSH1 32
            0x60, 0x00, // PUSH1 0
            0xf3,       // RETURN
        ];

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

        // Check that we stored and retrieved 0x42
        let mut expected = [0u8; 32];
        U256::from(0x42).to_big_endian(&mut expected);
        assert_eq!(output, expected);
    }

    #[test]
    fn test_evm_calldata_operations() {
        let state_db = Arc::new(StateDB::new());
        let mut integration = EVMIntegration::new(state_db);

        // Test CALLDATALOAD: PUSH1 0 CALLDATALOAD PUSH1 0 MSTORE PUSH1 32 PUSH1 0 RETURN
        let code = vec![
            0x60, 0x00, // PUSH1 0
            0x35,       // CALLDATALOAD
            0x60, 0x00, // PUSH1 0
            0x52,       // MSTORE
            0x60, 0x20, // PUSH1 32
            0x60, 0x00, // PUSH1 0
            0xf3,       // RETURN
        ];

        let mut input_data = vec![0u8; 32];
        U256::from(0x123456).to_big_endian(&mut input_data);

        let caller = Address([1u8; 20]);
        let contract = Address([2u8; 20]);

        let result = integration.execute(
            &code,
            &input_data,
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

        // Check that we loaded the input data correctly
        assert_eq!(output, input_data);
    }

    #[test]
    fn test_evm_context_operations() {
        let state_db = Arc::new(StateDB::new());
        let mut integration = EVMIntegration::new(state_db);

        // Test ADDRESS: ADDRESS PUSH1 0 MSTORE PUSH1 32 PUSH1 0 RETURN
        let code = vec![
            0x30,       // ADDRESS
            0x60, 0x00, // PUSH1 0
            0x52,       // MSTORE
            0x60, 0x20, // PUSH1 32
            0x60, 0x00, // PUSH1 0
            0xf3,       // RETURN
        ];

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

        // Check that ADDRESS returned the contract address
        let mut expected = [0u8; 32];
        expected[12..].copy_from_slice(&contract.0);
        assert_eq!(output, expected);
    }

    #[test]
    fn test_evm_balance_lookup() {
        let state_db = Arc::new(StateDB::new());
        let mut integration = EVMIntegration::new(state_db.clone());

        // Set up test balance
        let test_addr = Address([5u8; 20]);
        let test_balance = U256::from(98765u64);
        state_db.accounts.set_balance(test_addr, test_balance);

        // Test BALANCE: PUSH20 address BALANCE PUSH1 0 MSTORE PUSH1 32 PUSH1 0 RETURN
        let mut code = vec![0x73]; // PUSH20
        code.extend_from_slice(&test_addr.0);
        code.extend_from_slice(&[
            0x31,       // BALANCE
            0x60, 0x00, // PUSH1 0
            0x52,       // MSTORE
            0x60, 0x20, // PUSH1 32
            0x60, 0x00, // PUSH1 0
            0xf3,       // RETURN
        ]);

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

        // Check that BALANCE returned the correct balance
        let mut expected = [0u8; 32];
        test_balance.to_big_endian(&mut expected);
        assert_eq!(output, expected);
    }

    #[test]
    fn test_evm_comparison_operations() {
        let state_db = Arc::new(StateDB::new());
        let mut integration = EVMIntegration::new(state_db);

        // Test LT (less than): PUSH1 10 PUSH1 5 LT PUSH1 0 MSTORE PUSH1 32 PUSH1 0 RETURN
        let code = vec![
            0x60, 0x0a, // PUSH1 10
            0x60, 0x05, // PUSH1 5
            0x10,       // LT (5 < 10 should be true = 1)
            0x60, 0x00, // PUSH1 0
            0x52,       // MSTORE
            0x60, 0x20, // PUSH1 32
            0x60, 0x00, // PUSH1 0
            0xf3,       // RETURN
        ];

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

        // Check that LT returned 1 (true)
        let mut expected = [0u8; 32];
        U256::from(1).to_big_endian(&mut expected);
        assert_eq!(output, expected);
    }

    #[test]
    fn test_evm_bitwise_operations() {
        let state_db = Arc::new(StateDB::new());
        let mut integration = EVMIntegration::new(state_db);

        // Test AND: PUSH1 0x0F PUSH1 0xFF AND PUSH1 0 MSTORE PUSH1 32 PUSH1 0 RETURN
        let code = vec![
            0x60, 0x0f, // PUSH1 0x0F
            0x60, 0xff, // PUSH1 0xFF
            0x16,       // AND (0xFF & 0x0F = 0x0F)
            0x60, 0x00, // PUSH1 0
            0x52,       // MSTORE
            0x60, 0x20, // PUSH1 32
            0x60, 0x00, // PUSH1 0
            0xf3,       // RETURN
        ];

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

        // Check that AND returned 0x0F
        let mut expected = [0u8; 32];
        U256::from(0x0f).to_big_endian(&mut expected);
        assert_eq!(output, expected);
    }

    #[test]
    fn test_evm_stack_operations() {
        let state_db = Arc::new(StateDB::new());
        let mut integration = EVMIntegration::new(state_db);

        // Test DUP1: PUSH1 0x42 DUP1 ADD PUSH1 0 MSTORE PUSH1 32 PUSH1 0 RETURN
        let code = vec![
            0x60, 0x42, // PUSH1 0x42
            0x80,       // DUP1 (duplicate top item)
            0x01,       // ADD (0x42 + 0x42 = 0x84)
            0x60, 0x00, // PUSH1 0
            0x52,       // MSTORE
            0x60, 0x20, // PUSH1 32
            0x60, 0x00, // PUSH1 0
            0xf3,       // RETURN
        ];

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

        // Check that DUP1 + ADD returned 0x84 (0x42 + 0x42)
        let mut expected = [0u8; 32];
        U256::from(0x84).to_big_endian(&mut expected);
        assert_eq!(output, expected);
    }

    #[test]
    fn test_evm_gas_tracking() {
        let state_db = Arc::new(StateDB::new());
        let mut integration = EVMIntegration::new(state_db);

        // Simple operation with known gas cost
        let code = vec![
            0x60, 0x01, // PUSH1 1 (gas: 3)
            0x60, 0x02, // PUSH1 2 (gas: 3)
            0x01,       // ADD     (gas: 3)
            0x00,       // STOP    (gas: 0)
        ];

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
        let (_, gas_used) = result.unwrap();

        // Should have used at least 9 gas (3 + 3 + 3)
        assert!(gas_used >= 9);
    }

    #[test]
    fn test_evm_out_of_gas() {
        let state_db = Arc::new(StateDB::new());
        let mut integration = EVMIntegration::new(state_db);

        // Simple operation but with very low gas limit
        let code = vec![
            0x60, 0x01, // PUSH1 1
            0x60, 0x02, // PUSH1 2
            0x01,       // ADD
        ];

        let caller = Address([1u8; 20]);
        let contract = Address([2u8; 20]);

        let result = integration.execute(
            &code,
            &[],
            caller,
            contract,
            U256::zero(),
            5, // Very low gas limit
            U256::from(1000000000u64),
            caller,
            100,
            1000000,
            [0u8; 32],
            [0u8; 20],
            1,
            U256::from(1000000000u64),
        );

        // Should fail with out of gas
        assert!(result.is_err());
        if let Err(e) = result {
            assert!(matches!(e, crate::types::ExecutionError::OutOfGas));
        }
    }
}