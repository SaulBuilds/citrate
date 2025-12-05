//! Development Mode Utilities
//!
//! Provides functions for dev mode gating to ensure mock/demo code
//! is never executed in production builds.
//!
//! # Usage
//! ```rust
//! use crate::dev_mode::{is_dev_mode, dev_only, dev_log};
//!
//! // Check dev mode
//! if is_dev_mode() {
//!     println!("Dev mode is enabled");
//! }
//!
//! // Execute only in dev mode
//! dev_only(|| {
//!     enable_mock_data();
//! });
//!
//! // Log only in dev mode
//! dev_log!("Debug message: {}", value);
//! ```

/// Build timestamp from compile time
const BUILD_TIMESTAMP: &str = env!("BUILD_TIMESTAMP");

/// Check if the app is running in development mode.
/// This is determined at compile time via cfg flag.
#[inline]
pub const fn is_dev_mode() -> bool {
    cfg!(dev_mode) || cfg!(debug_assertions)
}

/// Check if the app is running in production mode.
#[inline]
pub const fn is_prod_mode() -> bool {
    !is_dev_mode()
}

/// Get the package version.
pub fn get_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

/// Get the build timestamp.
pub fn get_build_timestamp() -> &'static str {
    BUILD_TIMESTAMP
}

/// Execute a closure only in development mode.
/// In production, this is a no-op and should be optimized away.
#[inline]
pub fn dev_only<F, T>(f: F) -> Option<T>
where
    F: FnOnce() -> T,
{
    if is_dev_mode() {
        Some(f())
    } else {
        None
    }
}

/// Execute different closures based on dev/prod mode.
#[inline]
pub fn dev_or_prod<F1, F2, T>(dev_fn: F1, prod_fn: F2) -> T
where
    F1: FnOnce() -> T,
    F2: FnOnce() -> T,
{
    if is_dev_mode() {
        dev_fn()
    } else {
        prod_fn()
    }
}

/// Log a message only in development mode.
#[macro_export]
macro_rules! dev_log {
    ($($arg:tt)*) => {
        if $crate::dev_mode::is_dev_mode() {
            tracing::debug!("[DEV] {}", format!($($arg)*));
        }
    };
}

/// Log a warning only in development mode.
#[macro_export]
macro_rules! dev_warn {
    ($($arg:tt)*) => {
        if $crate::dev_mode::is_dev_mode() {
            tracing::warn!("[DEV] {}", format!($($arg)*));
        }
    };
}

/// Log an error only in development mode.
#[macro_export]
macro_rules! dev_error {
    ($($arg:tt)*) => {
        if $crate::dev_mode::is_dev_mode() {
            tracing::error!("[DEV] {}", format!($($arg)*));
        }
    };
}

/// Assert a condition only in development mode.
/// In production, this is a no-op.
#[macro_export]
macro_rules! dev_assert {
    ($condition:expr) => {
        if $crate::dev_mode::is_dev_mode() {
            assert!($condition);
        }
    };
    ($condition:expr, $($arg:tt)*) => {
        if $crate::dev_mode::is_dev_mode() {
            assert!($condition, $($arg)*);
        }
    };
}

/// Create a mock value for development.
/// In production, returns the fallback value.
#[inline]
pub fn mock_value<T>(mock_value: T, fallback_value: T) -> T {
    if is_dev_mode() {
        mock_value
    } else {
        fallback_value
    }
}

/// Build info structure
#[derive(Debug, Clone, serde::Serialize)]
pub struct BuildInfo {
    pub version: String,
    pub build_timestamp: String,
    pub is_dev_mode: bool,
    pub environment: &'static str,
}

/// Get complete build information.
pub fn get_build_info() -> BuildInfo {
    BuildInfo {
        version: get_version().to_string(),
        build_timestamp: get_build_timestamp().to_string(),
        is_dev_mode: is_dev_mode(),
        environment: if is_dev_mode() {
            "development"
        } else {
            "production"
        },
    }
}

/// Feature flags for development features.
/// These are all compile-time constants that should be tree-shaken in production.
pub mod features {
    use super::is_dev_mode;

    /// Enable mock blockchain data
    #[inline]
    pub const fn mock_blockchain() -> bool {
        is_dev_mode()
    }

    /// Enable mock AI responses
    #[inline]
    pub const fn mock_ai() -> bool {
        is_dev_mode()
    }

    /// Enable mock IPFS operations
    #[inline]
    pub const fn mock_ipfs() -> bool {
        is_dev_mode()
    }

    /// Enable verbose logging
    #[inline]
    pub const fn verbose_logging() -> bool {
        is_dev_mode()
    }

    /// Enable performance profiling
    #[inline]
    pub const fn profiling() -> bool {
        is_dev_mode()
    }

    /// Enable experimental features
    #[inline]
    pub const fn experimental() -> bool {
        is_dev_mode()
    }
}

/// Print build info to logs (dev mode only)
pub fn print_build_info() {
    if is_dev_mode() {
        let info = get_build_info();
        tracing::info!("🔧 Citrate Build Info:");
        tracing::info!("  Version: {}", info.version);
        tracing::info!("  Build Timestamp: {}", info.build_timestamp);
        tracing::info!("  Environment: {}", info.environment);
        tracing::info!("  Dev Mode: {}", info.is_dev_mode);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_info() {
        let info = get_build_info();
        assert!(!info.version.is_empty());
        assert!(!info.build_timestamp.is_empty());
    }

    #[test]
    fn test_dev_only() {
        let result = dev_only(|| 42);
        // In tests (debug mode), dev_only should execute
        assert!(result.is_some() || is_prod_mode());
    }

    #[test]
    fn test_mock_value() {
        let value = mock_value(100, 0);
        // In tests (debug mode), should return mock value
        assert!(value == 100 || is_prod_mode());
    }
}
