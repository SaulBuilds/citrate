// citrate/core/economics/src/token.rs

use citrate_execution::types::Address;
use primitive_types::U256;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Number of decimals for the native token
pub const DECIMALS: u32 = 18;

/// Token configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenConfig {
    pub name: String,
    pub symbol: String,
    pub decimals: u32,
    pub total_supply: U256,
    pub initial_distribution: HashMap<Address, U256>,
}

impl Default for TokenConfig {
    fn default() -> Self {
        Self {
            name: "Citrate".to_string(),
            symbol: "LATT".to_string(),
            decimals: DECIMALS,
            total_supply: U256::from(1_000_000_000) * U256::from(10).pow(U256::from(DECIMALS)),
            initial_distribution: HashMap::new(),
        }
    }
}

/// Native token representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Token {
    pub config: TokenConfig,
    pub balances: HashMap<Address, U256>,
    pub total_minted: U256,
    pub total_burned: U256,
}

impl Token {
    /// Create a new token with the given configuration
    pub fn new(config: TokenConfig) -> Self {
        let mut balances = HashMap::new();
        let mut total_minted = U256::zero();

        // Distribute initial tokens
        for (address, amount) in &config.initial_distribution {
            balances.insert(*address, *amount);
            total_minted += *amount;
        }

        Self {
            config,
            balances,
            total_minted,
            total_burned: U256::zero(),
        }
    }

    /// Get balance of an address
    pub fn balance_of(&self, address: &Address) -> U256 {
        self.balances.get(address).copied().unwrap_or(U256::zero())
    }

    /// Transfer tokens between addresses
    pub fn transfer(
        &mut self,
        from: &Address,
        to: &Address,
        amount: U256,
    ) -> Result<(), TokenError> {
        let from_balance = self.balance_of(from);

        if from_balance < amount {
            return Err(TokenError::InsufficientBalance);
        }

        // Update balances
        self.balances.insert(*from, from_balance - amount);
        let to_balance = self.balance_of(to);
        self.balances.insert(*to, to_balance + amount);

        Ok(())
    }

    /// Mint new tokens (for block rewards)
    pub fn mint(&mut self, to: &Address, amount: U256) -> Result<(), TokenError> {
        let new_total = self.total_minted + amount;

        // Check if minting would exceed total supply
        if new_total > self.config.total_supply {
            return Err(TokenError::ExceedsSupply);
        }

        let balance = self.balance_of(to);
        self.balances.insert(*to, balance + amount);
        self.total_minted = new_total;

        Ok(())
    }

    /// Burn tokens
    pub fn burn(&mut self, from: &Address, amount: U256) -> Result<(), TokenError> {
        let balance = self.balance_of(from);

        if balance < amount {
            return Err(TokenError::InsufficientBalance);
        }

        self.balances.insert(*from, balance - amount);
        self.total_burned += amount;

        Ok(())
    }

    /// Get circulating supply (minted - burned)
    pub fn circulating_supply(&self) -> U256 {
        self.total_minted - self.total_burned
    }
}

#[derive(Debug, thiserror::Error)]
pub enum TokenError {
    #[error("Insufficient balance")]
    InsufficientBalance,

    #[error("Minting would exceed total supply")]
    ExceedsSupply,

    #[error("Invalid amount")]
    InvalidAmount,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_creation() {
        let config = TokenConfig::default();
        let token = Token::new(config);

        assert_eq!(token.total_minted, U256::zero());
        assert_eq!(token.total_burned, U256::zero());
    }

    #[test]
    fn test_mint_and_transfer() {
        let config = TokenConfig::default();
        let mut token = Token::new(config);

        let alice = Address([1; 20]);
        let bob = Address([2; 20]);
        let amount = U256::from(100) * U256::from(10).pow(U256::from(DECIMALS));

        // Mint to Alice
        token.mint(&alice, amount).unwrap();
        assert_eq!(token.balance_of(&alice), amount);

        // Transfer from Alice to Bob
        let transfer_amount = amount / 2;
        token.transfer(&alice, &bob, transfer_amount).unwrap();

        assert_eq!(token.balance_of(&alice), amount - transfer_amount);
        assert_eq!(token.balance_of(&bob), transfer_amount);
    }
}
