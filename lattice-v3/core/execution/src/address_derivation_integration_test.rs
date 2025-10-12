// lattice-v3/core/execution/src/address_derivation_integration_test.rs

//! Integration tests for address derivation consistency across the entire transaction pipeline
//! This ensures that address handling is unified between wallet generation, transaction creation,
//! API processing, and execution.

#[cfg(test)]
mod tests {
    use crate::address_utils::normalize_address;
    use crate::types::Address;
    use lattice_consensus::types::PublicKey;

    /// Test that demonstrates the key issue: embedded EVM addresses should be handled
    /// consistently throughout the pipeline
    #[test]
    fn test_embedded_evm_address_consistency() {
        // Simulate an EVM address (20 bytes) embedded in a PublicKey (32 bytes)
        // This is how wallets typically send addresses to preserve compatibility
        let evm_address_bytes = [
            0x74, 0x2d, 0x35, 0xcc, 0x66, 0x34, 0xc0, 0x53, 0x29, 0x25, 0xa3, 0xb8, 0x44, 0xbc,
            0x9e, 0x75, 0x95, 0xf0, 0xbe, 0xb1,
        ];

        // Create a PublicKey with embedded address (first 20 bytes = address, last 12 bytes = zero)
        let mut embedded_pubkey_bytes = [0u8; 32];
        embedded_pubkey_bytes[0..20].copy_from_slice(&evm_address_bytes);
        // Last 12 bytes remain zero
        let embedded_pubkey = PublicKey::new(embedded_pubkey_bytes);

        // All these should produce the same address
        let addr_from_normalize = normalize_address(&embedded_pubkey);
        let addr_from_type_method = Address::from_public_key(&embedded_pubkey);
        let expected_addr = Address(evm_address_bytes);

        assert_eq!(addr_from_normalize, expected_addr);
        assert_eq!(addr_from_type_method, expected_addr);
        assert_eq!(addr_from_normalize, addr_from_type_method);

        println!(
            "✓ Embedded EVM address consistency verified: {}",
            addr_from_normalize
        );
    }

    /// Test that real 32-byte public keys are handled consistently
    #[test]
    fn test_real_pubkey_consistency() {
        // Simulate a real 32-byte public key (all non-zero)
        let real_pubkey_bytes = [0x55u8; 32];
        let real_pubkey = PublicKey::new(real_pubkey_bytes);

        // Both methods should derive the same address using Keccak256
        let addr_from_normalize = normalize_address(&real_pubkey);
        let addr_from_type_method = Address::from_public_key(&real_pubkey);

        assert_eq!(addr_from_normalize, addr_from_type_method);

        // Should NOT equal the first 20 bytes since it should be derived
        assert_ne!(addr_from_normalize.0, real_pubkey_bytes[0..20]);

        println!(
            "✓ Real pubkey derivation consistency verified: {}",
            addr_from_normalize
        );
    }

    /// Test various edge cases for address derivation
    #[test]
    fn test_address_edge_cases() {
        // Test all zeros (should be treated as real pubkey, not embedded address)
        let zero_pubkey = PublicKey::new([0u8; 32]);
        let addr_from_zero = normalize_address(&zero_pubkey);
        assert_eq!(addr_from_zero, Address::from_public_key(&zero_pubkey));

        // Test mixed pattern: non-zero in first 20, non-zero in last 12 (real pubkey)
        let mut mixed_bytes = [0u8; 32];
        mixed_bytes[0..20].fill(0xAB);
        mixed_bytes[20..].fill(0xCD); // Non-zero in last 12 bytes
        let mixed_pubkey = PublicKey::new(mixed_bytes);
        let addr_from_mixed = normalize_address(&mixed_pubkey);

        // Should derive address, not use first 20 bytes directly
        assert_ne!(addr_from_mixed.0, mixed_bytes[0..20]);
        assert_eq!(addr_from_mixed, Address::from_public_key(&mixed_pubkey));

        println!("✓ Edge case handling verified");
    }

    /// Test that demonstrates the transaction pipeline scenario
    #[test]
    fn test_transaction_pipeline_consistency() {
        // Simulate wallet creating a transaction with an embedded EVM address
        let wallet_address = [
            0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0, 0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc,
            0xde, 0xf0, 0x12, 0x34, 0x56, 0x78,
        ];

        // Wallet embeds this in a PublicKey for transaction
        let mut tx_from_bytes = [0u8; 32];
        tx_from_bytes[0..20].copy_from_slice(&wallet_address);
        let tx_from_pubkey = PublicKey::new(tx_from_bytes);

        // Transaction recipient (also embedded address)
        let recipient_address = [
            0xfe, 0xdc, 0xba, 0x98, 0x76, 0x54, 0x32, 0x10, 0xfe, 0xdc, 0xba, 0x98, 0x76, 0x54,
            0x32, 0x10, 0xfe, 0xdc, 0xba, 0x98,
        ];
        let mut tx_to_bytes = [0u8; 32];
        tx_to_bytes[0..20].copy_from_slice(&recipient_address);
        let tx_to_pubkey = PublicKey::new(tx_to_bytes);

        // API layer processes the transaction
        let api_from_addr = normalize_address(&tx_from_pubkey);
        let api_to_addr = normalize_address(&tx_to_pubkey);

        // Executor processes the transaction
        let exec_from_addr = Address::from_public_key(&tx_from_pubkey);
        let exec_to_addr = Address::from_public_key(&tx_to_pubkey);

        // All should be consistent
        assert_eq!(api_from_addr, exec_from_addr);
        assert_eq!(api_to_addr, exec_to_addr);
        assert_eq!(api_from_addr.0, wallet_address);
        assert_eq!(api_to_addr.0, recipient_address);

        println!("✓ Transaction pipeline consistency verified");
        println!("  From: {:?} -> {}", tx_from_pubkey, api_from_addr);
        println!("  To:   {:?} -> {}", tx_to_pubkey, api_to_addr);
    }
}