/**
 * Metrics Collector Module
 *
 * Re-exports MetricsTracker as MetricsCollector for backwards compatibility
 * with integration tests.
 */

import { MetricsTracker } from './metricsTracker';

// Re-export MetricsTracker as MetricsCollector for backwards compatibility
export { MetricsTracker as MetricsCollector };
