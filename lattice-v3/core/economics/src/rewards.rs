use primitive_types::U256;
use serde::{Deserialize, Serialize};
use lattice_execution::types::Address;
use lattice_consensus::types::Block;
use crate::token::DECIMALS;

/// Block reward configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RewardConfig {
    /// Base block reward in LATT
    pub block_reward: u64,
    
    /// Halving interval (number of blocks)
    pub halving_interval: u64,
    
    /// Inference bonus per inference in block (in LATT)
    pub inference_bonus: u64,
    
    /// Model deployment bonus (in LATT)
    pub model_deployment_bonus: u64,
    
    /// Treasury allocation percentage (0-100)
    pub treasury_percentage: u8,
    
    /// Treasury address
    pub treasury_address: Address,
}

impl Default for RewardConfig {
    fn default() -> Self {
        Self {
            block_reward: 10,  // 10 LATT per block
            halving_interval: 2_100_000,  // ~4 years at 2s blocks
            inference_bonus: 0,  // 0.01 LATT per inference
            model_deployment_bonus: 1,  // 1 LATT per model deployment
            treasury_percentage: 10,  // 10% to treasury
            treasury_address: Address([0; 20]),  // Will be set in genesis
        }
    }
}

/// Block reward calculation
#[derive(Debug, Clone)]
pub struct BlockReward {
    pub validator_reward: U256,
    pub treasury_reward: U256,
    pub total_reward: U256,
}

/// Reward calculator
pub struct RewardCalculator {
    config: RewardConfig,
}

impl RewardCalculator {
    pub fn new(config: RewardConfig) -> Self {
        Self { config }
    }
    
    /// Calculate block reward for a given block
    pub fn calculate_reward(&self, block: &Block) -> BlockReward {
        // Calculate base reward with halving
        let halvings = block.header.height / self.config.halving_interval;
        let base_reward = if halvings >= 64 {
            0  // No more rewards after 64 halvings
        } else {
            self.config.block_reward >> halvings  // Divide by 2^halvings
        };
        
        // Convert to wei
        let mut total_reward = U256::from(base_reward) * U256::from(10).pow(U256::from(DECIMALS));
        
        // Add inference bonuses
        let inference_count = self.count_inferences(block);
        if inference_count > 0 {
            let inference_reward = U256::from(self.config.inference_bonus) 
                * U256::from(inference_count)
                * U256::from(10).pow(U256::from(DECIMALS - 2));  // 0.01 LATT units
            total_reward += inference_reward;
        }
        
        // Add model deployment bonus
        if self.has_model_deployment(block) {
            let model_reward = U256::from(self.config.model_deployment_bonus) 
                * U256::from(10).pow(U256::from(DECIMALS));
            total_reward += model_reward;
        }
        
        // Calculate treasury allocation
        let treasury_reward = total_reward * U256::from(self.config.treasury_percentage) / U256::from(100);
        let validator_reward = total_reward - treasury_reward;
        
        BlockReward {
            validator_reward,
            treasury_reward,
            total_reward,
        }
    }
    
    /// Count inference transactions in block
    fn count_inferences(&self, block: &Block) -> u64 {
        // Heuristic based on execution encoding:
        // Inference requests are marked with first 4 data bytes [0x02, 0x00, 0x00, 0x00]
        let mut count = 0u64;
        for tx in &block.transactions {
            if tx.data.len() >= 4 && tx.data[0..4] == [0x02, 0x00, 0x00, 0x00] {
                count += 1;
            }
        }
        count
    }
    
    /// Check if block contains model deployment
    fn has_model_deployment(&self, block: &Block) -> bool {
        // Consider either a contract deployment (tx.to == None) or
        // a model registration call (selector [0x01, 0x00, 0x00, 0x00]) as a deployment event.
        for tx in &block.transactions {
            if tx.to.is_none() {
                return true;
            }
            if tx.data.len() >= 4 && tx.data[0..4] == [0x01, 0x00, 0x00, 0x00] {
                return true;
            }
        }
        false
    }
    
    /// Calculate total supply at a given block height
    pub fn total_supply_at_height(&self, height: u64) -> U256 {
        let mut total = U256::zero();
        let mut current_reward = self.config.block_reward;
        let mut blocks_processed = 0u64;
        
        for halving in 0..64 {
            let blocks_in_period = if halving == 0 {
                self.config.halving_interval.min(height)
            } else {
                let start = halving * self.config.halving_interval;
                let end = ((halving + 1) * self.config.halving_interval).min(height);
                if start >= height {
                    break;
                }
                end - start
            };
            
            let period_reward = U256::from(current_reward) 
                * U256::from(blocks_in_period)
                * U256::from(10).pow(U256::from(DECIMALS));
            total += period_reward;
            
            blocks_processed += blocks_in_period;
            if blocks_processed >= height {
                break;
            }
            
            current_reward /= 2;
            if current_reward == 0 {
                break;
            }
        }
        
        total
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lattice_consensus::types::{BlockHeader, Hash};
    
    #[test]
    fn test_block_reward_calculation() {
        let config = RewardConfig::default();
        let calculator = RewardCalculator::new(config);
        
        // Create a test block at height 0
        let block = Block {
            header: BlockHeader {
                block_hash: Hash::new([0; 32]),
                height: 0,
                selected_parent_hash: Hash::new([0; 32]),
                merge_parent_hashes: vec![],
                timestamp: 0,
                blue_score: 0,
                blue_work: 0,
                pruning_point: 0,
                version: 1,
            },
            transactions: vec![],
            state_root: Hash::new([0; 32]),
            tx_root: Hash::new([0; 32]),
        };
        
        let reward = calculator.calculate_reward(&block);
        
        // 10 LATT = 10 * 10^18 wei
        let expected_total = U256::from(10) * U256::from(10).pow(U256::from(18));
        assert_eq!(reward.total_reward, expected_total);
        
        // 10% to treasury
        let expected_treasury = expected_total / 10;
        assert_eq!(reward.treasury_reward, expected_treasury);
    }
    
    #[test]
    fn test_halving() {
        let config = RewardConfig::default();
        let calculator = RewardCalculator::new(config);
        
        // Test block after first halving
        let block = Block {
            header: BlockHeader {
                block_hash: Hash::new([0; 32]),
                height: 2_100_000,
                selected_parent_hash: Hash::new([0; 32]),
                merge_parent_hashes: vec![],
                timestamp: 0,
                blue_score: 0,
                blue_work: 0,
                pruning_point: 0,
                version: 1,
            },
            transactions: vec![],
            state_root: Hash::new([0; 32]),
            tx_root: Hash::new([0; 32]),
        };
        
        let reward = calculator.calculate_reward(&block);
        
        // After halving: 5 LATT = 5 * 10^18 wei
        let expected_total = U256::from(5) * U256::from(10).pow(U256::from(18));
        assert_eq!(reward.total_reward, expected_total);
    }
}
