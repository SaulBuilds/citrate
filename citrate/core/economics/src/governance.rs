// citrate/core/economics/src/governance.rs

use citrate_execution::types::Address;
use primitive_types::U256;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use anyhow::{Result, anyhow};

/// Governance configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceConfig {
    /// Minimum LATT required to create a proposal
    pub proposal_threshold: U256,

    /// Minimum LATT required to vote
    pub vote_threshold: U256,

    /// Voting period in blocks
    pub voting_period: u64,

    /// Execution delay in blocks after proposal passes
    pub execution_delay: u64,

    /// Quorum requirement as percentage (0-100)
    pub quorum_percentage: u8,

    /// Minimum approval percentage to pass (0-100)
    pub approval_threshold: u8,

    /// Grace period for execution in blocks
    pub grace_period: u64,
}

impl Default for GovernanceConfig {
    fn default() -> Self {
        Self {
            proposal_threshold: U256::from(10_000) * U256::from(10).pow(U256::from(18)), // 10,000 LATT
            vote_threshold: U256::from(1) * U256::from(10).pow(U256::from(18)), // 1 LATT
            voting_period: 50_400, // ~7 days at 2s blocks
            execution_delay: 7_200, // ~1 day delay
            quorum_percentage: 10, // 10% of total supply must vote
            approval_threshold: 60, // 60% approval needed
            grace_period: 50_400, // 7 days to execute
        }
    }
}

/// Governance proposal types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProposalType {
    /// Parameter change (gas prices, reward rates, etc.)
    ParameterChange {
        parameter: String,
        new_value: U256,
    },
    /// Network upgrade
    NetworkUpgrade {
        version: String,
        upgrade_block: u64,
    },
    /// Treasury spending
    TreasurySpend {
        recipient: Address,
        amount: U256,
        description: String,
    },
    /// Emergency action (fast track with higher threshold)
    Emergency {
        action: String,
        reason: String,
    },
    /// Model marketplace governance
    MarketplaceGovernance {
        action: MarketplaceAction,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MarketplaceAction {
    SetMinModelStake(U256),
    SetMarketplaceFee(u8), // Percentage
    BanModel(String), // Model ID
    UpdateQualityThreshold(f32),
}

/// Governance proposal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Proposal {
    pub id: u64,
    pub proposer: Address,
    pub proposal_type: ProposalType,
    pub title: String,
    pub description: String,
    pub created_at: u64, // Block height
    pub voting_starts: u64,
    pub voting_ends: u64,
    pub execution_eta: Option<u64>, // Set when proposal passes
    pub status: ProposalStatus,
    pub for_votes: U256,
    pub against_votes: U256,
    pub abstain_votes: U256,
    pub voters: HashMap<Address, Vote>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProposalStatus {
    Pending,     // Before voting starts
    Active,      // Currently voting
    Succeeded,   // Passed and waiting for execution
    Queued,      // In execution delay period
    Executed,    // Successfully executed
    Failed,      // Did not meet requirements
    Canceled,    // Canceled by proposer
    Expired,     // Passed grace period without execution
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vote {
    pub voter: Address,
    pub support: VoteType,
    pub weight: U256, // Voting power at time of vote
    pub block_height: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VoteType {
    For,
    Against,
    Abstain,
}

/// Delegated voting system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VotingDelegation {
    pub delegator: Address,
    pub delegate: Address,
    pub delegated_amount: U256,
    pub valid_from: u64, // Block height
}

/// Governance state manager
pub struct GovernanceManager {
    config: GovernanceConfig,
    proposals: HashMap<u64, Proposal>,
    next_proposal_id: u64,
    delegations: HashMap<Address, Vec<VotingDelegation>>,
    voting_power_cache: HashMap<(Address, u64), U256>, // (address, block) -> power
}

impl GovernanceManager {
    pub fn new(config: GovernanceConfig) -> Self {
        Self {
            config,
            proposals: HashMap::new(),
            next_proposal_id: 1,
            delegations: HashMap::new(),
            voting_power_cache: HashMap::new(),
        }
    }

    /// Create a new proposal
    pub fn create_proposal(
        &mut self,
        proposer: Address,
        proposal_type: ProposalType,
        title: String,
        description: String,
        current_block: u64,
        proposer_balance: U256,
    ) -> Result<u64> {
        // Check proposal threshold
        if proposer_balance < self.config.proposal_threshold {
            return Err(anyhow!("Insufficient balance to create proposal"));
        }

        let proposal_id = self.next_proposal_id;
        self.next_proposal_id += 1;

        let voting_starts = current_block + 1; // Starts next block
        let voting_ends = voting_starts + self.config.voting_period;

        let proposal = Proposal {
            id: proposal_id,
            proposer,
            proposal_type,
            title,
            description,
            created_at: current_block,
            voting_starts,
            voting_ends,
            execution_eta: None,
            status: ProposalStatus::Pending,
            for_votes: U256::zero(),
            against_votes: U256::zero(),
            abstain_votes: U256::zero(),
            voters: HashMap::new(),
        };

        self.proposals.insert(proposal_id, proposal);
        Ok(proposal_id)
    }

    /// Cast a vote on a proposal
    pub fn vote(
        &mut self,
        proposal_id: u64,
        voter: Address,
        support: VoteType,
        current_block: u64,
        total_supply: U256,
    ) -> Result<()> {
        // First, get voting_starts and do validation without mutable borrow
        let (voting_starts, _voting_ends) = {
            let proposal = self.proposals.get(&proposal_id)
                .ok_or_else(|| anyhow!("Proposal not found"))?;

            // Check voting period
            if current_block < proposal.voting_starts {
                return Err(anyhow!("Voting has not started"));
            }
            if current_block > proposal.voting_ends {
                return Err(anyhow!("Voting has ended"));
            }

            // Check if already voted
            if proposal.voters.contains_key(&voter) {
                return Err(anyhow!("Already voted"));
            }

            (proposal.voting_starts, proposal.voting_ends)
        };

        // Calculate voting power (including delegations)
        let voting_power = self.calculate_voting_power(voter, voting_starts, total_supply)?;

        if voting_power < self.config.vote_threshold {
            return Err(anyhow!("Insufficient voting power"));
        }

        // Record vote
        let vote = Vote {
            voter,
            support: support.clone(),
            weight: voting_power,
            block_height: current_block,
        };

        // Now get mutable reference to update the proposal
        let proposal = self.proposals.get_mut(&proposal_id).unwrap(); // Safe because we checked above

        // Update vote counts
        match support {
            VoteType::For => proposal.for_votes += voting_power,
            VoteType::Against => proposal.against_votes += voting_power,
            VoteType::Abstain => proposal.abstain_votes += voting_power,
        }

        proposal.voters.insert(voter, vote);
        proposal.status = ProposalStatus::Active;

        Ok(())
    }

    /// Delegate voting power to another address
    pub fn delegate_vote(
        &mut self,
        delegator: Address,
        delegate: Address,
        amount: U256,
        current_block: u64,
    ) -> Result<()> {
        let delegation = VotingDelegation {
            delegator,
            delegate,
            delegated_amount: amount,
            valid_from: current_block,
        };

        self.delegations.entry(delegate).or_insert_with(Vec::new).push(delegation);
        Ok(())
    }

    /// Process proposals at current block
    pub fn process_proposals(&mut self, current_block: u64, total_supply: U256) -> Vec<ProposalUpdate> {
        let mut updates = Vec::new();

        for proposal in self.proposals.values_mut() {
            match proposal.status {
                ProposalStatus::Pending if current_block >= proposal.voting_starts => {
                    proposal.status = ProposalStatus::Active;
                    updates.push(ProposalUpdate::VotingStarted(proposal.id));
                }
                ProposalStatus::Active if current_block > proposal.voting_ends => {
                    // Check if proposal passed
                    let total_votes = proposal.for_votes + proposal.against_votes + proposal.abstain_votes;
                    let quorum_required = total_supply * U256::from(self.config.quorum_percentage) / U256::from(100);
                    let approval_required = total_votes * U256::from(self.config.approval_threshold) / U256::from(100);

                    if total_votes >= quorum_required && proposal.for_votes >= approval_required {
                        proposal.status = ProposalStatus::Succeeded;
                        proposal.execution_eta = Some(current_block + self.config.execution_delay);
                        updates.push(ProposalUpdate::Passed(proposal.id));
                    } else {
                        proposal.status = ProposalStatus::Failed;
                        updates.push(ProposalUpdate::Failed(proposal.id));
                    }
                }
                ProposalStatus::Succeeded if proposal.execution_eta.map_or(false, |eta| current_block >= eta) => {
                    proposal.status = ProposalStatus::Queued;
                    updates.push(ProposalUpdate::ReadyForExecution(proposal.id));
                }
                ProposalStatus::Queued if proposal.execution_eta.map_or(false, |eta| current_block > eta + self.config.grace_period) => {
                    proposal.status = ProposalStatus::Expired;
                    updates.push(ProposalUpdate::Expired(proposal.id));
                }
                _ => {}
            }
        }

        updates
    }

    /// Execute a queued proposal
    pub fn execute_proposal(&mut self, proposal_id: u64) -> Result<ProposalType> {
        let proposal = self.proposals.get_mut(&proposal_id)
            .ok_or_else(|| anyhow!("Proposal not found"))?;

        if proposal.status != ProposalStatus::Queued {
            return Err(anyhow!("Proposal not ready for execution"));
        }

        proposal.status = ProposalStatus::Executed;
        Ok(proposal.proposal_type.clone())
    }

    /// Calculate voting power including delegations
    fn calculate_voting_power(&mut self, address: Address, block_height: u64, total_supply: U256) -> Result<U256> {
        // Check cache first
        if let Some(&cached_power) = self.voting_power_cache.get(&(address, block_height)) {
            return Ok(cached_power);
        }

        // Base voting power (token balance at snapshot)
        // In a real implementation, this would query the token balance at specific block
        let base_power = total_supply / U256::from(1000); // Placeholder

        // Add delegated power
        let delegated_power = self.delegations.get(&address)
            .map(|delegations| {
                delegations.iter()
                    .filter(|d| d.valid_from <= block_height)
                    .map(|d| d.delegated_amount)
                    .fold(U256::zero(), |acc, amount| acc + amount)
            })
            .unwrap_or(U256::zero());

        let total_power = base_power + delegated_power;
        self.voting_power_cache.insert((address, block_height), total_power);

        Ok(total_power)
    }

    /// Get proposal by ID
    pub fn get_proposal(&self, id: u64) -> Option<&Proposal> {
        self.proposals.get(&id)
    }

    /// Get all active proposals
    pub fn get_active_proposals(&self) -> Vec<&Proposal> {
        self.proposals.values()
            .filter(|p| matches!(p.status, ProposalStatus::Active))
            .collect()
    }

    /// Update governance configuration via executed proposal
    pub fn update_config(&mut self, parameter: &str, value: U256) -> Result<()> {
        match parameter {
            "proposal_threshold" => self.config.proposal_threshold = value,
            "vote_threshold" => self.config.vote_threshold = value,
            "voting_period" => self.config.voting_period = value.as_u64(),
            "execution_delay" => self.config.execution_delay = value.as_u64(),
            "quorum_percentage" => self.config.quorum_percentage = value.as_u64() as u8,
            "approval_threshold" => self.config.approval_threshold = value.as_u64() as u8,
            "grace_period" => self.config.grace_period = value.as_u64(),
            _ => return Err(anyhow!("Unknown parameter: {}", parameter)),
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub enum ProposalUpdate {
    VotingStarted(u64),
    Passed(u64),
    Failed(u64),
    ReadyForExecution(u64),
    Expired(u64),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_proposal() {
        let config = GovernanceConfig::default();
        let mut gov = GovernanceManager::new(config.clone());

        let proposer = Address([1; 20]);
        let balance = config.proposal_threshold;

        let proposal_id = gov.create_proposal(
            proposer,
            ProposalType::ParameterChange {
                parameter: "block_reward".to_string(),
                new_value: U256::from(15),
            },
            "Increase block reward".to_string(),
            "Proposal to increase block reward to 15 LATT".to_string(),
            100,
            balance,
        ).unwrap();

        assert_eq!(proposal_id, 1);
        let proposal = gov.get_proposal(1).unwrap();
        assert_eq!(proposal.proposer, proposer);
    }

    #[test]
    fn test_voting() {
        let config = GovernanceConfig::default();
        let mut gov = GovernanceManager::new(config.clone());

        let proposer = Address([1; 20]);
        let voter = Address([2; 20]);
        let total_supply = U256::from(1_000_000_000) * U256::from(10).pow(U256::from(18));

        // Create proposal
        let proposal_id = gov.create_proposal(
            proposer,
            ProposalType::ParameterChange {
                parameter: "block_reward".to_string(),
                new_value: U256::from(15),
            },
            "Test proposal".to_string(),
            "Test description".to_string(),
            100,
            config.proposal_threshold,
        ).unwrap();

        // Vote on proposal
        let result = gov.vote(proposal_id, voter, VoteType::For, 102, total_supply);
        assert!(result.is_ok());

        let proposal = gov.get_proposal(proposal_id).unwrap();
        assert!(proposal.for_votes > U256::zero());
    }
}