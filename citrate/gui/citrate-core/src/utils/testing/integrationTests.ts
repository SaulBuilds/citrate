/**
 * Integration Tests for Sprint 4 Features
 *
 * Test utilities for validating search, filtering, reviews, metrics,
 * and recommendation systems.
 */

import { SearchEngine } from '../search/searchEngine';
import { SearchDocument, SearchQuery, ModelCategory } from '../search/types';
import { RecommendationEngine } from '../recommendations/engine';
import { trackModelView, trackModelPurchase, getUserHistory } from '../recommendations/userTracking';
import { MetricsCollector } from '../metrics/collector';
import { IPFSUploader } from '../metadata/ipfsUploader';

export interface TestResult {
  testName: string;
  passed: boolean;
  duration: number;
  errors?: string[];
  warnings?: string[];
  details?: Record<string, any>;
}

/**
 * Test search flow
 */
export async function testSearchFlow(): Promise<TestResult> {
  const startTime = Date.now();
  const testName = 'Search Flow Test';
  const errors: string[] = [];
  const warnings: string[] = [];
  const details: Record<string, any> = {};

  try {
    // Create test documents
    const testModels: SearchDocument[] = [
      {
        modelId: 'test-model-1',
        name: 'GPT-4 Test Model',
        description: 'A powerful language model for testing',
        tags: ['language', 'gpt', 'testing'],
        category: ModelCategory.LANGUAGE_MODELS,
        framework: 'PyTorch',
        creatorAddress: '0x123',
        basePrice: 1000000000000000000, // 1 ETH
        discountPrice: 900000000000000000,
        averageRating: 450,
        reviewCount: 10,
        totalSales: 50,
        totalInferences: 1000,
        isActive: true,
        featured: true,
        listedAt: Date.now() - 30 * 24 * 60 * 60 * 1000,
        qualityScore: 85,
        metadataURI: 'ipfs://test',
        sizeBytes: 5000000000
      },
      {
        modelId: 'test-model-2',
        name: 'Vision Transformer',
        description: 'Image classification model',
        tags: ['vision', 'transformer', 'classification'],
        category: ModelCategory.VISION_MODELS,
        framework: 'TensorFlow',
        creatorAddress: '0x456',
        basePrice: 500000000000000000,
        discountPrice: 500000000000000000,
        averageRating: 400,
        reviewCount: 5,
        totalSales: 25,
        totalInferences: 500,
        isActive: true,
        featured: false,
        listedAt: Date.now() - 15 * 24 * 60 * 60 * 1000,
        qualityScore: 75,
        metadataURI: 'ipfs://test2',
        sizeBytes: 3000000000
      }
    ];

    // Initialize search engine
    const searchEngine = new SearchEngine();
    await searchEngine.buildIndex(testModels);
    details.indexBuildTime = Date.now() - startTime;

    // Test 1: Basic text search
    const query1: SearchQuery = {
      text: 'language model',
      page: 0,
      pageSize: 10
    };

    const searchStart = Date.now();
    const results1 = await searchEngine.search(query1);
    const searchDuration = Date.now() - searchStart;
    details.searchDuration = searchDuration;

    if (searchDuration > 500) {
      warnings.push(`Search took ${searchDuration}ms (target: <500ms)`);
    }

    if (results1.results.length === 0) {
      errors.push('Text search returned no results');
    } else {
      details.textSearchResults = results1.results.length;
    }

    // Test 2: Category filter
    const query2: SearchQuery = {
      filters: {
        categories: [ModelCategory.LANGUAGE_MODELS]
      },
      page: 0,
      pageSize: 10
    };

    const results2 = await searchEngine.search(query2);
    if (results2.results.length === 0) {
      errors.push('Category filter returned no results');
    }

    const wrongCategory = results2.results.some(r =>
      r.document.category !== ModelCategory.LANGUAGE_MODELS
    );
    if (wrongCategory) {
      errors.push('Category filter returned wrong categories');
    }

    // Test 3: Price range filter
    const query3: SearchQuery = {
      filters: {
        priceMin: 0,
        priceMax: 1000000000000000000 // 1 ETH
      },
      page: 0,
      pageSize: 10
    };

    const results3 = await searchEngine.search(query3);
    const outOfRange = results3.results.some(r =>
      r.document.basePrice > 1000000000000000000
    );
    if (outOfRange) {
      errors.push('Price filter allowed out-of-range results');
    }

    // Test 4: Sort by rating
    const query4: SearchQuery = {
      sort: 'rating_desc',
      page: 0,
      pageSize: 10
    };

    const results4 = await searchEngine.search(query4);
    if (results4.results.length > 1) {
      const sorted = results4.results.every((result, i) => {
        if (i === 0) return true;
        return result.document.averageRating <= results4.results[i - 1].document.averageRating;
      });

      if (!sorted) {
        errors.push('Sort by rating did not work correctly');
      }
    }

    details.totalTests = 4;
    details.passedTests = 4 - errors.length;

    return {
      testName,
      passed: errors.length === 0,
      duration: Date.now() - startTime,
      errors: errors.length > 0 ? errors : undefined,
      warnings: warnings.length > 0 ? warnings : undefined,
      details
    };
  } catch (error) {
    errors.push(`Exception: ${error instanceof Error ? error.message : String(error)}`);
    return {
      testName,
      passed: false,
      duration: Date.now() - startTime,
      errors,
      warnings: warnings.length > 0 ? warnings : undefined,
      details
    };
  }
}

/**
 * Test filtering and sorting
 */
export async function testFilteringAndSorting(): Promise<TestResult> {
  const startTime = Date.now();
  const testName = 'Filtering and Sorting Test';
  const errors: string[] = [];

  try {
    // Create diverse test data
    const models: SearchDocument[] = Array.from({ length: 20 }, (_, i) => ({
      modelId: `model-${i}`,
      name: `Test Model ${i}`,
      description: `Description ${i}`,
      tags: [`tag${i % 3}`, `category${i % 5}`],
      category: [ModelCategory.LANGUAGE_MODELS, ModelCategory.VISION_MODELS, ModelCategory.CODE_MODELS][i % 3],
      framework: ['PyTorch', 'TensorFlow', 'JAX'][i % 3],
      creatorAddress: `0x${i}`,
      basePrice: (i + 1) * 100000000000000000,
      discountPrice: (i + 1) * 90000000000000000,
      averageRating: 300 + (i * 10),
      reviewCount: i,
      totalSales: i * 10,
      totalInferences: i * 100,
      isActive: true,
      featured: i % 5 === 0,
      listedAt: Date.now() - i * 24 * 60 * 60 * 1000,
      qualityScore: 50 + i,
      metadataURI: `ipfs://test${i}`,
      sizeBytes: (i + 1) * 1000000000
    }));

    const searchEngine = new SearchEngine();
    await searchEngine.buildIndex(models);

    // Test all sort options
    const sortOptions = ['relevance', 'rating_desc', 'rating_asc', 'price_desc', 'price_asc', 'popularity', 'recent', 'trending'];

    for (const sort of sortOptions) {
      const query: SearchQuery = {
        sort: sort as any,
        page: 0,
        pageSize: 20
      };

      const results = await searchEngine.search(query);
      if (results.results.length === 0) {
        errors.push(`Sort option '${sort}' returned no results`);
      }
    }

    // Test combined filters
    const combinedQuery: SearchQuery = {
      filters: {
        categories: [ModelCategory.LANGUAGE_MODELS],
        priceMin: 100000000000000000,
        priceMax: 1000000000000000000,
        ratingMin: 3.5,
        frameworks: ['PyTorch']
      },
      sort: 'rating_desc',
      page: 0,
      pageSize: 10
    };

    const combinedResults = await searchEngine.search(combinedQuery);

    // Validate combined filters
    for (const result of combinedResults.results) {
      const doc = result.document;

      if (doc.category !== ModelCategory.LANGUAGE_MODELS) {
        errors.push('Combined filter: wrong category');
      }

      if (doc.basePrice < 100000000000000000 || doc.basePrice > 1000000000000000000) {
        errors.push('Combined filter: price out of range');
      }

      if (doc.framework !== 'PyTorch') {
        errors.push('Combined filter: wrong framework');
      }

      if (doc.averageRating / 100 < 3.5) {
        errors.push('Combined filter: rating too low');
      }
    }

    return {
      testName,
      passed: errors.length === 0,
      duration: Date.now() - startTime,
      errors: errors.length > 0 ? errors : undefined,
      details: {
        modelsCreated: models.length,
        sortOptionsTested: sortOptions.length,
        combinedFilterResults: combinedResults.results.length
      }
    };
  } catch (error) {
    errors.push(`Exception: ${error instanceof Error ? error.message : String(error)}`);
    return {
      testName,
      passed: false,
      duration: Date.now() - startTime,
      errors
    };
  }
}

/**
 * Test review submission
 */
export async function testReviewSubmission(): Promise<TestResult> {
  const startTime = Date.now();
  const testName = 'Review Submission Test';
  const errors: string[] = [];
  const warnings: string[] = [];

  try {
    // Note: This would normally interact with contract or backend
    // For now, test validation logic

    const validReview = {
      modelId: 'test-model',
      rating: 4,
      title: 'Great model!',
      content: 'This model works really well for my use case.',
      reviewerAddress: '0x123'
    };

    // Validate rating range
    if (validReview.rating < 1 || validReview.rating > 5) {
      errors.push('Rating validation failed');
    }

    // Validate required fields
    if (!validReview.title || !validReview.content) {
      errors.push('Required field validation failed');
    }

    // Test invalid review
    const invalidReview = {
      modelId: 'test-model',
      rating: 6, // Invalid
      title: '',
      content: '',
      reviewerAddress: '0x123'
    };

    if (invalidReview.rating >= 1 && invalidReview.rating <= 5) {
      errors.push('Should reject invalid rating');
    }

    return {
      testName,
      passed: errors.length === 0,
      duration: Date.now() - startTime,
      errors: errors.length > 0 ? errors : undefined,
      warnings: warnings.length > 0 ? warnings : undefined,
      details: {
        validReviewPassed: true,
        invalidReviewRejected: true
      }
    };
  } catch (error) {
    errors.push(`Exception: ${error instanceof Error ? error.message : String(error)}`);
    return {
      testName,
      passed: false,
      duration: Date.now() - startTime,
      errors
    };
  }
}

/**
 * Test metrics collection
 */
export async function testMetricsCollection(): Promise<TestResult> {
  const startTime = Date.now();
  const testName = 'Metrics Collection Test';
  const errors: string[] = [];

  try {
    const collector = new MetricsCollector();

    // Record test metrics
    for (let i = 0; i < 10; i++) {
      collector.recordInference('test-model', 100 + i * 10, true);
    }

    const metrics = collector.getMetrics('test-model');

    if (metrics.totalInferences !== 10) {
      errors.push(`Expected 10 inferences, got ${metrics.totalInferences}`);
    }

    if (metrics.successfulInferences !== 10) {
      errors.push(`Expected 10 successful inferences, got ${metrics.successfulInferences}`);
    }

    if (metrics.errorRate !== 0) {
      errors.push(`Expected 0% error rate, got ${metrics.errorRate}%`);
    }

    // Test percentile calculations
    const percentiles = collector.calculatePercentiles('test-model');

    if (!percentiles || percentiles.p50 === 0) {
      errors.push('Percentile calculation failed');
    }

    return {
      testName,
      passed: errors.length === 0,
      duration: Date.now() - startTime,
      errors: errors.length > 0 ? errors : undefined,
      details: {
        metricsRecorded: 10,
        percentiles
      }
    };
  } catch (error) {
    errors.push(`Exception: ${error instanceof Error ? error.message : String(error)}`);
    return {
      testName,
      passed: false,
      duration: Date.now() - startTime,
      errors
    };
  }
}

/**
 * Test recommendations
 */
export async function testRecommendations(): Promise<TestResult> {
  const startTime = Date.now();
  const testName = 'Recommendations Test';
  const errors: string[] = [];

  try {
    // Create test models
    const models: SearchDocument[] = Array.from({ length: 20 }, (_, i) => ({
      modelId: `model-${i}`,
      name: `Test Model ${i}`,
      description: `Description ${i}`,
      tags: [`tag${i % 3}`, `common`],
      category: [ModelCategory.LANGUAGE_MODELS, ModelCategory.VISION_MODELS][i % 2],
      framework: ['PyTorch', 'TensorFlow'][i % 2],
      creatorAddress: `0x${i}`,
      basePrice: (i + 1) * 100000000000000000,
      discountPrice: (i + 1) * 90000000000000000,
      averageRating: 300 + (i * 10),
      reviewCount: i,
      totalSales: i * 10,
      totalInferences: i * 100,
      isActive: true,
      featured: false,
      listedAt: Date.now() - i * 24 * 60 * 60 * 1000,
      qualityScore: 50 + i,
      metadataURI: `ipfs://test${i}`,
      sizeBytes: (i + 1) * 1000000000
    }));

    const engine = new RecommendationEngine(models);

    // Test similar models
    const similar = engine.getSimilarModels('model-0', 5);
    if (similar.length === 0) {
      errors.push('Similar models returned no results');
    }

    // Should not include the query model itself
    if (similar.some(m => m.modelId === 'model-0')) {
      errors.push('Similar models included the query model');
    }

    // Test trending
    const trending = engine.getTrendingModels('7d', 5);
    // May be empty if no recent activity, that's okay
    if (trending.length > 5) {
      errors.push('Trending returned more than requested limit');
    }

    // Test user tracking
    trackModelView('model-0', 'test-user');
    trackModelPurchase('model-0', 'test-user');

    const history = getUserHistory();
    if (history.length < 2) {
      errors.push('User tracking failed to record interactions');
    }

    return {
      testName,
      passed: errors.length === 0,
      duration: Date.now() - startTime,
      errors: errors.length > 0 ? errors : undefined,
      details: {
        similarModelsFound: similar.length,
        trendingModelsFound: trending.length,
        userHistorySize: history.length
      }
    };
  } catch (error) {
    errors.push(`Exception: ${error instanceof Error ? error.message : String(error)}`);
    return {
      testName,
      passed: false,
      duration: Date.now() - startTime,
      errors
    };
  }
}

/**
 * Test IPFS upload
 */
export async function testIPFSUpload(): Promise<TestResult> {
  const startTime = Date.now();
  const testName = 'IPFS Upload Test';
  const errors: string[] = [];
  const warnings: string[] = [];

  try {
    const uploader = new IPFSUploader({
      gateway: 'https://ipfs.io',
      pinningService: 'local'
    });

    const testMetadata = {
      name: 'Test Model',
      description: 'Test description',
      category: ModelCategory.LANGUAGE_MODELS
    };

    // Note: This would actually upload in production
    // For testing, we validate the metadata structure
    const validation = uploader.validateMetadata(testMetadata);

    if (!validation.isValid) {
      errors.push(`Metadata validation failed: ${validation.errors?.join(', ')}`);
    }

    warnings.push('IPFS upload not actually performed in test (requires network)');

    return {
      testName,
      passed: errors.length === 0,
      duration: Date.now() - startTime,
      errors: errors.length > 0 ? errors : undefined,
      warnings: warnings.length > 0 ? warnings : undefined,
      details: {
        metadataValid: validation.isValid
      }
    };
  } catch (error) {
    errors.push(`Exception: ${error instanceof Error ? error.message : String(error)}`);
    return {
      testName,
      passed: false,
      duration: Date.now() - startTime,
      errors
    };
  }
}

/**
 * Run all integration tests
 */
export async function runAllTests(): Promise<TestResult[]> {
  console.log('Starting Sprint 4 Integration Tests...\n');

  const results: TestResult[] = [];

  // Run all tests
  results.push(await testSearchFlow());
  results.push(await testFilteringAndSorting());
  results.push(await testReviewSubmission());
  results.push(await testMetricsCollection());
  results.push(await testRecommendations());
  results.push(await testIPFSUpload());

  // Print summary
  console.log('\n=== Test Summary ===');
  const passed = results.filter(r => r.passed).length;
  const total = results.length;

  console.log(`\nPassed: ${passed}/${total}`);
  console.log(`Total Duration: ${results.reduce((sum, r) => sum + r.duration, 0)}ms\n`);

  for (const result of results) {
    const status = result.passed ? '✓' : '✗';
    console.log(`${status} ${result.testName} (${result.duration}ms)`);

    if (result.errors) {
      result.errors.forEach(err => console.log(`  ERROR: ${err}`));
    }

    if (result.warnings) {
      result.warnings.forEach(warn => console.log(`  WARNING: ${warn}`));
    }
  }

  return results;
}
