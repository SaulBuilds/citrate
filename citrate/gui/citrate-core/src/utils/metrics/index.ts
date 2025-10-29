/**
 * Metrics Module
 *
 * Exports all metrics types, utilities, and tracker functionality.
 */

// Type exports
export type {
  PerformanceMetrics,
  MetricDataPoint,
  MetricsHistory,
  MetricsAggregation,
  QualityScoreBreakdown,
  ReliabilityMetrics,
  EngagementMetrics,
  TimeRange,
  ChartDataPoint,
  PercentileData,
  ExportFormat,
  MetricsExportConfig,
  MetricThreshold,
  MetricAlert,
} from './types';

// Tracker exports
export {
  MetricsTracker,
  getMockMetricsHistory,
  timeRangeToMs,
  formatTimestamp,
  historyToChartData,
} from './metricsTracker';
