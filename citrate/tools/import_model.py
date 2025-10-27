#!/usr/bin/env python3
"""
Import and deploy AI models to Citrate blockchain.
Supports HuggingFace models with automatic CoreML conversion for Apple Silicon.
"""

import argparse
import json
import os
import subprocess
import sys
import hashlib
from pathlib import Path
from typing import Dict, Any, Optional
import requests
import tempfile
import shutil

try:
    from web3 import Web3
    import ipfshttpclient
except ImportError as e:
    print(f"Missing dependency: {e}")
    print("Install with: pip install web3 ipfshttpclient")
    sys.exit(1)

# Default configuration
DEFAULT_RPC = "http://localhost:8545"
DEFAULT_IPFS = "/ip4/127.0.0.1/tcp/5001"

# Popular models optimized for Mac
RECOMMENDED_MODELS = {
    "text": [
        {
            "name": "bert-base-uncased",
            "description": "General text understanding",
            "size_mb": 440,
            "use_case": "Classification, NER, Q&A",
        },
        {
            "name": "distilbert-base-uncased",
            "description": "Lightweight BERT",
            "size_mb": 265,
            "use_case": "Fast text processing",
        },
        {
            "name": "microsoft/deberta-v3-base",
            "description": "Enhanced BERT",
            "size_mb": 500,
            "use_case": "High-accuracy NLP",
        },
    ],
    "generation": [
        {
            "name": "distilgpt2",
            "description": "Lightweight GPT-2",
            "size_mb": 350,
            "use_case": "Text generation",
        },
        {
            "name": "microsoft/phi-2",
            "description": "Small but capable LLM",
            "size_mb": 2800,
            "use_case": "Code & text generation",
        },
    ],
    "vision": [
        {
            "name": "google/vit-base-patch16-224",
            "description": "Vision Transformer",
            "size_mb": 350,
            "use_case": "Image classification",
        },
        {
            "name": "microsoft/resnet-50",
            "description": "Classic CNN",
            "size_mb": 100,
            "use_case": "Fast image classification",
        },
    ],
    "multimodal": [
        {
            "name": "openai/clip-vit-base-patch32",
            "description": "Vision-Language model",
            "size_mb": 600,
            "use_case": "Image-text matching",
        },
    ],
}


class ModelImporter:
    """Import and deploy models to Citrate."""
    
    def __init__(self, rpc_url: str = DEFAULT_RPC, ipfs_api: str = DEFAULT_IPFS):
        self.rpc_url = rpc_url
        self.ipfs_api = ipfs_api
        
        # Connect to IPFS
        try:
            self.ipfs = ipfshttpclient.connect(ipfs_api)
            print(f"Connected to IPFS at {ipfs_api}")
        except Exception as e:
            print(f"Failed to connect to IPFS: {e}")
            print("Make sure IPFS daemon is running: ipfs daemon")
            sys.exit(1)
        
        # Connect to blockchain
        try:
            self.w3 = Web3(Web3.HTTPProvider(rpc_url))
            if not self.w3.is_connected():
                raise Exception("Not connected")
            print(f"Connected to Citrate at {rpc_url}")
        except Exception as e:
            print(f"Failed to connect to Citrate node: {e}")
            print("Make sure Citrate node is running")
            sys.exit(1)
    
    def import_from_huggingface(self, model_name: str, optimize_neural_engine: bool = False) -> Dict[str, Any]:
        """Import a model from HuggingFace."""
        print(f"\n🤖 Importing {model_name} from HuggingFace...")
        
        with tempfile.TemporaryDirectory() as temp_dir:
            # Step 1: Convert to CoreML
            print("\n1️⃣ Converting to CoreML...")
            coreml_path = self._convert_to_coreml(
                model_name, 
                temp_dir, 
                optimize_neural_engine
            )
            
            # Step 2: Upload to IPFS
            print("\n2️⃣ Uploading to IPFS...")
            ipfs_cid = self._upload_to_ipfs(coreml_path)
            
            # Step 3: Create metadata
            print("\n3️⃣ Creating metadata...")
            metadata = self._create_metadata(model_name, coreml_path, ipfs_cid)
            
            # Step 4: Register on-chain
            print("\n4️⃣ Registering on blockchain...")
            tx_hash = self._register_on_chain(metadata)
            
            return {
                "model_name": model_name,
                "ipfs_cid": ipfs_cid,
                "tx_hash": tx_hash,
                "metadata": metadata,
            }
    
    def _convert_to_coreml(self, model_name: str, output_dir: str, optimize: bool) -> Path:
        """Convert model to CoreML format."""
        # Run the converter script
        cmd = [
            sys.executable,
            "convert_to_coreml.py",
            model_name,
            "--output-dir", output_dir,
        ]
        
        if optimize:
            cmd.append("--optimize-neural-engine")
        
        try:
            result = subprocess.run(
                cmd,
                cwd=Path(__file__).parent,
                capture_output=True,
                text=True,
                check=True,
            )
            print(result.stdout)
        except subprocess.CalledProcessError as e:
            print(f"Conversion failed: {e.stderr}")
            raise
        
        # Find the generated .mlpackage
        model_dir = Path(output_dir)
        mlpackages = list(model_dir.glob("*.mlpackage"))
        
        if not mlpackages:
            raise FileNotFoundError("No .mlpackage file generated")
        
        return mlpackages[0]
    
    def _upload_to_ipfs(self, model_path: Path) -> str:
        """Upload model to IPFS."""
        # Create a tar archive of the model package
        import tarfile
        
        tar_path = model_path.parent / f"{model_path.stem}.tar.gz"
        
        with tarfile.open(tar_path, "w:gz") as tar:
            tar.add(model_path, arcname=model_path.name)
        
        # Upload to IPFS
        result = self.ipfs.add(str(tar_path))
        cid = result["Hash"]
        
        # Pin the content
        self.ipfs.pin.add(cid)
        
        print(f"Model uploaded to IPFS: {cid}")
        print(f"Size: {result['Size']} bytes")
        
        return cid
    
    def _create_metadata(self, model_name: str, model_path: Path, ipfs_cid: str) -> Dict[str, Any]:
        """Create model metadata."""
        # Calculate model hash
        hasher = hashlib.sha256()
        with open(model_path, "rb") as f:
            for chunk in iter(lambda: f.read(4096), b""):
                hasher.update(chunk)
        
        model_hash = hasher.hexdigest()
        
        # Get file size
        size_bytes = model_path.stat().st_size
        
        # Determine model type
        model_type = "text"  # Default
        if "vit" in model_name.lower() or "resnet" in model_name.lower():
            model_type = "vision"
        elif "clip" in model_name.lower():
            model_type = "multimodal"
        elif "gpt" in model_name.lower() or "phi" in model_name.lower():
            model_type = "generation"
        
        metadata = {
            "name": model_name,
            "model_hash": model_hash,
            "ipfs_cid": ipfs_cid,
            "size_bytes": size_bytes,
            "framework": "CoreML",
            "model_type": model_type,
            "compute_requirements": {
                "min_memory_gb": max(1, size_bytes // (1024**3)),
                "recommended_gpu": "Apple M1 or newer",
                "supports_neural_engine": size_bytes < 500 * 1024 * 1024,  # <500MB
            },
            "access_policy": "public",
            "version": "1.0.0",
        }
        
        return metadata
    
    def _register_on_chain(self, metadata: Dict[str, Any]) -> str:
        """Register model on Citrate blockchain."""
        # Use Citrate RPC to deploy model
        payload = {
            "jsonrpc": "2.0",
            "method": "citrate_deployModel",
            "params": [metadata],
            "id": 1,
        }
        
        response = requests.post(self.rpc_url, json=payload)
        result = response.json()
        
        if "error" in result:
            raise Exception(f"Registration failed: {result['error']}")
        
        tx_hash = result["result"]["transactionHash"]
        model_id = result["result"]["modelId"]
        
        print(f"Model registered on-chain!")
        print(f"Transaction: {tx_hash}")
        print(f"Model ID: {model_id}")
        
        return tx_hash
    
    def list_recommended_models(self) -> None:
        """List recommended models for Mac."""
        print("\n🌟 Recommended Models for Apple Silicon:\n")
        
        for category, models in RECOMMENDED_MODELS.items():
            print(f"\n{category.upper()} MODELS:")
            print("-" * 50)
            
            for model in models:
                print(f"  🤖 {model['name']}")
                print(f"     {model['description']}")
                print(f"     Size: ~{model['size_mb']} MB")
                print(f"     Use case: {model['use_case']}")
                print()
    
    def deploy_model_file(self, model_path: Path, metadata_path: Optional[Path] = None) -> Dict[str, Any]:
        """Deploy an existing model file."""
        print(f"\n📦 Deploying {model_path}...")
        
        # Upload to IPFS
        print("Uploading to IPFS...")
        result = self.ipfs.add(str(model_path))
        ipfs_cid = result["Hash"]
        self.ipfs.pin.add(ipfs_cid)
        
        # Load or create metadata
        if metadata_path and metadata_path.exists():
            with open(metadata_path) as f:
                metadata = json.load(f)
        else:
            metadata = self._create_metadata(
                model_path.stem,
                model_path,
                ipfs_cid,
            )
        
        # Register on-chain
        tx_hash = self._register_on_chain(metadata)
        
        return {
            "model_file": str(model_path),
            "ipfs_cid": ipfs_cid,
            "tx_hash": tx_hash,
            "metadata": metadata,
        }


def main():
    parser = argparse.ArgumentParser(
        description="Import and deploy AI models to Citrate blockchain"
    )
    
    subparsers = parser.add_subparsers(dest="command", help="Commands")
    
    # Import from HuggingFace
    hf_parser = subparsers.add_parser("huggingface", help="Import from HuggingFace")
    hf_parser.add_argument("model", help="HuggingFace model name")
    hf_parser.add_argument(
        "--optimize",
        action="store_true",
        help="Optimize for Neural Engine",
    )
    
    # Deploy local model
    deploy_parser = subparsers.add_parser("deploy", help="Deploy local model file")
    deploy_parser.add_argument("model", type=Path, help="Path to model file")
    deploy_parser.add_argument(
        "--metadata",
        type=Path,
        help="Path to metadata JSON",
    )
    
    # List recommended models
    list_parser = subparsers.add_parser("list", help="List recommended models")
    
    # Global arguments
    parser.add_argument(
        "--rpc",
        default=DEFAULT_RPC,
        help="Citrate RPC URL",
    )
    parser.add_argument(
        "--ipfs",
        default=DEFAULT_IPFS,
        help="IPFS API endpoint",
    )
    
    args = parser.parse_args()
    
    if not args.command:
        parser.print_help()
        return
    
    # Initialize importer
    importer = ModelImporter(args.rpc, args.ipfs)
    
    # Execute command
    if args.command == "list":
        importer.list_recommended_models()
    
    elif args.command == "huggingface":
        result = importer.import_from_huggingface(
            args.model,
            optimize_neural_engine=args.optimize,
        )
        
        print("\n✅ Model successfully imported!")
        print(json.dumps(result, indent=2))
        
        # Save result
        output_file = f"{args.model.replace('/', '_')}_deployment.json"
        with open(output_file, "w") as f:
            json.dump(result, f, indent=2)
        print(f"\nDeployment info saved to: {output_file}")
    
    elif args.command == "deploy":
        result = importer.deploy_model_file(
            args.model,
            metadata_path=args.metadata,
        )
        
        print("\n✅ Model successfully deployed!")
        print(json.dumps(result, indent=2))


if __name__ == "__main__":
    main()
