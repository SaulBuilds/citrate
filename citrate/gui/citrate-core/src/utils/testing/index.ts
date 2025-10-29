/**
 * Testing Utilities - Exports
 *
 * Unified exports for integration tests and performance benchmarks.
 */

export * from './integrationTests';
export * from './performanceBenchmarks';

// Re-export commonly used functions
export {
  runAllTests,
  testSearchFlow,
  testFilteringAndSorting,
  testReviewSubmission,
  testMetricsCollection,
  testRecommendations,
  testIPFSUpload
} from './integrationTests';

export {
  generatePerformanceReport,
  benchmarkSearchPerformance,
  benchmarkFilterPerformance,
  benchmarkRecommendations,
  benchmarkIPFSUpload,
  exportPerformanceReport,
  compareReports
} from './performanceBenchmarks';
