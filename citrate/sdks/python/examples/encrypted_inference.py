#!/usr/bin/env python3
"""
Encrypted inference example for Citrate Python SDK

This example demonstrates:
1. Deploying an encrypted model
2. Setting up access controls
3. Running encrypted inference
4. Key sharing and threshold schemes
"""

import os
import json
from pathlib import Path
from citrate_sdk import CitrateClient, ModelConfig, ModelType, AccessType
from citrate_sdk.crypto import KeyManager, EncryptionConfig


def main():
    """Run encrypted inference example"""

    # Configuration
    RPC_URL = os.getenv("LATTICE_RPC_URL", "http://localhost:8545")
    PRIVATE_KEY = os.getenv("LATTICE_PRIVATE_KEY")

    if not PRIVATE_KEY:
        print("Please set LATTICE_PRIVATE_KEY environment variable")
        return

    # Connect to Citrate
    print(f"Connecting to Citrate at {RPC_URL}...")
    client = CitrateClient(rpc_url=RPC_URL, private_key=PRIVATE_KEY)

    try:
        address = client.key_manager.get_address()
        balance = client.get_balance(address)
        print(f"Account: {address}")
        print(f"Balance: {balance / 10**18:.4f} ETH")

    except Exception as e:
        print(f"Connection failed: {e}")
        return

    # Create a sensitive AI model
    model_path = Path("sensitive_model.json")
    sensitive_model = {
        "type": "proprietary_classifier",
        "version": "2.0",
        "algorithm": "advanced_neural_network",
        "parameters": {
            "layers": [128, 64, 32, 10],
            "activation": "relu",
            "learning_rate": 0.001
        },
        "proprietary_weights": [0.123456] * 1000,  # Sensitive model weights
        "training_data_info": "confidential_dataset_v2"
    }

    with open(model_path, 'w') as f:
        json.dump(sensitive_model, f, indent=2)

    print(f"Created sensitive model: {model_path}")

    # Configure encryption with threshold sharing
    encryption_config = EncryptionConfig(
        algorithm="AES-256-GCM",
        key_derivation="HKDF-SHA256",
        access_control=True,
        threshold_shares=2,  # Need 2 shares to decrypt
        total_shares=3       # Create 3 total shares
    )

    # Configure encrypted model deployment
    config = ModelConfig(
        name="Sensitive AI Model",
        description="Proprietary model with encrypted weights",
        model_type=ModelType.CUSTOM,
        access_type=AccessType.PAID,
        access_price=500000000000000000,  # 0.5 ETH per inference
        encrypted=True,
        encryption_config=encryption_config,
        metadata={
            "sensitivity_level": "high",
            "compliance": "GDPR,CCPA",
            "encryption": "AES-256-GCM"
        }
    )

    try:
        # Deploy encrypted model
        print("\n🔒 Deploying encrypted model...")
        deployment = client.deploy_model(model_path, config)

        print(f"✅ Encrypted model deployed!")
        print(f"Model ID: {deployment.model_id}")
        print(f"Transaction: {deployment.tx_hash}")
        print(f"Encrypted: {deployment.encrypted}")

        # Demonstrate key sharing scenario
        print("\n🔑 Demonstrating key sharing...")

        # Create additional key managers (simulating other parties)
        alice = KeyManager()
        bob = KeyManager()

        print(f"Alice address: {alice.get_address()}")
        print(f"Bob address: {bob.get_address()}")

        # Get shared keys for secure communication
        owner_pubkey = client.key_manager.get_public_key()
        alice_pubkey = alice.get_public_key()
        bob_pubkey = bob.get_public_key()

        # Derive shared keys
        alice_shared = client.key_manager.derive_shared_key(alice_pubkey)
        bob_shared = client.key_manager.derive_shared_key(bob_pubkey)

        print(f"Shared key with Alice: {alice_shared[:8].hex()}...")
        print(f"Shared key with Bob: {bob_shared[:8].hex()}...")

        # Run encrypted inference
        print("\n🧠 Running encrypted inference...")

        # Prepare sensitive input data
        sensitive_input = {
            "patient_data": {
                "age": 45,
                "symptoms": ["fever", "cough"],
                "medical_history": "confidential"
            },
            "analysis_level": "detailed"
        }

        # Execute encrypted inference
        result = client.inference(
            model_id=deployment.model_id,
            input_data=sensitive_input,
            encrypted=True,
            max_gas=1500000
        )

        print(f"✅ Encrypted inference completed!")
        print(f"Output (encrypted): {str(result.output_data)[:100]}...")
        print(f"Gas used: {result.gas_used}")
        print(f"Execution time: {result.execution_time}ms")

        # Demonstrate data encryption utilities
        print("\n🔐 Testing encryption utilities...")

        # Encrypt arbitrary data
        test_data = "This is sensitive information that needs protection"
        encrypted_data = client.key_manager.encrypt_data(test_data)
        decrypted_data = client.key_manager.decrypt_data(encrypted_data)

        print(f"Original: {test_data}")
        print(f"Encrypted: {encrypted_data[:50]}...")
        print(f"Decrypted: {decrypted_data}")
        print(f"Match: {test_data == decrypted_data}")

        # Test model integrity
        print("\n🛡️  Testing model integrity...")
        from citrate_sdk.crypto import hash_model_data, verify_model_integrity

        model_data = model_path.read_bytes()
        model_hash = hash_model_data(model_data)
        is_valid = verify_model_integrity(model_data, model_hash)

        print(f"Model hash: {model_hash}")
        print(f"Integrity check: {is_valid}")

    except Exception as e:
        print(f"❌ Operation failed: {e}")
        import traceback
        traceback.print_exc()

    finally:
        # Cleanup
        if model_path.exists():
            model_path.unlink()
            print(f"Cleaned up: {model_path}")

    print("\n✅ Encrypted inference demo completed!")


if __name__ == "__main__":
    main()