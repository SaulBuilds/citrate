/**
 * Performance Benchmarks for Sprint 4 Features
 *
 * Measures and reports performance metrics for search, filtering,
 * recommendations, and other marketplace features.
 */

import { SearchEngine } from '../search/searchEngine';
import { SearchDocument, SearchQuery, ModelCategory } from '../search/types';
import { RecommendationEngine } from '../recommendations/engine';

export interface BenchmarkResult {
  testName: string;
  operationsPerSecond: number;
  averageLatency: number;
  p50Latency: number;
  p95Latency: number;
  p99Latency: number;
  minLatency: number;
  maxLatency: number;
  memoryUsed: number;
  samples: number;
}

export interface PerformanceReport {
  timestamp: number;
  benchmarks: BenchmarkResult[];
  summary: {
    totalTests: number;
    totalDuration: number;
    overallThroughput: number;
    passedBenchmarks: number;
    failedBenchmarks: number;
  };
  thresholds: {
    searchLatency: { target: number; actual: number; passed: boolean };
    recommendationLatency: { target: number; actual: number; passed: boolean };
    filterLatency: { target: number; actual: number; passed: boolean };
  };
}

/**
 * Create test dataset
 */
function createTestDataset(size: number): SearchDocument[] {
  return Array.from({ length: size }, (_, i) => ({
    modelId: `model-${i}`,
    name: `Test Model ${i}`,
    description: `A comprehensive description for model ${i} with various keywords and details`,
    tags: [`tag${i % 10}`, `category${i % 5}`, 'common', 'test'],
    category: [
      ModelCategory.LANGUAGE_MODELS,
      ModelCategory.VISION_MODELS,
      ModelCategory.CODE_MODELS,
      ModelCategory.EMBEDDING_MODELS
    ][i % 4],
    framework: ['PyTorch', 'TensorFlow', 'JAX', 'ONNX'][i % 4],
    creatorAddress: `0x${i.toString(16).padStart(40, '0')}`,
    basePrice: (i + 1) * 100000000000000000,
    discountPrice: (i + 1) * 90000000000000000,
    averageRating: 200 + (i % 300),
    reviewCount: i % 50,
    totalSales: i % 100,
    totalInferences: i * 10,
    isActive: true,
    featured: i % 20 === 0,
    listedAt: Date.now() - (i * 60 * 60 * 1000),
    qualityScore: 50 + (i % 50),
    metadataURI: `ipfs://Qm${i}`,
    sizeBytes: (i + 1) * 1000000000
  }));
}

/**
 * Calculate percentiles from latency samples
 */
function calculatePercentiles(samples: number[]): {
  p50: number;
  p95: number;
  p99: number;
  min: number;
  max: number;
  avg: number;
} {
  const sorted = samples.slice().sort((a, b) => a - b);
  const len = sorted.length;

  return {
    p50: sorted[Math.floor(len * 0.50)],
    p95: sorted[Math.floor(len * 0.95)],
    p99: sorted[Math.floor(len * 0.99)],
    min: sorted[0],
    max: sorted[len - 1],
    avg: samples.reduce((sum, val) => sum + val, 0) / len
  };
}

/**
 * Benchmark search performance
 */
export function benchmarkSearchPerformance(queries: string[]): BenchmarkResult {
  const testName = 'Search Performance';
  const testDataset = createTestDataset(1000);
  const searchEngine = new SearchEngine();

  // Build index (not counted in benchmark)
  const indexStart = Date.now();
  searchEngine.buildIndex(testDataset);
  const indexTime = Date.now() - indexStart;

  console.log(`Index build time: ${indexTime}ms for ${testDataset.length} documents`);

  const latencies: number[] = [];
  const memoryBefore = (performance as any).memory?.usedJSHeapSize || 0;

  // Run queries
  for (const queryText of queries) {
    const query: SearchQuery = {
      text: queryText,
      page: 0,
      pageSize: 20
    };

    const start = performance.now();
    searchEngine.search(query);
    const duration = performance.now() - start;

    latencies.push(duration);
  }

  const memoryAfter = (performance as any).memory?.usedJSHeapSize || 0;
  const memoryUsed = memoryAfter - memoryBefore;

  const percentiles = calculatePercentiles(latencies);
  const totalTime = latencies.reduce((sum, val) => sum + val, 0);

  return {
    testName,
    operationsPerSecond: (queries.length / (totalTime / 1000)),
    averageLatency: percentiles.avg,
    p50Latency: percentiles.p50,
    p95Latency: percentiles.p95,
    p99Latency: percentiles.p99,
    minLatency: percentiles.min,
    maxLatency: percentiles.max,
    memoryUsed,
    samples: queries.length
  };
}

/**
 * Benchmark filter performance
 */
export function benchmarkFilterPerformance(): BenchmarkResult {
  const testName = 'Filter Performance';
  const testDataset = createTestDataset(1000);
  const searchEngine = new SearchEngine();

  searchEngine.buildIndex(testDataset);

  const latencies: number[] = [];
  const memoryBefore = (performance as any).memory?.usedJSHeapSize || 0;

  // Test various filter combinations
  const filterQueries: SearchQuery[] = [
    {
      filters: { categories: [ModelCategory.LANGUAGE_MODELS] },
      page: 0,
      pageSize: 20
    },
    {
      filters: {
        priceMin: 100000000000000000,
        priceMax: 1000000000000000000
      },
      page: 0,
      pageSize: 20
    },
    {
      filters: {
        ratingMin: 4.0,
        frameworks: ['PyTorch']
      },
      page: 0,
      pageSize: 20
    },
    {
      filters: {
        categories: [ModelCategory.LANGUAGE_MODELS, ModelCategory.CODE_MODELS],
        priceMax: 500000000000000000,
        ratingMin: 3.5
      },
      page: 0,
      pageSize: 20
    }
  ];

  // Run each filter query multiple times
  for (let i = 0; i < 25; i++) {
    const query = filterQueries[i % filterQueries.length];

    const start = performance.now();
    searchEngine.search(query);
    const duration = performance.now() - start;

    latencies.push(duration);
  }

  const memoryAfter = (performance as any).memory?.usedJSHeapSize || 0;
  const memoryUsed = memoryAfter - memoryBefore;

  const percentiles = calculatePercentiles(latencies);
  const totalTime = latencies.reduce((sum, val) => sum + val, 0);

  return {
    testName,
    operationsPerSecond: (latencies.length / (totalTime / 1000)),
    averageLatency: percentiles.avg,
    p50Latency: percentiles.p50,
    p95Latency: percentiles.p95,
    p99Latency: percentiles.p99,
    minLatency: percentiles.min,
    maxLatency: percentiles.max,
    memoryUsed,
    samples: latencies.length
  };
}

/**
 * Benchmark recommendations
 */
export function benchmarkRecommendations(): BenchmarkResult {
  const testName = 'Recommendations Performance';
  const testDataset = createTestDataset(500);
  const engine = new RecommendationEngine(testDataset);

  const latencies: number[] = [];
  const memoryBefore = (performance as any).memory?.usedJSHeapSize || 0;

  // Benchmark different recommendation types
  for (let i = 0; i < 100; i++) {
    const modelId = `model-${i % 500}`;

    // Similar models
    const start1 = performance.now();
    engine.getSimilarModels(modelId, 10);
    latencies.push(performance.now() - start1);

    // Trending
    const start2 = performance.now();
    engine.getTrendingModels('7d', 10);
    latencies.push(performance.now() - start2);

    // Collaborative
    const start3 = performance.now();
    engine.getUsersWhoBoughtAlsoBought(modelId, 5);
    latencies.push(performance.now() - start3);
  }

  const memoryAfter = (performance as any).memory?.usedJSHeapSize || 0;
  const memoryUsed = memoryAfter - memoryBefore;

  const percentiles = calculatePercentiles(latencies);
  const totalTime = latencies.reduce((sum, val) => sum + val, 0);

  return {
    testName,
    operationsPerSecond: (latencies.length / (totalTime / 1000)),
    averageLatency: percentiles.avg,
    p50Latency: percentiles.p50,
    p95Latency: percentiles.p95,
    p99Latency: percentiles.p99,
    minLatency: percentiles.min,
    maxLatency: percentiles.max,
    memoryUsed,
    samples: latencies.length
  };
}

/**
 * Benchmark IPFS upload (simulated)
 */
export function benchmarkIPFSUpload(): BenchmarkResult {
  const testName = 'IPFS Upload Performance (Simulated)';

  const latencies: number[] = [];

  // Simulate metadata preparation and validation
  for (let i = 0; i < 100; i++) {
    const metadata = {
      name: `Test Model ${i}`,
      description: 'A'.repeat(1000), // 1KB description
      category: ModelCategory.LANGUAGE_MODELS,
      tags: ['tag1', 'tag2', 'tag3'],
      framework: 'PyTorch'
    };

    const start = performance.now();

    // Simulate validation and JSON serialization
    JSON.stringify(metadata);

    // Simulate hashing (mock)
    const mockHash = `Qm${Date.now()}${i}`;

    const duration = performance.now() - start;
    latencies.push(duration);
  }

  const percentiles = calculatePercentiles(latencies);
  const totalTime = latencies.reduce((sum, val) => sum + val, 0);

  return {
    testName,
    operationsPerSecond: (latencies.length / (totalTime / 1000)),
    averageLatency: percentiles.avg,
    p50Latency: percentiles.p50,
    p95Latency: percentiles.p95,
    p99Latency: percentiles.p99,
    minLatency: percentiles.min,
    maxLatency: percentiles.max,
    memoryUsed: 0,
    samples: latencies.length
  };
}

/**
 * Generate comprehensive performance report
 */
export function generatePerformanceReport(): PerformanceReport {
  console.log('Running Performance Benchmarks...\n');

  const startTime = Date.now();
  const benchmarks: BenchmarkResult[] = [];

  // Run benchmarks
  console.log('1. Benchmarking search...');
  const searchQueries = [
    'language model',
    'vision transformer',
    'code generation',
    'embedding',
    'GPT',
    'BERT',
    'ResNet',
    'text classification',
    'image segmentation',
    'neural network'
  ];
  benchmarks.push(benchmarkSearchPerformance(searchQueries));

  console.log('2. Benchmarking filters...');
  benchmarks.push(benchmarkFilterPerformance());

  console.log('3. Benchmarking recommendations...');
  benchmarks.push(benchmarkRecommendations());

  console.log('4. Benchmarking IPFS operations...');
  benchmarks.push(benchmarkIPFSUpload());

  const totalDuration = Date.now() - startTime;

  // Calculate thresholds
  const searchBench = benchmarks[0];
  const filterBench = benchmarks[1];
  const recBench = benchmarks[2];

  const thresholds = {
    searchLatency: {
      target: 500,
      actual: searchBench.p95Latency,
      passed: searchBench.p95Latency < 500
    },
    recommendationLatency: {
      target: 200,
      actual: recBench.p95Latency,
      passed: recBench.p95Latency < 200
    },
    filterLatency: {
      target: 300,
      actual: filterBench.p95Latency,
      passed: filterBench.p95Latency < 300
    }
  };

  const passedBenchmarks = [
    thresholds.searchLatency.passed,
    thresholds.recommendationLatency.passed,
    thresholds.filterLatency.passed
  ].filter(Boolean).length;

  // Print report
  console.log('\n=== Performance Report ===\n');

  for (const benchmark of benchmarks) {
    console.log(`${benchmark.testName}:`);
    console.log(`  Throughput: ${benchmark.operationsPerSecond.toFixed(2)} ops/sec`);
    console.log(`  Avg Latency: ${benchmark.averageLatency.toFixed(2)}ms`);
    console.log(`  P50: ${benchmark.p50Latency.toFixed(2)}ms`);
    console.log(`  P95: ${benchmark.p95Latency.toFixed(2)}ms`);
    console.log(`  P99: ${benchmark.p99Latency.toFixed(2)}ms`);
    console.log(`  Min/Max: ${benchmark.minLatency.toFixed(2)}ms / ${benchmark.maxLatency.toFixed(2)}ms`);
    console.log(`  Samples: ${benchmark.samples}`);
    console.log('');
  }

  console.log('=== Threshold Checks ===\n');
  console.log(`Search P95 Latency: ${thresholds.searchLatency.actual.toFixed(2)}ms (target: <${thresholds.searchLatency.target}ms) ${thresholds.searchLatency.passed ? '✓' : '✗'}`);
  console.log(`Recommendation P95 Latency: ${thresholds.recommendationLatency.actual.toFixed(2)}ms (target: <${thresholds.recommendationLatency.target}ms) ${thresholds.recommendationLatency.passed ? '✓' : '✗'}`);
  console.log(`Filter P95 Latency: ${thresholds.filterLatency.actual.toFixed(2)}ms (target: <${thresholds.filterLatency.target}ms) ${thresholds.filterLatency.passed ? '✓' : '✗'}`);

  console.log(`\nTotal Duration: ${totalDuration}ms`);
  console.log(`Passed: ${passedBenchmarks}/3 threshold checks\n`);

  return {
    timestamp: Date.now(),
    benchmarks,
    summary: {
      totalTests: benchmarks.length,
      totalDuration,
      overallThroughput: benchmarks.reduce((sum, b) => sum + b.operationsPerSecond, 0) / benchmarks.length,
      passedBenchmarks,
      failedBenchmarks: 3 - passedBenchmarks
    },
    thresholds
  };
}

/**
 * Export performance report to JSON
 */
export function exportPerformanceReport(report: PerformanceReport): string {
  return JSON.stringify(report, null, 2);
}

/**
 * Compare two performance reports
 */
export function compareReports(
  baseline: PerformanceReport,
  current: PerformanceReport
): {
  improved: string[];
  regressed: string[];
  unchanged: string[];
} {
  const improved: string[] = [];
  const regressed: string[] = [];
  const unchanged: string[] = [];

  for (let i = 0; i < baseline.benchmarks.length; i++) {
    const baselineBench = baseline.benchmarks[i];
    const currentBench = current.benchmarks[i];

    if (!currentBench) continue;

    const baselineLatency = baselineBench.p95Latency;
    const currentLatency = currentBench.p95Latency;

    const percentChange = ((currentLatency - baselineLatency) / baselineLatency) * 100;

    if (percentChange < -5) {
      improved.push(`${currentBench.testName}: ${Math.abs(percentChange).toFixed(1)}% faster`);
    } else if (percentChange > 5) {
      regressed.push(`${currentBench.testName}: ${percentChange.toFixed(1)}% slower`);
    } else {
      unchanged.push(`${currentBench.testName}: no significant change`);
    }
  }

  return { improved, regressed, unchanged };
}
