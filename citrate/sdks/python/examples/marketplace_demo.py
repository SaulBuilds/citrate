#!/usr/bin/env python3
"""
Marketplace demo for Citrate Python SDK

This example demonstrates:
1. Browsing model marketplace
2. Purchasing model access
3. Revenue sharing setup
4. Model rating and reviews
"""

import os
import time
from citrate_sdk import CitrateClient, ModelConfig, ModelType, AccessType
from citrate_sdk.crypto import KeyManager


def main():
    """Run marketplace demo"""

    # Configuration
    RPC_URL = os.getenv("LATTICE_RPC_URL", "http://localhost:8545")

    # Create multiple accounts for marketplace simulation
    print("Creating marketplace participants...")

    # Model seller
    seller = CitrateClient(rpc_url=RPC_URL, private_key=KeyManager().get_private_key())
    print(f"Seller: {seller.key_manager.get_address()}")

    # Model buyer
    buyer = CitrateClient(rpc_url=RPC_URL, private_key=KeyManager().get_private_key())
    print(f"Buyer: {buyer.key_manager.get_address()}")

    # Revenue partner (e.g., dataset provider)
    partner = KeyManager()
    print(f"Partner: {partner.get_address()}")

    try:
        # Check balances
        seller_balance = seller.get_balance(seller.key_manager.get_address())
        buyer_balance = buyer.get_balance(buyer.key_manager.get_address())

        print(f"\nInitial balances:")
        print(f"Seller: {seller_balance / 10**18:.4f} ETH")
        print(f"Buyer: {buyer_balance / 10**18:.4f} ETH")

        # Seller deploys a premium model with revenue sharing
        print("\n💼 Seller: Deploying premium model with revenue sharing...")

        revenue_shares = {
            seller.key_manager.get_address(): 0.70,  # 70% to model creator
            partner.get_address(): 0.25,             # 25% to data provider
            "0x0000000000000000000000000000000000000001": 0.05  # 5% to platform
        }

        config = ModelConfig(
            name="Premium Image Classifier",
            description="High-accuracy image classification model trained on premium dataset",
            model_type=ModelType.COREML,
            access_type=AccessType.PAID,
            access_price=100000000000000000,  # 0.1 ETH per inference
            revenue_shares=revenue_shares,
            metadata={
                "accuracy": 0.98,
                "dataset_size": "1M images",
                "training_time": "100 GPU hours",
                "category": "computer_vision"
            },
            tags=["image", "classification", "premium", "high-accuracy"]
        )

        # Create dummy model file
        import json
        from pathlib import Path

        model_path = Path("premium_classifier.json")
        premium_model = {
            "type": "image_classifier",
            "architecture": "ResNet-50",
            "input_size": [224, 224, 3],
            "output_classes": 1000,
            "accuracy": 0.98,
            "model_size": "25MB"
        }

        with open(model_path, 'w') as f:
            json.dump(premium_model, f)

        deployment = seller.deploy_model(model_path, config)

        print(f"✅ Premium model deployed!")
        print(f"Model ID: {deployment.model_id}")
        print(f"Price: {config.access_price / 10**18} ETH per inference")

        # Browse marketplace
        print("\n🛒 Buyer: Browsing marketplace...")

        # List available models
        available_models = buyer.list_models(limit=10)
        print(f"Found {len(available_models)} models in marketplace:")

        for model in available_models:
            print(f"  📦 {model.get('name', 'Unnamed')}")
            print(f"     ID: {model.get('model_id')}")
            print(f"     Price: {model.get('access_price', 0) / 10**18} ETH")
            print(f"     Owner: {model.get('owner', 'Unknown')}")

        # Get detailed model info
        print(f"\n🔍 Examining model: {deployment.model_id}")
        model_info = buyer.get_model_info(deployment.model_id)

        print(f"Model details:")
        print(f"  Name: {model_info.get('name')}")
        print(f"  Description: {model_info.get('description')}")
        print(f"  Price: {model_info.get('access_price', 0) / 10**18} ETH")
        print(f"  Total inferences: {model_info.get('total_inferences', 0)}")
        print(f"  Revenue: {model_info.get('total_revenue', 0) / 10**18} ETH")

        # Purchase model access
        print(f"\n💳 Buyer: Purchasing access to model...")

        purchase_tx = buyer.purchase_model_access(
            model_id=deployment.model_id,
            payment_amount=config.access_price
        )

        print(f"✅ Access purchased! Transaction: {purchase_tx}")

        # Wait for transaction confirmation
        print("Waiting for transaction confirmation...")
        time.sleep(5)

        # Use the model
        print(f"\n🧠 Buyer: Running inference on purchased model...")

        inference_input = {
            "image": "base64_encoded_image_data_here",
            "format": "jpg",
            "preprocessing": "resize_224x224"
        }

        result = buyer.inference(
            model_id=deployment.model_id,
            input_data=inference_input
        )

        print(f"✅ Inference completed!")
        print(f"Classification: {result.output_data.get('class', 'unknown')}")
        print(f"Confidence: {result.output_data.get('confidence', 0)}")
        print(f"Gas used: {result.gas_used}")

        # Simulate multiple users and usage
        print(f"\n📊 Simulating marketplace activity...")

        # Create more buyers
        buyers = []
        for i in range(3):
            buyer_client = CitrateClient(rpc_url=RPC_URL, private_key=KeyManager().get_private_key())
            buyers.append(buyer_client)

        # Simulate purchases and usage
        total_revenue = 0
        for i, buyer_client in enumerate(buyers):
            print(f"User {i+1}: Purchasing and using model...")

            try:
                # Purchase access
                purchase_tx = buyer_client.purchase_model_access(
                    model_id=deployment.model_id,
                    payment_amount=config.access_price
                )

                # Run inference
                result = buyer_client.inference(
                    model_id=deployment.model_id,
                    input_data={"image": f"test_image_{i+1}"}
                )

                total_revenue += config.access_price
                print(f"  ✅ User {i+1} completed inference")

            except Exception as e:
                print(f"  ❌ User {i+1} failed: {e}")

        # Check final marketplace stats
        print(f"\n📈 Final marketplace statistics:")

        updated_model_info = seller.get_model_info(deployment.model_id)
        print(f"Total inferences: {updated_model_info.get('total_inferences', 0)}")
        print(f"Total revenue: {updated_model_info.get('total_revenue', 0) / 10**18} ETH")

        # Check seller revenue
        final_seller_balance = seller.get_balance(seller.key_manager.get_address())
        revenue_earned = (final_seller_balance - seller_balance) / 10**18

        print(f"Seller revenue earned: {revenue_earned:.4f} ETH")

        print(f"\n💡 Revenue sharing breakdown (per inference):")
        for address, percentage in revenue_shares.items():
            amount = (config.access_price * percentage) / 10**18
            print(f"  {address[:10]}...: {percentage*100}% = {amount:.4f} ETH")

    except Exception as e:
        print(f"❌ Marketplace demo failed: {e}")
        import traceback
        traceback.print_exc()

    finally:
        # Cleanup
        if 'model_path' in locals() and model_path.exists():
            model_path.unlink()

    print("\n✅ Marketplace demo completed!")


if __name__ == "__main__":
    main()