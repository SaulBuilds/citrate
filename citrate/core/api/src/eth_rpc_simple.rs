// citrate/core/api/src/eth_rpc_simple.rs

use crate::methods::{ChainApi, StateApi};
use futures::executor::block_on;
use jsonrpc_core::{IoHandler, Params, Value};
use citrate_consensus::types::Hash;
use citrate_execution::executor::Executor;
use citrate_execution::types::Address;
use citrate_sequencer::mempool::Mempool;
use citrate_storage::StorageManager;
use serde_json::json;
use std::sync::Arc;

/// Add simplified Ethereum-compatible RPC methods to the IoHandler
pub fn register_eth_methods(
    io_handler: &mut IoHandler,
    storage: Arc<StorageManager>,
    mempool: Arc<Mempool>,
    executor: Arc<Executor>,
    chain_id: u64,
) {
    // eth_blockNumber - Returns the latest block number
    let storage_bn = storage.clone();
    io_handler.add_sync_method("eth_blockNumber", move |_params: Params| {
        let api = ChainApi::new(storage_bn.clone());
        match block_on(api.get_height()) {
            Ok(height) => Ok(Value::String(format!("0x{:x}", height))),
            Err(_) => Ok(Value::String("0x0".to_string())),
        }
    });

    // eth_getBlockByNumber - Returns block by number
    let storage_gbn = storage.clone();
    io_handler.add_sync_method("eth_getBlockByNumber", move |params: Params| {
        let api = ChainApi::new(storage_gbn.clone());
        
        let params: Vec<Value> = match params.parse() {
            Ok(p) => p,
            Err(_) => return Ok(Value::Null),
        };
        
        if params.is_empty() {
            return Ok(Value::Null);
        }
        
        let block_number = match params[0].as_str() {
            Some("latest") => {
                match block_on(api.get_height()) {
                    Ok(h) => h,
                    Err(_) => return Ok(Value::Null),
                }
            },
            Some(hex_str) if hex_str.starts_with("0x") => {
                match u64::from_str_radix(&hex_str[2..], 16) {
                    Ok(n) => n,
                    Err(_) => return Ok(Value::Null),
                }
            },
            _ => return Ok(Value::Null),
        };
        
        match block_on(api.get_block(crate::types::request::BlockId::Number(block_number))) {
            Ok(block) => {
                Ok(json!({
                    "number": format!("0x{:x}", block.height),
                    "hash": format!("0x{}", hex::encode(block.hash.as_bytes())),
                    "parentHash": format!("0x{}", hex::encode(block.parent_hash.as_bytes())),
                    "timestamp": format!("0x{:x}", block.timestamp),
                    "gasLimit": "0x1c9c380",
                    "gasUsed": "0x5208",
                    "difficulty": "0x0",
                    "totalDifficulty": "0x0",
                    "transactions": [],
                    "miner": "0x0000000000000000000000000000000000000000",
                    "mixHash": "0x0000000000000000000000000000000000000000000000000000000000000000",
                    "nonce": "0x0000000000000000",
                    "sha3Uncles": "0x1dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d49347",
                    "logsBloom": "0x00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000",
                    "transactionsRoot": format!("0x{}", hex::encode(block.tx_root.as_bytes())),
                    "stateRoot": format!("0x{}", hex::encode(block.state_root.as_bytes())),
                    "receiptsRoot": "0x56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421",
                    "size": "0x3e8",
                    "extraData": "0x",
                    "baseFeePerGas": "0x7",
                    "uncles": []
                }))
            },
            Err(_) => Ok(Value::Null),
        }
    });

    // eth_getBlockByHash - Returns block by hash
    let storage_gbh = storage.clone();
    io_handler.add_sync_method("eth_getBlockByHash", move |params: Params| {
        let api = ChainApi::new(storage_gbh.clone());
        
        let params: Vec<Value> = match params.parse() {
            Ok(p) => p,
            Err(_) => return Ok(Value::Null),
        };
        
        if params.is_empty() {
            return Ok(Value::Null);
        }
        
        let hash_str = match params[0].as_str() {
            Some(h) if h.starts_with("0x") => &h[2..],
            Some(h) => h,
            None => return Ok(Value::Null),
        };
        
        let hash_bytes = match hex::decode(hash_str) {
            Ok(b) if b.len() == 32 => {
                let mut arr = [0u8; 32];
                arr.copy_from_slice(&b);
                arr
            },
            _ => return Ok(Value::Null),
        };
        
        match block_on(api.get_block(crate::types::request::BlockId::Hash(Hash::new(hash_bytes)))) {
            Ok(block) => {
                Ok(json!({
                    "number": format!("0x{:x}", block.height),
                    "hash": format!("0x{}", hex::encode(block.hash.as_bytes())),
                    "parentHash": format!("0x{}", hex::encode(block.parent_hash.as_bytes())),
                    "timestamp": format!("0x{:x}", block.timestamp),
                    "gasLimit": "0x1c9c380",
                    "gasUsed": "0x5208",
                    "difficulty": "0x0",
                    "totalDifficulty": "0x0",
                    "transactions": [],
                    "miner": "0x0000000000000000000000000000000000000000",
                    "mixHash": "0x0000000000000000000000000000000000000000000000000000000000000000",
                    "nonce": "0x0000000000000000",
                    "sha3Uncles": "0x1dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d49347",
                    "logsBloom": "0x00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000",
                    "transactionsRoot": format!("0x{}", hex::encode(block.tx_root.as_bytes())),
                    "stateRoot": format!("0x{}", hex::encode(block.state_root.as_bytes())),
                    "receiptsRoot": "0x56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421",
                    "size": "0x3e8",
                    "extraData": "0x",
                    "baseFeePerGas": "0x7",
                    "uncles": []
                }))
            },
            Err(_) => Ok(Value::Null),
        }
    });

    // eth_getTransactionByHash - Returns null for now
    io_handler.add_sync_method("eth_getTransactionByHash", move |_params: Params| {
        Ok(Value::Null)
    });

    // eth_getTransactionReceipt - Returns null for now
    io_handler.add_sync_method("eth_getTransactionReceipt", move |_params: Params| {
        Ok(Value::Null)
    });

    // eth_chainId - Returns the configured chain ID
    io_handler.add_sync_method("eth_chainId", move |_params: Params| {
        Ok(Value::String(format!("0x{:x}", chain_id)))
    });

    // eth_syncing - Returns sync status
    io_handler.add_sync_method("eth_syncing", move |_params: Params| Ok(Value::Bool(false)));

    // net_peerCount - Returns number of peers
    io_handler.add_sync_method("net_peerCount", move |_params: Params| {
        Ok(Value::String("0x0".to_string()))
    });

    // eth_gasPrice - Returns current gas price
    io_handler.add_sync_method("eth_gasPrice", move |_params: Params| {
        Ok(Value::String("0x3b9aca00".to_string()))
    });

    // eth_getBalance - Returns account balance
    let storage_bal = storage.clone();
    let executor_bal = executor.clone();
    io_handler.add_sync_method("eth_getBalance", move |params: Params| {
        let state_api = StateApi::new(storage_bal.clone(), executor_bal.clone());

        let params: Vec<Value> = match params.parse() {
            Ok(p) => p,
            Err(_) => return Ok(Value::String("0x0".to_string())),
        };

        if params.is_empty() {
            return Ok(Value::String("0x0".to_string()));
        }

        let addr_str = match params[0].as_str() {
            Some(a) if a.starts_with("0x") => &a[2..],
            Some(a) => a,
            None => return Ok(Value::String("0x0".to_string())),
        };

        let addr_bytes = match hex::decode(addr_str) {
            Ok(b) if b.len() == 20 => {
                let mut arr = [0u8; 20];
                arr.copy_from_slice(&b);
                arr
            }
            _ => return Ok(Value::String("0x0".to_string())),
        };

        match block_on(state_api.get_balance(Address(addr_bytes))) {
            Ok(balance) => Ok(Value::String(format!("0x{:x}", balance))),
            Err(_) => Ok(Value::String("0x0".to_string())),
        }
    });

    // eth_getCode - Returns contract code
    let storage_code = storage.clone();
    let executor_code = executor.clone();
    io_handler.add_sync_method("eth_getCode", move |params: Params| {
        let state_api = StateApi::new(storage_code.clone(), executor_code.clone());

        let params: Vec<Value> = match params.parse() {
            Ok(p) => p,
            Err(_) => return Ok(Value::String("0x".to_string())),
        };

        if params.is_empty() {
            return Ok(Value::String("0x".to_string()));
        }

        let addr_str = match params[0].as_str() {
            Some(a) if a.starts_with("0x") => &a[2..],
            Some(a) => a,
            None => return Ok(Value::String("0x".to_string())),
        };

        let addr_bytes = match hex::decode(addr_str) {
            Ok(b) if b.len() == 20 => {
                let mut arr = [0u8; 20];
                arr.copy_from_slice(&b);
                arr
            }
            _ => return Ok(Value::String("0x".to_string())),
        };

        match block_on(state_api.get_code(Address(addr_bytes))) {
            Ok(code) => Ok(Value::String(format!("0x{}", hex::encode(code)))),
            Err(_) => Ok(Value::String("0x".to_string())),
        }
    });

    // eth_getTransactionCount - Returns account nonce
    let storage_nonce = storage.clone();
    let executor_nonce = executor.clone();
    let mempool_nonce = mempool.clone();
    io_handler.add_sync_method("eth_getTransactionCount", move |params: Params| {
        let state_api = StateApi::new(storage_nonce.clone(), executor_nonce.clone());

        let params: Vec<Value> = match params.parse() {
            Ok(p) => p,
            Err(_) => return Ok(Value::String("0x0".to_string())),
        };

        if params.is_empty() {
            return Ok(Value::String("0x0".to_string()));
        }

        let addr_str = match params[0].as_str() {
            Some(a) if a.starts_with("0x") => &a[2..],
            Some(a) => a,
            None => return Ok(Value::String("0x0".to_string())),
        };

        let addr_bytes = match hex::decode(addr_str) {
            Ok(b) if b.len() == 20 => {
                let mut arr = [0u8; 20];
                arr.copy_from_slice(&b);
                arr
            }
            _ => return Ok(Value::String("0x0".to_string())),
        };

        // Optional second param: block tag ("latest" | "pending" | "earliest")
        let tag = params.get(1).and_then(|v| v.as_str()).unwrap_or("latest");

        let base_nonce = match block_on(state_api.get_nonce(Address(addr_bytes))) {
            Ok(nonce) => nonce,
            Err(_) => return Ok(Value::String("0x0".to_string())),
        };

        if tag.eq_ignore_ascii_case("pending") {
            // Include pending mempool transactions from this sender
            let mp = mempool_nonce.clone();
            let stats = block_on(mp.stats());
            let total = stats.total_transactions;
            let txs = block_on(mp.get_transactions(total));

            let mut max_pending_nonce = None;
            for tx in txs {
                // Derive sender address from tx.from using unified address logic
                let sender_addr = citrate_execution::address_utils::normalize_address(&tx.from);
                if sender_addr.0 == addr_bytes {
                    max_pending_nonce = Some(max_pending_nonce.map_or(tx.nonce, |m: u64| m.max(tx.nonce)));
                }
            }

            let pending_nonce = match max_pending_nonce {
                Some(max_nonce) => (max_nonce + 1).max(base_nonce),
                None => base_nonce,
            };

            return Ok(Value::String(format!("0x{:x}", pending_nonce)));
        }

        Ok(Value::String(format!("0x{:x}", base_nonce)))
    });

    // eth_sendRawTransaction - Submit signed transaction
    io_handler.add_sync_method("eth_sendRawTransaction", move |_params: Params| {
        // Return mock hash for now
        let mock_hash = Hash::new([0u8; 32]);
        Ok(Value::String(format!(
            "0x{}",
            hex::encode(mock_hash.as_bytes())
        )))
    });

    // eth_call - Execute call without creating transaction
    io_handler.add_sync_method("eth_call", move |_params: Params| {
        Ok(Value::String("0x".to_string()))
    });

    // eth_estimateGas - Estimate gas for transaction
    // For simple RPC, we return estimates based on transaction type
    io_handler.add_sync_method("eth_estimateGas", move |params: Params| {
        let params: Vec<serde_json::Value> = match params.parse() {
            Ok(p) => p,
            Err(_) => return Ok(Value::String("0x5208".to_string())), // 21000 default
        };

        if params.is_empty() {
            return Ok(Value::String("0x5208".to_string())); // 21000 default
        }

        // Check if call object has data (indicates contract call/deployment)
        if let Some(obj) = params[0].as_object() {
            let has_data = obj.get("data")
                .and_then(|v| v.as_str())
                .map(|s| s.len() > 2) // "0x" is empty
                .unwrap_or(false);

            let has_to = obj.get("to").is_some();

            if !has_to && has_data {
                // Contract deployment - estimate higher
                return Ok(Value::String("0x4c4b40".to_string())); // 5000000 gas
            } else if has_data {
                // Contract call - estimate medium
                return Ok(Value::String("0x186a0".to_string())); // 100000 gas
            }
        }

        // Simple transfer
        Ok(Value::String("0x5208".to_string())) // 21000 gas
    });
}
