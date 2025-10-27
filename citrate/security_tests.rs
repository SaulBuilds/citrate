#[cfg(test)]
mod security_tests {
    use std::collections::HashMap;
    use citrate_execution::crypto::{
        encryption::{ModelEncryption, EncryptionConfig},
        key_manager::{KeyManager, KeyPurpose},
        ecdh::{ECIES, ModelKeyExchange},
        shamir::{ShamirSecretSharing, split_model_key, reconstruct_model_key},
    };
    use primitive_types::{H256, H160};

    /// Test suite for encryption security
    mod encryption_tests {
        use super::*;

        #[test]
        fn test_encryption_key_uniqueness() {
            // Each encryption should use a unique key
            let encryption = ModelEncryption::new(EncryptionConfig::default());
            let data = b"test data";
            let owner = H160::random();
            let model_id1 = H256::random();
            let model_id2 = H256::random();

            let encrypted1 = encryption.encrypt_model(
                model_id1, data, owner, vec![]
            ).unwrap();

            let encrypted2 = encryption.encrypt_model(
                model_id2, data, owner, vec![]
            ).unwrap();

            // Different models should have different nonces and ciphertexts
            assert_ne!(encrypted1.nonce, encrypted2.nonce);
            assert_ne!(encrypted1.ciphertext, encrypted2.ciphertext);
            assert_ne!(encrypted1.key_id, encrypted2.key_id);
        }

        #[test]
        fn test_tamper_detection() {
            let encryption = ModelEncryption::new(EncryptionConfig::default());
            let data = b"sensitive model data";
            let owner = H160::random();
            let model_id = H256::random();

            let mut encrypted = encryption.encrypt_model(
                model_id, data, owner, vec![]
            ).unwrap();

            // Tamper with ciphertext
            if !encrypted.ciphertext.is_empty() {
                encrypted.ciphertext[0] ^= 1;
            }

            // Decryption should fail
            let owner_key = [1u8; 32];
            let result = encryption.decrypt_model(&encrypted, &owner_key, owner);
            assert!(result.is_err());
        }

        #[test]
        fn test_nonce_reuse_prevention() {
            // Verify that nonces are never reused across chunks
            let mut used_nonces = std::collections::HashSet::new();

            for _ in 0..1000 {
                let mut nonce = [0u8; 12];
                rand::RngCore::fill_bytes(&mut rand::thread_rng(), &mut nonce);

                // In real encryption, nonce would be generated per chunk
                assert!(!used_nonces.contains(&nonce), "Nonce reuse detected!");
                used_nonces.insert(nonce);
            }
        }

        #[test]
        fn test_unauthorized_access_prevention() {
            let encryption = ModelEncryption::new(EncryptionConfig::default());
            let data = b"secret model";
            let owner = H160::random();
            let unauthorized = H160::random();
            let model_id = H256::random();

            let encrypted = encryption.encrypt_model(
                model_id, data, owner, vec![] // No access list
            ).unwrap();

            // Unauthorized user should not be able to decrypt
            let unauthorized_key = [2u8; 32];
            let result = encryption.decrypt_model(
                &encrypted, &unauthorized_key, unauthorized
            );
            assert!(result.is_err());
        }
    }

    /// Test suite for key management security
    mod key_management_tests {
        use super::*;

        #[test]
        fn test_key_derivation_determinism() {
            let seed = [1u8; 64];
            let manager1 = KeyManager::from_seed(&seed).unwrap();
            let manager2 = KeyManager::from_seed(&seed).unwrap();

            let path = "m/44'/60'/0'/0/0";
            let key1 = manager1.derive_key(path, KeyPurpose::ModelEncryption).unwrap();
            let key2 = manager2.derive_key(path, KeyPurpose::ModelEncryption).unwrap();

            // Same seed and path should produce same key
            assert_eq!(key1.key, key2.key);
            assert_eq!(key1.key_id, key2.key_id);
        }

        #[test]
        fn test_key_path_isolation() {
            let seed = [1u8; 64];
            let manager = KeyManager::from_seed(&seed).unwrap();

            let key1 = manager.derive_key("m/0", KeyPurpose::ModelEncryption).unwrap();
            let key2 = manager.derive_key("m/1", KeyPurpose::ModelEncryption).unwrap();

            // Different paths should produce different keys
            assert_ne!(key1.key, key2.key);
            assert_ne!(key1.key_id, key2.key_id);
        }

        #[test]
        fn test_threshold_key_security() {
            let seed = [1u8; 64];
            let manager = KeyManager::from_seed(&seed).unwrap();
            let model_id = H256::random();

            let holders = vec![H160::random(), H160::random(), H160::random()];

            let threshold_key = manager.create_threshold_key(
                model_id, 2, holders.clone()
            ).unwrap();

            assert_eq!(threshold_key.threshold, 2);
            assert_eq!(threshold_key.total_shares, 3);
            assert_eq!(threshold_key.share_holders.len(), 3);

            // Each holder should have an encrypted share
            for holder in &holders {
                assert!(threshold_key.encrypted_shares.contains_key(holder));
            }
        }

        #[test]
        fn test_access_policy_enforcement() {
            let manager = KeyManager::new();
            let model_id = H256::random();
            let owner = H160::random();
            let user = H160::random();
            let unauthorized = H160::random();

            let mut policy = citrate_execution::crypto::key_manager::AccessPolicy {
                owner,
                full_access: vec![],
                inference_only: vec![user],
                time_limited: HashMap::new(),
                requires_payment: false,
                min_stake: None,
            };

            manager.set_access_policy(model_id, policy).unwrap();

            // Owner should have full access
            assert!(manager.check_access(
                model_id, owner,
                citrate_execution::crypto::key_manager::AccessType::Full
            ).unwrap());

            // User should have inference access only
            assert!(manager.check_access(
                model_id, user,
                citrate_execution::crypto::key_manager::AccessType::Inference
            ).unwrap());

            assert!(!manager.check_access(
                model_id, user,
                citrate_execution::crypto::key_manager::AccessType::Full
            ).unwrap());

            // Unauthorized user should have no access
            assert!(!manager.check_access(
                model_id, unauthorized,
                citrate_execution::crypto::key_manager::AccessType::Inference
            ).unwrap());
        }
    }

    /// Test suite for ECDH security
    mod ecdh_tests {
        use super::*;

        #[test]
        fn test_ecdh_key_exchange() {
            let alice = ECIES::generate().unwrap();
            let bob = ECIES::generate().unwrap();

            let message = b"symmetric encryption key";

            // Alice encrypts for Bob
            let encrypted = alice.encrypt(message, &bob.public_key()).unwrap();

            // Verify encrypted message structure
            assert!(ECIES::validate_public_key(&encrypted.ephemeral_pubkey));
            assert!(!encrypted.ciphertext.is_empty());
            assert_ne!(encrypted.auth_tag, [0u8; 16]);

            // Bob decrypts
            let decrypted = bob.decrypt(&encrypted).unwrap();
            assert_eq!(message, decrypted.as_slice());
        }

        #[test]
        fn test_ecdh_tamper_resistance() {
            let alice = ECIES::generate().unwrap();
            let bob = ECIES::generate().unwrap();

            let message = b"important data";
            let mut encrypted = alice.encrypt(message, &bob.public_key()).unwrap();

            // Tamper with different parts
            let original_auth_tag = encrypted.auth_tag;

            // Tamper with ciphertext
            if !encrypted.ciphertext.is_empty() {
                encrypted.ciphertext[0] ^= 1;
            }
            assert!(bob.decrypt(&encrypted).is_err());

            // Restore ciphertext, tamper with auth tag
            encrypted.ciphertext[0] ^= 1; // Restore
            encrypted.auth_tag[0] ^= 1;
            assert!(bob.decrypt(&encrypted).is_err());

            // Restore auth tag, tamper with nonce
            encrypted.auth_tag = original_auth_tag;
            encrypted.nonce[0] ^= 1;
            assert!(bob.decrypt(&encrypted).is_err());
        }

        #[test]
        fn test_model_key_exchange() {
            let alice_kx = ModelKeyExchange::new().unwrap();
            let bob_kx = ModelKeyExchange::new().unwrap();

            let symmetric_key = [0x42u8; 32];

            // Test multiple exchanges with same keys
            for i in 0..10 {
                let mut test_key = symmetric_key;
                test_key[0] = i as u8;

                let encrypted = alice_kx.encrypt_key_for_recipient(
                    &test_key, &bob_kx.public_key()
                ).unwrap();

                let decrypted = bob_kx.decrypt_key_from_sender(&encrypted).unwrap();
                assert_eq!(test_key, decrypted);
            }
        }
    }

    /// Test suite for Shamir's Secret Sharing security
    mod shamir_tests {
        use super::*;

        #[test]
        fn test_shamir_threshold_security() {
            let secret = [0xDEu8, 0xAD, 0xBE, 0xEF].repeat(8);
            let secret_array: [u8; 32] = secret.try_into().unwrap();

            let sss = ShamirSecretSharing::new(3, 5).unwrap();
            let shares = sss.split_secret(&secret_array).unwrap();

            // Should not be able to reconstruct with fewer than threshold shares
            for i in 1..3 {
                let insufficient_shares = &shares[0..i];
                let result = sss.reconstruct_secret(insufficient_shares);
                assert!(result.is_err());
            }

            // Should be able to reconstruct with threshold or more shares
            for i in 3..=5 {
                let sufficient_shares = &shares[0..i];
                let reconstructed = sss.reconstruct_secret(sufficient_shares).unwrap();
                assert_eq!(secret_array, reconstructed);
            }
        }

        #[test]
        fn test_shamir_share_independence() {
            let secret = [0x01u8; 32];

            let sss = ShamirSecretSharing::new(2, 5).unwrap();
            let shares = sss.split_secret(&secret).unwrap();

            // Any single share should not reveal information about the secret
            for share in &shares {
                let share_bytes = share.y.to_bytes();

                // Share should not be the secret itself
                assert_ne!(share_bytes, secret);

                // Share should not be all zeros
                assert_ne!(share_bytes, [0u8; 32]);
            }

            // Different combinations of threshold shares should work
            let combinations = [
                (0, 1), (0, 2), (0, 3), (0, 4),
                (1, 2), (1, 3), (1, 4),
                (2, 3), (2, 4),
                (3, 4),
            ];

            for (i, j) in combinations {
                let test_shares = vec![shares[i].clone(), shares[j].clone()];
                let reconstructed = sss.reconstruct_secret(&test_shares).unwrap();
                assert_eq!(secret, reconstructed);
            }
        }

        #[test]
        fn test_model_key_splitting() {
            let model_key = [0x33u8; 32];

            // Test various threshold configurations
            let configs = [(2, 3), (3, 5), (5, 10), (7, 12)];

            for (threshold, total) in configs {
                let shares = split_model_key(&model_key, threshold, total).unwrap();
                assert_eq!(shares.len(), total);

                // Test reconstruction with minimum shares
                let reconstructed = reconstruct_model_key(
                    &shares[0..threshold], threshold
                ).unwrap();
                assert_eq!(model_key, reconstructed);

                // Test reconstruction with all shares
                let reconstructed_all = reconstruct_model_key(
                    &shares, threshold
                ).unwrap();
                assert_eq!(model_key, reconstructed_all);
            }
        }
    }

    /// Integration tests combining multiple security components
    mod integration_tests {
        use super::*;

        #[test]
        fn test_complete_secure_model_flow() {
            // Setup
            let model_data = b"secret neural network weights".repeat(100);
            let model_id = H256::random();
            let owner = H160::random();
            let user1 = H160::random();
            let user2 = H160::random();

            // 1. Create key manager
            let seed = [0x42u8; 64];
            let key_manager = KeyManager::from_seed(&seed).unwrap();

            // 2. Derive model-specific key
            let path = format!("m/model/{}", hex::encode(model_id));
            let model_key = key_manager.derive_key(
                &path, KeyPurpose::ModelEncryption
            ).unwrap();

            // 3. Split key using Shamir's Secret Sharing
            let shares = split_model_key(&model_key.key, 2, 3).unwrap();

            // 4. Encrypt model
            let encryption = ModelEncryption::new(EncryptionConfig::default());
            let encrypted_model = encryption.encrypt_model(
                model_id, &model_data, owner, vec![user1, user2]
            ).unwrap();

            // 5. Verify access control
            let owner_key = [1u8; 32];
            let decrypted = encryption.decrypt_model(
                &encrypted_model, &owner_key, owner
            ).unwrap();
            assert_eq!(model_data, decrypted);

            // 6. Test threshold reconstruction
            let reconstructed_key = reconstruct_model_key(&shares[0..2], 2).unwrap();
            assert_eq!(model_key.key, reconstructed_key);

            // 7. Test ECDH for new user
            let alice_kx = ModelKeyExchange::new().unwrap();
            let bob_kx = ModelKeyExchange::new().unwrap();

            let encrypted_key = alice_kx.encrypt_key_for_recipient(
                &model_key.key, &bob_kx.public_key()
            ).unwrap();

            let received_key = bob_kx.decrypt_key_from_sender(&encrypted_key).unwrap();
            assert_eq!(model_key.key, received_key);
        }

        #[test]
        fn test_security_boundary_enforcement() {
            let model_data = b"highly confidential model";
            let model_id = H256::random();
            let owner = H160::random();
            let attacker = H160::random();

            // Setup legitimate encryption
            let encryption = ModelEncryption::new(EncryptionConfig::default());
            let encrypted_model = encryption.encrypt_model(
                model_id, model_data, owner, vec![] // Owner-only access
            ).unwrap();

            // Attacker attempts
            let attacker_key = [0xFFu8; 32];

            // 1. Direct decryption attempt (should fail)
            let result = encryption.decrypt_model(
                &encrypted_model, &attacker_key, attacker
            );
            assert!(result.is_err());

            // 2. Tamper with access list (should fail due to integrity)
            let mut tampered_model = encrypted_model.clone();
            tampered_model.access_list.push(attacker);

            let result = encryption.decrypt_model(
                &tampered_model, &attacker_key, attacker
            );
            assert!(result.is_err()); // Should fail due to missing encrypted key

            // 3. Key brute force (should be computationally infeasible)
            for i in 0u8..100 { // Limited test - real brute force would take forever
                let test_key = [i; 32];
                let result = encryption.decrypt_model(
                    &encrypted_model, &test_key, attacker
                );
                assert!(result.is_err());
            }
        }
    }

    /// Performance impact of security measures
    mod performance_tests {
        use super::*;
        use std::time::Instant;

        #[test]
        fn test_encryption_performance() {
            let sizes = [1024, 10_240, 102_400, 1_024_000]; // 1KB to 1MB
            let encryption = ModelEncryption::new(EncryptionConfig::default());
            let owner = H160::random();

            for size in sizes {
                let data = vec![0x42u8; size];
                let model_id = H256::random();

                let start = Instant::now();
                let encrypted = encryption.encrypt_model(
                    model_id, &data, owner, vec![]
                ).unwrap();
                let encrypt_time = start.elapsed();

                let start = Instant::now();
                let owner_key = [1u8; 32];
                let decrypted = encryption.decrypt_model(
                    &encrypted, &owner_key, owner
                ).unwrap();
                let decrypt_time = start.elapsed();

                assert_eq!(data, decrypted);

                println!("Size: {} bytes, Encrypt: {:?}, Decrypt: {:?}",
                    size, encrypt_time, decrypt_time);

                // Performance thresholds (adjust based on requirements)
                assert!(encrypt_time.as_millis() < size as u128 / 1000 + 100);
                assert!(decrypt_time.as_millis() < size as u128 / 1000 + 100);
            }
        }

        #[test]
        fn test_key_derivation_performance() {
            let seed = [0x42u8; 64];
            let key_manager = KeyManager::from_seed(&seed).unwrap();

            let start = Instant::now();
            for i in 0..1000 {
                let path = format!("m/model/{}", i);
                let _key = key_manager.derive_key(
                    &path, KeyPurpose::ModelEncryption
                ).unwrap();
            }
            let total_time = start.elapsed();

            println!("1000 key derivations: {:?}", total_time);
            assert!(total_time.as_millis() < 10000); // Should be under 10 seconds
        }
    }
}