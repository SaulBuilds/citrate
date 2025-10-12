use crate::eth_tx_decoder;
use crate::methods::{ChainApi, StateApi, TransactionApi};
use futures::executor::block_on;
use hex;
use jsonrpc_core::{IoHandler, Params, Value};
use lattice_consensus::types::{Hash, Transaction};
use lattice_execution::executor::Executor;
use lattice_execution::types::Address;
use lattice_sequencer::mempool::{Mempool, TxClass};
use lattice_storage::StorageManager;
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
                    let from_addr = lattice_execution::types::Address::from_public_key(&tx.from);
                    let to_addr_opt = tx.to.as_ref().map(lattice_execution::types::Address::from_public_key);
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
                let sender_addr = Address::from_public_key(&tx.from);
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

        // Parse transaction - handles both Ethereum RLP and Lattice bincode formats
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
        use lattice_consensus::types::{Block, BlockHeader, PublicKey, Signature, VrfProof};

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
        let to_pk = Some(lattice_consensus::types::PublicKey::new(to_pk_bytes));

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
                block_hash: lattice_consensus::types::Hash::default(),
                selected_parent_hash: lattice_consensus::types::Hash::default(),
                merge_parent_hashes: vec![],
                timestamp: 0,
                height: 0,
                blue_score: 0,
                blue_work: 0,
                pruning_point: lattice_consensus::types::Hash::default(),
                proposer_pubkey: PublicKey::new([0u8; 32]),
                vrf_reveal: VrfProof {
                    proof: vec![],
                    output: lattice_consensus::types::Hash::default(),
                },
            },
            state_root: lattice_consensus::types::Hash::default(),
            tx_root: lattice_consensus::types::Hash::default(),
            receipt_root: lattice_consensus::types::Hash::default(),
            artifact_root: lattice_consensus::types::Hash::default(),
            ghostdag_params: Default::default(),
            transactions: vec![],
            signature: Signature::new([0u8; 64]),
        };

        // Create a pseudo-transaction
        let mut tx = lattice_consensus::types::Transaction {
            hash: lattice_consensus::types::Hash::default(),
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

    // eth_estimateGas - Estimate gas for transaction
    io_handler.add_sync_method("eth_estimateGas", move |_params: Params| {
        // Return default gas estimate
        Ok(Value::String("0x5208".to_string())) // 21000 gas
    });

    // eth_feeHistory - Get fee history for EIP-1559
    io_handler.add_sync_method("eth_feeHistory", move |_params: Params| {
        // Return mock fee history for EIP-1559 support
        Ok(json!({
            "oldestBlock": "0x1",
            "reward": [["0x3b9aca00"]],  // 1 gwei
            "baseFeePerGas": ["0x3b9aca00", "0x3b9aca00"], // 1 gwei base fee
            "gasUsedRatio": [0.5]
        }))
    });

    // eth_maxPriorityFeePerGas - Get max priority fee
    io_handler.add_sync_method("eth_maxPriorityFeePerGas", move |_params: Params| {
        // Return 1 gwei max priority fee
        Ok(Value::String("0x3b9aca00".to_string()))
    });

    // lattice_getMempoolSnapshot - Get all pending transactions in mempool
    let mempool_snapshot = mempool.clone();
    io_handler.add_sync_method("lattice_getMempoolSnapshot", move |_params: Params| {
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
            let from_addr = lattice_execution::types::Address::from_public_key(&tx.from);
            tx_obj.insert(
                "from".to_string(),
                Value::String(format!("0x{}", hex::encode(from_addr.0))),
            );

            // Convert to address
            if let Some(to_pk) = tx.to {
                let to_addr = lattice_execution::types::Address::from_public_key(&to_pk);
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
}
