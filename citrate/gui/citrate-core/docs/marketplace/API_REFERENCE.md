# Marketplace API Reference

Complete API reference for all marketplace utilities and components.

## Table of Contents

1. [Search API](#search-api)
2. [Recommendation Engine](#recommendation-engine)
3. [Metrics Tracking](#metrics-tracking)
4. [Review Management](#review-management)
5. [Metadata Validation](#metadata-validation)
6. [IPFS Uploader](#ipfs-uploader)
7. [React Hooks](#react-hooks)
8. [React Components](#react-components)

---

## Search API

### SearchEngine

Main search engine class using FlexSearch.

```typescript
import { SearchEngine } from '@/utils/search/searchEngine';

const engine = new SearchEngine(config?: FlexSearchConfig);
```

#### Methods

##### `buildIndex(documents: SearchDocument[]): Promise<void>`

Build search index from documents.

```typescript
await engine.buildIndex(models);
```

##### `search(query: SearchQuery): Promise<SearchResponse>`

Perform search with filters and sorting.

```typescript
const results = await engine.search({
  text: 'language model',
  filters: {
    categories: [ModelCategory.LANGUAGE_MODELS],
    priceMax: 1000000000000000000
  },
  sort: 'rating_desc',
  page: 0,
  pageSize: 20
});
```

##### `addDocument(document: SearchDocument): Promise<void>`

Add single document to index.

```typescript
await engine.addDocument(newModel);
```

##### `updateDocument(modelId: string, document: SearchDocument): Promise<void>`

Update existing document.

```typescript
await engine.updateDocument('model-123', updatedModel);
```

##### `removeDocument(modelId: string): Promise<void>`

Remove document from index.

```typescript
await engine.removeDocument('model-123');
```

##### `getSuggestions(text: string, limit?: number): Promise<SearchSuggestion[]>`

Get autocomplete suggestions.

```typescript
const suggestions = await engine.getSuggestions('gpt', 10);
```

##### `getStatistics(): SearchStatistics`

Get index statistics.

```typescript
const stats = engine.getStatistics();
console.log(`Total models: ${stats.totalModels}`);
```

---

## Recommendation Engine

### RecommendationEngine

Multi-algorithm recommendation system.

```typescript
import { RecommendationEngine } from '@/utils/recommendations/engine';

const engine = new RecommendationEngine(
  models: SearchDocument[],
  config?: RecommendationEngineConfig
);
```

#### Methods

##### `getRecommendations(context: RecommendationContext): RecommendationResult`

Get recommendations based on context.

```typescript
const result = engine.getRecommendations({
  modelId: 'model-123',
  userAddress: '0xabc...',
  limit: 10,
  algorithms: ['content-based', 'collaborative', 'trending']
});
```

##### `getSimilarModels(modelId: string, limit: number): SearchDocument[]`

Content-based filtering for similar models.

```typescript
const similar = engine.getSimilarModels('model-123', 5);
```

##### `getUsersWhoBoughtAlsoBought(modelId: string, limit: number): SearchDocument[]`

Collaborative filtering recommendations.

```typescript
const collab = engine.getUsersWhoBoughtAlsoBought('model-123', 3);
```

##### `getTrendingModels(timeWindow: TimeWindow, limit: number): SearchDocument[]`

Get trending models.

```typescript
const trending = engine.getTrendingModels('7d', 10);
```

##### `getCategoryRecommendations(category: ModelCategory, limit: number): SearchDocument[]`

Get popular models in category.

```typescript
const categoryModels = engine.getCategoryRecommendations(
  ModelCategory.LANGUAGE_MODELS,
  10
);
```

##### `getPersonalizedRecommendations(userAddress: string, limit: number): SearchDocument[]`

Personalized recommendations based on user history.

```typescript
const personalized = engine.getPersonalizedRecommendations('0xabc...', 5);
```

##### `clearCache(): void`

Clear recommendation cache.

```typescript
engine.clearCache();
```

##### `updateModels(models: SearchDocument[]): void`

Update models and clear cache.

```typescript
engine.updateModels(updatedModels);
```

---

### User Tracking

Privacy-conscious local tracking.

```typescript
import {
  trackModelView,
  trackModelPurchase,
  trackModelInference,
  getUserHistory,
  getRecentlyViewed,
  clearHistory
} from '@/utils/recommendations/userTracking';
```

#### Functions

##### `trackModelView(modelId: string, userAddress?: string, metadata?: object): void`

Track model view.

```typescript
trackModelView('model-123', '0xabc...', {
  duration: 45,
  fromSearch: true,
  searchQuery: 'language model'
});
```

##### `trackModelPurchase(modelId: string, userAddress: string): void`

Track model purchase.

```typescript
trackModelPurchase('model-123', '0xabc...');
```

##### `trackModelInference(modelId: string, userAddress: string): void`

Track model inference.

```typescript
trackModelInference('model-123', '0xabc...');
```

##### `getUserHistory(): UserInteraction[]`

Get full user history.

```typescript
const history = getUserHistory();
```

##### `getRecentlyViewed(limit: number): string[]`

Get recently viewed model IDs.

```typescript
const recent = getRecentlyViewed(10);
```

##### `clearHistory(): void`

Clear all tracking data.

```typescript
clearHistory();
```

---

## Metrics Tracking

### MetricsCollector

Real-time metrics collection.

```typescript
import { MetricsCollector } from '@/utils/metrics/collector';

const collector = new MetricsCollector();
```

#### Methods

##### `recordInference(modelId: string, latency: number, success: boolean): void`

Record inference event.

```typescript
collector.recordInference('model-123', 245, true);
```

##### `getMetrics(modelId: string, timeRange?: TimeRange): PerformanceMetrics`

Get metrics for model.

```typescript
const metrics = collector.getMetrics('model-123', '7d');
```

##### `calculatePercentiles(modelId: string): PercentileData`

Calculate latency percentiles.

```typescript
const percentiles = collector.calculatePercentiles('model-123');
console.log(`P95 latency: ${percentiles.p95}ms`);
```

##### `getMetricsHistory(modelId: string, timeRange: TimeRange): MetricsHistory`

Get time-series metrics.

```typescript
const history = collector.getMetricsHistory('model-123', '30d');
```

##### `exportMetrics(config: MetricsExportConfig): string`

Export metrics data.

```typescript
const json = collector.exportMetrics({
  format: 'json',
  timeRange: '30d',
  includeRawData: true,
  includeAggregates: true,
  includeCharts: false
});
```

---

## Review Management

### Review Functions

```typescript
import {
  submitReview,
  getModelReviews,
  updateReview,
  deleteReview,
  voteReview
} from '@/utils/reviews';
```

##### `submitReview(review: Review): Promise<void>`

Submit new review.

```typescript
await submitReview({
  modelId: 'model-123',
  rating: 5,
  title: 'Excellent model',
  content: 'Works great for my use case...',
  reviewerAddress: '0xabc...'
});
```

##### `getModelReviews(modelId: string, filters?: ReviewFilters): Promise<Review[]>`

Get reviews for model.

```typescript
const reviews = await getModelReviews('model-123', {
  minRating: 4,
  verifiedOnly: true,
  sortBy: 'helpful'
});
```

##### `updateReview(reviewId: string, updates: Partial<Review>): Promise<void>`

Update existing review.

```typescript
await updateReview('review-456', {
  rating: 4,
  content: 'Updated review...'
});
```

##### `deleteReview(reviewId: string): Promise<void>`

Delete review.

```typescript
await deleteReview('review-456');
```

##### `voteReview(reviewId: string, helpful: boolean): Promise<void>`

Vote on review helpfulness.

```typescript
await voteReview('review-456', true); // helpful
```

---

## Metadata Validation

### validateModelMetadata

```typescript
import { validateModelMetadata } from '@/utils/metadata/validator';

const result = validateModelMetadata(metadata);

if (!result.isValid) {
  console.error('Validation errors:', result.errors);
}
```

#### Validation Rules

- **name**: Required, 1-100 characters
- **description**: Required, 10-5000 characters
- **category**: Required, valid ModelCategory
- **tags**: Required, 1-20 tags, each 2-30 characters
- **framework**: Required, non-empty string
- **version**: Optional, semver format
- **license**: Optional, valid SPDX identifier

---

## IPFS Uploader

### IPFSUploader

Upload and pin metadata to IPFS.

```typescript
import { IPFSUploader } from '@/utils/metadata/ipfsUploader';

const uploader = new IPFSUploader({
  gateway: 'https://ipfs.io',
  pinningService: 'pinata',
  apiKey: 'your-api-key'
});
```

#### Methods

##### `uploadMetadata(metadata: object): Promise<IPFSUploadResult>`

Upload metadata to IPFS.

```typescript
const result = await uploader.uploadMetadata(modelMetadata);
console.log(`CID: ${result.cid}`);
console.log(`URI: ${result.uri}`);
```

##### `fetchMetadata(cid: string): Promise<object>`

Fetch metadata from IPFS.

```typescript
const metadata = await uploader.fetchMetadata('QmXyz...');
```

##### `pinContent(cid: string): Promise<void>`

Pin content to ensure availability.

```typescript
await uploader.pinContent('QmXyz...');
```

##### `validateMetadata(metadata: object): ValidationResult`

Validate metadata before upload.

```typescript
const validation = uploader.validateMetadata(metadata);
```

---

## React Hooks

### useSearch

Search hook with debouncing.

```typescript
import { useSearch } from '@/hooks/useSearch';

const {
  results,
  isLoading,
  error,
  search,
  clearResults
} = useSearch(models, debounceMs);

// Trigger search
search({
  text: 'language model',
  filters: { categories: [ModelCategory.LANGUAGE_MODELS] },
  sort: 'rating_desc'
});
```

### useRecommendations

Recommendations hook.

```typescript
import { useRecommendations } from '@/hooks/useRecommendations';

const {
  similarModels,
  trendingModels,
  recentlyViewed,
  collaborative,
  personalized,
  isLoading,
  error,
  refreshRecommendations
} = useRecommendations({
  modelId: 'model-123',
  userAddress: '0xabc...',
  models: allModels
});
```

### useModelMetadata

Metadata management hook.

```typescript
import { useModelMetadata } from '@/hooks/useModelMetadata';

const {
  metadata,
  isLoading,
  error,
  updateMetadata,
  uploadToIPFS,
  validateMetadata
} = useModelMetadata(modelId);
```

### useReviews

Reviews management hook.

```typescript
import { useReviews } from '@/hooks/useReviews';

const {
  reviews,
  isLoading,
  error,
  submitReview,
  updateReview,
  deleteReview
} = useReviews(modelId);
```

---

## React Components

### SearchBar

Full-featured search bar with autocomplete.

```typescript
import { SearchBar } from '@/components/marketplace/SearchBar';

<SearchBar
  onSearch={(query) => console.log(query)}
  placeholder="Search models..."
  showFilters={true}
  showSuggestions={true}
/>
```

### FilterPanel

Advanced filter panel.

```typescript
import { FilterPanel } from '@/components/marketplace/FilterPanel';

<FilterPanel
  filters={currentFilters}
  onFiltersChange={(filters) => console.log(filters)}
  availableCategories={categories}
  availableFrameworks={frameworks}
/>
```

### SortDropdown

Sort options dropdown.

```typescript
import { SortDropdown } from '@/components/marketplace/SortDropdown';

<SortDropdown
  value={currentSort}
  onChange={(sort) => console.log(sort)}
/>
```

### ModelCard

Model display card.

```typescript
import { ModelCard } from '@/components/marketplace/ModelCard';

<ModelCard
  model={model}
  onClick={() => navigate(`/model/${model.modelId}`)}
  showMetrics={true}
/>
```

### MetricsDashboard

Comprehensive metrics dashboard.

```typescript
import { MetricsDashboard } from '@/components/marketplace/MetricsDashboard';

<MetricsDashboard
  modelId="model-123"
  timeRange="30d"
  showCharts={true}
/>
```

### ReviewList

Review list with filtering.

```typescript
import { ReviewList } from '@/components/marketplace/ReviewList';

<ReviewList
  modelId="model-123"
  filters={{ minRating: 4, verifiedOnly: true }}
  sortBy="helpful"
/>
```

### RecommendationsPanel

Unified recommendations panel.

```typescript
import { RecommendationsPanel } from '@/components/marketplace/RecommendationsPanel';

<RecommendationsPanel
  modelId="model-123"
  userAddress="0xabc..."
  models={allModels}
  onModelClick={(id) => navigate(`/model/${id}`)}
  layout="sections"
  collapsible={true}
/>
```

---

## Type Definitions

### SearchDocument

```typescript
interface SearchDocument {
  modelId: string;
  name: string;
  description: string;
  tags: string[];
  category: ModelCategory;
  framework: string;
  creatorAddress: string;
  basePrice: number;
  discountPrice: number;
  averageRating: number;
  reviewCount: number;
  totalSales: number;
  totalInferences: number;
  isActive: boolean;
  featured: boolean;
  listedAt: number;
  qualityScore: number;
  metadataURI: string;
  sizeBytes?: number;
  modelSize?: ModelSize;
  version?: string;
}
```

### SearchQuery

```typescript
interface SearchQuery {
  text?: string;
  filters?: SearchFilters;
  sort?: SortOption;
  page?: number;
  pageSize?: number;
}
```

### RecommendationContext

```typescript
interface RecommendationContext {
  modelId?: string;
  userAddress?: string;
  excludeModelIds?: string[];
  limit?: number;
  minScore?: number;
  algorithms?: RecommendationAlgorithm[];
}
```

### PerformanceMetrics

```typescript
interface PerformanceMetrics {
  avgLatency: number;
  p50Latency: number;
  p90Latency: number;
  p95Latency: number;
  p99Latency: number;
  totalInferences: number;
  successfulInferences: number;
  failedInferences: number;
  errorRate: number;
  throughputPerSecond: number;
  startTime: number;
  endTime: number;
}
```

---

## Error Handling

All async functions throw errors that should be handled:

```typescript
try {
  const results = await engine.search(query);
} catch (error) {
  if (error instanceof SearchError) {
    console.error('Search failed:', error.message);
  } else if (error instanceof ValidationError) {
    console.error('Invalid query:', error.details);
  } else {
    console.error('Unexpected error:', error);
  }
}
```

---

## Performance Considerations

- **Search**: Target < 500ms for most queries
- **Recommendations**: Target < 200ms for similar models
- **Filters**: Target < 300ms for combined filters
- **IPFS Upload**: Variable (network dependent)
- **Metrics Calculation**: O(n) where n = data points

---

## Best Practices

1. **Debounce search input** (300-500ms recommended)
2. **Cache recommendation results** (5 minute TTL)
3. **Batch IPFS requests** when possible
4. **Use pagination** for large result sets
5. **Implement loading states** for all async operations
6. **Handle errors gracefully** with user-friendly messages
7. **Validate inputs** before API calls
8. **Clear caches** when data changes

---

## Support

For API support and questions:
- GitHub Issues: [github.com/citrate/citrate](https://github.com/citrate/citrate)
- Discord: [discord.gg/citrate](https://discord.gg/citrate)
- Documentation: [docs.citrate.ai](https://docs.citrate.ai)
