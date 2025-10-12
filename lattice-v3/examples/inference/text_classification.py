#!/usr/bin/env python3
"""
Text Classification Example using Lattice AI Pipeline
Demonstrates sentiment analysis with DistilBERT on Apple Silicon
"""

import json
import subprocess
import sys
from pathlib import Path

def run_sentiment_analysis():
    """Run sentiment analysis on sample texts."""

    # Sample texts for analysis
    test_texts = [
        "The new MacBook Pro with M3 chip delivers incredible performance for AI workloads.",
        "This product completely failed to meet my expectations. Worst purchase ever.",
        "The weather today is partly cloudy with a chance of rain.",
        "I absolutely love how fast the Neural Engine processes my models!",
        "The documentation could be better, but overall it works as expected."
    ]

    print("🎯 Lattice AI - Sentiment Analysis Example")
    print("=" * 60)
    print()

    # Deploy DistilBERT model
    print("📦 Deploying DistilBERT model...")
    deploy_result = subprocess.run([
        "python3", "tools/import_model.py",
        "huggingface", "distilbert-base-uncased-finetuned-sst-2-english",
        "--optimize"
    ], capture_output=True, text=True)

    if deploy_result.returncode != 0:
        print(f"❌ Deployment failed: {deploy_result.stderr}")
        return

    # Extract model ID from deployment output
    deployment_data = json.loads(deploy_result.stdout)
    model_id = deployment_data["metadata"]["model_hash"]

    print(f"✅ Model deployed with ID: {model_id}")
    print()

    # Run inference on each text
    print("🧠 Running Sentiment Analysis:")
    print("-" * 40)

    for text in test_texts:
        # Prepare input
        input_data = {
            "text": text,
            "task": "sentiment-analysis"
        }

        # Save input to temp file
        input_file = Path("/tmp/sentiment_input.json")
        with open(input_file, "w") as f:
            json.dump(input_data, f)

        # Run inference via CLI
        result = subprocess.run([
            "./target/release/lattice-cli", "model", "inference",
            "--model-id", model_id,
            "--input", str(input_file),
            "--output", "/tmp/sentiment_output.json"
        ], capture_output=True, text=True)

        if result.returncode == 0:
            with open("/tmp/sentiment_output.json") as f:
                output = json.load(f)

            sentiment = output.get("label", "unknown")
            confidence = output.get("score", 0.0)

            # Display with emoji
            emoji = "😊" if sentiment == "POSITIVE" else "😔"
            print(f"\n📝 Text: \"{text[:60]}...\"")
            print(f"   {emoji} Sentiment: {sentiment} (confidence: {confidence:.2%})")
        else:
            print(f"❌ Inference failed for text: {text[:30]}...")

    print()
    print("✨ Sentiment analysis complete!")

if __name__ == "__main__":
    run_sentiment_analysis()