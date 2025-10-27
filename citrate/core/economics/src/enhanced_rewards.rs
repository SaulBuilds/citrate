// citrate/core/economics/src/enhanced_rewards.rs

use crate::dynamic_pricing::UtilizationMetrics;
use citrate_execution::types::Address;
use primitive_types::U256;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use anyhow::Result;

/// Enhanced reward configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedRewardConfig {
    /// Base block reward in LATT
    pub base_block_reward: U256,

    /// Validator performance bonus pool (% of base reward)
    pub performance_bonus_pool: u8,

    /// AI contribution reward pool (% of base reward)
    pub ai_contribution_pool: u8,

    /// Network health bonus pool (% of base reward)
    pub network_health_pool: u8,

    /// Long-term staking bonus pool (% of base reward)
    pub staking_bonus_pool: u8,

    /// Minimum stake for validator rewards
    pub min_validator_stake: U256,

    /// Performance window in blocks
    pub performance_window: u64,

    /// AI quality threshold for rewards
    pub ai_quality_threshold: f64,

    /// Network participation threshold
    pub participation_threshold: f64,
}

impl Default for EnhancedRewardConfig {
    fn default() -> Self {
        Self {
            base_block_reward: U256::from(10) * U256::from(10).pow(U256::from(18)), // 10 LATT
            performance_bonus_pool: 30, // 30% for performance
            ai_contribution_pool: 25,   // 25% for AI contributions
            network_health_pool: 20,    // 20% for network health
            staking_bonus_pool: 25,     // 25% for long-term staking
            min_validator_stake: U256::from(32_000) * U256::from(10).pow(U256::from(18)), // 32k LATT
            performance_window: 100,    // 100 blocks
            ai_quality_threshold: 0.85, // 85% quality threshold
            participation_threshold: 0.9, // 90% participation
        }
    }
}

/// Validator performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorPerformance {
    pub address: Address,
    pub blocks_proposed: u64,
    pub blocks_validated: u64,
    pub uptime_percentage: f64,
    pub transaction_throughput: u64,
    pub ai_operations_processed: u64,
    pub consensus_participation: f64,
    pub stake_amount: U256,
    pub stake_duration: u64, // blocks
    pub slash_count: u64,
    pub quality_score: f64,
}

/// AI contribution metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIContribution {
    pub contributor: Address,
    pub models_deployed: u64,
    pub inferences_served: u64,
    pub quality_ratings: Vec<f64>,
    pub compute_provided: u64, // compute units
    pub data_contributions: u64, // datasets shared
    pub successful_trainings: u64,
    pub peer_reviews_given: u64,
    pub community_reputation: f64,
}

/// Network health metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkHealth {
    pub total_validators: u64,
    pub active_validators: u64,
    pub average_uptime: f64,
    pub consensus_efficiency: f64,
    pub transaction_success_rate: f64,
    pub ai_operation_success_rate: f64,
    pub network_decentralization: f64, // Nakamoto coefficient normalized
    pub security_incidents: u64,
}

/// Enhanced reward calculation result
#[derive(Debug, Clone)]
pub struct EnhancedRewardDistribution {
    pub block_height: u64,
    pub total_rewards: U256,
    pub validator_rewards: HashMap<Address, ValidatorReward>,
    pub ai_contributor_rewards: HashMap<Address, AIReward>,
    pub network_health_bonus: U256,
    pub treasury_allocation: U256,
    pub burn_amount: U256, // For deflationary pressure
}

#[derive(Debug, Clone)]
pub struct ValidatorReward {
    pub base_reward: U256,
    pub performance_bonus: U256,
    pub staking_bonus: U256,
    pub uptime_bonus: U256,
    pub total_reward: U256,
    pub penalty: U256,
}

#[derive(Debug, Clone)]
pub struct AIReward {
    pub quality_bonus: U256,
    pub compute_bonus: U256,
    pub innovation_bonus: U256,
    pub community_bonus: U256,
    pub total_reward: U256,
}

/// Enhanced reward calculator
pub struct EnhancedRewardCalculator {
    config: EnhancedRewardConfig,
    validator_history: HashMap<Address, Vec<ValidatorPerformance>>,
    ai_contribution_history: HashMap<Address, Vec<AIContribution>>,
    network_health_history: Vec<NetworkHealth>,
}

impl EnhancedRewardCalculator {
    pub fn new(config: EnhancedRewardConfig) -> Self {
        Self {
            config,
            validator_history: HashMap::new(),
            ai_contribution_history: HashMap::new(),
            network_health_history: Vec::new(),
        }
    }

    /// Calculate enhanced rewards for a block
    pub fn calculate_rewards(
        &mut self,
        block_height: u64,
        utilization: &UtilizationMetrics,
        validator_performances: Vec<ValidatorPerformance>,
        ai_contributions: Vec<AIContribution>,
        network_health: NetworkHealth,
    ) -> Result<EnhancedRewardDistribution> {
        // Update historical data
        self.update_histories(&validator_performances, &ai_contributions, network_health.clone());

        // Calculate total reward pool
        let total_reward_pool = self.calculate_total_reward_pool(block_height, utilization)?;

        // Distribute rewards
        let validator_rewards = self.calculate_validator_rewards(&validator_performances, &total_reward_pool)?;
        let ai_rewards = self.calculate_ai_rewards(&ai_contributions, &total_reward_pool)?;
        let network_bonus = self.calculate_network_health_bonus(&network_health, &total_reward_pool)?;

        // Calculate burn amount for deflationary pressure
        let burn_amount = self.calculate_burn_amount(&total_reward_pool, &network_health)?;

        // Treasury allocation (remaining after all distributions)
        let distributed = validator_rewards.values().map(|r| r.total_reward).fold(U256::zero(), |acc, x| acc + x)
            + ai_rewards.values().map(|r| r.total_reward).fold(U256::zero(), |acc, x| acc + x)
            + network_bonus
            + burn_amount;

        let treasury_allocation = if total_reward_pool > distributed {
            total_reward_pool - distributed
        } else {
            U256::zero()
        };

        Ok(EnhancedRewardDistribution {
            block_height,
            total_rewards: total_reward_pool,
            validator_rewards,
            ai_contributor_rewards: ai_rewards,
            network_health_bonus: network_bonus,
            treasury_allocation,
            burn_amount,
        })
    }

    /// Calculate total reward pool based on network conditions
    fn calculate_total_reward_pool(&self, block_height: u64, utilization: &UtilizationMetrics) -> Result<U256> {
        let mut base_pool = self.config.base_block_reward;

        // Apply halving (every ~4 years assuming 2s blocks)
        let halving_interval = 2_100_000u64;
        let halvings = block_height / halving_interval;
        if halvings > 0 && halvings < 64 {
            base_pool = base_pool / U256::from(2u64.pow(halvings as u32));
        }

        // Network activity bonus
        let utilization_rate = utilization.gas_used as f64 / utilization.gas_limit as f64;
        let activity_multiplier = if utilization_rate > 0.8 {
            110 // 10% bonus for high activity
        } else if utilization_rate < 0.3 {
            95  // 5% reduction for low activity
        } else {
            100
        };

        let adjusted_pool = base_pool * U256::from(activity_multiplier) / U256::from(100);

        // AI operations bonus
        let ai_bonus = if utilization.ai_operations > 0 {
            let ai_multiplier = (100 + utilization.ai_operations.min(50)) as u64; // Up to 50% bonus
            adjusted_pool * U256::from(ai_multiplier) / U256::from(100)
        } else {
            adjusted_pool
        };

        Ok(ai_bonus)
    }

    /// Calculate validator rewards based on performance
    fn calculate_validator_rewards(
        &self,
        performances: &[ValidatorPerformance],
        total_pool: &U256,
    ) -> Result<HashMap<Address, ValidatorReward>> {
        let mut rewards = HashMap::new();

        if performances.is_empty() {
            return Ok(rewards);
        }

        let validator_pool = *total_pool * U256::from(self.config.performance_bonus_pool + self.config.staking_bonus_pool) / U256::from(100);

        // Calculate total performance score
        let total_score: f64 = performances.iter()
            .map(|p| self.calculate_validator_score(p))
            .sum();

        if total_score == 0.0 {
            return Ok(rewards);
        }

        for performance in performances {
            if performance.stake_amount < self.config.min_validator_stake {
                continue; // Skip validators with insufficient stake
            }

            let score = self.calculate_validator_score(performance);
            let reward_share = score / total_score;

            let base_reward = validator_pool * U256::from((reward_share * 1000.0) as u64) / U256::from(1000);

            let performance_bonus = self.calculate_performance_bonus(performance, &base_reward);
            let staking_bonus = self.calculate_staking_bonus(performance, &base_reward);
            let uptime_bonus = self.calculate_uptime_bonus(performance, &base_reward);
            let penalty = self.calculate_validator_penalty(performance, &base_reward);

            let total_reward = base_reward + performance_bonus + staking_bonus + uptime_bonus - penalty;

            rewards.insert(performance.address, ValidatorReward {
                base_reward,
                performance_bonus,
                staking_bonus,
                uptime_bonus,
                total_reward,
                penalty,
            });
        }

        Ok(rewards)
    }

    /// Calculate AI contributor rewards
    fn calculate_ai_rewards(
        &self,
        contributions: &[AIContribution],
        total_pool: &U256,
    ) -> Result<HashMap<Address, AIReward>> {
        let mut rewards = HashMap::new();

        if contributions.is_empty() {
            return Ok(rewards);
        }

        let ai_pool = *total_pool * U256::from(self.config.ai_contribution_pool) / U256::from(100);

        // Calculate total AI contribution score
        let total_score: f64 = contributions.iter()
            .map(|c| self.calculate_ai_contribution_score(c))
            .sum();

        if total_score == 0.0 {
            return Ok(rewards);
        }

        for contribution in contributions {
            let score = self.calculate_ai_contribution_score(contribution);
            let reward_share = score / total_score;

            let base_ai_reward = ai_pool * U256::from((reward_share * 1000.0) as u64) / U256::from(1000);

            let quality_bonus = self.calculate_quality_bonus(contribution, &base_ai_reward);
            let compute_bonus = self.calculate_compute_bonus(contribution, &base_ai_reward);
            let innovation_bonus = self.calculate_innovation_bonus(contribution, &base_ai_reward);
            let community_bonus = self.calculate_community_bonus(contribution, &base_ai_reward);

            let total_ai_reward = base_ai_reward + quality_bonus + compute_bonus + innovation_bonus + community_bonus;

            rewards.insert(contribution.contributor, AIReward {
                quality_bonus,
                compute_bonus,
                innovation_bonus,
                community_bonus,
                total_reward: total_ai_reward,
            });
        }

        Ok(rewards)
    }

    /// Calculate network health bonus
    fn calculate_network_health_bonus(&self, health: &NetworkHealth, total_pool: &U256) -> Result<U256> {
        let health_pool = *total_pool * U256::from(self.config.network_health_pool) / U256::from(100);

        // Network health score (0.0 to 1.0)
        let health_score = (health.average_uptime * 0.3 +
                           health.consensus_efficiency * 0.25 +
                           health.transaction_success_rate * 0.2 +
                           health.ai_operation_success_rate * 0.15 +
                           health.network_decentralization * 0.1) / 5.0;

        let bonus = health_pool * U256::from((health_score * 100.0) as u64) / U256::from(100);
        Ok(bonus)
    }

    /// Calculate burn amount based on network conditions
    fn calculate_burn_amount(&self, total_pool: &U256, health: &NetworkHealth) -> Result<U256> {
        // Burn more when network is congested (deflationary pressure)
        // Burn less when network needs growth incentives

        let base_burn_rate = if health.transaction_success_rate > 0.95 && health.average_uptime > 0.95 {
            5 // 5% burn rate when network is healthy
        } else if health.transaction_success_rate < 0.85 || health.average_uptime < 0.85 {
            1 // 1% burn rate when network needs support
        } else {
            3 // 3% default burn rate
        };

        let burn_amount = *total_pool * U256::from(base_burn_rate) / U256::from(100);
        Ok(burn_amount)
    }

    /// Calculate individual validator performance score
    fn calculate_validator_score(&self, performance: &ValidatorPerformance) -> f64 {
        let uptime_score = performance.uptime_percentage;
        let participation_score = performance.consensus_participation;
        let efficiency_score = if performance.blocks_proposed > 0 {
            performance.blocks_validated as f64 / performance.blocks_proposed as f64
        } else {
            1.0
        };
        let quality_score = performance.quality_score;
        let penalty_score = 1.0 - (performance.slash_count as f64 * 0.1).min(0.5);

        (uptime_score * 0.25 + participation_score * 0.25 + efficiency_score * 0.2 +
         quality_score * 0.2 + penalty_score * 0.1).max(0.0)
    }

    /// Calculate AI contribution score
    fn calculate_ai_contribution_score(&self, contribution: &AIContribution) -> f64 {
        let quality_score = contribution.quality_ratings.iter().sum::<f64>() / contribution.quality_ratings.len().max(1) as f64;
        let compute_score = (contribution.compute_provided as f64).log10().max(0.0) / 6.0; // Log scale, max at 1M units
        let innovation_score = (contribution.models_deployed as f64 * 0.3 +
                               contribution.successful_trainings as f64 * 0.7).min(100.0) / 100.0;
        let community_score = contribution.community_reputation;

        (quality_score * 0.4 + compute_score * 0.25 + innovation_score * 0.25 + community_score * 0.1).max(0.0)
    }

    /// Helper methods for bonus calculations
    fn calculate_performance_bonus(&self, performance: &ValidatorPerformance, base: &U256) -> U256 {
        if performance.uptime_percentage > self.config.participation_threshold {
            *base * U256::from(20) / U256::from(100) // 20% bonus
        } else {
            U256::zero()
        }
    }

    fn calculate_staking_bonus(&self, performance: &ValidatorPerformance, base: &U256) -> U256 {
        let stake_ratio = performance.stake_amount.as_u128() as f64 / self.config.min_validator_stake.as_u128() as f64;
        let duration_bonus = (performance.stake_duration as f64 / self.config.performance_window as f64).min(2.0);
        let bonus_rate = (stake_ratio.log2() * duration_bonus * 0.1).min(0.3); // Max 30% bonus

        *base * U256::from((bonus_rate * 100.0) as u64) / U256::from(100)
    }

    fn calculate_uptime_bonus(&self, performance: &ValidatorPerformance, base: &U256) -> U256 {
        if performance.uptime_percentage > 0.99 {
            *base * U256::from(10) / U256::from(100) // 10% bonus for 99%+ uptime
        } else {
            U256::zero()
        }
    }

    fn calculate_validator_penalty(&self, performance: &ValidatorPerformance, base: &U256) -> U256 {
        if performance.slash_count > 0 {
            *base * U256::from(performance.slash_count * 10) / U256::from(100) // 10% penalty per slash
        } else {
            U256::zero()
        }
    }

    fn calculate_quality_bonus(&self, contribution: &AIContribution, base: &U256) -> U256 {
        let avg_quality = contribution.quality_ratings.iter().sum::<f64>() / contribution.quality_ratings.len().max(1) as f64;
        if avg_quality > self.config.ai_quality_threshold {
            let bonus_rate = ((avg_quality - self.config.ai_quality_threshold) * 0.5).min(0.25); // Max 25% bonus
            *base * U256::from((bonus_rate * 100.0) as u64) / U256::from(100)
        } else {
            U256::zero()
        }
    }

    fn calculate_compute_bonus(&self, contribution: &AIContribution, base: &U256) -> U256 {
        let compute_tier = match contribution.compute_provided {
            0..=1000 => 0,
            1001..=10000 => 5,
            10001..=100000 => 10,
            _ => 15,
        };
        *base * U256::from(compute_tier) / U256::from(100)
    }

    fn calculate_innovation_bonus(&self, contribution: &AIContribution, base: &U256) -> U256 {
        let innovation_score = contribution.models_deployed + contribution.successful_trainings * 2;
        let bonus_rate = (innovation_score as f64 * 0.01).min(0.2); // Max 20% bonus
        *base * U256::from((bonus_rate * 100.0) as u64) / U256::from(100)
    }

    fn calculate_community_bonus(&self, contribution: &AIContribution, base: &U256) -> U256 {
        if contribution.community_reputation > 0.8 && contribution.peer_reviews_given > 10 {
            *base * U256::from(15) / U256::from(100) // 15% bonus for active community members
        } else {
            U256::zero()
        }
    }

    /// Update historical data
    fn update_histories(
        &mut self,
        validator_performances: &[ValidatorPerformance],
        ai_contributions: &[AIContribution],
        network_health: NetworkHealth,
    ) {
        // Update validator history
        for performance in validator_performances {
            let history = self.validator_history.entry(performance.address).or_insert_with(Vec::new);
            history.push(performance.clone());
            if history.len() > self.config.performance_window as usize {
                history.remove(0);
            }
        }

        // Update AI contribution history
        for contribution in ai_contributions {
            let history = self.ai_contribution_history.entry(contribution.contributor).or_insert_with(Vec::new);
            history.push(contribution.clone());
            if history.len() > self.config.performance_window as usize {
                history.remove(0);
            }
        }

        // Update network health history
        self.network_health_history.push(network_health);
        if self.network_health_history.len() > self.config.performance_window as usize {
            self.network_health_history.remove(0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enhanced_reward_calculation() {
        let config = EnhancedRewardConfig::default();
        let mut calculator = EnhancedRewardCalculator::new(config);

        let metrics = UtilizationMetrics {
            block_height: 1,
            gas_used: 8_000_000,
            gas_limit: 10_000_000,
            transaction_count: 100,
            ai_operations: 10,
            compute_intensity: 0.5,
        };

        let validator_performance = ValidatorPerformance {
            address: Address([1; 20]),
            blocks_proposed: 10,
            blocks_validated: 10,
            uptime_percentage: 0.99,
            transaction_throughput: 1000,
            ai_operations_processed: 5,
            consensus_participation: 0.95,
            stake_amount: U256::from(50_000) * U256::from(10).pow(U256::from(18)),
            stake_duration: 1000,
            slash_count: 0,
            quality_score: 0.9,
        };

        let ai_contribution = AIContribution {
            contributor: Address([2; 20]),
            models_deployed: 5,
            inferences_served: 1000,
            quality_ratings: vec![0.9, 0.85, 0.95],
            compute_provided: 10000,
            data_contributions: 3,
            successful_trainings: 2,
            peer_reviews_given: 15,
            community_reputation: 0.85,
        };

        let network_health = NetworkHealth {
            total_validators: 100,
            active_validators: 95,
            average_uptime: 0.98,
            consensus_efficiency: 0.95,
            transaction_success_rate: 0.99,
            ai_operation_success_rate: 0.97,
            network_decentralization: 0.85,
            security_incidents: 0,
        };

        let result = calculator.calculate_rewards(
            1,
            &metrics,
            vec![validator_performance],
            vec![ai_contribution],
            network_health,
        ).unwrap();

        assert!(result.total_rewards > U256::zero());
        assert!(!result.validator_rewards.is_empty());
        assert!(!result.ai_contributor_rewards.is_empty());
    }
}