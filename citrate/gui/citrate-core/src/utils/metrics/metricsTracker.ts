/**
 * Metrics Tracker
 *
 * Production-ready class for collecting, aggregating, and analyzing performance metrics.
 * Supports percentile calculations, time-series data, and quality score computation.
 */

import {
  PerformanceMetrics,
  MetricsHistory,
  MetricDataPoint,
  PercentileData,
  QualityScoreBreakdown,
  TimeRange,
  ChartDataPoint,
} from './types';

/**
 * Single inference measurement
 */
interface InferenceMeasurement {
  timestamp: number;
  latencyMs: number;
  success: boolean;
  tokensUsed?: number;
  responseSize?: number;
  errorType?: string;
}

/**
 * MetricsTracker Class
 * Collects and aggregates performance metrics for AI models
 */
export class MetricsTracker {
  private measurements: InferenceMeasurement[] = [];
  private startTime: number;

  constructor(_modelId: string) {
    this.startTime = Date.now();
  }

  /**
   * Record a single inference measurement
   */
  recordInference(measurement: Omit<InferenceMeasurement, 'timestamp'>): void {
    this.measurements.push({
      ...measurement,
      timestamp: Date.now(),
    });
  }

  /**
   * Calculate percentiles from an array of values
   */
  static calculatePercentiles(values: number[]): PercentileData {
    if (values.length === 0) {
      return {
        p50: 0,
        p75: 0,
        p90: 0,
        p95: 0,
        p99: 0,
        min: 0,
        max: 0,
        avg: 0,
        stdDev: 0,
      };
    }

    const sorted = [...values].sort((a, b) => a - b);
    const n = sorted.length;

    const percentile = (p: number): number => {
      const index = Math.ceil((p / 100) * n) - 1;
      return sorted[Math.max(0, index)];
    };

    const avg = values.reduce((sum, val) => sum + val, 0) / n;
    const variance = values.reduce((sum, val) => sum + Math.pow(val - avg, 2), 0) / n;
    const stdDev = Math.sqrt(variance);

    return {
      p50: percentile(50),
      p75: percentile(75),
      p90: percentile(90),
      p95: percentile(95),
      p99: percentile(99),
      min: sorted[0],
      max: sorted[n - 1],
      avg,
      stdDev,
    };
  }

  /**
   * Get current performance metrics
   */
  getPerformanceMetrics(timeRangeMs?: number): PerformanceMetrics {
    const now = Date.now();
    const cutoff = timeRangeMs ? now - timeRangeMs : this.startTime;
    const relevantMeasurements = this.measurements.filter(
      (m) => m.timestamp >= cutoff
    );

    if (relevantMeasurements.length === 0) {
      return this.getEmptyMetrics();
    }

    const latencies = relevantMeasurements.map((m) => m.latencyMs);
    const percentiles = MetricsTracker.calculatePercentiles(latencies);

    const successful = relevantMeasurements.filter((m) => m.success);
    const failed = relevantMeasurements.filter((m) => !m.success);

    const durationSeconds = (now - cutoff) / 1000;
    const totalInferences = relevantMeasurements.length;

    const tokensData = relevantMeasurements
      .filter((m) => m.tokensUsed !== undefined)
      .map((m) => m.tokensUsed!);
    const avgTokens = tokensData.length > 0
      ? tokensData.reduce((sum, t) => sum + t, 0) / tokensData.length
      : undefined;

    const responseSizes = relevantMeasurements
      .filter((m) => m.responseSize !== undefined)
      .map((m) => m.responseSize!);
    const avgResponseSize = responseSizes.length > 0
      ? responseSizes.reduce((sum, s) => sum + s, 0) / responseSizes.length
      : undefined;

    return {
      avgLatency: percentiles.avg,
      p50Latency: percentiles.p50,
      p90Latency: percentiles.p90,
      p95Latency: percentiles.p95,
      p99Latency: percentiles.p99,
      minLatency: percentiles.min,
      maxLatency: percentiles.max,

      totalInferences,
      successfulInferences: successful.length,
      failedInferences: failed.length,
      throughputPerSecond: totalInferences / durationSeconds,
      throughputPerMinute: (totalInferences / durationSeconds) * 60,
      throughputPerHour: (totalInferences / durationSeconds) * 3600,

      errorRate: (failed.length / totalInferences) * 100,
      accuracy: successful.length > 0 ? (successful.length / totalInferences) * 100 : 0,

      avgTokensPerRequest: avgTokens,
      avgResponseSize,

      startTime: cutoff,
      endTime: now,
      durationSeconds,
    };
  }

  /**
   * Generate time-series metrics history
   */
  getMetricsHistory(intervalMs: number = 3600000): MetricsHistory {
    if (this.measurements.length === 0) {
      return {
        latency: [],
        throughput: [],
        errorRate: [],
        successRate: [],
      };
    }

    const startTime = this.measurements[0].timestamp;
    const intervals: Map<number, InferenceMeasurement[]> = new Map();

    // Group measurements by interval
    for (const measurement of this.measurements) {
      const intervalKey = Math.floor((measurement.timestamp - startTime) / intervalMs);
      if (!intervals.has(intervalKey)) {
        intervals.set(intervalKey, []);
      }
      intervals.get(intervalKey)!.push(measurement);
    }

    const latency: MetricDataPoint[] = [];
    const throughput: MetricDataPoint[] = [];
    const errorRate: MetricDataPoint[] = [];
    const successRate: MetricDataPoint[] = [];

    // Calculate metrics for each interval
    for (const [intervalKey, measurements] of intervals.entries()) {
      const timestamp = startTime + intervalKey * intervalMs;
      const latencies = measurements.map((m) => m.latencyMs);
      const avgLatency = latencies.reduce((sum, l) => sum + l, 0) / latencies.length;
      const successful = measurements.filter((m) => m.success).length;
      const failed = measurements.filter((m) => !m.success).length;
      const total = measurements.length;

      latency.push({ timestamp, value: avgLatency });
      throughput.push({ timestamp, value: (total / intervalMs) * 1000 }); // per second
      errorRate.push({ timestamp, value: (failed / total) * 100 });
      successRate.push({ timestamp, value: (successful / total) * 100 });
    }

    return {
      latency,
      throughput,
      errorRate,
      successRate,
    };
  }

  /**
   * Calculate quality score breakdown
   */
  calculateQualityScore(
    averageRating: number,
    totalReviews: number,
    totalSales: number,
    totalInferences: number
  ): QualityScoreBreakdown {
    const metrics = this.getPerformanceMetrics();

    // Rating component (40% weight)
    const ratingScore = Math.min((averageRating / 5) * 100, 100);
    const ratingWeight = 0.4;

    // Performance component (30% weight)
    // Lower latency = higher score, using logarithmic scale
    // Target: 100ms = 100 score, 1000ms = 50 score
    const latencyScore = Math.max(0, 100 - (Math.log10(metrics.avgLatency) - 2) * 50);
    const reliabilityScore = Math.max(0, 100 - metrics.errorRate);
    const consistencyScore = metrics.p99Latency > 0
      ? Math.max(0, 100 - ((metrics.p99Latency - metrics.p50Latency) / metrics.p99Latency) * 100)
      : 100;
    const performanceScore = (latencyScore * 0.4 + reliabilityScore * 0.4 + consistencyScore * 0.2);
    const performanceWeight = 0.3;

    // Reliability component (20% weight)
    const uptime = Math.max(0, 100 - metrics.errorRate * 2); // Error rate amplified
    const mtbf = metrics.failedInferences > 0
      ? (metrics.durationSeconds / 3600) / metrics.failedInferences
      : 999; // High value if no failures
    const reliabilityComponentScore = Math.min(
      100,
      uptime * 0.7 + Math.min(100, mtbf * 10) * 0.3
    );
    const reliabilityWeight = 0.2;

    // Engagement component (10% weight)
    // Logarithmic scaling for sales and inferences
    const salesScore = Math.min(100, (Math.log10(totalSales + 1) / 4) * 100);
    const inferencesScore = Math.min(100, (Math.log10(totalInferences + 1) / 6) * 100);
    const engagementScore = (salesScore + inferencesScore) / 2;
    const engagementWeight = 0.1;

    // Calculate overall weighted score
    const overall =
      ratingScore * ratingWeight +
      performanceScore * performanceWeight +
      reliabilityComponentScore * reliabilityWeight +
      engagementScore * engagementWeight;

    return {
      overall: Math.round(overall),
      rating: {
        score: Math.round(ratingScore),
        weight: ratingWeight,
        details: {
          averageStars: averageRating,
          totalReviews,
          recentTrend: 'stable', // Would need historical data for actual trend
        },
      },
      performance: {
        score: Math.round(performanceScore),
        weight: performanceWeight,
        details: {
          avgLatency: metrics.avgLatency,
          reliability: reliabilityScore,
          consistencyScore,
        },
      },
      reliability: {
        score: Math.round(reliabilityComponentScore),
        weight: reliabilityWeight,
        details: {
          uptime,
          errorRate: metrics.errorRate,
          meanTimeBetweenFailures: mtbf,
        },
      },
      engagement: {
        score: Math.round(engagementScore),
        weight: engagementWeight,
        details: {
          totalSales,
          totalInferences,
          activeUsers: 0, // Would need user tracking
          growthRate: 0, // Would need historical comparison
        },
      },
    };
  }

  /**
   * Clear all measurements
   */
  clear(): void {
    this.measurements = [];
    this.startTime = Date.now();
  }

  /**
   * Get measurement count
   */
  getMeasurementCount(): number {
    return this.measurements.length;
  }

  /**
   * Export raw measurements
   */
  exportMeasurements(): InferenceMeasurement[] {
    return [...this.measurements];
  }

  private getEmptyMetrics(): PerformanceMetrics {
    const now = Date.now();
    return {
      avgLatency: 0,
      p50Latency: 0,
      p90Latency: 0,
      p95Latency: 0,
      p99Latency: 0,
      minLatency: 0,
      maxLatency: 0,
      totalInferences: 0,
      successfulInferences: 0,
      failedInferences: 0,
      throughputPerSecond: 0,
      throughputPerMinute: 0,
      throughputPerHour: 0,
      errorRate: 0,
      startTime: this.startTime,
      endTime: now,
      durationSeconds: (now - this.startTime) / 1000,
    };
  }
}

/**
 * Generate mock metrics history for development and testing
 * This is a development utility, not used in production
 */
export function getMockMetricsHistory(days: number = 30): MetricsHistory {
  const now = Date.now();
  const intervalMs = 3600000; // 1 hour
  const points = days * 24; // Hourly data points

  const latency: MetricDataPoint[] = [];
  const throughput: MetricDataPoint[] = [];
  const errorRate: MetricDataPoint[] = [];
  const successRate: MetricDataPoint[] = [];

  for (let i = 0; i < points; i++) {
    const timestamp = now - (points - i) * intervalMs;

    // Simulate realistic patterns
    const hourOfDay = new Date(timestamp).getHours();
    const isBusinessHours = hourOfDay >= 9 && hourOfDay <= 17;

    // Latency: lower during off-peak, higher during business hours
    const baseLatency = isBusinessHours ? 250 : 150;
    const latencyVariance = Math.random() * 100 - 50;
    const latencyValue = Math.max(50, baseLatency + latencyVariance);

    // Throughput: higher during business hours
    const baseThroughput = isBusinessHours ? 15 : 5;
    const throughputVariance = Math.random() * 5 - 2.5;
    const throughputValue = Math.max(0, baseThroughput + throughputVariance);

    // Error rate: slightly higher during peak times
    const baseErrorRate = isBusinessHours ? 1.5 : 0.5;
    const errorRateVariance = Math.random() * 1 - 0.5;
    const errorRateValue = Math.max(0, Math.min(5, baseErrorRate + errorRateVariance));

    latency.push({ timestamp, value: latencyValue });
    throughput.push({ timestamp, value: throughputValue });
    errorRate.push({ timestamp, value: errorRateValue });
    successRate.push({ timestamp, value: 100 - errorRateValue });
  }

  return {
    latency,
    throughput,
    errorRate,
    successRate,
  };
}

/**
 * Convert time range to milliseconds
 */
export function timeRangeToMs(range: TimeRange): number {
  switch (range) {
    case '24h':
      return 24 * 3600000;
    case '7d':
      return 7 * 24 * 3600000;
    case '30d':
      return 30 * 24 * 3600000;
    case '90d':
      return 90 * 24 * 3600000;
    case 'all':
      return Number.MAX_SAFE_INTEGER;
  }
}

/**
 * Format timestamp to readable date string
 */
export function formatTimestamp(timestamp: number, format: 'short' | 'long' = 'short'): string {
  const date = new Date(timestamp);

  if (format === 'short') {
    return date.toLocaleDateString('en-US', {
      month: 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit',
    });
  }

  return date.toLocaleDateString('en-US', {
    year: 'numeric',
    month: 'long',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit',
  });
}

/**
 * Convert MetricsHistory to ChartDataPoint format for Recharts
 */
export function historyToChartData(history: MetricsHistory): ChartDataPoint[] {
  const dataMap = new Map<number, ChartDataPoint>();

  // Combine all metrics by timestamp
  for (const point of history.latency) {
    if (!dataMap.has(point.timestamp)) {
      dataMap.set(point.timestamp, {
        timestamp: point.timestamp,
        date: formatTimestamp(point.timestamp),
      });
    }
    dataMap.get(point.timestamp)!.latency = point.value;
  }

  for (const point of history.throughput) {
    if (!dataMap.has(point.timestamp)) {
      dataMap.set(point.timestamp, {
        timestamp: point.timestamp,
        date: formatTimestamp(point.timestamp),
      });
    }
    dataMap.get(point.timestamp)!.throughput = point.value;
  }

  for (const point of history.errorRate) {
    if (!dataMap.has(point.timestamp)) {
      dataMap.set(point.timestamp, {
        timestamp: point.timestamp,
        date: formatTimestamp(point.timestamp),
      });
    }
    dataMap.get(point.timestamp)!.errorRate = point.value;
  }

  for (const point of history.successRate) {
    if (!dataMap.has(point.timestamp)) {
      dataMap.set(point.timestamp, {
        timestamp: point.timestamp,
        date: formatTimestamp(point.timestamp),
      });
    }
    dataMap.get(point.timestamp)!.successRate = point.value;
  }

  // Convert to array and sort by timestamp
  return Array.from(dataMap.values()).sort((a, b) => a.timestamp - b.timestamp);
}
