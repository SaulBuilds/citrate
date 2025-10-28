# Sprint 2: Testing Checklist

**Sprint Goal:** Comprehensive testing for state persistence, dark mode, performance, and accessibility

**Last Updated:** [Sprint Start Date]

---

## Testing Summary

| Test Category | Total Tests | Passing | Failing | Pending |
|---------------|-------------|---------|---------|---------|
| Unit Tests | 35 | 0 | 0 | 35 |
| Integration Tests | 12 | 0 | 0 | 12 |
| Performance Tests | 8 | 0 | 0 | 8 |
| Accessibility Tests | 10 | 0 | 0 | 10 |
| Manual Tests | 45 | 0 | 0 | 45 |
| **Total** | **110** | **0** | **0** | **110** |

---

## Unit Tests

### Story 1: State Persistence

#### Storage Utility Tests
**File:** `gui/citrate-core/src/utils/storage.test.ts`

```typescript
describe('Storage Utility', () => {
  beforeEach(() => {
    localStorage.clear();
  });

  describe('getStorageItem / setStorageItem', () => {
    it('should store and retrieve theme', () => {
      // ⏳
    });

    it('should return null for non-existent keys', () => {
      // ⏳
    });

    it('should handle objects correctly', () => {
      // ⏳
    });

    it('should handle arrays correctly', () => {
      // ⏳
    });

    it('should handle null values', () => {
      // ⏳
    });

    it('should handle undefined gracefully', () => {
      // ⏳
    });
  });

  describe('addRecentAddress', () => {
    it('should add address to list', () => {
      // ⏳
    });

    it('should limit to 10 addresses', () => {
      // ⏳
    });

    it('should not duplicate addresses', () => {
      // ⏳
    });

    it('should be case-insensitive', () => {
      // ⏳
    });

    it('should add new address to beginning', () => {
      // ⏳
    });
  });

  describe('clearRecentAddresses', () => {
    it('should clear all addresses', () => {
      // ⏳
    });

    it('should not affect other storage', () => {
      // ⏳
    });
  });

  describe('exportSettings', () => {
    it('should export settings as JSON', () => {
      // ⏳
    });

    it('should include version number', () => {
      // ⏳
    });

    it('should include export date', () => {
      // ⏳
    });

    it('should include all settings', () => {
      // ⏳
    });
  });

  describe('importSettings', () => {
    it('should import valid settings', () => {
      // ⏳
    });

    it('should reject invalid JSON', () => {
      // ⏳
    });

    it('should handle version mismatch', () => {
      // ⏳
    });

    it('should validate settings structure', () => {
      // ⏳
    });
  });

  describe('migrateStorage', () => {
    it('should migrate from v0 to v1', () => {
      // ⏳
    });

    it('should not lose data during migration', () => {
      // ⏳
    });

    it('should update version number', () => {
      // ⏳
    });
  });

  describe('Error Handling', () => {
    it('should handle quota exceeded error', () => {
      // ⏳
    });

    it('should clear old data when quota exceeded', () => {
      // ⏳
    });

    it('should return false on storage failure', () => {
      // ⏳
    });
  });
});
```

**Status:** ⏳ Pending
**Total Tests:** 23

---

### Story 2: Dark Mode Theme

#### Theme Context Tests
**File:** `gui/citrate-core/src/contexts/ThemeContext.test.tsx`

```typescript
describe('ThemeContext', () => {
  beforeEach(() => {
    localStorage.clear();
  });

  describe('Theme Initialization', () => {
    it('should apply light theme by default', () => {
      // ⏳
    });

    it('should load theme from localStorage', () => {
      // ⏳
    });

    it('should detect system theme when mode is "system"', () => {
      // ⏳
    });
  });

  describe('Theme Switching', () => {
    it('should toggle theme', () => {
      // ⏳
    });

    it('should switch to dark theme', () => {
      // ⏳
    });

    it('should switch to light theme', () => {
      // ⏳
    });

    it('should persist theme to localStorage', () => {
      // ⏳
    });

    it('should apply CSS variables on theme change', () => {
      // ⏳
    });
  });

  describe('System Theme Detection', () => {
    it('should detect dark mode preference', () => {
      // ⏳
    });

    it('should detect light mode preference', () => {
      // ⏳
    });

    it('should listen for system theme changes', () => {
      // ⏳
    });

    it('should clean up event listeners', () => {
      // ⏳
    });
  });
});
```

**Status:** ⏳ Pending
**Total Tests:** 12

---

### Story 3: Performance Optimization

#### Debounce Hook Tests
**File:** `gui/citrate-core/src/hooks/useDebounce.test.ts`

```typescript
describe('useDebounce', () => {
  it('should return initial value immediately', () => {
    // ⏳
  });

  it('should debounce value changes', async () => {
    // ⏳
  });

  it('should use custom delay', async () => {
    // ⏳
  });

  it('should cancel pending debounce on unmount', () => {
    // ⏳
  });

  it('should handle rapid changes correctly', async () => {
    // ⏳
  });

  it('should work with different types', () => {
    // ⏳
  });
});
```

**Status:** ⏳ Pending
**Total Tests:** 6

---

#### Web Worker Tests (Optional)
**File:** `gui/citrate-core/src/workers/dagWorker.test.ts`

```typescript
describe('DAG Web Worker', () => {
  let worker: Worker;

  beforeEach(() => {
    worker = new Worker(new URL('../dagWorker.ts', import.meta.url));
  });

  afterEach(() => {
    worker.terminate();
  });

  it('should calculate blue set', (done) => {
    // ⏳
  });

  it('should perform topological sort', (done) => {
    // ⏳
  });

  it('should handle invalid input', (done) => {
    // ⏳
  });

  it('should handle empty block list', (done) => {
    // ⏳
  });
});
```

**Status:** ⏳ Pending (Optional)
**Total Tests:** 4

---

### Story 4: Accessibility

#### Keyboard Shortcuts Tests
**File:** `gui/citrate-core/src/hooks/useKeyboardShortcuts.test.ts`

```typescript
describe('useKeyboardShortcuts', () => {
  it('should call handler when shortcut pressed', () => {
    // ⏳
  });

  it('should match Ctrl key', () => {
    // ⏳
  });

  it('should match Shift key', () => {
    // ⏳
  });

  it('should match Alt key', () => {
    // ⏳
  });

  it('should prevent default behavior', () => {
    // ⏳
  });

  it('should handle multiple shortcuts', () => {
    // ⏳
  });

  it('should clean up event listeners', () => {
    // ⏳
  });

  it('should not conflict with browser shortcuts', () => {
    // ⏳
  });
});
```

**Status:** ⏳ Pending
**Total Tests:** 8

---

## Integration Tests

### Story 1: State Persistence

#### AppContext Integration
**File:** `gui/citrate-core/src/contexts/AppContext.test.tsx`

```typescript
describe('AppContext Integration', () => {
  it('should persist tab across remount', () => {
    // ⏳
  });

  it('should restore tab on app start', () => {
    // ⏳
  });

  it('should sync recent addresses', () => {
    // ⏳
  });

  it('should auto-save settings', () => {
    // ⏳
  });

  it('should handle storage quota exceeded', () => {
    // ⏳
  });
});
```

**Status:** ⏳ Pending
**Total Tests:** 5

---

### Story 2: Dark Mode Theme

#### Theme Integration
**File:** `gui/citrate-core/src/__tests__/theme-integration.test.tsx`

```typescript
describe('Theme Integration', () => {
  it('should apply theme to all components', () => {
    // ⏳
  });

  it('should transition smoothly between themes', () => {
    // ⏳
  });

  it('should persist theme across navigation', () => {
    // ⏳
  });

  it('should match loading skeletons to theme', () => {
    // ⏳
  });
});
```

**Status:** ⏳ Pending
**Total Tests:** 4

---

### Story 3: Performance Optimization

#### Virtual Scrolling Integration
**File:** `gui/citrate-core/src/__tests__/virtual-scrolling.test.tsx`

```typescript
describe('Virtual Scrolling Integration', () => {
  it('should render only visible items', () => {
    // ⏳
  });

  it('should handle scroll events', () => {
    // ⏳
  });

  it('should work with 1000+ items', () => {
    // ⏳
  });
});
```

**Status:** ⏳ Pending
**Total Tests:** 3

---

## Performance Tests

### Rendering Performance
**File:** `gui/citrate-core/src/__tests__/performance.test.ts`

```typescript
describe('Performance Tests', () => {
  describe('Dashboard', () => {
    it('should render in less than 100ms', () => {
      // ⏳
    });

    it('should maintain 60fps during interactions', () => {
      // ⏳
    });
  });

  describe('Wallet with Virtual Scrolling', () => {
    it('should render 1000 items in less than 100ms', () => {
      // ⏳
    });

    it('should maintain 60fps during scroll', () => {
      // ⏳
    });
  });

  describe('DAG Visualization', () => {
    it('should render 1000 blocks in less than 200ms', () => {
      // ⏳
    });

    it('should handle lazy loading efficiently', () => {
      // ⏳
    });
  });

  describe('Theme Switching', () => {
    it('should switch themes in less than 50ms', () => {
      // ⏳
    });

    it('should not cause layout thrashing', () => {
      // ⏳
    });
  });
});
```

**Status:** ⏳ Pending
**Total Tests:** 8

---

## Accessibility Tests

### Automated Accessibility
**File:** `gui/citrate-core/src/__tests__/accessibility.test.tsx`

```typescript
import { axe, toHaveNoViolations } from 'jest-axe';

expect.extend(toHaveNoViolations);

describe('Accessibility Tests', () => {
  describe('WCAG Compliance', () => {
    it('should have no accessibility violations in Dashboard', async () => {
      // ⏳
    });

    it('should have no accessibility violations in Wallet', async () => {
      // ⏳
    });

    it('should have no accessibility violations in Settings', async () => {
      // ⏳
    });

    it('should have no accessibility violations in DAG', async () => {
      // ⏳
    });

    it('should have no accessibility violations in Models', async () => {
      // ⏳
    });
  });

  describe('Color Contrast', () => {
    it('should meet WCAG AA contrast in light theme', () => {
      // ⏳
    });

    it('should meet WCAG AA contrast in dark theme', () => {
      // ⏳
    });
  });

  describe('Keyboard Navigation', () => {
    it('should be fully keyboard navigable', () => {
      // ⏳
    });

    it('should have visible focus indicators', () => {
      // ⏳
    });

    it('should trap focus in modals', () => {
      // ⏳
    });
  });
});
```

**Status:** ⏳ Pending
**Total Tests:** 10

---

## Manual Testing

### Story 1: State Persistence (10 tests)

#### Test 1.1: Theme Persistence
**Priority:** P0
- [ ] Set theme to dark
- [ ] Close and reopen app
- [ ] Verify dark theme is applied
- [ ] Switch to light theme
- [ ] Refresh page
- [ ] Verify light theme persisted

**Expected:** Theme persists across sessions
**Actual:** _____
**Status:** ⏳ Pending

---

#### Test 1.2: Tab Persistence
**Priority:** P0
- [ ] Navigate to Wallet tab
- [ ] Close and reopen app
- [ ] Verify Wallet tab is active
- [ ] Navigate to Settings
- [ ] Refresh page
- [ ] Verify Settings tab is active

**Expected:** Last viewed tab restored on app start
**Actual:** _____
**Status:** ⏳ Pending

---

#### Test 1.3: Window State Persistence (Tauri)
**Priority:** P1
- [ ] Resize window to 1200x800
- [ ] Move window to custom position
- [ ] Close app
- [ ] Reopen app
- [ ] Verify window size is 1200x800
- [ ] Verify window position matches

**Expected:** Window size and position restored
**Actual:** _____
**Status:** ⏳ Pending

---

#### Test 1.4: Recent Addresses
**Priority:** P1
- [ ] Send transaction to address A
- [ ] Send transaction to address B
- [ ] Open send form
- [ ] Click on recipient field
- [ ] Verify addresses A and B appear in dropdown
- [ ] Select address A from dropdown
- [ ] Verify field populated with address A

**Expected:** Recent addresses saved and accessible
**Actual:** _____
**Status:** ⏳ Pending

---

#### Test 1.5: Recent Addresses Limit
**Priority:** P2
- [ ] Send transactions to 15 different addresses
- [ ] Open send form dropdown
- [ ] Verify only 10 most recent addresses shown
- [ ] Verify oldest addresses removed

**Expected:** Max 10 addresses maintained
**Actual:** _____
**Status:** ⏳ Pending

---

#### Test 1.6: Settings Auto-Save
**Priority:** P1
- [ ] Open Settings
- [ ] Change RPC URL
- [ ] Close Settings (don't save manually)
- [ ] Reopen Settings
- [ ] Verify RPC URL persisted

**Expected:** Settings auto-saved on change
**Actual:** _____
**Status:** ⏳ Pending

---

#### Test 1.7: Export Settings
**Priority:** P2
- [ ] Configure custom settings
- [ ] Click "Export Settings"
- [ ] Verify JSON file downloaded
- [ ] Open JSON file
- [ ] Verify all settings included
- [ ] Verify version number present

**Expected:** Valid settings JSON exported
**Actual:** _____
**Status:** ⏳ Pending

---

#### Test 1.8: Import Settings
**Priority:** P2
- [ ] Export current settings
- [ ] Change some settings
- [ ] Click "Import Settings"
- [ ] Select exported JSON file
- [ ] Verify confirmation dialog
- [ ] Confirm import
- [ ] Verify settings restored

**Expected:** Settings imported correctly
**Actual:** _____
**Status:** ⏳ Pending

---

#### Test 1.9: Import Invalid Settings
**Priority:** P2
- [ ] Create invalid JSON file
- [ ] Attempt to import
- [ ] Verify error message shown
- [ ] Verify current settings unchanged

**Expected:** Invalid imports rejected with error
**Actual:** _____
**Status:** ⏳ Pending

---

#### Test 1.10: Storage Quota Exceeded
**Priority:** P3
- [ ] Fill localStorage to capacity (simulate)
- [ ] Attempt to save settings
- [ ] Verify graceful handling
- [ ] Verify old data cleared
- [ ] Verify essential data retained

**Expected:** Quota exceeded handled gracefully
**Actual:** _____
**Status:** ⏳ Pending

---

### Story 2: Dark Mode Theme (12 tests)

#### Test 2.1: Theme Toggle Light to Dark
**Priority:** P0
- [ ] Open Settings
- [ ] Click "Dark" theme button
- [ ] Verify theme switches immediately
- [ ] Verify no flicker
- [ ] Verify all components update
- [ ] Verify smooth transition

**Expected:** Instant, smooth theme switch
**Actual:** _____
**Status:** ⏳ Pending

---

#### Test 2.2: Theme Toggle Dark to Light
**Priority:** P0
- [ ] Set theme to dark
- [ ] Click "Light" theme button
- [ ] Verify theme switches immediately
- [ ] Verify all colors update
- [ ] Verify smooth transition

**Expected:** Instant, smooth theme switch
**Actual:** _____
**Status:** ⏳ Pending

---

#### Test 2.3: System Theme Detection
**Priority:** P1
- [ ] Set theme to "System"
- [ ] Change OS theme to dark
- [ ] Verify app switches to dark
- [ ] Change OS theme to light
- [ ] Verify app switches to light

**Expected:** App follows system theme
**Actual:** _____
**Status:** ⏳ Pending

---

#### Test 2.4: Dashboard in Dark Mode
**Priority:** P0
- [ ] Switch to dark theme
- [ ] Navigate to Dashboard
- [ ] Verify background color dark
- [ ] Verify text color light
- [ ] Verify stats cards readable
- [ ] Verify orange brand color visible
- [ ] Verify borders visible
- [ ] Verify shadows appropriate

**Expected:** Dashboard fully supports dark theme
**Actual:** _____
**Status:** ⏳ Pending

---

#### Test 2.5: Wallet in Dark Mode
**Priority:** P0
- [ ] Switch to dark theme
- [ ] Navigate to Wallet
- [ ] Verify account list readable
- [ ] Verify balance displays correctly
- [ ] Verify input fields visible
- [ ] Verify buttons have good contrast
- [ ] Verify transaction list readable

**Expected:** Wallet fully supports dark theme
**Actual:** _____
**Status:** ⏳ Pending

---

#### Test 2.6: DAG in Dark Mode
**Priority:** P1
- [ ] Switch to dark theme
- [ ] Navigate to DAG tab
- [ ] Verify block table readable
- [ ] Verify graph visualization clear
- [ ] Verify tooltips readable
- [ ] Verify no white backgrounds

**Expected:** DAG visualization works in dark mode
**Actual:** _____
**Status:** ⏳ Pending

---

#### Test 2.7: Models in Dark Mode
**Priority:** P1
- [ ] Switch to dark theme
- [ ] Navigate to Models tab
- [ ] Verify model cards readable
- [ ] Verify images/icons visible
- [ ] Verify buttons clear

**Expected:** Models page supports dark theme
**Actual:** _____
**Status:** ⏳ Pending

---

#### Test 2.8: Settings in Dark Mode
**Priority:** P1
- [ ] Switch to dark theme
- [ ] Navigate to Settings
- [ ] Verify form inputs visible
- [ ] Verify labels readable
- [ ] Verify theme selector highlighted

**Expected:** Settings fully supports dark theme
**Actual:** _____
**Status:** ⏳ Pending

---

#### Test 2.9: Modals in Dark Mode
**Priority:** P1
- [ ] Switch to dark theme
- [ ] Open keyboard shortcuts modal
- [ ] Verify modal background dark
- [ ] Verify modal text readable
- [ ] Verify close button visible

**Expected:** Modals support dark theme
**Actual:** _____
**Status:** ⏳ Pending

---

#### Test 2.10: Loading Skeletons in Dark Mode
**Priority:** P2
- [ ] Switch to dark theme
- [ ] Trigger loading state
- [ ] Verify skeleton colors match theme
- [ ] Verify shimmer effect visible

**Expected:** Skeletons adapt to theme
**Actual:** _____
**Status:** ⏳ Pending

---

#### Test 2.11: Error Messages in Dark Mode
**Priority:** P1
- [ ] Switch to dark theme
- [ ] Trigger validation error
- [ ] Verify error text readable
- [ ] Verify error background visible
- [ ] Verify sufficient contrast

**Expected:** Errors readable in dark theme
**Actual:** _____
**Status:** ⏳ Pending

---

#### Test 2.12: Orange Brand Color Visibility
**Priority:** P0
- [ ] Switch between themes
- [ ] Verify orange color visible in light mode
- [ ] Verify orange color visible in dark mode (lighter shade)
- [ ] Verify buttons use correct orange
- [ ] Verify highlights use correct orange

**Expected:** Brand color visible in both themes
**Actual:** _____
**Status:** ⏳ Pending

---

### Story 3: Performance Optimization (13 tests)

#### Test 3.1: Wallet Virtual Scrolling
**Priority:** P0
- [ ] Generate 1000+ transactions (or use mock data)
- [ ] Navigate to Wallet activity list
- [ ] Scroll through list
- [ ] Verify smooth scrolling (60fps)
- [ ] Verify no lag or stutter
- [ ] Open DevTools Performance
- [ ] Record scrolling
- [ ] Verify no long tasks

**Expected:** Smooth scrolling with 1000+ items
**Actual:** _____
**Status:** ⏳ Pending

---

#### Test 3.2: DAG Lazy Loading
**Priority:** P0
- [ ] Navigate to DAG with 500+ blocks
- [ ] Verify initial load shows first 100 blocks
- [ ] Scroll to bottom
- [ ] Verify "Load More" button appears
- [ ] Click "Load More"
- [ ] Verify next 100 blocks load
- [ ] Verify loading skeleton shown
- [ ] Verify no lag during load

**Expected:** Blocks load incrementally without lag
**Actual:** _____
**Status:** ⏳ Pending

---

#### Test 3.3: Dashboard Performance
**Priority:** P1
- [ ] Open DevTools Performance
- [ ] Navigate to Dashboard
- [ ] Record render time
- [ ] Verify render < 100ms
- [ ] Verify no unnecessary re-renders
- [ ] Interact with stats
- [ ] Verify smooth interactions

**Expected:** Dashboard renders in <100ms
**Actual:** _____ms
**Status:** ⏳ Pending

---

#### Test 3.4: Debounced Search
**Priority:** P1
- [ ] Open search input (if implemented)
- [ ] Type quickly: "test query"
- [ ] Verify search doesn't trigger on each keystroke
- [ ] Verify search triggers after 300ms
- [ ] Verify immediate visual feedback
- [ ] Check network tab
- [ ] Verify only one request sent

**Expected:** Search debounced to 300ms
**Actual:** _____
**Status:** ⏳ Pending

---

#### Test 3.5: Component Memoization
**Priority:** P2
- [ ] Open React DevTools Profiler
- [ ] Navigate to Dashboard
- [ ] Click unrelated button
- [ ] Record render
- [ ] Verify StatCards didn't re-render
- [ ] Verify memoization working

**Expected:** Memoized components skip unnecessary renders
**Actual:** _____
**Status:** ⏳ Pending

---

#### Test 3.6: Web Worker Performance (Optional)
**Priority:** P3
- [ ] Navigate to DAG
- [ ] Trigger blue set calculation
- [ ] Open DevTools Performance
- [ ] Verify calculation happens off main thread
- [ ] Verify UI remains responsive
- [ ] Verify results correct

**Expected:** Heavy calculations don't block UI
**Actual:** _____
**Status:** ⏳ Pending (Optional)

---

#### Test 3.7: Theme Switch Performance
**Priority:** P2
- [ ] Open DevTools Performance
- [ ] Record theme switch
- [ ] Verify theme switches in <50ms
- [ ] Verify smooth transition
- [ ] Verify no layout thrashing

**Expected:** Instant theme switch
**Actual:** _____ms
**Status:** ⏳ Pending

---

#### Test 3.8: Bundle Size
**Priority:** P2
- [ ] Run production build
- [ ] Check dist folder size
- [ ] Verify total bundle < 2.2MB
- [ ] Run bundle analyzer
- [ ] Verify no duplicate dependencies

**Expected:** Bundle size <2.2MB
**Actual:** _____MB
**Status:** ⏳ Pending

---

#### Test 3.9: First Contentful Paint
**Priority:** P2
- [ ] Run Lighthouse audit
- [ ] Check First Contentful Paint (FCP)
- [ ] Verify FCP < 1.5s
- [ ] Check Time to Interactive (TTI)
- [ ] Verify TTI < 3s

**Expected:** Fast initial render
**Actual:** FCP: _____s, TTI: _____s
**Status:** ⏳ Pending

---

#### Test 3.10: Memory Leaks
**Priority:** P2
- [ ] Open DevTools Memory
- [ ] Take heap snapshot
- [ ] Navigate through all tabs
- [ ] Take another heap snapshot
- [ ] Compare snapshots
- [ ] Verify no significant memory growth
- [ ] Verify event listeners cleaned up

**Expected:** No memory leaks
**Actual:** _____
**Status:** ⏳ Pending

---

#### Test 3.11: Large Dataset Handling
**Priority:** P1
- [ ] Load 10,000+ transactions
- [ ] Navigate to Wallet
- [ ] Verify app remains responsive
- [ ] Scroll through list
- [ ] Verify 60fps maintained
- [ ] Check memory usage

**Expected:** Handles large datasets efficiently
**Actual:** _____
**Status:** ⏳ Pending

---

#### Test 3.12: Network Performance
**Priority:** P2
- [ ] Open DevTools Network tab
- [ ] Navigate through app
- [ ] Verify lazy loading of chunks
- [ ] Verify no unnecessary requests
- [ ] Verify efficient caching

**Expected:** Optimized network requests
**Actual:** _____
**Status:** ⏳ Pending

---

#### Test 3.13: Low-End Device Testing
**Priority:** P3
- [ ] Test on low-end device or throttled CPU
- [ ] Navigate through app
- [ ] Verify still usable
- [ ] Verify no crashes
- [ ] Verify acceptable performance

**Expected:** Works on low-end devices
**Actual:** _____
**Status:** ⏳ Pending

---

### Story 4: Accessibility (10 tests)

#### Test 4.1: Keyboard Navigation
**Priority:** P0
- [ ] Tab from top of page
- [ ] Verify skip navigation link appears
- [ ] Press Enter on skip link
- [ ] Verify focus moves to main content
- [ ] Tab through all interactive elements
- [ ] Verify logical tab order
- [ ] Verify all elements reachable

**Expected:** Full keyboard navigation support
**Actual:** _____
**Status:** ⏳ Pending

---

#### Test 4.2: Focus Indicators
**Priority:** P0
- [ ] Tab through app
- [ ] Verify visible focus ring on every element
- [ ] Test in light theme
- [ ] Test in dark theme
- [ ] Verify high contrast focus indicators
- [ ] Verify no `:focus { outline: none }` without replacement

**Expected:** Clear focus indicators on all elements
**Actual:** _____
**Status:** ⏳ Pending

---

#### Test 4.3: Keyboard Shortcut: Ctrl+K
**Priority:** P1
- [ ] Press Ctrl+K
- [ ] Verify command palette opens (if implemented) or action occurs
- [ ] Verify browser search doesn't open
- [ ] Press Escape
- [ ] Verify closes

**Expected:** Ctrl+K opens command palette
**Actual:** _____
**Status:** ⏳ Pending

---

#### Test 4.4: Keyboard Shortcut: Ctrl+S
**Priority:** P1
- [ ] Press Ctrl+S
- [ ] Verify quick send dialog opens or navigates to send form
- [ ] Verify browser save dialog doesn't open
- [ ] Test from different tabs

**Expected:** Ctrl+S triggers quick send
**Actual:** _____
**Status:** ⏳ Pending

---

#### Test 4.5: Keyboard Shortcut: Ctrl+1-5
**Priority:** P1
- [ ] Press Ctrl+1
- [ ] Verify navigates to Dashboard
- [ ] Press Ctrl+2
- [ ] Verify navigates to Wallet
- [ ] Test all tab shortcuts

**Expected:** Tab shortcuts work
**Actual:** _____
**Status:** ⏳ Pending

---

#### Test 4.6: Keyboard Shortcut: ?
**Priority:** P2
- [ ] Press ?
- [ ] Verify keyboard shortcuts help modal opens
- [ ] Verify all shortcuts listed
- [ ] Press Escape
- [ ] Verify modal closes

**Expected:** ? opens shortcuts help
**Actual:** _____
**Status:** ⏳ Pending

---

#### Test 4.7: Screen Reader Testing (NVDA/VoiceOver)
**Priority:** P0
- [ ] Start screen reader (NVDA on Windows or VoiceOver on Mac)
- [ ] Navigate to app
- [ ] Verify page title announced
- [ ] Tab through navigation
- [ ] Verify all button labels read
- [ ] Verify form labels associated
- [ ] Fill out send form
- [ ] Verify validation errors announced
- [ ] Submit form
- [ ] Verify success message announced

**Expected:** Full screen reader support
**Actual:** _____
**Status:** ⏳ Pending

---

#### Test 4.8: ARIA Labels
**Priority:** P0
- [ ] Inspect all buttons in DevTools
- [ ] Verify aria-label present
- [ ] Inspect all icons
- [ ] Verify aria-label or alt text present
- [ ] Verify navigation has landmarks
- [ ] Verify live regions for announcements

**Expected:** All elements have proper ARIA labels
**Actual:** _____
**Status:** ⏳ Pending

---

#### Test 4.9: Color Contrast (WAVE Tool)
**Priority:** P0
- [ ] Install WAVE browser extension
- [ ] Run WAVE on Dashboard (light theme)
- [ ] Verify no contrast errors
- [ ] Switch to dark theme
- [ ] Run WAVE again
- [ ] Verify no contrast errors
- [ ] Test all major pages

**Expected:** WCAG AA contrast compliance
**Actual:** _____
**Status:** ⏳ Pending

---

#### Test 4.10: Lighthouse Accessibility Audit
**Priority:** P1
- [ ] Open DevTools Lighthouse
- [ ] Run accessibility audit
- [ ] Verify score >90
- [ ] Review any issues
- [ ] Fix critical issues
- [ ] Rerun audit

**Expected:** Lighthouse accessibility score >90
**Actual:** _____
**Status:** ⏳ Pending

---

## Cross-Browser Testing

### Desktop Browsers

#### Chrome
- [ ] All manual tests pass
- [ ] DevTools checks pass
- [ ] Performance acceptable

**Status:** ⏳ Pending

---

#### Firefox
- [ ] All manual tests pass
- [ ] Theme switching works
- [ ] Virtual scrolling works

**Status:** ⏳ Pending

---

#### Safari
- [ ] All manual tests pass
- [ ] CSS variables supported
- [ ] Web Workers work (if used)

**Status:** ⏳ Pending

---

#### Edge
- [ ] All manual tests pass
- [ ] No browser-specific issues

**Status:** ⏳ Pending

---

### Tauri Desktop App

#### macOS
- [ ] Window state persistence works
- [ ] Theme switching works
- [ ] All features functional

**Status:** ⏳ Pending

---

#### Windows
- [ ] Window state persistence works
- [ ] Theme switching works
- [ ] All features functional

**Status:** ⏳ Pending

---

#### Linux
- [ ] Window state persistence works
- [ ] Theme switching works
- [ ] All features functional

**Status:** ⏳ Pending

---

## Regression Testing

### Sprint 1 Features

#### Password Security
- [ ] Can still create wallet with custom password
- [ ] Password strength indicator works
- [ ] Password confirmation works

**Status:** ⏳ Pending

---

#### Input Validation
- [ ] Address validation still works
- [ ] Amount validation still works
- [ ] All validation functions work

**Status:** ⏳ Pending

---

#### Error Boundaries
- [ ] Error boundary still catches errors
- [ ] Fallback UI displays correctly

**Status:** ⏳ Pending

---

#### Loading Skeletons
- [ ] Skeletons still show during loading
- [ ] Skeletons match current theme

**Status:** ⏳ Pending

---

## Testing Commands

### Run All Tests
```bash
cd gui/citrate-core
npm test
```

### Run Unit Tests Only
```bash
npm test -- --testPathPattern='\.test\.(ts|tsx)$'
```

### Run Integration Tests Only
```bash
npm test -- --testPathPattern='integration'
```

### Run Performance Tests
```bash
npm test -- --testPathPattern='performance'
```

### Run Accessibility Tests
```bash
npm test -- --testPathPattern='accessibility'
```

### Run Tests with Coverage
```bash
npm test -- --coverage
```

### Run Tests in Watch Mode
```bash
npm test -- --watch
```

---

## Coverage Requirements

### Minimum Coverage Targets

| Metric | Target | Current |
|--------|--------|---------|
| Statements | >80% | ___% |
| Branches | >80% | ___% |
| Functions | >80% | ___% |
| Lines | >80% | ___% |

### Files Requiring High Coverage (>90%)
- storage.ts
- AppContext.tsx
- ThemeContext.tsx
- useDebounce.ts
- useKeyboardShortcuts.ts

---

## Bug Tracking

### Bugs Found During Testing

#### Bug #1
**Title:** _____
**Severity:** P0 / P1 / P2 / P3
**Description:** _____
**Steps to Reproduce:** _____
**Expected:** _____
**Actual:** _____
**Status:** Open / In Progress / Fixed
**Fix Commit:** _____

---

#### Bug #2
**Title:** _____
**Severity:** _____
**Description:** _____
**Status:** _____

---

## Final Checklist

### Before Sprint Close
- [ ] All unit tests passing (100%)
- [ ] All integration tests passing (100%)
- [ ] All performance tests passing (100%)
- [ ] All accessibility tests passing (100%)
- [ ] All manual tests completed (100%)
- [ ] Cross-browser testing completed
- [ ] Regression testing completed
- [ ] All critical bugs fixed
- [ ] Coverage targets met (>80%)
- [ ] No console errors in production build

---

**Sprint 2 Testing Status:** ⏳ Pending → 🚧 In Progress → ✅ Completed

**Last Updated:** [Date]
**Test Execution Rate:** 0/110 (0%)
