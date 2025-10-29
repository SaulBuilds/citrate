# Sprint 4: Testing Checklist - Discovery & Quality Metrics

**Sprint Goal:** Ensure all search, metrics, and quality features work correctly and meet performance targets

---

## Unit Tests

### Search Engine Tests

#### Backend (Rust)
```rust
// core/api/tests/search_tests.rs

#[tokio::test]
async fn test_search_document_creation() {
    // ✅ Creates valid search document from model listing
    // ✅ All required fields populated
    // ✅ Handles missing optional fields
}

#[tokio::test]
async fn test_ipfs_metadata_fetching() {
    // ✅ Fetches metadata from valid IPFS CID
    // ✅ Handles timeout gracefully (10s)
    // ✅ Returns error for invalid CID
    // ✅ Parses JSON correctly
}

#[tokio::test]
async fn test_search_index_building() {
    // ✅ Builds index from model array
    // ✅ Incremental updates work
    // ✅ Handles duplicate model IDs
    // ✅ Index serialization/deserialization
}

#[tokio::test]
async fn test_search_query_parsing() {
    // ✅ Parses simple text queries
    // ✅ Parses filter parameters
    // ✅ Validates sort options
    // ✅ Handles invalid input gracefully
}
```

#### Frontend (TypeScript)
```typescript
// gui/citrate-core/src/__tests__/search/SearchEngine.test.ts

describe('SearchEngine', () => {
  test('adds documents to index', () => {
    // ✅ Document added successfully
    // ✅ Document retrievable by ID
  });

  test('performs full-text search', () => {
    // ✅ Returns relevant results
    // ✅ Ranks by relevance
    // ✅ Respects field weights
  });

  test('applies filters correctly', () => {
    // ✅ Category filter works
    // ✅ Price range filter works
    // ✅ Rating filter works
    // ✅ Multiple filters combinable (AND)
  });

  test('sorts results correctly', () => {
    // ✅ Sort by relevance
    // ✅ Sort by rating (desc/asc)
    // ✅ Sort by price (desc/asc)
    // ✅ Sort by popularity
    // ✅ Sort by recent
  });

  test('handles fuzzy matching', () => {
    // ✅ Tolerates typos (edit distance ≤2)
    // ✅ Returns close matches
  });

  test('autocomplete suggestions', () => {
    // ✅ Returns relevant suggestions
    // ✅ Limits results correctly
    // ✅ Handles empty query
  });
});
```

### Metrics Tests

#### Backend (Rust)
```rust
// core/api/tests/metrics_tests.rs

#[test]
fn test_percentile_calculation() {
    // ✅ p50 (median) correct
    // ✅ p95 correct
    // ✅ p99 correct
    // ✅ Handles empty array
    // ✅ Handles single value
}

#[tokio::test]
async fn test_metrics_collection() {
    // ✅ Records inference metric
    // ✅ Buffers metrics before flush
    // ✅ Flushes to storage
    // ✅ Async collection doesn't block
}

#[tokio::test]
async fn test_metrics_aggregation() {
    // ✅ Aggregates metrics for time window
    // ✅ Calculates success rate
    // ✅ Calculates percentiles
    // ✅ Calculates revenue
    // ✅ Handles no data gracefully
}

#[test]
fn test_quality_score_calculation() {
    // ✅ Rating component (40% weight)
    // ✅ Performance component (30% weight)
    // ✅ Reliability component (20% weight)
    // ✅ Engagement component (10% weight)
    // ✅ Total score 0-100
    // ✅ Handles edge cases (no reviews, no inferences)
}
```

#### Frontend (TypeScript)
```typescript
// gui/citrate-core/src/__tests__/metrics/QualityScore.test.ts

describe('Quality Score', () => {
  test('calculates rating score', () => {
    // ✅ Correct calculation
    // ✅ Applies confidence factor
    // ✅ Weights correctly (40%)
  });

  test('calculates performance score', () => {
    // ✅ Compares with category average
    // ✅ Normalizes to 0-30
  });

  test('calculates reliability score', () => {
    // ✅ Based on success rate
    // ✅ Weights correctly (20%)
  });

  test('calculates engagement score', () => {
    // ✅ Sales component
    // ✅ Review component
    // ✅ Weights correctly (10%)
  });

  test('combines components correctly', () => {
    // ✅ Total score 0-100
    // ✅ Rounded to integer
  });
});
```

### Review Tests

```typescript
// gui/citrate-core/src/__tests__/reviews/ReviewVoting.test.tsx

describe('Review Voting', () => {
  test('submits helpful vote', async () => {
    // ✅ Vote recorded
    // ✅ Count updated
    // ✅ UI updates
  });

  test('submits not helpful vote', async () => {
    // ✅ Vote recorded
    // ✅ Count updated
  });

  test('prevents duplicate voting', () => {
    // ✅ Can't vote twice
    // ✅ Error message shown
  });

  test('allows vote change', () => {
    // ✅ Can switch from helpful to not helpful
    // ✅ Counts update correctly
  });
});
```

---

## Integration Tests

### Search Flow Tests

```typescript
// gui/citrate-core/src/__tests__/integration/search.test.tsx

describe('Search Integration', () => {
  test('end-to-end search flow', async () => {
    // ✅ Type search query
    // ✅ See autocomplete suggestions
    // ✅ Select suggestion
    // ✅ Results displayed
    // ✅ Apply filter
    // ✅ Results update
    // ✅ Change sort order
    // ✅ Results reorder
    // ✅ Navigate to page 2
    // ✅ Correct results shown
  });

  test('search with no results', async () => {
    // ✅ Empty state shown
    // ✅ Helpful suggestions provided
  });

  test('search performance', async () => {
    // ✅ Results appear in <500ms
    // ✅ Autocomplete in <100ms
  });
});
```

### Metrics Flow Tests

```typescript
// gui/citrate-core/src/__tests__/integration/metrics.test.tsx

describe('Metrics Integration', () => {
  test('view model metrics', async () => {
    // ✅ Navigate to model page
    // ✅ Metrics dashboard visible
    // ✅ All metric cards loaded
    // ✅ Charts rendered
  });

  test('change time window', async () => {
    // ✅ Select 7d/30d/90d
    // ✅ Metrics update
    // ✅ Charts update
  });

  test('export metrics', async () => {
    // ✅ Click export button
    // ✅ CSV downloaded
    // ✅ Data accurate
  });
});
```

### Review Flow Tests

```typescript
// gui/citrate-core/src/__tests__/integration/reviews.test.tsx

describe('Review Integration', () => {
  test('submit review', async () => {
    // ✅ Click "Write Review" button
    // ✅ Fill out form (rating, comment)
    // ✅ Submit review
    // ✅ Review appears in list
    // ✅ Verified badge shown (if purchased)
  });

  test('vote on reviews', async () => {
    // ✅ Click helpful button
    // ✅ Count increments
    // ✅ Button shows active state
  });

  test('sort reviews', async () => {
    // ✅ Sort by most helpful
    // ✅ Reviews reorder
    // ✅ Sort by newest
    // ✅ Reviews reorder
  });

  test('report review', async () => {
    // ✅ Click report button
    // ✅ Modal opens
    // ✅ Select reason
    // ✅ Submit report
    // ✅ Confirmation message
  });
});
```

### Metadata Flow Tests

```typescript
// gui/citrate-core/src/__tests__/integration/metadata.test.tsx

describe('Metadata Integration', () => {
  test('create rich model card', async () => {
    // ✅ Open model card editor
    // ✅ Fill all fields
    // ✅ Add examples
    // ✅ Upload images
    // ✅ Preview rendering correct
    // ✅ Save to IPFS
    // ✅ CID returned
    // ✅ Update model listing with CID
  });

  test('fetch and display metadata', async () => {
    // ✅ Navigate to model page
    // ✅ Metadata loads from IPFS
    // ✅ All fields displayed
    // ✅ Images rendered
    // ✅ Examples shown
  });
});
```

### Recommendation Tests

```typescript
// gui/citrate-core/src/__tests__/integration/recommendations.test.tsx

describe('Recommendations Integration', () => {
  test('similar models widget', async () => {
    // ✅ View model page
    // ✅ Similar models displayed
    // ✅ Click similar model
    // ✅ Navigate to that model
  });

  test('trending models', async () => {
    // ✅ Trending widget on homepage
    // ✅ Shows top trending models
    // ✅ Click trending model
    // ✅ Navigate to model
  });

  test('recently viewed', async () => {
    // ✅ View several models
    // ✅ Recently viewed tracks history
    // ✅ Click recently viewed model
    // ✅ Navigate back to model
  });
});
```

---

## Performance Tests

### Search Performance

```typescript
// gui/citrate-core/src/__tests__/performance/search.bench.ts

describe('Search Performance', () => {
  test('search with 1,000 models', async () => {
    // ✅ Index build time <10s
    // ✅ Query response <500ms (p95)
    // ✅ Autocomplete <100ms
  });

  test('search with 10,000 models', async () => {
    // ✅ Index build time <60s
    // ✅ Query response <500ms (p95)
  });

  test('search with 100,000 models', async () => {
    // ✅ Index build time <10min
    // ✅ Query response <1s (p95)
  });

  test('filter performance', async () => {
    // ✅ Apply 5 filters <200ms
  });

  test('sort performance', async () => {
    // ✅ Sort 1000 results <100ms
  });

  test('pagination performance', async () => {
    // ✅ Page navigation <50ms
  });
});
```

### Metrics Performance

```rust
// core/api/benches/metrics_bench.rs

#[bench]
fn bench_metrics_collection(b: &mut Bencher) {
    // ✅ Record metric <5ms
    // ✅ Flush 100 metrics <50ms
}

#[bench]
fn bench_metrics_aggregation(b: &mut Bencher) {
    // ✅ Aggregate 1,000 metrics <100ms
    // ✅ Aggregate 10,000 metrics <500ms
}

#[bench]
fn bench_quality_score(b: &mut Bencher) {
    // ✅ Calculate quality score <100ms
}
```

### IPFS Performance

```typescript
describe('IPFS Performance', () => {
  test('fetch metadata', async () => {
    // ✅ Fetch from IPFS <3s
    // ✅ Timeout after 10s
  });

  test('upload metadata', async () => {
    // ✅ Upload to IPFS <5s
  });

  test('caching', async () => {
    // ✅ Second fetch from cache <100ms
  });
});
```

---

## Security Tests

### Input Validation

```typescript
describe('Security', () => {
  test('prevents XSS in search', () => {
    // ✅ Script tags escaped
    // ✅ HTML entities encoded
  });

  test('prevents XSS in reviews', () => {
    // ✅ Review comments sanitized
    // ✅ No script execution
  });

  test('prevents SQL injection', () => {
    // ✅ N/A (no SQL, but test query parsing)
    // ✅ Special characters handled
  });

  test('rate limiting', async () => {
    // ✅ Search limited to 100/min
    // ✅ Review voting limited
    // ✅ Report limited to 5/day
  });

  test('authentication', () => {
    // ✅ Review voting requires auth
    // ✅ Report requires auth
    // ✅ Moderation requires admin role
  });
});
```

---

## Accessibility Tests

### Keyboard Navigation

```typescript
describe('Accessibility', () => {
  test('search keyboard navigation', () => {
    // ✅ Tab through search input
    // ✅ Arrow keys in autocomplete
    // ✅ Enter to select suggestion
    // ✅ Escape to close suggestions
  });

  test('filters keyboard navigation', () => {
    // ✅ Tab through filter controls
    // ✅ Space/Enter to toggle
  });

  test('focus management', () => {
    // ✅ Focus indicators visible
    // ✅ Focus order logical
    // ✅ Modal traps focus
  });
});
```

### Screen Reader

```typescript
describe('Screen Reader Support', () => {
  test('ARIA labels present', () => {
    // ✅ Search input labeled
    // ✅ Filter controls labeled
    // ✅ Buttons have text/aria-label
  });

  test('announcements', () => {
    // ✅ Search results announced
    // ✅ Filter changes announced
    // ✅ Loading states announced
  });
});
```

---

## Cross-Browser Tests

### Browser Compatibility

```
Browsers to Test:
- ✅ Chrome 120+
- ✅ Firefox 120+
- ✅ Safari 17+
- ✅ Edge 120+

Features to Verify:
- ✅ Search functionality
- ✅ Charts rendering
- ✅ IPFS client works
- ✅ LocalStorage/IndexedDB works
- ✅ WebWorkers work
```

---

## Mobile Tests

### Responsive Design

```
Devices to Test:
- ✅ iPhone 15 (iOS 17)
- ✅ Samsung Galaxy S23 (Android 14)
- ✅ iPad Pro (iOS 17)

Features to Verify:
- ✅ Search UI responsive
- ✅ Filters accessible on mobile
- ✅ Charts render correctly
- ✅ Touch interactions work
- ✅ Performance acceptable
```

---

## Manual Test Scenarios

### Scenario 1: First-Time User Search
```
Steps:
1. Open marketplace (not logged in)
2. See search bar and trending models
3. Type "gpt" in search
4. See autocomplete suggestions
5. Select "GPT-2"
6. See search results
7. Apply filter: "Price < 0.01 ETH"
8. Results update
9. Sort by "Rating (highest)"
10. Click first result
11. View model details with metrics

Expected:
✅ All steps work smoothly
✅ No errors in console
✅ Performance feels fast
✅ UI is intuitive
```

### Scenario 2: Model Owner Viewing Metrics
```
Steps:
1. Log in as model owner
2. Navigate to "My Models"
3. Click on owned model
4. See metrics dashboard
5. Change time window to 30 days
6. Metrics update
7. Export metrics as CSV
8. Open CSV and verify data

Expected:
✅ Metrics accurate
✅ Charts render correctly
✅ CSV data matches dashboard
✅ No performance issues
```

### Scenario 3: User Reviewing Model
```
Steps:
1. Log in as user (with purchase)
2. Navigate to model page
3. Scroll to reviews section
4. Click "Write Review"
5. Fill rating (4 stars) and comment
6. Submit review
7. See review appear with verified badge
8. Vote helpful on another review
9. Sort reviews by "Most Helpful"
10. Reviews reorder

Expected:
✅ Review submitted successfully
✅ Verified badge appears
✅ Voting works
✅ Sorting works
```

### Scenario 4: Creating Rich Model Card
```
Steps:
1. Log in as model owner
2. Navigate to model editor
3. Fill all metadata fields
4. Add markdown description
5. Upload example image
6. Add input/output examples
7. Preview model card
8. Save to IPFS
9. Wait for confirmation
10. View published model card

Expected:
✅ All fields saved
✅ IPFS upload successful
✅ Preview matches published card
✅ Images display correctly
```

### Scenario 5: Discovering Models via Recommendations
```
Steps:
1. View model "GPT-2"
2. See "Similar Models" section
3. Click similar model
4. Navigate to that model
5. See "Users Also Bought" section
6. Navigate to homepage
7. See "Trending Now" widget
8. Click trending model
9. View recently viewed widget
10. Navigate back to previous model

Expected:
✅ All recommendations relevant
✅ Navigation smooth
✅ Recently viewed accurate
✅ Trending models sensible
```

---

## Regression Tests

### Existing Functionality
```
After Sprint 4 changes, verify:
- ✅ Model listing still works
- ✅ Model purchase still works
- ✅ Inference execution still works
- ✅ Wallet operations still work
- ✅ DAG visualization still works
- ✅ Transaction tracking still works
- ✅ Contract deployment still works
```

---

## Testing Tools & Commands

### Run All Tests
```bash
# Backend tests
cargo test --workspace

# Frontend tests
npm test

# Integration tests
npm run test:integration

# E2E tests
npm run test:e2e

# Performance tests
cargo bench
npm run bench
```

### Coverage
```bash
# Backend coverage
cargo tarpaulin --out Html

# Frontend coverage
npm run test:coverage

# Target: >80% coverage
```

### Linting
```bash
# Rust
cargo clippy --all-targets --all-features

# TypeScript
npm run lint
npm run type-check
```

---

## Test Data Setup

### Seed Test Data
```typescript
// scripts/seed-test-data.ts

async function seedTestData() {
  // Create 100 test models
  // Various categories, prices, ratings
  // With IPFS metadata
  // With reviews and metrics

  // Create test users
  // With purchase history
  // With review history

  // Generate metrics data
  // Various performance profiles
}
```

---

## Bug Tracking

### Found Bugs Template
```markdown
## Bug ID: S4-B001
**Severity:** High / Medium / Low
**Component:** Search / Metrics / Reviews / Metadata
**Description:** [Detailed description]
**Steps to Reproduce:**
1. Step 1
2. Step 2
**Expected:** [Expected behavior]
**Actual:** [Actual behavior]
**Environment:** Browser / OS / Node version
**Status:** Open / In Progress / Fixed
**Fixed In:** [Commit hash or PR number]
```

---

## Definition of Done (Testing)

A feature is considered "Done" from testing perspective when:

- [ ] All unit tests pass (>80% coverage)
- [ ] All integration tests pass
- [ ] Performance benchmarks met
- [ ] Security tests pass
- [ ] Accessibility tests pass
- [ ] Manual test scenarios pass
- [ ] Cross-browser tests pass
- [ ] Mobile tests pass
- [ ] No critical bugs open
- [ ] Regression tests pass
- [ ] Code coverage >80%
- [ ] All tests documented
- [ ] Test data cleanup scripts work

---

**Document Version:** 1.0
**Last Updated:** February 18, 2026
**Status:** ✅ Ready for Testing
