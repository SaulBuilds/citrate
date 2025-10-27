// citrate/core/economics/src/dynamic_pricing.rs

use primitive_types::U256;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use anyhow::Result;

/// Dynamic pricing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DynamicPricingConfig {
    /// Base gas price in wei
    pub base_gas_price: U256,

    /// Target block utilization (0-100)
    pub target_utilization: u8,

    /// Maximum gas price multiplier
    pub max_price_multiplier: u32,

    /// Minimum gas price multiplier (fraction)
    pub min_price_multiplier: u32, // e.g., 50 = 0.5x

    /// Adjustment factor (how quickly prices change)
    pub adjustment_factor: u32, // basis points (100 = 1%)

    /// Window size for utilization calculation
    pub utilization_window: usize,

    /// AI inference base cost multiplier
    pub ai_inference_multiplier: u32,

    /// Model deployment base cost
    pub model_deployment_base: U256,

    /// Compute intensity scaling factor
    pub compute_scaling_factor: u32,
}

impl Default for DynamicPricingConfig {
    fn default() -> Self {
        Self {
            base_gas_price: U256::from(1_000_000_000), // 1 Gwei
            target_utilization: 70, // 70% target
            max_price_multiplier: 50, // 50x max
            min_price_multiplier: 10, // 0.1x min
            adjustment_factor: 125, // 1.25% per block
            utilization_window: 20, // 20 blocks
            ai_inference_multiplier: 200, // 2x for AI ops
            model_deployment_base: U256::from(100) * U256::from(10).pow(U256::from(18)), // 100 LATT
            compute_scaling_factor: 150, // 1.5x scaling per compute unit
        }
    }
}

/// Network utilization metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UtilizationMetrics {
    pub block_height: u64,
    pub gas_used: u64,
    pub gas_limit: u64,
    pub transaction_count: u32,
    pub ai_operations: u32,
    pub compute_intensity: f64, // 0.0 to 1.0
}

/// Dynamic pricing manager
pub struct DynamicPricingManager {
    config: DynamicPricingConfig,
    current_gas_price: U256,
    utilization_history: VecDeque<UtilizationMetrics>,
    price_history: VecDeque<(u64, U256)>, // (block, price)
}

impl DynamicPricingManager {
    pub fn new(config: DynamicPricingConfig) -> Self {
        let current_gas_price = config.base_gas_price;

        Self {
            config,
            current_gas_price,
            utilization_history: VecDeque::new(),
            price_history: VecDeque::new(),
        }
    }

    /// Update pricing based on latest block metrics
    pub fn update_pricing(&mut self, metrics: UtilizationMetrics) -> Result<PricingUpdate> {
        // Add to history
        self.utilization_history.push_back(metrics.clone());
        if self.utilization_history.len() > self.config.utilization_window {
            self.utilization_history.pop_front();
        }

        // Calculate average utilization
        let avg_utilization = self.calculate_average_utilization();

        // Calculate new gas price
        let new_gas_price = self.calculate_new_gas_price(avg_utilization, &metrics)?;

        // Update price history
        self.price_history.push_back((metrics.block_height, new_gas_price));
        if self.price_history.len() > 100 { // Keep last 100 blocks
            self.price_history.pop_front();
        }

        let price_change = if new_gas_price > self.current_gas_price {
            PriceChange::Increase {
                old_price: self.current_gas_price,
                new_price: new_gas_price,
                factor: self.calculate_change_factor(self.current_gas_price, new_gas_price),
            }
        } else if new_gas_price < self.current_gas_price {
            PriceChange::Decrease {
                old_price: self.current_gas_price,
                new_price: new_gas_price,
                factor: self.calculate_change_factor(self.current_gas_price, new_gas_price),
            }
        } else {
            PriceChange::NoChange(self.current_gas_price)
        };

        self.current_gas_price = new_gas_price;

        Ok(PricingUpdate {
            block_height: metrics.block_height,
            utilization: avg_utilization,
            price_change,
            ai_operation_cost: self.calculate_ai_operation_cost(&metrics),
            model_deployment_cost: self.calculate_model_deployment_cost(&metrics),
        })
    }

    /// Calculate dynamic gas price based on network utilization
    fn calculate_new_gas_price(&self, utilization: f64, metrics: &UtilizationMetrics) -> Result<U256> {
        let target = self.config.target_utilization as f64 / 100.0;
        let utilization_ratio = utilization / target;

        // Calculate adjustment based on deviation from target
        let adjustment = if utilization_ratio > 1.0 {
            // Above target - increase price
            let excess = utilization_ratio - 1.0;
            1.0 + (excess * self.config.adjustment_factor as f64 / 10000.0)
        } else {
            // Below target - decrease price
            let deficit = 1.0 - utilization_ratio;
            1.0 - (deficit * self.config.adjustment_factor as f64 / 10000.0)
        };

        // Apply AI workload scaling
        let ai_scaling = if metrics.ai_operations > 0 {
            1.0 + (metrics.compute_intensity * self.config.compute_scaling_factor as f64 / 100.0)
        } else {
            1.0
        };

        // Calculate new price
        let base_price_f64 = self.current_gas_price.as_u128() as f64;
        let new_price_f64 = base_price_f64 * adjustment * ai_scaling;

        // Apply bounds
        let max_price = self.config.base_gas_price.as_u128() as f64 * self.config.max_price_multiplier as f64;
        let min_price = self.config.base_gas_price.as_u128() as f64 * self.config.min_price_multiplier as f64 / 100.0;

        let bounded_price = new_price_f64.min(max_price).max(min_price);

        Ok(U256::from(bounded_price as u128))
    }

    /// Calculate average utilization over window
    fn calculate_average_utilization(&self) -> f64 {
        if self.utilization_history.is_empty() {
            return 0.0;
        }

        let total_utilization: f64 = self.utilization_history
            .iter()
            .map(|m| m.gas_used as f64 / m.gas_limit as f64)
            .sum();

        total_utilization / self.utilization_history.len() as f64
    }

    /// Calculate cost for AI operations
    fn calculate_ai_operation_cost(&self, metrics: &UtilizationMetrics) -> U256 {
        let base_cost = self.current_gas_price * U256::from(21_000); // Base transaction cost
        let ai_multiplier = U256::from(self.config.ai_inference_multiplier);
        let compute_scaling = U256::from((100.0 + metrics.compute_intensity * 100.0) as u64);

        base_cost * ai_multiplier * compute_scaling / U256::from(10_000)
    }

    /// Calculate cost for model deployment
    fn calculate_model_deployment_cost(&self, metrics: &UtilizationMetrics) -> U256 {
        let base_deployment = self.config.model_deployment_base;
        let network_factor = U256::from((100.0 + metrics.compute_intensity * 50.0) as u64);

        base_deployment * network_factor / U256::from(100)
    }

    /// Calculate price change factor
    fn calculate_change_factor(&self, old_price: U256, new_price: U256) -> f64 {
        if old_price.is_zero() {
            return 1.0;
        }

        new_price.as_u128() as f64 / old_price.as_u128() as f64
    }

    /// Get current gas price
    pub fn current_gas_price(&self) -> U256 {
        self.current_gas_price
    }

    /// Get price for specific operation type
    pub fn get_operation_price(&self, operation: OperationType) -> U256 {
        match operation {
            OperationType::StandardTransaction => self.current_gas_price * U256::from(21_000),
            OperationType::ContractCall => self.current_gas_price * U256::from(50_000),
            OperationType::AIInference { compute_units } => {
                let base = self.current_gas_price * U256::from(100_000);
                let scaling = U256::from(compute_units) * U256::from(self.config.compute_scaling_factor) / U256::from(100);
                base + (base * scaling / U256::from(100))
            },
            OperationType::ModelDeployment { model_size_mb } => {
                let base = self.config.model_deployment_base;
                let size_scaling = U256::from(model_size_mb) * U256::from(10).pow(U256::from(16)); // 0.01 LATT per MB
                base + size_scaling
            },
            OperationType::ModelTraining { dataset_size_gb } => {
                let base = self.current_gas_price * U256::from(1_000_000);
                let data_scaling = U256::from(dataset_size_gb) * U256::from(10).pow(U256::from(17)); // 0.1 LATT per GB
                base + data_scaling
            },
        }
    }

    /// Get pricing history for analysis
    pub fn get_price_history(&self, blocks: usize) -> Vec<(u64, U256)> {
        self.price_history
            .iter()
            .rev()
            .take(blocks)
            .cloned()
            .collect()
    }

    /// Predict future pricing based on trends
    pub fn predict_price_trend(&self, _blocks_ahead: u64) -> PriceTrend {
        if self.price_history.len() < 5 {
            return PriceTrend::Stable;
        }

        let recent_prices: Vec<f64> = self.price_history
            .iter()
            .rev()
            .take(10)
            .map(|(_, price)| price.as_u128() as f64)
            .collect();

        // Simple linear regression for trend
        let n = recent_prices.len() as f64;
        let sum_x: f64 = (0..recent_prices.len()).map(|i| i as f64).sum();
        let sum_y: f64 = recent_prices.iter().sum();
        let sum_xy: f64 = recent_prices.iter().enumerate()
            .map(|(i, &y)| i as f64 * y)
            .sum();
        let sum_x2: f64 = (0..recent_prices.len()).map(|i| (i as f64).powi(2)).sum();

        let slope = (n * sum_xy - sum_x * sum_y) / (n * sum_x2 - sum_x.powi(2));

        if slope > 0.01 {
            PriceTrend::Increasing
        } else if slope < -0.01 {
            PriceTrend::Decreasing
        } else {
            PriceTrend::Stable
        }
    }
}

#[derive(Debug, Clone)]
pub struct PricingUpdate {
    pub block_height: u64,
    pub utilization: f64,
    pub price_change: PriceChange,
    pub ai_operation_cost: U256,
    pub model_deployment_cost: U256,
}

#[derive(Debug, Clone)]
pub enum PriceChange {
    Increase { old_price: U256, new_price: U256, factor: f64 },
    Decrease { old_price: U256, new_price: U256, factor: f64 },
    NoChange(U256),
}

#[derive(Debug, Clone)]
pub enum OperationType {
    StandardTransaction,
    ContractCall,
    AIInference { compute_units: u64 },
    ModelDeployment { model_size_mb: u64 },
    ModelTraining { dataset_size_gb: u64 },
}

#[derive(Debug, Clone)]
pub enum PriceTrend {
    Increasing,
    Decreasing,
    Stable,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dynamic_pricing_update() {
        let config = DynamicPricingConfig::default();
        let mut pricing = DynamicPricingManager::new(config.clone());

        let metrics = UtilizationMetrics {
            block_height: 1,
            gas_used: 8_000_000,
            gas_limit: 10_000_000,
            transaction_count: 100,
            ai_operations: 10,
            compute_intensity: 0.5,
        };

        let update = pricing.update_pricing(metrics).unwrap();
        assert!(update.utilization > 0.0);
    }

    #[test]
    fn test_operation_pricing() {
        let config = DynamicPricingConfig::default();
        let pricing = DynamicPricingManager::new(config);

        let standard_cost = pricing.get_operation_price(OperationType::StandardTransaction);
        let ai_cost = pricing.get_operation_price(OperationType::AIInference { compute_units: 100 });

        assert!(ai_cost > standard_cost);
    }
}