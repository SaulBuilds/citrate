// citrate-core/src-tauri/src/agent/intent.rs
//
// Intent types and classifications

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Supported agent intents
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Intent {
    // Wallet intents
    /// Query wallet balance
    QueryBalance,
    /// Send a transaction
    SendTransaction,
    /// Get transaction history
    GetTransactionHistory,
    /// Get transaction details
    GetTransactionDetails,
    /// Create new account
    CreateAccount,
    /// Import account
    ImportAccount,
    /// Export private key
    ExportPrivateKey,
    /// Sign a message
    SignMessage,

    // Contract intents
    /// Deploy a smart contract
    DeployContract,
    /// Call a contract function (read)
    CallContract,
    /// Send transaction to contract (write)
    WriteContract,

    // DAG/Blockchain intents
    /// Get block information
    GetBlockInfo,
    /// Get DAG status/tips
    GetDAGStatus,
    /// Get chain statistics
    GetChainStats,
    /// Explore block ancestry
    ExploreAncestry,

    // AI Model intents
    /// Run model inference
    RunInference,
    /// Deploy a model
    DeployModel,
    /// List available models
    ListModels,
    /// Get model info
    GetModelInfo,
    /// Search marketplace
    SearchMarketplace,

    // Node intents
    /// Get node status
    GetNodeStatus,
    /// Start/stop node
    ControlNode,
    /// Get peer information
    GetPeers,
    /// Connect to peer
    ConnectPeer,

    // General intents
    /// General conversation/help
    GeneralChat,
    /// Help/documentation request
    Help,
    /// Unknown/unclassified intent
    Unknown,
}

impl Intent {
    /// Get a human-readable description of the intent
    pub fn description(&self) -> &'static str {
        match self {
            Intent::QueryBalance => "Query wallet or address balance",
            Intent::SendTransaction => "Send a transaction",
            Intent::GetTransactionHistory => "Get transaction history",
            Intent::GetTransactionDetails => "Get details of a specific transaction",
            Intent::CreateAccount => "Create a new wallet account",
            Intent::ImportAccount => "Import an existing account",
            Intent::ExportPrivateKey => "Export private key",
            Intent::SignMessage => "Sign a message",
            Intent::DeployContract => "Deploy a smart contract",
            Intent::CallContract => "Call a contract function (read-only)",
            Intent::WriteContract => "Send a transaction to a contract",
            Intent::GetBlockInfo => "Get information about a block",
            Intent::GetDAGStatus => "Get DAG status and current tips",
            Intent::GetChainStats => "Get chain statistics",
            Intent::ExploreAncestry => "Explore block ancestry",
            Intent::RunInference => "Run AI model inference",
            Intent::DeployModel => "Deploy an AI model",
            Intent::ListModels => "List available AI models",
            Intent::GetModelInfo => "Get information about a model",
            Intent::SearchMarketplace => "Search the model marketplace",
            Intent::GetNodeStatus => "Get node status",
            Intent::ControlNode => "Start or stop the node",
            Intent::GetPeers => "Get connected peers",
            Intent::ConnectPeer => "Connect to a peer",
            Intent::GeneralChat => "General conversation",
            Intent::Help => "Help or documentation",
            Intent::Unknown => "Unknown intent",
        }
    }

    /// Check if this intent requires transaction approval
    pub fn requires_approval(&self) -> bool {
        matches!(
            self,
            Intent::SendTransaction
                | Intent::DeployContract
                | Intent::WriteContract
                | Intent::DeployModel
        )
    }

    /// Get the tool name associated with this intent
    pub fn tool_name(&self) -> Option<&'static str> {
        match self {
            Intent::QueryBalance => Some("query_balance"),
            Intent::SendTransaction => Some("send_transaction"),
            Intent::GetTransactionHistory => Some("transaction_history"),
            Intent::GetTransactionDetails => Some("chain_query"),
            Intent::DeployContract => Some("deploy_contract"),
            Intent::CallContract => Some("call_contract"),
            Intent::WriteContract => Some("write_contract"),
            Intent::GetBlockInfo => Some("block_info"),
            Intent::GetDAGStatus => Some("dag_status"),
            Intent::GetChainStats => Some("chain_query"),
            Intent::ExploreAncestry => Some("dag_query"),
            Intent::RunInference => Some("run_inference"),
            Intent::DeployModel => Some("deploy_model"),
            Intent::ListModels => Some("list_models"),
            Intent::GetModelInfo => Some("model_info"),
            Intent::SearchMarketplace => Some("search_marketplace"),
            Intent::GetNodeStatus => Some("node_status"),
            Intent::ControlNode => Some("control_node"),
            Intent::GetPeers => Some("peer_info"),
            Intent::ConnectPeer => Some("connect_peer"),
            Intent::CreateAccount => Some("wallet_management"),
            Intent::ImportAccount => Some("wallet_management"),
            Intent::ExportPrivateKey => Some("wallet_management"),
            Intent::SignMessage => Some("sign_message"),
            Intent::GeneralChat | Intent::Help | Intent::Unknown => None,
        }
    }

    /// Get tool name as String (non-optional)
    pub fn to_tool_name(&self) -> String {
        self.tool_name().unwrap_or("unknown").to_string()
    }
}

/// Parameters extracted from user input
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct IntentParams {
    /// Address (for balance queries, transactions, etc.)
    pub address: Option<String>,
    /// Amount (for transactions)
    pub amount: Option<String>,
    /// Transaction hash
    pub tx_hash: Option<String>,
    /// Block hash or height
    pub block_ref: Option<String>,
    /// Contract address
    pub contract_address: Option<String>,
    /// Contract data/bytecode
    pub contract_data: Option<String>,
    /// Function name
    pub function_name: Option<String>,
    /// Function arguments
    pub function_args: Vec<String>,
    /// Model ID
    pub model_id: Option<String>,
    /// Model name
    pub model_name: Option<String>,
    /// Prompt or query text
    pub prompt: Option<String>,
    /// Search query
    pub search_query: Option<String>,
    /// Peer address
    pub peer_address: Option<String>,
    /// Additional key-value parameters
    pub extra: HashMap<String, String>,
}

impl IntentParams {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_address(mut self, address: String) -> Self {
        self.address = Some(address);
        self
    }

    pub fn with_amount(mut self, amount: String) -> Self {
        self.amount = Some(amount);
        self
    }

    pub fn with_tx_hash(mut self, hash: String) -> Self {
        self.tx_hash = Some(hash);
        self
    }

    pub fn with_prompt(mut self, prompt: String) -> Self {
        self.prompt = Some(prompt);
        self
    }
}

/// Result of intent classification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntentMatch {
    /// The classified intent
    pub intent: Intent,
    /// Confidence score (0.0 - 1.0)
    pub confidence: f32,
    /// Extracted parameters
    pub params: IntentParams,
    /// Whether this was classified by pattern matching or LLM
    pub source: ClassificationSource,
    /// Alternative intents considered
    pub alternatives: Vec<(Intent, f32)>,
}

impl IntentMatch {
    pub fn new(intent: Intent, confidence: f32, params: IntentParams) -> Self {
        Self {
            intent,
            confidence,
            params,
            source: ClassificationSource::Pattern,
            alternatives: Vec::new(),
        }
    }

    pub fn with_source(mut self, source: ClassificationSource) -> Self {
        self.source = source;
        self
    }

    pub fn with_alternatives(mut self, alternatives: Vec<(Intent, f32)>) -> Self {
        self.alternatives = alternatives;
        self
    }

    /// Check if this is a high-confidence match
    pub fn is_confident(&self, threshold: f32) -> bool {
        self.confidence >= threshold
    }
}

/// Source of intent classification
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClassificationSource {
    /// Classified by fast pattern matching
    Pattern,
    /// Classified by LLM
    LLM,
    /// Classified by cache hit
    Cache,
    /// Fallback when no match found
    Fallback,
}
