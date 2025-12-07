// citrate/core/api/src/eth_rpc.rs

use crate::eth_tx_decoder;
use crate::filter::{FilterRegistry, FilterType};
use crate::methods::{ChainApi, StateApi, TransactionApi};
use futures::executor::block_on;
use hex;
use jsonrpc_core::{IoHandler, Params, Value};
use citrate_consensus::types::{Hash, Transaction};
use citrate_execution::executor::Executor;
use citrate_execution::types::Address;
use citrate_sequencer::mempool::{Mempool, TxClass};
use citrate_storage::StorageManager;
use primitive_types::U256;
use serde_json::json;
use std::sync::Arc;

/// Add Ethereum-compatible RPC methods to the IoHandler
pub fn register_eth_methods(
    io_handler: &mut IoHandler,
    storage: Arc<StorageManager>,
    mempool: Arc<Mempool>,
    executor: Arc<Executor>,
    chain_id: u64,
    filter_registry: Arc<FilterRegistry>,
) {
    // eth_blockNumber - Returns the latest block number
    let storage_bn = storage.clone();
    io_handler.add_sync_method("eth_blockNumber", move |_params: Params| {
        let api = ChainApi::new(storage_bn.clone());
        match block_on(api.get_height()) {
            Ok(height) => {
                // Return as hex string as per Ethereum JSON-RPC spec
                Ok(Value::String(format!("0x{:x}", height)))
            }
            Err(_) => Ok(Value::String("0x0".to_string())),
        }
    });

    // eth_getBlockByNumber - Returns block by number
    let storage_gbn = storage.clone();
    io_handler.add_sync_method("eth_getBlockByNumber", move |params: Params| {
        let api = ChainApi::new(storage_gbn.clone());
        
        // Parse params: [blockNumber, includeTransactions]
        let params: Vec<Value> = match params.parse() {
            Ok(p) => p,
            Err(e) => return Err(jsonrpc_core::Error::invalid_params(e.to_string())),
        };
        
        if params.is_empty() {
            return Err(jsonrpc_core::Error::invalid_params("Missing block number"));
        }
        
        // Parse includeTransactions flag (default false)
        let include_transactions = params.get(1)
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        
        // Parse block number from hex string or "latest"
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
                    Err(_) => return Err(jsonrpc_core::Error::invalid_params("Invalid block number")),
                }
            },
            _ => return Err(jsonrpc_core::Error::invalid_params("Invalid block number format")),
        };
        
        // Get block from storage
        match block_on(api.get_block(crate::types::request::BlockId::Number(block_number))) {
            Ok(block) => {
                // Build transactions array based on includeTransactions flag
                let transactions = if include_transactions {
                    // Return full transaction objects
                    block.transactions.iter().enumerate().map(|(index, tx)| {
                        json!({
                            "hash": format!("0x{}", hex::encode(tx.hash.as_bytes())),
                            "from": format!("0x{}", tx.from),
                            "to": tx.to.as_ref().map(|addr| format!("0x{}", addr)),
                            "value": format!("0x{:x}", tx.value),
                            "gas": format!("0x{:x}", tx.gas_limit),
                            "gasPrice": format!("0x{:x}", tx.gas_price),
                            "nonce": format!("0x{:x}", tx.nonce),
                            "input": format!("0x{}", hex::encode(&tx.data)),
                            "blockHash": format!("0x{}", hex::encode(block.hash.as_bytes())),
                            "blockNumber": format!("0x{:x}", block.height),
                            "transactionIndex": format!("0x{:x}", index)
                        })
                    }).collect::<Vec<_>>()
                } else {
                    // Return just transaction hashes
                    block.transactions.iter()
                        .map(|tx| Value::String(format!("0x{}", hex::encode(tx.hash.as_bytes()))))
                        .collect::<Vec<_>>()
                };
                
                // Convert to Ethereum-compatible format
                Ok(json!({
                    "number": format!("0x{:x}", block.height),
                    "hash": format!("0x{}", hex::encode(block.hash.as_bytes())),
                    "parentHash": format!("0x{}", hex::encode(block.parent_hash.as_bytes())),
                    "timestamp": format!("0x{:x}", block.timestamp),
                    "gasLimit": "0x1c9c380", // 30M gas
                    "gasUsed": "0x5208",
                    "difficulty": "0x0", // PoS
                    "totalDifficulty": "0x0",
                    "transactions": transactions,
                    "miner": "0x0000000000000000000000000000000000000000",
                    "mixHash": "0x0000000000000000000000000000000000000000000000000000000000000000",
                    "nonce": "0x0000000000000000",
                    "sha3Uncles": "0x1dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d49347",
                    "logsBloom": "0x00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000",
                    "transactionsRoot": format!("0x{}", hex::encode(block.tx_root.as_bytes())),
                    "stateRoot": format!("0x{}", hex::encode(block.state_root.as_bytes())),
                    "receiptsRoot": "0x56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421",
                    "size": format!("0x{:x}", 1000), // Approximate
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
            Err(e) => return Err(jsonrpc_core::Error::invalid_params(e.to_string())),
        };
        
        if params.is_empty() {
            return Err(jsonrpc_core::Error::invalid_params("Missing block hash"));
        }
        
        // Parse includeTransactions flag (default false)
        let include_transactions = params.get(1)
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        
        // Parse block hash
        let hash_str = match params[0].as_str() {
            Some(h) if h.starts_with("0x") => &h[2..],
            Some(h) => h,
            None => return Err(jsonrpc_core::Error::invalid_params("Invalid hash format")),
        };
        
        let hash_bytes = match hex::decode(hash_str) {
            Ok(b) if b.len() == 32 => {
                let mut arr = [0u8; 32];
                arr.copy_from_slice(&b);
                arr
            },
            _ => return Err(jsonrpc_core::Error::invalid_params("Invalid hash length")),
        };
        
        match block_on(api.get_block(crate::types::request::BlockId::Hash(Hash::new(hash_bytes)))) {
            Ok(block) => {
                // Build transactions array based on includeTransactions flag
                let transactions = if include_transactions {
                    // Return full transaction objects
                    block.transactions.iter().enumerate().map(|(index, tx)| {
                        json!({
                            "hash": format!("0x{}", hex::encode(tx.hash.as_bytes())),
                            "from": format!("0x{}", tx.from),
                            "to": tx.to.as_ref().map(|addr| format!("0x{}", addr)),
                            "value": format!("0x{:x}", tx.value),
                            "gas": format!("0x{:x}", tx.gas_limit),
                            "gasPrice": format!("0x{:x}", tx.gas_price),
                            "nonce": format!("0x{:x}", tx.nonce),
                            "input": format!("0x{}", hex::encode(&tx.data)),
                            "blockHash": format!("0x{}", hex::encode(block.hash.as_bytes())),
                            "blockNumber": format!("0x{:x}", block.height),
                            "transactionIndex": format!("0x{:x}", index)
                        })
                    }).collect::<Vec<_>>()
                } else {
                    // Return just transaction hashes
                    block.transactions.iter()
                        .map(|tx| Value::String(format!("0x{}", hex::encode(tx.hash.as_bytes()))))
                        .collect::<Vec<_>>()
                };
                
                Ok(json!({
                    "number": format!("0x{:x}", block.height),
                    "hash": format!("0x{}", hex::encode(block.hash.as_bytes())),
                    "parentHash": format!("0x{}", hex::encode(block.parent_hash.as_bytes())),
                    "timestamp": format!("0x{:x}", block.timestamp),
                    "gasLimit": "0x1c9c380",
                    "gasUsed": "0x5208",
                    "difficulty": "0x0",
                    "totalDifficulty": "0x0",
                    "transactions": transactions,
                    "miner": "0x0000000000000000000000000000000000000000",
                    "mixHash": "0x0000000000000000000000000000000000000000000000000000000000000000",
                    "nonce": "0x0000000000000000",
                    "sha3Uncles": "0x1dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d49347",
                    "logsBloom": "0x00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000",
                    "transactionsRoot": format!("0x{}", hex::encode(block.tx_root.as_bytes())),
                    "stateRoot": format!("0x{}", hex::encode(block.state_root.as_bytes())),
                    "receiptsRoot": "0x56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421",
                    "size": format!("0x{:x}", 1000),
                    "extraData": "0x",
                    "baseFeePerGas": "0x7",
                    "uncles": []
                }))
            },
            Err(_) => Ok(Value::Null),
        }
    });

    // eth_getTransactionByHash - Returns transaction by hash
    let storage_tx = storage.clone();
    let mempool_tx_lookup = mempool.clone();
    io_handler.add_sync_method("eth_getTransactionByHash", move |params: Params| {
        let api = ChainApi::new(storage_tx.clone());
        
        let params: Vec<Value> = match params.parse() {
            Ok(p) => p,
            Err(e) => return Err(jsonrpc_core::Error::invalid_params(e.to_string())),
        };
        
        if params.is_empty() {
            return Err(jsonrpc_core::Error::invalid_params("Missing transaction hash"));
        }
        
        let hash_str = match params[0].as_str() {
            Some(h) if h.starts_with("0x") => &h[2..],
            Some(h) => h,
            None => return Err(jsonrpc_core::Error::invalid_params("Invalid hash format")),
        };
        
        let hash_bytes = match hex::decode(hash_str) {
            Ok(b) if b.len() == 32 => {
                let mut arr = [0u8; 32];
                arr.copy_from_slice(&b);
                arr
            },
            _ => return Err(jsonrpc_core::Error::invalid_params("Invalid hash length")),
        };
        
        let h = Hash::new(hash_bytes);
        match block_on(api.get_transaction(h)) {
            Ok(tx) => {
                let from_hex = format!("0x{}", tx.from);
                let to_hex_opt = tx.to.as_ref().map(|s| format!("0x{}", s));
                Ok(json!({
                    "hash": format!("0x{}", hex::encode(tx.hash.as_bytes())),
                    "nonce": format!("0x{:x}", tx.nonce),
                    "blockHash": "0x0000000000000000000000000000000000000000000000000000000000000000",
                    "blockNumber": "0x0",
                    "transactionIndex": "0x0",
                    "from": from_hex,
                    "to": to_hex_opt,
                    "value": format!("0x{:x}", tx.value),
                    "gasPrice": format!("0x{:x}", tx.gas_price),
                    "gas": format!("0x{:x}", tx.gas_limit),
                    "input": format!("0x{}", hex::encode(&tx.data)),
                    "v": "0x1b",
                    "r": "0x0000000000000000000000000000000000000000000000000000000000000000",
                    "s": "0x0000000000000000000000000000000000000000000000000000000000000000",
                    "type": "0x0"
                }))
            },
            Err(_) => {
                // Fallback: check mempool for pending transaction
                if let Some(tx) = block_on(mempool_tx_lookup.get_transaction(&h)) {
                    let from_addr = citrate_execution::address_utils::normalize_address(&tx.from);
                    let to_addr_opt = tx.to.as_ref().map(citrate_execution::address_utils::normalize_address);
                    Ok(json!({
                        "hash": format!("0x{}", hex::encode(tx.hash.as_bytes())),
                        "nonce": format!("0x{:x}", tx.nonce),
                        "blockHash": Value::Null,
                        "blockNumber": Value::Null,
                        "transactionIndex": Value::Null,
                        "from": format!("0x{}", hex::encode(from_addr.0)),
                        "to": to_addr_opt.map(|a| format!("0x{}", hex::encode(a.0))),
                        "value": format!("0x{:x}", tx.value),
                        "gasPrice": format!("0x{:x}", tx.gas_price),
                        "gas": format!("0x{:x}", tx.gas_limit),
                        "input": format!("0x{}", hex::encode(&tx.data)),
                        "v": "0x1b",
                        "r": "0x0000000000000000000000000000000000000000000000000000000000000000",
                        "s": "0x0000000000000000000000000000000000000000000000000000000000000000",
                        "type": "0x0"
                    }))
                } else {
                    Ok(Value::Null)
                }
            },
        }
    });

    // eth_getTransactionReceipt - Returns transaction receipt
    let storage_rcpt = storage.clone();
    io_handler.add_sync_method("eth_getTransactionReceipt", move |params: Params| {
        let api = ChainApi::new(storage_rcpt.clone());
        
        let params: Vec<Value> = match params.parse() {
            Ok(p) => p,
            Err(e) => return Err(jsonrpc_core::Error::invalid_params(e.to_string())),
        };
        
        if params.is_empty() {
            return Err(jsonrpc_core::Error::invalid_params("Missing transaction hash"));
        }
        
        let hash_str = match params[0].as_str() {
            Some(h) if h.starts_with("0x") => &h[2..],
            Some(h) => h,
            None => return Err(jsonrpc_core::Error::invalid_params("Invalid hash format")),
        };
        
        let hash_bytes = match hex::decode(hash_str) {
            Ok(b) if b.len() == 32 => {
                let mut arr = [0u8; 32];
                arr.copy_from_slice(&b);
                arr
            },
            _ => return Err(jsonrpc_core::Error::invalid_params("Invalid hash length")),
        };
        
        match block_on(api.get_receipt(Hash::new(hash_bytes))) {
            Ok(receipt) => {
                // Derive contractAddress if deployment output encodes address
                let contract_address = if receipt.to.is_none() && receipt.output.len() == 20 {
                    Some(format!("0x{}", hex::encode(&receipt.output)))
                } else {
                    None
                };

                Ok(json!({
                    "transactionHash": format!("0x{}", hex::encode(receipt.tx_hash.as_bytes())),
                    "transactionIndex": "0x0",
                    "blockHash": format!("0x{}", hex::encode(receipt.block_hash.as_bytes())),
                    "blockNumber": format!("0x{:x}", receipt.block_number),
                    "from": format!("0x{}", hex::encode(receipt.from.0)),
                    "to": receipt.to.as_ref().map(|t| format!("0x{}", hex::encode(t.0))),
                    "cumulativeGasUsed": format!("0x{:x}", receipt.gas_used),
                    "gasUsed": format!("0x{:x}", receipt.gas_used),
                    "contractAddress": contract_address,
                    "logs": receipt.logs.iter().map(|log| json!({
                        "address": format!("0x{}", hex::encode(log.address.0)),
                        "topics": log.topics.iter()
                            .map(|t| format!("0x{}", hex::encode(t.as_bytes())))
                            .collect::<Vec<_>>(),
                        "data": format!("0x{}", hex::encode(&log.data)),
                        "logIndex": "0x0",
                        "transactionIndex": "0x0",
                        "transactionHash": format!("0x{}", hex::encode(receipt.tx_hash.as_bytes())),
                        "blockHash": format!("0x{}", hex::encode(receipt.block_hash.as_bytes())),
                        "blockNumber": format!("0x{:x}", receipt.block_number),
                        "removed": false
                    })).collect::<Vec<_>>(),
                    "status": if receipt.status { "0x1" } else { "0x0" },
                    "logsBloom": "0x00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000",
                    "type": "0x0",
                    "effectiveGasPrice": "0x0"
                }))
            },
            Err(_) => Ok(Value::Null),
        }
    });

    // eth_chainId - Returns the chain ID
    io_handler.add_sync_method("eth_chainId", move |_params: Params| {
        // Return configured chain ID in hex
        Ok(Value::String(format!("0x{:x}", chain_id)))
    });

    // eth_syncing - Returns sync status
    io_handler.add_sync_method("eth_syncing", move |_params: Params| {
        // Return false when fully synced
        Ok(Value::Bool(false))
    });

    // net_peerCount handled in server.rs with NetworkApi to reflect real peers

    // eth_gasPrice - Returns current gas price
    io_handler.add_sync_method("eth_gasPrice", move |_params: Params| {
        // Return 1 gwei
        Ok(Value::String("0x3b9aca00".to_string()))
    });

    // eth_getBalance - Returns account balance
    let storage_bal = storage.clone();
    let executor_bal = executor.clone();
    io_handler.add_sync_method("eth_getBalance", move |params: Params| {
        let state_api = StateApi::new(storage_bal.clone(), executor_bal.clone());

        let params: Vec<Value> = match params.parse() {
            Ok(p) => p,
            Err(e) => return Err(jsonrpc_core::Error::invalid_params(e.to_string())),
        };

        if params.is_empty() {
            return Err(jsonrpc_core::Error::invalid_params("Missing address"));
        }

        let addr_str = match params[0].as_str() {
            Some(a) if a.starts_with("0x") => &a[2..],
            Some(a) => a,
            None => {
                return Err(jsonrpc_core::Error::invalid_params(
                    "Invalid address format",
                ))
            }
        };

        let addr_bytes = match hex::decode(addr_str) {
            Ok(b) if b.len() == 20 => {
                let mut arr = [0u8; 20];
                arr.copy_from_slice(&b);
                arr
            }
            _ => {
                return Err(jsonrpc_core::Error::invalid_params(
                    "Invalid address length",
                ))
            }
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
            Err(e) => return Err(jsonrpc_core::Error::invalid_params(e.to_string())),
        };

        if params.is_empty() {
            return Err(jsonrpc_core::Error::invalid_params("Missing address"));
        }

        let addr_str = match params[0].as_str() {
            Some(a) if a.starts_with("0x") => &a[2..],
            Some(a) => a,
            None => {
                return Err(jsonrpc_core::Error::invalid_params(
                    "Invalid address format",
                ))
            }
        };

        let addr_bytes = match hex::decode(addr_str) {
            Ok(b) if b.len() == 20 => {
                let mut arr = [0u8; 20];
                arr.copy_from_slice(&b);
                arr
            }
            _ => {
                return Err(jsonrpc_core::Error::invalid_params(
                    "Invalid address length",
                ))
            }
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
            Err(e) => return Err(jsonrpc_core::Error::invalid_params(e.to_string())),
        };

        if params.is_empty() {
            return Err(jsonrpc_core::Error::invalid_params("Missing address"));
        }

        let addr_str = match params[0].as_str() {
            Some(a) if a.starts_with("0x") => &a[2..],
            Some(a) => a,
            None => {
                return Err(jsonrpc_core::Error::invalid_params(
                    "Invalid address format",
                ))
            }
        };

        let addr_bytes = match hex::decode(addr_str) {
            Ok(b) if b.len() == 20 => {
                let mut arr = [0u8; 20];
                arr.copy_from_slice(&b);
                arr
            }
            _ => {
                return Err(jsonrpc_core::Error::invalid_params(
                    "Invalid address length",
                ))
            }
        };

        // Optional second param: block tag ("latest" | "pending" | "earliest")
        let tag = params.get(1).and_then(|v| v.as_str()).unwrap_or("latest");

        let base_nonce = block_on(state_api.get_nonce(Address(addr_bytes))).unwrap_or_default();

        if tag.eq_ignore_ascii_case("pending") {
            // Include pending mempool transactions from this sender
            let mp = mempool_nonce.clone();
            let total = block_on(mp.stats()).total_transactions;
            let txs = block_on(mp.get_transactions(total));
            let mut max_nonce = None;
            for tx in txs {
                // Derive sender address from tx.from
                let sender_addr = citrate_execution::address_utils::normalize_address(&tx.from);
                if sender_addr.0 == addr_bytes {
                    max_nonce = Some(max_nonce.map_or(tx.nonce, |m: u64| m.max(tx.nonce)));
                }
            }
            let pending_nonce = match max_nonce {
                Some(m) if m + 1 > base_nonce => m + 1,
                _ => base_nonce,
            };
            return Ok(Value::String(format!("0x{:x}", pending_nonce)));
        }

        Ok(Value::String(format!("0x{:x}", base_nonce)))
    });

    // eth_sendTransaction - Submit transaction (object form)
    let mempool_send_tx = mempool.clone();
    let executor_send_tx = executor.clone();
    io_handler.add_sync_method("eth_sendTransaction", move |params: Params| {
        let api = TransactionApi::new(mempool_send_tx.clone(), executor_send_tx.clone());

        // Expect params: [txObject]
        let params: Vec<Value> = match params.parse() {
            Ok(p) => p,
            Err(e) => return Err(jsonrpc_core::Error::invalid_params(e.to_string())),
        };
        if params.is_empty() {
            return Err(jsonrpc_core::Error::invalid_params(
                "Missing transaction object",
            ));
        }
        let obj = match &params[0] {
            Value::Object(map) => map,
            _ => {
                return Err(jsonrpc_core::Error::invalid_params(
                    "Invalid transaction object",
                ))
            }
        };

        // from (required)
        let from_s = obj
            .get("from")
            .and_then(|v| v.as_str())
            .ok_or_else(|| jsonrpc_core::Error::invalid_params("Missing 'from'"))?;
        let from_hex = from_s.trim().trim_start_matches("0x");
        let from_bytes = hex::decode(from_hex)
            .map_err(|_| jsonrpc_core::Error::invalid_params("Invalid 'from' hex"))?;
        if from_bytes.len() != 20 {
            return Err(jsonrpc_core::Error::invalid_params(
                "'from' must be 20 bytes",
            ));
        }
        let mut from20 = [0u8; 20];
        from20.copy_from_slice(&from_bytes);

        // to (optional)
        let to20_opt = if let Some(to_s) = obj.get("to").and_then(|v| v.as_str()) {
            let to_hex = to_s.trim().trim_start_matches("0x");
            let to_bytes = hex::decode(to_hex)
                .map_err(|_| jsonrpc_core::Error::invalid_params("Invalid 'to' hex"))?;
            if to_bytes.len() != 20 {
                return Err(jsonrpc_core::Error::invalid_params("'to' must be 20 bytes"));
            }
            let mut to20 = [0u8; 20];
            to20.copy_from_slice(&to_bytes);
            Some(to20)
        } else {
            None
        };

        // value (hex string) optional
        let value_u256 = if let Some(vs) = obj.get("value").and_then(|v| v.as_str()) {
            let s = vs.trim();
            let s = s.strip_prefix("0x").unwrap_or(s);
            U256::from_str_radix(s, 16).unwrap_or_else(|_| U256::from(0u64))
        } else {
            U256::from(0u64)
        };

        // gas and gasPrice (hex strings) optional
        let gas = if let Some(gs) = obj.get("gas").and_then(|v| v.as_str()) {
            let s = gs.trim();
            let s = s.strip_prefix("0x").unwrap_or(s);
            u64::from_str_radix(s, 16).unwrap_or(21000)
        } else {
            21000
        };
        let gas_price = if let Some(gps) = obj.get("gasPrice").and_then(|v| v.as_str()) {
            let s = gps.trim();
            let s = s.strip_prefix("0x").unwrap_or(s);
            u64::from_str_radix(s, 16).unwrap_or(1_000_000_000)
        } else {
            1_000_000_000
        };

        // nonce (hex string) optional
        let nonce_opt = if let Some(ns) = obj.get("nonce").and_then(|v| v.as_str()) {
            let s = ns.trim();
            let s = s.strip_prefix("0x").unwrap_or(s);
            Some(u64::from_str_radix(s, 16).unwrap_or(0))
        } else {
            None
        };

        // data (hex string) optional
        let data = if let Some(ds) = obj.get("data").and_then(|v| v.as_str()) {
            let s = ds.trim();
            let s = s.strip_prefix("0x").unwrap_or(s);
            hex::decode(s).unwrap_or_default()
        } else {
            Vec::new()
        };

        // Build TransactionRequest
        let req = crate::types::request::TransactionRequest {
            from: Address(from20),
            to: to20_opt.map(Address),
            value: Some(value_u256),
            gas: Some(gas),
            gas_price: Some(gas_price),
            nonce: nonce_opt,
            data: Some(data),
        };

        match block_on(api.send_transaction(req)) {
            Ok(hash) => Ok(Value::String(format!("0x{}", hex::encode(hash.as_bytes())))),
            Err(e) => Err(jsonrpc_core::Error::invalid_params(e.to_string())),
        }
    });

    // eth_sendRawTransaction - Submit signed transaction
    let mempool_send = mempool.clone();
    io_handler.add_sync_method("eth_sendRawTransaction", move |params: Params| {
        let mempool = mempool_send.clone();

        tracing::info!("eth_sendRawTransaction called");

        let params: Vec<Value> = match params.parse() {
            Ok(p) => p,
            Err(e) => {
                tracing::error!("Failed to parse params: {}", e);
                return Err(jsonrpc_core::Error::invalid_params(e.to_string()));
            }
        };

        if params.is_empty() {
            tracing::error!("Missing transaction data");
            return Err(jsonrpc_core::Error::invalid_params(
                "Missing transaction data",
            ));
        }

        let tx_data = match params[0].as_str() {
            Some(d) if d.starts_with("0x") => &d[2..],
            Some(d) => d,
            None => {
                tracing::error!("Invalid transaction format");
                return Err(jsonrpc_core::Error::invalid_params(
                    "Invalid transaction format",
                ));
            }
        };

        tracing::debug!(
            "Raw tx data (first 100 bytes): {}",
            &tx_data[..tx_data.len().min(200)]
        );

        let tx_bytes = match hex::decode(tx_data) {
            Ok(b) => b,
            Err(e) => {
                tracing::error!("Failed to decode hex: {}", e);
                return Err(jsonrpc_core::Error::invalid_params("Invalid hex data"));
            }
        };

        tracing::debug!("Decoded {} bytes of transaction data", tx_bytes.len());

        // Parse transaction - handles both Ethereum RLP and Citrate bincode formats
        let tx: Transaction = match eth_tx_decoder::decode_eth_transaction(&tx_bytes) {
            Ok(t) => {
                tracing::info!("Successfully decoded transaction");
                t
            }
            Err(e) => {
                tracing::error!("Failed to decode transaction: {}", e);
                return Err(jsonrpc_core::Error::invalid_params(format!(
                    "Failed to parse transaction: {}",
                    e
                )));
            }
        };

        // Get transaction hash (now always properly set by decoder)
        let tx_hash = tx.hash;
        tracing::info!("Transaction hash: 0x{}", hex::encode(tx_hash.as_bytes()));

        // Submit to mempool using block_on to execute async function
        match block_on(mempool.add_transaction(tx, TxClass::Standard)) {
            Ok(_) => {
                tracing::info!(
                    "✓ Transaction {} successfully added to mempool",
                    hex::encode(tx_hash.as_bytes())
                );
                Ok(Value::String(format!(
                    "0x{}",
                    hex::encode(tx_hash.as_bytes())
                )))
            }
            Err(e) => {
                tracing::error!("✗ Failed to submit transaction to mempool: {:?}", e);
                Err(jsonrpc_core::Error::invalid_params(format!(
                    "Failed to submit transaction: {:?}",
                    e
                )))
            }
        }
    });

    // eth_call - Execute call without creating transaction
    let executor_call = executor.clone();
    io_handler.add_sync_method("eth_call", move |params: Params| {
        use citrate_consensus::types::{Block, BlockHeader, PublicKey, Signature, VrfProof};

        let exec = executor_call.clone();

        // Parse params: [callObject, blockTag]
        let params: Vec<Value> = match params.parse() {
            Ok(p) => p,
            Err(e) => return Err(jsonrpc_core::Error::invalid_params(e.to_string())),
        };

        if params.is_empty() {
            return Err(jsonrpc_core::Error::invalid_params("Missing call object"));
        }

        // call object
        let obj = match &params[0] {
            Value::Object(map) => map,
            _ => return Err(jsonrpc_core::Error::invalid_params("Invalid call object")),
        };

        // to (required)
        let to_str = match obj.get("to").and_then(|v| v.as_str()) {
            Some(s) => s.trim().trim_start_matches("0x"),
            None => return Err(jsonrpc_core::Error::invalid_params("Missing 'to' address")),
        };
        let to_bytes = match hex::decode(to_str) {
            Ok(b) if b.len() == 20 => b,
            _ => return Err(jsonrpc_core::Error::invalid_params("Invalid 'to' address")),
        };
        let mut to_pk_bytes = [0u8; 32];
        to_pk_bytes[..20].copy_from_slice(&to_bytes);
        let to_pk = Some(citrate_consensus::types::PublicKey::new(to_pk_bytes));

        // from (optional)
        let from_pk = if let Some(from_s) = obj.get("from").and_then(|v| v.as_str()) {
            let fs = from_s.trim().trim_start_matches("0x");
            let fbytes = match hex::decode(fs) {
                Ok(b) if b.len() == 20 => b,
                _ => {
                    return Err(jsonrpc_core::Error::invalid_params(
                        "Invalid 'from' address",
                    ))
                }
            };
            let mut pkb = [0u8; 32];
            pkb[..20].copy_from_slice(&fbytes);
            PublicKey::new(pkb)
        } else {
            PublicKey::new([0u8; 32])
        };

        // data (optional but usually required)
        let data = if let Some(d) = obj.get("data").and_then(|v| v.as_str()) {
            let ds = d.trim();
            let ds = ds.strip_prefix("0x").unwrap_or(ds);
            match hex::decode(ds) {
                Ok(b) => b,
                Err(_) => return Err(jsonrpc_core::Error::invalid_params("Invalid data hex")),
            }
        } else {
            Vec::new()
        };

        // value (optional)
        let value_u128: u128 = if let Some(vs) = obj.get("value").and_then(|v| v.as_str()) {
            let s = vs.trim();
            if let Some(hexs) = s.strip_prefix("0x") {
                u128::from_str_radix(hexs, 16).unwrap_or(0u128)
            } else {
                s.parse::<u128>().unwrap_or(0u128)
            }
        } else {
            0u128
        };

        // gas and gasPrice (optional)
        let gas_limit: u64 = if let Some(gs) = obj.get("gas").and_then(|v| v.as_str()) {
            let s = gs.trim();
            if let Some(hexs) = s.strip_prefix("0x") {
                u64::from_str_radix(hexs, 16).unwrap_or(1_000_000)
            } else {
                s.parse::<u64>().unwrap_or(1_000_000)
            }
        } else {
            1_000_000
        };

        let gas_price: u64 = if let Some(gps) = obj.get("gasPrice").and_then(|v| v.as_str()) {
            let s = gps.trim();
            if let Some(hexs) = s.strip_prefix("0x") {
                u64::from_str_radix(hexs, 16).unwrap_or(1)
            } else {
                s.parse::<u64>().unwrap_or(1)
            }
        } else {
            1
        };

        // Build a lightweight block context
        let blk = Block {
            header: BlockHeader {
                version: 1,
                block_hash: citrate_consensus::types::Hash::default(),
                selected_parent_hash: citrate_consensus::types::Hash::default(),
                merge_parent_hashes: vec![],
                timestamp: 0,
                height: 0,
                blue_score: 0,
                blue_work: 0,
                pruning_point: citrate_consensus::types::Hash::default(),
                proposer_pubkey: PublicKey::new([0u8; 32]),
                vrf_reveal: VrfProof {
                    proof: vec![],
                    output: citrate_consensus::types::Hash::default(),
                },
                base_fee_per_gas: 1_000_000_000, // 1 gwei
                gas_used: 0,
                gas_limit: 30_000_000,
            },
            state_root: citrate_consensus::types::Hash::default(),
            tx_root: citrate_consensus::types::Hash::default(),
            receipt_root: citrate_consensus::types::Hash::default(),
            artifact_root: citrate_consensus::types::Hash::default(),
            ghostdag_params: Default::default(),
            transactions: vec![],
            signature: Signature::new([0u8; 64]),
            embedded_models: vec![],
            required_pins: vec![],
        };

        // Create a pseudo-transaction
        let mut tx = citrate_consensus::types::Transaction {
            hash: citrate_consensus::types::Hash::default(),
            nonce: 0,
            from: from_pk,
            to: to_pk,
            value: value_u128,
            gas_limit,
            gas_price,
            data,
            signature: Signature::new([0u8; 64]),
            tx_type: None,
        };

        // Determine transaction type from data
        tx.determine_type();

        // Snapshot state, execute, then restore
        let snapshot = exec.state_db().snapshot();
        let res = block_on(exec.execute_transaction(&blk, &tx));
        exec.state_db().restore(snapshot);

        match res {
            Ok(receipt) => Ok(Value::String(format!("0x{}", hex::encode(receipt.output)))),
            Err(e) => Err(jsonrpc_core::Error::invalid_params(format!(
                "eth_call failed: {}",
                e
            ))),
        }
    });

    // eth_estimateGas - Estimate gas for transaction by dry-running execution
    let executor_estimate = executor.clone();
    io_handler.add_sync_method("eth_estimateGas", move |params: Params| {
        use citrate_consensus::types::{Block, BlockHeader, PublicKey, Signature, VrfProof};

        let exec = executor_estimate.clone();

        // Parse params: [callObject, blockTag (optional)]
        let params: Vec<Value> = match params.parse() {
            Ok(p) => p,
            Err(_) => {
                // No params - return default gas for simple transfer
                return Ok(Value::String("0x5208".to_string())); // 21000 gas
            }
        };

        if params.is_empty() {
            // No call object - return default gas for simple transfer
            return Ok(Value::String("0x5208".to_string())); // 21000 gas
        }

        // call object
        let obj = match &params[0] {
            Value::Object(map) => map,
            _ => {
                // Invalid params - return default
                return Ok(Value::String("0x5208".to_string()));
            }
        };

        // to (optional for contract deployment)
        let to_pk = if let Some(to_s) = obj.get("to").and_then(|v| v.as_str()) {
            let ts = to_s.trim().trim_start_matches("0x");
            if let Ok(to_bytes) = hex::decode(ts) {
                if to_bytes.len() == 20 {
                    let mut to_pk_bytes = [0u8; 32];
                    to_pk_bytes[..20].copy_from_slice(&to_bytes);
                    Some(PublicKey::new(to_pk_bytes))
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        };

        // from (optional)
        let from_pk = if let Some(from_s) = obj.get("from").and_then(|v| v.as_str()) {
            let fs = from_s.trim().trim_start_matches("0x");
            if let Ok(fbytes) = hex::decode(fs) {
                if fbytes.len() == 20 {
                    let mut pkb = [0u8; 32];
                    pkb[..20].copy_from_slice(&fbytes);
                    PublicKey::new(pkb)
                } else {
                    PublicKey::new([0u8; 32])
                }
            } else {
                PublicKey::new([0u8; 32])
            }
        } else {
            PublicKey::new([0u8; 32])
        };

        // data (optional)
        let data = if let Some(d) = obj.get("data").and_then(|v| v.as_str()) {
            let ds = d.trim().trim_start_matches("0x");
            hex::decode(ds).unwrap_or_default()
        } else {
            Vec::new()
        };

        // value (optional)
        let value_u128: u128 = if let Some(vs) = obj.get("value").and_then(|v| v.as_str()) {
            let s = vs.trim();
            if let Some(hexs) = s.strip_prefix("0x") {
                u128::from_str_radix(hexs, 16).unwrap_or(0)
            } else {
                s.parse::<u128>().unwrap_or(0)
            }
        } else {
            0
        };

        // Use a high gas limit for estimation (will return actual used)
        let gas_limit: u64 = if let Some(gs) = obj.get("gas").and_then(|v| v.as_str()) {
            let s = gs.trim();
            if let Some(hexs) = s.strip_prefix("0x") {
                u64::from_str_radix(hexs, 16).unwrap_or(15_000_000)
            } else {
                s.parse::<u64>().unwrap_or(15_000_000)
            }
        } else {
            15_000_000 // Default to block gas limit for estimation
        };

        // Check if this is a simple transfer (no data, has to address)
        if data.is_empty() && to_pk.is_some() {
            // Simple value transfer - return 21000 gas
            return Ok(Value::String("0x5208".to_string()));
        }

        // Build a lightweight block context for execution
        let blk = Block {
            header: BlockHeader {
                version: 1,
                block_hash: citrate_consensus::types::Hash::default(),
                selected_parent_hash: citrate_consensus::types::Hash::default(),
                merge_parent_hashes: vec![],
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs())
                    .unwrap_or(0),
                height: 0,
                blue_score: 0,
                blue_work: 0,
                pruning_point: citrate_consensus::types::Hash::default(),
                proposer_pubkey: PublicKey::new([0u8; 32]),
                vrf_reveal: VrfProof {
                    proof: vec![],
                    output: citrate_consensus::types::Hash::default(),
                },
                base_fee_per_gas: 1_000_000_000, // 1 gwei
                gas_used: 0,
                gas_limit: 30_000_000,
            },
            state_root: citrate_consensus::types::Hash::default(),
            tx_root: citrate_consensus::types::Hash::default(),
            receipt_root: citrate_consensus::types::Hash::default(),
            artifact_root: citrate_consensus::types::Hash::default(),
            ghostdag_params: Default::default(),
            transactions: vec![],
            signature: Signature::new([0u8; 64]),
            embedded_models: vec![],
            required_pins: vec![],
        };

        // Create a pseudo-transaction for estimation
        let mut tx = citrate_consensus::types::Transaction {
            hash: citrate_consensus::types::Hash::default(),
            nonce: 0,
            from: from_pk,
            to: to_pk,
            value: value_u128,
            gas_limit,
            gas_price: 1, // Minimal gas price for estimation
            data,
            signature: Signature::new([0u8; 64]),
            tx_type: None,
        };

        // Determine transaction type from data
        tx.determine_type();

        // Snapshot state, execute, then restore
        let snapshot = exec.state_db().snapshot();
        let res = block_on(exec.execute_transaction(&blk, &tx));
        exec.state_db().restore(snapshot);

        match res {
            Ok(receipt) => {
                // Return gas used plus 10% buffer for safety margin
                let gas_with_buffer = receipt.gas_used.saturating_add(receipt.gas_used / 10);
                // Minimum 21000 for any transaction
                let final_gas = gas_with_buffer.max(21000);
                Ok(Value::String(format!("0x{:x}", final_gas)))
            }
            Err(_) => {
                // Execution failed - return a higher estimate or error
                // For failed executions, we still return an estimate so users
                // can see what gas would be needed (they may have insufficient balance etc)
                Ok(Value::String("0x7a120".to_string())) // 500000 gas as fallback
            }
        }
    });

    // eth_feeHistory - Get fee history for EIP-1559
    let storage_fee = storage.clone();
    io_handler.add_sync_method("eth_feeHistory", move |params: Params| {
        // Parse params: [blockCount, newestBlock, rewardPercentiles]
        let params: Vec<Value> = match params.parse() {
            Ok(p) => p,
            Err(_) => {
                // Return default if no params
                return Ok(json!({
                    "oldestBlock": "0x1",
                    "reward": [],
                    "baseFeePerGas": ["0x3b9aca00"],
                    "gasUsedRatio": []
                }));
            }
        };

        // Parse block count (default 1)
        let block_count: u64 = params.get(0)
            .and_then(|v| v.as_str())
            .and_then(|s| u64::from_str_radix(s.trim_start_matches("0x"), 16).ok())
            .unwrap_or(1)
            .min(1024); // Cap at 1024 blocks

        // Get current height
        let api = ChainApi::new(storage_fee.clone());
        let current_height = match block_on(api.get_height()) {
            Ok(h) => h,
            Err(_) => 0,
        };

        if current_height == 0 {
            return Ok(json!({
                "oldestBlock": "0x0",
                "reward": [],
                "baseFeePerGas": ["0x3b9aca00"],
                "gasUsedRatio": []
            }));
        }

        // Calculate start height
        let start_height = current_height.saturating_sub(block_count - 1);

        // Collect fee data from blocks
        let mut base_fees: Vec<String> = Vec::new();
        let mut gas_used_ratios: Vec<f64> = Vec::new();
        let mut rewards: Vec<Vec<String>> = Vec::new();

        // Parse reward percentiles if provided
        let percentiles: Vec<f64> = params.get(2)
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter()
                .filter_map(|p| p.as_f64())
                .collect())
            .unwrap_or_default();

        for height in start_height..=current_height {
            // Get block hash at height
            if let Ok(Some(block_hash)) = storage_fee.blocks.get_block_by_height(height) {
                // Get block data
                if let Ok(Some(block)) = storage_fee.blocks.get_block(&block_hash) {
                    // Use persisted gas_used and gas_limit from block header (EIP-1559 fields)
                    // These are set during block building and represent actual execution results
                    let block_gas_limit = if block.header.gas_limit > 0 {
                        block.header.gas_limit
                    } else {
                        30_000_000 // Default 30M for older blocks without this field
                    };

                    let total_gas_used = if block.header.gas_used > 0 {
                        // Use persisted gas_used from block header (best source)
                        block.header.gas_used
                    } else {
                        // Fallback: calculate from receipts or estimate
                        let mut gas_from_receipts: u64 = 0;
                        let mut has_receipt_data = false;

                        for tx in &block.transactions {
                            if let Ok(Some(receipt)) = storage_fee.transactions.get_receipt(&tx.hash) {
                                gas_from_receipts += receipt.gas_used;
                                has_receipt_data = true;
                            }
                        }

                        if has_receipt_data {
                            gas_from_receipts
                        } else {
                            // Last resort: estimate from transactions
                            block.transactions.iter()
                                .map(|tx| {
                                    if tx.to.is_none() {
                                        tx.gas_limit.min(100_000) // Contract creation
                                    } else if tx.data.is_empty() {
                                        21000 // Simple transfer
                                    } else {
                                        tx.gas_limit.min(50_000) // Contract call
                                    }
                                })
                                .sum()
                        }
                    };

                    let gas_ratio = total_gas_used as f64 / block_gas_limit as f64;
                    gas_used_ratios.push(gas_ratio.min(1.0));

                    // Use persisted base_fee_per_gas from block header (EIP-1559)
                    let base_fee = if block.header.base_fee_per_gas > 0 {
                        // Use the persisted value (best source - set during block building)
                        block.header.base_fee_per_gas
                    } else {
                        // Fallback: calculate from block fullness (older blocks)
                        let target_gas = block_gas_limit / 2;
                        if total_gas_used > target_gas {
                            let delta = total_gas_used - target_gas;
                            1_000_000_000_u64 + (delta as f64 / target_gas as f64 * 125_000_000.0) as u64
                        } else {
                            let delta = target_gas - total_gas_used;
                            1_000_000_000_u64.saturating_sub((delta as f64 / target_gas as f64 * 125_000_000.0) as u64)
                        }.max(1_000_000_000)
                    };
                    base_fees.push(format!("0x{:x}", base_fee));

                    // Calculate reward percentiles from transactions (priority fees)
                    // For legacy transactions, tip = gas_price - base_fee
                    if !percentiles.is_empty() && !block.transactions.is_empty() {
                        let mut tips: Vec<u64> = block.transactions.iter()
                            .map(|tx| tx.gas_price.saturating_sub(base_fee))
                            .collect();
                        tips.sort();

                        let mut block_rewards: Vec<String> = Vec::new();
                        for pct in &percentiles {
                            let idx = ((pct / 100.0) * (tips.len() - 1) as f64).floor() as usize;
                            let tip = tips.get(idx).copied().unwrap_or(0);
                            block_rewards.push(format!("0x{:x}", tip));
                        }
                        rewards.push(block_rewards);
                    } else {
                        rewards.push(vec!["0x0".to_string()]);
                    }
                } else {
                    // Block not found, use defaults
                    base_fees.push("0x3b9aca00".to_string()); // 1 gwei
                    gas_used_ratios.push(0.0);
                    rewards.push(vec!["0x0".to_string()]);
                }
            } else {
                // No block at height, use defaults
                base_fees.push("0x3b9aca00".to_string());
                gas_used_ratios.push(0.0);
                rewards.push(vec!["0x0".to_string()]);
            }
        }

        // Add one more base fee for the next block
        base_fees.push(base_fees.last().cloned().unwrap_or_else(|| "0x3b9aca00".to_string()));

        Ok(json!({
            "oldestBlock": format!("0x{:x}", start_height),
            "reward": rewards,
            "baseFeePerGas": base_fees,
            "gasUsedRatio": gas_used_ratios
        }))
    });

    // eth_maxPriorityFeePerGas - Get max priority fee
    io_handler.add_sync_method("eth_maxPriorityFeePerGas", move |_params: Params| {
        // Return 1 gwei max priority fee
        Ok(Value::String("0x3b9aca00".to_string()))
    });

    // eth_getLogs - Get logs matching filter criteria
    let storage_logs = storage.clone();
    io_handler.add_sync_method("eth_getLogs", move |params: Params| {
        let params: Vec<Value> = match params.parse() {
            Ok(p) => p,
            Err(e) => return Err(jsonrpc_core::Error::invalid_params(e.to_string())),
        };

        if params.is_empty() {
            return Err(jsonrpc_core::Error::invalid_params("Missing filter object"));
        }

        let filter = &params[0];

        // Get current height for "latest" resolution
        let current_height = storage_logs.blocks.get_latest_height().unwrap_or(0);

        // Parse fromBlock (default to 0)
        let from_block = match filter.get("fromBlock").and_then(|v| v.as_str()) {
            Some("latest") | Some("pending") => current_height,
            Some("earliest") => 0,
            Some(hex_str) if hex_str.starts_with("0x") => {
                u64::from_str_radix(&hex_str[2..], 16).unwrap_or(0)
            }
            None => 0,
            _ => 0,
        };

        // Parse toBlock (default to latest)
        let to_block = match filter.get("toBlock").and_then(|v| v.as_str()) {
            Some("latest") | Some("pending") => current_height,
            Some("earliest") => 0,
            Some(hex_str) if hex_str.starts_with("0x") => {
                u64::from_str_radix(&hex_str[2..], 16).unwrap_or(current_height)
            }
            None => current_height,
            _ => current_height,
        };

        // Limit block range to prevent excessive queries
        let max_block_range = 1000u64;
        let effective_to = to_block.min(from_block.saturating_add(max_block_range));

        // Parse address filter (single address or array)
        let address_filter: Vec<Address> = match filter.get("address") {
            Some(Value::String(addr_str)) => {
                let addr_hex = addr_str.trim_start_matches("0x");
                if let Ok(bytes) = hex::decode(addr_hex) {
                    if bytes.len() == 20 {
                        let mut arr = [0u8; 20];
                        arr.copy_from_slice(&bytes);
                        vec![Address(arr)]
                    } else {
                        vec![]
                    }
                } else {
                    vec![]
                }
            }
            Some(Value::Array(addrs)) => {
                addrs
                    .iter()
                    .filter_map(|v| v.as_str())
                    .filter_map(|addr_str| {
                        let addr_hex = addr_str.trim_start_matches("0x");
                        hex::decode(addr_hex).ok().and_then(|bytes| {
                            if bytes.len() == 20 {
                                let mut arr = [0u8; 20];
                                arr.copy_from_slice(&bytes);
                                Some(Address(arr))
                            } else {
                                None
                            }
                        })
                    })
                    .collect()
            }
            _ => vec![],
        };

        // Parse topics filter (array of arrays, each position can be null or array of hashes)
        let topics_filter: Vec<Option<Vec<Hash>>> = match filter.get("topics") {
            Some(Value::Array(topics)) => {
                topics
                    .iter()
                    .map(|topic_entry| {
                        match topic_entry {
                            Value::Null => None, // null means "any"
                            Value::String(hash_str) => {
                                // Single topic hash
                                let hash_hex = hash_str.trim_start_matches("0x");
                                hex::decode(hash_hex).ok().and_then(|bytes| {
                                    if bytes.len() == 32 {
                                        let mut arr = [0u8; 32];
                                        arr.copy_from_slice(&bytes);
                                        Some(vec![Hash::new(arr)])
                                    } else {
                                        None
                                    }
                                })
                            }
                            Value::Array(hashes) => {
                                // Array of topic hashes (OR logic)
                                let parsed: Vec<Hash> = hashes
                                    .iter()
                                    .filter_map(|v| v.as_str())
                                    .filter_map(|hash_str| {
                                        let hash_hex = hash_str.trim_start_matches("0x");
                                        hex::decode(hash_hex).ok().and_then(|bytes| {
                                            if bytes.len() == 32 {
                                                let mut arr = [0u8; 32];
                                                arr.copy_from_slice(&bytes);
                                                Some(Hash::new(arr))
                                            } else {
                                                None
                                            }
                                        })
                                    })
                                    .collect();
                                if parsed.is_empty() {
                                    None
                                } else {
                                    Some(parsed)
                                }
                            }
                            _ => None,
                        }
                    })
                    .collect()
            }
            _ => vec![],
        };

        // Collect matching logs
        let mut result_logs: Vec<Value> = Vec::new();
        let mut log_index_global = 0usize;

        for height in from_block..=effective_to {
            // Get block hash at this height
            let block_hash = match storage_logs.blocks.get_block_by_height(height) {
                Ok(Some(hash)) => hash,
                _ => continue,
            };

            // Get all transaction hashes in this block
            let tx_hashes = match storage_logs.transactions.get_block_transactions(&block_hash) {
                Ok(hashes) => hashes,
                Err(_) => continue,
            };

            for (tx_index, tx_hash) in tx_hashes.iter().enumerate() {
                // Get receipt for this transaction
                let receipt = match storage_logs.transactions.get_receipt(tx_hash) {
                    Ok(Some(r)) => r,
                    _ => continue,
                };

                // Filter and collect logs from this receipt
                for (log_index_in_tx, log) in receipt.logs.iter().enumerate() {
                    // Check address filter
                    if !address_filter.is_empty() && !address_filter.contains(&log.address) {
                        continue;
                    }

                    // Check topics filter
                    let topics_match = topics_filter.iter().enumerate().all(|(i, topic_filter)| {
                        match topic_filter {
                            None => true, // null means any
                            Some(allowed_topics) => {
                                if i >= log.topics.len() {
                                    false // Log doesn't have this topic position
                                } else {
                                    allowed_topics.contains(&log.topics[i])
                                }
                            }
                        }
                    });

                    if !topics_match {
                        continue;
                    }

                    // Log matches all filters
                    result_logs.push(json!({
                        "address": format!("0x{}", hex::encode(log.address.0)),
                        "topics": log.topics.iter()
                            .map(|t| format!("0x{}", hex::encode(t.as_bytes())))
                            .collect::<Vec<_>>(),
                        "data": format!("0x{}", hex::encode(&log.data)),
                        "blockNumber": format!("0x{:x}", height),
                        "blockHash": format!("0x{}", hex::encode(block_hash.as_bytes())),
                        "transactionHash": format!("0x{}", hex::encode(tx_hash.as_bytes())),
                        "transactionIndex": format!("0x{:x}", tx_index),
                        "logIndex": format!("0x{:x}", log_index_global),
                        "removed": false
                    }));

                    log_index_global += 1;
                    let _ = log_index_in_tx; // Suppress unused warning
                }
            }
        }

        Ok(Value::Array(result_logs))
    });

    // eth_newFilter - Create a new log filter
    let storage_new_filter = storage.clone();
    let filter_registry_new = filter_registry.clone();
    io_handler.add_sync_method("eth_newFilter", move |params: Params| {
        let params: Vec<Value> = match params.parse() {
            Ok(p) => p,
            Err(e) => return Err(jsonrpc_core::Error::invalid_params(e.to_string())),
        };

        if params.is_empty() {
            return Err(jsonrpc_core::Error::invalid_params("Missing filter object"));
        }

        let filter = &params[0];
        let current_height = storage_new_filter.blocks.get_latest_height().unwrap_or(0);

        // Parse fromBlock
        let from_block = match filter.get("fromBlock").and_then(|v| v.as_str()) {
            Some("latest") | Some("pending") => Some(current_height),
            Some("earliest") => Some(0),
            Some(hex_str) if hex_str.starts_with("0x") => {
                u64::from_str_radix(&hex_str[2..], 16).ok()
            }
            None => None,
            _ => None,
        };

        // Parse toBlock
        let to_block = match filter.get("toBlock").and_then(|v| v.as_str()) {
            Some("latest") | Some("pending") => Some(current_height),
            Some("earliest") => Some(0),
            Some(hex_str) if hex_str.starts_with("0x") => {
                u64::from_str_radix(&hex_str[2..], 16).ok()
            }
            None => None,
            _ => None,
        };

        // Parse address filter
        let addresses: Vec<Address> = match filter.get("address") {
            Some(Value::String(addr_str)) => {
                let addr_hex = addr_str.trim_start_matches("0x");
                if let Ok(bytes) = hex::decode(addr_hex) {
                    if bytes.len() == 20 {
                        let mut arr = [0u8; 20];
                        arr.copy_from_slice(&bytes);
                        vec![Address(arr)]
                    } else {
                        vec![]
                    }
                } else {
                    vec![]
                }
            }
            Some(Value::Array(addrs)) => {
                addrs
                    .iter()
                    .filter_map(|v| v.as_str())
                    .filter_map(|addr_str| {
                        let addr_hex = addr_str.trim_start_matches("0x");
                        hex::decode(addr_hex).ok().and_then(|bytes| {
                            if bytes.len() == 20 {
                                let mut arr = [0u8; 20];
                                arr.copy_from_slice(&bytes);
                                Some(Address(arr))
                            } else {
                                None
                            }
                        })
                    })
                    .collect()
            }
            _ => vec![],
        };

        // Parse topics filter
        let topics: Vec<Option<Vec<Hash>>> = match filter.get("topics") {
            Some(Value::Array(topics)) => {
                topics
                    .iter()
                    .map(|topic_entry| {
                        match topic_entry {
                            Value::Null => None,
                            Value::String(hash_str) => {
                                let hash_hex = hash_str.trim_start_matches("0x");
                                hex::decode(hash_hex).ok().and_then(|bytes| {
                                    if bytes.len() == 32 {
                                        let mut arr = [0u8; 32];
                                        arr.copy_from_slice(&bytes);
                                        Some(vec![Hash::new(arr)])
                                    } else {
                                        None
                                    }
                                })
                            }
                            Value::Array(hashes) => {
                                let parsed: Vec<Hash> = hashes
                                    .iter()
                                    .filter_map(|v| v.as_str())
                                    .filter_map(|hash_str| {
                                        let hash_hex = hash_str.trim_start_matches("0x");
                                        hex::decode(hash_hex).ok().and_then(|bytes| {
                                            if bytes.len() == 32 {
                                                let mut arr = [0u8; 32];
                                                arr.copy_from_slice(&bytes);
                                                Some(Hash::new(arr))
                                            } else {
                                                None
                                            }
                                        })
                                    })
                                    .collect();
                                if parsed.is_empty() { None } else { Some(parsed) }
                            }
                            _ => None,
                        }
                    })
                    .collect()
            }
            _ => vec![],
        };

        let filter_id = filter_registry_new.new_log_filter(
            from_block,
            to_block,
            addresses,
            topics,
            current_height,
        );

        Ok(Value::String(format!("0x{:x}", filter_id)))
    });

    // eth_newBlockFilter - Create a new block filter
    let storage_block_filter = storage.clone();
    let filter_registry_block = filter_registry.clone();
    io_handler.add_sync_method("eth_newBlockFilter", move |_params: Params| {
        let current_height = storage_block_filter.blocks.get_latest_height().unwrap_or(0);
        let filter_id = filter_registry_block.new_block_filter(current_height);
        Ok(Value::String(format!("0x{:x}", filter_id)))
    });

    // eth_newPendingTransactionFilter - Create a new pending transaction filter
    let filter_registry_pending = filter_registry.clone();
    io_handler.add_sync_method("eth_newPendingTransactionFilter", move |_params: Params| {
        let filter_id = filter_registry_pending.new_pending_transaction_filter();
        Ok(Value::String(format!("0x{:x}", filter_id)))
    });

    // eth_uninstallFilter - Remove a filter
    let filter_registry_uninstall = filter_registry.clone();
    io_handler.add_sync_method("eth_uninstallFilter", move |params: Params| {
        let params: Vec<Value> = match params.parse() {
            Ok(p) => p,
            Err(e) => return Err(jsonrpc_core::Error::invalid_params(e.to_string())),
        };

        if params.is_empty() {
            return Err(jsonrpc_core::Error::invalid_params("Missing filter ID"));
        }

        let filter_id = match params[0].as_str() {
            Some(hex_str) => {
                let hex = hex_str.trim_start_matches("0x");
                u64::from_str_radix(hex, 16).unwrap_or(0)
            }
            None => return Err(jsonrpc_core::Error::invalid_params("Invalid filter ID")),
        };

        let removed = filter_registry_uninstall.uninstall_filter(filter_id);
        Ok(Value::Bool(removed))
    });

    // eth_getFilterChanges - Get changes since last poll
    let storage_filter_changes = storage.clone();
    let filter_registry_changes = filter_registry.clone();
    io_handler.add_sync_method("eth_getFilterChanges", move |params: Params| {
        let params: Vec<Value> = match params.parse() {
            Ok(p) => p,
            Err(e) => return Err(jsonrpc_core::Error::invalid_params(e.to_string())),
        };

        if params.is_empty() {
            return Err(jsonrpc_core::Error::invalid_params("Missing filter ID"));
        }

        let filter_id = match params[0].as_str() {
            Some(hex_str) => {
                let hex = hex_str.trim_start_matches("0x");
                u64::from_str_radix(hex, 16).unwrap_or(0)
            }
            None => return Err(jsonrpc_core::Error::invalid_params("Invalid filter ID")),
        };

        let filter = match filter_registry_changes.get_filter(filter_id) {
            Some(f) => f,
            None => {
                return Err(jsonrpc_core::Error {
                    code: jsonrpc_core::ErrorCode::InvalidRequest,
                    message: "Filter not found".to_string(),
                    data: None,
                })
            }
        };

        let current_height = storage_filter_changes.blocks.get_latest_height().unwrap_or(0);
        let last_poll_block = filter.last_poll_block;

        match filter.filter_type {
            FilterType::Block => {
                // Return new block hashes since last poll
                let mut block_hashes = Vec::new();
                for height in (last_poll_block + 1)..=current_height {
                    if let Ok(Some(hash)) = storage_filter_changes.blocks.get_block_by_height(height) {
                        block_hashes.push(Value::String(format!("0x{}", hex::encode(hash.as_bytes()))));
                    }
                }
                filter_registry_changes.update_last_poll_block(filter_id, current_height);
                Ok(Value::Array(block_hashes))
            }
            FilterType::PendingTransaction => {
                // For pending transactions, we'd need to track which ones are new
                // This is a simplified implementation that returns empty array
                Ok(Value::Array(vec![]))
            }
            FilterType::Log { from_block, to_block, ref addresses, ref topics } => {
                // Calculate effective block range
                let effective_from = last_poll_block + 1;
                let effective_to = match to_block {
                    Some(t) => t.min(current_height),
                    None => current_height,
                };

                // Skip if from_block was set and we haven't reached it yet
                if let Some(fb) = from_block {
                    if effective_from < fb {
                        filter_registry_changes.update_last_poll_block(filter_id, current_height);
                        return Ok(Value::Array(vec![]));
                    }
                }

                let mut result_logs: Vec<Value> = Vec::new();

                for height in effective_from..=effective_to {
                    let block_hash = match storage_filter_changes.blocks.get_block_by_height(height) {
                        Ok(Some(hash)) => hash,
                        _ => continue,
                    };

                    let tx_hashes = match storage_filter_changes.transactions.get_block_transactions(&block_hash) {
                        Ok(hashes) => hashes,
                        Err(_) => continue,
                    };

                    for (tx_index, tx_hash) in tx_hashes.iter().enumerate() {
                        let receipt = match storage_filter_changes.transactions.get_receipt(tx_hash) {
                            Ok(Some(r)) => r,
                            _ => continue,
                        };

                        for (log_index, log) in receipt.logs.iter().enumerate() {
                            // Check address filter
                            if !addresses.is_empty() && !addresses.contains(&log.address) {
                                continue;
                            }

                            // Check topics filter
                            let topics_match = topics.iter().enumerate().all(|(i, topic_filter)| {
                                match topic_filter {
                                    None => true,
                                    Some(allowed_topics) => {
                                        if i >= log.topics.len() {
                                            false
                                        } else {
                                            allowed_topics.contains(&log.topics[i])
                                        }
                                    }
                                }
                            });

                            if !topics_match {
                                continue;
                            }

                            result_logs.push(json!({
                                "address": format!("0x{}", hex::encode(log.address.0)),
                                "topics": log.topics.iter()
                                    .map(|t| format!("0x{}", hex::encode(t.as_bytes())))
                                    .collect::<Vec<_>>(),
                                "data": format!("0x{}", hex::encode(&log.data)),
                                "blockNumber": format!("0x{:x}", height),
                                "blockHash": format!("0x{}", hex::encode(block_hash.as_bytes())),
                                "transactionHash": format!("0x{}", hex::encode(tx_hash.as_bytes())),
                                "transactionIndex": format!("0x{:x}", tx_index),
                                "logIndex": format!("0x{:x}", log_index),
                                "removed": false
                            }));
                        }
                    }
                }

                filter_registry_changes.update_last_poll_block(filter_id, current_height);
                Ok(Value::Array(result_logs))
            }
        }
    });

    // eth_getFilterLogs - Get all logs matching filter (for log filters only)
    let storage_filter_logs = storage.clone();
    let filter_registry_logs = filter_registry.clone();
    io_handler.add_sync_method("eth_getFilterLogs", move |params: Params| {
        let params: Vec<Value> = match params.parse() {
            Ok(p) => p,
            Err(e) => return Err(jsonrpc_core::Error::invalid_params(e.to_string())),
        };

        if params.is_empty() {
            return Err(jsonrpc_core::Error::invalid_params("Missing filter ID"));
        }

        let filter_id = match params[0].as_str() {
            Some(hex_str) => {
                let hex = hex_str.trim_start_matches("0x");
                u64::from_str_radix(hex, 16).unwrap_or(0)
            }
            None => return Err(jsonrpc_core::Error::invalid_params("Invalid filter ID")),
        };

        let filter = match filter_registry_logs.get_filter(filter_id) {
            Some(f) => f,
            None => {
                return Err(jsonrpc_core::Error {
                    code: jsonrpc_core::ErrorCode::InvalidRequest,
                    message: "Filter not found".to_string(),
                    data: None,
                })
            }
        };

        let current_height = storage_filter_logs.blocks.get_latest_height().unwrap_or(0);

        match filter.filter_type {
            FilterType::Log { from_block, to_block, ref addresses, ref topics } => {
                let effective_from = from_block.unwrap_or(0);
                let effective_to = to_block.unwrap_or(current_height).min(current_height);
                let max_range = 1000u64;
                let effective_to = effective_to.min(effective_from.saturating_add(max_range));

                let mut result_logs: Vec<Value> = Vec::new();

                for height in effective_from..=effective_to {
                    let block_hash = match storage_filter_logs.blocks.get_block_by_height(height) {
                        Ok(Some(hash)) => hash,
                        _ => continue,
                    };

                    let tx_hashes = match storage_filter_logs.transactions.get_block_transactions(&block_hash) {
                        Ok(hashes) => hashes,
                        Err(_) => continue,
                    };

                    for (tx_index, tx_hash) in tx_hashes.iter().enumerate() {
                        let receipt = match storage_filter_logs.transactions.get_receipt(tx_hash) {
                            Ok(Some(r)) => r,
                            _ => continue,
                        };

                        for (log_index, log) in receipt.logs.iter().enumerate() {
                            if !addresses.is_empty() && !addresses.contains(&log.address) {
                                continue;
                            }

                            let topics_match = topics.iter().enumerate().all(|(i, topic_filter)| {
                                match topic_filter {
                                    None => true,
                                    Some(allowed_topics) => {
                                        if i >= log.topics.len() {
                                            false
                                        } else {
                                            allowed_topics.contains(&log.topics[i])
                                        }
                                    }
                                }
                            });

                            if !topics_match {
                                continue;
                            }

                            result_logs.push(json!({
                                "address": format!("0x{}", hex::encode(log.address.0)),
                                "topics": log.topics.iter()
                                    .map(|t| format!("0x{}", hex::encode(t.as_bytes())))
                                    .collect::<Vec<_>>(),
                                "data": format!("0x{}", hex::encode(&log.data)),
                                "blockNumber": format!("0x{:x}", height),
                                "blockHash": format!("0x{}", hex::encode(block_hash.as_bytes())),
                                "transactionHash": format!("0x{}", hex::encode(tx_hash.as_bytes())),
                                "transactionIndex": format!("0x{:x}", tx_index),
                                "logIndex": format!("0x{:x}", log_index),
                                "removed": false
                            }));
                        }
                    }
                }

                Ok(Value::Array(result_logs))
            }
            _ => {
                Err(jsonrpc_core::Error {
                    code: jsonrpc_core::ErrorCode::InvalidRequest,
                    message: "eth_getFilterLogs only works with log filters".to_string(),
                    data: None,
                })
            }
        }
    });

    // citrate_getMempoolSnapshot - Get all pending transactions in mempool
    let mempool_snapshot = mempool.clone();
    io_handler.add_sync_method("citrate_getMempoolSnapshot", move |_params: Params| {
        let mp = mempool_snapshot.clone();

        // Get mempool stats to know how many transactions to fetch
        let stats = block_on(mp.stats());
        let total = stats.total_transactions;

        // Get all transactions from mempool
        let txs = block_on(mp.get_transactions(total));

        // Convert to JSON format
        let mut tx_list = Vec::new();
        for tx in txs {
            let mut tx_obj = serde_json::Map::new();
            tx_obj.insert(
                "hash".to_string(),
                Value::String(format!("0x{}", hex::encode(tx.hash.as_bytes()))),
            );

            // Convert from address
            let from_addr = citrate_execution::address_utils::normalize_address(&tx.from);
            tx_obj.insert(
                "from".to_string(),
                Value::String(format!("0x{}", hex::encode(from_addr.0))),
            );

            // Convert to address
            if let Some(to_pk) = tx.to {
                let to_addr = citrate_execution::address_utils::normalize_address(&to_pk);
                tx_obj.insert(
                    "to".to_string(),
                    Value::String(format!("0x{}", hex::encode(to_addr.0))),
                );
            } else {
                tx_obj.insert("to".to_string(), Value::Null);
            }

            tx_obj.insert(
                "value".to_string(),
                Value::String(format!("0x{:x}", tx.value)),
            );
            tx_obj.insert(
                "nonce".to_string(),
                Value::String(format!("0x{:x}", tx.nonce)),
            );
            tx_obj.insert(
                "gasPrice".to_string(),
                Value::String(format!("0x{:x}", tx.gas_price)),
            );
            tx_obj.insert(
                "gasLimit".to_string(),
                Value::String(format!("0x{:x}", tx.gas_limit)),
            );
            tx_obj.insert("dataSize".to_string(), Value::Number(tx.data.len().into()));

            tx_list.push(Value::Object(tx_obj));
        }

        // Return mempool info
        let mut result = serde_json::Map::new();
        result.insert("pending".to_string(), Value::Array(tx_list));
        result.insert("totalTransactions".to_string(), Value::Number(total.into()));
        result.insert(
            "totalBytes".to_string(),
            Value::Number(stats.total_size.into()),
        );

        Ok(Value::Object(result))
    });

    // citrate_getTransactionStatus - Check if transaction is in mempool or mined
    let mempool_status = mempool.clone();
    let storage_status = storage.clone();
    io_handler.add_sync_method("citrate_getTransactionStatus", move |params: Params| {
        let params: Vec<Value> = match params.parse() {
            Ok(p) => p,
            Err(e) => return Err(jsonrpc_core::Error::invalid_params(e.to_string())),
        };

        if params.is_empty() {
            return Err(jsonrpc_core::Error::invalid_params("Missing transaction hash"));
        }

        let tx_hash_str = match params[0].as_str() {
            Some(s) => s,
            None => return Err(jsonrpc_core::Error::invalid_params("Invalid transaction hash")),
        };

        // Parse transaction hash
        let tx_hash_hex = tx_hash_str.trim_start_matches("0x");
        let tx_hash_bytes = match hex::decode(tx_hash_hex) {
            Ok(b) => b,
            Err(_) => return Err(jsonrpc_core::Error::invalid_params("Invalid hex format")),
        };

        if tx_hash_bytes.len() != 32 {
            return Err(jsonrpc_core::Error::invalid_params("Transaction hash must be 32 bytes"));
        }

        let mut hash_array = [0u8; 32];
        hash_array.copy_from_slice(&tx_hash_bytes);
        let tx_hash = Hash::new(hash_array);

        let mp = mempool_status.clone();
        let storage = storage_status.clone();

        // Check if transaction is mined
        if let Ok(Some(_receipt)) = storage.transactions.get_receipt(&tx_hash) {
            return Ok(json!({
                "status": "mined",
                "location": "blockchain",
                "hash": tx_hash_str
            }));
        }

        // Check if transaction is in mempool
        if let Some(tx) = block_on(mp.get_transaction(&tx_hash)) {
            let stats = block_on(mp.stats());
            return Ok(json!({
                "status": "pending",
                "location": "mempool",
                "hash": tx_hash_str,
                "nonce": tx.nonce,
                "from": format!("0x{}", hex::encode(citrate_execution::address_utils::normalize_address(&tx.from).0)),
                "gasPrice": format!("0x{:x}", tx.gas_price),
                "mempoolSize": stats.total_transactions
            }));
        }

        // Transaction not found
        Ok(json!({
            "status": "not_found",
            "location": null,
            "hash": tx_hash_str
        }))
    });
}
