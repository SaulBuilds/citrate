// lattice-v3/core/marketplace/src/performance_tracker.rs

use crate::types::*;
use anyhow::Result;
use chrono::{DateTime, Duration, Utc};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{interval, Duration as TokioDuration};
use tracing::{debug, error, info};

/// Performance tracking configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    pub metrics_retention_days: u64,
    pub sampling_interval_seconds: u64,
    pub benchmark_threshold_ms: u64,
    pub error_rate_threshold: f32,
    pub enable_real_time_monitoring: bool,
    pub alert_thresholds: AlertThresholds,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertThresholds {
    pub high_latency_ms: u64,
    pub high_error_rate: f32,
    pub low_uptime_percentage: f32,
    pub performance_degradation_threshold: f32,
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            metrics_retention_days: 30,
            sampling_interval_seconds: 60,
            benchmark_threshold_ms: 1000,
            error_rate_threshold: 0.05,
            enable_real_time_monitoring: true,
            alert_thresholds: AlertThresholds {
                high_latency_ms: 5000,
                high_error_rate: 0.1,
                low_uptime_percentage: 95.0,
                performance_degradation_threshold: 0.2,
            },
        }
    }
}

/// Real-time performance data point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceDataPoint {
    pub timestamp: DateTime<Utc>,
    pub latency_ms: u64,
    pub success: bool,
    pub error_type: Option<String>,
    pub input_size_bytes: u64,
    pub output_size_bytes: u64,
    pub compute_cost: f32,
    pub user_id: Option<Address>,
}

/// Aggregated performance window
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceWindow {
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub avg_latency_ms: f32,
    pub p95_latency_ms: u64,
    pub p99_latency_ms: u64,
    pub throughput_rps: f32,
    pub error_rate: f32,
    pub total_compute_cost: f32,
    pub unique_users: u64,
}

/// Performance alert
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceAlert {
    pub model_id: ModelId,
    pub alert_type: AlertType,
    pub severity: AlertSeverity,
    pub message: String,
    pub current_value: f32,
    pub threshold: f32,
    pub timestamp: DateTime<Utc>,
    pub resolved: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertType {
    HighLatency,
    HighErrorRate,
    LowUptime,
    PerformanceDegradation,
    ResourceExhaustion,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
}

/// Benchmark result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkResult {
    pub model_id: ModelId,
    pub benchmark_name: String,
    pub dataset: String,
    pub metric_name: String,
    pub score: f32,
    pub baseline_score: Option<f32>,
    pub improvement_percentage: Option<f32>,
    pub hardware_config: String,
    pub test_duration_seconds: u64,
    pub sample_size: u64,
    pub confidence_interval: (f32, f32),
    pub timestamp: DateTime<Utc>,
}

/// Model health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelHealthStatus {
    pub model_id: ModelId,
    pub overall_health: HealthLevel,
    pub uptime_percentage: f32,
    pub current_latency_ms: u64,
    pub current_error_rate: f32,
    pub performance_trend: PerformanceTrend,
    pub active_alerts: Vec<PerformanceAlert>,
    pub last_benchmark: Option<DateTime<Utc>>,
    pub health_score: f32, // 0.0 to 1.0
    pub last_updated: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthLevel {
    Excellent,
    Good,
    Fair,
    Poor,
    Critical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PerformanceTrend {
    Improving,
    Stable,
    Degrading,
}

/// Performance tracker
pub struct PerformanceTracker {
    config: PerformanceConfig,
    real_time_data: Arc<DashMap<ModelId, VecDeque<PerformanceDataPoint>>>,
    performance_windows: Arc<DashMap<ModelId, VecDeque<PerformanceWindow>>>,
    benchmark_results: Arc<DashMap<ModelId, Vec<BenchmarkResult>>>,
    active_alerts: Arc<DashMap<ModelId, Vec<PerformanceAlert>>>,
    model_health: Arc<DashMap<ModelId, ModelHealthStatus>>,
    alert_history: Arc<RwLock<VecDeque<PerformanceAlert>>>,
}

impl PerformanceTracker {
    /// Create a new performance tracker
    pub fn new(config: PerformanceConfig) -> Self {
        Self {
            config,
            real_time_data: Arc::new(DashMap::new()),
            performance_windows: Arc::new(DashMap::new()),
            benchmark_results: Arc::new(DashMap::new()),
            active_alerts: Arc::new(DashMap::new()),
            model_health: Arc::new(DashMap::new()),
            alert_history: Arc::new(RwLock::new(VecDeque::new())),
        }
    }

    /// Start background monitoring
    pub async fn start_monitoring(&self) -> Result<()> {
        if !self.config.enable_real_time_monitoring {
            return Ok(());
        }

        let real_time_data = Arc::clone(&self.real_time_data);
        let performance_windows = Arc::clone(&self.performance_windows);
        let model_health = Arc::clone(&self.model_health);
        let config = self.config.clone();

        tokio::spawn(async move {
            let mut interval = interval(TokioDuration::from_secs(config.sampling_interval_seconds));

            info!("Performance monitoring started");

            loop {
                interval.tick().await;

                if let Err(e) = Self::aggregate_performance_data(
                    Arc::clone(&real_time_data),
                    Arc::clone(&performance_windows),
                    Arc::clone(&model_health),
                    &config,
                ).await {
                    error!(error = %e, "Failed to aggregate performance data");
                }
            }
        });

        info!("Performance tracker initialized");
        Ok(())
    }

    /// Record a performance data point
    pub async fn record_performance(&self, model_id: &ModelId, data_point: PerformanceDataPoint) -> Result<()> {
        // Add to real-time data
        let mut entry = self.real_time_data.entry(*model_id).or_insert_with(VecDeque::new);
        entry.push_back(data_point.clone());

        // Limit data retention
        let cutoff_time = Utc::now() - Duration::days(self.config.metrics_retention_days as i64);
        while let Some(front) = entry.front() {
            if front.timestamp < cutoff_time {
                entry.pop_front();
            } else {
                break;
            }
        }

        // Check for immediate alerts
        self.check_immediate_alerts(model_id, &data_point).await?;

        debug!(
            model_id = ?model_id,
            latency_ms = data_point.latency_ms,
            success = data_point.success,
            "Performance data recorded"
        );

        Ok(())
    }

    /// Submit benchmark results
    pub async fn submit_benchmark(&self, result: BenchmarkResult) -> Result<()> {
        let mut benchmarks = self.benchmark_results
            .entry(result.model_id)
            .or_insert_with(Vec::new);

        benchmarks.push(result.clone());

        // Keep only recent benchmarks
        let cutoff_time = Utc::now() - Duration::days(self.config.metrics_retention_days as i64);
        benchmarks.retain(|b| b.timestamp > cutoff_time);

        info!(
            model_id = ?result.model_id,
            benchmark = %result.benchmark_name,
            score = result.score,
            "Benchmark result submitted"
        );

        Ok(())
    }

    /// Get model health status
    pub async fn get_model_health(&self, model_id: &ModelId) -> Option<ModelHealthStatus> {
        self.model_health.get(model_id).map(|entry| entry.value().clone())
    }

    /// Get performance metrics for a time range
    pub async fn get_performance_metrics(
        &self,
        model_id: &ModelId,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
    ) -> Result<Vec<PerformanceWindow>> {
        let windows = self.performance_windows
            .get(model_id)
            .map(|entry| {
                entry.iter()
                    .filter(|w| w.start_time >= start_time && w.end_time <= end_time)
                    .cloned()
                    .collect()
            })
            .unwrap_or_default();

        Ok(windows)
    }

    /// Get latest benchmark results
    pub async fn get_benchmark_results(&self, model_id: &ModelId, limit: usize) -> Vec<BenchmarkResult> {
        self.benchmark_results
            .get(model_id)
            .map(|entry| {
                let mut results = entry.clone();
                results.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
                results.truncate(limit);
                results
            })
            .unwrap_or_default()
    }

    /// Get active alerts for a model
    pub async fn get_active_alerts(&self, model_id: &ModelId) -> Vec<PerformanceAlert> {
        self.active_alerts
            .get(model_id)
            .map(|entry| entry.clone())
            .unwrap_or_default()
    }

    /// Get performance summary for all models
    pub async fn get_performance_summary(&self) -> HashMap<ModelId, ModelHealthStatus> {
        self.model_health
            .iter()
            .map(|entry| (*entry.key(), entry.value().clone()))
            .collect()
    }

    /// Resolve an alert
    pub async fn resolve_alert(&self, model_id: &ModelId, alert_type: AlertType) -> Result<()> {
        if let Some(mut alerts) = self.active_alerts.get_mut(model_id) {
            for alert in alerts.iter_mut() {
                if std::mem::discriminant(&alert.alert_type) == std::mem::discriminant(&alert_type) && !alert.resolved {
                    alert.resolved = true;
                    info!(
                        model_id = ?model_id,
                        alert_type = ?alert_type,
                        "Alert resolved"
                    );
                }
            }
        }

        Ok(())
    }

    // Private helper methods

    async fn aggregate_performance_data(
        real_time_data: Arc<DashMap<ModelId, VecDeque<PerformanceDataPoint>>>,
        performance_windows: Arc<DashMap<ModelId, VecDeque<PerformanceWindow>>>,
        model_health: Arc<DashMap<ModelId, ModelHealthStatus>>,
        config: &PerformanceConfig,
    ) -> Result<()> {
        let window_duration = Duration::seconds(config.sampling_interval_seconds as i64);
        let current_time = Utc::now();
        let window_start = current_time - window_duration;

        for entry in real_time_data.iter() {
            let model_id = *entry.key();
            let data_points = entry.value();

            // Filter data points for current window
            let window_points: Vec<&PerformanceDataPoint> = data_points
                .iter()
                .filter(|dp| dp.timestamp >= window_start && dp.timestamp <= current_time)
                .collect();

            if window_points.is_empty() {
                continue;
            }

            // Calculate aggregated metrics
            let total_requests = window_points.len() as u64;
            let successful_requests = window_points.iter().filter(|dp| dp.success).count() as u64;
            let failed_requests = total_requests - successful_requests;

            let latencies: Vec<u64> = window_points.iter().map(|dp| dp.latency_ms).collect();
            let avg_latency_ms = latencies.iter().sum::<u64>() as f32 / latencies.len() as f32;

            let mut sorted_latencies = latencies.clone();
            sorted_latencies.sort();
            let p95_latency_ms = Self::percentile(&sorted_latencies, 0.95);
            let p99_latency_ms = Self::percentile(&sorted_latencies, 0.99);

            let throughput_rps = total_requests as f32 / window_duration.num_seconds() as f32;
            let error_rate = failed_requests as f32 / total_requests as f32;

            let total_compute_cost = window_points.iter().map(|dp| dp.compute_cost).sum();
            let unique_users = window_points
                .iter()
                .filter_map(|dp| dp.user_id.as_ref())
                .collect::<std::collections::HashSet<_>>()
                .len() as u64;

            let window = PerformanceWindow {
                start_time: window_start,
                end_time: current_time,
                total_requests,
                successful_requests,
                failed_requests,
                avg_latency_ms,
                p95_latency_ms,
                p99_latency_ms,
                throughput_rps,
                error_rate,
                total_compute_cost,
                unique_users,
            };

            // Store performance window
            let mut windows = performance_windows.entry(model_id).or_insert_with(VecDeque::new);
            windows.push_back(window.clone());

            // Limit window retention
            let cutoff_time = current_time - Duration::days(config.metrics_retention_days as i64);
            while let Some(front) = windows.front() {
                if front.start_time < cutoff_time {
                    windows.pop_front();
                } else {
                    break;
                }
            }

            // Update model health
            Self::update_model_health(&model_id, &window, &windows, &model_health, config).await;
        }

        Ok(())
    }

    fn percentile(sorted_values: &[u64], percentile: f64) -> u64 {
        if sorted_values.is_empty() {
            return 0;
        }

        let index = (percentile * (sorted_values.len() - 1) as f64) as usize;
        sorted_values[index]
    }

    async fn update_model_health(
        model_id: &ModelId,
        current_window: &PerformanceWindow,
        windows: &VecDeque<PerformanceWindow>,
        model_health: &Arc<DashMap<ModelId, ModelHealthStatus>>,
        config: &PerformanceConfig,
    ) {
        // Calculate uptime percentage over recent windows
        let recent_windows: Vec<&PerformanceWindow> = windows
            .iter()
            .rev()
            .take(10) // Last 10 windows
            .collect();

        let total_requests: u64 = recent_windows.iter().map(|w| w.total_requests).sum();
        let successful_requests: u64 = recent_windows.iter().map(|w| w.successful_requests).sum();
        let uptime_percentage = if total_requests > 0 {
            (successful_requests as f32 / total_requests as f32) * 100.0
        } else {
            100.0
        };

        // Determine performance trend
        let trend = if recent_windows.len() >= 3 {
            let recent_avg = recent_windows.iter().take(3).map(|w| w.avg_latency_ms).sum::<f32>() / 3.0;
            let older_avg = recent_windows.iter().skip(3).take(3).map(|w| w.avg_latency_ms).sum::<f32>() / 3.0;

            if recent_avg < older_avg * 0.9 {
                PerformanceTrend::Improving
            } else if recent_avg > older_avg * 1.1 {
                PerformanceTrend::Degrading
            } else {
                PerformanceTrend::Stable
            }
        } else {
            PerformanceTrend::Stable
        };

        // Calculate health score
        let latency_score = (1.0 - (current_window.avg_latency_ms / config.alert_thresholds.high_latency_ms as f32).min(1.0)).max(0.0);
        let error_score = (1.0 - (current_window.error_rate / config.alert_thresholds.high_error_rate).min(1.0)).max(0.0);
        let uptime_score = (uptime_percentage / 100.0).max(0.0);

        let health_score = (latency_score * 0.4 + error_score * 0.3 + uptime_score * 0.3).max(0.0).min(1.0);

        // Determine overall health level
        let overall_health = match health_score {
            s if s >= 0.9 => HealthLevel::Excellent,
            s if s >= 0.8 => HealthLevel::Good,
            s if s >= 0.6 => HealthLevel::Fair,
            s if s >= 0.4 => HealthLevel::Poor,
            _ => HealthLevel::Critical,
        };

        let health_status = ModelHealthStatus {
            model_id: *model_id,
            overall_health,
            uptime_percentage,
            current_latency_ms: current_window.avg_latency_ms as u64,
            current_error_rate: current_window.error_rate,
            performance_trend: trend,
            active_alerts: Vec::new(), // Will be populated separately
            last_benchmark: None, // Will be updated when benchmarks are run
            health_score,
            last_updated: Utc::now(),
        };

        model_health.insert(*model_id, health_status);
    }

    async fn check_immediate_alerts(&self, model_id: &ModelId, data_point: &PerformanceDataPoint) -> Result<()> {
        let mut alerts = Vec::new();

        // High latency alert
        if data_point.latency_ms > self.config.alert_thresholds.high_latency_ms {
            alerts.push(PerformanceAlert {
                model_id: *model_id,
                alert_type: AlertType::HighLatency,
                severity: if data_point.latency_ms > self.config.alert_thresholds.high_latency_ms * 2 {
                    AlertSeverity::Critical
                } else {
                    AlertSeverity::Warning
                },
                message: format!("High latency detected: {}ms", data_point.latency_ms),
                current_value: data_point.latency_ms as f32,
                threshold: self.config.alert_thresholds.high_latency_ms as f32,
                timestamp: data_point.timestamp,
                resolved: false,
            });
        }

        // Error alert
        if !data_point.success {
            alerts.push(PerformanceAlert {
                model_id: *model_id,
                alert_type: AlertType::HighErrorRate,
                severity: AlertSeverity::Warning,
                message: format!("Request failed: {:?}", data_point.error_type),
                current_value: 1.0,
                threshold: 0.0,
                timestamp: data_point.timestamp,
                resolved: false,
            });
        }

        // Store active alerts
        if !alerts.is_empty() {
            let mut model_alerts = self.active_alerts.entry(*model_id).or_insert_with(Vec::new);
            model_alerts.extend(alerts.clone());

            // Store in alert history
            let mut history = self.alert_history.write().await;
            history.extend(alerts);

            // Limit history size
            if history.len() > 10000 {
                while history.len() > 8000 {
                    history.pop_front();
                }
            }
        }

        Ok(())
    }
}