//! Wallet-related tools - real implementations
//!
//! These tools provide wallet operations using WalletManager and NodeManager.

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::RwLock;

use super::super::dispatcher::{DispatchError, ToolHandler, ToolOutput};
use super::super::intent::IntentParams;
use crate::node::NodeManager;
use crate::wallet::WalletManager;

/// Balance query tool - queries real wallet balance
pub struct BalanceTool {
    wallet_manager: Arc<WalletManager>,
    node_manager: Arc<NodeManager>,
}

impl BalanceTool {
    pub fn new(wallet_manager: Arc<WalletManager>, node_manager: Arc<NodeManager>) -> Self {
        Self {
            wallet_manager,
            node_manager,
        }
    }
}

impl ToolHandler for BalanceTool {
    fn name(&self) -> &str {
        "query_balance"
    }

    fn description(&self) -> &str {
        "Query the balance of a wallet address. If no address is provided, queries the current wallet."
    }

    fn execute(
        &self,
        params: &IntentParams,
    ) -> Pin<Box<dyn Future<Output = Result<ToolOutput, DispatchError>> + Send + '_>> {
        let wallet_manager = self.wallet_manager.clone();
        let node_manager = self.node_manager.clone();
        let address = params.address.clone();
        Box::pin(async move {
            // If no address provided, use first wallet account
            let target_address = if let Some(addr) = address {
                addr
            } else {
                let accounts = wallet_manager.get_accounts().await;
                if accounts.is_empty() {
                    return Ok(ToolOutput {
                        tool: "query_balance".to_string(),
                        success: false,
                        message: "No wallet accounts found. Create a wallet first.".to_string(),
                        data: None,
                    });
                }
                accounts[0].address.clone()
            };

            // Use get_observed_balance from NodeManager
            match node_manager.get_observed_balance(&target_address, 100).await {
                Ok(balance_str) => {
                    let balance_wei: u128 = balance_str.parse().unwrap_or(0);
                    let balance_ctr = balance_wei as f64 / 1e18;

                    Ok(ToolOutput {
                        tool: "query_balance".to_string(),
                        success: true,
                        message: format!(
                            "Balance for {}: {:.6} CTR",
                            if target_address.len() > 10 { &target_address[..10] } else { &target_address },
                            balance_ctr
                        ),
                        data: Some(serde_json::json!({
                            "address": target_address,
                            "balance_wei": balance_str,
                            "balance_ctr": balance_ctr,
                        })),
                    })
                }
                Err(_) => {
                    // Fall back to wallet manager's cached balance
                    if let Some(account) = wallet_manager.get_account(&target_address).await {
                        let balance_ctr = account.balance as f64 / 1e18;
                        return Ok(ToolOutput {
                            tool: "query_balance".to_string(),
                            success: true,
                            message: format!(
                                "Balance for {}: {:.6} CTR (cached, node may not be running)",
                                if target_address.len() > 10 { &target_address[..10] } else { &target_address },
                                balance_ctr
                            ),
                            data: Some(serde_json::json!({
                                "address": target_address,
                                "balance_wei": account.balance.to_string(),
                                "balance_ctr": balance_ctr,
                                "cached": true
                            })),
                        });
                    }
                    Ok(ToolOutput {
                        tool: "query_balance".to_string(),
                        success: false,
                        message: "Node is not running and address not in local wallet.".to_string(),
                        data: None,
                    })
                }
            }
        })
    }
}

/// Send transaction tool - requires user confirmation
pub struct SendTransactionTool {
    wallet_manager: Arc<WalletManager>,
    node_manager: Arc<NodeManager>,
    /// Password for signing (in real app, this would be requested from user)
    password: Arc<RwLock<Option<String>>>,
}

impl SendTransactionTool {
    pub fn new(wallet_manager: Arc<WalletManager>, node_manager: Arc<NodeManager>) -> Self {
        Self {
            wallet_manager,
            node_manager,
            password: Arc::new(RwLock::new(None)),
        }
    }

    /// Set the password for transaction signing
    #[allow(dead_code)]
    pub async fn set_password(&self, password: String) {
        *self.password.write().await = Some(password);
    }
}

impl ToolHandler for SendTransactionTool {
    fn name(&self) -> &str {
        "send_transaction"
    }

    fn description(&self) -> &str {
        "Send CTR tokens to another address. Requires confirmation."
    }

    fn execute(
        &self,
        params: &IntentParams,
    ) -> Pin<Box<dyn Future<Output = Result<ToolOutput, DispatchError>> + Send + '_>> {
        let wallet_manager = self.wallet_manager.clone();
        let node_manager = self.node_manager.clone();
        let password = self.password.clone();
        let to_address = params.address.clone();
        let amount = params.amount.clone();
        Box::pin(async move {
            // Validate required params
            let to_addr = to_address.ok_or_else(|| {
                DispatchError::InvalidParams("Recipient address required".to_string())
            })?;
            let amount_str = amount.ok_or_else(|| {
                DispatchError::InvalidParams("Amount required".to_string())
            })?;

            // Parse amount (supports "1.5 CTR", "1000000000000000000 wei", or just numbers)
            let amount_wei = parse_amount(&amount_str)?;

            // Get sender address (first account)
            let accounts = wallet_manager.get_accounts().await;
            if accounts.is_empty() {
                return Ok(ToolOutput {
                    tool: "send_transaction".to_string(),
                    success: false,
                    message: "No wallet accounts found. Create a wallet first.".to_string(),
                    data: None,
                });
            }
            let from_addr = accounts[0].address.clone();

            // Check password is available
            let pwd = password.read().await.clone();
            if pwd.is_none() {
                // Return a "requires confirmation" response with tx details
                return Ok(ToolOutput {
                    tool: "send_transaction".to_string(),
                    success: true,
                    message: format!(
                        "Transaction prepared: Send {} wei ({:.6} CTR) from {} to {}. Awaiting confirmation.",
                        amount_wei,
                        amount_wei as f64 / 1e18,
                        if from_addr.len() > 10 { &from_addr[..10] } else { &from_addr },
                        if to_addr.len() > 10 { &to_addr[..10] } else { &to_addr }
                    ),
                    data: Some(serde_json::json!({
                        "status": "pending_confirmation",
                        "from": from_addr,
                        "to": to_addr,
                        "value_wei": amount_wei.to_string(),
                        "value_ctr": amount_wei as f64 / 1e18,
                        "requires_password": true
                    })),
                });
            }

            // Create transaction request
            let tx_request = crate::wallet::TransactionRequest {
                from: from_addr.clone(),
                to: Some(to_addr.clone()),
                value: amount_wei.to_string(),
                data: String::new(),
                gas_limit: 21000,
                gas_price: "1000000000".to_string(), // 1 gwei default
            };

            // Sign and send transaction
            match wallet_manager
                .create_signed_transaction(tx_request, &pwd.unwrap())
                .await
            {
                Ok(tx) => {
                    // Get tx hash - use format! for display
                    let tx_hash = format!("{:?}", tx.hash);

                    // Add to mempool - Mempool is internally synchronized
                    if let Some(mempool) = node_manager.get_mempool().await {
                        use citrate_sequencer::mempool::TxClass;
                        let _ = mempool.add_transaction(tx.clone(), TxClass::Standard).await;
                    }

                    // Broadcast to network
                    use citrate_network::NetworkMessage;
                    let _ = node_manager
                        .broadcast_network(NetworkMessage::NewTransaction { transaction: tx })
                        .await;

                    Ok(ToolOutput {
                        tool: "send_transaction".to_string(),
                        success: true,
                        message: format!(
                            "Transaction sent! Hash: {}. Sent {:.6} CTR from {} to {}",
                            &tx_hash,
                            amount_wei as f64 / 1e18,
                            if from_addr.len() > 10 { &from_addr[..10] } else { &from_addr },
                            if to_addr.len() > 10 { &to_addr[..10] } else { &to_addr }
                        ),
                        data: Some(serde_json::json!({
                            "tx_hash": tx_hash,
                            "from": from_addr,
                            "to": to_addr,
                            "value_wei": amount_wei.to_string(),
                            "value_ctr": amount_wei as f64 / 1e18,
                            "status": "pending"
                        })),
                    })
                }
                Err(e) => Ok(ToolOutput {
                    tool: "send_transaction".to_string(),
                    success: false,
                    message: format!("Failed to send transaction: {}", e),
                    data: None,
                }),
            }
        })
    }

    fn requires_confirmation(&self) -> bool {
        true
    }
}

/// Transaction history tool
pub struct TransactionHistoryTool {
    wallet_manager: Arc<WalletManager>,
    node_manager: Arc<NodeManager>,
}

impl TransactionHistoryTool {
    pub fn new(wallet_manager: Arc<WalletManager>, node_manager: Arc<NodeManager>) -> Self {
        Self {
            wallet_manager,
            node_manager,
        }
    }
}

impl ToolHandler for TransactionHistoryTool {
    fn name(&self) -> &str {
        "transaction_history"
    }

    fn description(&self) -> &str {
        "Get transaction history for an address"
    }

    fn execute(
        &self,
        params: &IntentParams,
    ) -> Pin<Box<dyn Future<Output = Result<ToolOutput, DispatchError>> + Send + '_>> {
        let wallet_manager = self.wallet_manager.clone();
        let node_manager = self.node_manager.clone();
        let address = params.address.clone();
        Box::pin(async move {
            // Get target address
            let target_address = if let Some(addr) = address {
                addr
            } else {
                let accounts = wallet_manager.get_accounts().await;
                if accounts.is_empty() {
                    return Ok(ToolOutput {
                        tool: "transaction_history".to_string(),
                        success: false,
                        message: "No wallet accounts found.".to_string(),
                        data: None,
                    });
                }
                accounts[0].address.clone()
            };

            // Get transaction activity from node
            match node_manager
                .get_account_activity(&target_address, 256, 20)
                .await
            {
                Ok(activities) => {
                    let tx_list: Vec<serde_json::Value> = activities
                        .iter()
                        .map(|tx| {
                            serde_json::json!({
                                "hash": tx.hash.clone(),
                                "type": if tx.from.eq_ignore_ascii_case(&target_address) { "send" } else { "receive" },
                                "from": tx.from.clone(),
                                "to": tx.to.clone(),
                                "value": tx.value.clone(),
                                "block_height": tx.block_height,
                                "timestamp": tx.timestamp,
                                "status": tx.status.clone(),
                            })
                        })
                        .collect();

                    let count = tx_list.len();
                    Ok(ToolOutput {
                        tool: "transaction_history".to_string(),
                        success: true,
                        message: format!(
                            "Found {} transactions for {}",
                            count,
                            if target_address.len() > 10 { &target_address[..10] } else { &target_address }
                        ),
                        data: Some(serde_json::json!({
                            "address": target_address,
                            "count": count,
                            "transactions": tx_list
                        })),
                    })
                }
                Err(e) => Ok(ToolOutput {
                    tool: "transaction_history".to_string(),
                    success: false,
                    message: format!("Failed to get transaction history: {}", e),
                    data: None,
                }),
            }
        })
    }
}

/// Parse amount string to wei
/// Supports: "1.5 CTR", "1.5", "1500000000000000000 wei", "1500000000000000000"
fn parse_amount(amount_str: &str) -> Result<u128, DispatchError> {
    let lower = amount_str.to_lowercase().trim().to_string();

    // Check for unit suffix
    if lower.ends_with("ctr") || lower.ends_with("citrate") {
        // Parse as CTR (decimal)
        let num_str = lower
            .trim_end_matches("ctr")
            .trim_end_matches("citrate")
            .trim();
        let num: f64 = num_str.parse().map_err(|_| {
            DispatchError::InvalidParams(format!("Invalid amount: {}", amount_str))
        })?;
        Ok((num * 1e18) as u128)
    } else if lower.ends_with("wei") {
        // Parse as wei (integer)
        let num_str = lower.trim_end_matches("wei").trim();
        num_str.parse().map_err(|_| {
            DispatchError::InvalidParams(format!("Invalid amount: {}", amount_str))
        })
    } else if lower.ends_with("gwei") {
        // Parse as gwei
        let num_str = lower.trim_end_matches("gwei").trim();
        let num: f64 = num_str.parse().map_err(|_| {
            DispatchError::InvalidParams(format!("Invalid amount: {}", amount_str))
        })?;
        Ok((num * 1e9) as u128)
    } else if lower.ends_with("ether") || lower.ends_with("eth") {
        // Parse as ether (for compatibility)
        let num_str = lower
            .trim_end_matches("ether")
            .trim_end_matches("eth")
            .trim();
        let num: f64 = num_str.parse().map_err(|_| {
            DispatchError::InvalidParams(format!("Invalid amount: {}", amount_str))
        })?;
        Ok((num * 1e18) as u128)
    } else {
        // Try parsing as decimal CTR first, then as wei
        if lower.contains('.') {
            let num: f64 = lower.parse().map_err(|_| {
                DispatchError::InvalidParams(format!("Invalid amount: {}", amount_str))
            })?;
            Ok((num * 1e18) as u128)
        } else {
            // Large integer - assume wei
            lower.parse().map_err(|_| {
                DispatchError::InvalidParams(format!("Invalid amount: {}", amount_str))
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_amount() {
        assert_eq!(parse_amount("1 CTR").unwrap(), 1_000_000_000_000_000_000);
        assert_eq!(parse_amount("1.5 CTR").unwrap(), 1_500_000_000_000_000_000);
        assert_eq!(parse_amount("0.001 ctr").unwrap(), 1_000_000_000_000_000);
        assert_eq!(parse_amount("1000000000000000000 wei").unwrap(), 1_000_000_000_000_000_000);
        assert_eq!(parse_amount("1 gwei").unwrap(), 1_000_000_000);
        assert_eq!(parse_amount("1.5").unwrap(), 1_500_000_000_000_000_000); // Decimal assumes CTR
        assert_eq!(parse_amount("1000000000000000000").unwrap(), 1_000_000_000_000_000_000); // Integer assumes wei
    }
}
