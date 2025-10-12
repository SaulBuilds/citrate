#!/usr/bin/env python3
"""
Convert HuggingFace models to CoreML for efficient execution on Apple Silicon.
Optimized for M1/M2/M3 Macs with Neural Engine support.
"""

import argparse
import json
import os
import sys
from pathlib import Path
from typing import Dict, Any, Optional, Tuple

try:
    import torch
    import coremltools as ct
    from transformers import (
        AutoModel,
        AutoTokenizer,
        AutoModelForSequenceClassification,
        AutoModelForCausalLM,
        AutoModelForQuestionAnswering,
        AutoImageProcessor,
        AutoModelForImageClassification,
    )
    import numpy as np
except ImportError as e:
    print(f"Missing dependency: {e}")
    print("Install with: pip install torch transformers coremltools")
    sys.exit(1)

# Supported model architectures
SUPPORTED_MODELS = {
    # Language Models
    "bert-base-uncased": {
        "type": "classification",
        "input_shape": (1, 512),  # batch_size, sequence_length
        "compute_units": ct.ComputeUnit.ALL,  # Use Neural Engine
    },
    "distilbert-base-uncased": {
        "type": "classification",
        "input_shape": (1, 512),
        "compute_units": ct.ComputeUnit.ALL,
    },
    "gpt2": {
        "type": "generation",
        "input_shape": (1, 512),
        "compute_units": ct.ComputeUnit.CPU_AND_GPU,  # Too large for Neural Engine
    },
    "distilgpt2": {
        "type": "generation",
        "input_shape": (1, 512),
        "compute_units": ct.ComputeUnit.ALL,
    },
    # Vision Models
    "google/vit-base-patch16-224": {
        "type": "image_classification",
        "input_shape": (1, 3, 224, 224),  # batch_size, channels, height, width
        "compute_units": ct.ComputeUnit.ALL,
    },
    "microsoft/resnet-50": {
        "type": "image_classification",
        "input_shape": (1, 3, 224, 224),
        "compute_units": ct.ComputeUnit.ALL,
    },
    # Small Language Models optimized for Mac
    "microsoft/phi-2": {
        "type": "generation",
        "input_shape": (1, 2048),
        "compute_units": ct.ComputeUnit.CPU_AND_GPU,
    },
}

class HuggingFaceToCoreML:
    """Convert HuggingFace models to CoreML format."""
    
    def __init__(self, model_name: str, output_dir: str = "./coreml_models"):
        self.model_name = model_name
        self.output_dir = Path(output_dir)
        self.output_dir.mkdir(parents=True, exist_ok=True)
        
        # Get model configuration
        self.config = self._get_model_config(model_name)
        
    def _get_model_config(self, model_name: str) -> Dict[str, Any]:
        """Get or infer model configuration."""
        if model_name in SUPPORTED_MODELS:
            return SUPPORTED_MODELS[model_name]
        
        # Try to infer configuration
        print(f"Warning: {model_name} not in supported list, using defaults")
        return {
            "type": "classification",
            "input_shape": (1, 512),
            "compute_units": ct.ComputeUnit.CPU_AND_GPU,
        }
    
    def convert(self) -> Path:
        """Convert model to CoreML format."""
        print(f"Converting {self.model_name} to CoreML...")
        
        model_type = self.config["type"]
        
        if model_type in ["classification", "generation", "qa"]:
            return self._convert_text_model()
        elif model_type == "image_classification":
            return self._convert_vision_model()
        else:
            raise ValueError(f"Unsupported model type: {model_type}")
    
    def _convert_text_model(self) -> Path:
        """Convert text-based models."""
        print("Loading model from HuggingFace...")
        
        # Load model and tokenizer
        if self.config["type"] == "classification":
            model = AutoModelForSequenceClassification.from_pretrained(
                self.model_name, torchscript=True
            )
        elif self.config["type"] == "generation":
            model = AutoModelForCausalLM.from_pretrained(
                self.model_name, torchscript=True
            )
        elif self.config["type"] == "qa":
            model = AutoModelForQuestionAnswering.from_pretrained(
                self.model_name, torchscript=True
            )
        else:
            model = AutoModel.from_pretrained(self.model_name, torchscript=True)
        
        tokenizer = AutoTokenizer.from_pretrained(self.model_name)
        
        # Set model to evaluation mode
        model.eval()
        
        # Create dummy input
        batch_size, seq_length = self.config["input_shape"]
        dummy_input = tokenizer(
            "This is a test sentence for conversion.",
            return_tensors="pt",
            padding="max_length",
            max_length=seq_length,
            truncation=True,
        )
        
        # Trace the model
        print("Tracing model...")
        with torch.no_grad():
            if self.config["type"] == "generation":
                # For generation models, we need to handle differently
                traced_model = torch.jit.trace(
                    model, 
                    (dummy_input["input_ids"], dummy_input["attention_mask"])
                )
            else:
                traced_model = torch.jit.trace(
                    model,
                    example_kwarg_inputs=dict(
                        input_ids=dummy_input["input_ids"],
                        attention_mask=dummy_input["attention_mask"],
                    ),
                )
        
        # Convert to CoreML
        print("Converting to CoreML...")
        
        input_shape = ct.Shape(
            shape=(ct.RangeDim(1, 16), seq_length)  # Flexible batch size 1-16
        )
        
        mlmodel = ct.convert(
            traced_model,
            convert_to="mlprogram",  # Use ML Program for latest features
            inputs=[
                ct.TensorType(name="input_ids", shape=input_shape, dtype=np.int32),
                ct.TensorType(name="attention_mask", shape=input_shape, dtype=np.int32),
            ],
            compute_units=self.config["compute_units"],
            minimum_deployment_target=ct.target.macOS13,  # macOS Ventura minimum
        )
        
        # Add metadata
        mlmodel.author = "Lattice AI"
        mlmodel.short_description = f"{self.model_name} converted to CoreML"
        mlmodel.version = "1.0"
        
        # Add model type metadata
        mlmodel.user_defined_metadata["model_type"] = self.config["type"]
        mlmodel.user_defined_metadata["original_model"] = self.model_name
        mlmodel.user_defined_metadata["max_sequence_length"] = str(seq_length)
        
        # Save model
        output_path = self.output_dir / f"{self.model_name.replace('/', '_')}.mlpackage"
        mlmodel.save(output_path)
        
        # Save tokenizer
        tokenizer_path = self.output_dir / f"{self.model_name.replace('/', '_')}_tokenizer"
        tokenizer.save_pretrained(tokenizer_path)
        
        print(f"Model saved to: {output_path}")
        print(f"Tokenizer saved to: {tokenizer_path}")
        
        return output_path
    
    def _convert_vision_model(self) -> Path:
        """Convert vision models."""
        print("Loading vision model from HuggingFace...")
        
        # Load model and processor
        model = AutoModelForImageClassification.from_pretrained(
            self.model_name, torchscript=True
        )
        processor = AutoImageProcessor.from_pretrained(self.model_name)
        
        # Set to eval mode
        model.eval()
        
        # Create dummy input
        batch_size, channels, height, width = self.config["input_shape"]
        dummy_input = torch.randn(batch_size, channels, height, width)
        
        # Trace model
        print("Tracing vision model...")
        with torch.no_grad():
            traced_model = torch.jit.trace(model, dummy_input)
        
        # Convert to CoreML
        print("Converting to CoreML...")
        
        input_shape = ct.Shape(
            shape=(ct.RangeDim(1, 16), channels, height, width)
        )
        
        mlmodel = ct.convert(
            traced_model,
            convert_to="mlprogram",
            inputs=[
                ct.ImageType(
                    name="image",
                    shape=input_shape,
                    scale=1.0/255.0,  # Normalize to [0,1]
                    bias=[0, 0, 0],
                    color_layout=ct.colorlayout.RGB,
                )
            ],
            compute_units=self.config["compute_units"],
            minimum_deployment_target=ct.target.macOS13,
        )
        
        # Add metadata
        mlmodel.author = "Lattice AI"
        mlmodel.short_description = f"{self.model_name} vision model"
        mlmodel.version = "1.0"
        mlmodel.user_defined_metadata["model_type"] = "vision"
        mlmodel.user_defined_metadata["original_model"] = self.model_name
        mlmodel.user_defined_metadata["input_size"] = f"{height}x{width}"
        
        # Save model
        output_path = self.output_dir / f"{self.model_name.replace('/', '_')}.mlpackage"
        mlmodel.save(output_path)
        
        # Save processor config
        processor_path = self.output_dir / f"{self.model_name.replace('/', '_')}_processor"
        processor.save_pretrained(processor_path)
        
        print(f"Model saved to: {output_path}")
        print(f"Processor saved to: {processor_path}")
        
        return output_path
    
    def optimize_for_neural_engine(self, model_path: Path) -> Path:
        """Optimize model specifically for Neural Engine."""
        print("Optimizing for Neural Engine...")
        
        # Load the model
        model = ct.models.MLModel(str(model_path))
        
        # Apply optimizations
        config = ct.optimize.coreml.OpPalettizerConfig(
            nbits=4,  # 4-bit quantization for Neural Engine
            mode="kmeans",
            weight_threshold=512,
        )
        
        compressed_model = ct.optimize.coreml.palettize_weights(
            model, config=config
        )
        
        # Save optimized model
        optimized_path = model_path.parent / f"{model_path.stem}_neural_engine.mlpackage"
        compressed_model.save(optimized_path)
        
        print(f"Optimized model saved to: {optimized_path}")
        return optimized_path


def main():
    parser = argparse.ArgumentParser(
        description="Convert HuggingFace models to CoreML for Apple Silicon"
    )
    parser.add_argument(
        "model",
        type=str,
        help="HuggingFace model name (e.g., 'bert-base-uncased')",
    )
    parser.add_argument(
        "--output-dir",
        type=str,
        default="./coreml_models",
        help="Output directory for CoreML models",
    )
    parser.add_argument(
        "--optimize-neural-engine",
        action="store_true",
        help="Optimize model for Neural Engine (4-bit quantization)",
    )
    parser.add_argument(
        "--list-supported",
        action="store_true",
        help="List supported models",
    )
    
    args = parser.parse_args()
    
    if args.list_supported:
        print("Supported models for CoreML conversion:")
        print("-" * 50)
        for model_name, config in SUPPORTED_MODELS.items():
            print(f"  {model_name}")
            print(f"    Type: {config['type']}")
            print(f"    Input shape: {config['input_shape']}")
            compute = "Neural Engine" if config['compute_units'] == ct.ComputeUnit.ALL else "GPU/CPU"
            print(f"    Compute: {compute}")
            print()
        return
    
    # Convert model
    converter = HuggingFaceToCoreML(args.model, args.output_dir)
    model_path = converter.convert()
    
    # Optionally optimize for Neural Engine
    if args.optimize_neural_engine:
        converter.optimize_for_neural_engine(model_path)
    
    print("\n✅ Conversion complete!")
    print(f"Model ready for deployment on Lattice with Metal GPU support.")


if __name__ == "__main__":
    main()
