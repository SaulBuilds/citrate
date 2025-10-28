# Sprint 2: Technical Tasks Breakdown

**Sprint Goal:** Enhance UX with state persistence, dark mode, performance optimization, and accessibility

**Total Story Points:** 13 points
**Duration:** 5 working days (32 hours)

---

## Task Completion Tracking

### ✅ Completed | 🚧 In Progress | ⏳ Pending | ❌ Blocked

---

## Day 1: State Persistence Foundation (6 hours)

### Story 1: State Persistence (5 points)

#### Task 1.1: Create Storage Utility Module (2 hours) ⏳
**File:** `gui/citrate-core/src/utils/storage.ts` (NEW FILE)

**Implementation Steps:**
- [ ] Create StorageSchema interface with type definitions
- [ ] Implement getStorageItem<T>() with type safety
- [ ] Implement setStorageItem<T>() with error handling
- [ ] Implement removeStorageItem() and clearStorage()
- [ ] Add addRecentAddress() with max 10 limit
- [ ] Add exportSettings() → JSON string
- [ ] Add importSettings() with validation
- [ ] Add migrateStorage() for version handling
- [ ] Add quota exceeded error handling
- [ ] Add clearOldData() to free space

**Dependencies:** None

**Estimated Time:** 2 hours

**Acceptance:**
- [ ] All functions typed correctly
- [ ] Error handling for localStorage failures
- [ ] Quota exceeded handled gracefully
- [ ] Recent addresses capped at 10
- [ ] Export/import validation working
- [ ] Storage versioning implemented

**Testing:**
```bash
cd gui/citrate-core
npm test src/utils/storage.test.ts
```

---

#### Task 1.2: Create AppContext for Global State (2 hours) ⏳
**File:** `gui/citrate-core/src/contexts/AppContext.tsx` (NEW FILE)

**Implementation Steps:**
- [ ] Define AppState interface
- [ ] Create AppContext with createContext()
- [ ] Create useApp() hook with error checking
- [ ] Implement AppProvider component
- [ ] Load persisted tab on mount
- [ ] Load persisted recent addresses
- [ ] Load persisted settings
- [ ] Implement setCurrentTab() with persistence
- [ ] Implement addRecentAddress() with deduplication
- [ ] Implement clearRecentAddresses()
- [ ] Implement updateSettings() with auto-save
- [ ] Initialize storage on provider mount

**Dependencies:** Task 1.1 (storage utility)

**Estimated Time:** 2 hours

**Acceptance:**
- [ ] Context provides all required state
- [ ] State persists automatically
- [ ] useApp() hook works in components
- [ ] No prop drilling required
- [ ] Storage initialized on app start
- [ ] Default values provided when no storage

**Testing:**
```typescript
describe('AppContext', () => {
  it('should provide current tab from storage', () => {
    // Test implementation
  });

  it('should persist tab changes', () => {
    // Test implementation
  });
});
```

---

#### Task 1.3: Integrate AppContext into App.tsx (1.5 hours) ⏳
**File:** `gui/citrate-core/src/App.tsx`

**Implementation Steps:**
- [ ] Import AppProvider and useApp
- [ ] Wrap app content with AppProvider
- [ ] Create AppContent component using useApp()
- [ ] Replace currentView state with context
- [ ] Update tab navigation to use setCurrentTab()
- [ ] Remove local storage code (now in context)
- [ ] Test tab persistence on refresh
- [ ] Verify no regression in navigation

**Code Pattern:**
```typescript
function App() {
  return (
    <ErrorBoundary>
      <AppProvider>
        <ThemeProvider>
          <AppContent />
        </ThemeProvider>
      </AppProvider>
    </ErrorBoundary>
  );
}

function AppContent() {
  const { currentTab, setCurrentTab } = useApp();
  // ... use context instead of local state
}
```

**Dependencies:** Task 1.2

**Estimated Time:** 1.5 hours

**Acceptance:**
- [ ] AppProvider wraps entire app
- [ ] Tab state from context
- [ ] Tab persists across refresh
- [ ] Navigation still works correctly
- [ ] No console errors
- [ ] Context hierarchy correct

---

#### Task 1.4: Add Recent Addresses to Wallet Component (0.5 hours) ⏳
**File:** `gui/citrate-core/src/components/Wallet.tsx`

**Implementation Steps:**
- [ ] Import useApp() hook
- [ ] Destructure recentAddresses and addRecentAddress
- [ ] Add datalist element to recipient input
- [ ] Populate datalist with recent addresses
- [ ] Call addRecentAddress() on successful send
- [ ] Add "Clear History" button in UI (optional)
- [ ] Test autocomplete dropdown shows addresses
- [ ] Test selecting from dropdown populates field

**Code Pattern:**
```typescript
const { recentAddresses, addRecentAddress } = useApp();

<input
  list="recent-addresses"
  value={recipient}
  onChange={(e) => setRecipient(e.target.value)}
/>
<datalist id="recent-addresses">
  {recentAddresses.map(addr => (
    <option key={addr} value={addr} />
  ))}
</datalist>
```

**Dependencies:** Task 1.3

**Estimated Time:** 0.5 hours

**Acceptance:**
- [ ] Datalist shows recent addresses
- [ ] Selecting address populates input
- [ ] New addresses saved on successful send
- [ ] Max 10 addresses maintained
- [ ] No duplicates in list
- [ ] Clear history works (if implemented)

---

## Day 2: Dark Mode Implementation (6 hours)

### Story 2: Dark Mode Theme (3 points)

#### Task 2.1: Create Theme Definitions (1.5 hours) ⏳
**File:** `gui/citrate-core/src/styles/themes.ts` (NEW FILE)

**Implementation Steps:**
- [ ] Define Theme interface with all color properties
- [ ] Create lightTheme object with light colors
- [ ] Create darkTheme object with dark colors
- [ ] Export themes object { light, dark }
- [ ] Define ThemeMode type ('light' | 'dark' | 'system')
- [ ] Implement applyTheme() to set CSS variables
- [ ] Implement getSystemTheme() to detect preference
- [ ] Add data-theme attribute to root element
- [ ] Adjust orange brand color for dark mode (#ffb84d)
- [ ] Ensure WCAG AA contrast compliance

**Color Definitions:**

Light Theme:
- Background primary: #ffffff
- Background secondary: #f9fafb
- Text primary: #1a1a1a
- Brand primary: #ffa500 (orange)
- Success: #10b981
- Error: #ef4444

Dark Theme:
- Background primary: #1a1a1a
- Background secondary: #242424
- Text primary: #ffffff
- Brand primary: #ffb84d (lighter orange)
- Success: #34d399
- Error: #f87171

**Dependencies:** None

**Estimated Time:** 1.5 hours

**Acceptance:**
- [ ] Theme interface complete
- [ ] Light and dark themes defined
- [ ] All colors have good contrast
- [ ] Orange adjusted for dark mode
- [ ] applyTheme() sets CSS variables
- [ ] getSystemTheme() detects preference
- [ ] TypeScript types exported

---

#### Task 2.2: Create ThemeContext (1.5 hours) ⏳
**File:** `gui/citrate-core/src/contexts/ThemeContext.tsx` (NEW FILE)

**Implementation Steps:**
- [ ] Define ThemeContextType interface
- [ ] Create ThemeContext with createContext()
- [ ] Create useTheme() hook
- [ ] Implement ThemeProvider component
- [ ] Load theme from storage on mount
- [ ] Apply theme on provider mount
- [ ] Update theme when themeMode changes
- [ ] Listen for system theme changes
- [ ] Implement setThemeMode() with persistence
- [ ] Implement toggleTheme() helper
- [ ] Handle 'system' mode by resolving to light/dark

**Dependencies:** Task 2.1 (theme definitions)

**Estimated Time:** 1.5 hours

**Acceptance:**
- [ ] ThemeProvider wraps app
- [ ] useTheme() hook works in components
- [ ] Theme persists in localStorage
- [ ] System theme detection works
- [ ] Theme changes apply immediately
- [ ] No flicker on load
- [ ] System theme changes trigger update

**Testing:**
```typescript
describe('ThemeContext', () => {
  it('should apply light theme by default', () => {
    render(
      <ThemeProvider>
        <TestComponent />
      </ThemeProvider>
    );
    expect(document.documentElement.getAttribute('data-theme')).toBe('light');
  });

  it('should toggle theme', () => {
    const { getByRole } = render(
      <ThemeProvider>
        <ThemeToggle />
      </ThemeProvider>
    );
    fireEvent.click(getByRole('button'));
    expect(document.documentElement.getAttribute('data-theme')).toBe('dark');
  });
});
```

---

#### Task 2.3: Update CSS with Variables (2 hours) ⏳
**Files:**
- `gui/citrate-core/src/App.css`
- All component stylesheets

**Implementation Steps:**
- [ ] Replace hardcoded colors with CSS variables
- [ ] Update body styles to use var(--bg-primary)
- [ ] Update text colors to use var(--text-primary)
- [ ] Update button styles to use var(--brand-primary)
- [ ] Update card styles to use var(--bg-secondary)
- [ ] Update border colors to use var(--border-primary)
- [ ] Update shadows to use var(--shadow-*)
- [ ] Add smooth transitions for theme changes
- [ ] Update component-specific stylesheets
- [ ] Test all components in both themes

**CSS Pattern:**
```css
/* Before */
.button {
  background-color: #ffa500;
  color: #1a1a1a;
}

/* After */
.button {
  background-color: var(--brand-primary);
  color: var(--text-primary);
  transition: background-color 200ms ease, color 200ms ease;
}
```

**Dependencies:** Task 2.1 (theme definitions)

**Estimated Time:** 2 hours

**Acceptance:**
- [ ] All hardcoded colors replaced
- [ ] Variables used consistently
- [ ] Smooth transitions between themes
- [ ] No flicker during theme change
- [ ] All components styled for both themes
- [ ] Contrast ratios meet WCAG AA
- [ ] Orange brand color visible in both themes

**Files to Update:**
- `gui/citrate-core/src/App.css`
- `gui/citrate-core/src/components/Dashboard.css` (if exists)
- `gui/citrate-core/src/components/Wallet.css`
- `gui/citrate-core/src/components/Settings.css`
- All other component stylesheets

---

#### Task 2.4: Add Theme Toggle to Settings (1 hour) ⏳
**File:** `gui/citrate-core/src/components/Settings.tsx`

**Implementation Steps:**
- [ ] Import useTheme() hook
- [ ] Import Sun, Moon, Monitor icons from lucide-react
- [ ] Add "Appearance" section to Settings
- [ ] Create theme selector with 3 buttons
- [ ] Add Light button with Sun icon
- [ ] Add Dark button with Moon icon
- [ ] Add System button with Monitor icon
- [ ] Highlight active theme button
- [ ] Call setThemeMode() on button click
- [ ] Add CSS for theme selector buttons
- [ ] Test theme switching works

**UI Design:**
```
Appearance
Theme: [Light] [Dark] [System]
```

**Dependencies:** Task 2.2 (ThemeContext)

**Estimated Time:** 1 hour

**Acceptance:**
- [ ] Theme selector UI implemented
- [ ] Three theme options available
- [ ] Active theme highlighted
- [ ] Icons displayed correctly
- [ ] Theme changes immediately on click
- [ ] Visual feedback for selection
- [ ] Accessible (keyboard navigation)

---

## Day 3: Dark Mode Polish + Performance Start (6 hours)

#### Task 3.1: Test and Fix Dark Mode Across Components (2 hours) ⏳
**Files:** All component files

**Testing Checklist:**
- [ ] Dashboard component in dark mode
- [ ] Wallet component in dark mode
- [ ] DAG visualization in dark mode
- [ ] Models component in dark mode
- [ ] Settings component in dark mode
- [ ] FirstTimeSetup component in dark mode
- [ ] Modals and dialogs in dark mode
- [ ] Loading skeletons match theme
- [ ] Error messages readable
- [ ] Success messages readable
- [ ] Input fields visible
- [ ] Buttons have sufficient contrast
- [ ] Orange brand color visible
- [ ] Borders visible in both themes
- [ ] Shadows appropriate

**Bugs to Fix:**
- [ ] Any contrast issues
- [ ] Text not visible on backgrounds
- [ ] Borders too faint
- [ ] Shadows too strong/weak
- [ ] Icons not visible
- [ ] Loading spinners not visible

**Dependencies:** Task 2.4

**Estimated Time:** 2 hours

**Acceptance:**
- [ ] All components support both themes
- [ ] No readability issues
- [ ] Contrast ratios meet WCAG AA
- [ ] Visual polish consistent
- [ ] No flicker or artifacts
- [ ] Theme transitions smooth

---

#### Task 3.2: Add System Theme Detection (1 hour) ⏳
**File:** `gui/citrate-core/src/contexts/ThemeContext.tsx`

**Implementation Steps:**
- [ ] Already implemented in Task 2.2
- [ ] Test system theme detection works
- [ ] Test theme updates when system changes
- [ ] Add useEffect to listen for changes
- [ ] Test "System" mode resolves correctly
- [ ] Verify mediaQuery listener cleanup

**Testing:**
```typescript
// Change system theme preference
window.matchMedia('(prefers-color-scheme: dark)').matches
// Verify app theme updates
```

**Dependencies:** Task 2.2

**Estimated Time:** 1 hour

**Acceptance:**
- [ ] System theme detected on first load
- [ ] System theme changes trigger update
- [ ] "System" mode works correctly
- [ ] No memory leaks from listeners
- [ ] Works on all browsers

---

### Story 3: Performance Optimization (3 points)

#### Task 3.3: Create VirtualList Component (2 hours) ⏳
**File:** `gui/citrate-core/src/components/VirtualList.tsx` (NEW FILE)

**Implementation Steps:**

**Option 1: Custom Implementation**
- [ ] Create VirtualListProps interface
- [ ] Calculate visible items based on scroll position
- [ ] Render only visible items + overscan
- [ ] Use transform for positioning
- [ ] Add scroll event handler with throttle
- [ ] Calculate total height for scrollbar
- [ ] Test with 1000+ items

**Option 2: Use react-window library**
- [ ] Install react-window: `npm install react-window`
- [ ] Install types: `npm install --save-dev @types/react-window`
- [ ] Create wrapper component
- [ ] Export FixedSizeList or VariableSizeList
- [ ] Add TypeScript types
- [ ] Create example usage

**Recommended:** Use react-window for reliability

```bash
npm install react-window @types/react-window
```

**Dependencies:** None

**Estimated Time:** 2 hours

**Acceptance:**
- [ ] VirtualList component created
- [ ] Renders only visible items
- [ ] Smooth scrolling maintained
- [ ] Works with 1000+ items without lag
- [ ] TypeScript types correct
- [ ] Properly typed props
- [ ] Example usage documented

---

#### Task 3.4: Add Debounced Search Input (1 hour) ⏳
**File:** `gui/citrate-core/src/hooks/useDebounce.ts` (NEW FILE)

**Implementation Steps:**
- [ ] Create useDebounce<T>() hook
- [ ] Add delay parameter (default 300ms)
- [ ] Use setTimeout to delay value update
- [ ] Clear timeout on value change
- [ ] Return debounced value
- [ ] Add TypeScript generic for type safety
- [ ] Test with search inputs

**Usage Pattern:**
```typescript
const [searchQuery, setSearchQuery] = useState('');
const debouncedQuery = useDebounce(searchQuery, 300);

useEffect(() => {
  if (debouncedQuery) {
    performSearch(debouncedQuery);
  }
}, [debouncedQuery]);
```

**Dependencies:** None

**Estimated Time:** 1 hour

**Acceptance:**
- [ ] useDebounce hook created
- [ ] Delays value updates correctly
- [ ] Clears timeout on unmount
- [ ] TypeScript generic works
- [ ] Default delay 300ms
- [ ] Works with strings, numbers, objects
- [ ] Unit tests passing

**Testing:**
```typescript
describe('useDebounce', () => {
  it('should debounce value changes', async () => {
    const { result, rerender } = renderHook(
      ({ value }) => useDebounce(value, 300),
      { initialProps: { value: 'initial' } }
    );

    expect(result.current).toBe('initial');

    rerender({ value: 'updated' });
    expect(result.current).toBe('initial'); // Not updated yet

    await waitFor(() => expect(result.current).toBe('updated'), { timeout: 400 });
  });
});
```

---

## Day 4: Performance Optimization (6 hours)

#### Task 4.1: Add Virtual Scrolling to Wallet Activity (1.5 hours) ⏳
**File:** `gui/citrate-core/src/components/Wallet.tsx`

**Implementation Steps:**
- [ ] Import VirtualList or FixedSizeList
- [ ] Replace flat transaction list rendering
- [ ] Set container height (600px)
- [ ] Set item height (60px)
- [ ] Create renderItem function
- [ ] Test with 100+ transactions
- [ ] Test with 1000+ transactions
- [ ] Verify smooth scrolling
- [ ] Check FPS during scroll

**Before:**
```typescript
{transactions.map(tx => (
  <TransactionItem key={tx.hash} tx={tx} />
))}
```

**After:**
```typescript
<FixedSizeList
  height={600}
  itemCount={transactions.length}
  itemSize={60}
  width="100%"
>
  {({ index, style }) => (
    <div style={style}>
      <TransactionItem tx={transactions[index]} />
    </div>
  )}
</FixedSizeList>
```

**Dependencies:** Task 3.3 (VirtualList component)

**Estimated Time:** 1.5 hours

**Acceptance:**
- [ ] Virtual scrolling implemented
- [ ] Only visible items rendered
- [ ] Smooth scrolling maintained
- [ ] Works with 1000+ items
- [ ] No layout shifts
- [ ] Performance improved

**Performance Target:**
- Before: 100 items → 80ms render
- After: 1000 items → <100ms render

---

#### Task 4.2: Optimize DAG Visualization with Lazy Loading (2 hours) ⏳
**File:** `gui/citrate-core/src/components/DAGVisualization.tsx`

**Implementation Steps:**
- [ ] Implement pagination or infinite scroll
- [ ] Load blocks in chunks (50-100 at a time)
- [ ] Show "Load More" button or auto-load on scroll
- [ ] Use VirtualList for block table
- [ ] Add loading skeleton while fetching
- [ ] Cache loaded blocks
- [ ] Prevent duplicate fetches
- [ ] Test with 500+ blocks
- [ ] Test with 1000+ blocks

**Lazy Loading Pattern:**
```typescript
const [blocks, setBlocks] = useState<Block[]>([]);
const [page, setPage] = useState(0);
const [loading, setLoading] = useState(false);
const [hasMore, setHasMore] = useState(true);

const loadMoreBlocks = async () => {
  if (loading || !hasMore) return;

  setLoading(true);
  const newBlocks = await fetchBlocks(page, 100);
  setBlocks(prev => [...prev, ...newBlocks]);
  setPage(prev => prev + 1);
  setHasMore(newBlocks.length === 100);
  setLoading(false);
};
```

**Dependencies:** Task 3.3 (VirtualList)

**Estimated Time:** 2 hours

**Acceptance:**
- [ ] Blocks loaded incrementally
- [ ] Virtual scrolling on block table
- [ ] "Load More" or infinite scroll works
- [ ] Loading indicator shown
- [ ] No duplicate fetches
- [ ] Handles 1000+ blocks smoothly
- [ ] FPS maintained during scroll

**Performance Target:**
- Before: 500 blocks → 300ms render
- After: 1000 blocks → <200ms render

---

#### Task 4.3: Create Web Worker for DAG Calculations (1.5 hours) ⏳
**File:** `gui/citrate-core/src/workers/dagWorker.ts` (NEW FILE)

**Implementation Steps:**
- [ ] Create dagWorker.ts file
- [ ] Implement message handler (onmessage)
- [ ] Add CALCULATE_BLUE_SET case
- [ ] Add TOPOLOGICAL_SORT case
- [ ] Implement calculation logic
- [ ] Post results back to main thread
- [ ] Create hook for using worker
- [ ] Test worker communication
- [ ] Add fallback for unsupported browsers

**Worker Implementation:**
```typescript
// dagWorker.ts
self.onmessage = (e: MessageEvent) => {
  const { type, payload } = e.data;

  switch (type) {
    case 'CALCULATE_BLUE_SET':
      const blueSet = calculateBlueSet(payload.blocks, payload.k);
      self.postMessage({ type: 'BLUE_SET_RESULT', payload: blueSet });
      break;
  }
};
```

**Usage in Component:**
```typescript
const workerRef = useRef<Worker | null>(null);

useEffect(() => {
  workerRef.current = new Worker(
    new URL('../workers/dagWorker.ts', import.meta.url),
    { type: 'module' }
  );

  workerRef.current.onmessage = (e) => {
    const { type, payload } = e.data;
    if (type === 'BLUE_SET_RESULT') {
      setBlueSet(payload);
    }
  };

  return () => workerRef.current?.terminate();
}, []);
```

**Dependencies:** None (can be done in parallel)

**Estimated Time:** 1.5 hours

**Acceptance:**
- [ ] Web Worker created
- [ ] Message passing works
- [ ] Calculations offloaded to worker
- [ ] UI thread not blocked
- [ ] Results returned correctly
- [ ] Worker terminated on unmount
- [ ] Fallback for unsupported environments

**Optional:** Use Comlink library to simplify worker communication

---

#### Task 4.4: Add React.memo to Heavy Components (1 hour) ⏳
**Files:**
- `gui/citrate-core/src/components/Dashboard.tsx`
- `gui/citrate-core/src/components/Wallet.tsx`
- `gui/citrate-core/src/components/DAGVisualization.tsx`

**Implementation Steps:**
- [ ] Identify expensive components
- [ ] Wrap components with React.memo()
- [ ] Add custom comparison function if needed
- [ ] Use useMemo for expensive calculations
- [ ] Use useCallback for event handlers
- [ ] Profile with React DevTools Profiler
- [ ] Verify re-renders reduced

**Pattern:**
```typescript
// Before
export const StatCard = ({ title, value, icon }: StatCardProps) => {
  return <div>...</div>;
};

// After
export const StatCard = memo(({ title, value, icon }: StatCardProps) => {
  return <div>...</div>;
});

// With custom comparison
export const StatCard = memo(
  ({ title, value, icon }: StatCardProps) => {
    return <div>...</div>;
  },
  (prevProps, nextProps) => {
    return prevProps.value === nextProps.value;
  }
);
```

**useMemo Example:**
```typescript
const stats = useMemo(() => {
  return {
    totalBlocks: blocks.length,
    avgBlockTime: calculateAvgBlockTime(blocks),
    networkHashRate: calculateHashRate(blocks)
  };
}, [blocks]);
```

**useCallback Example:**
```typescript
const handleRefresh = useCallback(async () => {
  const data = await fetchData();
  setData(data);
}, []);
```

**Dependencies:** None

**Estimated Time:** 1 hour

**Acceptance:**
- [ ] Heavy components memoized
- [ ] useMemo for expensive calculations
- [ ] useCallback for handlers
- [ ] Re-renders reduced (verify with Profiler)
- [ ] No functional regressions
- [ ] Performance improved

---

## Day 5: Accessibility & Testing (8 hours)

### Story 4: Accessibility & Keyboard Shortcuts (2 points)

#### Task 5.1: Create Keyboard Shortcuts Hook (2 hours) ⏳
**File:** `gui/citrate-core/src/hooks/useKeyboardShortcuts.ts` (NEW FILE)

**Implementation Steps:**
- [ ] Define KeyboardShortcut interface
- [ ] Create useKeyboardShortcuts() hook
- [ ] Add keydown event listener
- [ ] Match key combinations (Ctrl, Shift, Alt)
- [ ] Call handler when match found
- [ ] Prevent default browser behavior
- [ ] Clean up listener on unmount
- [ ] Export globalShortcuts array
- [ ] Add keyboard shortcut descriptions

**Shortcuts to Implement:**
- Ctrl+K: Open command palette
- Ctrl+S: Quick send transaction
- Ctrl+,: Open settings
- Ctrl+1-5: Switch tabs
- ?: Show keyboard shortcuts help
- Escape: Close modals

**Dependencies:** None

**Estimated Time:** 2 hours

**Acceptance:**
- [ ] Hook created and working
- [ ] All shortcuts implemented
- [ ] Prevents browser defaults
- [ ] Multiple shortcuts supported
- [ ] Descriptions for each shortcut
- [ ] Works across all browsers
- [ ] No conflicts with browser shortcuts

**Testing:**
```typescript
describe('useKeyboardShortcuts', () => {
  it('should call handler when shortcut pressed', () => {
    const handler = vi.fn();
    renderHook(() =>
      useKeyboardShortcuts([
        { key: 'k', ctrl: true, handler, description: 'Test' }
      ])
    );

    fireEvent.keyDown(window, { key: 'k', ctrlKey: true });
    expect(handler).toHaveBeenCalled();
  });
});
```

---

#### Task 5.2: Add ARIA Labels to All Components (2 hours) ⏳
**Files:** All component files

**Implementation Steps:**
- [ ] Add aria-label to all buttons
- [ ] Add aria-label to all icons
- [ ] Add role attributes where needed
- [ ] Add aria-live regions for announcements
- [ ] Add aria-current for navigation
- [ ] Add aria-describedby for help text
- [ ] Add landmark roles (nav, main, aside)
- [ ] Add skip navigation link
- [ ] Test with screen reader
- [ ] Verify all elements announced correctly

**ARIA Patterns:**

Buttons:
```typescript
<button
  onClick={handleClick}
  aria-label="Send transaction"
  aria-describedby="send-help"
>
  Send
</button>
<span id="send-help" className="sr-only">
  Ctrl+S to quick send
</span>
```

Navigation:
```typescript
<nav aria-label="Main navigation">
  <ul role="list">
    <li role="listitem">
      <button
        aria-label="Dashboard"
        aria-current={tab === 'dashboard' ? 'page' : undefined}
      >
        Dashboard
      </button>
    </li>
  </ul>
</nav>
```

Live Regions:
```typescript
<div
  role="status"
  aria-live="polite"
  aria-atomic="true"
  className="sr-only"
>
  {statusMessage}
</div>
```

Skip Link:
```typescript
<a href="#main-content" className="skip-link">
  Skip to main content
</a>
<main id="main-content" tabIndex={-1}>
  {/* Content */}
</main>
```

**Dependencies:** None

**Estimated Time:** 2 hours

**Acceptance:**
- [ ] All buttons have aria-label
- [ ] All icons have aria-label or alt
- [ ] Navigation has landmarks
- [ ] Live regions for announcements
- [ ] Skip navigation link added
- [ ] Screen reader tested
- [ ] All elements properly announced

---

#### Task 5.3: Create Keyboard Shortcuts Help Modal (1 hour) ⏳
**File:** `gui/citrate-core/src/components/KeyboardShortcutsHelp.tsx` (NEW FILE)

**Implementation Steps:**
- [ ] Create modal component
- [ ] Import globalShortcuts
- [ ] Display shortcuts in table/grid
- [ ] Show key combination + description
- [ ] Add close button (X)
- [ ] Close on Escape key
- [ ] Focus trap inside modal
- [ ] Return focus on close
- [ ] Style with current theme
- [ ] Test keyboard navigation

**UI Design:**
```
Keyboard Shortcuts
─────────────────────────
Ctrl+K    Open command palette
Ctrl+S    Quick send transaction
Ctrl+,    Open settings
Ctrl+1    Dashboard
...
─────────────────────────
Press Esc to close
```

**Dependencies:** Task 5.1 (keyboard shortcuts hook)

**Estimated Time:** 1 hour

**Acceptance:**
- [ ] Modal displays all shortcuts
- [ ] Formatted clearly
- [ ] Close button works
- [ ] Escape key closes modal
- [ ] Focus trapped in modal
- [ ] Styled for both themes
- [ ] Accessible (ARIA labels)

---

#### Task 5.4: Accessibility Audit and Fixes (2 hours) ⏳
**Tools:** WAVE, axe DevTools, Lighthouse

**Audit Checklist:**

##### Color Contrast
- [ ] Run WAVE browser extension
- [ ] Check all text contrast (4.5:1 minimum)
- [ ] Check large text contrast (3:1 minimum)
- [ ] Check UI component contrast (3:1 minimum)
- [ ] Fix any failures

##### Keyboard Navigation
- [ ] Tab through entire application
- [ ] Verify logical tab order
- [ ] Test all keyboard shortcuts
- [ ] Verify focus visible on all elements
- [ ] Test Escape closes modals
- [ ] Test Enter activates buttons
- [ ] Fix any issues

##### Screen Reader
- [ ] Test with NVDA (Windows) or VoiceOver (Mac)
- [ ] Verify page titles announced
- [ ] Verify button labels read correctly
- [ ] Verify form labels associated
- [ ] Verify live region announcements
- [ ] Fix any issues

##### Semantic HTML
- [ ] Verify heading hierarchy (h1 → h2 → h3)
- [ ] Check semantic elements (nav, main, aside)
- [ ] Verify lists use ul/ol
- [ ] Verify forms use fieldset/legend
- [ ] Fix any issues

##### Automated Testing
- [ ] Run axe-core automated tests
- [ ] Run Lighthouse accessibility audit
- [ ] Fix all critical issues
- [ ] Fix all serious issues
- [ ] Document minor issues

**Dependencies:** Tasks 5.1-5.3

**Estimated Time:** 2 hours

**Acceptance:**
- [ ] WCAG 2.1 AA compliant
- [ ] Lighthouse score >90
- [ ] axe-core no critical violations
- [ ] All fixes implemented
- [ ] Screen reader tested
- [ ] Keyboard navigation smooth

---

#### Task 5.5: Performance Testing and Final Optimization (1 hour) ⏳

**Performance Testing:**

##### Measure Baseline
- [ ] Dashboard render time
- [ ] Wallet activity (1000 items) render time
- [ ] DAG visualization (1000 blocks) render time
- [ ] Bundle size
- [ ] First contentful paint (FCP)
- [ ] Time to interactive (TTI)

##### Run Performance Tests
- [ ] Open Chrome DevTools Performance tab
- [ ] Record while scrolling large lists
- [ ] Record while switching tabs
- [ ] Record during theme toggle
- [ ] Analyze flame graph
- [ ] Identify bottlenecks

##### Optimize
- [ ] Fix any long tasks (>50ms)
- [ ] Optimize expensive functions
- [ ] Add more memoization if needed
- [ ] Code split large components if needed
- [ ] Optimize bundle size

##### Measure Final
- [ ] Re-run all measurements
- [ ] Compare to baseline
- [ ] Verify targets met:
  - Dashboard render: <100ms ✓
  - Wallet (1000 items): <100ms ✓
  - DAG (1000 blocks): <200ms ✓
  - Bundle size: <2.2MB ✓
  - 60fps scrolling ✓

**Performance Targets:**
| Metric | Before | After | Target | Status |
|--------|--------|-------|--------|--------|
| Dashboard render | 150ms | __ms | <100ms | ⏳ |
| Wallet (1000 items) | N/A | __ms | <100ms | ⏳ |
| DAG (1000 blocks) | 300ms | __ms | <200ms | ⏳ |
| Bundle size | 2.5MB | __MB | <2.2MB | ⏳ |
| FPS (scrolling) | ~45fps | __fps | 60fps | ⏳ |

**Dependencies:** All performance tasks (4.1-4.4)

**Estimated Time:** 1 hour

**Acceptance:**
- [ ] All performance benchmarks met
- [ ] 60fps maintained during interactions
- [ ] No long tasks blocking UI
- [ ] Bundle size reduced
- [ ] Lighthouse score >90
- [ ] Performance report documented

---

## Final Sprint 2 Checklist

### Code Quality
- [ ] All TypeScript types correct
- [ ] No console.log statements (except intentional)
- [ ] No commented-out code
- [ ] All imports organized
- [ ] No unused variables or imports
- [ ] Consistent code formatting
- [ ] All ESLint warnings resolved

### State Persistence
- [ ] Theme persists across sessions
- [ ] Tab persists across sessions
- [ ] Settings persist across sessions
- [ ] Recent addresses saved
- [ ] Import/export settings works
- [ ] Storage versioning implemented

### Dark Mode
- [ ] Theme toggle works
- [ ] All components support both themes
- [ ] No hardcoded colors
- [ ] Smooth transitions
- [ ] System theme detection works
- [ ] WCAG AA contrast compliance

### Performance
- [ ] Virtual scrolling implemented
- [ ] Lazy loading working
- [ ] React.memo on heavy components
- [ ] Debounced inputs
- [ ] Web Worker created (optional)
- [ ] 60fps maintained
- [ ] Performance targets met

### Accessibility
- [ ] Keyboard shortcuts working
- [ ] ARIA labels on all elements
- [ ] Screen reader tested
- [ ] WCAG 2.1 AA compliant
- [ ] Keyboard navigation smooth
- [ ] Focus indicators visible

### Testing
- [ ] Unit tests passing
- [ ] Integration tests passing
- [ ] Manual testing completed
- [ ] Performance tests passing
- [ ] Accessibility tests passing
- [ ] No console errors

### Documentation
- [ ] All sprint docs updated
- [ ] Implementation log complete
- [ ] Performance metrics documented
- [ ] Accessibility report complete
- [ ] Code comments added

---

## Time Tracking Summary

| Day | Tasks | Estimated Hours | Actual Hours | Notes |
|-----|-------|----------------|--------------|-------|
| Day 1 | State Persistence | 6.0 | | |
| Day 2 | Dark Mode | 6.0 | | |
| Day 3 | Dark Mode + Performance | 6.0 | | |
| Day 4 | Performance | 6.0 | | |
| Day 5 | Accessibility + Testing | 8.0 | | |
| **Total** | | **32.0** | | |

---

## Risk Management

### Identified Risks
- [ ] Risk 1: Dark mode CSS conflicts → Test incrementally
- [ ] Risk 2: Virtual scrolling breaks layouts → Maintain DOM structure
- [ ] Risk 3: Web Worker compatibility → Add fallback
- [ ] Risk 4: Keyboard shortcuts conflict → Use unique combinations

### Mitigation Actions
- [ ] Test each feature in isolation
- [ ] Maintain backwards compatibility
- [ ] Add feature detection
- [ ] Document all changes

---

## Next Steps (Transition to Sprint 3)

**Sprint 3 Preview:**
- Advanced wallet features
- Transaction batching
- Multi-signature support
- Hardware wallet integration

**Preparation:**
- [ ] Review Sprint 3 objectives
- [ ] Set up hardware wallet testing
- [ ] Review Web3 libraries
- [ ] Plan transaction flow refactoring

---

**Sprint 2 Status:** ⏳ Pending → 🚧 In Progress → ✅ Completed

**Last Updated:** [To be filled during sprint]

**Total Tasks:** 19 tasks across 5 days

**Completion Rate:** ___% (to be calculated at end)
