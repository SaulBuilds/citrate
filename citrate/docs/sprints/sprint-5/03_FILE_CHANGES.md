# Sprint 5: File Changes - Advanced Marketplace Features & Optimization

**Sprint Goal:** Build advanced marketplace features and optimize performance

---

## New Files to Create

### GUI / Frontend

#### Comparison Components
```
gui/citrate-core/src/components/comparison/
├── ComparisonTable.tsx           # Main side-by-side comparison table
├── CompareButton.tsx            # Button to add/remove models from comparison
├── ComparisonPage.tsx           # Full comparison page layout
├── ComparisonToolbar.tsx        # Actions toolbar (export, share, clear)
└── ComparisonSkeleton.tsx       # Loading skeleton for comparison

gui/citrate-core/src/hooks/
├── useComparison.ts             # Comparison state management hook
└── useComparisonData.ts         # Fetch comparison model data

gui/citrate-core/src/utils/
├── exportComparison.ts          # CSV and PDF export utilities
└── comparisonHelpers.ts         # Comparison calculation helpers
```

#### Analytics Components
```
gui/citrate-core/src/components/analytics/
├── CreatorDashboard.tsx         # Main analytics dashboard
├── RevenueSection.tsx           # Revenue analytics section
├── UsageSection.tsx             # Usage pattern analysis section
├── DemographicsSection.tsx      # User demographics section
├── ModelPerformanceSection.tsx  # Model performance comparison
├── UsageHeatmap.tsx             # Peak usage times heatmap
├── AnalyticsSummaryCard.tsx     # Summary metric cards
├── ExportReportButton.tsx       # Export analytics report
└── TimeWindowSelector.tsx       # Date range selector

gui/citrate-core/src/hooks/
├── useCreatorAnalytics.ts       # Analytics data fetching hook
└── useAnalyticsExport.ts        # Export functionality hook

gui/citrate-core/src/services/
└── analytics/
    ├── aggregator.ts            # Analytics aggregation logic
    ├── calculator.ts            # Metrics calculation utilities
    └── api.ts                   # Analytics API client
```

#### Batch Operations Components
```
gui/citrate-core/src/components/batch/
├── BatchActionsToolbar.tsx      # Batch actions toolbar
├── BatchUpdateModal.tsx         # Modal for batch operations
├── BatchConfirmationDialog.tsx  # Confirmation before batch action
├── BatchProgressIndicator.tsx   # Progress indicator for batch ops
├── UndoToast.tsx               # Undo notification toast
└── BatchAuditLog.tsx           # Audit log viewer

gui/citrate-core/src/hooks/
├── useBatchSelection.ts         # Multi-select state management
├── useBatchOperations.ts        # Batch operation execution
└── useBatchUndo.ts             # Undo functionality

gui/citrate-core/src/services/
└── batch/
    ├── operations.ts            # Batch operation implementations
    └── api.ts                   # Batch API client
```

#### Mobile Components
```
gui/citrate-core/src/components/mobile/
├── MobileNavigation.tsx         # Bottom navigation bar
├── MobileHeader.tsx            # Mobile-optimized header
├── HamburgerMenu.tsx           # Side drawer menu
├── MobileSearchBar.tsx         # Mobile search interface
├── TouchGestureWrapper.tsx     # Wrapper for swipe gestures
└── PullToRefresh.tsx           # Pull-to-refresh component

gui/citrate-core/src/hooks/
├── useSwipeGesture.ts          # Swipe gesture hook
├── useMobileDetection.ts       # Detect mobile device
└── useTouchEvents.ts           # Touch event handlers
```

#### Performance Components
```
gui/citrate-core/src/components/performance/
├── VirtualizedModelList.tsx    # Virtualized list for large catalogs
├── LazyLoadImage.tsx          # Lazy loading image component
├── OptimizedImage.tsx         # WebP optimized image component
├── ImageSkeleton.tsx          # Image loading skeleton
└── PerformanceMonitor.tsx     # Performance monitoring component

gui/citrate-core/src/hooks/
├── useVirtualization.ts       # Virtualization hook
├── useLazyLoad.ts            # Lazy loading hook
└── usePerformance.ts         # Performance monitoring hook

gui/citrate-core/src/utils/
├── imageOptimization.ts      # Image optimization utilities
├── bundleOptimization.ts     # Bundle size optimization
└── cacheManager.ts           # Cache management utilities
```

### Types
```
gui/citrate-core/src/types/
├── comparison.ts              # Comparison-related types
├── analytics.ts               # Analytics types
├── batch.ts                   # Batch operation types
├── mobile.ts                  # Mobile-specific types
└── performance.ts             # Performance monitoring types
```

### Styles
```
gui/citrate-core/src/styles/
├── comparison.css             # Comparison component styles
├── analytics.css              # Analytics dashboard styles
├── batch.css                  # Batch operations styles
├── mobile.css                 # Mobile-specific styles
├── responsive.css             # Responsive breakpoints
├── touch.css                  # Touch-friendly UI styles
└── performance.css            # Performance-related styles
```

### Service Worker
```
gui/citrate-core/
├── serviceWorker.ts           # Service worker implementation
└── workbox-config.js          # Workbox configuration
```

### Utilities
```
gui/citrate-core/src/utils/
├── format.ts                  # Format utilities (currency, dates, etc.)
├── validation.ts              # Validation utilities
├── memoization.ts             # Memoization helpers
└── errorHandling.ts           # Error handling utilities
```

---

## Files to Modify

### GUI / Frontend

#### Core Application Files
```
gui/citrate-core/src/App.tsx
├── Add: Comparison page route
├── Add: Analytics dashboard route
├── Add: Mobile navigation component
├── Add: Service worker registration
└── Update: Performance optimization wrappers

gui/citrate-core/src/components/Marketplace.tsx
├── Add: Batch selection mode toggle
├── Add: BatchActionsToolbar integration
├── Add: VirtualizedModelList integration
└── Update: Responsive grid layout

gui/citrate-core/src/components/ModelCard.tsx
├── Add: Batch selection checkbox
├── Add: CompareButton integration
├── Add: Touch-friendly interactions
├── Add: Lazy loading images
└── Update: Mobile-optimized layout

gui/citrate-core/src/components/ModelDetails.tsx
├── Add: "Add to Compare" button
├── Add: Link to creator analytics (if owner)
├── Update: Mobile-optimized layout
└── Update: Optimized image loading

gui/citrate-core/src/components/Models.tsx
├── Add: Batch operations toolbar
├── Add: Virtual scrolling
├── Update: Responsive grid
└── Update: Performance optimizations
```

#### Layout Components
```
gui/citrate-core/src/components/Layout.tsx
├── Add: Mobile navigation integration
├── Add: Responsive breakpoint handling
└── Update: Mobile-first approach

gui/citrate-core/src/components/Header.tsx
├── Add: Mobile header variant
├── Add: Hamburger menu trigger
└── Update: Responsive design

gui/citrate-core/src/components/Navigation.tsx
├── Add: Mobile bottom nav
├── Add: Conditional rendering (desktop vs mobile)
└── Update: Touch-friendly sizing
```

#### Existing Components to Optimize
```
gui/citrate-core/src/components/SearchBar.tsx
├── Add: React.memo optimization
├── Add: useCallback for handlers
└── Update: Mobile-optimized input

gui/citrate-core/src/components/SearchResults.tsx
├── Add: Virtual scrolling for large results
├── Update: Responsive grid layout
└── Update: Performance optimizations

gui/citrate-core/src/components/MetricsDashboard.tsx
├── Add: Memoization for expensive charts
├── Update: Mobile-responsive charts
└── Update: Lazy load chart libraries
```

#### Styling Updates
```
gui/citrate-core/src/styles/globals.css
├── Add: Mobile-first base styles
├── Add: Touch-friendly sizing variables
├── Add: Responsive typography
└── Update: Dark mode mobile adjustments

gui/citrate-core/src/styles/variables.css
├── Add: Breakpoint variables
├── Add: Mobile-specific spacing
└── Add: Touch target size variables
```

#### Configuration
```
gui/citrate-core/vite.config.ts
├── Add: Bundle size optimization plugins
├── Add: Image optimization plugins
├── Add: Service worker plugin (workbox)
└── Update: Build optimization settings

gui/citrate-core/package.json
├── Add: New dependencies (react-window, use-gesture, jspdf, etc.)
├── Update: Scripts for performance testing
└── Add: Lighthouse CI configuration

gui/citrate-core/tsconfig.json
├── Update: Strict mode enabled
└── Add: Path aliases for new directories
```

---

## Backend / Core API (Optional Optimizations)

### Query Optimization
```
core/api/src/routes/models.rs
├── Add: Pagination optimization
├── Add: Query result caching
└── Update: Index-based queries

core/api/src/routes/analytics.rs (NEW)
├── Creator analytics endpoints
├── Aggregated metrics endpoints
└── Export report endpoints

core/storage/src/state.rs
├── Add: Indexes for frequently queried fields
├── Add: Query result cache
└── Update: Pagination logic
```

---

## Smart Contracts (Optional)

### Batch Operations Contract Enhancement
```
contracts/src/ModelMarketplace.sol
├── Add: batchUpdateModels() function (gas-optimized)
├── Add: batchToggleActive() function
└── Events for batch operations

contracts/test/ModelMarketplace.t.sol
├── Add: Batch operation tests
└── Gas optimization tests
```

---

## Database Schema Changes

### RocksDB Key Structure (Analytics)
```
# Analytics aggregates (pre-computed)
analytics:{creator_address}:{time_window} → CreatorAnalytics (bincode)
analytics_updated:{creator_address} → timestamp

# Revenue tracking
revenue:{creator_address}:{model_id}:{timestamp} → amount
revenue_daily:{creator_address}:{date} → total_amount

# Usage tracking
usage:{creator_address}:{model_id}:{timestamp} → inference_count
usage_hourly:{date}:{hour} → inference_count (for heatmap)

# User tracking
user_first_seen:{user_address} → timestamp
user_last_seen:{user_address} → timestamp
```

### IndexedDB Schema (Frontend)
```javascript
const comparisonDB = {
  name: 'citrate_comparison',
  version: 1,
  stores: [
    {
      name: 'comparison_history',
      keyPath: 'id',
      autoIncrement: true,
    },
    {
      name: 'comparison_cache',
      keyPath: 'key',
    }
  ]
};

const performanceDB = {
  name: 'citrate_performance',
  version: 1,
  stores: [
    {
      name: 'performance_metrics',
      keyPath: 'timestamp',
    },
    {
      name: 'error_logs',
      keyPath: 'id',
      autoIncrement: true,
    }
  ]
};
```

---

## Configuration Changes

### Vite Configuration
```javascript
// gui/citrate-core/vite.config.ts
export default defineConfig({
  plugins: [
    react(),
    // Add service worker plugin
    VitePWA({
      registerType: 'autoUpdate',
      workbox: {
        globPatterns: ['**/*.{js,css,html,ico,png,svg,webp}'],
        runtimeCaching: [
          {
            urlPattern: /^https:\/\/ipfs\.io\/.*/i,
            handler: 'CacheFirst',
            options: {
              cacheName: 'ipfs-cache',
              expiration: {
                maxEntries: 100,
                maxAgeSeconds: 60 * 60 * 24 * 30, // 30 days
              },
            },
          },
        ],
      },
    }),
    // Add image optimization
    imageOptimizer(),
  ],
  build: {
    // Code splitting
    rollupOptions: {
      output: {
        manualChunks: {
          'react-vendor': ['react', 'react-dom', 'react-router-dom'],
          'charts': ['recharts'],
          'comparison': ['jspdf', 'papaparse'],
          'virtualization': ['react-window'],
        },
      },
    },
    // Minification
    minify: 'terser',
    terserOptions: {
      compress: {
        drop_console: true,
      },
    },
    // Target modern browsers
    target: 'es2020',
  },
});
```

### Lighthouse CI Configuration
```javascript
// gui/citrate-core/lighthouserc.js
module.exports = {
  ci: {
    collect: {
      numberOfRuns: 3,
      startServerCommand: 'npm run preview',
      url: ['http://localhost:4173/', 'http://localhost:4173/marketplace'],
    },
    assert: {
      preset: 'lighthouse:recommended',
      assertions: {
        'categories:performance': ['error', { minScore: 0.9 }],
        'categories:accessibility': ['error', { minScore: 0.95 }],
        'first-contentful-paint': ['error', { maxNumericValue: 2000 }],
        'interactive': ['error', { maxNumericValue: 3500 }],
      },
    },
    upload: {
      target: 'temporary-public-storage',
    },
  },
};
```

---

## Test Files to Create

### Frontend Tests
```
gui/citrate-core/src/__tests__/
├── comparison/
│   ├── ComparisonTable.test.tsx
│   ├── CompareButton.test.tsx
│   ├── useComparison.test.ts
│   └── exportComparison.test.ts
├── analytics/
│   ├── CreatorDashboard.test.tsx
│   ├── RevenueSection.test.tsx
│   ├── UsageSection.test.tsx
│   └── DemographicsSection.test.tsx
├── batch/
│   ├── BatchActionsToolbar.test.tsx
│   ├── BatchUpdateModal.test.tsx
│   ├── useBatchSelection.test.ts
│   └── useBatchUndo.test.ts
├── mobile/
│   ├── MobileNavigation.test.tsx
│   ├── useSwipeGesture.test.ts
│   └── useMobileDetection.test.ts
└── performance/
    ├── VirtualizedModelList.test.tsx
    ├── OptimizedImage.test.tsx
    └── useVirtualization.test.ts
```

### Performance Tests
```
gui/citrate-core/performance-tests/
├── load-test.js               # k6 load testing script
├── lighthouse-test.js         # Lighthouse CI test
└── bundle-size-test.js        # Bundle size monitoring
```

### E2E Tests
```
gui/citrate-core/e2e/
├── comparison.spec.ts         # Model comparison E2E tests
├── analytics.spec.ts          # Analytics dashboard E2E tests
├── batch-operations.spec.ts   # Batch operations E2E tests
└── mobile.spec.ts            # Mobile experience E2E tests
```

---

## Documentation Updates

### User Documentation
```
docs/guides/
├── model-comparison.md        # How to compare models
├── creator-analytics.md       # Understanding analytics dashboard
├── batch-operations.md        # Guide to batch model management
├── mobile-app.md             # Using Citrate on mobile
└── performance-tips.md       # Performance best practices
```

### Developer Documentation
```
docs/technical/
├── comparison-architecture.md    # Comparison system design
├── analytics-pipeline.md         # Analytics data flow
├── batch-operations-api.md       # Batch operations API
├── mobile-optimization.md        # Mobile optimization guide
├── performance-optimization.md   # Performance optimization strategies
└── virtualization-guide.md       # Virtualization implementation
```

### API Documentation
```
docs/api/
├── comparison-api.md          # Comparison endpoints
├── analytics-api.md           # Analytics endpoints
└── batch-api.md              # Batch operation endpoints
```

---

## File Change Summary

### Statistics
- **New Frontend Files:** ~80 files
- **New Backend Files:** ~5 files (optional)
- **Modified Frontend Files:** ~20 files
- **Modified Backend Files:** ~3 files (optional)
- **New Test Files:** ~35 files
- **New Documentation:** ~12 files
- **New Configuration Files:** ~3 files

### Total Lines of Code (Estimate)
- **Frontend Components:** ~5,000 lines
- **Frontend Hooks/Utils:** ~2,000 lines
- **Frontend Tests:** ~2,500 lines
- **Styles:** ~1,500 lines
- **Backend (optional):** ~800 lines
- **Documentation:** ~2,000 lines
- **Configuration:** ~500 lines
- **Total:** ~14,300 lines

---

## Git Commit Strategy

### Recommended Commit Structure

#### Day 1: Model Comparison
```bash
git commit -m "feat(comparison): add comparison types and data structures"
git commit -m "feat(comparison): implement ComparisonTable component"
git commit -m "feat(comparison): add CompareButton and state management"
git commit -m "feat(comparison): implement export functionality (CSV, PDF)"
git commit -m "test(comparison): add comprehensive comparison tests"
```

#### Day 2: Creator Analytics
```bash
git commit -m "feat(analytics): add analytics types and data structures"
git commit -m "feat(analytics): implement CreatorDashboard component"
git commit -m "feat(analytics): add revenue charts and visualization"
git commit -m "feat(analytics): implement usage pattern analysis"
git commit -m "feat(analytics): add user demographics section"
```

#### Day 3: Batch Operations
```bash
git commit -m "feat(batch): add batch operation types and state management"
git commit -m "feat(batch): implement BatchActionsToolbar"
git commit -m "feat(batch): add multi-select functionality"
git commit -m "feat(batch): create batch update forms and modals"
git commit -m "feat(batch): implement undo capability and audit log"
```

#### Day 4: Mobile Optimization
```bash
git commit -m "feat(mobile): audit mobile experience and identify improvements"
git commit -m "feat(mobile): optimize responsive breakpoints and layouts"
git commit -m "feat(mobile): implement mobile navigation patterns"
git commit -m "feat(mobile): add touch gesture support"
git commit -m "perf(mobile): optimize images and implement lazy loading"
git commit -m "test(mobile): mobile performance testing and optimization"
```

#### Day 5: Performance & Polish
```bash
git commit -m "perf: implement virtualized lists for large catalogs"
git commit -m "perf: optimize React re-renders with memoization"
git commit -m "perf: add service worker for asset caching"
git commit -m "perf: database query optimization and indexing"
git commit -m "perf: load testing and performance benchmarking"
git commit -m "chore: bug fixes, polish, and documentation updates"
git commit -m "docs: Sprint 5 completion and summary"
```

---

## Migration Scripts

### Database Migration
```
scripts/migrations/
└── sprint5_add_analytics_tables.rs
    ├── Create analytics aggregation tables
    ├── Create batch operation audit log
    └── Add performance monitoring tables
```

---

## Asset Optimization

### Image Optimization
```bash
# Generate WebP versions of all images
npm run optimize-images

# Generate responsive variants (@2x, @3x)
npm run generate-responsive-images

# Compress images
npm run compress-images
```

### Font Optimization
```bash
# Subset fonts to include only used glyphs
npm run subset-fonts

# Generate WOFF2 versions
npm run convert-fonts
```

### Bundle Optimization
```bash
# Analyze bundle size
npm run analyze-bundle

# Generate bundle report
npm run bundle-report
```

---

## Environment Variables

### Add to `.env`
```bash
# Analytics Configuration
VITE_ANALYTICS_ENABLED=true
VITE_ANALYTICS_REFRESH_INTERVAL=60000

# Performance Monitoring
VITE_PERFORMANCE_MONITORING=true
VITE_LIGHTHOUSE_CI=true

# Service Worker
VITE_SERVICE_WORKER_ENABLED=true
VITE_CACHE_VERSION=v1

# Mobile Optimization
VITE_MOBILE_OPTIMIZATION=true
VITE_TOUCH_GESTURES=true

# Feature Flags
VITE_FEATURE_COMPARISON=true
VITE_FEATURE_ANALYTICS=true
VITE_FEATURE_BATCH_OPS=true
```

---

## Build Pipeline Updates

### Package.json Scripts
```json
{
  "scripts": {
    "dev": "vite",
    "build": "tsc && vite build",
    "build:analyze": "vite build --mode analyze",
    "preview": "vite preview",
    "test": "vitest",
    "test:coverage": "vitest --coverage",
    "test:e2e": "playwright test",
    "lighthouse": "lhci autorun",
    "load-test": "k6 run performance-tests/load-test.js",
    "optimize-images": "node scripts/optimize-images.js",
    "analyze-bundle": "vite-bundle-visualizer"
  }
}
```

---

**Document Version:** 1.0
**Last Updated:** February 25, 2026
**Status:** ✅ Ready for Implementation
