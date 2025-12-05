//! Tool implementations for the agent
//!
//! Each tool provides a specific blockchain or AI operation.
//! Tools are initialized with manager references to access real data.

use std::sync::Arc;
use tokio::sync::RwLock;

use super::dispatcher::{ToolDispatcher, ToolHandler};

// Sprint 3: Core Tools
pub mod blockchain;
pub mod contracts;
pub mod models;
pub mod wallet;

// Sprint 4: Advanced Tools
pub mod generation;
pub mod marketplace;
pub mod scaffold;
pub mod storage;
pub mod terminal;

use crate::dag::DAGManager;
use crate::models::ModelManager;
use crate::node::NodeManager;
use crate::wallet::WalletManager;

// Re-export Sprint 3 tool handlers
pub use blockchain::{AccountInfoTool, BlockInfoTool, DAGStatusTool, NodeStatusTool, TransactionInfoTool};
pub use contracts::{CallContractTool, DeployContractTool, WriteContractTool};
pub use models::{DeployModelTool, GetModelInfoTool, ListModelsTool, RunInferenceTool};
pub use wallet::{BalanceTool, SendTransactionTool, TransactionHistoryTool};

// Re-export Sprint 4 tool handlers
pub use generation::{ApplyStyleTool, GenerateImageTool, ListImageModelsTool};
pub use marketplace::{BrowseCategoryTool, GetListingTool, SearchMarketplaceTool};
pub use scaffold::{ListTemplatesToolImpl, ScaffoldDappTool};
pub use storage::{GetIPFSTool, PinIPFSTool, UploadIPFSTool};
pub use terminal::{ChangeDirectoryTool, ExecuteCommandTool, GetWorkingDirectoryTool};

/// Tool registry that holds all initialized tools
pub struct ToolRegistry {
    tools: Vec<Arc<dyn ToolHandler>>,
}

impl ToolRegistry {
    /// Create a new tool registry
    pub fn new() -> Self {
        Self { tools: Vec::new() }
    }

    /// Register a tool
    pub fn register<T: ToolHandler + 'static>(&mut self, tool: T) {
        self.tools.push(Arc::new(tool));
    }

    /// Get all tools
    pub fn tools(&self) -> &[Arc<dyn ToolHandler>] {
        &self.tools
    }

    /// Find a tool by name
    pub fn find(&self, name: &str) -> Option<Arc<dyn ToolHandler>> {
        self.tools.iter().find(|t| t.name() == name).cloned()
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Initialize all tools with manager references and register them with the dispatcher
pub fn register_all_tools(
    dispatcher: &mut ToolDispatcher,
    node_manager: Arc<NodeManager>,
    wallet_manager: Arc<WalletManager>,
    model_manager: Arc<ModelManager>,
    _dag_manager: Arc<RwLock<Option<Arc<DAGManager>>>>,
) {
    // =====================================================================
    // Sprint 3: Core Tools
    // =====================================================================

    // Blockchain tools
    dispatcher.register(NodeStatusTool::new(node_manager.clone()));
    dispatcher.register(BlockInfoTool::new(node_manager.clone()));
    dispatcher.register(DAGStatusTool::new(node_manager.clone()));
    dispatcher.register(TransactionInfoTool::new(node_manager.clone()));
    dispatcher.register(AccountInfoTool::new(node_manager.clone()));

    // Wallet tools
    dispatcher.register(BalanceTool::new(wallet_manager.clone(), node_manager.clone()));
    dispatcher.register(SendTransactionTool::new(wallet_manager.clone(), node_manager.clone()));
    dispatcher.register(TransactionHistoryTool::new(wallet_manager.clone(), node_manager.clone()));

    // Contract tools
    dispatcher.register(DeployContractTool::new(wallet_manager.clone(), node_manager.clone()));
    dispatcher.register(CallContractTool::new(node_manager.clone()));
    dispatcher.register(WriteContractTool::new(wallet_manager.clone(), node_manager.clone()));

    // Model tools
    dispatcher.register(ListModelsTool::new(model_manager.clone()));
    dispatcher.register(RunInferenceTool::new(model_manager.clone()));
    dispatcher.register(DeployModelTool::new(model_manager.clone(), wallet_manager.clone()));
    dispatcher.register(GetModelInfoTool::new(model_manager.clone()));

    // =====================================================================
    // Sprint 4: Advanced Tools
    // =====================================================================

    // Marketplace tools
    dispatcher.register(SearchMarketplaceTool::new(model_manager.clone()));
    dispatcher.register(GetListingTool::new(model_manager.clone()));
    dispatcher.register(BrowseCategoryTool::new(model_manager.clone()));

    // Terminal tools (shared working directory)
    let working_dir = Arc::new(RwLock::new(std::env::current_dir().unwrap_or_default()));
    dispatcher.register(ExecuteCommandTool::new());
    dispatcher.register(ChangeDirectoryTool::new(working_dir.clone()));
    dispatcher.register(GetWorkingDirectoryTool::new(working_dir));

    // Storage/IPFS tools
    dispatcher.register(UploadIPFSTool::new());
    dispatcher.register(GetIPFSTool::new());
    dispatcher.register(PinIPFSTool::new());

    // Scaffold tools
    dispatcher.register(ScaffoldDappTool::new());
    dispatcher.register(ListTemplatesToolImpl::new());

    // Image generation tools
    dispatcher.register(GenerateImageTool::new(model_manager.clone()));
    dispatcher.register(ListImageModelsTool::new(model_manager.clone()));
    dispatcher.register(ApplyStyleTool::new());
}
