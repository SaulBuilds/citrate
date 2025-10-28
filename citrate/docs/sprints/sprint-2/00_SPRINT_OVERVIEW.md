# Sprint 2: Polish & Performance

**Duration:** 1 week (5 working days)
**Story Points:** 13 points
**Sprint Goal:** Enhance user experience with advanced features, performance optimizations, and state persistence

---

## Sprint Objectives

### Primary Goals
1. 🎯 Implement state persistence across app restarts
2. 🎯 Add dark mode theme support
3. 🎯 Optimize rendering performance for large datasets
4. 🎯 Add keyboard shortcuts and accessibility features

### Success Criteria
- [ ] User preferences persist across sessions
- [ ] Smooth dark/light theme toggle with no flicker
- [ ] DAG visualization handles 1000+ blocks without lag
- [ ] All interactive elements keyboard-accessible
- [ ] WCAG 2.1 AA accessibility compliance
- [ ] Performance metrics improved by 30%
- [ ] All tests passing (unit + integration + e2e)

---

## User Stories

### Story 1: State Persistence (5 points)
**As a** user
**I want to** have my preferences and app state saved automatically
**So that** I don't have to reconfigure the app each time I open it

**Acceptance Criteria:**
- Theme preference persisted (dark/light)
- Last viewed tab restored on app start
- Window size and position remembered
- Recent addresses saved for quick access
- Settings automatically saved on change
- Import/export settings functionality

**Files to Create:**
- `gui/citrate-core/src/utils/storage.ts`
- `gui/citrate-core/src/contexts/AppContext.tsx`

**Files to Modify:**
- `gui/citrate-core/src/App.tsx`
- `gui/citrate-core/src/components/Settings.tsx`

---

### Story 2: Dark Mode Theme (3 points)
**As a** user
**I want to** switch between dark and light themes
**So that** I can use the app comfortably in different lighting conditions

**Acceptance Criteria:**
- Toggle switch in Settings
- Smooth transition between themes (no flicker)
- All components support both themes
- System theme auto-detection
- High contrast mode for accessibility
- Theme persists across sessions

**Files to Create:**
- `gui/citrate-core/src/styles/themes.ts`
- `gui/citrate-core/src/contexts/ThemeContext.tsx`

**Files to Modify:**
- `gui/citrate-core/src/App.tsx`
- `gui/citrate-core/src/App.css`
- All component stylesheets

---

### Story 3: Performance Optimization (3 points)
**As a** user
**I want to** experience smooth performance with large datasets
**So that** the app remains responsive during heavy operations

**Acceptance Criteria:**
- Virtual scrolling for long lists (transactions, blocks)
- Lazy loading for DAG visualization
- Debounced search inputs
- Memoized expensive calculations
- Web Workers for heavy computations
- 60fps maintained during interactions

**Files to Create:**
- `gui/citrate-core/src/components/VirtualList.tsx`
- `gui/citrate-core/src/workers/dagWorker.ts`

**Files to Modify:**
- `gui/citrate-core/src/components/DAGVisualization.tsx`
- `gui/citrate-core/src/components/Wallet.tsx`
- `gui/citrate-core/src/components/Dashboard.tsx`

---

### Story 4: Accessibility & Keyboard Shortcuts (2 points)
**As a** user with accessibility needs
**I want to** navigate the app using keyboard and screen readers
**So that** I can use the app regardless of my abilities

**Acceptance Criteria:**
- All interactive elements keyboard-accessible (Tab/Shift+Tab)
- Keyboard shortcuts for common actions (Ctrl+S, Ctrl+K, etc.)
- ARIA labels on all buttons and links
- Focus indicators visible on all elements
- Screen reader announcements for state changes
- Skip navigation links

**Files to Create:**
- `gui/citrate-core/src/hooks/useKeyboardShortcuts.ts`
- `gui/citrate-core/src/components/KeyboardShortcutsHelp.tsx`

**Files to Modify:**
- `gui/citrate-core/src/App.tsx`
- All interactive components

---

## Sprint Backlog (Detailed Tasks)

### Day 1: State Persistence Foundation
- [ ] **Task 1.1:** Create storage utility with localStorage wrapper (2 hours)
- [ ] **Task 1.2:** Create AppContext for global state (2 hours)
- [ ] **Task 1.3:** Implement settings persistence (1.5 hours)
- [ ] **Task 1.4:** Add recent addresses storage (0.5 hours)

**Total Day 1:** 6 hours

---

### Day 2: Dark Mode Implementation
- [ ] **Task 2.1:** Create theme definitions (light/dark) (1.5 hours)
- [ ] **Task 2.2:** Create ThemeContext and provider (1.5 hours)
- [ ] **Task 2.3:** Update CSS variables for theming (2 hours)
- [ ] **Task 2.4:** Add theme toggle to Settings (1 hour)

**Total Day 2:** 6 hours

---

### Day 3: Dark Mode Polish + Performance Start
- [ ] **Task 3.1:** Test and fix dark mode across all components (2 hours)
- [ ] **Task 3.2:** Add system theme detection (1 hour)
- [ ] **Task 3.3:** Create VirtualList component (2 hours)
- [ ] **Task 3.4:** Implement debounced search (1 hour)

**Total Day 3:** 6 hours

---

### Day 4: Performance Optimization
- [ ] **Task 4.1:** Add virtual scrolling to Wallet activity (1.5 hours)
- [ ] **Task 4.2:** Optimize DAG visualization with lazy loading (2 hours)
- [ ] **Task 4.3:** Create Web Worker for DAG calculations (1.5 hours)
- [ ] **Task 4.4:** Add React.memo to heavy components (1 hour)

**Total Day 4:** 6 hours

---

### Day 5: Accessibility & Testing
- [ ] **Task 5.1:** Add keyboard shortcuts hook (2 hours)
- [ ] **Task 5.2:** Implement ARIA labels across components (2 hours)
- [ ] **Task 5.3:** Create keyboard shortcuts help modal (1 hour)
- [ ] **Task 5.4:** Accessibility audit and fixes (2 hours)
- [ ] **Task 5.5:** Performance testing and optimization (1 hour)

**Total Day 5:** 8 hours

---

## Technical Debt Addressed

### Performance Improvements
- ✅ Virtual scrolling eliminates rendering bottlenecks
- ✅ Web Workers prevent UI thread blocking
- ✅ Memoization reduces unnecessary re-renders

### Code Quality
- ✅ Context API for global state management
- ✅ Custom hooks for reusable logic
- ✅ Separation of concerns (storage layer)

### User Experience
- ✅ Dark mode reduces eye strain
- ✅ Keyboard shortcuts increase productivity
- ✅ Accessibility ensures inclusivity

---

## Definition of Done

A story is considered "Done" when:
- [ ] Code is written and follows TypeScript/React best practices
- [ ] All acceptance criteria are met
- [ ] Performance benchmarks met (60fps, < 100ms interactions)
- [ ] Accessibility audit passed (WCAG 2.1 AA)
- [ ] Unit tests written and passing
- [ ] Manual testing completed on all platforms
- [ ] No console errors or warnings
- [ ] Code reviewed by another team member
- [ ] Documentation updated
- [ ] Merged to main branch

---

## Risk Register

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| Dark mode CSS variables conflict with existing styles | Medium | Medium | Test incrementally, use scoped variables |
| Virtual scrolling breaks existing layouts | Low | High | Maintain same DOM structure, test thoroughly |
| Web Workers add complexity without benefits | Low | Low | Benchmark before and after, keep fallback |
| Keyboard shortcuts conflict with browser defaults | Medium | Low | Use Ctrl+Shift combinations, document clearly |
| State persistence causes data corruption | Low | High | Add version checking, validation on load |

---

## Dependencies

### External Libraries Needed
- `react-window` or `react-virtuoso` (virtual scrolling)
- `framer-motion` (smooth animations)
- `comlink` (Web Worker abstraction) - Optional

### Blocked By
- Sprint 1 completion (foundation security features)

### Blocking
- Sprint 3 (Advanced Features) benefits from performance optimizations

---

## Sprint Metrics

### Planned Capacity
- **Team Size:** 1 developer
- **Available Hours:** 32 hours (6-8 hours/day × 5 days)
- **Story Points:** 13 points
- **Velocity:** 13 points/week

### Tracking
- **Daily Standup:** Update task completion in checklist
- **Burndown Chart:** Track remaining story points daily
- **Performance Metrics:** Track FPS, render times, bundle size
- **Blockers:** Document in this file if any arise

---

## Sprint Review Agenda

1. Demo all completed user stories
   - Show state persistence across restart
   - Demo dark mode toggle
   - Performance comparison (before/after)
   - Keyboard shortcuts walkthrough
2. Review acceptance criteria completion
3. Performance metrics review
4. Accessibility audit results
5. Discuss what went well
6. Discuss what could be improved
7. Carry over any incomplete work to Sprint 3
8. Celebrate wins! 🎉

---

## Performance Benchmarks

### Before Sprint 2
- Dashboard render: ~150ms
- Wallet activity (100 items): ~80ms
- DAG visualization (500 blocks): ~300ms
- Bundle size: 2.5MB

### Target After Sprint 2
- Dashboard render: <100ms (33% improvement)
- Wallet activity (1000 items): <100ms
- DAG visualization (1000 blocks): <200ms
- Bundle size: <2.2MB (12% reduction)

---

## Related Documentation

- [Sprint 1 Completion Report](../sprint-1/05_IMPLEMENTATION_LOG.md)
- [User Stories Details](./01_USER_STORIES.md)
- [Technical Tasks](./02_TECHNICAL_TASKS.md)
- [File Changes Tracking](./03_FILE_CHANGES.md)
- [Testing Checklist](./04_TESTING_CHECKLIST.md)
- [Implementation Log](./05_IMPLEMENTATION_LOG.md)

---

**Sprint Start Date:** February 4, 2026
**Sprint End Date:** February 8, 2026
**Sprint Status:** 🔵 Ready to Start
