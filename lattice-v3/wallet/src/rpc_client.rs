use crate::errors::WalletError;
use crate::transaction::SignedTransaction;
use lattice_consensus::types::Hash;
use lattice_execution::types::Address;
use primitive_types::U256;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

/// JSON-RPC request
#[derive(Debug, Serialize)]
struct RpcRequest {
    jsonrpc: String,
    method: String,
    params: Value,
    id: u64,
}

/// JSON-RPC response
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct RpcResponse {
    jsonrpc: String,
    #[serde(default)]
    result: Value,
    #[serde(default)]
    error: Option<RpcError>,
    id: u64,
}

/// JSON-RPC error
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct RpcError {
    code: i32,
    message: String,
    #[serde(default)]
    data: Option<Value>,
}

/// RPC client for blockchain interaction
pub struct RpcClient {
    url: String,
    client: Client,
    request_id: std::sync::atomic::AtomicU64,
}

impl RpcClient {
    /// Create new RPC client
    pub fn new(url: &str) -> Self {
        Self {
            url: url.to_string(),
            client: Client::new(),
            request_id: std::sync::atomic::AtomicU64::new(1),
        }
    }
    
    /// Make RPC call
    async fn call(&self, method: &str, params: Value) -> Result<Value, WalletError> {
        let id = self.request_id.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        
        let request = RpcRequest {
            jsonrpc: "2.0".to_string(),
            method: method.to_string(),
            params,
            id,
        };
        
        let response = self.client
            .post(&self.url)
            .json(&request)
            .send()
            .await
            .map_err(|e| WalletError::Rpc(e.to_string()))?;
        
        let rpc_response: RpcResponse = response
            .json()
            .await
            .map_err(|e| WalletError::Rpc(e.to_string()))?;
        
        if let Some(error) = rpc_response.error {
            return Err(WalletError::Rpc(format!("{}: {}", error.code, error.message)));
        }
        
        Ok(rpc_response.result)
    }
    
    /// Get balance
    pub async fn get_balance(&self, address: &Address) -> Result<U256, WalletError> {
        let params = json!([
            format!("0x{}", hex::encode(address.0)),
            "latest"
        ]);
        
        let result = self.call("eth_getBalance", params).await?;
        
        let balance_hex = result
            .as_str()
            .ok_or_else(|| WalletError::Rpc("Invalid balance response".to_string()))?;
        
        let balance_str = balance_hex.trim_start_matches("0x");
        let balance = U256::from_str_radix(balance_str, 16)
            .map_err(|e| WalletError::Rpc(format!("Failed to parse balance: {}", e)))?;
        
        Ok(balance)
    }
    
    /// Get nonce
    pub async fn get_nonce(&self, address: &Address) -> Result<u64, WalletError> {
        let params = json!([
            format!("0x{}", hex::encode(address.0)),
            "pending"
        ]);
        
        let result = self.call("eth_getTransactionCount", params).await?;
        
        let nonce_hex = result
            .as_str()
            .ok_or_else(|| WalletError::Rpc("Invalid nonce response".to_string()))?;
        
        let nonce_str = nonce_hex.trim_start_matches("0x");
        let nonce = u64::from_str_radix(nonce_str, 16)
            .map_err(|e| WalletError::Rpc(format!("Failed to parse nonce: {}", e)))?;
        
        Ok(nonce)
    }
    
    /// Send transaction
    pub async fn send_transaction(&self, tx: SignedTransaction) -> Result<Hash, WalletError> {
        // Convert transaction to hex
        let tx_hex = format!("0x{}", hex::encode(&tx.raw));
        
        let params = json!([tx_hex]);
        
        let result = self.call("eth_sendRawTransaction", params).await?;
        
        let tx_hash_hex = result
            .as_str()
            .ok_or_else(|| WalletError::Rpc("Invalid transaction hash response".to_string()))?;
        
        let hash_bytes = hex::decode(tx_hash_hex.trim_start_matches("0x"))
            .map_err(|e| WalletError::Rpc(format!("Failed to parse tx hash: {}", e)))?;
        
        if hash_bytes.len() != 32 {
            return Err(WalletError::Rpc("Invalid transaction hash length".to_string()));
        }
        
        let mut hash_array = [0u8; 32];
        hash_array.copy_from_slice(&hash_bytes);
        
        Ok(Hash::new(hash_array))
    }
    
    /// Get transaction receipt
    pub async fn get_transaction_receipt(&self, tx_hash: &Hash) -> Result<Option<Value>, WalletError> {
        let params = json!([format!("0x{}", hex::encode(tx_hash.as_bytes()))]);
        
        let result = self.call("eth_getTransactionReceipt", params).await?;
        
        if result.is_null() {
            Ok(None)
        } else {
            Ok(Some(result))
        }
    }
    
    /// Get block number
    pub async fn get_block_number(&self) -> Result<u64, WalletError> {
        let result = self.call("eth_blockNumber", json!([])).await?;
        
        let block_hex = result
            .as_str()
            .ok_or_else(|| WalletError::Rpc("Invalid block number response".to_string()))?;
        
        let block_str = block_hex.trim_start_matches("0x");
        let block_number = u64::from_str_radix(block_str, 16)
            .map_err(|e| WalletError::Rpc(format!("Failed to parse block number: {}", e)))?;
        
        Ok(block_number)
    }
    
    /// Get chain ID
    pub async fn get_chain_id(&self) -> Result<u64, WalletError> {
        let result = self.call("eth_chainId", json!([])).await?;
        
        let chain_hex = result
            .as_str()
            .ok_or_else(|| WalletError::Rpc("Invalid chain ID response".to_string()))?;
        
        let chain_str = chain_hex.trim_start_matches("0x");
        let chain_id = u64::from_str_radix(chain_str, 16)
            .map_err(|e| WalletError::Rpc(format!("Failed to parse chain ID: {}", e)))?;
        
        Ok(chain_id)
    }
    
    /// Get gas price
    pub async fn get_gas_price(&self) -> Result<u64, WalletError> {
        let result = self.call("eth_gasPrice", json!([])).await?;
        
        let gas_hex = result
            .as_str()
            .ok_or_else(|| WalletError::Rpc("Invalid gas price response".to_string()))?;
        
        let gas_str = gas_hex.trim_start_matches("0x");
        let gas_price = u64::from_str_radix(gas_str, 16)
            .map_err(|e| WalletError::Rpc(format!("Failed to parse gas price: {}", e)))?;
        
        Ok(gas_price)
    }
    
    /// Estimate gas
    pub async fn estimate_gas(&self, from: &Address, to: Option<&Address>, value: U256, data: Vec<u8>) -> Result<u64, WalletError> {
        let mut tx_object = json!({
            "from": format!("0x{}", hex::encode(from.0)),
            "value": format!("0x{:x}", value),
        });
        
        if let Some(to_addr) = to {
            tx_object["to"] = json!(format!("0x{}", hex::encode(to_addr.0)));
        }
        
        if !data.is_empty() {
            tx_object["data"] = json!(format!("0x{}", hex::encode(&data)));
        }
        
        let params = json!([tx_object]);
        
        let result = self.call("eth_estimateGas", params).await?;
        
        let gas_hex = result
            .as_str()
            .ok_or_else(|| WalletError::Rpc("Invalid gas estimate response".to_string()))?;
        
        let gas_str = gas_hex.trim_start_matches("0x");
        let gas_estimate = u64::from_str_radix(gas_str, 16)
            .map_err(|e| WalletError::Rpc(format!("Failed to parse gas estimate: {}", e)))?;
        
        Ok(gas_estimate)
    }
}
