# Sprint 4: Technical Tasks - Discovery & Quality Metrics

**Sprint Goal:** Build comprehensive model discovery, search, and quality assessment systems

---

## Day 1: Search Infrastructure (6 hours)

### Task 1.1: Design Search Indexing Schema
**Estimated Time:** 1.5 hours
**Assigned To:** Developer
**Priority:** P0

#### Implementation Steps
1. Design search document structure
2. Define indexed fields and weights
3. Plan index update strategy
4. Design cache structure

#### Search Document Schema
```typescript
// core/api/src/search/types.ts
export interface SearchDocument {
  modelId: string;           // Unique identifier
  name: string;              // Weight: 10
  description: string;       // Weight: 5
  tags: string[];            // Weight: 7
  category: ModelCategory;   // Weight: 3
  framework: string;         // Weight: 2
  creatorAddress: string;    // Exact match only
  creatorName?: string;      // Weight: 1

  // Metadata for filtering/sorting
  basePrice: number;
  discountPrice: number;
  averageRating: number;
  reviewCount: number;
  totalSales: number;
  totalInferences: number;
  isActive: boolean;
  featured: boolean;
  listedAt: number;

  // Quality score (pre-computed)
  qualityScore: number;

  // For IPFS metadata
  metadataURI: string;
  ipfsCID?: string;
}

export interface SearchIndex {
  documents: Map<string, SearchDocument>;
  lastUpdated: number;
  version: string;
}

export interface SearchQuery {
  text?: string;
  filters?: SearchFilters;
  sort?: SortOption;
  page?: number;
  pageSize?: number;
}

export interface SearchFilters {
  category?: ModelCategory[];
  priceMin?: number;
  priceMax?: number;
  ratingMin?: number;
  frameworks?: string[];
  modelSizes?: ModelSize[];
  activeOnly?: boolean;
}

export type SortOption =
  | 'relevance'
  | 'rating_desc'
  | 'rating_asc'
  | 'price_desc'
  | 'price_asc'
  | 'popularity'
  | 'recent';
```

#### Acceptance Criteria
- [ ] Schema supports all search/filter requirements
- [ ] Document size optimized (<5KB per model)
- [ ] TypeScript types defined
- [ ] Documentation written

---

### Task 1.2: Implement IPFS Metadata Crawler/Indexer
**Estimated Time:** 2 hours
**Assigned To:** Developer
**Priority:** P0

#### Implementation Steps
1. Create IPFS metadata fetcher
2. Implement index builder
3. Add incremental update logic
4. Background sync service

#### Code Structure
```rust
// core/api/src/search/indexer.rs
use ipfs_api_backend_hyper::{IpfsClient, IpfsApi};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ModelMetadata {
    pub name: String,
    pub description: String,
    pub tags: Vec<String>,
    pub examples: Vec<Example>,
    pub license: String,
    pub architecture: ArchitectureInfo,
    pub performance: PerformanceBenchmarks,
}

pub struct SearchIndexer {
    ipfs_client: IpfsClient,
    index: SearchIndex,
    update_queue: Vec<String>, // model IDs to update
}

impl SearchIndexer {
    pub async fn new(ipfs_gateway: &str) -> Result<Self> {
        let ipfs_client = IpfsClient::default();
        let index = SearchIndex::load_or_create()?;

        Ok(Self {
            ipfs_client,
            index,
            update_queue: Vec::new(),
        })
    }

    /// Fetch metadata from IPFS and update index
    pub async fn index_model(&mut self, model_id: String, ipfs_cid: String) -> Result<()> {
        // Fetch from IPFS
        let metadata = self.fetch_metadata(&ipfs_cid).await?;

        // Build search document
        let doc = self.build_search_document(model_id, metadata)?;

        // Update index
        self.index.add_document(doc)?;

        Ok(())
    }

    async fn fetch_metadata(&self, cid: &str) -> Result<ModelMetadata> {
        // Fetch from IPFS with timeout
        let data = tokio::time::timeout(
            Duration::from_secs(10),
            self.ipfs_client.cat(cid)
        ).await??;

        // Parse JSON
        let metadata: ModelMetadata = serde_json::from_slice(&data)?;
        Ok(metadata)
    }

    /// Sync all models from blockchain
    pub async fn sync_from_chain(&mut self) -> Result<()> {
        // Query ModelRegistry for all models
        let models = self.query_all_models().await?;

        for model in models {
            if let Err(e) = self.index_model(model.id, model.ipfs_cid).await {
                warn!("Failed to index model {}: {}", model.id, e);
                // Continue with next model
            }
        }

        Ok(())
    }

    /// Periodic update (call every 5 minutes)
    pub async fn update_index(&mut self) -> Result<()> {
        // Process update queue
        while let Some(model_id) = self.update_queue.pop() {
            // Re-index model
            self.reindex_model(&model_id).await?;
        }

        // Save index to disk
        self.index.save()?;

        Ok(())
    }
}
```

```typescript
// gui/citrate-core/src/services/search/indexer.ts
import { create as createIPFS } from 'ipfs-http-client';

export class SearchIndexBuilder {
  private ipfs: any;
  private index: SearchIndex;

  constructor() {
    this.ipfs = createIPFS({ url: 'https://ipfs.io' });
    this.index = { documents: new Map(), lastUpdated: 0, version: '1.0' };
  }

  async buildIndex(models: ModelListing[]): Promise<void> {
    for (const model of models) {
      try {
        // Fetch metadata from IPFS
        const metadata = await this.fetchMetadata(model.metadataURI);

        // Build search document
        const doc = this.buildSearchDocument(model, metadata);

        // Add to index
        this.index.documents.set(model.modelId, doc);
      } catch (error) {
        console.error(`Failed to index model ${model.modelId}:`, error);
      }
    }

    this.index.lastUpdated = Date.now();

    // Save to localStorage
    this.saveIndex();
  }

  private async fetchMetadata(ipfsURI: string): Promise<any> {
    // Extract CID from URI (ipfs://CID or https://...)
    const cid = this.extractCID(ipfsURI);

    // Fetch from IPFS with timeout
    const controller = new AbortController();
    const timeout = setTimeout(() => controller.abort(), 10000);

    try {
      const chunks = [];
      for await (const chunk of this.ipfs.cat(cid, { signal: controller.signal })) {
        chunks.push(chunk);
      }

      const data = Buffer.concat(chunks);
      return JSON.parse(data.toString());
    } finally {
      clearTimeout(timeout);
    }
  }

  private buildSearchDocument(
    listing: ModelListing,
    metadata: any
  ): SearchDocument {
    return {
      modelId: listing.modelId,
      name: metadata.name || 'Unnamed Model',
      description: metadata.description || '',
      tags: metadata.tags || [],
      category: listing.category,
      framework: metadata.framework || 'unknown',
      creatorAddress: listing.owner,
      creatorName: metadata.creatorName,
      basePrice: Number(listing.basePrice),
      discountPrice: Number(listing.discountPrice),
      averageRating: listing.averageRating / 100, // Convert from percentage
      reviewCount: listing.reviewCount,
      totalSales: listing.totalSales,
      totalInferences: 0, // TODO: Fetch from ModelRegistry
      isActive: listing.active,
      featured: listing.featured,
      listedAt: listing.listedAt,
      qualityScore: this.calculateQualityScore(listing),
      metadataURI: listing.metadataURI,
      ipfsCID: this.extractCID(listing.metadataURI),
    };
  }

  private calculateQualityScore(listing: ModelListing): number {
    const ratingScore = (listing.averageRating / 100) * 40;
    const reviewScore = Math.min(listing.reviewCount / 100, 1) * 10;
    const salesScore = Math.min(listing.totalSales / 1000, 1) * 30;
    const reliabilityScore = 20; // TODO: Calculate from metrics

    return Math.round(ratingScore + reviewScore + salesScore + reliabilityScore);
  }

  private saveIndex(): void {
    localStorage.setItem('searchIndex', JSON.stringify({
      documents: Array.from(this.index.documents.entries()),
      lastUpdated: this.index.lastUpdated,
      version: this.index.version,
    }));
  }
}
```

#### Acceptance Criteria
- [ ] Fetches metadata from IPFS successfully
- [ ] Handles IPFS timeouts gracefully
- [ ] Builds search index from blockchain data
- [ ] Incremental updates work
- [ ] Background sync every 5 minutes

---

### Task 1.3: Create Search API Endpoints
**Estimated Time:** 1.5 hours
**Assigned To:** Developer
**Priority:** P0

#### Implementation Steps
1. Add `/api/search` endpoint
2. Add `/api/search/autocomplete` endpoint
3. Add `/api/search/suggestions` endpoint
4. Implement query parsing and validation

#### Code Structure
```rust
// core/api/src/routes/search.rs
use axum::{extract::Query, Json, Router, routing::get};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct SearchParams {
    q: Option<String>,
    category: Option<Vec<String>>,
    price_min: Option<f64>,
    price_max: Option<f64>,
    rating_min: Option<f64>,
    sort: Option<String>,
    page: Option<usize>,
    page_size: Option<usize>,
}

#[derive(Debug, Serialize)]
pub struct SearchResponse {
    results: Vec<SearchDocument>,
    total: usize,
    page: usize,
    page_size: usize,
    took_ms: u64,
}

pub async fn search_models(
    Query(params): Query<SearchParams>,
) -> Result<Json<SearchResponse>> {
    let start = std::time::Instant::now();

    // Get search index
    let index = SEARCH_INDEX.read().await;

    // Parse query
    let query = SearchQuery {
        text: params.q,
        filters: build_filters(&params),
        sort: parse_sort_option(params.sort),
        page: params.page.unwrap_or(1),
        page_size: params.page_size.unwrap_or(20).min(100),
    };

    // Execute search
    let results = index.search(&query)?;

    let took_ms = start.elapsed().as_millis() as u64;

    Ok(Json(SearchResponse {
        results: results.documents,
        total: results.total,
        page: query.page,
        page_size: query.page_size,
        took_ms,
    }))
}

pub async fn autocomplete(
    Query(params): Query<AutocompleteParams>,
) -> Result<Json<Vec<String>>> {
    let index = SEARCH_INDEX.read().await;
    let suggestions = index.autocomplete(&params.q, params.limit.unwrap_or(10))?;
    Ok(Json(suggestions))
}

pub fn routes() -> Router {
    Router::new()
        .route("/search", get(search_models))
        .route("/search/autocomplete", get(autocomplete))
        .route("/search/suggestions", get(suggestions))
}
```

#### Acceptance Criteria
- [ ] `/api/search` endpoint works
- [ ] `/api/search/autocomplete` returns suggestions
- [ ] Query validation prevents injection
- [ ] Response time <500ms
- [ ] Proper error handling

---

### Task 1.4: Build Search UI Components
**Estimated Time:** 1 hour
**Assigned To:** Developer
**Priority:** P0

#### Implementation Steps
1. Create `SearchBar` component
2. Create `SearchFilters` component
3. Integrate with API endpoints
4. Add debouncing for search input

#### Code Structure
```typescript
// gui/citrate-core/src/components/SearchBar.tsx
import { useState, useEffect } from 'react';
import { Search, X } from 'lucide-react';
import { useDebounce } from '../hooks/useDebounce';

export const SearchBar: React.FC<SearchBarProps> = ({
  onSearch,
  placeholder = 'Search models...',
}) => {
  const [query, setQuery] = useState('');
  const [suggestions, setSuggestions] = useState<string[]>([]);
  const [showSuggestions, setShowSuggestions] = useState(false);

  const debouncedQuery = useDebounce(query, 300);

  useEffect(() => {
    if (debouncedQuery) {
      // Fetch suggestions
      fetchSuggestions(debouncedQuery).then(setSuggestions);
      // Trigger search
      onSearch(debouncedQuery);
    }
  }, [debouncedQuery]);

  return (
    <div className="search-bar">
      <div className="search-input-wrapper">
        <Search className="search-icon" />
        <input
          type="text"
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          onFocus={() => setShowSuggestions(true)}
          onBlur={() => setTimeout(() => setShowSuggestions(false), 200)}
          placeholder={placeholder}
          className="search-input"
        />
        {query && (
          <button onClick={() => setQuery('')} className="clear-button">
            <X />
          </button>
        )}
      </div>

      {showSuggestions && suggestions.length > 0 && (
        <div className="suggestions-dropdown">
          {suggestions.map((suggestion, i) => (
            <div
              key={i}
              className="suggestion-item"
              onClick={() => {
                setQuery(suggestion);
                setShowSuggestions(false);
              }}
            >
              {suggestion}
            </div>
          ))}
        </div>
      )}
    </div>
  );
};
```

#### Acceptance Criteria
- [ ] Search bar renders correctly
- [ ] Debouncing works (300ms delay)
- [ ] Autocomplete suggestions appear
- [ ] Keyboard navigation works
- [ ] Clear button functional

---

## Day 2: Search Filtering & UI (7 hours)

### Task 2.1: Implement Full-Text Search Algorithm
**Estimated Time:** 2 hours
**Assigned To:** Developer
**Priority:** P0

#### Implementation Steps
1. Choose search library (flexsearch or minisearch)
2. Configure index with field weights
3. Implement search scoring
4. Add fuzzy matching

#### Code Structure
```typescript
// gui/citrate-core/src/services/search/engine.ts
import FlexSearch from 'flexsearch';

export class SearchEngine {
  private index: FlexSearch.Index;
  private documents: Map<string, SearchDocument>;

  constructor() {
    this.index = new FlexSearch.Index({
      tokenize: 'forward',
      resolution: 9,
      depth: 3,
      cache: true,
      preset: 'performance',
      doc: {
        id: 'modelId',
        field: ['name', 'description', 'tags', 'framework'],
        weight: {
          name: 10,
          description: 5,
          tags: 7,
          framework: 2,
        },
      },
    });

    this.documents = new Map();
  }

  addDocument(doc: SearchDocument): void {
    this.index.add(doc);
    this.documents.set(doc.modelId, doc);
  }

  search(query: SearchQuery): SearchResults {
    const startTime = performance.now();

    // Execute search
    let results: string[];

    if (query.text) {
      results = this.index.search(query.text, {
        limit: 1000, // Get all matches for filtering
        suggest: true, // Enable fuzzy matching
      });
    } else {
      // No text query, return all
      results = Array.from(this.documents.keys());
    }

    // Convert IDs to documents
    let documents = results
      .map(id => this.documents.get(id))
      .filter(doc => doc !== undefined) as SearchDocument[];

    // Apply filters
    documents = this.applyFilters(documents, query.filters);

    // Apply sorting
    documents = this.applySorting(documents, query.sort || 'relevance');

    // Pagination
    const total = documents.length;
    const start = (query.page - 1) * query.pageSize;
    const end = start + query.pageSize;
    documents = documents.slice(start, end);

    const took = Math.round(performance.now() - startTime);

    return {
      documents,
      total,
      page: query.page,
      pageSize: query.pageSize,
      tookMs: took,
    };
  }

  private applyFilters(
    docs: SearchDocument[],
    filters?: SearchFilters
  ): SearchDocument[] {
    if (!filters) return docs;

    return docs.filter(doc => {
      if (filters.category && !filters.category.includes(doc.category)) {
        return false;
      }

      if (filters.priceMin !== undefined && doc.basePrice < filters.priceMin) {
        return false;
      }

      if (filters.priceMax !== undefined && doc.basePrice > filters.priceMax) {
        return false;
      }

      if (filters.ratingMin !== undefined && doc.averageRating < filters.ratingMin) {
        return false;
      }

      if (filters.frameworks && !filters.frameworks.includes(doc.framework)) {
        return false;
      }

      if (filters.activeOnly && !doc.isActive) {
        return false;
      }

      return true;
    });
  }

  private applySorting(
    docs: SearchDocument[],
    sort: SortOption
  ): SearchDocument[] {
    const sorted = [...docs];

    switch (sort) {
      case 'rating_desc':
        return sorted.sort((a, b) => b.averageRating - a.averageRating);

      case 'rating_asc':
        return sorted.sort((a, b) => a.averageRating - b.averageRating);

      case 'price_desc':
        return sorted.sort((a, b) => b.basePrice - a.basePrice);

      case 'price_asc':
        return sorted.sort((a, b) => a.basePrice - b.basePrice);

      case 'popularity':
        return sorted.sort((a, b) => b.totalSales - a.totalSales);

      case 'recent':
        return sorted.sort((a, b) => b.listedAt - a.listedAt);

      case 'relevance':
      default:
        // Already sorted by relevance from FlexSearch
        return sorted;
    }
  }

  autocomplete(query: string, limit: number = 10): string[] {
    const results = this.index.search(query, {
      limit,
      suggest: true,
    });

    return results
      .map(id => this.documents.get(id)?.name)
      .filter(name => name !== undefined) as string[];
  }
}
```

#### Acceptance Criteria
- [ ] Search returns relevant results
- [ ] Fuzzy matching works (typo tolerance)
- [ ] Field weighting correct
- [ ] Performance <500ms for 10K documents

---

### Task 2.2: Add Filtering and Sorting Logic
**Estimated Time:** 1.5 hours
**Assigned To:** Developer
**Priority:** P0

#### Implementation Steps
1. Implement filter UI components
2. Connect filters to search engine
3. Add sort dropdown
4. URL state management

#### Code Structure
```typescript
// gui/citrate-core/src/components/SearchFilters.tsx
export const SearchFilters: React.FC<SearchFiltersProps> = ({
  filters,
  onChange,
}) => {
  return (
    <div className="search-filters">
      <div className="filter-section">
        <h4>Category</h4>
        <CategoryFilter
          selected={filters.category}
          onChange={(categories) => onChange({ ...filters, category: categories })}
        />
      </div>

      <div className="filter-section">
        <h4>Price Range</h4>
        <PriceRangeSlider
          min={filters.priceMin}
          max={filters.priceMax}
          onChange={(min, max) => onChange({ ...filters, priceMin: min, priceMax: max })}
        />
      </div>

      <div className="filter-section">
        <h4>Rating</h4>
        <RatingFilter
          min={filters.ratingMin}
          onChange={(rating) => onChange({ ...filters, ratingMin: rating })}
        />
      </div>

      <div className="filter-section">
        <h4>Framework</h4>
        <FrameworkFilter
          selected={filters.frameworks}
          onChange={(frameworks) => onChange({ ...filters, frameworks })}
        />
      </div>

      <button
        onClick={() => onChange({})}
        className="clear-filters-button"
      >
        Clear All Filters
      </button>
    </div>
  );
};
```

#### Acceptance Criteria
- [ ] All filters functional
- [ ] Multiple filters combinable
- [ ] Clear filters button works
- [ ] URL updates with filters

---

### Task 2.3: Create Search Results Component
**Estimated Time:** 1.5 hours
**Assigned To:** Developer
**Priority:** P0

#### Code Structure
```typescript
// gui/citrate-core/src/components/SearchResults.tsx
export const SearchResults: React.FC<SearchResultsProps> = ({
  results,
  loading,
  error,
  viewMode = 'grid',
}) => {
  if (loading) {
    return <LoadingSpinner />;
  }

  if (error) {
    return <ErrorMessage message={error} />;
  }

  if (results.total === 0) {
    return (
      <EmptyState
        icon={<Search />}
        title="No models found"
        description="Try adjusting your search or filters"
      />
    );
  }

  return (
    <div className="search-results">
      <div className="results-header">
        <span className="results-count">
          Showing {results.documents.length} of {results.total} models
        </span>
        <span className="results-time">
          ({results.tookMs}ms)
        </span>
        <ViewModeToggle value={viewMode} onChange={setViewMode} />
      </div>

      <div className={`results-${viewMode}`}>
        {results.documents.map(model => (
          <ModelCard key={model.modelId} model={model} />
        ))}
      </div>

      <Pagination
        page={results.page}
        pageSize={results.pageSize}
        total={results.total}
        onChange={onPageChange}
      />
    </div>
  );
};
```

#### Acceptance Criteria
- [ ] Grid and list views work
- [ ] Pagination functional
- [ ] Loading states shown
- [ ] Empty state helpful

---

### Task 2.4: Implement Autocomplete/Suggestions
**Estimated Time:** 1 hour
**Assigned To:** Developer
**Priority:** P1

#### Acceptance Criteria
- [ ] Suggestions appear as user types
- [ ] Keyboard navigation works
- [ ] Click to select suggestion
- [ ] Recent searches shown

---

### Task 2.5: Performance Optimization and Caching
**Estimated Time:** 1 hour
**Assigned To:** Developer
**Priority:** P1

#### Implementation Steps
1. Implement result caching
2. Add index caching
3. Optimize rendering with virtualization
4. Web Worker for indexing

#### Acceptance Criteria
- [ ] Search results cached
- [ ] Index loads from cache
- [ ] Large result lists virtualized
- [ ] Performance targets met

---

## Day 3: Quality Metrics & Analytics (7 hours)

### Task 3.1: Design Metrics Collection System
**Estimated Time:** 1 hour
**Assigned To:** Developer
**Priority:** P0

#### Schema Design
```rust
// core/api/src/metrics/types.rs
#[derive(Debug, Serialize, Deserialize)]
pub struct InferenceMetric {
    pub model_id: String,
    pub timestamp: i64,
    pub latency_ms: u64,
    pub success: bool,
    pub error_type: Option<String>,
    pub input_size_bytes: u64,
    pub output_size_bytes: u64,
    pub cost: u128,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AggregatedMetrics {
    pub model_id: String,
    pub time_window: TimeWindow,
    pub total_inferences: u64,
    pub successful_inferences: u64,
    pub failed_inferences: u64,
    pub success_rate: f64,
    pub latency_p50: u64,
    pub latency_p95: u64,
    pub latency_p99: u64,
    pub total_revenue: u128,
    pub average_cost: u128,
}
```

#### Acceptance Criteria
- [ ] Schema supports all metrics
- [ ] Time-series data structure
- [ ] Efficient aggregation design

---

### Task 3.2: Implement Performance Tracking
**Estimated Time:** 2 hours
**Assigned To:** Developer
**Priority:** P0

#### Implementation
```rust
// core/api/src/metrics/collector.rs
pub struct MetricsCollector {
    db: Arc<RocksDB>,
    buffer: Vec<InferenceMetric>,
    flush_interval: Duration,
}

impl MetricsCollector {
    pub fn record_inference(&mut self, metric: InferenceMetric) {
        self.buffer.push(metric);

        if self.buffer.len() >= 100 {
            self.flush();
        }
    }

    pub fn flush(&mut self) {
        // Write to RocksDB
        for metric in self.buffer.drain(..) {
            let key = format!("metric:{}:{}", metric.model_id, metric.timestamp);
            self.db.put(key.as_bytes(), &bincode::serialize(&metric).unwrap());
        }
    }

    pub async fn aggregate_metrics(
        &self,
        model_id: &str,
        window: TimeWindow,
    ) -> Result<AggregatedMetrics> {
        // Query metrics from time window
        let metrics = self.query_metrics(model_id, window).await?;

        // Calculate aggregates
        let total = metrics.len() as u64;
        let successful = metrics.iter().filter(|m| m.success).count() as u64;
        let success_rate = (successful as f64 / total as f64) * 100.0;

        let mut latencies: Vec<u64> = metrics.iter()
            .filter(|m| m.success)
            .map(|m| m.latency_ms)
            .collect();
        latencies.sort();

        let p50 = percentile(&latencies, 0.50);
        let p95 = percentile(&latencies, 0.95);
        let p99 = percentile(&latencies, 0.99);

        Ok(AggregatedMetrics {
            model_id: model_id.to_string(),
            time_window: window,
            total_inferences: total,
            successful_inferences: successful,
            failed_inferences: total - successful,
            success_rate,
            latency_p50: p50,
            latency_p95: p95,
            latency_p99: p99,
            total_revenue: metrics.iter().map(|m| m.cost).sum(),
            average_cost: metrics.iter().map(|m| m.cost).sum::<u128>() / total as u128,
        })
    }
}
```

#### Acceptance Criteria
- [ ] Metrics recorded on each inference
- [ ] Async collection (no blocking)
- [ ] Aggregation functions work
- [ ] Percentile calculation correct

---

### Task 3.3: Create Quality Score Algorithm
**Estimated Time:** 1.5 hours
**Assigned To:** Developer
**Priority:** P0

#### Algorithm
```typescript
// gui/citrate-core/src/services/metrics/qualityScore.ts
export interface QualityScoreComponents {
  ratingScore: number;      // 0-40 (40% weight)
  performanceScore: number;  // 0-30 (30% weight)
  reliabilityScore: number;  // 0-20 (20% weight)
  engagementScore: number;   // 0-10 (10% weight)
}

export function calculateQualityScore(
  listing: ModelListing,
  metrics: AggregatedMetrics
): number {
  const components = {
    ratingScore: calculateRatingScore(listing),
    performanceScore: calculatePerformanceScore(metrics),
    reliabilityScore: calculateReliabilityScore(metrics),
    engagementScore: calculateEngagementScore(listing),
  };

  return Math.round(
    components.ratingScore +
    components.performanceScore +
    components.reliabilityScore +
    components.engagementScore
  );
}

function calculateRatingScore(listing: ModelListing): number {
  const avgRating = listing.averageRating / 100; // 0-5
  const reviewCount = listing.reviewCount;

  // Confidence factor (more reviews = higher confidence)
  const confidenceFactor = Math.min(reviewCount / 50, 1);

  // Normalized rating * confidence * max weight
  return (avgRating / 5.0) * confidenceFactor * 40;
}

function calculatePerformanceScore(metrics: AggregatedMetrics): number {
  // Compare latency against category average
  const categoryAvg = getCategoryAverageLatency(metrics.modelId);
  const relativePerformance = categoryAvg / metrics.latency_p95;

  // Normalize to 0-30
  return Math.min(relativePerformance, 1.5) * 20;
}

function calculateReliabilityScore(metrics: AggregatedMetrics): number {
  // Success rate weight
  return (metrics.success_rate / 100) * 20;
}

function calculateEngagementScore(listing: ModelListing): number {
  const salesScore = Math.min(listing.totalSales / 1000, 1) * 5;
  const reviewScore = Math.min(listing.reviewCount / 100, 1) * 5;
  return salesScore + reviewScore;
}
```

#### Acceptance Criteria
- [ ] Score calculation correct
- [ ] Components properly weighted
- [ ] Handles edge cases
- [ ] Performance optimized

---

### Task 3.4: Build Metrics Dashboard UI
**Estimated Time:** 1.5 hours
**Assigned To:** Developer
**Priority:** P0

#### Code Structure
```typescript
// gui/citrate-core/src/components/MetricsDashboard.tsx
export const MetricsDashboard: React.FC<MetricsDashboardProps> = ({
  modelId,
}) => {
  const [timeWindow, setTimeWindow] = useState<TimeWindow>('7d');
  const { data: metrics, loading } = useMetrics(modelId, timeWindow);

  if (loading) return <LoadingSpinner />;

  return (
    <div className="metrics-dashboard">
      <div className="dashboard-header">
        <h2>Performance Metrics</h2>
        <TimeWindowSelector value={timeWindow} onChange={setTimeWindow} />
      </div>

      <div className="metrics-grid">
        <MetricCard
          title="Total Inferences"
          value={metrics.total_inferences}
          trend={calculateTrend(metrics)}
        />

        <MetricCard
          title="Success Rate"
          value={`${metrics.success_rate.toFixed(1)}%`}
          subtitle={`${metrics.successful_inferences} / ${metrics.total_inferences}`}
        />

        <MetricCard
          title="Median Latency"
          value={`${metrics.latency_p50}ms`}
          subtitle={`p95: ${metrics.latency_p95}ms, p99: ${metrics.latency_p99}ms`}
        />

        <MetricCard
          title="Total Revenue"
          value={formatCurrency(metrics.total_revenue)}
        />
      </div>

      <div className="charts">
        <LatencyChart metrics={metrics} />
        <UsageTrendChart modelId={modelId} timeWindow={timeWindow} />
        <ErrorBreakdownChart metrics={metrics} />
      </div>
    </div>
  );
};
```

#### Acceptance Criteria
- [ ] All metric cards display correctly
- [ ] Time window selector works
- [ ] Charts render properly
- [ ] Responsive design

---

### Task 3.5: Add Analytics Charts
**Estimated Time:** 1 hour
**Assigned To:** Developer
**Priority:** P1

#### Charts to Implement
- Latency distribution histogram
- Usage trend line chart
- Success/failure pie chart
- Revenue over time

#### Acceptance Criteria
- [ ] All charts functional
- [ ] Interactive tooltips
- [ ] Responsive
- [ ] Performant

---

## Day 4: Reviews & Metadata Enhancement (6.5 hours)

### Task 4.1: Enhance Review Display with Voting
**Estimated Time:** 1.5 hours
**Assigned To:** Developer
**Priority:** P0

#### Implementation
```typescript
// gui/citrate-core/src/components/ReviewList.tsx
export const ReviewList: React.FC<ReviewListProps> = ({ modelId }) => {
  const [sortBy, setSortBy] = useState<ReviewSortOption>('helpful');
  const { data: reviews } = useReviews(modelId, sortBy);

  return (
    <div className="review-list">
      <div className="review-header">
        <h3>Customer Reviews</h3>
        <ReviewSortSelector value={sortBy} onChange={setSortBy} />
      </div>

      {reviews.map(review => (
        <ReviewItem key={review.id} review={review} />
      ))}
    </div>
  );
};

const ReviewItem: React.FC<{ review: Review }> = ({ review }) => {
  const [helpful, setHelpful] = useState(review.userVote);

  const handleVote = async (isHelpful: boolean) => {
    await voteOnReview(review.id, isHelpful);
    setHelpful(isHelpful);
  };

  return (
    <div className="review-item">
      <div className="review-header">
        <div className="reviewer-info">
          <span className="reviewer-name">{formatAddress(review.reviewer)}</span>
          {review.verified && <VerifiedBadge />}
        </div>
        <StarRating value={review.rating} readonly />
      </div>

      <p className="review-comment">{review.comment}</p>

      <div className="review-footer">
        <span className="review-date">{formatDate(review.timestamp)}</span>

        <div className="review-voting">
          <button
            onClick={() => handleVote(true)}
            className={helpful === true ? 'active' : ''}
          >
            👍 Helpful ({review.helpfulCount})
          </button>

          <button
            onClick={() => handleVote(false)}
            className={helpful === false ? 'active' : ''}
          >
            👎 Not Helpful ({review.notHelpfulCount})
          </button>

          <button onClick={() => reportReview(review.id)}>
            Report
          </button>
        </div>
      </div>
    </div>
  );
};
```

#### Acceptance Criteria
- [ ] Voting buttons functional
- [ ] Vote counts update
- [ ] One vote per user enforced
- [ ] Report button works

---

### Task 4.2: Add Review Moderation Tools
**Estimated Time:** 1 hour
**Assigned To:** Developer
**Priority:** P1

#### Features
- Admin dashboard for reported reviews
- Hide/delete review actions
- Ban abusive users

#### Acceptance Criteria
- [ ] Admin can view reports
- [ ] Hide/delete functional
- [ ] Audit log maintained

---

### Task 4.3: Create Rich Model Card Editor
**Estimated Time:** 2 hours
**Assigned To:** Developer
**Priority:** P0

#### Implementation
```typescript
// gui/citrate-core/src/components/ModelCardEditor.tsx
export const ModelCardEditor: React.FC<ModelCardEditorProps> = ({
  modelId,
  initialData,
  onSave,
}) => {
  const [metadata, setMetadata] = useState<ModelMetadata>(initialData);
  const [preview, setPreview] = useState(false);

  return (
    <div className="model-card-editor">
      <div className="editor-tabs">
        <button onClick={() => setPreview(false)}>Edit</button>
        <button onClick={() => setPreview(true)}>Preview</button>
      </div>

      {!preview ? (
        <div className="editor-form">
          <section>
            <h3>Basic Information</h3>
            <input
              placeholder="Model Name"
              value={metadata.name}
              onChange={(e) => setMetadata({ ...metadata, name: e.target.value })}
            />

            <MarkdownEditor
              value={metadata.description}
              onChange={(desc) => setMetadata({ ...metadata, description: desc })}
            />

            <TagInput
              tags={metadata.tags}
              onChange={(tags) => setMetadata({ ...metadata, tags })}
            />
          </section>

          <section>
            <h3>Technical Specifications</h3>
            <ArchitectureInput
              value={metadata.architecture}
              onChange={(arch) => setMetadata({ ...metadata, architecture: arch })}
            />

            <SupportedFormatsInput
              input={metadata.inputFormats}
              output={metadata.outputFormats}
              onChange={(formats) => setMetadata({ ...metadata, ...formats })}
            />
          </section>

          <section>
            <h3>Examples</h3>
            <ExampleEditor
              examples={metadata.examples}
              onChange={(examples) => setMetadata({ ...metadata, examples })}
            />
          </section>

          <section>
            <h3>License & Terms</h3>
            <LicenseSelector
              value={metadata.license}
              onChange={(license) => setMetadata({ ...metadata, license })}
            />
          </section>

          <button onClick={() => handleSave(metadata)}>
            Save to IPFS
          </button>
        </div>
      ) : (
        <ModelCardPreview metadata={metadata} />
      )}
    </div>
  );
};
```

#### Acceptance Criteria
- [ ] All fields editable
- [ ] Markdown rendering works
- [ ] Image upload functional
- [ ] Preview accurate
- [ ] Saves to IPFS

---

### Task 4.4: Implement Metadata Validation
**Estimated Time:** 1 hour
**Assigned To:** Developer
**Priority:** P1

#### Validation Rules
- Required fields (name, description)
- Field length limits
- Valid IPFS CIDs
- Valid license types

#### Acceptance Criteria
- [ ] All validations work
- [ ] Error messages clear
- [ ] Prevents invalid data

---

### Task 4.5: IPFS Metadata Storage Integration
**Estimated Time:** 1 hour
**Assigned To:** Developer
**Priority:** P0

#### Implementation
```typescript
async function saveMetadataToIPFS(metadata: ModelMetadata): Promise<string> {
  const ipfs = createIPFS({ url: IPFS_GATEWAY });

  // Convert to JSON
  const json = JSON.stringify(metadata, null, 2);

  // Add to IPFS
  const result = await ipfs.add(json);

  // Return CID
  return result.cid.toString();
}
```

#### Acceptance Criteria
- [ ] Uploads to IPFS successfully
- [ ] Returns valid CID
- [ ] Handles errors gracefully

---

## Day 5: Recommendations & Polish (7 hours)

### Task 5.1: Build Recommendation Algorithm
**Estimated Time:** 2 hours
**Assigned To:** Developer
**Priority:** P1

#### Algorithms
1. Similar models (category + tags)
2. Collaborative filtering (co-purchases)
3. Trending models (velocity)

#### Acceptance Criteria
- [ ] All algorithms implemented
- [ ] Results cached
- [ ] Performance acceptable

---

### Task 5.2: Create Recommendation Widgets
**Estimated Time:** 1.5 hours
**Assigned To:** Developer
**Priority:** P1

#### Widgets
- Similar Models
- Trending Now
- Recently Viewed
- You May Also Like

#### Acceptance Criteria
- [ ] All widgets functional
- [ ] Clickable navigation
- [ ] Responsive design

---

### Task 5.3: Integration Testing
**Estimated Time:** 2 hours
**Assigned To:** Developer
**Priority:** P0

#### Test Scenarios
- Search → filter → sort → view model
- Submit review → vote → moderate
- View metrics dashboard
- Edit model card → save to IPFS
- Click recommendations

#### Acceptance Criteria
- [ ] All flows work end-to-end
- [ ] No console errors
- [ ] Performance targets met

---

### Task 5.4: Performance Tuning
**Estimated Time:** 1 hour
**Assigned To:** Developer
**Priority:** P0

#### Optimization Areas
- Search index loading
- Metrics aggregation
- Chart rendering
- Image loading

#### Acceptance Criteria
- [ ] All benchmarks met
- [ ] No UI jank
- [ ] Memory usage acceptable

---

### Task 5.5: Documentation and Cleanup
**Estimated Time:** 0.5 hours
**Assigned To:** Developer
**Priority:** P1

#### Deliverables
- API documentation
- User guide updates
- Code comments
- CHANGELOG entry

#### Acceptance Criteria
- [ ] All docs updated
- [ ] Code commented
- [ ] Changelog complete

---

## Technical Dependencies

### NPM Packages
```bash
npm install flexsearch        # Full-text search
npm install @tanstack/react-query  # Data fetching
npm install date-fns          # Date formatting
npm install react-markdown    # Markdown rendering
npm install recharts          # Charts (if not installed)
npm install ipfs-http-client  # IPFS integration
```

### Rust Crates
```toml
[dependencies]
serde_json = "1.0"
flexbuffers = "2.0"
rocksdb = "0.21"
```

---

## Performance Targets

| Operation | Target | Measurement |
|-----------|--------|-------------|
| Search query (full-text) | <500ms | Time from query to results |
| Search (with filters) | <300ms | Time to apply filters |
| Autocomplete | <100ms | Suggestion response time |
| Index update (new model) | <10s | Time to index metadata |
| Quality score calculation | <100ms | Per model |
| Metrics aggregation | <1s | For 30-day window |
| Dashboard load | <1s | Time to render charts |
| IPFS metadata fetch | <3s | With timeout fallback |

---

## Code Quality Checklist

- [ ] All TypeScript types defined
- [ ] Error handling on all async operations
- [ ] Loading states for all data fetching
- [ ] Input validation on all forms
- [ ] ARIA labels on interactive elements
- [ ] Dark mode styles applied
- [ ] Mobile responsive
- [ ] Unit tests for utilities (>80% coverage)
- [ ] Integration tests for flows
- [ ] No console warnings/errors
- [ ] ESLint passes
- [ ] Build succeeds

---

**Document Version:** 1.0
**Last Updated:** February 18, 2026
**Status:** ✅ Ready for Implementation
