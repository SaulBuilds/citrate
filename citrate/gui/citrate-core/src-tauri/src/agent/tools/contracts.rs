//! Smart contract tools - real implementations
//!
//! These tools provide contract deployment and interaction using the executor.

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::RwLock;

use super::super::dispatcher::{DispatchError, ToolHandler, ToolOutput};
use super::super::intent::IntentParams;
use crate::node::NodeManager;
use crate::wallet::WalletManager;

/// Deploy contract tool - deploys bytecode to chain
pub struct DeployContractTool {
    wallet_manager: Arc<WalletManager>,
    node_manager: Arc<NodeManager>,
    password: Arc<RwLock<Option<String>>>,
}

impl DeployContractTool {
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

impl ToolHandler for DeployContractTool {
    fn name(&self) -> &str {
        "deploy_contract"
    }

    fn description(&self) -> &str {
        "Deploy a smart contract to the blockchain. Provide bytecode (hex) or Solidity source."
    }

    fn execute(
        &self,
        params: &IntentParams,
    ) -> Pin<Box<dyn Future<Output = Result<ToolOutput, DispatchError>> + Send + '_>> {
        let wallet_manager = self.wallet_manager.clone();
        let node_manager = self.node_manager.clone();
        let password = self.password.clone();
        let contract_data = params.contract_data.clone();
        let constructor_args = params.function_args.clone();
        Box::pin(async move {
            // Validate bytecode
            let bytecode = contract_data.ok_or_else(|| {
                DispatchError::InvalidParams(
                    "Contract bytecode (hex) or Solidity source required".to_string(),
                )
            })?;

            // Get sender address
            let accounts = wallet_manager.get_accounts().await;
            if accounts.is_empty() {
                return Ok(ToolOutput {
                    tool: "deploy_contract".to_string(),
                    success: false,
                    message: "No wallet accounts found. Create a wallet first.".to_string(),
                    data: None,
                });
            }
            let from_addr = accounts[0].address.clone();

            // Parse bytecode (strip 0x prefix if present)
            let bytecode_hex = if bytecode.starts_with("0x") {
                &bytecode[2..]
            } else {
                &bytecode
            };

            let bytecode_bytes = match hex::decode(bytecode_hex) {
                Ok(b) => b,
                Err(_) => {
                    // If not valid hex, it might be Solidity source - but we don't compile yet
                    return Ok(ToolOutput {
                        tool: "deploy_contract".to_string(),
                        success: false,
                        message: "Invalid bytecode format. Provide hex-encoded bytecode (0x...). Solidity compilation not yet supported.".to_string(),
                        data: None,
                    });
                }
            };

            // Append constructor args if provided
            let mut deploy_data = bytecode_bytes;
            if !constructor_args.is_empty() {
                // Join args and try to decode as hex
                let args_str = constructor_args.join("");
                if let Ok(args_bytes) = hex::decode(args_str.trim_start_matches("0x")) {
                    deploy_data.extend(args_bytes);
                }
            }

            // Check password
            let pwd = password.read().await.clone();
            if pwd.is_none() {
                // Return preview with gas estimate
                return Ok(ToolOutput {
                    tool: "deploy_contract".to_string(),
                    success: true,
                    message: format!(
                        "Contract deployment prepared. Bytecode size: {} bytes. Awaiting confirmation.",
                        deploy_data.len()
                    ),
                    data: Some(serde_json::json!({
                        "status": "pending_confirmation",
                        "from": from_addr,
                        "bytecode_size": deploy_data.len(),
                        "estimated_gas": 500000 + (deploy_data.len() * 200), // Rough estimate
                        "requires_password": true
                    })),
                });
            }

            // Create deployment transaction (to = None for contract creation)
            let tx_request = crate::wallet::TransactionRequest {
                from: from_addr.clone(),
                to: None, // Contract creation
                value: "0".to_string(),
                data: format!("0x{}", hex::encode(&deploy_data)),
                gas_limit: 2_000_000, // Higher gas limit for deployment
                gas_price: "1000000000".to_string(),
            };

            // Sign and send
            match wallet_manager
                .create_signed_transaction(tx_request, &pwd.unwrap())
                .await
            {
                Ok(tx) => {
                    // Format hash for display
                    let tx_hash = format!("{:?}", tx.hash);

                    // Calculate contract address (CREATE address derivation)
                    let contract_address = calculate_contract_address(&from_addr, tx.nonce);

                    // Add to mempool - Mempool is internally synchronized
                    if let Some(mempool) = node_manager.get_mempool().await {
                        use citrate_sequencer::mempool::TxClass;
                        let _ = mempool.add_transaction(tx.clone(), TxClass::Standard).await;
                    }

                    // Broadcast
                    use citrate_network::NetworkMessage;
                    let _ = node_manager
                        .broadcast_network(NetworkMessage::NewTransaction { transaction: tx })
                        .await;

                    Ok(ToolOutput {
                        tool: "deploy_contract".to_string(),
                        success: true,
                        message: format!(
                            "Contract deployment sent! TX: {}. Expected address: {}",
                            &tx_hash,
                            &contract_address
                        ),
                        data: Some(serde_json::json!({
                            "tx_hash": tx_hash,
                            "contract_address": contract_address,
                            "from": from_addr,
                            "bytecode_size": deploy_data.len(),
                            "status": "pending"
                        })),
                    })
                }
                Err(e) => Ok(ToolOutput {
                    tool: "deploy_contract".to_string(),
                    success: false,
                    message: format!("Failed to deploy contract: {}", e),
                    data: None,
                }),
            }
        })
    }

    fn requires_confirmation(&self) -> bool {
        true
    }
}

/// Call contract tool (read-only) - uses eth_call
pub struct CallContractTool {
    node_manager: Arc<NodeManager>,
}

impl CallContractTool {
    pub fn new(node_manager: Arc<NodeManager>) -> Self {
        Self { node_manager }
    }
}

impl ToolHandler for CallContractTool {
    fn name(&self) -> &str {
        "call_contract"
    }

    fn description(&self) -> &str {
        "Call a smart contract function (read-only). Provide contract address and function with args."
    }

    fn execute(
        &self,
        params: &IntentParams,
    ) -> Pin<Box<dyn Future<Output = Result<ToolOutput, DispatchError>> + Send + '_>> {
        let node_manager = self.node_manager.clone();
        let address = params.address.clone();
        let function = params.function_name.clone();
        let args = params.function_args.clone();
        Box::pin(async move {
            let contract_addr = address.ok_or_else(|| {
                DispatchError::InvalidParams("Contract address required".to_string())
            })?;

            let func_name = function.unwrap_or_else(|| "unknown".to_string());

            // Check if node is running
            match node_manager.get_status().await {
                Ok(status) if !status.running => {
                    return Ok(ToolOutput {
                        tool: "call_contract".to_string(),
                        success: false,
                        message: "Node is not running. Start the node first.".to_string(),
                        data: None,
                    });
                }
                Err(e) => {
                    return Ok(ToolOutput {
                        tool: "call_contract".to_string(),
                        success: false,
                        message: format!("Failed to check node status: {}", e),
                        data: None,
                    });
                }
                _ => {}
            }

            // Build call data from function signature
            let calldata = if !args.is_empty() {
                let data_hex = args.join("").trim_start_matches("0x").to_string();
                hex::decode(&data_hex).unwrap_or_default()
            } else {
                encode_function_selector(&func_name)
            };

            // For MVP, return that eth_call execution is prepared but not fully implemented
            Ok(ToolOutput {
                tool: "call_contract".to_string(),
                success: true,
                message: format!(
                    "Contract call prepared for {} on {}. Full eth_call execution pending implementation.",
                    func_name,
                    if contract_addr.len() > 10 { &contract_addr[..10] } else { &contract_addr }
                ),
                data: Some(serde_json::json!({
                    "contract": contract_addr,
                    "function": func_name,
                    "calldata": format!("0x{}", hex::encode(&calldata)),
                    "note": "Full eth_call execution coming soon"
                })),
            })
        })
    }
}

/// Write to contract tool (state-changing) - sends transaction
pub struct WriteContractTool {
    wallet_manager: Arc<WalletManager>,
    node_manager: Arc<NodeManager>,
    password: Arc<RwLock<Option<String>>>,
}

impl WriteContractTool {
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

impl ToolHandler for WriteContractTool {
    fn name(&self) -> &str {
        "write_contract"
    }

    fn description(&self) -> &str {
        "Execute a state-changing contract function. Requires confirmation."
    }

    fn execute(
        &self,
        params: &IntentParams,
    ) -> Pin<Box<dyn Future<Output = Result<ToolOutput, DispatchError>> + Send + '_>> {
        let wallet_manager = self.wallet_manager.clone();
        let node_manager = self.node_manager.clone();
        let password = self.password.clone();
        let address = params.address.clone();
        let function = params.function_name.clone();
        let args = params.function_args.clone();
        let value = params.amount.clone();
        Box::pin(async move {
            let contract_addr = address.ok_or_else(|| {
                DispatchError::InvalidParams("Contract address required".to_string())
            })?;
            let func_name = function.unwrap_or_else(|| "unknown".to_string());

            // Get sender
            let accounts = wallet_manager.get_accounts().await;
            if accounts.is_empty() {
                return Ok(ToolOutput {
                    tool: "write_contract".to_string(),
                    success: false,
                    message: "No wallet accounts found.".to_string(),
                    data: None,
                });
            }
            let from_addr = accounts[0].address.clone();

            // Build calldata
            let calldata = if !args.is_empty() {
                let data_hex = args.join("").trim_start_matches("0x").to_string();
                hex::decode(&data_hex).unwrap_or_else(|_| encode_function_selector(&func_name))
            } else {
                encode_function_selector(&func_name)
            };

            // Parse value
            let tx_value = value.unwrap_or_else(|| "0".to_string());

            // Check password
            let pwd = password.read().await.clone();
            if pwd.is_none() {
                return Ok(ToolOutput {
                    tool: "write_contract".to_string(),
                    success: true,
                    message: format!(
                        "Contract call prepared: {} on {}. Awaiting confirmation.",
                        func_name,
                        if contract_addr.len() > 10 { &contract_addr[..10] } else { &contract_addr }
                    ),
                    data: Some(serde_json::json!({
                        "status": "pending_confirmation",
                        "from": from_addr,
                        "to": contract_addr,
                        "function": func_name,
                        "calldata": format!("0x{}", hex::encode(&calldata)),
                        "value": tx_value,
                        "requires_password": true
                    })),
                });
            }

            // Create transaction
            let tx_request = crate::wallet::TransactionRequest {
                from: from_addr.clone(),
                to: Some(contract_addr.clone()),
                value: tx_value.clone(),
                data: format!("0x{}", hex::encode(&calldata)),
                gas_limit: 500_000,
                gas_price: "1000000000".to_string(),
            };

            // Sign and send
            match wallet_manager
                .create_signed_transaction(tx_request, &pwd.unwrap())
                .await
            {
                Ok(tx) => {
                    // Format hash for display
                    let tx_hash = format!("{:?}", tx.hash);

                    // Add to mempool - Mempool is internally synchronized
                    if let Some(mempool) = node_manager.get_mempool().await {
                        use citrate_sequencer::mempool::TxClass;
                        let _ = mempool.add_transaction(tx.clone(), TxClass::Standard).await;
                    }

                    // Broadcast
                    use citrate_network::NetworkMessage;
                    let _ = node_manager
                        .broadcast_network(NetworkMessage::NewTransaction { transaction: tx })
                        .await;

                    Ok(ToolOutput {
                        tool: "write_contract".to_string(),
                        success: true,
                        message: format!(
                            "Contract call sent! TX: {}. Called {} on {}",
                            &tx_hash,
                            func_name,
                            if contract_addr.len() > 10 { &contract_addr[..10] } else { &contract_addr }
                        ),
                        data: Some(serde_json::json!({
                            "tx_hash": tx_hash,
                            "from": from_addr,
                            "to": contract_addr,
                            "function": func_name,
                            "value": tx_value,
                            "status": "pending"
                        })),
                    })
                }
                Err(e) => Ok(ToolOutput {
                    tool: "write_contract".to_string(),
                    success: false,
                    message: format!("Failed to send contract call: {}", e),
                    data: None,
                }),
            }
        })
    }

    fn requires_confirmation(&self) -> bool {
        true
    }
}

/// Calculate contract address from sender and nonce (CREATE opcode)
fn calculate_contract_address(sender: &str, nonce: u64) -> String {
    use sha3::{Digest, Keccak256};

    let sender_hex = sender.trim_start_matches("0x");
    let sender_bytes = hex::decode(sender_hex).unwrap_or_default();

    // RLP encode [sender, nonce]
    let mut rlp = Vec::new();

    // List header
    let sender_len = sender_bytes.len();
    let nonce_bytes = if nonce == 0 {
        vec![0x80] // RLP encoding of 0
    } else {
        let mut n = nonce;
        let mut bytes = Vec::new();
        while n > 0 {
            bytes.push((n & 0xff) as u8);
            n >>= 8;
        }
        bytes.reverse();
        if bytes.len() == 1 && bytes[0] < 0x80 {
            bytes
        } else {
            let mut result = vec![0x80 + bytes.len() as u8];
            result.extend(bytes);
            result
        }
    };

    let content_len = 1 + sender_len + nonce_bytes.len();
    if content_len < 56 {
        rlp.push(0xc0 + content_len as u8);
    } else {
        // Longer list (unlikely for addresses)
        let len_bytes = content_len.to_be_bytes();
        let len_len = 8 - len_bytes.iter().position(|&b| b != 0).unwrap_or(7);
        rlp.push(0xf7 + len_len as u8);
        rlp.extend(&len_bytes[8 - len_len..]);
    }

    // Sender (address is 20 bytes, so 0x80 + 20 = 0x94)
    rlp.push(0x80 + sender_len as u8);
    rlp.extend(&sender_bytes);

    // Nonce
    rlp.extend(nonce_bytes);

    // Hash and take last 20 bytes
    let hash = Keccak256::digest(&rlp);
    format!("0x{}", hex::encode(&hash[12..]))
}

/// Encode function selector from signature
fn encode_function_selector(signature: &str) -> Vec<u8> {
    use sha3::{Digest, Keccak256};

    // If it's already hex data, return it
    if signature.starts_with("0x") {
        if let Ok(bytes) = hex::decode(&signature[2..]) {
            return bytes;
        }
    }

    // Otherwise compute selector from signature
    let hash = Keccak256::digest(signature.as_bytes());
    hash[0..4].to_vec()
}
