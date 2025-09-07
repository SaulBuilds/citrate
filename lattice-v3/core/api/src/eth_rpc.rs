use crate::methods::{ChainApi, StateApi, TransactionApi};
use crate::types::error::ApiError;
use crate::eth_tx_decoder;
use jsonrpc_core::{IoHandler, Params, Value};
use lattice_storage::StorageManager;
use lattice_sequencer::mempool::{Mempool, TxClass};
use lattice_execution::executor::Executor;
use lattice_execution::types::Address;
use primitive_types::U256;
use lattice_consensus::types::{Hash, Transaction};
use std::sync::Arc;
use futures::executor::block_on;
use serde_json::json;
use hex;
use bincode;
use sha3::{Digest, Keccak256};

/// Add Ethereum-compatible RPC methods to the IoHandler
pub fn register_eth_methods(
    io_handler: &mut IoHandler,
    storage: Arc<StorageManager>,
    mempool: Arc<Mempool>,
    executor: Arc<Executor>,
) {
    // eth_blockNumber - Returns the latest block number
    let storage_bn = storage.clone();
    io_handler.add_sync_method("eth_blockNumber", move |_params: Params| {
        let api = ChainApi::new(storage_bn.clone());
        match block_on(api.get_height()) {
            Ok(height) => {
                // Return as hex string as per Ethereum JSON-RPC spec
                Ok(Value::String(format!("0x{:x}", height)))
            },
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
                    "transactions": block.transactions.iter()
                        .map(|tx| format!("0x{}", hex::encode(tx.hash.as_bytes())))
                        .collect::<Vec<_>>(),
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
                Ok(json!({
                    "number": format!("0x{:x}", block.height),
                    "hash": format!("0x{}", hex::encode(block.hash.as_bytes())),
                    "parentHash": format!("0x{}", hex::encode(block.parent_hash.as_bytes())),
                    "timestamp": format!("0x{:x}", block.timestamp),
                    "gasLimit": "0x1c9c380",
                    "gasUsed": "0x5208",
                    "difficulty": "0x0",
                    "totalDifficulty": "0x0",
                    "transactions": block.transactions.iter()
                        .map(|tx| format!("0x{}", hex::encode(tx.hash.as_bytes())))
                        .collect::<Vec<_>>(),
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
    io_handler.add_sync_method("eth_getTransactionByHash", move |params: Params| {
        let api = TransactionApi::new(storage_tx.clone());
        
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
        
        match block_on(api.get_transaction(Hash::new(hash_bytes))) {
            Ok(tx) => {
                Ok(json!({
                    "hash": format!("0x{}", hex::encode(tx.hash.as_bytes())),
                    "nonce": format!("0x{:x}", tx.nonce),
                    "blockHash": "0x0000000000000000000000000000000000000000000000000000000000000000",
                    "blockNumber": "0x0",
                    "transactionIndex": "0x0",
                    "from": format!("0x{}", tx.from),
                    "to": tx.to.as_ref().map(|t| format!("0x{}", t)),
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
            Err(_) => Ok(Value::Null),
        }
    });

    // eth_getTransactionReceipt - Returns transaction receipt
    let storage_rcpt = storage.clone();
    io_handler.add_sync_method("eth_getTransactionReceipt", move |params: Params| {
        let api = TransactionApi::new(storage_rcpt.clone());
        
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
                Ok(json!({
                    "transactionHash": format!("0x{}", hex::encode(receipt.tx_hash.as_bytes())),
                    "transactionIndex": "0x0",
                    "blockHash": format!("0x{}", hex::encode(receipt.block_hash.as_bytes())),
                    "blockNumber": format!("0x{:x}", receipt.block_number),
                    "from": format!("0x{}", hex::encode(&receipt.from.0)),
                    "to": receipt.to.as_ref().map(|t| format!("0x{}", hex::encode(&t.0))),
                    "cumulativeGasUsed": format!("0x{:x}", receipt.gas_used),
                    "gasUsed": format!("0x{:x}", receipt.gas_used),
                    "contractAddress": receipt.contract_address.as_ref()
                        .map(|a| format!("0x{}", hex::encode(&a.0))),
                    "logs": receipt.logs.iter().map(|log| json!({
                        "address": format!("0x{}", hex::encode(&log.address.0)),
                        "topics": log.topics.iter()
                            .map(|t| format!("0x{}", hex::encode(t)))
                            .collect::<Vec<_>>(),
                        "data": format!("0x{}", hex::encode(&log.data)),
                        "logIndex": format!("0x{:x}", log.index),
                        "transactionIndex": format!("0x{:x}", receipt.tx_index),
                        "transactionHash": format!("0x{}", hex::encode(&receipt.tx_hash)),
                        "blockHash": format!("0x{}", hex::encode(&receipt.block_hash)),
                        "blockNumber": format!("0x{:x}", receipt.block_number),
                        "removed": false
                    })).collect::<Vec<_>>(),
                    "status": if receipt.success { "0x1" } else { "0x0" },
                    "logsBloom": "0x00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000",
                    "type": "0x0",
                    "effectiveGasPrice": format!("0x{:x}", receipt.gas_price)
                }))
            },
            Err(_) => Ok(Value::Null),
        }
    });

    // eth_chainId - Returns the chain ID
    io_handler.add_sync_method("eth_chainId", move |_params: Params| {
        // Return testnet chain ID
        Ok(Value::String("0x539".to_string())) // 1337 in hex
    });

    // eth_syncing - Returns sync status
    io_handler.add_sync_method("eth_syncing", move |_params: Params| {
        // Return false when fully synced
        Ok(Value::Bool(false))
    });

    // net_peerCount - Returns number of peers
    io_handler.add_sync_method("net_peerCount", move |_params: Params| {
        // Return 0 for now (single node testnet)
        Ok(Value::String("0x0".to_string()))
    });

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
            None => return Err(jsonrpc_core::Error::invalid_params("Invalid address format")),
        };
        
        let addr_bytes = match hex::decode(addr_str) {
            Ok(b) if b.len() == 20 => {
                let mut arr = [0u8; 20];
                arr.copy_from_slice(&b);
                arr
            },
            _ => return Err(jsonrpc_core::Error::invalid_params("Invalid address length")),
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
            None => return Err(jsonrpc_core::Error::invalid_params("Invalid address format")),
        };
        
        let addr_bytes = match hex::decode(addr_str) {
            Ok(b) if b.len() == 20 => {
                let mut arr = [0u8; 20];
                arr.copy_from_slice(&b);
                arr
            },
            _ => return Err(jsonrpc_core::Error::invalid_params("Invalid address length")),
        };
        
        match block_on(state_api.get_code(Address(addr_bytes))) {
            Ok(code) => Ok(Value::String(format!("0x{}", hex::encode(code)))),
            Err(_) => Ok(Value::String("0x".to_string())),
        }
    });

    // eth_getTransactionCount - Returns account nonce
    let storage_nonce = storage.clone();
    let executor_nonce = executor.clone();
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
            None => return Err(jsonrpc_core::Error::invalid_params("Invalid address format")),
        };
        
        let addr_bytes = match hex::decode(addr_str) {
            Ok(b) if b.len() == 20 => {
                let mut arr = [0u8; 20];
                arr.copy_from_slice(&b);
                arr
            },
            _ => return Err(jsonrpc_core::Error::invalid_params("Invalid address length")),
        };
        
        match block_on(state_api.get_nonce(Address(addr_bytes))) {
            Ok(nonce) => Ok(Value::String(format!("0x{:x}", nonce))),
            Err(_) => Ok(Value::String("0x0".to_string())),
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
                return Err(jsonrpc_core::Error::invalid_params(e.to_string()))
            },
        };
        
        if params.is_empty() {
            tracing::error!("Missing transaction data");
            return Err(jsonrpc_core::Error::invalid_params("Missing transaction data"));
        }
        
        let tx_data = match params[0].as_str() {
            Some(d) if d.starts_with("0x") => &d[2..],
            Some(d) => d,
            None => {
                tracing::error!("Invalid transaction format");
                return Err(jsonrpc_core::Error::invalid_params("Invalid transaction format"))
            },
        };
        
        tracing::debug!("Raw tx data (first 100 bytes): {}", &tx_data[..tx_data.len().min(200)]);
        
        let tx_bytes = match hex::decode(tx_data) {
            Ok(b) => b,
            Err(e) => {
                tracing::error!("Failed to decode hex: {}", e);
                return Err(jsonrpc_core::Error::invalid_params("Invalid hex data"))
            },
        };
        
        tracing::debug!("Decoded {} bytes of transaction data", tx_bytes.len());
        
        // Parse transaction - handles both Ethereum RLP and Lattice bincode formats
        let tx: Transaction = match eth_tx_decoder::decode_eth_transaction(&tx_bytes) {
            Ok(t) => {
                tracing::info!("Successfully decoded transaction");
                t
            },
            Err(e) => {
                tracing::error!("Failed to decode transaction: {}", e);
                return Err(jsonrpc_core::Error::invalid_params(
                    format!("Failed to parse transaction: {}", e)
                ))
            },
        };
        
        // Get transaction hash (now always properly set by decoder)
        let tx_hash = tx.hash;
        tracing::info!("Transaction hash: 0x{}", hex::encode(tx_hash.as_bytes()));
        
        // Submit to mempool using block_on to execute async function
        match block_on(mempool.add_transaction(tx, TxClass::Standard)) {
            Ok(_) => {
                tracing::info!("✓ Transaction {} successfully added to mempool", hex::encode(tx_hash.as_bytes()));
                Ok(Value::String(format!("0x{}", hex::encode(tx_hash.as_bytes()))))
            },
            Err(e) => {
                tracing::error!("✗ Failed to submit transaction to mempool: {:?}", e);
                Err(jsonrpc_core::Error::invalid_request(
                    format!("Failed to submit transaction: {:?}", e)
                ))
            }
        }
    });

    // eth_call - Execute call without creating transaction
    let executor_call = executor.clone();
    io_handler.add_sync_method("eth_call", move |params: Params| {
        // TODO: Implement eth_call
        Ok(Value::String("0x".to_string()))
    });

    // eth_estimateGas - Estimate gas for transaction
    io_handler.add_sync_method("eth_estimateGas", move |params: Params| {
        // Return default gas estimate
        Ok(Value::String("0x5208".to_string())) // 21000 gas
    });

    // eth_feeHistory - Get fee history for EIP-1559
    io_handler.add_sync_method("eth_feeHistory", move |params: Params| {
        // Return mock fee history for EIP-1559 support
        Ok(json!({
            "oldestBlock": "0x1",
            "reward": [["0x3b9aca00"]],  // 1 gwei
            "baseFeePerGas": ["0x3b9aca00", "0x3b9aca00"], // 1 gwei base fee
            "gasUsedRatio": [0.5]
        }))
    });
    
    // eth_maxPriorityFeePerGas - Get max priority fee
    io_handler.add_sync_method("eth_maxPriorityFeePerGas", move |params: Params| {
        // Return 1 gwei max priority fee
        Ok(Value::String("0x3b9aca00".to_string()))
    });
}