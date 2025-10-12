// lattice-v3/core/economics/src/genesis.rs

use crate::latt_to_wei;
use crate::token::DECIMALS;
use lattice_execution::types::Address;
use primitive_types::U256;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Genesis account configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenesisAccount {
    pub address: Address,
    pub balance: U256,
    pub nonce: u64,
    pub code: Option<Vec<u8>>,
}

/// Genesis configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenesisConfig {
    /// Chain ID
    pub chain_id: u64,

    /// Initial accounts
    pub accounts: Vec<GenesisAccount>,

    /// Treasury address
    pub treasury_address: Address,

    /// Team addresses and allocations
    pub team_allocations: HashMap<Address, U256>,

    /// Ecosystem fund address
    pub ecosystem_fund: Address,

    /// Mining rewards pool (not pre-allocated, minted as needed)
    pub mining_pool_max: U256,
}

impl Default for GenesisConfig {
    fn default() -> Self {
        // Default addresses for testnet
        let treasury = Address([0x11; 20]);
        let ecosystem = Address([0x22; 20]);
        let faucet = Address([0x33; 20]);

        // Test accounts with initial balances
        let test_accounts = vec![
            // Faucet account (10 million LATT for testnet distribution)
            GenesisAccount {
                address: faucet,
                balance: latt_to_wei(10_000_000),
                nonce: 0,
                code: None,
            },
            // Treasury (100 million LATT)
            GenesisAccount {
                address: treasury,
                balance: latt_to_wei(100_000_000),
                nonce: 0,
                code: None,
            },
            // Ecosystem fund (250 million LATT)
            GenesisAccount {
                address: ecosystem,
                balance: latt_to_wei(250_000_000),
                nonce: 0,
                code: None,
            },
            // Test account 1
            GenesisAccount {
                address: Address([0x01; 20]),
                balance: latt_to_wei(1000),
                nonce: 0,
                code: None,
            },
            // Test account 2
            GenesisAccount {
                address: Address([0x02; 20]),
                balance: latt_to_wei(1000),
                nonce: 0,
                code: None,
            },
        ];

        Self {
            chain_id: 1337, // Local testnet
            accounts: test_accounts,
            treasury_address: treasury,
            team_allocations: HashMap::new(), // No team allocations for testnet
            ecosystem_fund: ecosystem,
            mining_pool_max: latt_to_wei(500_000_000), // 500M LATT for mining
        }
    }
}

impl GenesisConfig {
    /// Create mainnet genesis configuration
    pub fn mainnet() -> Self {
        let treasury = address_from_hex("0x1111111111111111111111111111111111111111").unwrap();
        let ecosystem = address_from_hex("0x2222222222222222222222222222222222222222").unwrap();

        // Team allocations (15% = 150M LATT, vested over 4 years)
        let team_allocations = HashMap::new();
        // Add team member addresses and allocations here

        Self {
            chain_id: 1, // Mainnet
            accounts: vec![
                // Treasury
                GenesisAccount {
                    address: treasury,
                    balance: latt_to_wei(100_000_000),
                    nonce: 0,
                    code: None,
                },
                // Ecosystem fund
                GenesisAccount {
                    address: ecosystem,
                    balance: latt_to_wei(250_000_000),
                    nonce: 0,
                    code: None,
                },
            ],
            treasury_address: treasury,
            team_allocations,
            ecosystem_fund: ecosystem,
            mining_pool_max: latt_to_wei(500_000_000),
        }
    }

    /// Get total pre-allocated supply
    pub fn total_preallocation(&self) -> U256 {
        let mut total = U256::zero();

        for account in &self.accounts {
            total += account.balance;
        }

        for balance in self.team_allocations.values() {
            total += *balance;
        }

        total
    }

    /// Validate genesis configuration
    pub fn validate(&self) -> Result<(), GenesisError> {
        // Check total allocation doesn't exceed supply
        let total_supply = U256::from(1_000_000_000) * U256::from(10).pow(U256::from(DECIMALS));
        let preallocated = self.total_preallocation();

        if preallocated + self.mining_pool_max > total_supply {
            return Err(GenesisError::ExceedsSupply);
        }

        // Check for duplicate addresses
        let mut addresses = std::collections::HashSet::new();
        for account in &self.accounts {
            if !addresses.insert(account.address) {
                return Err(GenesisError::DuplicateAddress(account.address));
            }
        }

        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum GenesisError {
    #[error("Total allocation exceeds maximum supply")]
    ExceedsSupply,

    #[error("Duplicate address in genesis: {0:?}")]
    DuplicateAddress(Address),

    #[error("Invalid configuration: {0}")]
    Invalid(String),
}

// Helper function to create Address from hex string
fn address_from_hex(hex: &str) -> Result<Address, hex::FromHexError> {
    let bytes = hex::decode(hex.trim_start_matches("0x"))?;
    if bytes.len() != 20 {
        return Err(hex::FromHexError::InvalidStringLength);
    }
    let mut addr = [0u8; 20];
    addr.copy_from_slice(&bytes);
    Ok(Address(addr))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_genesis_config_validation() {
        let config = GenesisConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_total_preallocation() {
        let config = GenesisConfig::default();
        let total = config.total_preallocation();

        // Should be 360M LATT (10M faucet + 100M treasury + 250M ecosystem + 2K test accounts)
        let expected = latt_to_wei(360_002_000);
        assert_eq!(total, expected);
    }
}
