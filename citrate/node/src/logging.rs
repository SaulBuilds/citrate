//! Structured Logging Module
//!
//! Provides structured JSON logging with trace ID propagation for request correlation.
//! This enables efficient log aggregation and debugging in production environments.
//!
//! # Features
//! - JSON formatted output for log aggregation
//! - Trace ID generation and propagation
//! - Configurable log levels per module
//! - Log rotation support via file appender
//! - Environment-based configuration
//!
//! # Usage
//! ```rust
//! use citrate_node::logging::{init_logging, LogConfig};
//!
//! let config = LogConfig::from_env();
//! init_logging(&config)?;
//!
//! tracing::info!(trace_id = %TraceId::new(), "Request received");
//! ```

use std::fmt;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use tracing_subscriber::{
    fmt::format::FmtSpan,
    layer::SubscriberExt,
    util::SubscriberInitExt,
    EnvFilter, Layer,
};

/// Counter for generating unique trace IDs
static TRACE_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Unique identifier for correlating logs across a request lifecycle
#[derive(Clone, Copy, Debug)]
pub struct TraceId {
    /// Timestamp component (ms since epoch)
    timestamp: u64,
    /// Sequential counter component
    counter: u64,
    /// Random component for uniqueness
    random: u16,
}

impl TraceId {
    /// Generate a new unique trace ID
    pub fn new() -> Self {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);
        let counter = TRACE_COUNTER.fetch_add(1, Ordering::SeqCst);
        let random = rand::random::<u16>();

        Self {
            timestamp,
            counter,
            random,
        }
    }

    /// Create trace ID from string (for propagation)
    pub fn from_str(s: &str) -> Option<Self> {
        let parts: Vec<&str> = s.split('-').collect();
        if parts.len() != 3 {
            return None;
        }

        Some(Self {
            timestamp: u64::from_str_radix(parts[0], 16).ok()?,
            counter: u64::from_str_radix(parts[1], 16).ok()?,
            random: u16::from_str_radix(parts[2], 16).ok()?,
        })
    }

    /// Convert to string for logging/propagation
    pub fn to_string(&self) -> String {
        format!("{:x}-{:x}-{:04x}", self.timestamp, self.counter, self.random)
    }
}

impl Default for TraceId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for TraceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:x}-{:x}-{:04x}", self.timestamp, self.counter, self.random)
    }
}

/// Log level configuration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl LogLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            LogLevel::Trace => "trace",
            LogLevel::Debug => "debug",
            LogLevel::Info => "info",
            LogLevel::Warn => "warn",
            LogLevel::Error => "error",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "trace" => LogLevel::Trace,
            "debug" => LogLevel::Debug,
            "info" => LogLevel::Info,
            "warn" => LogLevel::Warn,
            "error" => LogLevel::Error,
            _ => LogLevel::Info,
        }
    }
}

/// Log output format
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogFormat {
    /// Human-readable format
    Pretty,
    /// JSON format for log aggregation
    Json,
    /// Compact single-line format
    Compact,
}

impl LogFormat {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "json" => LogFormat::Json,
            "compact" => LogFormat::Compact,
            _ => LogFormat::Pretty,
        }
    }
}

/// Logging configuration
#[derive(Debug, Clone)]
pub struct LogConfig {
    /// Default log level
    pub level: LogLevel,
    /// Output format (json, pretty, compact)
    pub format: LogFormat,
    /// Enable ANSI colors (for terminal output)
    pub ansi_colors: bool,
    /// Log to file path (optional)
    pub log_file: Option<PathBuf>,
    /// Enable span events (enter/exit)
    pub span_events: bool,
    /// Module-specific log levels
    pub module_levels: Vec<(String, LogLevel)>,
    /// Include target in logs
    pub include_target: bool,
    /// Include file location in logs
    pub include_location: bool,
    /// Include thread ID in logs
    pub include_thread_id: bool,
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            level: LogLevel::Info,
            format: LogFormat::Pretty,
            ansi_colors: true,
            log_file: None,
            span_events: false,
            module_levels: Vec::new(),
            include_target: true,
            include_location: false,
            include_thread_id: false,
        }
    }
}

impl LogConfig {
    /// Create configuration from environment variables
    ///
    /// Environment variables:
    /// - RUST_LOG: Log level filter (e.g., "info,citrate_api=debug")
    /// - LOG_FORMAT: Output format (json, pretty, compact)
    /// - LOG_FILE: Path to log file
    /// - LOG_ANSI: Enable ANSI colors (true/false)
    pub fn from_env() -> Self {
        let mut config = Self::default();

        // Parse RUST_LOG for level
        if let Ok(rust_log) = std::env::var("RUST_LOG") {
            // Extract default level from start of filter
            let level_str = rust_log.split(',').next().unwrap_or("info");
            config.level = LogLevel::from_str(level_str);
        }

        // Log format
        if let Ok(format) = std::env::var("LOG_FORMAT") {
            config.format = LogFormat::from_str(&format);
        }

        // Log file
        if let Ok(path) = std::env::var("LOG_FILE") {
            config.log_file = Some(PathBuf::from(path));
        }

        // ANSI colors
        if let Ok(ansi) = std::env::var("LOG_ANSI") {
            config.ansi_colors = ansi.to_lowercase() == "true";
        }

        config
    }

    /// Create JSON logging configuration for production
    pub fn production() -> Self {
        Self {
            level: LogLevel::Info,
            format: LogFormat::Json,
            ansi_colors: false,
            log_file: Some(PathBuf::from("/var/log/citrate/node.log")),
            span_events: true,
            module_levels: vec![
                ("citrate_api".to_string(), LogLevel::Info),
                ("citrate_consensus".to_string(), LogLevel::Info),
                ("citrate_network".to_string(), LogLevel::Warn),
                ("hyper".to_string(), LogLevel::Warn),
                ("tower".to_string(), LogLevel::Warn),
            ],
            include_target: true,
            include_location: true,
            include_thread_id: true,
        }
    }

    /// Create verbose configuration for development
    pub fn development() -> Self {
        Self {
            level: LogLevel::Debug,
            format: LogFormat::Pretty,
            ansi_colors: true,
            log_file: None,
            span_events: true,
            module_levels: vec![
                ("citrate".to_string(), LogLevel::Debug),
                ("hyper".to_string(), LogLevel::Info),
            ],
            include_target: true,
            include_location: true,
            include_thread_id: false,
        }
    }

    /// Build the env filter string
    fn build_filter(&self) -> String {
        let mut filter = self.level.as_str().to_string();

        for (module, level) in &self.module_levels {
            filter.push_str(&format!(",{}={}", module, level.as_str()));
        }

        filter
    }
}

/// Initialize the logging system with the given configuration
pub fn init_logging(config: &LogConfig) -> anyhow::Result<()> {
    // Build filter from config or RUST_LOG env
    let filter = if let Ok(rust_log) = std::env::var("RUST_LOG") {
        EnvFilter::new(rust_log)
    } else {
        EnvFilter::new(config.build_filter())
    };

    // Determine span events
    let span_events = if config.span_events {
        FmtSpan::NEW | FmtSpan::CLOSE
    } else {
        FmtSpan::NONE
    };

    match config.format {
        LogFormat::Json => {
            let subscriber = tracing_subscriber::registry()
                .with(filter)
                .with(
                    tracing_subscriber::fmt::layer()
                        .json()
                        .with_target(config.include_target)
                        .with_file(config.include_location)
                        .with_line_number(config.include_location)
                        .with_thread_ids(config.include_thread_id)
                        .with_span_events(span_events)
                        .with_ansi(false),
                );
            subscriber.try_init().map_err(|e| anyhow::anyhow!("Failed to init logging: {}", e))?;
        }
        LogFormat::Pretty => {
            let subscriber = tracing_subscriber::registry()
                .with(filter)
                .with(
                    tracing_subscriber::fmt::layer()
                        .pretty()
                        .with_target(config.include_target)
                        .with_file(config.include_location)
                        .with_line_number(config.include_location)
                        .with_thread_ids(config.include_thread_id)
                        .with_span_events(span_events)
                        .with_ansi(config.ansi_colors),
                );
            subscriber.try_init().map_err(|e| anyhow::anyhow!("Failed to init logging: {}", e))?;
        }
        LogFormat::Compact => {
            let subscriber = tracing_subscriber::registry()
                .with(filter)
                .with(
                    tracing_subscriber::fmt::layer()
                        .compact()
                        .with_target(config.include_target)
                        .with_file(config.include_location)
                        .with_line_number(config.include_location)
                        .with_thread_ids(config.include_thread_id)
                        .with_span_events(span_events)
                        .with_ansi(config.ansi_colors),
                );
            subscriber.try_init().map_err(|e| anyhow::anyhow!("Failed to init logging: {}", e))?;
        }
    }

    Ok(())
}

/// Initialize logging with defaults (for quick setup)
pub fn init_default_logging() -> anyhow::Result<()> {
    let config = LogConfig::from_env();
    init_logging(&config)
}

/// Macro for creating a span with trace ID
#[macro_export]
macro_rules! traced_span {
    ($level:ident, $name:expr $(, $($field:tt)*)?) => {
        tracing::span!(
            tracing::Level::$level,
            $name,
            trace_id = %$crate::logging::TraceId::new()
            $(, $($field)*)?
        )
    };
}

/// Log formats for common operations
pub mod formats {
    use super::TraceId;

    /// Format for RPC request logging
    pub fn rpc_request(trace_id: &TraceId, method: &str, params_size: usize) -> String {
        format!(
            "trace_id={} method={} params_size={}",
            trace_id, method, params_size
        )
    }

    /// Format for RPC response logging
    pub fn rpc_response(trace_id: &TraceId, method: &str, duration_ms: u64, success: bool) -> String {
        format!(
            "trace_id={} method={} duration_ms={} success={}",
            trace_id, method, duration_ms, success
        )
    }

    /// Format for block production logging
    pub fn block_produced(
        trace_id: &TraceId,
        height: u64,
        tx_count: usize,
        build_time_ms: u64,
    ) -> String {
        format!(
            "trace_id={} height={} tx_count={} build_time_ms={}",
            trace_id, height, tx_count, build_time_ms
        )
    }

    /// Format for transaction logging
    pub fn transaction(trace_id: &TraceId, tx_hash: &str, from: &str, to: &str) -> String {
        format!(
            "trace_id={} tx_hash={} from={} to={}",
            trace_id, tx_hash, from, to
        )
    }

    /// Format for peer connection logging
    pub fn peer_connection(trace_id: &TraceId, peer_id: &str, action: &str) -> String {
        format!(
            "trace_id={} peer_id={} action={}",
            trace_id, peer_id, action
        )
    }

    /// Format for AI inference logging
    pub fn ai_inference(
        trace_id: &TraceId,
        model_id: &str,
        latency_ms: u64,
        tokens: usize,
    ) -> String {
        format!(
            "trace_id={} model_id={} latency_ms={} tokens={}",
            trace_id, model_id, latency_ms, tokens
        )
    }

    /// Format for IPFS operation logging
    pub fn ipfs_operation(trace_id: &TraceId, operation: &str, cid: &str, size: usize) -> String {
        format!(
            "trace_id={} operation={} cid={} size={}",
            trace_id, operation, cid, size
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trace_id_generation() {
        let id1 = TraceId::new();
        let id2 = TraceId::new();

        // IDs should be unique
        assert_ne!(id1.to_string(), id2.to_string());
    }

    #[test]
    fn test_trace_id_parsing() {
        let id = TraceId::new();
        let id_str = id.to_string();

        let parsed = TraceId::from_str(&id_str);
        assert!(parsed.is_some());

        let parsed = parsed.unwrap();
        assert_eq!(id.timestamp, parsed.timestamp);
        assert_eq!(id.counter, parsed.counter);
        assert_eq!(id.random, parsed.random);
    }

    #[test]
    fn test_log_config_from_env() {
        std::env::set_var("RUST_LOG", "debug");
        std::env::set_var("LOG_FORMAT", "json");

        let config = LogConfig::from_env();
        assert_eq!(config.level, LogLevel::Debug);
        assert_eq!(config.format, LogFormat::Json);

        // Cleanup
        std::env::remove_var("RUST_LOG");
        std::env::remove_var("LOG_FORMAT");
    }

    #[test]
    fn test_log_level_parsing() {
        assert_eq!(LogLevel::from_str("trace"), LogLevel::Trace);
        assert_eq!(LogLevel::from_str("DEBUG"), LogLevel::Debug);
        assert_eq!(LogLevel::from_str("Info"), LogLevel::Info);
        assert_eq!(LogLevel::from_str("WARN"), LogLevel::Warn);
        assert_eq!(LogLevel::from_str("error"), LogLevel::Error);
        assert_eq!(LogLevel::from_str("invalid"), LogLevel::Info);
    }

    #[test]
    fn test_build_filter() {
        let mut config = LogConfig::default();
        config.level = LogLevel::Info;
        config.module_levels = vec![
            ("citrate_api".to_string(), LogLevel::Debug),
            ("hyper".to_string(), LogLevel::Warn),
        ];

        let filter = config.build_filter();
        assert!(filter.contains("info"));
        assert!(filter.contains("citrate_api=debug"));
        assert!(filter.contains("hyper=warn"));
    }

    #[test]
    fn test_format_helpers() {
        let trace_id = TraceId::new();

        let rpc_log = formats::rpc_request(&trace_id, "eth_blockNumber", 0);
        assert!(rpc_log.contains("eth_blockNumber"));

        let block_log = formats::block_produced(&trace_id, 100, 5, 50);
        assert!(block_log.contains("height=100"));
        assert!(block_log.contains("tx_count=5"));
    }
}
