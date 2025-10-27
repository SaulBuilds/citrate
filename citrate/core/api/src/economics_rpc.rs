// citrate/core/api/src/economics_rpc.rs

use futures::executor::block_on;
use jsonrpc_core::{IoHandler, Params, Value};
use citrate_economics::UnifiedEconomicsManager;
use citrate_sequencer::mempool::Mempool;
use serde_json::json;
use std::sync::Arc;

/// Add economics-related RPC methods to the IoHandler
pub fn register_economics_methods(
    io_handler: &mut IoHandler,
    economics_manager: Option<Arc<UnifiedEconomicsManager>>,
    mempool: Option<Arc<Mempool>>,
) {
    // lattice_gasPrice - Returns current dynamic gas price
    let economics_gp = economics_manager.clone();
    io_handler.add_sync_method("citrate_gasPrice", move |_params: Params| {
        if let Some(economics) = &economics_gp {
            // Use the default gas price from config for now
            let config = economics.get_config();
            let gas_price = config.pricing_config.base_gas_price;
            Ok(Value::String(format!("0x{:x}", gas_price)))
        } else {
            // Fallback to 1 Gwei if no economics manager
            Ok(Value::String("0x3b9aca00".to_string())) // 1 Gwei in hex
        }
    });

    // lattice_getEconomicState - Returns current economic metrics
    let economics_es = economics_manager.clone();
    io_handler.add_sync_method("citrate_getEconomicState", move |_params: Params| {
        if let Some(economics) = &economics_es {
            if let Some(state) = economics.get_economic_state() {
                Ok(json!({
                    "blockHeight": state.block_height,
                    "totalSupply": format!("0x{:x}", state.total_supply),
                    "circulatingSupply": format!("0x{:x}", state.circulating_supply),
                    "gasPrice": format!("0x{:x}", state.gas_price),
                    "stakedAmount": format!("0x{:x}", state.staked_amount),
                    "burnAmount": format!("0x{:x}", state.burned_amount),
                    "treasuryBalance": format!("0x{:x}", state.treasury_balance),
                    "governanceParticipation": state.governance_participation,
                    "networkSecurityBudget": format!("0x{:x}", state.network_security_budget),
                    "aiEconomyValue": format!("0x{:x}", state.ai_economy_value),
                }))
            } else {
                Ok(json!({
                    "error": "No economic state available"
                }))
            }
        } else {
            Err(jsonrpc_core::Error::method_not_found())
        }
    });

    // lattice_getVotingPower - Returns voting power for an address
    let economics_vp = economics_manager.clone();
    io_handler.add_sync_method("citrate_getVotingPower", move |params: Params| {
        if let Some(economics) = &economics_vp {
            // Parse address parameter
            let params: Vec<Value> = match params.parse() {
                Ok(p) => p,
                Err(e) => return Err(jsonrpc_core::Error::invalid_params(e.to_string())),
            };

            if params.is_empty() {
                return Err(jsonrpc_core::Error::invalid_params("Missing address parameter"));
            }

            let address_str = match params[0].as_str() {
                Some(s) => s,
                None => return Err(jsonrpc_core::Error::invalid_params("Invalid address format")),
            };

            // Parse hex address
            let address_bytes = match hex::decode(address_str.trim_start_matches("0x")) {
                Ok(bytes) => bytes,
                Err(_) => return Err(jsonrpc_core::Error::invalid_params("Invalid hex address")),
            };

            if address_bytes.len() != 20 {
                return Err(jsonrpc_core::Error::invalid_params("Address must be 20 bytes"));
            }

            let mut addr_array = [0u8; 20];
            addr_array.copy_from_slice(&address_bytes);
            let address = citrate_execution::types::Address(addr_array);

            // Calculate voting power - simplified version
            match economics.calculate_voting_power(address, 0) {
                Ok(voting_power) => Ok(json!({
                    "tokenPower": format!("0x{:x}", voting_power.token_power),
                    "gasUsagePower": format!("0x{:x}", voting_power.gas_usage_power),
                    "stakingPower": format!("0x{:x}", voting_power.staking_power),
                    "reputationPower": format!("0x{:x}", voting_power.reputation_power),
                    "totalPower": format!("0x{:x}", voting_power.total_power),
                    "quadraticPower": format!("0x{:x}", voting_power.quadratic_power),
                })),
                Err(_e) => Err(jsonrpc_core::Error::internal_error()),
            }
        } else {
            Err(jsonrpc_core::Error::method_not_found())
        }
    });

    // lattice_getStakeholderInfo - Returns revenue sharing info for an address
    let economics_si = economics_manager.clone();
    io_handler.add_sync_method("citrate_getStakeholderInfo", move |params: Params| {
        if let Some(economics) = &economics_si {
            // Parse address parameter
            let params: Vec<Value> = match params.parse() {
                Ok(p) => p,
                Err(e) => return Err(jsonrpc_core::Error::invalid_params(e.to_string())),
            };

            if params.is_empty() {
                return Err(jsonrpc_core::Error::invalid_params("Missing address parameter"));
            }

            let address_str = match params[0].as_str() {
                Some(s) => s,
                None => return Err(jsonrpc_core::Error::invalid_params("Invalid address format")),
            };

            // Parse hex address
            let address_bytes = match hex::decode(address_str.trim_start_matches("0x")) {
                Ok(bytes) => bytes,
                Err(_) => return Err(jsonrpc_core::Error::invalid_params("Invalid hex address")),
            };

            if address_bytes.len() != 20 {
                return Err(jsonrpc_core::Error::invalid_params("Address must be 20 bytes"));
            }

            let mut addr_array = [0u8; 20];
            addr_array.copy_from_slice(&address_bytes);
            let address = citrate_execution::types::Address(addr_array);

            if let Some(stakeholder_info) = economics.get_stakeholder_revenue_info(&address) {
                Ok(json!({
                    "stakeholderType": format!("{:?}", stakeholder_info.stakeholder_type),
                    "contributionScore": stakeholder_info.contribution_score,
                    "totalContribution": format!("0x{:x}", stakeholder_info.total_contribution),
                    "blocksActive": stakeholder_info.blocks_active,
                    "qualityScore": stakeholder_info.quality_score,
                }))
            } else {
                Ok(json!({
                    "error": "No stakeholder info found for this address"
                }))
            }
        } else {
            Err(jsonrpc_core::Error::method_not_found())
        }
    });

    // lattice_getRevenueHistory - Returns revenue distribution history
    let economics_rh = economics_manager.clone();
    io_handler.add_sync_method("citrate_getRevenueHistory", move |_params: Params| {
        if let Some(economics) = &economics_rh {
            let distributions = economics.get_revenue_distribution_history(None);
            let history_json: Vec<_> = distributions.iter().map(|dist| {
                json!({
                    "blockHeight": dist.block_height,
                    "totalRevenue": format!("0x{:x}", dist.total_revenue),
                    "poolType": format!("{:?}", dist.pool_type),
                    "distributionCount": dist.distributions.len(),
                    "timestamp": dist.timestamp,
                })
            }).collect();

            Ok(json!(history_json))
        } else {
            Err(jsonrpc_core::Error::method_not_found())
        }
    });

    // lattice_getMempoolSnapshot - Get current mempool status (economics related)
    let mempool_snap = mempool.clone();
    io_handler.add_sync_method("citrate_getMempoolSnapshot", move |_params: Params| {
        if let Some(mempool) = &mempool_snap {
            match block_on(mempool.stats()) {
                stats => {
                    let pending_txs = block_on(mempool.get_transactions(1000)); // Get up to 1000 transactions
                    let mut total_gas_fees = primitive_types::U256::zero();
                    let mut ai_operations = 0u32;
                    let mut total_gas_used = 0u64;
                    let tx_count = pending_txs.len() as u32;

                    // Calculate total fees and identify AI operations
                    for tx in &pending_txs {
                        total_gas_used += tx.gas_limit;

                        // Calculate fee (gas_limit * gas_price)
                        let fee = primitive_types::U256::from(tx.gas_limit) * primitive_types::U256::from(tx.gas_price);
                        total_gas_fees = total_gas_fees + fee;

                        // Check if this is an AI operation (simplified heuristic)
                        if tx.gas_limit > 500_000 {  // AI operations typically use more gas
                            ai_operations += 1;
                        }
                    }

                    let avg_gas_price = if tx_count > 0 {
                        total_gas_used / tx_count as u64
                    } else {
                        1_000_000_000 // 1 Gwei default
                    };

                    Ok(json!({
                        "pendingTransactions": tx_count,
                        "totalGasFees": format!("0x{:x}", total_gas_fees),
                        "avgGasPrice": format!("0x{:x}", avg_gas_price),
                        "aiOperations": ai_operations,
                        "queuedForExecution": tx_count,
                        "mempoolSize": stats.total_size,
                        "byClass": stats.by_class,
                    }))
                }
            }
        } else {
            // Fallback when no mempool available
            Ok(json!({
                "pendingTransactions": 0,
                "totalGasFees": "0x0",
                "avgGasPrice": "0x3b9aca00", // 1 Gwei
                "aiOperations": 0,
                "queuedForExecution": 0
            }))
        }
    });

    // lattice_getToken - Get token information
    let economics_token = economics_manager.clone();
    io_handler.add_sync_method("citrate_getToken", move |_params: Params| {
        if let Some(economics) = &economics_token {
            let token = economics.get_token();
            Ok(json!({
                "name": token.config.name,
                "symbol": token.config.symbol,
                "decimals": token.config.decimals,
                "totalSupply": format!("0x{:x}", token.config.total_supply),
                "totalMinted": format!("0x{:x}", token.total_minted),
            }))
        } else {
            Ok(json!({
                "name": "Citrate",
                "symbol": "LATT",
                "decimals": 18,
                "totalSupply": "0xc9f2c9cd04674edea40000000", // 1B LATT
                "maxSupply": "0xc9f2c9cd04674edea40000000",
            }))
        }
    });

    // lattice_getStakedBalance - Get staked balance for an address
    let economics_sb = economics_manager.clone();
    io_handler.add_sync_method("citrate_getStakedBalance", move |params: Params| {
        if let Some(economics) = &economics_sb {
            // Parse address parameter
            let params: Vec<Value> = match params.parse() {
                Ok(p) => p,
                Err(e) => return Err(jsonrpc_core::Error::invalid_params(e.to_string())),
            };

            if params.is_empty() {
                return Err(jsonrpc_core::Error::invalid_params("Missing address parameter"));
            }

            let address_str = match params[0].as_str() {
                Some(s) => s,
                None => return Err(jsonrpc_core::Error::invalid_params("Invalid address format")),
            };

            // Parse hex address
            let address_bytes = match hex::decode(address_str.trim_start_matches("0x")) {
                Ok(bytes) => bytes,
                Err(_) => return Err(jsonrpc_core::Error::invalid_params("Invalid hex address")),
            };

            if address_bytes.len() != 20 {
                return Err(jsonrpc_core::Error::invalid_params("Address must be 20 bytes"));
            }

            let mut addr_array = [0u8; 20];
            addr_array.copy_from_slice(&address_bytes);
            let address = citrate_execution::types::Address(addr_array);

            let staked_balance = economics.get_staked_balance(&address);
            Ok(Value::String(format!("0x{:x}", staked_balance)))
        } else {
            Ok(Value::String("0x0".to_string()))
        }
    });

    // lattice_getReputationScore - Get reputation score for an address
    let economics_rs = economics_manager.clone();
    io_handler.add_sync_method("citrate_getReputationScore", move |params: Params| {
        if let Some(economics) = &economics_rs {
            // Parse address parameter
            let params: Vec<Value> = match params.parse() {
                Ok(p) => p,
                Err(e) => return Err(jsonrpc_core::Error::invalid_params(e.to_string())),
            };

            if params.is_empty() {
                return Err(jsonrpc_core::Error::invalid_params("Missing address parameter"));
            }

            let address_str = match params[0].as_str() {
                Some(s) => s,
                None => return Err(jsonrpc_core::Error::invalid_params("Invalid address format")),
            };

            // Parse hex address
            let address_bytes = match hex::decode(address_str.trim_start_matches("0x")) {
                Ok(bytes) => bytes,
                Err(_) => return Err(jsonrpc_core::Error::invalid_params("Invalid hex address")),
            };

            if address_bytes.len() != 20 {
                return Err(jsonrpc_core::Error::invalid_params("Address must be 20 bytes"));
            }

            let mut addr_array = [0u8; 20];
            addr_array.copy_from_slice(&address_bytes);
            let address = citrate_execution::types::Address(addr_array);

            let reputation_score = economics.get_reputation_score(&address);
            Ok(Value::Number(serde_json::Number::from_f64(reputation_score).unwrap_or_else(|| serde_json::Number::from_f64(0.5).unwrap())))
        } else {
            Ok(Value::Number(serde_json::Number::from_f64(0.5).unwrap()))
        }
    });
}