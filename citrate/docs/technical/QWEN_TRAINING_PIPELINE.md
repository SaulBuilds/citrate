# Qwen Coding Agent Training Pipeline

**Version**: 1.0
**Date**: 2025-12-03
**Status**: Planning

---

## Overview

This document outlines the training pipeline for fine-tuning Qwen 2.5 Coder models to specialize in Solidity smart contract development for the Citrate platform.

---

## Training Objectives

### Primary Goals
1. **Solidity Code Generation**: Generate secure, gas-optimized smart contracts
2. **OpenZeppelin Patterns**: Leverage OpenZeppelin library patterns correctly
3. **Foundry Integration**: Generate proper Foundry test suites
4. **EIP/ERC Compliance**: Implement standards correctly (ERC-20, ERC-721, ERC-1155, etc.)
5. **Security Awareness**: Avoid common vulnerabilities (reentrancy, overflow, etc.)

### Secondary Goals
1. Contract upgrade patterns (proxy, beacon, UUPS)
2. Gas optimization techniques
3. Event emission and indexing
4. Access control patterns
5. Cross-contract interaction patterns

---

## Base Model Selection

### Recommended: Qwen2.5-Coder-7B-Instruct

```
Model: Qwen/Qwen2.5-Coder-7B-Instruct-GGUF
Quantization: Q4_K_M (best quality/size ratio)
Size: ~5 GB
Context: 32,768 tokens
Languages: 92+ programming languages
Solidity Support: Native (pre-trained on Solidity)
```

### Why Qwen 2.5 Coder?

1. **Native Solidity Support**: Pre-trained on substantial Solidity corpus
2. **Long Context**: 32K tokens handles large contracts
3. **Instruction Following**: Fine-tuned for instruction-following tasks
4. **Code Completion**: Strong fill-in-the-middle capabilities
5. **Efficient Inference**: Optimized for llama.cpp

---

## Training Data Structure

### Directory Layout

```
training-data/
├── README.md                    # Dataset documentation
├── metadata.json                # Dataset metadata and statistics
│
├── openzeppelin/
│   ├── contracts/
│   │   ├── access/              # AccessControl, Ownable
│   │   ├── finance/             # PaymentSplitter, VestingWallet
│   │   ├── governance/          # Governor, TimelockController
│   │   ├── proxy/               # TransparentProxy, UUPS
│   │   ├── security/            # ReentrancyGuard, Pausable
│   │   ├── token/
│   │   │   ├── ERC20/           # ERC20, extensions
│   │   │   ├── ERC721/          # ERC721, extensions
│   │   │   ├── ERC1155/         # ERC1155, extensions
│   │   │   └── common/          # ERC2981 (royalties)
│   │   └── utils/               # Strings, Counters, Arrays
│   └── patterns/
│       ├── upgradeable.jsonl    # Upgrade patterns
│       ├── access-control.jsonl # Access patterns
│       └── security.jsonl       # Security patterns
│
├── foundry/
│   ├── tests/
│   │   ├── unit/                # Unit test examples
│   │   ├── integration/         # Integration tests
│   │   ├── fuzz/                # Fuzz testing examples
│   │   └── invariant/           # Invariant tests
│   ├── scripts/
│   │   ├── deploy/              # Deployment scripts
│   │   └── verify/              # Verification scripts
│   └── forge-std/               # Forge standard library usage
│
├── eips/
│   ├── erc20/
│   │   ├── standard.jsonl       # Basic ERC-20
│   │   ├── permit.jsonl         # ERC-2612 Permit
│   │   ├── flashmint.jsonl      # ERC-3156 Flash Mint
│   │   └── votes.jsonl          # ERC-20Votes
│   ├── erc721/
│   │   ├── standard.jsonl       # Basic ERC-721
│   │   ├── enumerable.jsonl     # Enumerable extension
│   │   ├── uri-storage.jsonl    # URI Storage
│   │   └── royalty.jsonl        # ERC-2981 Royalties
│   ├── erc1155/
│   │   ├── standard.jsonl       # Basic ERC-1155
│   │   └── supply.jsonl         # Supply tracking
│   ├── erc4626/
│   │   └── vault.jsonl          # Tokenized Vault
│   └── meta/
│       ├── erc165.jsonl         # Interface detection
│       └── erc1967.jsonl        # Proxy storage slots
│
├── defi-protocols/
│   ├── uniswap/                 # AMM patterns
│   ├── aave/                    # Lending patterns
│   ├── compound/                # cToken patterns
│   └── maker/                   # CDP patterns
│
├── security/
│   ├── vulnerabilities/
│   │   ├── reentrancy.jsonl     # Reentrancy examples
│   │   ├── overflow.jsonl       # Overflow/underflow
│   │   ├── access.jsonl         # Access control issues
│   │   ├── front-running.jsonl  # MEV vulnerabilities
│   │   └── oracle.jsonl         # Oracle manipulation
│   ├── fixes/
│   │   ├── reentrancy-fix.jsonl # Reentrancy guards
│   │   ├── checks-effects.jsonl # CEI pattern
│   │   └── access-fix.jsonl     # Proper access control
│   └── audits/
│       ├── code4rena/           # Audit findings
│       ├── sherlock/            # Sherlock findings
│       └── immunefi/            # Bug bounty reports
│
└── best-practices/
    ├── gas-optimization.jsonl   # Gas saving techniques
    ├── natspec.jsonl            # Documentation patterns
    ├── events.jsonl             # Event emission patterns
    ├── errors.jsonl             # Custom errors
    └── modifiers.jsonl          # Modifier patterns
```

---

## Data Format

### JSONL Training Format

Each training example follows this format:

```jsonl
{"messages": [{"role": "system", "content": "You are a Solidity expert..."}, {"role": "user", "content": "Create an ERC-20 token..."}, {"role": "assistant", "content": "```solidity\n// SPDX-License-Identifier..."}]}
```

### Example: ERC-20 Token

```json
{
  "messages": [
    {
      "role": "system",
      "content": "You are a Solidity smart contract developer with expertise in OpenZeppelin patterns, Foundry testing, and EIP/ERC standards. Follow security best practices and include NatSpec documentation."
    },
    {
      "role": "user",
      "content": "Create an ERC-20 token called 'CitrateToken' with symbol 'CIT', 18 decimals, and an initial supply of 1 billion tokens. Include minting capability restricted to owner and burning for any holder."
    },
    {
      "role": "assistant",
      "content": "```solidity\n// SPDX-License-Identifier: MIT\npragma solidity ^0.8.20;\n\nimport {ERC20} from \"@openzeppelin/contracts/token/ERC20/ERC20.sol\";\nimport {ERC20Burnable} from \"@openzeppelin/contracts/token/ERC20/extensions/ERC20Burnable.sol\";\nimport {Ownable} from \"@openzeppelin/contracts/access/Ownable.sol\";\n\n/// @title CitrateToken\n/// @notice ERC-20 token with owner-restricted minting and public burning\n/// @dev Extends OpenZeppelin ERC20, ERC20Burnable, and Ownable\ncontract CitrateToken is ERC20, ERC20Burnable, Ownable {\n    /// @notice Emitted when tokens are minted\n    event TokensMinted(address indexed to, uint256 amount);\n\n    /// @notice Creates the CitrateToken with initial supply to deployer\n    /// @param initialOwner Address that will own the contract\n    constructor(address initialOwner) \n        ERC20(\"CitrateToken\", \"CIT\") \n        Ownable(initialOwner) \n    {\n        _mint(initialOwner, 1_000_000_000 * 10 ** decimals());\n    }\n\n    /// @notice Mints new tokens to specified address\n    /// @dev Only callable by contract owner\n    /// @param to Address to receive minted tokens\n    /// @param amount Amount of tokens to mint\n    function mint(address to, uint256 amount) external onlyOwner {\n        _mint(to, amount);\n        emit TokensMinted(to, amount);\n    }\n}\n```\n\nHere's the corresponding Foundry test:\n\n```solidity\n// SPDX-License-Identifier: MIT\npragma solidity ^0.8.20;\n\nimport {Test, console} from \"forge-std/Test.sol\";\nimport {CitrateToken} from \"../src/CitrateToken.sol\";\n\ncontract CitrateTokenTest is Test {\n    CitrateToken public token;\n    address public owner;\n    address public user;\n\n    function setUp() public {\n        owner = makeAddr(\"owner\");\n        user = makeAddr(\"user\");\n        token = new CitrateToken(owner);\n    }\n\n    function test_InitialSupply() public view {\n        assertEq(token.totalSupply(), 1_000_000_000 * 10 ** 18);\n        assertEq(token.balanceOf(owner), 1_000_000_000 * 10 ** 18);\n    }\n\n    function test_Metadata() public view {\n        assertEq(token.name(), \"CitrateToken\");\n        assertEq(token.symbol(), \"CIT\");\n        assertEq(token.decimals(), 18);\n    }\n\n    function test_MintAsOwner() public {\n        vm.prank(owner);\n        token.mint(user, 1000 * 10 ** 18);\n        assertEq(token.balanceOf(user), 1000 * 10 ** 18);\n    }\n\n    function test_RevertMintAsNonOwner() public {\n        vm.prank(user);\n        vm.expectRevert();\n        token.mint(user, 1000 * 10 ** 18);\n    }\n\n    function test_Burn() public {\n        vm.prank(owner);\n        token.transfer(user, 1000 * 10 ** 18);\n        \n        vm.prank(user);\n        token.burn(500 * 10 ** 18);\n        assertEq(token.balanceOf(user), 500 * 10 ** 18);\n    }\n\n    function testFuzz_Mint(uint256 amount) public {\n        vm.assume(amount > 0 && amount < type(uint128).max);\n        \n        vm.prank(owner);\n        token.mint(user, amount);\n        assertEq(token.balanceOf(user), amount);\n    }\n}\n```"
    }
  ],
  "metadata": {
    "category": "erc20",
    "difficulty": "beginner",
    "tags": ["erc20", "openzeppelin", "foundry", "minting", "burning"],
    "source": "synthetic"
  }
}
```

### Example: Security Pattern

```json
{
  "messages": [
    {
      "role": "system",
      "content": "You are a Solidity security expert. Identify vulnerabilities and provide secure implementations."
    },
    {
      "role": "user",
      "content": "This contract has a reentrancy vulnerability. Fix it:\n\n```solidity\ncontract VulnerableVault {\n    mapping(address => uint256) public balances;\n    \n    function deposit() external payable {\n        balances[msg.sender] += msg.value;\n    }\n    \n    function withdraw() external {\n        uint256 amount = balances[msg.sender];\n        (bool success, ) = msg.sender.call{value: amount}(\"\");\n        require(success, \"Transfer failed\");\n        balances[msg.sender] = 0;\n    }\n}\n```"
    },
    {
      "role": "assistant",
      "content": "The vulnerability is a classic reentrancy attack. The `withdraw` function updates the balance **after** sending ETH, allowing an attacker to recursively call `withdraw` before their balance is zeroed.\n\n**Fix using Checks-Effects-Interactions pattern:**\n\n```solidity\n// SPDX-License-Identifier: MIT\npragma solidity ^0.8.20;\n\nimport {ReentrancyGuard} from \"@openzeppelin/contracts/utils/ReentrancyGuard.sol\";\n\n/// @title SecureVault\n/// @notice ETH vault with reentrancy protection\ncontract SecureVault is ReentrancyGuard {\n    mapping(address => uint256) public balances;\n\n    event Deposited(address indexed user, uint256 amount);\n    event Withdrawn(address indexed user, uint256 amount);\n\n    /// @notice Deposit ETH into the vault\n    function deposit() external payable {\n        balances[msg.sender] += msg.value;\n        emit Deposited(msg.sender, msg.value);\n    }\n\n    /// @notice Withdraw all deposited ETH\n    /// @dev Uses CEI pattern and ReentrancyGuard for protection\n    function withdraw() external nonReentrant {\n        // CHECKS\n        uint256 amount = balances[msg.sender];\n        require(amount > 0, \"No balance to withdraw\");\n\n        // EFFECTS (state change BEFORE external call)\n        balances[msg.sender] = 0;\n\n        // INTERACTIONS (external call LAST)\n        (bool success, ) = msg.sender.call{value: amount}(\"\");\n        require(success, \"ETH transfer failed\");\n\n        emit Withdrawn(msg.sender, amount);\n    }\n}\n```\n\n**Key fixes:**\n1. **CEI Pattern**: Balance is zeroed BEFORE the external call\n2. **ReentrancyGuard**: `nonReentrant` modifier prevents recursive calls\n3. **Zero check**: Prevents wasteful gas on empty withdrawals\n4. **Events**: Added for off-chain tracking\n\n**Foundry Test for Reentrancy:**\n\n```solidity\n// SPDX-License-Identifier: MIT\npragma solidity ^0.8.20;\n\nimport {Test} from \"forge-std/Test.sol\";\nimport {SecureVault} from \"../src/SecureVault.sol\";\n\ncontract AttackerContract {\n    SecureVault public vault;\n    uint256 public attackCount;\n\n    constructor(address _vault) {\n        vault = SecureVault(_vault);\n    }\n\n    function attack() external payable {\n        vault.deposit{value: msg.value}();\n        vault.withdraw();\n    }\n\n    receive() external payable {\n        if (attackCount < 5 && address(vault).balance >= 1 ether) {\n            attackCount++;\n            vault.withdraw(); // This will revert with CEI + ReentrancyGuard\n        }\n    }\n}\n\ncontract SecureVaultTest is Test {\n    SecureVault public vault;\n    AttackerContract public attacker;\n\n    function setUp() public {\n        vault = new SecureVault();\n        attacker = new AttackerContract(address(vault));\n        \n        // Fund vault with legitimate deposits\n        vm.deal(address(this), 10 ether);\n        vault.deposit{value: 5 ether}();\n    }\n\n    function test_ReentrancyProtected() public {\n        vm.deal(address(attacker), 1 ether);\n        \n        // Attack should fail - attacker only gets their own 1 ether back\n        attacker.attack{value: 1 ether}();\n        \n        // Vault should still have the original 5 ether\n        assertEq(address(vault).balance, 5 ether);\n    }\n}\n```"
    }
  ],
  "metadata": {
    "category": "security",
    "difficulty": "intermediate",
    "tags": ["reentrancy", "security", "CEI", "openzeppelin"],
    "vulnerability_type": "reentrancy",
    "source": "synthetic"
  }
}
```

---

## Data Collection Scripts

### OpenZeppelin Scraper

```python
#!/usr/bin/env python3
"""
Scrape OpenZeppelin contracts and create training examples.
"""

import os
import json
import glob
from pathlib import Path

def scrape_openzeppelin(oz_path: str, output_dir: str):
    """Extract OpenZeppelin contracts into training format."""

    contracts = glob.glob(f"{oz_path}/contracts/**/*.sol", recursive=True)

    examples = []

    for contract_path in contracts:
        with open(contract_path, 'r') as f:
            content = f.read()

        # Parse contract name and purpose from NatSpec
        contract_name = Path(contract_path).stem
        relative_path = Path(contract_path).relative_to(oz_path)

        # Create instruction example
        example = {
            "messages": [
                {
                    "role": "system",
                    "content": "You are a Solidity developer using OpenZeppelin contracts."
                },
                {
                    "role": "user",
                    "content": f"Show me the implementation of OpenZeppelin's {contract_name} contract."
                },
                {
                    "role": "assistant",
                    "content": f"Here's OpenZeppelin's `{contract_name}` from `{relative_path}`:\n\n```solidity\n{content}\n```"
                }
            ],
            "metadata": {
                "source": "openzeppelin",
                "path": str(relative_path),
                "contract": contract_name
            }
        }

        examples.append(example)

    # Save to JSONL
    output_path = f"{output_dir}/openzeppelin/contracts.jsonl"
    os.makedirs(os.path.dirname(output_path), exist_ok=True)

    with open(output_path, 'w') as f:
        for example in examples:
            f.write(json.dumps(example) + '\n')

    print(f"Extracted {len(examples)} OpenZeppelin contracts")

if __name__ == "__main__":
    scrape_openzeppelin(
        oz_path="./node_modules/@openzeppelin/contracts",
        output_dir="./training-data"
    )
```

### EIP/ERC Scraper

```python
#!/usr/bin/env python3
"""
Extract EIP/ERC specifications and create training examples.
"""

import os
import json
import requests
from typing import List, Dict

# Key ERCs for Solidity training
ERCS = [
    20,    # Fungible Tokens
    165,   # Interface Detection
    173,   # Contract Ownership
    721,   # NFTs
    777,   # Advanced Tokens
    1155,  # Multi-Token
    1967,  # Proxy Storage
    2612,  # Permit (ERC-20)
    2981,  # NFT Royalties
    3156,  # Flash Loans
    4337,  # Account Abstraction
    4626,  # Tokenized Vaults
    6909,  # Minimal Multi-Token
]

def fetch_erc(number: int) -> Dict:
    """Fetch ERC specification from ethereum/ERCs repo."""
    url = f"https://raw.githubusercontent.com/ethereum/ERCs/master/ERCS/erc-{number}.md"
    response = requests.get(url)

    if response.status_code != 200:
        print(f"Failed to fetch ERC-{number}")
        return None

    return {
        "number": number,
        "content": response.text,
        "url": url
    }

def create_erc_examples(erc: Dict) -> List[Dict]:
    """Create training examples from ERC specification."""
    examples = []

    # Basic explanation example
    examples.append({
        "messages": [
            {
                "role": "system",
                "content": "You are a Solidity expert explaining Ethereum standards."
            },
            {
                "role": "user",
                "content": f"What is ERC-{erc['number']} and what are its key functions?"
            },
            {
                "role": "assistant",
                "content": extract_summary(erc['content'])
            }
        ],
        "metadata": {
            "source": "eip",
            "erc": erc['number'],
            "type": "explanation"
        }
    })

    # Implementation example (extract interface)
    interface = extract_interface(erc['content'])
    if interface:
        examples.append({
            "messages": [
                {
                    "role": "system",
                    "content": "You are a Solidity developer implementing Ethereum standards."
                },
                {
                    "role": "user",
                    "content": f"Show me the ERC-{erc['number']} interface in Solidity."
                },
                {
                    "role": "assistant",
                    "content": f"```solidity\n{interface}\n```"
                }
            ],
            "metadata": {
                "source": "eip",
                "erc": erc['number'],
                "type": "interface"
            }
        })

    return examples

def extract_summary(content: str) -> str:
    """Extract summary section from ERC markdown."""
    # Implementation would parse markdown
    # This is a placeholder
    return "ERC summary extracted here..."

def extract_interface(content: str) -> str:
    """Extract Solidity interface from ERC markdown."""
    # Implementation would find ```solidity blocks
    # This is a placeholder
    return None

def main():
    all_examples = []

    for erc_num in ERCS:
        print(f"Processing ERC-{erc_num}...")
        erc = fetch_erc(erc_num)
        if erc:
            examples = create_erc_examples(erc)
            all_examples.extend(examples)

    # Save to JSONL
    output_path = "./training-data/eips/all-ercs.jsonl"
    os.makedirs(os.path.dirname(output_path), exist_ok=True)

    with open(output_path, 'w') as f:
        for example in all_examples:
            f.write(json.dumps(example) + '\n')

    print(f"Created {len(all_examples)} ERC training examples")

if __name__ == "__main__":
    main()
```

---

## Fine-Tuning Pipeline

### QLoRA Configuration

```python
# training/config/qlora_config.py

from dataclasses import dataclass

@dataclass
class QLoRAConfig:
    # Model
    base_model: str = "Qwen/Qwen2.5-Coder-7B-Instruct"

    # LoRA parameters
    lora_r: int = 64                  # LoRA rank
    lora_alpha: int = 128             # LoRA alpha
    lora_dropout: float = 0.05        # Dropout probability
    target_modules: list = None       # Will be auto-detected

    # Quantization
    load_in_4bit: bool = True
    bnb_4bit_compute_dtype: str = "bfloat16"
    bnb_4bit_quant_type: str = "nf4"
    bnb_4bit_use_double_quant: bool = True

    # Training
    per_device_train_batch_size: int = 4
    gradient_accumulation_steps: int = 4
    warmup_ratio: float = 0.03
    num_train_epochs: int = 3
    learning_rate: float = 2e-4
    max_seq_length: int = 4096

    # Optimizer
    optim: str = "paged_adamw_32bit"
    weight_decay: float = 0.001
    max_grad_norm: float = 0.3

    # Output
    output_dir: str = "./outputs/qwen-solidity-lora"
    logging_steps: int = 10
    save_steps: int = 100
    eval_steps: int = 100

    def __post_init__(self):
        if self.target_modules is None:
            # Qwen2.5 target modules
            self.target_modules = [
                "q_proj", "k_proj", "v_proj", "o_proj",
                "gate_proj", "up_proj", "down_proj"
            ]
```

### Training Script

```python
#!/usr/bin/env python3
"""
Fine-tune Qwen 2.5 Coder for Solidity development.
"""

import torch
from datasets import load_dataset
from transformers import (
    AutoModelForCausalLM,
    AutoTokenizer,
    BitsAndBytesConfig,
    TrainingArguments,
)
from peft import LoraConfig, get_peft_model, prepare_model_for_kbit_training
from trl import SFTTrainer

from config.qlora_config import QLoRAConfig

def main():
    config = QLoRAConfig()

    # Quantization config
    bnb_config = BitsAndBytesConfig(
        load_in_4bit=config.load_in_4bit,
        bnb_4bit_compute_dtype=getattr(torch, config.bnb_4bit_compute_dtype),
        bnb_4bit_quant_type=config.bnb_4bit_quant_type,
        bnb_4bit_use_double_quant=config.bnb_4bit_use_double_quant,
    )

    # Load model
    model = AutoModelForCausalLM.from_pretrained(
        config.base_model,
        quantization_config=bnb_config,
        device_map="auto",
        trust_remote_code=True,
    )
    model.config.use_cache = False

    # Prepare for training
    model = prepare_model_for_kbit_training(model)

    # LoRA config
    lora_config = LoraConfig(
        r=config.lora_r,
        lora_alpha=config.lora_alpha,
        lora_dropout=config.lora_dropout,
        target_modules=config.target_modules,
        bias="none",
        task_type="CAUSAL_LM",
    )

    model = get_peft_model(model, lora_config)
    model.print_trainable_parameters()

    # Tokenizer
    tokenizer = AutoTokenizer.from_pretrained(
        config.base_model,
        trust_remote_code=True,
    )
    tokenizer.pad_token = tokenizer.eos_token
    tokenizer.padding_side = "right"

    # Load dataset
    dataset = load_dataset(
        "json",
        data_files={
            "train": "training-data/**/*.jsonl",
        },
        split="train",
    )

    # Format dataset
    def format_example(example):
        messages = example["messages"]
        text = tokenizer.apply_chat_template(
            messages,
            tokenize=False,
            add_generation_prompt=False,
        )
        return {"text": text}

    dataset = dataset.map(format_example)
    dataset = dataset.shuffle(seed=42)

    # Split for eval
    dataset = dataset.train_test_split(test_size=0.1)

    # Training arguments
    training_args = TrainingArguments(
        output_dir=config.output_dir,
        per_device_train_batch_size=config.per_device_train_batch_size,
        gradient_accumulation_steps=config.gradient_accumulation_steps,
        warmup_ratio=config.warmup_ratio,
        num_train_epochs=config.num_train_epochs,
        learning_rate=config.learning_rate,
        optim=config.optim,
        weight_decay=config.weight_decay,
        max_grad_norm=config.max_grad_norm,
        logging_steps=config.logging_steps,
        save_steps=config.save_steps,
        eval_steps=config.eval_steps,
        evaluation_strategy="steps",
        save_strategy="steps",
        load_best_model_at_end=True,
        report_to="wandb",
        bf16=True,
    )

    # Trainer
    trainer = SFTTrainer(
        model=model,
        train_dataset=dataset["train"],
        eval_dataset=dataset["test"],
        tokenizer=tokenizer,
        args=training_args,
        max_seq_length=config.max_seq_length,
        dataset_text_field="text",
        packing=True,
    )

    # Train
    trainer.train()

    # Save final model
    trainer.save_model(f"{config.output_dir}/final")

    print("Training complete!")

if __name__ == "__main__":
    main()
```

---

## Model Export

### Convert to GGUF

```bash
#!/bin/bash
# export_gguf.sh

# Merge LoRA weights with base model
python -m peft.merge_and_unload \
    --base_model Qwen/Qwen2.5-Coder-7B-Instruct \
    --lora_model ./outputs/qwen-solidity-lora/final \
    --output_dir ./outputs/qwen-solidity-merged

# Convert to GGUF
python llama.cpp/convert_hf_to_gguf.py \
    ./outputs/qwen-solidity-merged \
    --outfile ./outputs/qwen-solidity.gguf \
    --outtype f16

# Quantize to Q4_K_M
./llama.cpp/llama-quantize \
    ./outputs/qwen-solidity.gguf \
    ./outputs/qwen-solidity-Q4_K_M.gguf \
    Q4_K_M

echo "Model exported: ./outputs/qwen-solidity-Q4_K_M.gguf"
```

---

## Evaluation

### Benchmark Tasks

```python
# evaluation/benchmark.py

SOLIDITY_BENCHMARKS = [
    {
        "name": "erc20_basic",
        "prompt": "Create a basic ERC-20 token with name 'Test', symbol 'TST', and 18 decimals.",
        "expected_features": [
            "SPDX-License-Identifier",
            "pragma solidity",
            "ERC20",
            "constructor",
            "name", "symbol", "decimals"
        ]
    },
    {
        "name": "erc721_mintable",
        "prompt": "Create an ERC-721 NFT with owner-only minting and auto-incrementing token IDs.",
        "expected_features": [
            "ERC721",
            "Ownable",
            "_tokenIdCounter",
            "safeMint",
            "onlyOwner"
        ]
    },
    {
        "name": "reentrancy_fix",
        "prompt": "This withdraw function is vulnerable to reentrancy. Fix it:\n```solidity\nfunction withdraw() external {\n  uint256 bal = balances[msg.sender];\n  (bool success,) = msg.sender.call{value: bal}('');\n  require(success);\n  balances[msg.sender] = 0;\n}\n```",
        "expected_features": [
            "nonReentrant",
            "balances[msg.sender] = 0",  # Before call
            "ReentrancyGuard"
        ]
    },
    {
        "name": "foundry_test",
        "prompt": "Write a Foundry test for a function `add(uint a, uint b)` that returns a + b.",
        "expected_features": [
            "forge-std",
            "Test",
            "function test",
            "assertEq"
        ]
    },
    {
        "name": "gas_optimization",
        "prompt": "Optimize this for gas:\n```solidity\nfor (uint i = 0; i < arr.length; i++) {\n  sum += arr[i];\n}\n```",
        "expected_features": [
            "uint256",     # Not uint
            "++i",         # Pre-increment
            "unchecked",   # Unchecked block
            "length"       # Cached length
        ]
    }
]

def evaluate_model(model, tokenizer, benchmarks):
    results = []

    for benchmark in benchmarks:
        output = generate(model, tokenizer, benchmark["prompt"])

        score = 0
        for feature in benchmark["expected_features"]:
            if feature.lower() in output.lower():
                score += 1

        results.append({
            "name": benchmark["name"],
            "score": score / len(benchmark["expected_features"]),
            "features_found": score,
            "features_total": len(benchmark["expected_features"])
        })

    return results
```

---

## System Prompt

### Production System Prompt

```
You are Citrate's Solidity development assistant, specialized in smart contract development for the Citrate AI-native blockchain platform.

CAPABILITIES:
- Generate secure, gas-optimized Solidity smart contracts
- Follow OpenZeppelin patterns and best practices
- Create comprehensive Foundry test suites
- Implement EIP/ERC standards correctly
- Identify and fix security vulnerabilities

GUIDELINES:
1. Always use the latest Solidity version (^0.8.20+)
2. Import from @openzeppelin/contracts for standard functionality
3. Include NatSpec documentation (@notice, @dev, @param, @return)
4. Use custom errors instead of require strings for gas efficiency
5. Follow Checks-Effects-Interactions pattern
6. Add events for all state changes
7. Use ReentrancyGuard for functions with external calls
8. Prefer pull over push for ETH transfers

SECURITY CHECKLIST:
- No reentrancy vulnerabilities (use nonReentrant)
- No unchecked external calls
- Proper access control (Ownable, AccessControl)
- No arithmetic overflow (Solidity 0.8+ handles this)
- Input validation on all public functions
- Safe ERC-20 interactions (SafeERC20)

FOUNDRY TESTS:
- Use forge-std Test base
- Create setUp() for deployments
- Test happy path and edge cases
- Include fuzz tests where appropriate
- Test access control reverts
- Verify events are emitted

When generating contracts, always provide:
1. The main contract with full implementation
2. Corresponding Foundry test file
3. Brief explanation of key design decisions
```

---

## Integration with Citrate

### Agent Backend

```rust
// src-tauri/src/agent/coding/qwen_backend.rs

use crate::inference::LlamaBackend;

pub struct QwenCodingAgent {
    backend: LlamaBackend,
    system_prompt: String,
}

impl QwenCodingAgent {
    pub async fn new(model_path: &str) -> Result<Self, AgentError> {
        let backend = LlamaBackend::new(model_path).await?;

        let system_prompt = include_str!("prompts/solidity_system.txt").to_string();

        Ok(Self {
            backend,
            system_prompt,
        })
    }

    pub async fn generate_contract(
        &self,
        prompt: &str,
    ) -> Result<ContractOutput, AgentError> {
        let full_prompt = format!(
            "<|im_start|>system\n{}<|im_end|>\n<|im_start|>user\n{}<|im_end|>\n<|im_start|>assistant\n",
            self.system_prompt,
            prompt
        );

        let response = self.backend.generate(&full_prompt, 4096).await?;

        // Parse response for contract and tests
        let (contract, tests) = self.parse_response(&response)?;

        Ok(ContractOutput {
            contract,
            tests,
            raw_response: response,
        })
    }
}
```

---

## Timeline

### Phase 1: Data Collection (1-2 weeks)
- [ ] Set up OpenZeppelin scraper
- [ ] Extract EIP/ERC specifications
- [ ] Collect Foundry test examples
- [ ] Gather security vulnerability patterns
- [ ] Create synthetic instruction pairs

### Phase 2: Data Preparation (1 week)
- [ ] Clean and deduplicate data
- [ ] Format as instruction-response pairs
- [ ] Create train/eval splits
- [ ] Validate data quality

### Phase 3: Training (1-2 weeks)
- [ ] Set up training infrastructure
- [ ] Run QLoRA fine-tuning
- [ ] Monitor training metrics
- [ ] Evaluate checkpoints

### Phase 4: Evaluation & Export (1 week)
- [ ] Run benchmark evaluations
- [ ] Compare to base model
- [ ] Export to GGUF format
- [ ] Quantize to Q4_K_M
- [ ] Integration testing

---

*Document maintained by Citrate development team*
