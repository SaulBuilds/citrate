# Sprint 2: File Changes Tracking

**Sprint Goal:** Track all file modifications, creations, and deletions for Sprint 2

**Last Updated:** [Sprint Start Date]

---

## Summary

| Category | Count | Status |
|----------|-------|--------|
| New Files | 15 | ⏳ |
| Modified Files | 12 | ⏳ |
| Deleted Files | 0 | ⏳ |
| Test Files | 8 | ⏳ |
| **Total Files** | **35** | ⏳ |

---

## New Files to Create

### Story 1: State Persistence (5 new files)

#### 1.1 Storage Utility
**File:** `gui/citrate-core/src/utils/storage.ts`
**Type:** TypeScript Utility
**Size Estimate:** ~300 lines
**Purpose:** Centralized localStorage management with type safety

**Key Functions:**
- getStorageItem<T>()
- setStorageItem<T>()
- removeStorageItem()
- clearStorage()
- addRecentAddress()
- exportSettings()
- importSettings()
- migrateStorage()

**Status:** ⏳ Pending

---

#### 1.2 AppContext
**File:** `gui/citrate-core/src/contexts/AppContext.tsx`
**Type:** React Context
**Size Estimate:** ~150 lines
**Purpose:** Global app state management

**Exports:**
- AppContext
- AppProvider
- useApp() hook
- AppState interface

**Status:** ⏳ Pending

---

#### 1.3 Storage Tests
**File:** `gui/citrate-core/src/utils/storage.test.ts`
**Type:** Unit Tests
**Size Estimate:** ~200 lines
**Purpose:** Test storage utility functions

**Test Suites:**
- getStorageItem / setStorageItem
- addRecentAddress
- exportSettings / importSettings
- migrateStorage
- Quota exceeded handling

**Status:** ⏳ Pending

---

#### 1.4 AppContext Tests
**File:** `gui/citrate-core/src/contexts/AppContext.test.tsx`
**Type:** Integration Tests
**Size Estimate:** ~150 lines
**Purpose:** Test AppContext integration

**Test Cases:**
- Persist tab across remount
- Recent addresses management
- Settings auto-save
- Context initialization

**Status:** ⏳ Pending

---

#### 1.5 Storage Types
**File:** `gui/citrate-core/src/types/storage.ts` (Optional)
**Type:** TypeScript Types
**Size Estimate:** ~50 lines
**Purpose:** Shared type definitions for storage

**Status:** ⏳ Pending (Optional)

---

### Story 2: Dark Mode Theme (3 new files)

#### 2.1 Theme Definitions
**File:** `gui/citrate-core/src/styles/themes.ts`
**Type:** TypeScript Constants
**Size Estimate:** ~150 lines
**Purpose:** Light and dark theme definitions

**Exports:**
- Theme interface
- lightTheme
- darkTheme
- themes object
- ThemeMode type
- applyTheme()
- getSystemTheme()

**Status:** ⏳ Pending

---

#### 2.2 ThemeContext
**File:** `gui/citrate-core/src/contexts/ThemeContext.tsx`
**Type:** React Context
**Size Estimate:** ~120 lines
**Purpose:** Theme management context

**Exports:**
- ThemeContext
- ThemeProvider
- useTheme() hook
- ThemeContextType interface

**Status:** ⏳ Pending

---

#### 2.3 Theme Tests
**File:** `gui/citrate-core/src/contexts/ThemeContext.test.tsx`
**Type:** Unit Tests
**Size Estimate:** ~100 lines
**Purpose:** Test theme switching and persistence

**Test Cases:**
- Apply light theme by default
- Toggle theme
- System theme detection
- Theme persistence

**Status:** ⏳ Pending

---

### Story 3: Performance Optimization (4 new files)

#### 3.1 VirtualList Component
**File:** `gui/citrate-core/src/components/VirtualList.tsx`
**Type:** React Component
**Size Estimate:** ~100 lines (custom) or ~50 lines (wrapper)
**Purpose:** Virtual scrolling for long lists

**Props:**
- items: T[]
- height: number
- itemHeight: number
- renderItem: (item: T, index: number) => ReactNode
- overscan?: number

**Alternative:** Use react-window library

**Status:** ⏳ Pending

---

#### 3.2 Debounce Hook
**File:** `gui/citrate-core/src/hooks/useDebounce.ts`
**Type:** React Hook
**Size Estimate:** ~30 lines
**Purpose:** Debounce input values

**Signature:**
```typescript
function useDebounce<T>(value: T, delay?: number): T
```

**Status:** ⏳ Pending

---

#### 3.3 DAG Web Worker
**File:** `gui/citrate-core/src/workers/dagWorker.ts`
**Type:** Web Worker
**Size Estimate:** ~100 lines
**Purpose:** Offload heavy DAG calculations

**Message Types:**
- CALCULATE_BLUE_SET
- TOPOLOGICAL_SORT

**Status:** ⏳ Pending (Optional)

---

#### 3.4 Performance Tests
**File:** `gui/citrate-core/src/__tests__/performance.test.ts`
**Type:** Performance Tests
**Size Estimate:** ~150 lines
**Purpose:** Benchmark rendering and interactions

**Test Cases:**
- Virtual list render time
- Debounced search
- Component memoization
- Worker communication

**Status:** ⏳ Pending

---

### Story 4: Accessibility (3 new files)

#### 4.1 Keyboard Shortcuts Hook
**File:** `gui/citrate-core/src/hooks/useKeyboardShortcuts.ts`
**Type:** React Hook
**Size Estimate:** ~100 lines
**Purpose:** Global keyboard shortcut management

**Exports:**
- KeyboardShortcut interface
- useKeyboardShortcuts() hook
- globalShortcuts array

**Status:** ⏳ Pending

---

#### 4.2 Keyboard Shortcuts Help Modal
**File:** `gui/citrate-core/src/components/KeyboardShortcutsHelp.tsx`
**Type:** React Component
**Size Estimate:** ~80 lines
**Purpose:** Display keyboard shortcuts to user

**Props:**
- isOpen: boolean
- onClose: () => void

**Status:** ⏳ Pending

---

#### 4.3 Accessibility Tests
**File:** `gui/citrate-core/src/__tests__/accessibility.test.tsx`
**Type:** Accessibility Tests
**Size Estimate:** ~100 lines
**Purpose:** Automated a11y testing with jest-axe

**Test Cases:**
- No accessibility violations
- Keyboard navigation
- ARIA labels
- Focus management

**Status:** ⏳ Pending

---

## Files to Modify

### Story 1: State Persistence (2 modifications)

#### M1.1 App.tsx
**File:** `gui/citrate-core/src/App.tsx`
**Changes:**
- Import AppProvider and useApp
- Wrap app with AppProvider
- Replace currentView state with context
- Update tab navigation logic
- Remove local storage code

**Lines Modified:** ~50 lines
**Risk Level:** Medium (core app structure)

**Status:** ⏳ Pending

---

#### M1.2 Wallet.tsx
**File:** `gui/citrate-core/src/components/Wallet.tsx`
**Changes:**
- Import useApp hook
- Add datalist for recent addresses
- Call addRecentAddress on successful send
- Optional: Add clear history button

**Lines Modified:** ~30 lines
**Risk Level:** Low

**Status:** ⏳ Pending

---

### Story 2: Dark Mode Theme (6 modifications)

#### M2.1 App.tsx (Theme)
**File:** `gui/citrate-core/src/App.tsx`
**Changes:**
- Import ThemeProvider
- Wrap app with ThemeProvider
- Ensure provider hierarchy correct

**Lines Modified:** ~10 lines
**Risk Level:** Low

**Status:** ⏳ Pending

---

#### M2.2 App.css
**File:** `gui/citrate-core/src/App.css`
**Changes:**
- Replace all hardcoded colors with CSS variables
- Add smooth transitions for theme changes
- Update root styles
- Add theme-specific overrides

**Lines Modified:** ~100+ lines (extensive changes)
**Risk Level:** High (visual changes)

**Status:** ⏳ Pending

---

#### M2.3 Settings.tsx (Theme Toggle)
**File:** `gui/citrate-core/src/components/Settings.tsx`
**Changes:**
- Import useTheme hook
- Add Appearance section
- Add theme selector buttons (Light/Dark/System)
- Add Sun/Moon/Monitor icons
- Style theme selector

**Lines Modified:** ~50 lines
**Risk Level:** Low

**Status:** ⏳ Pending

---

#### M2.4 Dashboard.css
**File:** `gui/citrate-core/src/components/Dashboard.css` (if exists)
**Changes:**
- Replace hardcoded colors with CSS variables
- Update shadows and borders

**Lines Modified:** ~50 lines
**Risk Level:** Low

**Status:** ⏳ Pending

---

#### M2.5 Wallet.css
**File:** `gui/citrate-core/src/components/Wallet.css` (if exists)
**Changes:**
- Replace hardcoded colors with CSS variables
- Update component-specific styles

**Lines Modified:** ~40 lines
**Risk Level:** Low

**Status:** ⏳ Pending

---

#### M2.6 Other Component Stylesheets
**Files:** Multiple component CSS files
**Changes:**
- Replace hardcoded colors with CSS variables
- Ensure consistent theming

**Affected Files:**
- `gui/citrate-core/src/components/DAGVisualization.css`
- `gui/citrate-core/src/components/Models.css`
- `gui/citrate-core/src/components/FirstTimeSetup.css`
- Any other component stylesheets

**Lines Modified:** ~30 lines each
**Risk Level:** Low per file

**Status:** ⏳ Pending

---

### Story 3: Performance Optimization (3 modifications)

#### M3.1 Wallet.tsx (Virtual Scrolling)
**File:** `gui/citrate-core/src/components/Wallet.tsx`
**Changes:**
- Import VirtualList or react-window
- Replace flat transaction list with virtual list
- Set container height and item height
- Create renderItem function

**Lines Modified:** ~40 lines
**Risk Level:** Medium (rendering changes)

**Status:** ⏳ Pending

---

#### M3.2 DAGVisualization.tsx
**File:** `gui/citrate-core/src/components/DAGVisualization.tsx`
**Changes:**
- Implement lazy loading / pagination
- Add VirtualList to block table
- Add "Load More" or infinite scroll
- Cache loaded blocks

**Lines Modified:** ~80 lines
**Risk Level:** High (complex logic)

**Status:** ⏳ Pending

---

#### M3.3 Dashboard.tsx
**File:** `gui/citrate-core/src/components/Dashboard.tsx`
**Changes:**
- Add React.memo to StatCard components
- Add useMemo for expensive calculations
- Add useCallback for event handlers
- Optimize re-renders

**Lines Modified:** ~50 lines
**Risk Level:** Low

**Status:** ⏳ Pending

---

### Story 4: Accessibility (1 modification)

#### M4.1 All Components (ARIA Labels)
**Files:** All interactive components
**Changes:**
- Add aria-label to buttons
- Add aria-label to icons
- Add role attributes
- Add aria-live regions
- Add landmark roles
- Add skip navigation link

**Affected Files:**
- `gui/citrate-core/src/App.tsx`
- `gui/citrate-core/src/components/Dashboard.tsx`
- `gui/citrate-core/src/components/Wallet.tsx`
- `gui/citrate-core/src/components/DAGVisualization.tsx`
- `gui/citrate-core/src/components/Models.tsx`
- `gui/citrate-core/src/components/Settings.tsx`
- `gui/citrate-core/src/components/FirstTimeSetup.tsx`

**Lines Modified:** ~20 lines per component
**Risk Level:** Low (additive changes)

**Status:** ⏳ Pending

---

## Files to Delete

**No files scheduled for deletion in Sprint 2**

---

## Package Dependencies

### New Dependencies to Add

#### Production Dependencies
```json
{
  "react-window": "^1.8.10"
}
```

**Installation:**
```bash
cd gui/citrate-core
npm install react-window
npm install --save-dev @types/react-window
```

**Purpose:** Virtual scrolling for performance optimization

**Status:** ⏳ Pending

---

#### Development Dependencies
```json
{
  "@types/react-window": "^1.8.8",
  "jest-axe": "^8.0.0"
}
```

**Installation:**
```bash
npm install --save-dev jest-axe
```

**Purpose:** Accessibility testing

**Status:** ⏳ Pending

---

### Optional Dependencies

#### Comlink (Web Worker Helper)
```json
{
  "comlink": "^4.4.1"
}
```

**Installation:**
```bash
npm install comlink
```

**Purpose:** Simplify Web Worker communication

**Status:** ⏳ Optional

---

#### Framer Motion (Animation Library)
```json
{
  "framer-motion": "^10.18.0"
}
```

**Installation:**
```bash
npm install framer-motion
```

**Purpose:** Smooth theme transitions and animations

**Status:** ⏳ Optional

---

## Configuration Files

### Vite Config (if needed)
**File:** `gui/citrate-core/vite.config.ts`
**Changes:**
- Add Web Worker support configuration
- Configure code splitting
- Optimize bundle size

**Status:** ⏳ Check if needed

---

### TypeScript Config (if needed)
**File:** `gui/citrate-core/tsconfig.json`
**Changes:**
- Add Web Worker types
- Ensure strict mode enabled

**Status:** ⏳ Check if needed

---

### Package.json Scripts
**File:** `gui/citrate-core/package.json`
**Changes:**
- Add accessibility test script
- Add performance test script

```json
{
  "scripts": {
    "test:a11y": "vitest run --testNamePattern='accessibility'",
    "test:perf": "vitest run --testNamePattern='performance'"
  }
}
```

**Status:** ⏳ Pending

---

## File Organization After Sprint 2

```
gui/citrate-core/src/
├── components/
│   ├── Dashboard.tsx                    [Modified]
│   ├── Wallet.tsx                       [Modified]
│   ├── DAGVisualization.tsx             [Modified]
│   ├── Models.tsx                       [Modified]
│   ├── Settings.tsx                     [Modified]
│   ├── FirstTimeSetup.tsx               [Modified]
│   ├── VirtualList.tsx                  [NEW]
│   ├── KeyboardShortcutsHelp.tsx        [NEW]
│   ├── ErrorBoundary.tsx                [Existing]
│   └── skeletons/
│       └── ...                          [Existing]
│
├── contexts/
│   ├── AppContext.tsx                   [NEW]
│   └── ThemeContext.tsx                 [NEW]
│
├── hooks/
│   ├── useDebounce.ts                   [NEW]
│   └── useKeyboardShortcuts.ts          [NEW]
│
├── utils/
│   ├── storage.ts                       [NEW]
│   └── validation.ts                    [Existing]
│
├── styles/
│   ├── themes.ts                        [NEW]
│   └── App.css                          [Modified]
│
├── workers/
│   └── dagWorker.ts                     [NEW] [Optional]
│
├── types/
│   └── storage.ts                       [NEW] [Optional]
│
└── __tests__/
    ├── accessibility.test.tsx           [NEW]
    └── performance.test.ts              [NEW]
```

---

## File Modification Log

### Day 1: State Persistence
- [ ] ✅ Created `storage.ts`
- [ ] ✅ Created `storage.test.ts`
- [ ] ✅ Created `AppContext.tsx`
- [ ] ✅ Created `AppContext.test.tsx`
- [ ] ✅ Modified `App.tsx`
- [ ] ✅ Modified `Wallet.tsx`

### Day 2: Dark Mode
- [ ] ✅ Created `themes.ts`
- [ ] ✅ Created `ThemeContext.tsx`
- [ ] ✅ Created `ThemeContext.test.tsx`
- [ ] ✅ Modified `App.tsx` (theme provider)
- [ ] ✅ Modified `App.css`
- [ ] ✅ Modified `Settings.tsx`
- [ ] ✅ Modified component stylesheets

### Day 3: Performance Start
- [ ] ✅ Created `VirtualList.tsx`
- [ ] ✅ Created `useDebounce.ts`
- [ ] ✅ Modified `Wallet.tsx` (virtual scrolling)

### Day 4: Performance Complete
- [ ] ✅ Modified `DAGVisualization.tsx`
- [ ] ✅ Modified `Dashboard.tsx` (memoization)
- [ ] ✅ Created `dagWorker.ts` (optional)
- [ ] ✅ Created `performance.test.ts`

### Day 5: Accessibility
- [ ] ✅ Created `useKeyboardShortcuts.ts`
- [ ] ✅ Created `KeyboardShortcutsHelp.tsx`
- [ ] ✅ Created `accessibility.test.tsx`
- [ ] ✅ Modified all components (ARIA labels)

---

## Backup Strategy

### Before Starting Sprint 2
```bash
# Create backup branch
git checkout -b sprint-1-backup
git push origin sprint-1-backup

# Return to main/develop branch
git checkout main
git checkout -b sprint-2-implementation
```

### Commit Strategy
- Commit after each major file creation/modification
- Use descriptive commit messages
- Reference task numbers in commits

**Example Commits:**
```
git commit -m "S2-T1.1: Create storage utility module"
git commit -m "S2-T1.2: Create AppContext for global state"
git commit -m "S2-T2.1: Create theme definitions"
```

---

## Rollback Plan

### If Major Issues Occur

**Rollback Single File:**
```bash
git checkout HEAD -- gui/citrate-core/src/path/to/file.tsx
```

**Rollback Entire Sprint:**
```bash
git reset --hard sprint-1-backup
```

**Selective Revert:**
```bash
git revert <commit-hash>
```

---

## Code Review Checklist

### Before Marking Files as Complete
- [ ] TypeScript types correct
- [ ] No console.log statements
- [ ] ESLint warnings resolved
- [ ] Prettier formatting applied
- [ ] Tests written and passing
- [ ] Documentation updated
- [ ] No hardcoded values
- [ ] Error handling added
- [ ] Accessibility considerations

---

## File Change Metrics

### Complexity Rating

| File | Lines Added | Lines Modified | Lines Deleted | Complexity | Risk |
|------|-------------|----------------|---------------|------------|------|
| storage.ts | 300 | 0 | 0 | Medium | Low |
| AppContext.tsx | 150 | 0 | 0 | Medium | Low |
| App.tsx | 50 | 50 | 30 | High | Medium |
| App.css | 100 | 100 | 50 | High | High |
| themes.ts | 150 | 0 | 0 | Low | Low |
| ThemeContext.tsx | 120 | 0 | 0 | Medium | Low |
| VirtualList.tsx | 100 | 0 | 0 | Medium | Low |
| useDebounce.ts | 30 | 0 | 0 | Low | Low |
| useKeyboardShortcuts.ts | 100 | 0 | 0 | Medium | Low |
| Wallet.tsx | 70 | 40 | 10 | Medium | Medium |
| DAGVisualization.tsx | 80 | 60 | 20 | High | High |
| Dashboard.tsx | 50 | 30 | 10 | Medium | Low |

---

## Dependencies Between Files

```
storage.ts
  └─> AppContext.tsx
      └─> App.tsx
          └─> All Components

themes.ts
  └─> ThemeContext.tsx
      └─> App.tsx

useDebounce.ts
  └─> Wallet.tsx, DAGVisualization.tsx

VirtualList.tsx
  └─> Wallet.tsx, DAGVisualization.tsx

useKeyboardShortcuts.ts
  └─> App.tsx
  └─> KeyboardShortcutsHelp.tsx
```

---

## Testing Coverage by File

| File | Unit Tests | Integration Tests | E2E Tests |
|------|-----------|------------------|-----------|
| storage.ts | ✅ Required | - | - |
| AppContext.tsx | ✅ Required | ✅ Required | - |
| themes.ts | ⚠️ Optional | - | - |
| ThemeContext.tsx | ✅ Required | - | ⚠️ Manual |
| VirtualList.tsx | ✅ Required | ⚠️ Performance | - |
| useDebounce.ts | ✅ Required | - | - |
| dagWorker.ts | ✅ Required | - | - |
| useKeyboardShortcuts.ts | ✅ Required | - | ⚠️ Manual |
| KeyboardShortcutsHelp.tsx | ⚠️ Optional | - | ⚠️ Manual |

---

## Final Verification

### Before Sprint Close
- [ ] All "NEW" files created
- [ ] All "Modified" files updated
- [ ] All tests passing
- [ ] No untracked files in git
- [ ] No uncommitted changes
- [ ] Build succeeds
- [ ] App runs without errors
- [ ] Manual testing completed

---

**Sprint 2 File Changes Status:** ⏳ Pending → 🚧 In Progress → ✅ Completed

**Total Files Tracked:** 35 files
**Estimated Total Lines Changed:** ~2000+ lines
