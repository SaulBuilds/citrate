# Metrics Module

Production-ready quality metrics and analytics system for AI model performance tracking.

## Overview

This module provides comprehensive tools for collecting, analyzing, and visualizing model performance metrics. It includes:

- **Performance tracking**: Latency, throughput, error rates
- **Quality scoring**: Multi-component quality score calculation
- **Time-series analytics**: Historical data with percentile calculations
- **Visualization components**: React components with Recharts integration

## Files

### Core Types
- **`types.ts`**: TypeScript interfaces for all metrics data structures

### Utilities
- **`metricsTracker.ts`**: MetricsTracker class for collecting and aggregating metrics
- **`index.ts`**: Module exports

### React Components
Located in `src/components/marketplace/`:
- **`MetricsDashboard.tsx`**: Full-featured dashboard with charts and tabs
- **`QualityScoreBreakdown.tsx`**: Quality score visualization component

## Quick Start

### 1. Import the Module

```typescript
import {
  MetricsTracker,
  getMockMetricsHistory,
  type PerformanceMetrics,
  type QualityScoreBreakdown,
} from './utils/metrics';
```

### 2. Create a Metrics Tracker

```typescript
const tracker = new MetricsTracker('your-model-id');
```

### 3. Record Measurements

```typescript
// After each inference
tracker.recordInference({
  latencyMs: 250,
  success: true,
  tokensUsed: 150,
  responseSize: 2048,
  errorType: undefined, // or error type if failed
});
```

### 4. Get Metrics

```typescript
// Get current performance metrics
const metrics = tracker.getPerformanceMetrics();

console.log(metrics.avgLatency);        // 250
console.log(metrics.p95Latency);        // 95th percentile
console.log(metrics.errorRate);         // 2.5%
console.log(metrics.throughputPerHour); // 1200

// Get time-series history
const history = tracker.getMetricsHistory(3600000); // 1 hour intervals

// Calculate quality score
const qualityScore = tracker.calculateQualityScore(
  4.5,   // averageRating (0-5 stars)
  127,   // totalReviews
  1542,  // totalSales
  45231  // totalInferences
);
```

## React Components

### MetricsDashboard

Full-featured dashboard with multiple tabs and time range selection.

```tsx
import MetricsDashboard from './components/marketplace/MetricsDashboard';

<MetricsDashboard
  modelId="model-123"
  modelName="GPT-4 Turbo"
  performanceMetrics={performanceMetrics}
  metricsHistory={metricsHistory}
  qualityScore={qualityScore}
  onExport={(format) => {
    // Handle export (json or csv)
    console.log(`Export as ${format}`);
  }}
/>
```

**Features:**
- Overview tab with key metrics cards
- Performance tab with latency distribution and trends
- Reliability tab with success/error rate charts
- Quality Score tab with detailed breakdown
- Time range selector (24h, 7d, 30d, 90d, all)
- Export functionality (JSON/CSV)

### QualityScoreBreakdown

Standalone component for quality score visualization.

```tsx
import QualityScoreBreakdownComponent from './components/marketplace/QualityScoreBreakdown';

<QualityScoreBreakdownComponent
  qualityScore={qualityScore}
  showDetails={true}
/>
```

**Features:**
- Overall score with ring chart
- Component breakdown (Rating, Performance, Reliability, Engagement)
- Weighted scoring visualization
- Detailed metrics for each component

## Quality Score Calculation

The quality score (0-100) is a weighted average of four components:

### 1. Rating (40% weight)
- Based on user reviews and star ratings
- Score = (averageStars / 5) * 100

### 2. Performance (30% weight)
- Latency score: Lower latency = higher score
- Reliability: 100 - errorRate
- Consistency: Low variance between p50 and p99 latency

### 3. Reliability (20% weight)
- Uptime percentage
- Error rate (amplified)
- Mean time between failures (MTBF)

### 4. Engagement (10% weight)
- Total sales (logarithmic scaling)
- Total inferences (logarithmic scaling)
- Active users and growth rate

```typescript
// Example calculation
const qualityScore = tracker.calculateQualityScore(
  4.5,   // 4.5/5 stars → 90 rating score
  200,   // 200 reviews
  5000,  // 5000 sales
  150000 // 150k inferences
);

console.log(qualityScore.overall);           // 87
console.log(qualityScore.rating.score);      // 90
console.log(qualityScore.performance.score); // 85
console.log(qualityScore.reliability.score); // 88
console.log(qualityScore.engagement.score);  // 82
```

## API Reference

### MetricsTracker

#### Constructor
```typescript
new MetricsTracker(modelId: string)
```

#### Methods

**`recordInference(measurement)`**
```typescript
tracker.recordInference({
  latencyMs: number;
  success: boolean;
  tokensUsed?: number;
  responseSize?: number;
  errorType?: string;
});
```

**`getPerformanceMetrics(timeRangeMs?)`**
```typescript
const metrics: PerformanceMetrics = tracker.getPerformanceMetrics();
const last24h: PerformanceMetrics = tracker.getPerformanceMetrics(86400000);
```

Returns:
- `avgLatency`, `p50Latency`, `p90Latency`, `p95Latency`, `p99Latency`
- `minLatency`, `maxLatency`
- `totalInferences`, `successfulInferences`, `failedInferences`
- `throughputPerSecond`, `throughputPerMinute`, `throughputPerHour`
- `errorRate`, `accuracy`
- `avgTokensPerRequest`, `avgResponseSize`

**`getMetricsHistory(intervalMs)`**
```typescript
const history: MetricsHistory = tracker.getMetricsHistory(3600000); // 1 hour intervals
```

Returns time-series data:
- `latency`: Average latency per interval
- `throughput`: Requests per second
- `errorRate`: Error percentage
- `successRate`: Success percentage

**`calculateQualityScore(rating, reviews, sales, inferences)`**
```typescript
const score: QualityScoreBreakdown = tracker.calculateQualityScore(
  4.5,   // averageRating (0-5)
  200,   // totalReviews
  5000,  // totalSales
  150000 // totalInferences
);
```

**`calculatePercentiles(values)`**
Static method for percentile calculations:
```typescript
const percentiles = MetricsTracker.calculatePercentiles([100, 150, 200, 250, 300]);
console.log(percentiles.p50);    // Median
console.log(percentiles.p95);    // 95th percentile
console.log(percentiles.avg);    // Average
console.log(percentiles.stdDev); // Standard deviation
```

**`clear()`**
Clear all measurements:
```typescript
tracker.clear();
```

**`getMeasurementCount()`**
```typescript
const count = tracker.getMeasurementCount();
```

**`exportMeasurements()`**
```typescript
const rawData = tracker.exportMeasurements();
```

### Utility Functions

**`getMockMetricsHistory(days)`**
Generate mock historical data for development:
```typescript
const mockHistory = getMockMetricsHistory(30); // 30 days
```

**`timeRangeToMs(range)`**
Convert TimeRange to milliseconds:
```typescript
const ms = timeRangeToMs('7d'); // 604800000
```

**`formatTimestamp(timestamp, format)`**
Format Unix timestamp to readable string:
```typescript
const short = formatTimestamp(Date.now(), 'short'); // "Oct 28, 11:30 PM"
const long = formatTimestamp(Date.now(), 'long');   // "October 28, 2024, 11:30:45 PM"
```

**`historyToChartData(history)`**
Convert MetricsHistory to Recharts-compatible format:
```typescript
const chartData = historyToChartData(metricsHistory);
// Returns: Array<{ timestamp, date, latency, throughput, errorRate, successRate }>
```

## Performance Considerations

### Memory Usage
- The tracker stores all measurements in memory
- For long-running applications, periodically export and clear:

```typescript
// Every hour
setInterval(() => {
  const data = tracker.exportMeasurements();
  saveToDatabase(data);
  tracker.clear();
}, 3600000);
```

### Percentile Calculation
- Percentiles are calculated on-demand using sorting
- Complexity: O(n log n) where n = number of measurements
- For very large datasets (>100k measurements), consider sampling

### Time-Series Aggregation
- `getMetricsHistory()` groups measurements by intervals
- Default: 1 hour intervals
- Adjust for your data volume:

```typescript
const hourly = tracker.getMetricsHistory(3600000);   // 1 hour
const daily = tracker.getMetricsHistory(86400000);   // 24 hours
```

## Integration Examples

### With Real-Time Updates

```typescript
const tracker = new MetricsTracker('model-123');

// Record each inference
async function runInference(prompt: string) {
  const start = Date.now();
  try {
    const result = await model.generate(prompt);
    tracker.recordInference({
      latencyMs: Date.now() - start,
      success: true,
      tokensUsed: result.tokens,
      responseSize: result.text.length,
    });
    return result;
  } catch (error) {
    tracker.recordInference({
      latencyMs: Date.now() - start,
      success: false,
      errorType: error.name,
    });
    throw error;
  }
}

// Update UI every 10 seconds
setInterval(() => {
  const metrics = tracker.getPerformanceMetrics();
  updateDashboard(metrics);
}, 10000);
```

### With Marketplace Service

```typescript
import { getMarketplaceService } from './utils/marketplaceService';
import { MetricsTracker } from './utils/metrics';

const marketplace = getMarketplaceService();
const tracker = new MetricsTracker(modelId);

// Fetch marketplace data
const listing = await marketplace.getListing(modelId);
const reviews = await marketplace.getModelReviews(modelId);

// Calculate rating
const avgRating = reviews.reduce((sum, r) => sum + r.rating, 0) / reviews.length;

// Calculate quality score
const qualityScore = tracker.calculateQualityScore(
  avgRating,
  reviews.length,
  Number(listing.totalSales),
  Number(listing.totalInferences)
);
```

## TypeScript Types

All types are fully documented with JSDoc comments. Import types:

```typescript
import type {
  PerformanceMetrics,
  MetricsHistory,
  QualityScoreBreakdown,
  TimeRange,
  ChartDataPoint,
  PercentileData,
  ReliabilityMetrics,
  EngagementMetrics,
  MetricDataPoint,
  MetricsAggregation,
} from './utils/metrics';
```

## Testing

### Unit Tests

```typescript
import { MetricsTracker } from './utils/metrics';

describe('MetricsTracker', () => {
  it('calculates percentiles correctly', () => {
    const values = [100, 150, 200, 250, 300];
    const percentiles = MetricsTracker.calculatePercentiles(values);

    expect(percentiles.p50).toBe(200);
    expect(percentiles.min).toBe(100);
    expect(percentiles.max).toBe(300);
  });

  it('records inference measurements', () => {
    const tracker = new MetricsTracker('test-model');
    tracker.recordInference({ latencyMs: 100, success: true });

    expect(tracker.getMeasurementCount()).toBe(1);

    const metrics = tracker.getPerformanceMetrics();
    expect(metrics.avgLatency).toBe(100);
    expect(metrics.totalInferences).toBe(1);
  });
});
```

## License

Part of the Citrate project. See LICENSE file in the repository root.

## Support

For issues or questions:
- Check the example in `src/examples/MetricsDemo.tsx`
- Review the full implementation in `src/utils/metrics/`
- See the dashboard documentation in component files
