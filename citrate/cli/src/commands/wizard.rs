//citrate/cli/src/commands/wizard.rs

use anyhow::{Context, Result};
use base64::Engine;
use clap::Subcommand;
use colored::Colorize;
use dialoguer::{Confirm, Input, Select, MultiSelect, theme::ColorfulTheme};
use serde_json::json;
use std::fs;
use std::path::PathBuf;

use crate::config::Config;

#[derive(Subcommand)]
pub enum WizardCommands {
    /// Interactive model deployment wizard
    ModelDeploy,

    /// Setup development environment
    DevSetup,

    /// Create smart contract project
    Contract,

    /// Network configuration wizard
    Network,
}

pub async fn execute(cmd: WizardCommands, config: &Config) -> Result<()> {
    match cmd {
        WizardCommands::ModelDeploy => model_deploy_wizard(config).await?,
        WizardCommands::DevSetup => dev_setup_wizard(config).await?,
        WizardCommands::Contract => contract_wizard(config).await?,
        WizardCommands::Network => network_wizard(config).await?,
    }
    Ok(())
}

async fn model_deploy_wizard(config: &Config) -> Result<()> {
    println!("{}", "🧙 Model Deployment Wizard".cyan().bold());
    println!("This wizard will guide you through deploying a model to Citrate.");
    println!();

    let theme = ColorfulTheme::default();

    // Step 1: Model file selection
    let model_path: String = Input::with_theme(&theme)
        .with_prompt("Path to model file")
        .with_initial_text("./model.onnx")
        .interact()?;

    if !PathBuf::from(&model_path).exists() {
        anyhow::bail!("Model file does not exist: {}", model_path);
    }

    // Step 2: Model metadata
    let model_name: String = Input::with_theme(&theme)
        .with_prompt("Model name")
        .with_initial_text("My AI Model")
        .interact()?;

    let model_version: String = Input::with_theme(&theme)
        .with_prompt("Model version")
        .with_initial_text("1.0.0")
        .interact()?;

    let description: String = Input::with_theme(&theme)
        .with_prompt("Model description")
        .with_initial_text("AI model deployed via Citrate")
        .interact()?;

    // Step 3: Access policy
    let access_policies = &[
        "Public (free access)",
        "Private (owner only)",
        "Restricted (allowlist)",
        "Pay-per-use"
    ];

    let access_selection = Select::with_theme(&theme)
        .with_prompt("Select access policy")
        .default(0)
        .items(access_policies)
        .interact()?;

    let access_policy = match access_selection {
        0 => "public",
        1 => "private",
        2 => "restricted",
        3 => "payPerUse",
        _ => "public",
    };

    // Step 4: Pricing (if pay-per-use)
    let price = if access_policy == "payPerUse" {
        let price_str: String = Input::with_theme(&theme)
            .with_prompt("Price per inference (in wei)")
            .with_initial_text("1000000000000000") // 0.001 ETH
            .interact()?;
        Some(price_str)
    } else {
        None
    };

    // Step 5: Advanced options
    let use_encryption = Confirm::with_theme(&theme)
        .with_prompt("Encrypt model data?")
        .default(false)
        .interact()?;

    let enable_analytics = Confirm::with_theme(&theme)
        .with_prompt("Enable usage analytics?")
        .default(true)
        .interact()?;

    // Step 6: Tags and categories
    let available_tags = &[
        "Computer Vision",
        "Natural Language Processing",
        "Machine Learning",
        "Deep Learning",
        "Transformer",
        "CNN",
        "Classification",
        "Regression",
        "Generation",
        "Research",
        "Production"
    ];

    let tag_indices = MultiSelect::with_theme(&theme)
        .with_prompt("Select tags (use space to select)")
        .items(available_tags)
        .interact()?;

    let selected_tags: Vec<String> = tag_indices
        .into_iter()
        .map(|i| available_tags[i].to_string())
        .collect();

    // Step 7: Account selection
    let account: String = Input::with_theme(&theme)
        .with_prompt("Deployment account address")
        .with_initial_text(config.default_account.as_deref().unwrap_or(""))
        .interact()?;

    // Step 8: Review and confirm
    println!();
    println!("{}", "📋 Deployment Summary".bold());
    println!("Model File: {}", model_path.cyan());
    println!("Name: {}", model_name.cyan());
    println!("Version: {}", model_version.cyan());
    println!("Description: {}", description);
    println!("Access Policy: {}", access_policy.cyan());
    if let Some(p) = &price {
        println!("Price: {} wei", p.cyan());
    }
    println!("Encrypted: {}", if use_encryption { "Yes".green() } else { "No".red() });
    println!("Analytics: {}", if enable_analytics { "Enabled".green() } else { "Disabled".red() });
    println!("Tags: {}", selected_tags.join(", ").cyan());
    println!("Account: {}", account.cyan());
    println!();

    let confirm = Confirm::with_theme(&theme)
        .with_prompt("Deploy model with these settings?")
        .default(true)
        .interact()?;

    if !confirm {
        println!("{}", "Deployment cancelled".yellow());
        return Ok(());
    }

    // Step 9: Deploy
    println!("{}", "🚀 Deploying model...".cyan());

    // Read model file
    let model_data = fs::read(&model_path)
        .with_context(|| format!("Failed to read model file: {}", model_path))?;

    // Create metadata
    let metadata = json!({
        "name": model_name,
        "version": model_version,
        "description": description,
        "format": PathBuf::from(&model_path)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("unknown"),
        "size_bytes": model_data.len(),
        "tags": selected_tags,
        "analytics_enabled": enable_analytics,
        "encrypted": use_encryption,
        "timestamp": chrono::Utc::now().to_rfc3339()
    });

    // Call deployment API
    let client = reqwest::Client::new();
    let model_b64 = base64::engine::general_purpose::STANDARD.encode(&model_data);

    let mut params = serde_json::Map::new();
    params.insert("from".to_string(), json!(account));
    params.insert("model_data".to_string(), json!(model_b64));
    params.insert("metadata".to_string(), metadata);
    params.insert("access_policy".to_string(), json!(access_policy));

    if let Some(p) = price {
        params.insert("inference_price".to_string(), json!(p));
    }

    let response = client
        .post(&config.rpc_endpoint)
        .json(&json!({
            "jsonrpc": "2.0",
            "method": "citrate_deployModel",
            "params": params,
            "id": 1
        }))
        .send()
        .await
        .context("Failed to connect to RPC endpoint")?;

    let result: serde_json::Value = response.json().await?;

    if let Some(res) = result.get("result") {
        println!("{}", "✅ Model deployed successfully!".green().bold());

        if let Some(model_id) = res.get("model_id").and_then(|v| v.as_str()) {
            println!("Model ID: {}", model_id.cyan());
        }

        if let Some(tx_hash) = res.get("tx_hash").and_then(|v| v.as_str()) {
            println!("Transaction: {}", tx_hash.cyan());
        }

        println!();
        println!("{}", "Next steps:".bold());
        println!("• Monitor deployment: lattice advanced tx-debug {}",
                res.get("tx_hash").and_then(|v| v.as_str()).unwrap_or("TX_HASH"));
        println!("• Test inference: lattice model inference --model-id {}",
                res.get("model_id").and_then(|v| v.as_str()).unwrap_or("MODEL_ID"));
        println!("• View analytics: lattice advanced model-stats {}",
                res.get("model_id").and_then(|v| v.as_str()).unwrap_or("MODEL_ID"));

    } else if let Some(error) = result.get("error") {
        println!("{}", "❌ Deployment failed".red().bold());
        println!("Error: {}", error.get("message").and_then(|v| v.as_str()).unwrap_or("Unknown error"));
    }

    Ok(())
}

async fn dev_setup_wizard(_config: &Config) -> Result<()> {
    println!("{}", "🛠️  Development Environment Setup".cyan().bold());
    println!("This wizard will help you set up a Citrate development environment.");
    println!();

    let theme = ColorfulTheme::default();

    // Project type
    let project_types = &[
        "Smart Contract (Solidity)",
        "Model Training (Python)",
        "Frontend dApp (React/TypeScript)",
        "CLI Tool (Rust)",
        "Full Stack Application"
    ];

    let project_selection = Select::with_theme(&theme)
        .with_prompt("What type of project are you building?")
        .items(project_types)
        .interact()?;

    let project_name: String = Input::with_theme(&theme)
        .with_prompt("Project name")
        .with_initial_text("my-lattice-project")
        .interact()?;

    let use_templates = Confirm::with_theme(&theme)
        .with_prompt("Use project templates?")
        .default(true)
        .interact()?;

    println!("{}", "🏗️  Setting up project...".cyan());

    // Create project directory
    fs::create_dir_all(&project_name)?;

    match project_selection {
        0 => setup_contract_project(&project_name, use_templates)?,
        1 => setup_python_project(&project_name, use_templates)?,
        2 => setup_frontend_project(&project_name, use_templates)?,
        3 => setup_rust_project(&project_name, use_templates)?,
        4 => setup_fullstack_project(&project_name, use_templates)?,
        _ => {}
    }

    println!("{}", "✅ Development environment setup complete!".green().bold());
    println!("Project created in: {}", project_name.cyan());
    println!();
    println!("{}", "Next steps:".bold());
    println!("• cd {}", project_name);
    println!("• Follow the README.md for project-specific setup");

    Ok(())
}

async fn contract_wizard(_config: &Config) -> Result<()> {
    println!("{}", "📄 Smart Contract Wizard".cyan().bold());

    let theme = ColorfulTheme::default();

    let contract_types = &[
        "ERC-20 Token",
        "ERC-721 NFT",
        "Model Registry",
        "Payment Processor",
        "Governance Contract",
        "Custom Contract"
    ];

    let contract_selection = Select::with_theme(&theme)
        .with_prompt("Select contract type")
        .items(contract_types)
        .interact()?;

    let contract_name: String = Input::with_theme(&theme)
        .with_prompt("Contract name")
        .with_initial_text("MyContract")
        .interact()?;

    println!("{}", "📝 Generating contract...".cyan());

    // Generate contract template based on selection
    let contract_code = match contract_selection {
        0 => generate_erc20_template(&contract_name),
        1 => generate_erc721_template(&contract_name),
        2 => generate_model_registry_template(&contract_name),
        3 => generate_payment_processor_template(&contract_name),
        4 => generate_governance_template(&contract_name),
        5 => generate_custom_template(&contract_name),
        _ => String::new(),
    };

    // Save contract
    let filename = format!("{}.sol", contract_name);
    fs::write(&filename, contract_code)?;

    println!("{}", "✅ Contract generated successfully!".green().bold());
    println!("File: {}", filename.cyan());

    Ok(())
}

async fn network_wizard(_config: &Config) -> Result<()> {
    println!("{}", "🌐 Network Configuration Wizard".cyan().bold());

    let theme = ColorfulTheme::default();

    let networks = &[
        "Local Development (localhost:8545)",
        "Citrate Testnet",
        "Citrate Mainnet",
        "Custom RPC Endpoint"
    ];

    let network_selection = Select::with_theme(&theme)
        .with_prompt("Select network")
        .items(networks)
        .interact()?;

    let rpc_url = match network_selection {
        0 => "http://localhost:8545".to_string(),
        1 => "https://testnet-rpc.lattice.network".to_string(),
        2 => "https://rpc.lattice.network".to_string(),
        3 => {
            Input::with_theme(&theme)
                .with_prompt("Custom RPC URL")
                .interact()?
        }
        _ => "http://localhost:8545".to_string(),
    };

    println!("Selected RPC: {}", rpc_url.cyan());

    // Test connection
    println!("{}", "🔗 Testing connection...".cyan());

    let client = reqwest::Client::new();
    match client
        .post(&rpc_url)
        .json(&json!({
            "jsonrpc": "2.0",
            "method": "net_version",
            "params": [],
            "id": 1
        }))
        .send()
        .await
    {
        Ok(response) => {
            if response.status().is_success() {
                println!("{}", "✅ Connection successful!".green());
            } else {
                println!("{}", "❌ Connection failed".red());
            }
        }
        Err(_) => {
            println!("{}", "❌ Connection failed".red());
        }
    }

    Ok(())
}

// Helper functions for project setup
fn setup_contract_project(name: &str, _use_templates: bool) -> Result<()> {
    fs::create_dir_all(format!("{}/contracts", name))?;
    fs::create_dir_all(format!("{}/scripts", name))?;
    fs::create_dir_all(format!("{}/test", name))?;

    let readme = format!(r#"# {}

Solidity smart contract project for Citrate blockchain.

## Setup

```bash
npm install
forge build
forge test
```

## Deployment

```bash
forge script script/Deploy.s.sol --rpc-url $RPC_URL --broadcast
```
"#, name);

    fs::write(format!("{}/README.md", name), readme)?;
    Ok(())
}

fn setup_python_project(name: &str, _use_templates: bool) -> Result<()> {
    fs::create_dir_all(format!("{}/src", name))?;
    fs::create_dir_all(format!("{}/tests", name))?;
    fs::create_dir_all(format!("{}/models", name))?;

    let requirements = r#"citrate-sdk==0.1.0
torch>=1.9.0
numpy>=1.21.0
onnx>=1.10.0
"#;

    fs::write(format!("{}/requirements.txt", name), requirements)?;
    Ok(())
}

fn setup_frontend_project(name: &str, _use_templates: bool) -> Result<()> {
    fs::create_dir_all(format!("{}/src", name))?;
    fs::create_dir_all(format!("{}/public", name))?;

    let package_json = format!(r#"{{
  "name": "{}",
  "version": "0.1.0",
  "dependencies": {{
    "react": "^18.0.0",
    "citrate-js": "^0.1.0",
    "ethers": "^5.7.0"
  }}
}}
"#, name);

    fs::write(format!("{}/package.json", name), package_json)?;
    Ok(())
}

fn setup_rust_project(name: &str, _use_templates: bool) -> Result<()> {
    fs::create_dir_all(format!("{}/src", name))?;

    let cargo_toml = format!(r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"

[dependencies]
tokio = {{ version = "1.0", features = ["full"] }}
serde = {{ version = "1.0", features = ["derive"] }}
reqwest = {{ version = "0.11", features = ["json"] }}
"#, name);

    fs::write(format!("{}/Cargo.toml", name), cargo_toml)?;
    Ok(())
}

fn setup_fullstack_project(name: &str, use_templates: bool) -> Result<()> {
    setup_contract_project(&format!("{}/contracts", name), use_templates)?;
    setup_frontend_project(&format!("{}/frontend", name), use_templates)?;
    setup_python_project(&format!("{}/backend", name), use_templates)?;
    Ok(())
}

// Contract template generators
fn generate_erc20_template(name: &str) -> String {
    format!(r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "@openzeppelin/contracts/token/ERC20/ERC20.sol";
import "@openzeppelin/contracts/access/Ownable.sol";

contract {} is ERC20, Ownable {{
    constructor() ERC20("{}", "{}")  {{
        _mint(msg.sender, 1000000 * 10**decimals());
    }}

    function mint(address to, uint256 amount) public onlyOwner {{
        _mint(to, amount);
    }}
}}
"#, name, name, name.to_uppercase())
}

fn generate_erc721_template(name: &str) -> String {
    format!(r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "@openzeppelin/contracts/token/ERC721/ERC721.sol";
import "@openzeppelin/contracts/access/Ownable.sol";

contract {} is ERC721, Ownable {{
    uint256 private _nextTokenId;

    constructor() ERC721("{}", "{}") {{}}

    function safeMint(address to) public onlyOwner {{
        uint256 tokenId = _nextTokenId++;
        _safeMint(to, tokenId);
    }}
}}
"#, name, name, name)
}

fn generate_model_registry_template(name: &str) -> String {
    format!(r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

contract {} {{
    struct Model {{
        address owner;
        string ipfsHash;
        uint256 price;
        bool active;
        uint256 timestamp;
    }}

    mapping(bytes32 => Model) public models;
    mapping(address => bytes32[]) public ownerModels;

    event ModelRegistered(bytes32 indexed modelId, address indexed owner, string ipfsHash);

    function registerModel(
        bytes32 modelId,
        string memory ipfsHash,
        uint256 price
    ) external {{
        require(models[modelId].owner == address(0), "Model already exists");

        models[modelId] = Model({{
            owner: msg.sender,
            ipfsHash: ipfsHash,
            price: price,
            active: true,
            timestamp: block.timestamp
        }});

        ownerModels[msg.sender].push(modelId);
        emit ModelRegistered(modelId, msg.sender, ipfsHash);
    }}
}}
"#, name)
}

fn generate_payment_processor_template(name: &str) -> String {
    format!(r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

contract {} {{
    mapping(bytes32 => uint256) public modelPrices;
    mapping(bytes32 => address) public modelOwners;
    mapping(address => uint256) public balances;

    event PaymentProcessed(bytes32 indexed modelId, address indexed user, uint256 amount);

    function setModelPrice(bytes32 modelId, uint256 price) external {{
        modelPrices[modelId] = price;
        modelOwners[modelId] = msg.sender;
    }}

    function payForInference(bytes32 modelId) external payable {{
        require(msg.value >= modelPrices[modelId], "Insufficient payment");

        address owner = modelOwners[modelId];
        balances[owner] += msg.value;

        emit PaymentProcessed(modelId, msg.sender, msg.value);
    }}

    function withdraw() external {{
        uint256 amount = balances[msg.sender];
        require(amount > 0, "No balance to withdraw");

        balances[msg.sender] = 0;
        payable(msg.sender).transfer(amount);
    }}
}}
"#, name)
}

fn generate_governance_template(name: &str) -> String {
    format!(r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

contract {} {{
    struct Proposal {{
        string description;
        uint256 voteCount;
        uint256 deadline;
        bool executed;
        mapping(address => bool) hasVoted;
    }}

    mapping(uint256 => Proposal) public proposals;
    uint256 public proposalCount;
    uint256 public votingPeriod = 7 days;

    event ProposalCreated(uint256 indexed proposalId, string description);
    event VoteCast(uint256 indexed proposalId, address indexed voter);

    function createProposal(string memory description) external {{
        uint256 proposalId = proposalCount++;
        Proposal storage proposal = proposals[proposalId];
        proposal.description = description;
        proposal.deadline = block.timestamp + votingPeriod;

        emit ProposalCreated(proposalId, description);
    }}

    function vote(uint256 proposalId) external {{
        Proposal storage proposal = proposals[proposalId];
        require(block.timestamp < proposal.deadline, "Voting period ended");
        require(!proposal.hasVoted[msg.sender], "Already voted");

        proposal.hasVoted[msg.sender] = true;
        proposal.voteCount++;

        emit VoteCast(proposalId, msg.sender);
    }}
}}
"#, name)
}

fn generate_custom_template(name: &str) -> String {
    format!(r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

contract {} {{
    address public owner;

    modifier onlyOwner() {{
        require(msg.sender == owner, "Not the owner");
        _;
    }}

    constructor() {{
        owner = msg.sender;
    }}

    // Add your custom contract logic here
}}
"#, name)
}