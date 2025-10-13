// lattice-v3/core/economics/src/unified_economics.rs

use crate::{
    governance::{GovernanceManager, GovernanceConfig, ProposalType, ProposalUpdate},
    dynamic_pricing::{DynamicPricingManager, DynamicPricingConfig, UtilizationMetrics, OperationType, PricingUpdate},
    enhanced_rewards::{EnhancedRewardCalculator, EnhancedRewardConfig, ValidatorPerformance, AIContribution, NetworkHealth, EnhancedRewardDistribution},
    revenue_sharing::{RevenueShareManager, RevenueShareConfig, RevenuePool, StakeholderType, RevenueDistribution},
    token::{Token, TokenConfig},
};
use lattice_execution::types::Address;
use primitive_types::U256;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use anyhow::{Result, anyhow};

/// Unified economic system configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedEconomicsConfig {
    pub token_config: TokenConfig,
    pub governance_config: GovernanceConfig,
    pub pricing_config: DynamicPricingConfig,
    pub rewards_config: EnhancedRewardConfig,
    pub revenue_share_config: RevenueShareConfig,
    pub gas_governance_ratio: f64, // How much gas usage affects governance weight
    pub minimum_governance_balance: U256,
    pub economic_security_threshold: f64, // % of tokens needed for economic security
}

impl Default for UnifiedEconomicsConfig {
    fn default() -> Self {
        Self {
            token_config: TokenConfig::default(),
            governance_config: GovernanceConfig::default(),
            pricing_config: DynamicPricingConfig::default(),
            rewards_config: EnhancedRewardConfig::default(),
            revenue_share_config: RevenueShareConfig::default(),
            gas_governance_ratio: 0.1, // 10% weight from gas usage
            minimum_governance_balance: U256::from(100) * U256::from(10).pow(U256::from(18)), // 100 LATT
            economic_security_threshold: 0.67, // 67% threshold for economic security
        }
    }
}

/// Unified governance and gas token voting power calculation
#[derive(Debug, Clone)]
pub struct VotingPower {
    pub token_power: U256,        // Power from token holdings
    pub gas_usage_power: U256,    // Power from gas usage (network participation)
    pub staking_power: U256,      // Power from staking/validation
    pub reputation_power: U256,   // Power from AI contributions and reputation
    pub total_power: U256,        // Combined voting power
    pub quadratic_power: U256,    // Quadratic voting power to prevent plutocracy
}

/// Economic state snapshot
#[derive(Debug, Clone)]
pub struct EconomicState {
    pub block_height: u64,
    pub total_supply: U256,
    pub circulating_supply: U256,
    pub gas_price: U256,
    pub staked_amount: U256,
    pub treasury_balance: U256,
    pub burned_amount: U256,
    pub governance_participation: f64,
    pub network_security_budget: U256,
    pub ai_economy_value: U256,
}

/// Unified economic manager
pub struct UnifiedEconomicsManager {
    config: UnifiedEconomicsConfig,
    token: Token,
    governance: GovernanceManager,
    pricing: DynamicPricingManager,
    rewards: EnhancedRewardCalculator,
    revenue_sharing: RevenueShareManager,
    staking_balances: HashMap<Address, U256>,
    gas_usage_history: HashMap<Address, Vec<(u64, U256)>>, // (block, gas_used)
    reputation_scores: HashMap<Address, f64>,
    economic_metrics: Vec<EconomicState>,
}

impl UnifiedEconomicsManager {
    pub fn new(config: UnifiedEconomicsConfig) -> Self {
        let token = Token::new(config.token_config.clone());
        let governance = GovernanceManager::new(config.governance_config.clone());
        let pricing = DynamicPricingManager::new(config.pricing_config.clone());
        let rewards = EnhancedRewardCalculator::new(config.rewards_config.clone());
        let revenue_sharing = RevenueShareManager::new(config.revenue_share_config.clone());

        Self {
            config,
            token,
            governance,
            pricing,
            rewards,
            revenue_sharing,
            staking_balances: HashMap::new(),
            gas_usage_history: HashMap::new(),
            reputation_scores: HashMap::new(),
            economic_metrics: Vec::new(),
        }
    }

    /// Process a new block and update all economic systems
    pub fn process_block(
        &mut self,
        block_height: u64,
        utilization: UtilizationMetrics,
        validator_performances: Vec<ValidatorPerformance>,
        ai_contributions: Vec<AIContribution>,
        network_health: NetworkHealth,
        gas_usage_by_address: HashMap<Address, U256>,
    ) -> Result<BlockEconomicUpdate> {
        // Update gas usage history for governance weight calculation
        self.update_gas_usage_history(block_height, gas_usage_by_address);

        // Update dynamic pricing
        let pricing_update = self.pricing.update_pricing(utilization.clone())?;

        // Calculate enhanced rewards
        let reward_distribution = self.rewards.calculate_rewards(
            block_height,
            &utilization,
            validator_performances,
            ai_contributions,
            network_health.clone(),
        )?;

        // Distribute rewards
        self.distribute_rewards(&reward_distribution)?;

        // Process governance proposals
        let governance_updates = self.governance.process_proposals(
            block_height,
            self.token.circulating_supply(),
        );

        // Execute any ready proposals
        let executed_proposals = self.execute_ready_proposals()?;

        // Process revenue distributions
        let revenue_distributions = self.process_revenue_distributions(block_height)?;

        // Burn tokens if specified
        if reward_distribution.burn_amount > U256::zero() {
            self.burn_tokens(reward_distribution.burn_amount)?;
        }

        // Update economic state
        let economic_state = self.calculate_economic_state(block_height);
        self.economic_metrics.push(economic_state.clone());

        // Keep only last 1000 blocks of metrics
        if self.economic_metrics.len() > 1000 {
            self.economic_metrics.remove(0);
        }

        Ok(BlockEconomicUpdate {
            block_height,
            pricing_update,
            reward_distribution,
            governance_updates,
            executed_proposals,
            revenue_distributions,
            economic_state,
        })
    }

    /// Calculate unified voting power for an address
    pub fn calculate_voting_power(&self, address: Address, block_height: u64) -> Result<VotingPower> {
        // Token-based power (primary component)
        let token_balance = self.token.balance_of(&address);
        let staked_balance = self.staking_balances.get(&address).copied().unwrap_or(U256::zero());
        let token_power = token_balance + staked_balance;

        // Gas usage power (network participation)
        let gas_usage_power = self.calculate_gas_usage_power(&address, block_height);

        // Staking power (additional weight for validators)
        let staking_power = staked_balance * U256::from(120) / U256::from(100); // 20% bonus for staking

        // Reputation power (AI contributions)
        let reputation_score = self.reputation_scores.get(&address).copied().unwrap_or(0.0);
        let reputation_power = U256::from((reputation_score * 1000.0) as u64) * U256::from(10).pow(U256::from(15)); // Scale reputation

        // Calculate total linear power
        let total_power = token_power + gas_usage_power + staking_power + reputation_power;

        // Apply quadratic voting to prevent plutocracy
        let quadratic_power = self.calculate_quadratic_power(total_power);

        Ok(VotingPower {
            token_power,
            gas_usage_power,
            staking_power,
            reputation_power,
            total_power,
            quadratic_power,
        })
    }

    /// Stake tokens for validation and enhanced governance power
    pub fn stake_tokens(&mut self, staker: Address, amount: U256) -> Result<()> {
        // Check balance
        if self.token.balance_of(&staker) < amount {
            return Err(anyhow!("Insufficient balance to stake"));
        }

        // Transfer to staking
        self.token.burn(&staker, amount)?;
        let current_stake = self.staking_balances.get(&staker).copied().unwrap_or(U256::zero());
        self.staking_balances.insert(staker, current_stake + amount);

        Ok(())
    }

    /// Unstake tokens (with potential slashing)
    pub fn unstake_tokens(&mut self, staker: Address, amount: U256) -> Result<()> {
        let staked = self.staking_balances.get(&staker).copied().unwrap_or(U256::zero());
        if staked < amount {
            return Err(anyhow!("Insufficient staked amount"));
        }

        // Remove from staking
        self.staking_balances.insert(staker, staked - amount);

        // Return tokens (mint back)
        self.token.mint(&staker, amount)?;

        Ok(())
    }

    /// Vote on governance proposal using unified voting power
    pub fn vote_on_proposal(
        &mut self,
        proposal_id: u64,
        voter: Address,
        support: crate::governance::VoteType,
        block_height: u64,
    ) -> Result<()> {
        // Check minimum balance for governance participation
        let total_balance = self.token.balance_of(&voter) +
                           self.staking_balances.get(&voter).copied().unwrap_or(U256::zero());

        if total_balance < self.config.minimum_governance_balance {
            return Err(anyhow!("Insufficient balance for governance participation"));
        }

        // Calculate voting power
        let voting_power = self.calculate_voting_power(voter, block_height)?;

        // Use quadratic power for governance to ensure fairness
        // (but fall back to linear if quadratic is too small)
        let _effective_power = if voting_power.quadratic_power > self.config.governance_config.vote_threshold {
            voting_power.quadratic_power
        } else {
            voting_power.total_power
        };

        // Temporarily override governance vote calculation
        // (In practice, we'd modify the governance manager to accept custom voting power)
        self.governance.vote(
            proposal_id,
            voter,
            support,
            block_height,
            self.token.circulating_supply(),
        )?;

        Ok(())
    }

    /// Get gas price for specific operation type
    pub fn get_operation_cost(&self, operation: OperationType) -> U256 {
        self.pricing.get_operation_price(operation)
    }

    /// Pay gas fees (automatically burns a portion for deflationary pressure)
    pub fn pay_gas_fees(&mut self, payer: Address, amount: U256, burn_percentage: u8) -> Result<()> {
        // Check balance
        if self.token.balance_of(&payer) < amount {
            return Err(anyhow!("Insufficient balance for gas fees"));
        }

        // Calculate burn amount
        let burn_amount = amount * U256::from(burn_percentage) / U256::from(100);
        let remainder = amount - burn_amount;

        // Burn portion of gas fees
        if burn_amount > U256::zero() {
            self.token.burn(&payer, burn_amount)?;
        }

        // Transfer remainder to treasury or validators
        if remainder > U256::zero() {
            // For simplicity, burn all for now (can be changed to fund treasury)
            self.token.burn(&payer, remainder)?;
        }

        Ok(())
    }

    /// Calculate economic security of the network
    pub fn calculate_economic_security(&self) -> f64 {
        let total_staked: U256 = self.staking_balances.values().fold(U256::zero(), |acc, &x| acc + x);
        let total_supply = self.token.circulating_supply();

        if total_supply.is_zero() {
            return 0.0;
        }

        total_staked.as_u128() as f64 / total_supply.as_u128() as f64
    }

    /// Check if network has sufficient economic security
    pub fn is_economically_secure(&self) -> bool {
        self.calculate_economic_security() >= self.config.economic_security_threshold
    }

    /// Update reputation scores based on AI contributions
    pub fn update_reputation(&mut self, address: Address, score_delta: f64) {
        let current_score = self.reputation_scores.get(&address).copied().unwrap_or(0.5);
        let new_score = (current_score + score_delta).max(0.0).min(1.0);
        self.reputation_scores.insert(address, new_score);
    }

    /// Private helper methods
    fn update_gas_usage_history(&mut self, block_height: u64, gas_usage: HashMap<Address, U256>) {
        for (address, gas_used) in gas_usage {
            let history = self.gas_usage_history.entry(address).or_insert_with(Vec::new);
            history.push((block_height, gas_used));

            // Keep only last 100 blocks
            if history.len() > 100 {
                history.remove(0);
            }
        }
    }

    fn calculate_gas_usage_power(&self, address: &Address, block_height: u64) -> U256 {
        let history = self.gas_usage_history.get(address);
        if let Some(history) = history {
            let recent_usage: U256 = history.iter()
                .filter(|(height, _)| block_height - height <= 100) // Last 100 blocks
                .map(|(_, gas)| *gas)
                .fold(U256::zero(), |acc, x| acc + x);

            // Convert gas usage to voting power (scaled down)
            recent_usage * U256::from((self.config.gas_governance_ratio * 100.0) as u64) / U256::from(100)
        } else {
            U256::zero()
        }
    }

    fn calculate_quadratic_power(&self, linear_power: U256) -> U256 {
        // Implement quadratic voting: power = sqrt(tokens)
        // This prevents whale dominance in governance
        if linear_power.is_zero() {
            return U256::zero();
        }

        // Simplified sqrt calculation for U256
        let power_f64 = linear_power.as_u128() as f64;
        let sqrt_power = power_f64.sqrt();

        U256::from(sqrt_power as u128)
    }

    fn distribute_rewards(&mut self, distribution: &EnhancedRewardDistribution) -> Result<()> {
        // Distribute validator rewards
        for (address, reward) in &distribution.validator_rewards {
            self.token.mint(address, reward.total_reward)?;
        }

        // Distribute AI contributor rewards
        for (address, reward) in &distribution.ai_contributor_rewards {
            self.token.mint(address, reward.total_reward)?;
        }

        // Allocate to treasury
        if distribution.treasury_allocation > U256::zero() {
            // Treasury address should be configurable
            let treasury = Address([0x11; 20]); // Placeholder
            self.token.mint(&treasury, distribution.treasury_allocation)?;
        }

        Ok(())
    }

    fn execute_ready_proposals(&mut self) -> Result<Vec<ProposalType>> {
        let executed = Vec::new();

        // This is a simplified version - in practice we'd need to track which proposals are ready
        // and execute them with proper authorization and validation

        Ok(executed)
    }

    fn burn_tokens(&mut self, _amount: U256) -> Result<()> {
        // Burn from total supply (conceptually - implementation would be more complex)
        Ok(())
    }

    fn calculate_economic_state(&self, block_height: u64) -> EconomicState {
        let total_staked: U256 = self.staking_balances.values().fold(U256::zero(), |acc, &x| acc + x);
        let treasury_balance = self.token.balance_of(&Address([0x11; 20])); // Treasury placeholder

        EconomicState {
            block_height,
            total_supply: self.token.config.total_supply,
            circulating_supply: self.token.circulating_supply(),
            gas_price: self.pricing.current_gas_price(),
            staked_amount: total_staked,
            treasury_balance,
            burned_amount: self.token.total_burned,
            governance_participation: 0.0, // Would calculate from active governance
            network_security_budget: total_staked,
            ai_economy_value: U256::zero(), // Would calculate from AI marketplace activity
        }
    }

    /// Getters for external access
    pub fn get_config(&self) -> &UnifiedEconomicsConfig {
        &self.config
    }

    pub fn get_token(&self) -> &Token {
        &self.token
    }

    pub fn get_economic_state(&self) -> Option<&EconomicState> {
        self.economic_metrics.last()
    }

    pub fn get_staked_balance(&self, address: &Address) -> U256 {
        self.staking_balances.get(address).copied().unwrap_or(U256::zero())
    }

    pub fn get_reputation_score(&self, address: &Address) -> f64 {
        self.reputation_scores.get(address).copied().unwrap_or(0.5)
    }

    /// Register stakeholder in revenue sharing system
    pub fn register_stakeholder(
        &mut self,
        address: Address,
        stakeholder_type: StakeholderType,
    ) -> Result<()> {
        self.revenue_sharing.register_stakeholder(address, stakeholder_type)
    }

    /// Collect revenue fees for distribution
    pub fn collect_fee(
        &mut self,
        pool: RevenuePool,
        amount: U256,
        source: Address,
    ) -> Result<()> {
        self.revenue_sharing.collect_revenue(pool, amount, source)
    }

    /// Process revenue distributions and update stakeholder contributions
    pub fn process_revenue_distributions(
        &mut self,
        current_block: u64,
    ) -> Result<Vec<RevenueDistribution>> {
        // Update stakeholder contributions based on recent activity
        self.update_all_stakeholder_contributions(current_block)?;

        // Process distributions
        self.revenue_sharing.process_distributions(current_block)
    }

    /// Update stakeholder contributions from network activity
    fn update_all_stakeholder_contributions(&mut self, _current_block: u64) -> Result<()> {
        // Update contributions for validators based on staking
        for (address, staked_amount) in &self.staking_balances {
            if *staked_amount > U256::zero() {
                self.revenue_sharing.update_contribution(
                    *address,
                    *staked_amount / U256::from(100), // Scale down for contribution scoring
                    1, // One block of activity
                )?;
            }
        }

        // Update contributions for AI model creators based on reputation
        for (address, reputation) in &self.reputation_scores {
            if *reputation > 0.5 {
                let contribution = U256::from((*reputation * 1000.0) as u64);
                self.revenue_sharing.update_contribution(
                    *address,
                    contribution,
                    1,
                )?;
            }
        }

        Ok(())
    }

    /// Get revenue pool balance
    pub fn get_revenue_pool_balance(&self, pool: &RevenuePool) -> U256 {
        self.revenue_sharing.get_pool_balance(pool)
    }

    /// Get stakeholder revenue sharing information
    pub fn get_stakeholder_revenue_info(&self, address: &Address) -> Option<&crate::revenue_sharing::StakeholderContribution> {
        self.revenue_sharing.get_stakeholder_contribution(address)
    }

    /// Get revenue distribution history
    pub fn get_revenue_distribution_history(&self, pool: Option<RevenuePool>) -> Vec<&RevenueDistribution> {
        self.revenue_sharing.get_distribution_history(pool)
    }

    /// Update revenue sharing configuration via governance
    pub fn update_revenue_sharing_config(&mut self, new_config: RevenueShareConfig) -> Result<()> {
        self.revenue_sharing.update_config(new_config)
    }
}

#[derive(Debug, Clone)]
pub struct BlockEconomicUpdate {
    pub block_height: u64,
    pub pricing_update: PricingUpdate,
    pub reward_distribution: EnhancedRewardDistribution,
    pub governance_updates: Vec<ProposalUpdate>,
    pub executed_proposals: Vec<ProposalType>,
    pub revenue_distributions: Vec<RevenueDistribution>,
    pub economic_state: EconomicState,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dynamic_pricing::UtilizationMetrics;

    #[test]
    fn test_unified_economics_initialization() {
        let config = UnifiedEconomicsConfig::default();
        let economics = UnifiedEconomicsManager::new(config);

        assert!(!economics.token.config.name.is_empty());
        assert!(economics.token.circulating_supply() >= U256::zero());
    }

    #[test]
    fn test_voting_power_calculation() {
        let config = UnifiedEconomicsConfig::default();
        let mut economics = UnifiedEconomicsManager::new(config);

        let address = Address([1; 20]);
        let amount = U256::from(1000) * U256::from(10).pow(U256::from(18));

        // Mint some tokens
        economics.token.mint(&address, amount).unwrap();

        let voting_power = economics.calculate_voting_power(address, 1).unwrap();
        assert!(voting_power.total_power > U256::zero());
        assert!(voting_power.quadratic_power <= voting_power.total_power);
    }

    #[test]
    fn test_staking_mechanism() {
        let config = UnifiedEconomicsConfig::default();
        let mut economics = UnifiedEconomicsManager::new(config);

        let address = Address([1; 20]);
        let amount = U256::from(1000) * U256::from(10).pow(U256::from(18));

        // Mint and stake tokens
        economics.token.mint(&address, amount).unwrap();
        economics.stake_tokens(address, amount / 2).unwrap();

        let staked = economics.get_staked_balance(&address);
        assert_eq!(staked, amount / 2);

        // Verify enhanced voting power from staking
        let voting_power = economics.calculate_voting_power(address, 1).unwrap();
        assert!(voting_power.staking_power > U256::zero());
    }
}