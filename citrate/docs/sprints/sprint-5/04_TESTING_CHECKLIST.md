# Sprint 5: Testing Checklist - Advanced Marketplace Features & Optimization

**Sprint Goal:** Ensure all advanced features work correctly, mobile experience is optimized, and performance targets are met

---

## Unit Tests

### Model Comparison Tests

#### Comparison Logic Tests
```typescript
// gui/citrate-core/src/__tests__/comparison/comparisonHelpers.test.ts

describe('Comparison Helpers', () => {
  test('findBestValue identifies lowest price', () => {
    // ✅ Correctly identifies best value for price (lowest)
    // ✅ Correctly identifies best value for rating (highest)
    // ✅ Handles ties appropriately
  });

  test('formatComparisonValue formats correctly', () => {
    // ✅ Currency formatted as ETH
    // ✅ Percentages formatted with %
    // ✅ Arrays joined with commas
  });

  test('buildComparisonData creates correct structure', () => {
    // ✅ All categories included
    // ✅ Best/worst indices correct
    // ✅ Handles missing data gracefully
  });
});
```

#### useComparison Hook Tests
```typescript
// gui/citrate-core/src/__tests__/comparison/useComparison.test.ts

describe('useComparison', () => {
  test('adds model to comparison', () => {
    // ✅ Model added to selection
    // ✅ Count increments
    // ✅ SessionStorage updated
  });

  test('prevents adding more than 5 models', () => {
    // ✅ Cannot add 6th model
    // ✅ Error handling appropriate
  });

  test('removes model from comparison', () => {
    // ✅ Model removed from selection
    // ✅ Count decrements
    // ✅ SessionStorage updated
  });

  test('clears all comparisons', () => {
    // ✅ All models removed
    // ✅ Count resets to 0
    // ✅ SessionStorage cleared
  });

  test('persists across page navigation', () => {
    // ✅ Selection loaded from sessionStorage
    // ✅ Selection maintained after remount
  });
});
```

#### Export Tests
```typescript
// gui/citrate-core/src/__tests__/comparison/exportComparison.test.ts

describe('Export Comparison', () => {
  test('exports CSV with correct data', () => {
    // ✅ CSV structure correct
    // ✅ All data included
    // ✅ Values formatted properly
  });

  test('exports PDF with formatting', () => {
    // ✅ PDF generated successfully
    // ✅ Formatting preserved
    // ✅ Best/worst highlighting present
  });

  test('generates filename with timestamp', () => {
    // ✅ Filename includes timestamp
    // ✅ Format is model-comparison-{timestamp}
  });
});
```

---

### Creator Analytics Tests

#### Analytics Calculation Tests
```typescript
// gui/citrate-core/src/__tests__/analytics/calculator.test.ts

describe('Analytics Calculator', () => {
  test('calculates total revenue correctly', () => {
    // ✅ Sums all revenue accurately
    // ✅ Handles BigInt correctly
    // ✅ Groups by model correctly
  });

  test('calculates revenue trend', () => {
    // ✅ Percentage change correct
    // ✅ Handles zero baseline
    // ✅ Positive/negative trends identified
  });

  test('calculates retention rate', () => {
    // ✅ Formula correct: returning / total
    // ✅ Handles edge cases (no users)
  });

  test('generates usage heatmap data', () => {
    // ✅ 7x24 matrix created
    // ✅ Data aggregated by hour/day
    // ✅ Max value calculated
  });

  test('projects revenue for 30 days', () => {
    // ✅ Uses linear regression or simple trend
    // ✅ Reasonable projection
  });
});
```

#### useCreatorAnalytics Hook Tests
```typescript
// gui/citrate-core/src/__tests__/analytics/useCreatorAnalytics.test.ts

describe('useCreatorAnalytics', () => {
  test('fetches analytics for time window', () => {
    // ✅ API called with correct params
    // ✅ Data loaded successfully
    // ✅ Loading state managed
  });

  test('handles time window changes', () => {
    // ✅ Refetches on window change
    // ✅ Data updates correctly
  });

  test('handles errors gracefully', () => {
    // ✅ Error state set
    // ✅ User-friendly error message
  });
});
```

---

### Batch Operations Tests

#### useBatchSelection Tests
```typescript
// gui/citrate-core/src/__tests__/batch/useBatchSelection.test.ts

describe('useBatchSelection', () => {
  test('toggles model selection', () => {
    // ✅ Adds unselected model
    // ✅ Removes selected model
    // ✅ Updates selection array
  });

  test('selects all models', () => {
    // ✅ All IDs added to selection
    // ✅ allSelected flag set
  });

  test('deselects all models', () => {
    // ✅ Selection cleared
    // ✅ allSelected flag reset
  });

  test('maintains selection across pagination', () => {
    // ✅ Selection persists when changing pages
  });
});
```

#### useBatchUndo Tests
```typescript
// gui/citrate-core/src/__tests__/batch/useBatchUndo.test.ts

describe('useBatchUndo', () => {
  test('records batch operation', () => {
    // ✅ Operation added to undo stack
    // ✅ Timestamp recorded
  });

  test('undo restores previous state', async () => {
    // ✅ Undo data applied
    // ✅ State reverted
    // ✅ Operation removed from stack
  });

  test('undo expires after 60 seconds', () => {
    // ✅ Operation removed after timeout
    // ✅ canUndo returns false
  });

  test('countdown timer updates', () => {
    // ✅ Timer counts down each second
    // ✅ Displays remaining time
  });
});
```

---

### Mobile Optimization Tests

#### useSwipeGesture Tests
```typescript
// gui/citrate-core/src/__tests__/mobile/useSwipeGesture.test.ts

describe('useSwipeGesture', () => {
  test('detects swipe left', () => {
    // ✅ Callback triggered on swipe left
    // ✅ Threshold respected (100px)
  });

  test('detects swipe right', () => {
    // ✅ Callback triggered on swipe right
  });

  test('ignores small movements', () => {
    // ✅ No callback for < 100px movement
  });

  test('animates during swipe', () => {
    // ✅ Element moves with finger
    // ✅ Springs back on release
  });
});
```

#### useMobileDetection Tests
```typescript
// gui/citrate-core/src/__tests__/mobile/useMobileDetection.test.ts

describe('useMobileDetection', () => {
  test('detects mobile device', () => {
    // ✅ Returns true for mobile user agents
    // ✅ Returns false for desktop user agents
  });

  test('detects touch capability', () => {
    // ✅ Returns true if touch events supported
  });

  test('detects screen size', () => {
    // ✅ Returns correct breakpoint
  });
});
```

---

### Performance Tests

#### Virtual Scrolling Tests
```typescript
// gui/citrate-core/src/__tests__/performance/VirtualizedModelList.test.tsx

describe('VirtualizedModelList', () => {
  test('renders only visible items', () => {
    // ✅ Only viewport items rendered
    // ✅ Overscan rows rendered for smoothness
  });

  test('handles scrolling smoothly', () => {
    // ✅ Scroll position maintained
    // ✅ Items render as scrolled into view
  });

  test('maintains performance with 1000+ items', () => {
    // ✅ Smooth 60 FPS scrolling
    // ✅ Memory usage < 100MB
  });

  test('adjusts columns based on width', () => {
    // ✅ Responsive column count
    // ✅ Re-renders on resize
  });
});
```

#### Image Optimization Tests
```typescript
// gui/citrate-core/src/__tests__/performance/OptimizedImage.test.tsx

describe('OptimizedImage', () => {
  test('lazy loads images', () => {
    // ✅ Image not loaded until near viewport
    // ✅ Uses Intersection Observer
  });

  test('uses WebP with fallback', () => {
    // ✅ WebP source provided
    // ✅ Fallback to original format
  });

  test('displays skeleton while loading', () => {
    // ✅ Skeleton shown initially
    // ✅ Removed when image loads
  });

  test('handles loading errors', () => {
    // ✅ Fallback image displayed
    // ✅ No broken image icon
  });
});
```

---

## Integration Tests

### Model Comparison Flow
```typescript
// gui/citrate-core/src/__tests__/integration/comparison.test.tsx

describe('Model Comparison Flow', () => {
  test('complete comparison workflow', async () => {
    // ✅ Navigate to marketplace
    // ✅ Add 3 models to comparison
    // ✅ Compare button shows count (3)
    // ✅ Click compare button
    // ✅ Comparison page loads
    // ✅ All 3 models displayed side-by-side
    // ✅ Visual indicators show best/worst values
    // ✅ Export CSV works
    // ✅ Export PDF works
    // ✅ Share link generated
  });

  test('comparison from URL', async () => {
    // ✅ Load comparison from URL params
    // ✅ Models fetched and displayed
    // ✅ No selection state conflicts
  });

  test('remove model from comparison', async () => {
    // ✅ Remove button works
    // ✅ Model removed from table
    // ✅ Columns adjust
  });
});
```

### Creator Analytics Flow
```typescript
// gui/citrate-core/src/__tests__/integration/analytics.test.tsx

describe('Creator Analytics Flow', () => {
  test('view analytics dashboard', async () => {
    // ✅ Navigate to analytics
    // ✅ Dashboard loads within 2s
    // ✅ Revenue section displays
    // ✅ Usage section displays
    // ✅ Demographics section displays
    // ✅ Model performance table displays
  });

  test('change time window', async () => {
    // ✅ Select "Last 7 Days"
    // ✅ Data refetches
    // ✅ Charts update
    // ✅ Metrics recalculate
  });

  test('export analytics report', async () => {
    // ✅ Click export button
    // ✅ CSV downloads
    // ✅ Data matches dashboard
  });

  test('drill-down into metrics', async () => {
    // ✅ Click chart data point
    // ✅ Detailed view opens
    // ✅ Related data shown
  });
});
```

### Batch Operations Flow
```typescript
// gui/citrate-core/src/__tests__/integration/batchOperations.test.tsx

describe('Batch Operations Flow', () => {
  test('batch update prices', async () => {
    // ✅ Navigate to "My Models"
    // ✅ Select 5 models with checkboxes
    // ✅ Batch toolbar appears
    // ✅ Click "Update Price"
    // ✅ Modal opens
    // ✅ Enter new price
    // ✅ Confirm changes
    // ✅ Progress indicator shows
    // ✅ All models updated
    // ✅ Undo toast appears
  });

  test('batch toggle status', async () => {
    // ✅ Select 3 models
    // ✅ Click "Toggle Active/Inactive"
    // ✅ Confirm action
    // ✅ Status toggled for all
    // ✅ Success message shown
  });

  test('undo batch operation', async () => {
    // ✅ Perform batch update
    // ✅ Click "Undo" within 60s
    // ✅ Changes reverted
    // ✅ Models return to original state
  });

  test('partial batch failure', async () => {
    // ✅ Some operations fail
    // ✅ Success count displayed
    // ✅ Failure count displayed
    // ✅ Error details shown
  });
});
```

### Mobile Experience Flow
```typescript
// gui/citrate-core/src/__tests__/integration/mobile.test.tsx

describe('Mobile Experience Flow', () => {
  test('mobile navigation', async () => {
    // ✅ Bottom nav bar visible
    // ✅ Tap Home icon → navigates
    // ✅ Tap Search icon → opens search
    // ✅ Tap Profile icon → opens profile
    // ✅ Active tab highlighted
  });

  test('mobile search', async () => {
    // ✅ Search bar optimized for mobile
    // ✅ Results in single column
    // ✅ Cards touch-friendly
    // ✅ Filters accessible
  });

  test('swipe gestures', async () => {
    // ✅ Swipe left on carousel → next item
    // ✅ Swipe right on carousel → previous item
    // ✅ Pull to refresh → refreshes list
  });

  test('touch interactions', async () => {
    // ✅ All tap targets ≥ 44px
    // ✅ No hover-dependent actions
    // ✅ Long press shows context menu
  });
});
```

---

## Performance Tests

### Load Performance
```typescript
// gui/citrate-core/performance-tests/load-test.js

describe('Load Performance', () => {
  test('marketplace page load', () => {
    // ✅ Initial load < 2s
    // ✅ First Contentful Paint < 2s
    // ✅ Time to Interactive < 3.5s
  });

  test('model comparison load (5 models)', () => {
    // ✅ Comparison table renders < 1s
    // ✅ No blocking operations
  });

  test('analytics dashboard load', () => {
    // ✅ Dashboard loads < 2s
    // ✅ Charts render < 500ms each
  });

  test('virtual scroll with 1000 models', () => {
    // ✅ Smooth 60 FPS scrolling
    // ✅ No frame drops
    // ✅ Memory usage < 100MB
  });
});
```

### Network Performance
```typescript
describe('Network Performance', () => {
  test('slow 3G performance', () => {
    // ✅ Page usable on 3G
    // ✅ Progressive loading works
    // ✅ Critical content loads first
  });

  test('service worker caching', () => {
    // ✅ Assets cached after first load
    // ✅ Second load from cache < 1s
    // ✅ Offline fallback works
  });

  test('API response caching', () => {
    // ✅ Repeated requests served from cache
    // ✅ TTL respected
    // ✅ Cache invalidation works
  });
});
```

### Memory Performance
```typescript
describe('Memory Performance', () => {
  test('no memory leaks with virtual scrolling', () => {
    // ✅ Memory stable during prolonged scrolling
    // ✅ No continuous growth
  });

  test('component unmounting releases memory', () => {
    // ✅ Memory released on unmount
    // ✅ Event listeners cleaned up
  });

  test('large comparison doesn't leak', () => {
    // ✅ Memory usage reasonable with 5 models
    // ✅ Cleanup on comparison clear
  });
});
```

### Bundle Size Tests
```bash
# Run bundle analysis
npm run analyze-bundle

# Targets:
# ✅ Initial bundle < 200KB gzipped
# ✅ Vendor chunks properly split
# ✅ No duplicate dependencies
# ✅ Tree shaking working
```

---

## Lighthouse Tests

### Lighthouse CI Configuration
```javascript
// gui/citrate-core/lighthouserc.js

module.exports = {
  ci: {
    collect: {
      numberOfRuns: 3,
      url: [
        'http://localhost:4173/',
        'http://localhost:4173/marketplace',
        'http://localhost:4173/compare',
        'http://localhost:4173/analytics',
      ],
    },
    assert: {
      assertions: {
        // Performance
        'categories:performance': ['error', { minScore: 0.9 }],
        'first-contentful-paint': ['error', { maxNumericValue: 2000 }],
        'interactive': ['error', { maxNumericValue: 3500 }],
        'speed-index': ['error', { maxNumericValue: 3000 }],

        // Accessibility
        'categories:accessibility': ['error', { minScore: 0.95 }],
        'color-contrast': 'error',
        'button-name': 'error',
        'link-name': 'error',

        // Best Practices
        'categories:best-practices': ['warn', { minScore: 0.9 }],

        // SEO
        'categories:seo': ['warn', { minScore: 0.9 }],
      },
    },
  },
};
```

### Mobile Lighthouse Tests
```
Run Lighthouse with mobile emulation:
- ✅ Performance > 90
- ✅ Accessibility > 95
- ✅ First Contentful Paint < 2s
- ✅ Largest Contentful Paint < 2.5s
- ✅ Total Blocking Time < 300ms
- ✅ Cumulative Layout Shift < 0.1
```

---

## Accessibility Tests

### Keyboard Navigation
```typescript
describe('Keyboard Accessibility', () => {
  test('comparison table navigation', () => {
    // ✅ Tab through comparison table
    // ✅ Arrow keys navigate cells
    // ✅ Enter to activate buttons
    // ✅ Escape to close modals
  });

  test('batch operations keyboard access', () => {
    // ✅ Space to toggle checkboxes
    // ✅ Tab to batch action buttons
    // ✅ Enter to confirm actions
  });

  test('mobile menu keyboard access', () => {
    // ✅ Tab through menu items
    // ✅ Enter to navigate
    // ✅ Escape to close menu
  });
});
```

### Screen Reader Tests
```typescript
describe('Screen Reader Support', () => {
  test('ARIA labels present', () => {
    // ✅ Comparison table has aria-label
    // ✅ Buttons have descriptive labels
    // ✅ Form inputs labeled
  });

  test('live regions announce changes', () => {
    // ✅ Comparison count announced
    // ✅ Batch operation result announced
    // ✅ Loading states announced
  });

  test('semantic HTML', () => {
    // ✅ Proper heading hierarchy
    // ✅ Lists use <ul>/<ol>
    // ✅ Tables use proper structure
  });
});
```

---

## Cross-Browser Tests

### Browser Compatibility Matrix
```
Test all features on:
- ✅ Chrome 120+ (Windows, Mac, Android)
- ✅ Firefox 120+ (Windows, Mac)
- ✅ Safari 17+ (Mac, iOS)
- ✅ Edge 120+ (Windows)

Features to verify:
- ✅ Model comparison works
- ✅ Analytics charts render
- ✅ Batch operations functional
- ✅ Virtual scrolling smooth
- ✅ Touch gestures work (mobile)
- ✅ Service worker active
- ✅ WebP images with fallback
```

---

## Mobile Device Tests

### Device Testing Matrix
```
Physical Devices:
- ✅ iPhone 15 (iOS 17)
- ✅ iPhone SE (iOS 17) - small screen
- ✅ iPad Pro (iOS 17) - tablet
- ✅ Samsung Galaxy S23 (Android 14)
- ✅ Google Pixel 7 (Android 14)

Emulator Tests:
- ✅ Various screen sizes (320px - 428px)
- ✅ Various pixel densities (1x, 2x, 3x)

Features to verify:
- ✅ Bottom navigation functional
- ✅ Touch targets ≥ 44px
- ✅ No horizontal scroll
- ✅ Forms usable (inputs ≥ 44px)
- ✅ Swipe gestures work
- ✅ Performance acceptable (60 FPS)
- ✅ Text readable without zoom
```

---

## Load Testing

### k6 Load Test Script
```javascript
// performance-tests/load-test.js

import http from 'k6/http';
import { check, sleep } from 'k6';

export const options = {
  stages: [
    { duration: '2m', target: 50 },
    { duration: '5m', target: 100 },
    { duration: '2m', target: 0 },
  ],
  thresholds: {
    http_req_duration: ['p(95)<2000'],
    http_req_failed: ['rate<0.01'],
  },
};

export default function () {
  // Test comparison endpoint
  const comparisonRes = http.get('http://localhost:8545/api/models?ids=1,2,3,4,5');
  check(comparisonRes, {
    'comparison loaded': (r) => r.status === 200,
    'comparison fast': (r) => r.timings.duration < 1000,
  });

  // Test analytics endpoint
  const analyticsRes = http.get('http://localhost:8545/api/analytics/creator/0x123');
  check(analyticsRes, {
    'analytics loaded': (r) => r.status === 200,
    'analytics fast': (r) => r.timings.duration < 2000,
  });

  // Test batch operation
  const batchRes = http.post('http://localhost:8545/api/batch/update', {
    modelIds: ['1', '2', '3'],
    updates: { basePrice: '1000000000000000000' },
  });
  check(batchRes, {
    'batch succeeded': (r) => r.status === 200,
    'batch fast': (r) => r.timings.duration < 3000,
  });

  sleep(1);
}
```

---

## Regression Tests

### Existing Functionality Verification
```
After Sprint 5 changes, verify all Sprint 4 features still work:
- ✅ Search functionality intact
- ✅ Filtering and sorting work
- ✅ Quality metrics display correctly
- ✅ Reviews and ratings functional
- ✅ Recommendations work
- ✅ Model details page functional
- ✅ Purchase flow works
- ✅ Wallet operations work
```

---

## Manual Test Scenarios

### Scenario 1: Power User Comparison Workflow
```
Steps:
1. Browse marketplace
2. Add 5 models to comparison
3. Navigate to comparison page
4. Toggle "Show only differences"
5. Export comparison as PDF
6. Share comparison link
7. Open link in new tab
8. Verify comparison loads correctly

Expected:
✅ All steps complete smoothly
✅ No errors or glitches
✅ PDF formatting correct
✅ Share link works
```

### Scenario 2: Creator Managing Models
```
Steps:
1. Log in as creator with 20 models
2. View analytics dashboard
3. Identify underperforming model
4. Navigate to "My Models"
5. Select 10 models with checkboxes
6. Batch update prices
7. Confirm changes
8. Verify all prices updated
9. Undo batch operation
10. Verify prices reverted

Expected:
✅ Analytics insights actionable
✅ Batch operations smooth
✅ Undo works correctly
✅ Audit log updated
```

### Scenario 3: Mobile Shopping Experience
```
Steps (on mobile device):
1. Open marketplace on phone
2. Swipe through featured models
3. Search for specific model
4. View model details
5. Add to comparison
6. Compare 3 models
7. Make purchase decision
8. Complete purchase

Expected:
✅ Touch interactions smooth
✅ Navigation intuitive
✅ Performance fast (60 FPS)
✅ No horizontal scrolling
✅ Purchase flow works
```

### Scenario 4: Large Catalog Performance
```
Steps:
1. Load marketplace with 1000+ models
2. Scroll through entire catalog
3. Apply various filters
4. Change sort order multiple times
5. Search for models
6. Monitor performance metrics

Expected:
✅ Smooth 60 FPS scrolling
✅ No lag or stuttering
✅ Memory usage stable
✅ Filters apply quickly
✅ Search responds instantly
```

---

## Testing Tools & Commands

### Run All Tests
```bash
# Unit tests
npm test

# Integration tests
npm run test:integration

# E2E tests
npm run test:e2e

# Performance tests
npm run load-test
npm run lighthouse

# Coverage
npm run test:coverage

# All tests
npm run test:all
```

### Continuous Testing
```bash
# Watch mode for development
npm run test:watch

# CI mode (single run)
npm run test:ci
```

---

## Test Coverage Requirements

### Coverage Targets
- **Overall Coverage:** >85%
- **Unit Test Coverage:** >90%
- **Integration Test Coverage:** >80%
- **Critical Paths:** 100%

### Coverage Report
```bash
npm run test:coverage

# View HTML report
open coverage/index.html
```

---

## Definition of Done (Testing)

A feature is considered "Done" from testing perspective when:

- [ ] All unit tests pass (>85% coverage)
- [ ] All integration tests pass
- [ ] All E2E tests pass
- [ ] Performance benchmarks met
- [ ] Lighthouse scores meet targets (Performance > 90, Accessibility > 95)
- [ ] Load testing completed successfully
- [ ] Security tests pass
- [ ] Accessibility tests pass (WCAG 2.1 AA)
- [ ] Manual test scenarios pass
- [ ] Cross-browser tests pass (Chrome, Firefox, Safari, Edge)
- [ ] Mobile device tests pass (iOS, Android)
- [ ] No critical bugs open
- [ ] No P0/P1 bugs open
- [ ] Regression tests pass (Sprint 4 features work)
- [ ] Code coverage >85%
- [ ] All tests documented
- [ ] Performance metrics within targets

---

**Document Version:** 1.0
**Last Updated:** February 25, 2026
**Status:** ✅ Ready for Testing
