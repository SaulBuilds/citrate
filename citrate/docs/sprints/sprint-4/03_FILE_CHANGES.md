# Sprint 4: File Changes - Discovery & Quality Metrics

**Sprint Goal:** Build comprehensive model discovery, search, and quality assessment systems

---

## New Files to Create

### Backend / Core API

#### Search Infrastructure
```
core/api/src/search/
├── types.rs                 # Search types, query structs
├── indexer.rs              # IPFS metadata indexer
├── engine.rs               # Search engine implementation
├── scorer.rs               # Relevance scoring
└── mod.rs                  # Module exports

core/api/src/metrics/
├── types.rs                # Metrics types, aggregation structs
├── collector.rs            # Metrics collection system
├── aggregator.rs           # Time-series aggregation
├── quality_score.rs        # Quality score algorithm
└── mod.rs                  # Module exports

core/api/src/routes/
├── search.rs               # Search API endpoints
└── metrics.rs              # Metrics API endpoints
```

### GUI / Frontend

#### Search Components
```
gui/citrate-core/src/components/search/
├── SearchBar.tsx           # Main search input with autocomplete
├── SearchFilters.tsx       # Filter panel (category, price, rating)
├── SearchResults.tsx       # Results display with pagination
├── SearchSuggestions.tsx   # Autocomplete suggestions dropdown
├── ModelCard.tsx           # Search result card component
├── CategoryFilter.tsx      # Category multi-select
├── PriceRangeSlider.tsx    # Price range slider
├── RatingFilter.tsx        # Rating filter (1-5 stars)
├── FrameworkFilter.tsx     # Framework multi-select
└── SortSelector.tsx        # Sort dropdown

gui/citrate-core/src/pages/
└── SearchPage.tsx          # Full search page layout
```

#### Review Components
```
gui/citrate-core/src/components/reviews/
├── ReviewList.tsx          # List of reviews with sorting
├── ReviewItem.tsx          # Individual review card
├── ReviewForm.tsx          # Submit review form
├── ReviewVoting.tsx        # Helpful/Not helpful buttons
├── ReviewStats.tsx         # Rating distribution chart
├── VerifiedBadge.tsx       # Verified purchase badge
├── ReviewModeration.tsx    # Admin moderation panel
└── ReportReviewModal.tsx   # Report review dialog
```

#### Metrics Components
```
gui/citrate-core/src/components/metrics/
├── MetricsDashboard.tsx    # Main metrics dashboard
├── MetricCard.tsx          # Individual metric card
├── LatencyChart.tsx        # Latency distribution histogram
├── UsageTrendChart.tsx     # Usage over time line chart
├── SuccessRateChart.tsx    # Success/failure pie chart
├── RevenueChart.tsx        # Revenue over time
├── QualityScoreBadge.tsx   # Visual quality score indicator
└── ComparisonTable.tsx     # Compare with category averages
```

#### Metadata Components
```
gui/citrate-core/src/components/metadata/
├── ModelCardEditor.tsx     # Rich metadata editor
├── MarkdownEditor.tsx      # Markdown editor with preview
├── TagInput.tsx            # Tag management component
├── ArchitectureInput.tsx   # Architecture details form
├── ExampleEditor.tsx       # Example input/output editor
├── LicenseSelector.tsx     # License dropdown
├── ModelCardPreview.tsx    # Preview rendered model card
└── ImageUploader.tsx       # Image upload for examples
```

#### Recommendation Components
```
gui/citrate-core/src/components/recommendations/
├── SimilarModels.tsx       # Similar models widget
├── TrendingModels.tsx      # Trending models widget
├── RecentlyViewed.tsx      # Recently viewed models
├── CourchasedModels.tsx   # Users also bought widget
└── RecommendationCarousel.tsx  # Horizontal carousel
```

### Services & Utilities

#### Search Services
```
gui/citrate-core/src/services/search/
├── indexer.ts              # Client-side search indexer
├── engine.ts               # FlexSearch wrapper
├── api.ts                  # Search API client
└── cache.ts                # Search result caching

gui/citrate-core/src/services/metrics/
├── collector.ts            # Metrics collection client
├── aggregator.ts           # Metrics aggregation utils
├── qualityScore.ts         # Quality score calculation
└── api.ts                  # Metrics API client

gui/citrate-core/src/services/recommendations/
├── similarity.ts           # Similar models algorithm
├── collaborative.ts        # Collaborative filtering
├── trending.ts             # Trending models algorithm
└── api.ts                  # Recommendations API client
```

#### Utilities
```
gui/citrate-core/src/utils/
├── search.ts               # Search query parsing
├── filters.ts              # Filter combination logic
├── sorting.ts              # Sorting algorithms
├── pagination.ts           # Pagination helpers
├── ipfs.ts                 # IPFS client utilities
└── validation.ts           # Metadata validation
```

### Hooks
```
gui/citrate-core/src/hooks/
├── useSearch.ts            # Search query hook
├── useFilters.ts           # Filter state management
├── useMetrics.ts           # Metrics data fetching
├── useReviews.ts           # Reviews data fetching
├── useRecommendations.ts   # Recommendations hook
└── useDebounce.ts          # Debounce hook for search
```

### Types
```
gui/citrate-core/src/types/
├── search.ts               # Search-related types
├── metrics.ts              # Metrics types
├── reviews.ts              # Review types
└── metadata.ts             # Enhanced metadata types
```

---

## Files to Modify

### Backend / Core API

#### Existing Files
```
core/api/src/lib.rs
├── Add: mod search;
├── Add: mod metrics;
└── Register new routes

core/api/src/eth_rpc.rs
├── Add: citrate_searchModels method
├── Add: citrate_getMetrics method
├── Add: citrate_getQualityScore method
└── Add: citrate_getRecommendations method

core/execution/src/executor.rs
├── Add: Metrics collection on inference
└── Hook metrics collector

core/storage/src/state.rs
├── Add: Metrics storage tables
└── Add: Search index cache
```

### Smart Contracts (Optional)

#### ModelMarketplace Enhancement
```
contracts/src/ModelMarketplace.sol
├── Add: Review voting functions
│   ├── voteHelpful(bytes32 reviewId)
│   ├── voteNotHelpful(bytes32 reviewId)
│   └── getReviewVotes(bytes32 reviewId)
├── Add: Review moderation
│   ├── reportReview(bytes32 reviewId, string reason)
│   ├── hideReview(bytes32 reviewId) [admin]
│   └── deleteReview(bytes32 reviewId) [admin]
└── Events for voting/moderation
```

#### Optional: New Contract
```
contracts/src/ModelMetrics.sol
├── Contract for on-chain metrics tracking
├── recordInference(bytes32 modelId, bool success, uint256 latency)
├── getMetrics(bytes32 modelId, uint256 timeWindow)
└── Events for metrics updates
```

### GUI / Frontend

#### Core Application Files
```
gui/citrate-core/src/App.tsx
├── Add: Search page route
├── Add: Model details route enhancements
└── Update navigation

gui/citrate-core/src/components/Marketplace.tsx
├── Add: Search bar integration
├── Add: Filter sidebar
├── Add: Trending models section
└── Replace static listings with search results

gui/citrate-core/src/components/ModelDetails.tsx
├── Add: Quality score badge
├── Add: Metrics dashboard section
├── Add: Enhanced reviews section
├── Add: Similar models section
└── Add: Recently viewed tracking

gui/citrate-core/src/components/Models.tsx
├── Add: Search integration
├── Add: Recommendations widget
└── Update model list display
```

#### Styling
```
gui/citrate-core/src/styles/
├── search.css              # Search component styles
├── metrics.css             # Metrics dashboard styles
├── reviews.css             # Review component styles
└── recommendations.css     # Recommendation widget styles
```

#### Configuration
```
gui/citrate-core/src/config/
└── search.ts               # Search configuration (weights, limits)
```

---

## Dependencies to Add

### Backend (Rust)

#### Cargo.toml
```toml
[dependencies]
# Existing dependencies...

# Search & Indexing
flexbuffers = "2.0"          # Fast serialization
sonic-server = "0.4"         # Search server (optional)

# Metrics & Analytics
statrs = "0.16"              # Statistical functions
hdrhistogram = "7.5"         # Latency histograms

# IPFS Integration
ipfs-api = "0.17"            # IPFS HTTP API client
cid = "0.10"                 # Content identifier utilities

# Time Series
chrono = "0.4"               # Date/time handling
```

### Frontend (TypeScript)

#### package.json
```json
{
  "dependencies": {
    // Existing dependencies...

    // Search
    "flexsearch": "^0.7.31",
    "minisearch": "^6.3.0",

    // Data Fetching & Caching
    "@tanstack/react-query": "^5.17.0",

    // Utilities
    "lodash": "^4.17.21",
    "date-fns": "^3.0.6",

    // IPFS
    "ipfs-http-client": "^60.0.1",

    // Markdown
    "react-markdown": "^9.0.1",
    "remark-gfm": "^4.0.0",

    // Charts (if not installed)
    "recharts": "^2.10.3",

    // Form Handling
    "react-hook-form": "^7.49.3",
    "zod": "^3.22.4"
  },
  "devDependencies": {
    // Type definitions
    "@types/lodash": "^4.14.202"
  }
}
```

---

## Database Schema Changes

### RocksDB Key Structure (Core)

#### Metrics Storage
```
# Inference metrics (time-series)
metric:{model_id}:{timestamp} → InferenceMetric (bincode)

# Aggregated metrics (pre-computed)
agg_metric:{model_id}:{time_window} → AggregatedMetrics (bincode)

# Quality scores (cached)
quality:{model_id} → QualityScore (bincode)
quality_updated:{model_id} → timestamp

# Search index
search_index → SearchIndex (bincode)
search_index_updated → timestamp
```

#### Review Voting (Off-chain)
```
review_vote:{review_id}:{user_address} → bool (helpful/not helpful)
review_vote_count:{review_id} → VoteCounts { helpful: u64, not_helpful: u64 }
review_report:{review_id}:{user_address} → Report { reason: String, timestamp: u64 }
```

### IndexedDB Schema (Frontend)

#### Search Index
```javascript
const searchDB = {
  name: 'citrate_search',
  version: 1,
  stores: [
    {
      name: 'documents',
      keyPath: 'modelId',
      indexes: [
        { name: 'category', keyPath: 'category' },
        { name: 'rating', keyPath: 'averageRating' },
        { name: 'price', keyPath: 'basePrice' },
      ]
    },
    {
      name: 'metadata',
      keyPath: 'cid',
    },
    {
      name: 'cache',
      keyPath: 'key',
      autoIncrement: false,
    }
  ]
};
```

#### User Data
```javascript
const userDB = {
  name: 'citrate_user',
  version: 1,
  stores: [
    {
      name: 'recent_searches',
      keyPath: 'id',
      autoIncrement: true,
    },
    {
      name: 'recently_viewed',
      keyPath: 'modelId',
    },
    {
      name: 'review_votes',
      keyPath: 'reviewId',
    }
  ]
};
```

---

## Configuration Changes

### Node Configuration

#### node/config/devnet.toml
```toml
# Add search indexing config
[search]
enabled = true
ipfs_gateway = "https://ipfs.io"
index_update_interval = 300  # seconds
max_index_size = 100000      # models

# Add metrics config
[metrics]
enabled = true
collection_enabled = true
aggregation_interval = 60    # seconds
retention_days = 90
```

#### node/config/testnet.toml
```toml
# Same as devnet but with production settings
[search]
enabled = true
ipfs_gateway = "https://gateway.pinata.cloud"
index_update_interval = 300

[metrics]
enabled = true
collection_enabled = true
aggregation_interval = 60
retention_days = 365
```

### GUI Configuration

#### gui/citrate-core/config/search.json
```json
{
  "searchSettings": {
    "debounceMs": 300,
    "maxResults": 100,
    "defaultPageSize": 20,
    "fuzzyThreshold": 0.8,
    "fieldWeights": {
      "name": 10,
      "description": 5,
      "tags": 7,
      "framework": 2
    }
  },
  "filterDefaults": {
    "priceMin": 0,
    "priceMax": 1000000000000000000,
    "ratingMin": 0,
    "activeOnly": true
  },
  "meticsSettings": {
    "defaultTimeWindow": "7d",
    "refreshInterval": 60000,
    "maxDataPoints": 1000
  }
}
```

---

## Test Files to Create

### Backend Tests
```
core/api/tests/
├── search_tests.rs         # Search functionality tests
├── metrics_tests.rs        # Metrics collection tests
├── quality_score_tests.rs  # Quality score calculation tests
└── integration_tests.rs    # End-to-end API tests
```

### Frontend Tests
```
gui/citrate-core/src/__tests__/
├── search/
│   ├── SearchBar.test.tsx
│   ├── SearchFilters.test.tsx
│   ├── SearchResults.test.tsx
│   └── SearchEngine.test.ts
├── metrics/
│   ├── MetricsDashboard.test.tsx
│   ├── QualityScore.test.ts
│   └── MetricsCollector.test.ts
├── reviews/
│   ├── ReviewList.test.tsx
│   ├── ReviewVoting.test.tsx
│   └── ReviewModeration.test.tsx
└── recommendations/
    ├── SimilarModels.test.tsx
    ├── TrendingModels.test.tsx
    └── RecommendationEngine.test.ts
```

---

## Migration Scripts

### Database Migration
```
scripts/migrations/
└── sprint4_add_metrics_tables.rs
    ├── Create metrics tables
    ├── Create search index tables
    └── Migrate existing data
```

---

## Documentation Updates

### User Documentation
```
docs/guides/
├── searching-models.md     # How to search for models
├── quality-metrics.md      # Understanding quality scores
├── writing-reviews.md      # How to review models
└── model-metadata.md       # Creating rich model cards
```

### Developer Documentation
```
docs/technical/
├── search-architecture.md  # Search system design
├── metrics-collection.md   # Metrics pipeline
├── quality-algorithm.md    # Quality score algorithm
└── ipfs-integration.md     # IPFS metadata storage
```

### API Documentation
```
docs/api/
├── search-api.md          # Search endpoints
├── metrics-api.md         # Metrics endpoints
└── recommendations-api.md # Recommendation endpoints
```

---

## File Change Summary

### Statistics
- **New Backend Files:** ~20 files
- **New Frontend Files:** ~50 files
- **Modified Backend Files:** ~5 files
- **Modified Frontend Files:** ~10 files
- **New Test Files:** ~20 files
- **New Documentation:** ~10 files

### Total Lines of Code (Estimate)
- **Backend:** ~3,000 lines
- **Frontend:** ~4,500 lines
- **Tests:** ~2,000 lines
- **Documentation:** ~1,500 lines
- **Total:** ~11,000 lines

---

## Git Commit Strategy

### Recommended Commit Structure

#### Day 1: Search Infrastructure
```bash
git commit -m "feat(search): add search types and indexer"
git commit -m "feat(search): implement IPFS metadata crawler"
git commit -m "feat(api): add search API endpoints"
git commit -m "feat(gui): create search UI components"
```

#### Day 2: Search UI & Filtering
```bash
git commit -m "feat(search): implement FlexSearch engine"
git commit -m "feat(search): add filtering and sorting"
git commit -m "feat(gui): create search results page"
git commit -m "perf(search): optimize with caching"
```

#### Day 3: Metrics & Analytics
```bash
git commit -m "feat(metrics): add metrics collection system"
git commit -m "feat(metrics): implement performance tracking"
git commit -m "feat(metrics): add quality score algorithm"
git commit -m "feat(gui): create metrics dashboard"
```

#### Day 4: Reviews & Metadata
```bash
git commit -m "feat(reviews): enhance review display with voting"
git commit -m "feat(reviews): add moderation tools"
git commit -m "feat(metadata): create rich model card editor"
git commit -m "feat(ipfs): integrate metadata storage"
```

#### Day 5: Recommendations & Polish
```bash
git commit -m "feat(recommendations): implement recommendation algorithms"
git commit -m "feat(gui): create recommendation widgets"
git commit -m "test: add integration tests"
git commit -m "docs: update documentation for Sprint 4"
git commit -m "chore: Sprint 4 completion"
```

---

**Document Version:** 1.0
**Last Updated:** February 18, 2026
**Status:** ✅ Ready for Implementation
