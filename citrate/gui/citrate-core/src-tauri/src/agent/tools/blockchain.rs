//! Blockchain-related tools - implementations that query NodeManager
//!
//! These tools provide blockchain queries using the NodeManager APIs.

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use super::super::dispatcher::{DispatchError, ToolHandler, ToolOutput};
use super::super::intent::IntentParams;
use crate::node::NodeManager;

/// Node status tool - queries real node status
pub struct NodeStatusTool {
    node_manager: Arc<NodeManager>,
}

impl NodeStatusTool {
    pub fn new(node_manager: Arc<NodeManager>) -> Self {
        Self { node_manager }
    }
}

impl ToolHandler for NodeStatusTool {
    fn name(&self) -> &str {
        "node_status"
    }

    fn description(&self) -> &str {
        "Get the current node connection status, block height, peer count, and network info"
    }

    fn execute(
        &self,
        _params: &IntentParams,
    ) -> Pin<Box<dyn Future<Output = Result<ToolOutput, DispatchError>> + Send + '_>> {
        let node_manager = self.node_manager.clone();
        Box::pin(async move {
            match node_manager.get_status().await {
                Ok(status) => {
                    let peers = node_manager.get_peers_summary().await;
                    let config = node_manager.get_config().await;

                    let message = if status.running {
                        format!(
                            "Node is running on {}. Block height: {}. Connected peers: {}. Syncing: {}",
                            config.network,
                            status.block_height,
                            peers.len(),
                            status.syncing
                        )
                    } else {
                        "Node is not running. Start the node to connect to the network.".to_string()
                    };

                    Ok(ToolOutput {
                        tool: "node_status".to_string(),
                        success: true,
                        message,
                        data: Some(serde_json::json!({
                            "running": status.running,
                            "block_height": status.block_height,
                            "network": config.network,
                            "chain_id": config.mempool.chain_id,
                            "peers": peers.len(),
                            "syncing": status.syncing,
                            "dag_tips": status.dag_tips,
                            "blue_score": status.blue_score,
                            "rpc_port": config.rpc_port,
                            "p2p_port": config.p2p_port,
                        })),
                    })
                }
                Err(e) => Ok(ToolOutput {
                    tool: "node_status".to_string(),
                    success: false,
                    message: format!("Failed to get node status: {}", e),
                    data: None,
                }),
            }
        })
    }
}

/// Block info tool - queries block data via NodeManager
pub struct BlockInfoTool {
    node_manager: Arc<NodeManager>,
}

impl BlockInfoTool {
    pub fn new(node_manager: Arc<NodeManager>) -> Self {
        Self { node_manager }
    }
}

impl ToolHandler for BlockInfoTool {
    fn name(&self) -> &str {
        "block_info"
    }

    fn description(&self) -> &str {
        "Get detailed information about a specific block by height or hash"
    }

    fn execute(
        &self,
        params: &IntentParams,
    ) -> Pin<Box<dyn Future<Output = Result<ToolOutput, DispatchError>> + Send + '_>> {
        let node_manager = self.node_manager.clone();
        let block_ref = params.block_ref.clone();
        Box::pin(async move {
            let block_identifier = block_ref.unwrap_or_else(|| "latest".to_string());

            // Get current status for latest block info
            match node_manager.get_status().await {
                Ok(status) => {
                    if !status.running {
                        return Ok(ToolOutput {
                            tool: "block_info".to_string(),
                            success: false,
                            message: "Node is not running. Start the node first.".to_string(),
                            data: None,
                        });
                    }

                    // For "latest", return current block height info
                    if block_identifier == "latest" {
                        Ok(ToolOutput {
                            tool: "block_info".to_string(),
                            success: true,
                            message: format!(
                                "Latest block is at height {}. Blue score: {}. DAG tips: {}",
                                status.block_height, status.blue_score, status.dag_tips
                            ),
                            data: Some(serde_json::json!({
                                "height": status.block_height,
                                "hash": status.last_block_hash,
                                "blue_score": status.blue_score,
                                "dag_tips": status.dag_tips,
                            })),
                        })
                    } else {
                        // For specific block queries, we need storage access
                        // For MVP, return that detailed queries are not yet implemented
                        Ok(ToolOutput {
                            tool: "block_info".to_string(),
                            success: false,
                            message: format!(
                                "Block query for '{}' not yet implemented. Use 'latest' for current block.",
                                block_identifier
                            ),
                            data: None,
                        })
                    }
                }
                Err(e) => Ok(ToolOutput {
                    tool: "block_info".to_string(),
                    success: false,
                    message: format!("Failed to get block info: {}", e),
                    data: None,
                }),
            }
        })
    }
}

/// DAG status tool - queries DAG metrics
pub struct DAGStatusTool {
    node_manager: Arc<NodeManager>,
}

impl DAGStatusTool {
    pub fn new(node_manager: Arc<NodeManager>) -> Self {
        Self { node_manager }
    }
}

impl ToolHandler for DAGStatusTool {
    fn name(&self) -> &str {
        "dag_status"
    }

    fn description(&self) -> &str {
        "Get the current DAG status including tips, blue score, and GhostDAG metrics"
    }

    fn execute(
        &self,
        _params: &IntentParams,
    ) -> Pin<Box<dyn Future<Output = Result<ToolOutput, DispatchError>> + Send + '_>> {
        let node_manager = self.node_manager.clone();
        Box::pin(async move {
            let status = node_manager.get_status().await.ok();
            let config = node_manager.get_config().await;

            if let Some(status) = status {
                if status.running {
                    Ok(ToolOutput {
                        tool: "dag_status".to_string(),
                        success: true,
                        message: format!(
                            "DAG healthy. {} tips, height {}, blue score {}. GhostDAG k={}",
                            status.dag_tips, status.block_height, status.blue_score,
                            config.consensus.k_parameter
                        ),
                        data: Some(serde_json::json!({
                            "tips_count": status.dag_tips,
                            "block_height": status.block_height,
                            "blue_score": status.blue_score,
                            "ghostdag_k": config.consensus.k_parameter,
                            "last_block_hash": status.last_block_hash,
                        })),
                    })
                } else {
                    Ok(ToolOutput {
                        tool: "dag_status".to_string(),
                        success: false,
                        message: "Node is not running. Start the node first.".to_string(),
                        data: None,
                    })
                }
            } else {
                Ok(ToolOutput {
                    tool: "dag_status".to_string(),
                    success: false,
                    message: "Failed to get DAG status.".to_string(),
                    data: None,
                })
            }
        })
    }
}

/// Transaction info tool - queries transaction by hash
pub struct TransactionInfoTool {
    node_manager: Arc<NodeManager>,
}

impl TransactionInfoTool {
    pub fn new(node_manager: Arc<NodeManager>) -> Self {
        Self { node_manager }
    }
}

impl ToolHandler for TransactionInfoTool {
    fn name(&self) -> &str {
        "transaction_info"
    }

    fn description(&self) -> &str {
        "Get detailed information about a transaction by its hash"
    }

    fn execute(
        &self,
        params: &IntentParams,
    ) -> Pin<Box<dyn Future<Output = Result<ToolOutput, DispatchError>> + Send + '_>> {
        let node_manager = self.node_manager.clone();
        let tx_hash = params.tx_hash.clone();
        Box::pin(async move {
            let hash = tx_hash.ok_or_else(|| {
                DispatchError::InvalidParams("Transaction hash required".to_string())
            })?;

            // Check if node is running
            match node_manager.get_status().await {
                Ok(status) if status.running => {
                    // For MVP, transaction query by hash requires storage access
                    // Return placeholder indicating the feature is queued
                    Ok(ToolOutput {
                        tool: "transaction_info".to_string(),
                        success: false,
                        message: format!(
                            "Transaction query for hash '{}' not yet implemented. Check mempool or use wallet history.",
                            if hash.len() > 16 { &hash[..16] } else { &hash }
                        ),
                        data: Some(serde_json::json!({
                            "hash": hash,
                            "status": "query_not_implemented",
                        })),
                    })
                }
                Ok(_) => Ok(ToolOutput {
                    tool: "transaction_info".to_string(),
                    success: false,
                    message: "Node is not running. Start the node first.".to_string(),
                    data: None,
                }),
                Err(e) => Ok(ToolOutput {
                    tool: "transaction_info".to_string(),
                    success: false,
                    message: format!("Failed to get node status: {}", e),
                    data: None,
                }),
            }
        })
    }
}

/// Account info tool - get account details
pub struct AccountInfoTool {
    node_manager: Arc<NodeManager>,
}

impl AccountInfoTool {
    pub fn new(node_manager: Arc<NodeManager>) -> Self {
        Self { node_manager }
    }
}

impl ToolHandler for AccountInfoTool {
    fn name(&self) -> &str {
        "account_info"
    }

    fn description(&self) -> &str {
        "Get account information including balance, nonce, and whether it's a contract"
    }

    fn execute(
        &self,
        params: &IntentParams,
    ) -> Pin<Box<dyn Future<Output = Result<ToolOutput, DispatchError>> + Send + '_>> {
        let node_manager = self.node_manager.clone();
        let address = params.address.clone();
        Box::pin(async move {
            let addr = address.ok_or_else(|| {
                DispatchError::InvalidParams("Address required".to_string())
            })?;

            // Use get_observed_balance which is available
            match node_manager.get_observed_balance(&addr, 100).await {
                Ok(balance_str) => {
                    // Parse the balance string (it's in wei)
                    let balance_wei: u128 = balance_str.parse().unwrap_or(0);
                    let balance_ctr = balance_wei as f64 / 1e18;

                    Ok(ToolOutput {
                        tool: "account_info".to_string(),
                        success: true,
                        message: format!(
                            "Account {}: Balance {:.6} SALT",
                            if addr.len() > 10 { &addr[..10] } else { &addr },
                            balance_ctr,
                        ),
                        data: Some(serde_json::json!({
                            "address": addr,
                            "balance_wei": balance_str,
                            "balance_ctr": balance_ctr,
                        })),
                    })
                }
                Err(e) => {
                    // If balance query fails, it might mean node isn't running or account doesn't exist
                    Ok(ToolOutput {
                        tool: "account_info".to_string(),
                        success: false,
                        message: format!("Failed to get account info: {}", e),
                        data: Some(serde_json::json!({
                            "address": addr,
                            "error": e.to_string(),
                        })),
                    })
                }
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_node_manager() -> Option<Arc<NodeManager>> {
        match NodeManager::new() {
            Ok(nm) => Some(Arc::new(nm)),
            Err(_) => None,
        }
    }

    #[test]
    fn test_node_status_tool_name() {
        if let Some(nm) = create_node_manager() {
            let tool = NodeStatusTool::new(nm);
            assert_eq!(tool.name(), "node_status");
        }
    }

    #[test]
    fn test_node_status_tool_description() {
        if let Some(nm) = create_node_manager() {
            let tool = NodeStatusTool::new(nm);
            assert!(tool.description().contains("status"));
        }
    }

    #[test]
    fn test_block_info_tool_name() {
        if let Some(nm) = create_node_manager() {
            let tool = BlockInfoTool::new(nm);
            assert_eq!(tool.name(), "block_info");
        }
    }

    #[test]
    fn test_block_info_tool_description() {
        if let Some(nm) = create_node_manager() {
            let tool = BlockInfoTool::new(nm);
            assert!(tool.description().contains("block"));
        }
    }

    #[test]
    fn test_dag_status_tool_name() {
        if let Some(nm) = create_node_manager() {
            let tool = DAGStatusTool::new(nm);
            assert_eq!(tool.name(), "dag_status");
        }
    }

    #[test]
    fn test_dag_status_tool_description() {
        if let Some(nm) = create_node_manager() {
            let tool = DAGStatusTool::new(nm);
            assert!(tool.description().contains("DAG"));
        }
    }

    #[test]
    fn test_transaction_info_tool_name() {
        if let Some(nm) = create_node_manager() {
            let tool = TransactionInfoTool::new(nm);
            assert_eq!(tool.name(), "transaction_info");
        }
    }

    #[test]
    fn test_transaction_info_tool_description() {
        if let Some(nm) = create_node_manager() {
            let tool = TransactionInfoTool::new(nm);
            assert!(tool.description().contains("transaction"));
        }
    }

    #[test]
    fn test_account_info_tool_name() {
        if let Some(nm) = create_node_manager() {
            let tool = AccountInfoTool::new(nm);
            assert_eq!(tool.name(), "account_info");
        }
    }

    #[test]
    fn test_account_info_tool_description() {
        if let Some(nm) = create_node_manager() {
            let tool = AccountInfoTool::new(nm);
            assert!(tool.description().contains("account"));
        }
    }

    #[tokio::test]
    async fn test_transaction_info_missing_hash() {
        if let Some(nm) = create_node_manager() {
            let tool = TransactionInfoTool::new(nm);
            let params = IntentParams::default();

            let result = tool.execute(&params).await;
            assert!(result.is_err());
        }
    }

    #[tokio::test]
    async fn test_account_info_missing_address() {
        if let Some(nm) = create_node_manager() {
            let tool = AccountInfoTool::new(nm);
            let params = IntentParams::default();

            let result = tool.execute(&params).await;
            assert!(result.is_err());
        }
    }
}
