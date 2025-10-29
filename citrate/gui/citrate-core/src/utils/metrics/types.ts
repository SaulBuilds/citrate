/**
 * Metrics Types for Model Quality Analytics
 *
 * Defines interfaces for performance metrics, quality scores, and analytics data.
 */

/**
 * Performance Metrics - Real-time model performance indicators
 */
export interface PerformanceMetrics {
  // Latency metrics (in milliseconds)
  avgLatency: number;
  p50Latency: number;  // Median
  p90Latency: number;  // 90th percentile
  p95Latency: number;  // 95th percentile
  p99Latency: number;  // 99th percentile
  minLatency: number;
  maxLatency: number;

  // Throughput metrics
  totalInferences: number;
  successfulInferences: number;
  failedInferences: number;
  throughputPerSecond: number;
  throughputPerMinute: number;
  throughputPerHour: number;

  // Accuracy & Quality
  accuracy?: number;      // 0-100 (if applicable)
  errorRate: number;      // 0-100 percentage

  // Resource utilization
  avgTokensPerRequest?: number;
  avgResponseSize?: number;  // in bytes

  // Time range
  startTime: number;      // Unix timestamp
  endTime: number;        // Unix timestamp
  durationSeconds: number;
}

/**
 * Single metric data point for time-series
 */
export interface MetricDataPoint {
  timestamp: number;      // Unix timestamp
  value: number;
  label?: string;         // Optional label for display
}

/**
 * Time-series metrics history
 */
export interface MetricsHistory {
  latency: MetricDataPoint[];
  throughput: MetricDataPoint[];
  errorRate: MetricDataPoint[];
  successRate: MetricDataPoint[];
  activeUsers?: MetricDataPoint[];
}

/**
 * Aggregated metrics by time period
 */
export interface MetricsAggregation {
  daily: {
    date: string;         // YYYY-MM-DD
    metrics: PerformanceMetrics;
  }[];
  weekly: {
    weekStart: string;    // YYYY-MM-DD
    metrics: PerformanceMetrics;
  }[];
  monthly: {
    month: string;        // YYYY-MM
    metrics: PerformanceMetrics;
  }[];
}

/**
 * Quality Score Breakdown - Components of the overall quality score
 */
export interface QualityScoreBreakdown {
  // Overall score (weighted average)
  overall: number;        // 0-100

  // Component scores (0-100 each)
  rating: {
    score: number;
    weight: number;       // 0.4 (40%)
    details: {
      averageStars: number;     // 0-5
      totalReviews: number;
      recentTrend: 'up' | 'down' | 'stable';
    };
  };

  performance: {
    score: number;
    weight: number;       // 0.3 (30%)
    details: {
      avgLatency: number;
      reliability: number;      // Uptime percentage
      consistencyScore: number; // Low variance = high consistency
    };
  };

  reliability: {
    score: number;
    weight: number;       // 0.2 (20%)
    details: {
      uptime: number;           // Percentage
      errorRate: number;        // Percentage
      meanTimeBetweenFailures: number; // Hours
    };
  };

  engagement: {
    score: number;
    weight: number;       // 0.1 (10%)
    details: {
      totalSales: number;
      totalInferences: number;
      activeUsers: number;
      growthRate: number;       // Percentage
    };
  };
}

/**
 * Reliability Metrics - Uptime and error tracking
 */
export interface ReliabilityMetrics {
  uptime: number;         // Percentage (0-100)
  downtime: number;       // Total seconds
  totalRequests: number;
  successfulRequests: number;
  failedRequests: number;
  errorRate: number;      // Percentage (0-100)
  meanTimeBetweenFailures: number; // Hours
  incidents: number;      // Count of incidents
  lastIncidentTime?: number; // Unix timestamp
}

/**
 * Engagement Metrics - User interaction data
 */
export interface EngagementMetrics {
  totalSales: number;
  totalRevenue: string;   // In wei
  totalInferences: number;
  activeUsers: number;    // Unique users in time period
  returningUsers: number; // Users with >1 purchase
  avgUsagePerUser: number; // Inferences per user
  growthRate: number;     // Percentage change
  retentionRate: number;  // Percentage of returning users
}

/**
 * Time range selector options
 */
export type TimeRange = '24h' | '7d' | '30d' | '90d' | 'all';

/**
 * Chart data point for Recharts
 */
export interface ChartDataPoint {
  timestamp: number;
  date: string;           // Formatted date for display
  [key: string]: number | string; // Dynamic metric keys
}

/**
 * Percentile calculation result
 */
export interface PercentileData {
  p50: number;
  p75: number;
  p90: number;
  p95: number;
  p99: number;
  min: number;
  max: number;
  avg: number;
  stdDev: number;
}

/**
 * Export data format options
 */
export type ExportFormat = 'json' | 'csv' | 'xlsx';

/**
 * Metrics export configuration
 */
export interface MetricsExportConfig {
  format: ExportFormat;
  timeRange: TimeRange;
  includeRawData: boolean;
  includeAggregates: boolean;
  includeCharts: boolean;
}

/**
 * Alert threshold configuration
 */
export interface MetricThreshold {
  metric: keyof PerformanceMetrics;
  operator: 'gt' | 'lt' | 'eq' | 'gte' | 'lte';
  value: number;
  severity: 'info' | 'warning' | 'error' | 'critical';
  message: string;
}

/**
 * Metric alert
 */
export interface MetricAlert {
  id: string;
  threshold: MetricThreshold;
  triggered: boolean;
  timestamp: number;
  currentValue: number;
  message: string;
}
