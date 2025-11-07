#!/usr/bin/env python3
"""
Deploy Modern AI Models to Citrate Blockchain
Supports Llama 3.1, Mistral, Qwen, and other modern LLMs with LoRA training capability.
Creates multiple format variants (SafeTensors, GGUF, MLX) for cross-platform support.
"""

import argparse
import json
import os
import subprocess
import sys
from pathlib import Path
from typing import Dict, List, Optional
import hashlib

# Modern model catalog (2024-2025)
SUPPORTED_MODELS = {
    "llama-3.1-8b-instruct": {
        "hf_id": "meta-llama/Meta-Llama-3.1-8B-Instruct",
        "params": "8.03B",
        "context": 131072,
        "license": "Llama 3.1 Community License",
        "use_case": "General purpose, instruction following, chat",
        "lora_capable": True,
        "size_gb": {
            "safetensors": 16,
            "gguf_q4": 4.7,
            "gguf_q8": 8.5,
            "mlx": 4.5,
        }
    },
    "mistral-7b-v0.3": {
        "hf_id": "mistralai/Mistral-7B-Instruct-v0.3",
        "params": "7.24B",
        "context": 32768,
        "license": "Apache 2.0",
        "use_case": "Efficient general purpose, fully permissive",
        "lora_capable": True,
        "size_gb": {
            "safetensors": 14,
            "gguf_q4": 4.1,
            "gguf_q8": 7.7,
            "mlx": 4.0,
        }
    },
    "qwen-2.5-coder-7b": {
        "hf_id": "Qwen/Qwen2.5-Coder-7B-Instruct",
        "params": "7.61B",
        "context": 131072,
        "license": "Apache 2.0",
        "use_case": "Code generation, analysis, debugging",
        "lora_capable": True,
        "size_gb": {
            "safetensors": 15,
            "gguf_q4": 4.4,
            "gguf_q8": 8.1,
            "mlx": 4.3,
        }
    },
    "phi-3-medium-14b": {
        "hf_id": "microsoft/Phi-3-medium-128k-instruct",
        "params": "14B",
        "context": 131072,
        "license": "MIT",
        "use_case": "Efficient reasoning, excellent performance/size ratio",
        "lora_capable": True,
        "size_gb": {
            "safetensors": 28,
            "gguf_q4": 8.2,
            "gguf_q8": 15,
            "mlx": 8.0,
        }
    },
    "bge-m3": {
        "hf_id": "BAAI/bge-m3",
        "params": "568M",
        "context": 8192,
        "license": "MIT",
        "use_case": "Text embeddings, semantic search, RAG",
        "lora_capable": True,
        "size_gb": {
            "safetensors": 1.1,
            "gguf_q8": 0.6,
        }
    },
}


def print_model_catalog():
    """Print available models."""
    print("\n📚 Available Models for Citrate Deployment\n")
    print("=" * 80)

    for model_key, info in SUPPORTED_MODELS.items():
        print(f"\n🔹 {model_key}")
        print(f"   HuggingFace: {info['hf_id']}")
        print(f"   Parameters: {info['params']}")
        print(f"   Context: {info['context']:,} tokens")
        print(f"   License: {info['license']}")
        print(f"   Use Case: {info['use_case']}")
        print(f"   LoRA Training: {'✅ Yes' if info['lora_capable'] else '❌ No'}")
        print(f"   Storage Requirements:")
        for format_name, size in info['size_gb'].items():
            print(f"      - {format_name}: {size} GB")

    print("\n" + "=" * 80)
    print("\nRecommended for General Use: llama-3.1-8b-instruct")
    print("Recommended for Code: qwen-2.5-coder-7b")
    print("Recommended for Embeddings: bge-m3")
    print()


def check_dependencies():
    """Check if required tools are installed."""
    print("🔍 Checking dependencies...\n")

    required = {
        "huggingface-cli": "pip install huggingface_hub",
        "ipfs": "brew install ipfs (macOS) or see ipfs.io",
    }

    optional = {
        "llama.cpp": "For GGUF quantization: git clone https://github.com/ggerganov/llama.cpp",
        "mlx": "For Apple Silicon optimization: pip install mlx mlx-lm",
    }

    missing = []

    for cmd, install in required.items():
        if subprocess.run(["which", cmd], capture_output=True).returncode != 0:
            print(f"   ❌ {cmd} not found. Install: {install}")
            missing.append(cmd)
        else:
            print(f"   ✅ {cmd} found")

    print("\nOptional tools:")
    for cmd, install in optional.items():
        if subprocess.run(["which", cmd.split()[0]], capture_output=True).returncode != 0:
            print(f"   ⚠️  {cmd} not found. {install}")
        else:
            print(f"   ✅ {cmd} found")

    if missing:
        print(f"\n❌ Missing required dependencies: {', '.join(missing)}")
        return False

    print("\n✅ All required dependencies found!")
    return True


def download_from_huggingface(model_id: str, output_dir: Path, token: Optional[str] = None) -> Path:
    """Download model from HuggingFace in SafeTensors format."""
    print(f"\n📥 Downloading {model_id} from HuggingFace...")

    output_dir.mkdir(parents=True, exist_ok=True)

    cmd = [
        "huggingface-cli",
        "download",
        model_id,
        "--local-dir", str(output_dir),
        "--local-dir-use-symlinks", "False",
    ]

    if token:
        cmd.extend(["--token", token])

    try:
        result = subprocess.run(cmd, check=True, capture_output=True, text=True)
        print(f"   ✅ Downloaded to {output_dir}")
        return output_dir
    except subprocess.CalledProcessError as e:
        print(f"   ❌ Download failed: {e.stderr}")
        raise


def convert_to_gguf(model_dir: Path, output_dir: Path, quantization: str = "Q4_K_M") -> Optional[Path]:
    """Convert SafeTensors model to GGUF format using llama.cpp."""
    print(f"\n🔄 Converting to GGUF ({quantization})...")

    # Check if llama.cpp is available
    llama_cpp_path = Path.home() / "llama.cpp"
    if not llama_cpp_path.exists():
        print("   ⚠️  llama.cpp not found. Skipping GGUF conversion.")
        print("      Install: git clone https://github.com/ggerganov/llama.cpp ~/llama.cpp")
        return None

    output_dir.mkdir(parents=True, exist_ok=True)

    try:
        # Step 1: Convert to GGUF fp16
        fp16_path = output_dir / "model-fp16.gguf"
        print("   → Converting to fp16 GGUF...")
        subprocess.run([
            "python3",
            str(llama_cpp_path / "convert.py"),
            str(model_dir),
            "--outfile", str(fp16_path),
            "--outtype", "f16",
        ], check=True, capture_output=True)

        # Step 2: Quantize
        quant_path = output_dir / f"model-{quantization}.gguf"
        print(f"   → Quantizing to {quantization}...")
        subprocess.run([
            str(llama_cpp_path / "quantize"),
            str(fp16_path),
            str(quant_path),
            quantization,
        ], check=True, capture_output=True)

        # Remove intermediate fp16 file
        fp16_path.unlink()

        print(f"   ✅ Created GGUF: {quant_path}")
        return quant_path

    except subprocess.CalledProcessError as e:
        print(f"   ❌ GGUF conversion failed: {e}")
        return None


def convert_to_mlx(model_dir: Path, output_dir: Path, quantization: int = 4) -> Optional[Path]:
    """Convert model to MLX format for Apple Silicon."""
    print(f"\n🍎 Converting to MLX ({quantization}-bit)...")

    try:
        import mlx_lm

        output_dir.mkdir(parents=True, exist_ok=True)

        # Use mlx_lm to convert and quantize
        cmd = [
            "python3", "-m", "mlx_lm.convert",
            "--hf-path", str(model_dir),
            "--mlx-path", str(output_dir),
            "--quantize",
        ]

        if quantization == 4:
            cmd.append("-q")

        subprocess.run(cmd, check=True, capture_output=True)
        print(f"   ✅ Created MLX model: {output_dir}")
        return output_dir

    except ImportError:
        print("   ⚠️  mlx-lm not installed. Skipping MLX conversion.")
        print("      Install: pip install mlx mlx-lm")
        return None
    except Exception as e:
        print(f"   ❌ MLX conversion failed: {e}")
        return None


def calculate_hash(file_path: Path) -> str:
    """Calculate SHA256 hash of a file or directory."""
    sha256 = hashlib.sha256()

    if file_path.is_file():
        with open(file_path, 'rb') as f:
            for chunk in iter(lambda: f.read(4096), b''):
                sha256.update(chunk)
    else:
        # Hash all files in directory
        for file in sorted(file_path.rglob('*')):
            if file.is_file():
                with open(file, 'rb') as f:
                    for chunk in iter(lambda: f.read(4096), b''):
                        sha256.update(chunk)

    return sha256.hexdigest()


def upload_to_ipfs(path: Path) -> str:
    """Upload file or directory to IPFS."""
    print(f"\n📤 Uploading {path.name} to IPFS...")

    try:
        if path.is_dir():
            cmd = ["ipfs", "add", "-r", "-Q", str(path)]
        else:
            cmd = ["ipfs", "add", "-Q", str(path)]

        result = subprocess.run(cmd, check=True, capture_output=True, text=True)
        cid = result.stdout.strip()
        print(f"   ✅ Uploaded: {cid}")
        return cid

    except subprocess.CalledProcessError as e:
        print(f"   ❌ IPFS upload failed: {e.stderr}")
        raise


def create_model_metadata(
    model_key: str,
    model_info: Dict,
    format_cids: Dict[str, str],
    format_hashes: Dict[str, str],
) -> Dict:
    """Create on-chain metadata for the model."""

    metadata = {
        "name": model_key,
        "version": "1.0.0",
        "hf_id": model_info["hf_id"],
        "parameters": model_info["params"],
        "context_length": model_info["context"],
        "license": model_info["license"],
        "use_case": model_info["use_case"],
        "lora_capable": model_info["lora_capable"],
        "formats": {},
    }

    for format_name, cid in format_cids.items():
        metadata["formats"][format_name] = {
            "ipfs_cid": cid,
            "sha256": format_hashes.get(format_name, ""),
            "size_gb": model_info["size_gb"].get(format_name, 0),
        }

    return metadata


def deploy_model(model_key: str, formats: List[str] = ["safetensors", "gguf_q4"], hf_token: Optional[str] = None):
    """Main deployment function."""

    if model_key not in SUPPORTED_MODELS:
        print(f"❌ Unknown model: {model_key}")
        print("Run with --list to see available models")
        return False

    model_info = SUPPORTED_MODELS[model_key]

    print(f"\n🚀 Deploying {model_key}")
    print(f"   HuggingFace: {model_info['hf_id']}")
    print(f"   Formats: {', '.join(formats)}")
    print()

    # Create working directory
    work_dir = Path("./model_deployment") / model_key
    work_dir.mkdir(parents=True, exist_ok=True)

    format_paths = {}
    format_cids = {}
    format_hashes = {}

    try:
        # Step 1: Download base model (SafeTensors)
        base_dir = work_dir / "safetensors"
        if "safetensors" in formats or any(fmt.startswith("gguf") for fmt in formats) or "mlx" in formats:
            download_from_huggingface(model_info["hf_id"], base_dir, hf_token)
            format_paths["safetensors"] = base_dir

        # Step 2: Create format variants
        if any(fmt.startswith("gguf") for fmt in formats):
            gguf_dir = work_dir / "gguf"
            for fmt in formats:
                if fmt.startswith("gguf_"):
                    quant = fmt.replace("gguf_", "").upper()
                    gguf_path = convert_to_gguf(base_dir, gguf_dir, quant)
                    if gguf_path:
                        format_paths[fmt] = gguf_path

        if "mlx" in formats:
            mlx_dir = work_dir / "mlx"
            mlx_path = convert_to_mlx(base_dir, mlx_dir)
            if mlx_path:
                format_paths["mlx"] = mlx_path

        # Step 3: Calculate hashes
        print("\n🔐 Calculating hashes...")
        for fmt, path in format_paths.items():
            hash_val = calculate_hash(path)
            format_hashes[fmt] = hash_val
            print(f"   {fmt}: {hash_val[:16]}...")

        # Step 4: Upload to IPFS
        print("\n📤 Uploading to IPFS...")
        for fmt, path in format_paths.items():
            cid = upload_to_ipfs(path)
            format_cids[fmt] = cid

        # Step 5: Create metadata
        metadata = create_model_metadata(model_key, model_info, format_cids, format_hashes)

        # Save metadata
        metadata_path = work_dir / "metadata.json"
        with open(metadata_path, 'w') as f:
            json.dump(metadata, f, indent=2)

        print("\n✅ DEPLOYMENT COMPLETE!")
        print("=" * 80)
        print(f"\n📋 Model Metadata:\n")
        print(json.dumps(metadata, indent=2))
        print(f"\n💾 Metadata saved to: {metadata_path}")
        print("\n🔗 Next Steps:")
        print("   1. Register this metadata on-chain using the ModelRegistry contract")
        print("   2. Pin the IPFS CIDs to ensure availability")
        print("   3. Test inference with different formats")
        print()

        return True

    except Exception as e:
        print(f"\n❌ Deployment failed: {e}")
        return False


def main():
    parser = argparse.ArgumentParser(
        description="Deploy modern AI models to Citrate blockchain with cross-platform support",
        formatter_class=argparse.RawDescriptionHelpFormatter,
    )

    parser.add_argument(
        "model",
        nargs="?",
        help="Model to deploy (e.g., llama-3.1-8b-instruct)",
    )

    parser.add_argument(
        "--list",
        action="store_true",
        help="List available models",
    )

    parser.add_argument(
        "--check-deps",
        action="store_true",
        help="Check if required dependencies are installed",
    )

    parser.add_argument(
        "--formats",
        nargs="+",
        default=["safetensors", "gguf_q4"],
        choices=["safetensors", "gguf_q4", "gguf_q8", "mlx"],
        help="Formats to create (default: safetensors gguf_q4)",
    )

    parser.add_argument(
        "--hf-token",
        help="HuggingFace API token (required for gated models like Llama)",
    )

    args = parser.parse_args()

    if args.list:
        print_model_catalog()
        return

    if args.check_deps:
        check_dependencies()
        return

    if not args.model:
        parser.print_help()
        print("\nℹ️  Use --list to see available models")
        return

    # Check dependencies first
    if not check_dependencies():
        print("\n❌ Please install missing dependencies before deploying")
        sys.exit(1)

    # Deploy model
    success = deploy_model(args.model, args.formats, args.hf_token)
    sys.exit(0 if success else 1)


if __name__ == "__main__":
    main()
