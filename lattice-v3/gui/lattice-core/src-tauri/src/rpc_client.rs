use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::atomic::{AtomicU64, Ordering};

/// RPC client for connecting to external Lattice nodes
pub struct RpcClient {
    url: String,
    client: reqwest::Client,
    request_id: AtomicU64,
}

impl RpcClient {
    pub fn new(url: String) -> Self {
        Self {
            url,
            client: reqwest::Client::new(),
            request_id: AtomicU64::new(1),
        }
    }

    async fn call(&self, method: &str, params: Value) -> Result<Value> {
        let id = self.request_id.fetch_add(1, Ordering::SeqCst);
        
        let request = json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
            "id": id
        });

        let response = self.client
            .post(&self.url)
            .json(&request)
            .send()
            .await
            .map_err(|e| anyhow!("RPC request failed: {}", e))?;

        let result: JsonRpcResponse = response
            .json()
            .await
            .map_err(|e| anyhow!("Failed to parse RPC response: {}", e))?;

        if let Some(error) = result.error {
            return Err(anyhow!("RPC error {}: {}", error.code, error.message));
        }

        result.result.ok_or_else(|| anyhow!("Empty RPC response"))
    }

    pub async fn get_chain_id(&self) -> Result<u64> {
        let result = self.call("eth_chainId", json!([])).await?;
        let chain_id_hex = result
            .as_str()
            .ok_or_else(|| anyhow!("Invalid chain ID response"))?;
        
        // Parse hex string (0x prefix)
        let chain_id = u64::from_str_radix(chain_id_hex.trim_start_matches("0x"), 16)
            .map_err(|e| anyhow!("Failed to parse chain ID: {}", e))?;
        
        Ok(chain_id)
    }

    pub async fn get_block_number(&self) -> Result<u64> {
        let result = self.call("eth_blockNumber", json!([])).await?;
        let block_hex = result
            .as_str()
            .ok_or_else(|| anyhow!("Invalid block number response"))?;
        
        let block_number = u64::from_str_radix(block_hex.trim_start_matches("0x"), 16)
            .map_err(|e| anyhow!("Failed to parse block number: {}", e))?;
        
        Ok(block_number)
    }

    pub async fn get_balance(&self, address: &str) -> Result<String> {
        let params = json!([address, "latest"]);
        let result = self.call("eth_getBalance", params).await?;
        
        let balance_hex = result
            .as_str()
            .ok_or_else(|| anyhow!("Invalid balance response"))?;
        
        Ok(balance_hex.to_string())
    }

    pub async fn get_transaction_count(&self, address: &str) -> Result<u64> {
        let params = json!([address, "pending"]); // Use pending for correct nonce
        let result = self.call("eth_getTransactionCount", params).await?;
        
        let nonce_hex = result
            .as_str()
            .ok_or_else(|| anyhow!("Invalid nonce response"))?;
        
        let nonce = u64::from_str_radix(nonce_hex.trim_start_matches("0x"), 16)
            .map_err(|e| anyhow!("Failed to parse nonce: {}", e))?;
        
        Ok(nonce)
    }

    pub async fn send_raw_transaction(&self, tx_data: &str) -> Result<String> {
        let params = json!([tx_data]);
        let result = self.call("eth_sendRawTransaction", params).await?;
        
        let tx_hash = result
            .as_str()
            .ok_or_else(|| anyhow!("Invalid transaction hash response"))?;
        
        Ok(tx_hash.to_string())
    }

    pub async fn get_transaction_receipt(&self, tx_hash: &str) -> Result<Option<Value>> {
        let params = json!([tx_hash]);
        let result = self.call("eth_getTransactionReceipt", params).await?;
        
        if result.is_null() {
            Ok(None)
        } else {
            Ok(Some(result))
        }
    }

    pub async fn estimate_gas(&self, from: &str, to: Option<&str>, value: &str, data: &str) -> Result<u64> {
        let mut tx_obj = json!({
            "from": from,
            "value": value,
            "data": data
        });
        
        if let Some(to_addr) = to {
            tx_obj["to"] = json!(to_addr);
        }
        
        let params = json!([tx_obj]);
        let result = self.call("eth_estimateGas", params).await?;
        
        let gas_hex = result
            .as_str()
            .ok_or_else(|| anyhow!("Invalid gas estimate response"))?;
        
        let gas = u64::from_str_radix(gas_hex.trim_start_matches("0x"), 16)
            .map_err(|e| anyhow!("Failed to parse gas estimate: {}", e))?;
        
        Ok(gas)
    }

    pub async fn get_gas_price(&self) -> Result<u64> {
        let result = self.call("eth_gasPrice", json!([])).await?;
        let price_hex = result
            .as_str()
            .ok_or_else(|| anyhow!("Invalid gas price response"))?;
        
        let gas_price = u64::from_str_radix(price_hex.trim_start_matches("0x"), 16)
            .map_err(|e| anyhow!("Failed to parse gas price: {}", e))?;
        
        Ok(gas_price)
    }

    /// Check if the RPC endpoint is accessible
    pub async fn health_check(&self) -> Result<()> {
        // Try to get chain ID as a simple health check
        self.get_chain_id().await?;
        Ok(())
    }
}

#[derive(Debug, Deserialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<JsonRpcError>,
    id: u64,
}

#[derive(Debug, Deserialize)]
struct JsonRpcError {
    code: i32,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<Value>,
}