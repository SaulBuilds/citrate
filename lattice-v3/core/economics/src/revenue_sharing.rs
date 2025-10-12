// lattice-v3/core/economics/src/revenue_sharing.rs

use lattice_execution::types::Address;
use primitive_types::U256;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use anyhow::Result;

/// Multi-party revenue sharing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevenueShareConfig {
    /// Validator share of network fees (basis points, e.g., 3000 = 30%)
    pub validator_share_bps: u16,

    /// AI model creator share of inference fees (basis points)
    pub model_creator_share_bps: u16,

    /// Network infrastructure share (basis points)
    pub infrastructure_share_bps: u16,

    /// Treasury/DAO share (basis points)
    pub treasury_share_bps: u16,

    /// Staker rewards share (basis points)
    pub staker_share_bps: u16,

    /// Minimum revenue threshold to trigger distribution
    pub min_distribution_threshold: U256,

    /// Distribution frequency in blocks
    pub distribution_frequency: u64,

    /// Performance bonus multiplier (basis points)
    pub performance_bonus_bps: u16,
}

impl Default for RevenueShareConfig {
    fn default() -> Self {
        Self {
            validator_share_bps: 2500,      // 25%
            model_creator_share_bps: 3000,  // 30%
            infrastructure_share_bps: 1500, // 15%
            treasury_share_bps: 1500,       // 15%
            staker_share_bps: 1500,         // 15%
            min_distribution_threshold: U256::from(1000) * U256::from(10).pow(U256::from(18)), // 1000 LATT
            distribution_frequency: 7200,   // ~1 day at 2s blocks
            performance_bonus_bps: 500,     // 5% max bonus
        }
    }
}

/// Revenue pool types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum RevenuePool {
    /// Gas fees from transactions
    GasFees,
    /// AI inference operation fees
    AIInference,
    /// Model deployment fees
    ModelDeployment,
    /// Training operation fees
    ModelTraining,
    /// Marketplace transaction fees
    MarketplaceFees,
    /// Slashing penalties redistribution
    SlashingRedistribution,
}

/// Stakeholder types in revenue sharing
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum StakeholderType {
    Validator,
    ModelCreator,
    Infrastructure,
    Staker,
    Treasury,
}

/// Contribution tracking for stakeholders
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StakeholderContribution {
    pub address: Address,
    pub stakeholder_type: StakeholderType,
    pub contribution_score: f64, // 0.0 to 1.0
    pub total_contribution: U256,
    pub blocks_active: u64,
    pub quality_score: f64, // For AI models
    pub performance_metrics: PerformanceMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub uptime_percentage: f64,
    pub response_time_ms: f64,
    pub success_rate: f64,
    pub user_satisfaction: f64,
    pub network_contribution: f64,
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self {
            uptime_percentage: 0.0,
            response_time_ms: 0.0,
            success_rate: 0.0,
            user_satisfaction: 0.0,
            network_contribution: 0.0,
        }
    }
}

/// Revenue distribution record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevenueDistribution {
    pub block_height: u64,
    pub pool_type: RevenuePool,
    pub total_revenue: U256,
    pub distributions: HashMap<Address, U256>,
    pub timestamp: u64,
}

/// Revenue sharing event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RevenueEvent {
    FeeCollected {
        pool: RevenuePool,
        amount: U256,
        source: Address,
    },
    RevenueDistributed {
        pool: RevenuePool,
        total_amount: U256,
        recipient_count: usize,
    },
    StakeholderRegistered {
        address: Address,
        stakeholder_type: StakeholderType,
    },
    PerformanceUpdated {
        address: Address,
        old_score: f64,
        new_score: f64,
    },
}

/// Multi-party revenue sharing manager
pub struct RevenueShareManager {
    config: RevenueShareConfig,
    revenue_pools: HashMap<RevenuePool, U256>,
    stakeholder_contributions: HashMap<Address, StakeholderContribution>,
    distribution_history: Vec<RevenueDistribution>,
    last_distribution_block: HashMap<RevenuePool, u64>,
    events: Vec<RevenueEvent>,
}

impl RevenueShareManager {
    pub fn new(config: RevenueShareConfig) -> Self {
        Self {
            config,
            revenue_pools: HashMap::new(),
            stakeholder_contributions: HashMap::new(),
            distribution_history: Vec::new(),
            last_distribution_block: HashMap::new(),
            events: Vec::new(),
        }
    }

    /// Register a new stakeholder in the revenue sharing system
    pub fn register_stakeholder(
        &mut self,
        address: Address,
        stakeholder_type: StakeholderType,
    ) -> Result<()> {
        let contribution = StakeholderContribution {
            address,
            stakeholder_type: stakeholder_type.clone(),
            contribution_score: 0.0,
            total_contribution: U256::zero(),
            blocks_active: 0,
            quality_score: 0.0,
            performance_metrics: PerformanceMetrics::default(),
        };

        self.stakeholder_contributions.insert(address, contribution);
        self.events.push(RevenueEvent::StakeholderRegistered {
            address,
            stakeholder_type,
        });

        Ok(())
    }

    /// Collect revenue into appropriate pool
    pub fn collect_revenue(
        &mut self,
        pool: RevenuePool,
        amount: U256,
        source: Address,
    ) -> Result<()> {
        let current_pool = self.revenue_pools.entry(pool.clone()).or_insert(U256::zero());
        *current_pool += amount;

        self.events.push(RevenueEvent::FeeCollected {
            pool,
            amount,
            source,
        });

        Ok(())
    }

    /// Update stakeholder performance metrics
    pub fn update_performance(
        &mut self,
        address: Address,
        metrics: PerformanceMetrics,
    ) -> Result<()> {
        // Get old score before mutable borrow
        let old_score = self.stakeholder_contributions.get(&address)
            .map(|c| self.calculate_performance_score(&c.performance_metrics))
            .unwrap_or(0.0);

        if let Some(contribution) = self.stakeholder_contributions.get_mut(&address) {
            contribution.performance_metrics = metrics.clone();
            let new_score = self.calculate_performance_score(&metrics);

            self.events.push(RevenueEvent::PerformanceUpdated {
                address,
                old_score,
                new_score,
            });
        }

        Ok(())
    }

    /// Calculate performance score from metrics
    fn calculate_performance_score(&self, metrics: &PerformanceMetrics) -> f64 {
        let weights = [0.3, 0.2, 0.2, 0.15, 0.15]; // uptime, response, success, satisfaction, contribution
        let scores = [
            metrics.uptime_percentage / 100.0,
            (1000.0 - metrics.response_time_ms.min(1000.0)) / 1000.0, // Lower is better
            metrics.success_rate,
            metrics.user_satisfaction,
            metrics.network_contribution,
        ];

        weights.iter().zip(scores.iter()).map(|(w, s)| w * s).sum()
    }

    /// Distribute revenue from a specific pool
    pub fn distribute_revenue(
        &mut self,
        pool: RevenuePool,
        current_block: u64,
    ) -> Result<Option<RevenueDistribution>> {
        let pool_balance = self.revenue_pools.get(&pool).copied().unwrap_or(U256::zero());

        // Check if distribution threshold is met
        if pool_balance < self.config.min_distribution_threshold {
            return Ok(None);
        }

        // Check if enough time has passed since last distribution
        let last_distribution = self.last_distribution_block.get(&pool).copied().unwrap_or(0);
        if current_block - last_distribution < self.config.distribution_frequency {
            return Ok(None);
        }

        let distributions = self.calculate_distributions(&pool, pool_balance)?;

        let distribution = RevenueDistribution {
            block_height: current_block,
            pool_type: pool.clone(),
            total_revenue: pool_balance,
            distributions: distributions.clone(),
            timestamp: current_block, // In real implementation, use actual timestamp
        };

        // Reset pool balance
        self.revenue_pools.insert(pool.clone(), U256::zero());
        self.last_distribution_block.insert(pool.clone(), current_block);
        self.distribution_history.push(distribution.clone());

        self.events.push(RevenueEvent::RevenueDistributed {
            pool,
            total_amount: pool_balance,
            recipient_count: distributions.len(),
        });

        Ok(Some(distribution))
    }

    /// Calculate revenue distributions based on pool type and stakeholder contributions
    fn calculate_distributions(
        &self,
        pool: &RevenuePool,
        total_amount: U256,
    ) -> Result<HashMap<Address, U256>> {
        let mut distributions = HashMap::new();

        match pool {
            RevenuePool::GasFees => {
                // Distribute gas fees to validators and stakers
                let validator_amount = total_amount * U256::from(self.config.validator_share_bps) / U256::from(10000);
                let staker_amount = total_amount * U256::from(self.config.staker_share_bps) / U256::from(10000);
                let treasury_amount = total_amount - validator_amount - staker_amount;

                self.distribute_to_stakeholder_type(StakeholderType::Validator, validator_amount, &mut distributions);
                self.distribute_to_stakeholder_type(StakeholderType::Staker, staker_amount, &mut distributions);
                self.distribute_to_stakeholder_type(StakeholderType::Treasury, treasury_amount, &mut distributions);
            },
            RevenuePool::AIInference => {
                // Distribute AI inference fees to model creators, validators, and infrastructure
                let model_creator_amount = total_amount * U256::from(self.config.model_creator_share_bps) / U256::from(10000);
                let validator_amount = total_amount * U256::from(self.config.validator_share_bps) / U256::from(10000);
                let infrastructure_amount = total_amount * U256::from(self.config.infrastructure_share_bps) / U256::from(10000);
                let treasury_amount = total_amount - model_creator_amount - validator_amount - infrastructure_amount;

                self.distribute_to_stakeholder_type(StakeholderType::ModelCreator, model_creator_amount, &mut distributions);
                self.distribute_to_stakeholder_type(StakeholderType::Validator, validator_amount, &mut distributions);
                self.distribute_to_stakeholder_type(StakeholderType::Infrastructure, infrastructure_amount, &mut distributions);
                self.distribute_to_stakeholder_type(StakeholderType::Treasury, treasury_amount, &mut distributions);
            },
            RevenuePool::ModelDeployment | RevenuePool::ModelTraining => {
                // Similar to AI inference but with different weights
                let model_creator_amount = total_amount * U256::from(4000) / U256::from(10000); // 40% for model creators
                let infrastructure_amount = total_amount * U256::from(3000) / U256::from(10000); // 30% for infrastructure
                let validator_amount = total_amount * U256::from(2000) / U256::from(10000); // 20% for validators
                let treasury_amount = total_amount - model_creator_amount - infrastructure_amount - validator_amount;

                self.distribute_to_stakeholder_type(StakeholderType::ModelCreator, model_creator_amount, &mut distributions);
                self.distribute_to_stakeholder_type(StakeholderType::Infrastructure, infrastructure_amount, &mut distributions);
                self.distribute_to_stakeholder_type(StakeholderType::Validator, validator_amount, &mut distributions);
                self.distribute_to_stakeholder_type(StakeholderType::Treasury, treasury_amount, &mut distributions);
            },
            RevenuePool::MarketplaceFees => {
                // Distribute marketplace fees evenly with small treasury cut
                let treasury_amount = total_amount * U256::from(1000) / U256::from(10000); // 10%
                let remaining = total_amount - treasury_amount;
                let equal_share = remaining / U256::from(4); // Split between 4 stakeholder types

                self.distribute_to_stakeholder_type(StakeholderType::ModelCreator, equal_share, &mut distributions);
                self.distribute_to_stakeholder_type(StakeholderType::Validator, equal_share, &mut distributions);
                self.distribute_to_stakeholder_type(StakeholderType::Infrastructure, equal_share, &mut distributions);
                self.distribute_to_stakeholder_type(StakeholderType::Staker, equal_share, &mut distributions);
                self.distribute_to_stakeholder_type(StakeholderType::Treasury, treasury_amount, &mut distributions);
            },
            RevenuePool::SlashingRedistribution => {
                // Redistribute slashing penalties to good actors
                let validator_amount = total_amount * U256::from(4000) / U256::from(10000); // 40%
                let staker_amount = total_amount * U256::from(4000) / U256::from(10000); // 40%
                let treasury_amount = total_amount - validator_amount - staker_amount; // 20%

                self.distribute_to_stakeholder_type(StakeholderType::Validator, validator_amount, &mut distributions);
                self.distribute_to_stakeholder_type(StakeholderType::Staker, staker_amount, &mut distributions);
                self.distribute_to_stakeholder_type(StakeholderType::Treasury, treasury_amount, &mut distributions);
            },
        }

        Ok(distributions)
    }

    /// Distribute amount to all stakeholders of a specific type
    fn distribute_to_stakeholder_type(
        &self,
        stakeholder_type: StakeholderType,
        total_amount: U256,
        distributions: &mut HashMap<Address, U256>,
    ) {
        let stakeholders: Vec<_> = self.stakeholder_contributions
            .iter()
            .filter(|(_, contrib)| contrib.stakeholder_type == stakeholder_type)
            .collect();

        if stakeholders.is_empty() {
            return;
        }

        // Calculate total contribution score for normalization
        let total_score: f64 = stakeholders
            .iter()
            .map(|(_, contrib)| self.calculate_weighted_contribution(contrib))
            .sum();

        if total_score == 0.0 {
            // Equal distribution if no contributions recorded
            let equal_share = total_amount / U256::from(stakeholders.len());
            for (address, _) in stakeholders {
                *distributions.entry(*address).or_insert(U256::zero()) += equal_share;
            }
        } else {
            // Proportional distribution based on weighted contributions
            for (address, contrib) in stakeholders {
                let contribution_weight = self.calculate_weighted_contribution(contrib);
                let share_ratio = contribution_weight / total_score;
                let amount = total_amount * U256::from((share_ratio * 10000.0) as u64) / U256::from(10000);
                *distributions.entry(*address).or_insert(U256::zero()) += amount;
            }
        }
    }

    /// Calculate weighted contribution score including performance bonuses
    fn calculate_weighted_contribution(&self, contrib: &StakeholderContribution) -> f64 {
        let base_score = contrib.contribution_score;
        let performance_score = self.calculate_performance_score(&contrib.performance_metrics);
        let quality_bonus = contrib.quality_score * 0.2; // 20% max quality bonus

        // Performance bonus up to the configured maximum
        let performance_bonus = performance_score * (self.config.performance_bonus_bps as f64 / 10000.0);

        base_score * (1.0 + performance_bonus + quality_bonus)
    }

    /// Update stakeholder contribution score
    pub fn update_contribution(
        &mut self,
        address: Address,
        contribution_delta: U256,
        blocks_active_delta: u64,
    ) -> Result<()> {
        if let Some(contrib) = self.stakeholder_contributions.get_mut(&address) {
            contrib.total_contribution += contribution_delta;
            contrib.blocks_active += blocks_active_delta;

            // Recalculate contribution score (weighted by recency)
            let recent_weight = 0.7;
            let historical_weight = 0.3;
            let recent_contribution = contribution_delta.as_u128() as f64;
            let historical_contribution = contrib.total_contribution.as_u128() as f64;

            contrib.contribution_score = recent_weight * (recent_contribution / 1e18) +
                                       historical_weight * (historical_contribution / 1e18);
        }

        Ok(())
    }

    /// Get revenue pool balance
    pub fn get_pool_balance(&self, pool: &RevenuePool) -> U256 {
        self.revenue_pools.get(pool).copied().unwrap_or(U256::zero())
    }

    /// Get stakeholder contribution info
    pub fn get_stakeholder_contribution(&self, address: &Address) -> Option<&StakeholderContribution> {
        self.stakeholder_contributions.get(address)
    }

    /// Get distribution history
    pub fn get_distribution_history(&self, pool: Option<RevenuePool>) -> Vec<&RevenueDistribution> {
        match pool {
            Some(pool_type) => self.distribution_history
                .iter()
                .filter(|d| d.pool_type == pool_type)
                .collect(),
            None => self.distribution_history.iter().collect(),
        }
    }

    /// Get recent events
    pub fn get_recent_events(&self, limit: usize) -> Vec<&RevenueEvent> {
        self.events
            .iter()
            .rev()
            .take(limit)
            .collect()
    }

    /// Process all pending distributions for current block
    pub fn process_distributions(&mut self, current_block: u64) -> Result<Vec<RevenueDistribution>> {
        let mut distributions = Vec::new();

        // Try to distribute from each pool
        for pool in [
            RevenuePool::GasFees,
            RevenuePool::AIInference,
            RevenuePool::ModelDeployment,
            RevenuePool::ModelTraining,
            RevenuePool::MarketplaceFees,
            RevenuePool::SlashingRedistribution,
        ] {
            if let Some(distribution) = self.distribute_revenue(pool, current_block)? {
                distributions.push(distribution);
            }
        }

        Ok(distributions)
    }

    /// Update configuration via governance
    pub fn update_config(&mut self, new_config: RevenueShareConfig) -> Result<()> {
        // Validate configuration
        let total_bps = new_config.validator_share_bps as u32 +
                       new_config.model_creator_share_bps as u32 +
                       new_config.infrastructure_share_bps as u32 +
                       new_config.treasury_share_bps as u32 +
                       new_config.staker_share_bps as u32;

        if total_bps > 10000 {
            return Err(anyhow::anyhow!("Total revenue shares exceed 100%"));
        }

        self.config = new_config;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_revenue_collection_and_distribution() {
        let config = RevenueShareConfig::default();
        let mut manager = RevenueShareManager::new(config);

        // Register stakeholders
        let validator = Address([1; 20]);
        let model_creator = Address([2; 20]);

        manager.register_stakeholder(validator, StakeholderType::Validator).unwrap();
        manager.register_stakeholder(model_creator, StakeholderType::ModelCreator).unwrap();

        // Collect revenue
        let fee_amount = U256::from(2000) * U256::from(10).pow(U256::from(18)); // 2000 LATT
        manager.collect_revenue(RevenuePool::AIInference, fee_amount, validator).unwrap();

        // Update contributions
        manager.update_contribution(validator, U256::from(100), 100).unwrap();
        manager.update_contribution(model_creator, U256::from(200), 150).unwrap();

        // Try distribution (should work since we have enough funds)
        let distribution = manager.distribute_revenue(RevenuePool::AIInference, 7200).unwrap();
        assert!(distribution.is_some());

        let dist = distribution.unwrap();
        assert_eq!(dist.total_revenue, fee_amount);
        assert!(!dist.distributions.is_empty());
    }

    #[test]
    fn test_performance_scoring() {
        let config = RevenueShareConfig::default();
        let manager = RevenueShareManager::new(config);

        let high_performance = PerformanceMetrics {
            uptime_percentage: 99.9,
            response_time_ms: 50.0,
            success_rate: 0.99,
            user_satisfaction: 0.95,
            network_contribution: 0.8,
        };

        let score = manager.calculate_performance_score(&high_performance);
        assert!(score > 0.8); // Should be high score
    }
}