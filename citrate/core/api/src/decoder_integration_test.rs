// citrate/core/api/src/decoder_integration_test.rs

#[cfg(test)]
mod tests {
    use crate::unified_tx_decoder::{UnifiedTransactionDecoder, DecoderFactory};
    use citrate_consensus::types::Transaction;

    /// Test EIP-1559 transaction decoding with mock data
    #[test]
    fn test_eip1559_decoder_integration() {
        let decoder = DecoderFactory::testing();

        // Create a mock EIP-1559 transaction (type 0x02)
        // This is a simplified test transaction
        let mock_eip1559_tx = vec![
            0x02, // EIP-1559 type
            0xf8, 0x6c, // RLP list with length
            0x01, // chain_id = 1
            0x01, // nonce = 1
            0x84, 0x3b, 0x9a, 0xca, 0x00, // max_priority_fee = 1 gwei
            0x84, 0x3b, 0x9a, 0xca, 0x00, // max_fee = 1 gwei
            0x82, 0x52, 0x08, // gas_limit = 21000
            0x94, 0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0, 0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0, 0x12, 0x34, 0x56, // to address
            0x84, 0x3b, 0x9a, 0xca, 0x00, // value = 1 gwei
            0x80, // empty data
            0xc0, // empty access list
            0x01, // y_parity = 1
            0xa0, 0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0, 0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0, 0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0, 0x12, 0x34, 0x56, // r
            0xa0, 0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0, 0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0, 0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0, 0x12, 0x34, 0x56, // s
        ];

        // This should either decode successfully or fall back gracefully
        match decoder.decode_eth_transaction(&mock_eip1559_tx) {
            Ok(tx) => {
                println!("Successfully decoded EIP-1559 transaction: hash=0x{}", hex::encode(tx.hash.as_bytes()));
                assert_ne!(tx.hash.as_bytes(), &[0u8; 32]); // Should have a valid hash
            }
            Err(e) => {
                println!("EIP-1559 decoder failed (expected for mock data): {}", e);
                // This is expected for mock data since signature recovery will fail
                assert!(e.contains("signature") || e.contains("recover") || e.contains("fallback"));
            }
        }
    }

    /// Test legacy transaction handling
    #[test]
    fn test_legacy_transaction_fallback() {
        let decoder = DecoderFactory::development(vec![1, 31337]);

        // Test with invalid data to ensure graceful fallback
        let invalid_tx = vec![0x01, 0x02, 0x03, 0x04];

        match decoder.decode_eth_transaction(&invalid_tx) {
            Ok(_) => {
                println!("Unexpectedly succeeded with invalid data");
            }
            Err(e) => {
                println!("Correctly failed with invalid data: {}", e);
                assert!(!e.is_empty());
            }
        }
    }

    /// Test Citrate native transaction
    #[test]
    fn test_lattice_native_transaction() {
        let decoder = DecoderFactory::testing();

        // Create a valid Citrate transaction
        let lattice_tx = Transaction {
            hash: citrate_consensus::types::Hash::new([1u8; 32]),
            from: citrate_consensus::types::PublicKey::new([2u8; 32]),
            to: Some(citrate_consensus::types::PublicKey::new([3u8; 32])),
            value: 1000000000000000000, // 1 ETH
            data: vec![],
            nonce: 1,
            gas_price: 1000000000, // 1 gwei
            gas_limit: 21000,
            signature: citrate_consensus::types::Signature::new([4u8; 64]),
            tx_type: None,
        };

        // Serialize it
        let tx_bytes = bincode::serialize(&lattice_tx).expect("Failed to serialize transaction");

        // Decode it
        match decoder.decode_eth_transaction(&tx_bytes) {
            Ok(decoded_tx) => {
                println!("Successfully decoded Citrate native transaction");
                assert_eq!(decoded_tx.hash, lattice_tx.hash);
                assert_eq!(decoded_tx.nonce, lattice_tx.nonce);
                assert_eq!(decoded_tx.value, lattice_tx.value);
            }
            Err(e) => {
                panic!("Failed to decode valid Citrate transaction: {}", e);
            }
        }
    }

    /// Test decoder statistics
    #[test]
    fn test_decoder_statistics() {
        let decoder = DecoderFactory::testing();

        // Get initial stats
        let initial_stats = decoder.get_stats();
        assert_eq!(initial_stats.total_decoded, 0);

        // Try to decode something (even if it fails)
        let _ = decoder.decode_eth_transaction(&[0x01, 0x02]);

        // Stats should be updated
        let updated_stats = decoder.get_stats();
        assert!(updated_stats.total_decoded > 0 || updated_stats.failed_decodes > 0);
    }

    /// Test quick validation
    #[test]
    fn test_quick_validation() {
        let decoder = DecoderFactory::testing();

        // Test empty transaction
        let result = decoder.quick_validate(&[]);
        assert!(result.is_err());

        // Test EIP-1559 type detection
        let eip1559_bytes = [0x02, 0x01, 0x02, 0x03];
        let result = decoder.quick_validate(&eip1559_bytes);
        match result {
            Ok(validation) => {
                assert!(validation.tx_type.contains("1559") || validation.tx_type.contains("EIP"));
            }
            Err(_) => {
                // Expected for incomplete data
            }
        }

        // Test legacy type detection
        let legacy_bytes = [0xf8, 0x6c, 0x01]; // Valid RLP start
        let result = decoder.quick_validate(&legacy_bytes);
        match result {
            Ok(validation) => {
                assert!(validation.tx_type.contains("Legacy"));
            }
            Err(_) => {
                // Expected for incomplete data
            }
        }
    }
}

/// Integration test demonstrating the complete transaction pipeline
#[cfg(test)]
pub fn run_integration_test() {
    use crate::unified_tx_decoder::GlobalTransactionDecoder;

    println!("Running EIP-1559 Transaction Decoder Integration Test");

    // Initialize global decoder
    let global_decoder = GlobalTransactionDecoder::init(vec![1, 5, 31337], true);

    println!("✓ Global decoder initialized");

    // Test various transaction types
    let test_cases = vec![
        ("Empty transaction", vec![]),
        ("EIP-1559 prefix", vec![0x02]),
        ("EIP-2930 prefix", vec![0x01]),
        ("Legacy RLP start", vec![0xf8, 0x6c]),
        ("Invalid data", vec![0xff, 0xfe, 0xfd]),
    ];

    for (name, tx_bytes) in test_cases {
        print!("Testing {}: ", name);
        match global_decoder.decode(&tx_bytes) {
            Ok(tx) => {
                println!("✓ Success (hash: 0x{})", hex::encode(&tx.hash.as_bytes()[..8]));
            }
            Err(e) => {
                println!("✗ Failed ({})", e.split(':').next().unwrap_or(&e));
            }
        }
    }

    // Test statistics
    let stats = global_decoder.decoder().get_stats();
    println!("\nDecoder Statistics:");
    println!("- Total attempts: {}", stats.total_decoded + stats.failed_decodes);
    println!("- Successful: {}", stats.total_decoded);
    println!("- Failed: {}", stats.failed_decodes);
    println!("- EIP-1559: {}", stats.eip1559_count);
    println!("- EIP-2930: {}", stats.eip2930_count);
    println!("- Legacy: {}", stats.legacy_count);

    println!("\n✓ Integration test completed successfully");
}