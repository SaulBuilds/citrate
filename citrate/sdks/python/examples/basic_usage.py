#!/usr/bin/env python3
"""
Basic usage example for Citrate Python SDK

This example demonstrates:
1. Connecting to Citrate network
2. Deploying a simple model
3. Running inference
4. Handling results
"""

import os
import json
from pathlib import Path
from citrate_sdk import CitrateClient, ModelConfig, ModelType, AccessType
from citrate_sdk.crypto import KeyManager


def main():
    """Run basic Citrate SDK example"""

    # Configuration
    RPC_URL = os.getenv("LATTICE_RPC_URL", "http://localhost:8545")
    PRIVATE_KEY = os.getenv("CITRATE_PRIVATE_KEY")

    if not PRIVATE_KEY:
        print("Generating new private key...")
        key_manager = KeyManager()
        PRIVATE_KEY = key_manager.get_private_key()
        print(f"Address: {key_manager.get_address()}")
        print(f"Private Key: {PRIVATE_KEY}")
        print("Save this private key to use again!")

    # Connect to Citrate
    print(f"Connecting to Citrate at {RPC_URL}...")
    client = CitrateClient(rpc_url=RPC_URL, private_key=PRIVATE_KEY)

    # Check connection
    try:
        chain_id = client.get_chain_id()
        print(f"Connected to chain ID: {chain_id}")

        address = client.key_manager.get_address()
        balance = client.get_balance(address)
        print(f"Account balance: {balance / 10**18:.4f} ETH")

    except Exception as e:
        print(f"Connection failed: {e}")
        return

    # Create a dummy model file for demo
    model_path = Path("demo_model.json")
    demo_model = {
        "type": "simple_classifier",
        "version": "1.0",
        "parameters": {
            "input_size": 784,
            "output_classes": 10
        },
        "weights": [0.1] * 100  # Dummy weights
    }

    with open(model_path, 'w') as f:
        json.dump(demo_model, f)

    print(f"Created demo model: {model_path}")

    # Configure model deployment
    config = ModelConfig(
        name="Demo Classifier",
        description="A simple demo classifier model",
        model_type=ModelType.CUSTOM,
        access_type=AccessType.PUBLIC,
        encrypted=False,
        metadata={
            "author": "Citrate SDK Demo",
            "accuracy": 0.95,
            "dataset": "demo_dataset"
        }
    )

    try:
        # Deploy model
        print("Deploying model to Citrate...")
        deployment = client.deploy_model(model_path, config)

        print(f"✅ Model deployed successfully!")
        print(f"Model ID: {deployment.model_id}")
        print(f"Transaction: {deployment.tx_hash}")
        print(f"IPFS Hash: {deployment.ipfs_hash}")

        # Get model info
        print("\nRetrieving model information...")
        model_info = client.get_model_info(deployment.model_id)
        print(f"Model name: {model_info.get('name', 'Unknown')}")
        print(f"Owner: {model_info.get('owner', 'Unknown')}")

        # Run inference
        print("\nExecuting inference...")
        inference_input = {
            "data": [0.5] * 10,  # Dummy input data
            "format": "array"
        }

        result = client.inference(
            model_id=deployment.model_id,
            input_data=inference_input
        )

        print(f"✅ Inference completed!")
        print(f"Output: {result.output_data}")
        print(f"Gas used: {result.gas_used}")
        print(f"Execution time: {result.execution_time}ms")

        # List models
        print("\nListing available models...")
        models = client.list_models(limit=5)

        print(f"Found {len(models)} models:")
        for model in models:
            print(f"  - {model.get('name', 'Unnamed')}: {model.get('model_id')}")

    except Exception as e:
        print(f"❌ Operation failed: {e}")
        return

    finally:
        # Cleanup demo file
        if model_path.exists():
            model_path.unlink()
            print(f"Cleaned up: {model_path}")

    print("\n✅ Demo completed successfully!")


if __name__ == "__main__":
    main()