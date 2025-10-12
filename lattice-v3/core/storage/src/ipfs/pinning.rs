// lattice-v3/core/storage/src/ipfs/pinning.rs

//! Pinning incentive accounting for IPFS-backed model storage.
//!
//! This module keeps lightweight, in-memory accounting of which
//! participants are providing replicas for a given CID and the
//! rewards they have accrued so far.

use super::{Cid, ModelMetadata, ModelType};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

/// Reward information returned whenever a new pin report is recorded.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PinReward {
    pub cid: Cid,
    pub pinner_id: String,
    pub reward: u64,
    pub total_replicas: u32,
}

/// Summary of pinning state for a given CID.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PinningSummary {
    pub cid: Cid,
    pub metadata: ModelMetadata,
    pub total_pinned_bytes: u64,
    pub total_rewards: u128,
    pub total_reports: u64,
    pub total_replicas: u32,
    pub last_updated: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PinnerStats {
    total_pinned_bytes: u64,
    rewards_earned: u128,
    reports: u64,
    last_reported_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PinningRecord {
    cid: Cid,
    metadata: ModelMetadata,
    total_pinned_bytes: u64,
    total_rewards: u128,
    total_reports: u64,
    last_updated: u64,
    pinners: HashMap<String, PinnerStats>,
}

impl PinningRecord {
    fn new(cid: Cid, metadata: ModelMetadata) -> Self {
        let now = current_unix_time();
        Self {
            cid,
            metadata,
            total_pinned_bytes: 0,
            total_rewards: 0,
            total_reports: 0,
            last_updated: now,
            pinners: HashMap::new(),
        }
    }
}

/// Manages pinning records and reward estimation.
#[derive(Default)]
pub struct PinningManager {
    records: HashMap<String, PinningRecord>,
}

impl PinningManager {
    pub fn new() -> Self {
        Self {
            records: HashMap::new(),
        }
    }

    pub fn record_pin(
        &mut self,
        cid: Cid,
        pinner_id: String,
        metadata: ModelMetadata,
        pinned_bytes: u64,
    ) -> PinReward {
        let reward = Self::reward_for_bytes(pinned_bytes, &metadata.model_type);
        let key = cid.0.clone();
        let now = current_unix_time();

        let record_entry = self
            .records
            .entry(key.clone())
            .or_insert_with(|| PinningRecord::new(cid.clone(), metadata.clone()));

        record_entry.total_pinned_bytes =
            record_entry.total_pinned_bytes.saturating_add(pinned_bytes);
        record_entry.total_rewards = record_entry.total_rewards.saturating_add(reward as u128);
        record_entry.total_reports = record_entry.total_reports.saturating_add(1);
        record_entry.last_updated = now;

        let pinner_entry = record_entry
            .pinners
            .entry(pinner_id.clone())
            .or_insert(PinnerStats {
                total_pinned_bytes: 0,
                rewards_earned: 0,
                reports: 0,
                last_reported_at: 0,
            });

        pinner_entry.total_pinned_bytes =
            pinner_entry.total_pinned_bytes.saturating_add(pinned_bytes);
        pinner_entry.rewards_earned = pinner_entry.rewards_earned.saturating_add(reward as u128);
        pinner_entry.reports = pinner_entry.reports.saturating_add(1);
        pinner_entry.last_reported_at = now;

        PinReward {
            cid,
            pinner_id,
            reward,
            total_replicas: record_entry.pinners.len() as u32,
        }
    }

    pub fn summary(&self, cid: &Cid) -> Option<PinningSummary> {
        self.records.get(&cid.0).map(|record| PinningSummary {
            cid: record.cid.clone(),
            metadata: record.metadata.clone(),
            total_pinned_bytes: record.total_pinned_bytes,
            total_rewards: record.total_rewards,
            total_reports: record.total_reports,
            total_replicas: record.pinners.len() as u32,
            last_updated: record.last_updated,
        })
    }

    pub fn reward_for_duration(metadata: &ModelMetadata, duration_hours: u64) -> u64 {
        let size_gb = bytes_to_gb(metadata.size_bytes);
        let days = (duration_hours as f64 / 24.0).max(1.0);
        let base = (size_gb * days).ceil().max(1.0) as u64;
        base * reward_multiplier(&metadata.model_type)
    }

    pub fn reward_for_bytes(bytes: u64, model_type: &ModelType) -> u64 {
        let size_gb = bytes_to_gb(bytes).ceil().max(1.0) as u64;
        size_gb * reward_multiplier(model_type)
    }
}

fn current_unix_time() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn bytes_to_gb(bytes: u64) -> f64 {
    bytes as f64 / (1024.0 * 1024.0 * 1024.0)
}

fn reward_multiplier(model_type: &ModelType) -> u64 {
    match model_type {
        ModelType::Language => 2,
        ModelType::Vision => 3,
        ModelType::Audio => 2,
        ModelType::Multimodal => 4,
        ModelType::Reinforcement => 3,
        ModelType::Custom(_) => 1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_metadata(model_type: ModelType, size_bytes: u64) -> ModelMetadata {
        ModelMetadata {
            name: "Sample".to_string(),
            version: "1.0.0".to_string(),
            framework: super::super::ModelFramework::ONNX,
            model_type,
            size_bytes,
            input_shape: vec![1, 3, 224, 224],
            output_shape: vec![1, 1000],
            description: "Sample metadata".to_string(),
            author: "Lattice".to_string(),
            license: "MIT".to_string(),
            created_at: 0,
        }
    }

    #[test]
    fn reward_for_bytes_scales_with_model_type() {
        let language_reward = PinningManager::reward_for_bytes(1_073_741_824, &ModelType::Language);
        let vision_reward = PinningManager::reward_for_bytes(1_073_741_824, &ModelType::Vision);

        assert_eq!(language_reward, 2);
        assert_eq!(vision_reward, 3);
    }

    #[test]
    fn record_pin_tracks_replicas_and_rewards() {
        let mut manager = PinningManager::new();
        let cid = Cid("QmTestCID".to_string());
        let metadata = sample_metadata(ModelType::Vision, 1_500_000_000);

        let reward = manager.record_pin(
            cid.clone(),
            "node-1".to_string(),
            metadata.clone(),
            metadata.size_bytes,
        );

        assert!(reward.reward > 0);
        assert_eq!(reward.total_replicas, 1);

        let summary = manager.summary(&cid).expect("summary exists");
        assert_eq!(summary.total_replicas, 1);
        assert_eq!(summary.total_reports, 1);
        assert_eq!(summary.total_pinned_bytes, metadata.size_bytes);
        assert_eq!(summary.total_rewards, reward.reward as u128);
    }

    #[test]
    fn reward_for_duration_matches_expected_multiplier() {
        let metadata = sample_metadata(ModelType::Multimodal, 2_147_483_648);
        let reward = PinningManager::reward_for_duration(&metadata, 48);
        assert_eq!(reward, 16);
    }
}
