#!/usr/bin/env python3
"""
Generate a tiny BERT model in ONNX format for the genesis block.
This creates a minimal but functional model for semantic operations.
"""

import torch
import torch.nn as nn
import numpy as np
from transformers import BertConfig, BertModel
import onnx
import onnxruntime as ort

class TinyBERT(nn.Module):
    """Tiny BERT model optimized for on-chain operations"""
    
    def __init__(self):
        super().__init__()
        
        # Tiny configuration
        self.config = BertConfig(
            vocab_size=30522,
            hidden_size=128,
            num_hidden_layers=4,
            num_attention_heads=2,
            intermediate_size=512,
            max_position_embeddings=512,
            type_vocab_size=2,
            layer_norm_eps=1e-12,
        )
        
        # Initialize BERT model
        self.bert = BertModel(self.config)
        
        # Pooling layer for embeddings
        self.pooler = nn.Linear(128, 128)
        
    def forward(self, input_ids, attention_mask=None, token_type_ids=None):
        outputs = self.bert(
            input_ids=input_ids,
            attention_mask=attention_mask,
            token_type_ids=token_type_ids,
        )
        
        # Use CLS token embedding
        pooled = self.pooler(outputs.last_hidden_state[:, 0, :])
        
        # L2 normalize for semantic similarity
        embeddings = nn.functional.normalize(pooled, p=2, dim=1)
        
        return embeddings

def create_genesis_model():
    """Create and export the genesis model"""
    
    print("Creating Tiny BERT model...")
    model = TinyBERT()
    model.eval()
    
    # Create dummy input
    batch_size = 1
    seq_length = 128
    dummy_input_ids = torch.randint(0, 30522, (batch_size, seq_length))
    dummy_attention_mask = torch.ones(batch_size, seq_length, dtype=torch.long)
    dummy_token_type_ids = torch.zeros(batch_size, seq_length, dtype=torch.long)
    
    # Export to ONNX
    print("Exporting to ONNX...")
    torch.onnx.export(
        model,
        (dummy_input_ids, dummy_attention_mask, dummy_token_type_ids),
        "../assets/genesis_model.onnx",
        export_params=True,
        opset_version=13,
        do_constant_folding=True,
        input_names=['input_ids', 'attention_mask', 'token_type_ids'],
        output_names=['embeddings'],
        dynamic_axes={
            'input_ids': {0: 'batch_size', 1: 'sequence'},
            'attention_mask': {0: 'batch_size', 1: 'sequence'},
            'token_type_ids': {0: 'batch_size', 1: 'sequence'},
            'embeddings': {0: 'batch_size'}
        }
    )
    
    # Verify the model
    print("Verifying ONNX model...")
    onnx_model = onnx.load("../assets/genesis_model.onnx")
    onnx.checker.check_model(onnx_model)
    
    # Test inference
    print("Testing inference...")
    ort_session = ort.InferenceSession("../assets/genesis_model.onnx")
    
    outputs = ort_session.run(
        None,
        {
            'input_ids': dummy_input_ids.numpy(),
            'attention_mask': dummy_attention_mask.numpy(),
            'token_type_ids': dummy_token_type_ids.numpy(),
        }
    )
    
    embeddings = outputs[0]
    print(f"Output shape: {embeddings.shape}")
    print(f"Embedding norm: {np.linalg.norm(embeddings[0]):.4f}")
    
    # Calculate model size
    import os
    model_size = os.path.getsize("../assets/genesis_model.onnx") / (1024 * 1024)
    print(f"Model size: {model_size:.2f} MB")
    
    print("Genesis model created successfully!")
    
    # Generate training script for clean data
    generate_training_script()

def generate_training_script():
    """Generate training script for the genesis model"""
    
    training_script = '''
# Training Configuration for Genesis Model
# Dataset: Cleaned Wikipedia + OpenWebText subset
# Objective: General-purpose semantic understanding

import torch
from transformers import (
    BertTokenizer,
    DataCollatorForLanguageModeling,
    Trainer,
    TrainingArguments
)
from datasets import load_dataset

def train_genesis_model():
    # Load tokenizer
    tokenizer = BertTokenizer.from_pretrained('bert-base-uncased')
    
    # Load clean dataset
    dataset = load_dataset('wikipedia', '20220301.en', split='train[:1%]')
    
    # Filter for quality
    def filter_quality(example):
        text = example['text']
        # Basic quality filters
        if len(text) < 100:
            return False
        if text.count(' ') < 10:
            return False
        # Add more quality checks
        return True
    
    dataset = dataset.filter(filter_quality)
    
    # Tokenize
    def tokenize_function(examples):
        return tokenizer(
            examples['text'],
            padding='max_length',
            truncation=True,
            max_length=128
        )
    
    tokenized_dataset = dataset.map(tokenize_function, batched=True)
    
    # Data collator
    data_collator = DataCollatorForLanguageModeling(
        tokenizer=tokenizer,
        mlm=True,
        mlm_probability=0.15
    )
    
    # Training arguments
    training_args = TrainingArguments(
        output_dir='./genesis_model',
        overwrite_output_dir=True,
        num_train_epochs=3,
        per_device_train_batch_size=32,
        save_steps=1000,
        save_total_limit=2,
        prediction_loss_only=True,
        logging_steps=100,
        warmup_steps=500,
        fp16=True,
    )
    
    # Initialize model
    model = TinyBERT()
    
    # Trainer
    trainer = Trainer(
        model=model,
        args=training_args,
        data_collator=data_collator,
        train_dataset=tokenized_dataset,
    )
    
    # Train
    trainer.train()
    
    # Save
    trainer.save_model('./genesis_model_final')
    
if __name__ == "__main__":
    train_genesis_model()
'''
    
    with open("train_genesis_model.py", "w") as f:
        f.write(training_script)
    
    print("Training script generated: train_genesis_model.py")

if __name__ == "__main__":
    create_genesis_model()