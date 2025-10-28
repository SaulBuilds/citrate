# Sprint 1: Foundation & Security

**Duration:** 1 week (5 working days)
**Story Points:** 13 points
**Sprint Goal:** Fix critical security issues and establish quality baseline for production readiness

---

## Sprint Objectives

### Primary Goals
1. ✅ Remove hardcoded password from FirstTimeSetup component
2. ✅ Implement comprehensive input validation across all forms
3. ✅ Add React error boundaries for graceful error handling
4. ✅ Replace loading spinners with content-aware skeletons

### Success Criteria
- [ ] Zero hardcoded credentials in codebase
- [ ] All user inputs validated with clear error messages
- [ ] Application doesn't crash on errors, shows user-friendly messages
- [ ] Loading states provide smooth, informative user experience
- [ ] All tests passing (unit + integration)
- [ ] Code review completed and approved

---

## User Stories

### Story 1: Secure Password Management (3 points)
**As a** new user
**I want to** set my own secure password during wallet setup
**So that** my funds are protected with a password only I know

**Acceptance Criteria:**
- Password input field with show/hide toggle
- Password confirmation field
- Minimum 8 characters required
- Strength indicator (weak/medium/strong)
- Password stored securely in OS keyring
- No hardcoded passwords in code

**Files to Modify:**
- `gui/citrate-core/src/components/FirstTimeSetup.tsx`
- `gui/citrate-core/src-tauri/src/wallet/mod.rs`

---

### Story 2: Input Validation (5 points)
**As a** user
**I want to** receive clear feedback when I enter invalid data
**So that** I don't make mistakes that could result in lost funds

**Acceptance Criteria:**
- Address validation with checksum verification
- Amount validation (positive numbers, max supply check)
- Gas limit validation (reasonable bounds)
- Private key validation (correct format and length)
- Mnemonic validation (BIP39 wordlist check)
- Clear, specific error messages
- Red border on invalid fields
- Error text below each field

**Files to Modify:**
- `gui/citrate-core/src/components/Wallet.tsx`
- `gui/citrate-core/src/components/FirstTimeSetup.tsx`
- `gui/citrate-core/src/components/Settings.tsx`
- Create new: `gui/citrate-core/src/utils/validation.ts`

---

### Story 3: Error Boundaries (2 points)
**As a** user
**I want to** see helpful error messages when something goes wrong
**So that** I can understand what happened and potentially recover

**Acceptance Criteria:**
- React error boundary wrapping entire app
- User-friendly error page with "Reload" button
- Error details logged to console for debugging
- Fallback UI doesn't show stack traces to users
- Error state clears on navigation

**Files to Create:**
- `gui/citrate-core/src/components/ErrorBoundary.tsx`

**Files to Modify:**
- `gui/citrate-core/src/App.tsx`

---

### Story 4: Loading Skeletons (3 points)
**As a** user
**I want to** see content placeholders while data loads
**So that** the app feels faster and I know what to expect

**Acceptance Criteria:**
- Skeleton loaders for Dashboard (node status, block height)
- Skeleton loaders for Wallet (account list, balances)
- Skeleton loaders for DAG (block table)
- Skeleton loaders for Models (model list)
- Smooth transition from skeleton to actual content
- Skeletons match the shape of real content

**Files to Create:**
- `gui/citrate-core/src/components/skeletons/DashboardSkeleton.tsx`
- `gui/citrate-core/src/components/skeletons/WalletSkeleton.tsx`
- `gui/citrate-core/src/components/skeletons/TableSkeleton.tsx`
- `gui/citrate-core/src/components/skeletons/CardSkeleton.tsx`

**Files to Modify:**
- `gui/citrate-core/src/components/Dashboard.tsx`
- `gui/citrate-core/src/components/Wallet.tsx`
- `gui/citrate-core/src/components/DAGVisualization.tsx`
- `gui/citrate-core/src/components/Models.tsx`

---

## Sprint Backlog (Detailed Tasks)

### Day 1: Password Security
- [ ] **Task 1.1:** Remove hardcoded password constant (30 min)
- [ ] **Task 1.2:** Add password input fields to FirstTimeSetup UI (1 hour)
- [ ] **Task 1.3:** Implement password strength indicator (1.5 hours)
- [ ] **Task 1.4:** Update backend to accept user password (1 hour)
- [ ] **Task 1.5:** Test password creation flow (1 hour)
- [ ] **Task 1.6:** Code review and fixes (1 hour)

**Total Day 1:** 6 hours

---

### Day 2: Input Validation (Part 1)
- [ ] **Task 2.1:** Create validation utility functions (2 hours)
  - validateAddress()
  - validateAmount()
  - validateGasLimit()
  - validatePrivateKey()
  - validateMnemonic()

- [ ] **Task 2.2:** Add validation to Wallet send transaction form (2 hours)
- [ ] **Task 2.3:** Add validation to Wallet import account forms (2 hours)

**Total Day 2:** 6 hours

---

### Day 3: Input Validation (Part 2) + Error Boundaries
- [ ] **Task 3.1:** Add validation to FirstTimeSetup mnemonic display (1 hour)
- [ ] **Task 3.2:** Add validation to Settings bootnode/peer inputs (1.5 hours)
- [ ] **Task 3.3:** Create ErrorBoundary component (1.5 hours)
- [ ] **Task 3.4:** Wrap App with ErrorBoundary (0.5 hours)
- [ ] **Task 3.5:** Test error boundary with intentional errors (1.5 hours)

**Total Day 3:** 6 hours

---

### Day 4: Loading Skeletons
- [ ] **Task 4.1:** Create skeleton components (2.5 hours)
  - DashboardSkeleton
  - WalletSkeleton
  - TableSkeleton
  - CardSkeleton

- [ ] **Task 4.2:** Integrate skeletons into Dashboard (1 hour)
- [ ] **Task 4.3:** Integrate skeletons into Wallet (1 hour)
- [ ] **Task 4.4:** Integrate skeletons into DAG (1 hour)
- [ ] **Task 4.5:** Integrate skeletons into Models (0.5 hours)

**Total Day 4:** 6 hours

---

### Day 5: Testing, Documentation & Sprint Review
- [ ] **Task 5.1:** Write unit tests for validation functions (2 hours)
- [ ] **Task 5.2:** Manual testing of all modified components (2 hours)
- [ ] **Task 5.3:** Fix bugs found during testing (1.5 hours)
- [ ] **Task 5.4:** Update documentation (0.5 hours)
- [ ] **Task 5.5:** Sprint review and retrospective (1 hour)

**Total Day 5:** 7 hours

---

## Technical Debt Addressed

### Security Improvements
- ✅ Removed hardcoded password vulnerability
- ✅ Added input sanitization to prevent injection attacks
- ✅ Implemented secure password storage

### Code Quality
- ✅ Centralized validation logic (DRY principle)
- ✅ Added TypeScript interfaces for validation results
- ✅ Improved error handling across components

### User Experience
- ✅ Clearer error messages
- ✅ Visual feedback for loading states
- ✅ Password strength guidance

---

## Definition of Done

A story is considered "Done" when:
- [ ] Code is written and follows TypeScript/React best practices
- [ ] All acceptance criteria are met
- [ ] Unit tests written and passing
- [ ] Manual testing completed
- [ ] No console errors or warnings
- [ ] Code reviewed by another team member
- [ ] Documentation updated
- [ ] Merged to main branch

---

## Risk Register

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| Password storage library incompatible with Tauri | Low | High | Research Tauri keyring support before implementation |
| Validation regex too strict, blocks valid inputs | Medium | Medium | Test with diverse input samples |
| Skeletons don't match content layout | Low | Low | Use same CSS classes as actual content |
| Breaking changes to wallet API | Low | High | Keep backward compatibility, test thoroughly |

---

## Dependencies

### External Libraries Needed
- `zxcvbn` (password strength estimation) - Already available in Rust
- `react-loading-skeleton` (optional, can build custom)
- Tauri `keyring` plugin (for secure password storage)

### Blocked By
- None (sprint can start immediately)

### Blocking
- Sprint 2 (IPFS) depends on input validation for file metadata

---

## Sprint Metrics

### Planned Capacity
- **Team Size:** 1 developer
- **Available Hours:** 30 hours (6 hours/day × 5 days)
- **Story Points:** 13 points
- **Velocity:** 13 points/week

### Tracking
- **Daily Standup:** Update task completion in checklist
- **Burndown Chart:** Track remaining story points daily
- **Blockers:** Document in this file if any arise

---

## Sprint Review Agenda

1. Demo all completed user stories
2. Review acceptance criteria completion
3. Discuss what went well
4. Discuss what could be improved
5. Carry over any incomplete work to Sprint 2
6. Celebrate wins! 🎉

---

## Related Documentation

- [GUI Audit Report](../GUI_AUDIT_2026_Q1.md)
- [User Stories Details](./01_USER_STORIES.md)
- [Technical Tasks](./02_TECHNICAL_TASKS.md)
- [File Changes Tracking](./03_FILE_CHANGES.md)
- [Testing Checklist](./04_TESTING_CHECKLIST.md)
- [Implementation Log](./05_IMPLEMENTATION_LOG.md)

---

**Sprint Start Date:** January 28, 2026
**Sprint End Date:** February 1, 2026
**Sprint Status:** 🟡 Planning
