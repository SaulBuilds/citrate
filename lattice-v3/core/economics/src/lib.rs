pub mod token;
pub mod rewards;
pub mod genesis;

pub use token::{Token, TokenConfig, DECIMALS};
pub use rewards::{BlockReward, RewardCalculator, RewardConfig};
pub use genesis::{GenesisConfig, GenesisAccount};

use primitive_types::U256;

/// Native token symbol
pub const TOKEN_SYMBOL: &str = "LATT";

/// Native token name
pub const TOKEN_NAME: &str = "Lattice";

/// Total supply: 1 billion LATT
pub const TOTAL_SUPPLY: u128 = 1_000_000_000;

/// Convert LATT amount to wei (smallest unit)
pub fn latt_to_wei(latt: u64) -> U256 {
    U256::from(latt) * U256::from(10).pow(U256::from(DECIMALS))
}

/// Convert wei to LATT
pub fn wei_to_latt(wei: U256) -> u64 {
    (wei / U256::from(10).pow(U256::from(DECIMALS))).as_u64()
}
