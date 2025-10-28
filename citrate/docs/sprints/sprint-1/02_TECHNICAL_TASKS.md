# Sprint 1: Technical Tasks Breakdown

**Sprint Goal:** Fix critical security issues and establish quality baseline for production readiness

**Total Story Points:** 13 points
**Duration:** 5 working days (30 hours)

---

## Task Completion Tracking

### ✅ Completed | 🚧 In Progress | ⏳ Pending | ❌ Blocked

---

## Day 1: Password Security (6 hours)

### Story 1: Secure Password Management (3 points)

#### Task 1.1: Remove Hardcoded Password Constant (30 min) ⏳
**File:** `gui/citrate-core/src/components/FirstTimeSetup.tsx`

**Changes Required:**
- [ ] Remove line 52: `password: 'user_secure_password'`
- [ ] Add password state management
- [ ] Add confirmPassword state management

**Dependencies:** None

**Estimated Time:** 30 minutes

**Acceptance:**
- [ ] No hardcoded password strings in file
- [ ] Password state properly initialized
- [ ] Confirm password state properly initialized

---

#### Task 1.2: Add Password Input Fields to FirstTimeSetup UI (1 hour) ⏳
**File:** `gui/citrate-core/src/components/FirstTimeSetup.tsx`

**Changes Required:**
- [ ] Add password input field with label
- [ ] Add confirm password input field
- [ ] Add show/hide toggle button (Eye/EyeOff icons)
- [ ] Add visual styling (orange theme)
- [ ] Add onChange handlers
- [ ] Add showPassword state toggle

**UI Components to Add:**
```typescript
- Password input (type="password" or "text" based on showPassword)
- Confirm password input
- Show/hide toggle button
- Field labels
- Error message display area
```

**Dependencies:** Task 1.1

**Estimated Time:** 1 hour

**Acceptance:**
- [ ] Password field renders correctly
- [ ] Confirm password field renders correctly
- [ ] Show/hide toggle works
- [ ] Orange theme applied consistently
- [ ] Fields are accessible (labels, aria-labels)

---

#### Task 1.3: Implement Password Strength Indicator (1.5 hours) ⏳
**File:** `gui/citrate-core/src/components/FirstTimeSetup.tsx`

**Changes Required:**
- [ ] Create `calculatePasswordStrength()` function
- [ ] Add strength indicator UI component
- [ ] Add real-time strength calculation on input
- [ ] Add color coding (red/yellow/orange/green)
- [ ] Add strength text labels (Weak/Fair/Good/Strong)
- [ ] Add password requirements list

**Strength Calculation Rules:**
```typescript
- Length >= 8 characters: +1 point
- Length >= 12 characters: +1 point
- Mixed case (a-z, A-Z): +1 point
- Contains digits (0-9): +1 point
- Contains special characters: +1 point
- Total: 0-5 points mapped to Weak/Fair/Good/Strong
```

**Dependencies:** Task 1.2

**Estimated Time:** 1.5 hours

**Acceptance:**
- [ ] Strength indicator displays correctly
- [ ] Strength updates in real-time as user types
- [ ] Color coding is correct and accessible
- [ ] Requirements list is helpful and accurate
- [ ] Edge cases handled (empty password, very long password)

---

#### Task 1.4: Update Backend to Accept User Password (1 hour) ⏳
**File:** `gui/citrate-core/src-tauri/src/wallet/mod.rs`

**Changes Required:**
- [ ] Review `perform_first_time_setup` command signature
- [ ] Verify password parameter is properly typed (String)
- [ ] Ensure password is passed to keyring storage
- [ ] Add password validation (minimum length check)
- [ ] Add error handling for weak passwords

**Security Checklist:**
- [ ] Password not logged to console
- [ ] Password not stored in plaintext
- [ ] Password properly passed to OS keyring
- [ ] Error messages don't leak password info

**Dependencies:** None (can be done in parallel with UI tasks)

**Estimated Time:** 1 hour

**Acceptance:**
- [ ] Backend accepts user-provided password
- [ ] Password stored securely in OS keyring
- [ ] No plaintext password storage
- [ ] Error handling works correctly

---

#### Task 1.5: Test Password Creation Flow (1 hour) ⏳
**Testing:** Manual + Automated

**Test Cases:**
- [ ] Create wallet with strong password (12+ chars, mixed case, symbols)
- [ ] Create wallet with medium password (8+ chars, mixed case)
- [ ] Attempt weak password (should show warning but allow?)
- [ ] Test password mismatch (confirm password different)
- [ ] Test empty password
- [ ] Test very long password (100+ chars)
- [ ] Test show/hide toggle
- [ ] Test strength indicator accuracy
- [ ] Verify password stored in keyring (check OS keyring manager)
- [ ] Test wallet unlock with created password

**Dependencies:** Tasks 1.1-1.4

**Estimated Time:** 1 hour

**Acceptance:**
- [ ] All test cases pass
- [ ] No console errors
- [ ] Password properly stored and retrievable
- [ ] UX is smooth and intuitive

---

#### Task 1.6: Code Review and Fixes (1 hour) ⏳
**Review Checklist:**
- [ ] Code follows TypeScript best practices
- [ ] No hardcoded passwords anywhere
- [ ] Error handling is comprehensive
- [ ] UI is accessible (keyboard navigation, screen readers)
- [ ] Orange theme consistently applied
- [ ] No security vulnerabilities
- [ ] Code is well-commented
- [ ] State management is clean

**Dependencies:** Tasks 1.1-1.5

**Estimated Time:** 1 hour

**Acceptance:**
- [ ] Code review completed
- [ ] All issues resolved
- [ ] Story 1 fully completed and tested

---

## Day 2: Input Validation Part 1 (6 hours)

### Story 2: Input Validation (5 points)

#### Task 2.1: Create Validation Utility Functions (2 hours) ⏳
**File:** `gui/citrate-core/src/utils/validation.ts` (NEW FILE)

**Functions to Create:**
- [ ] `validateAddress(address: string): ValidationResult`
- [ ] `validateAmount(amount: string, balance?: string, maxSupply?: string): ValidationResult`
- [ ] `validateGasLimit(gasLimit: string): ValidationResult`
- [ ] `validatePrivateKey(privateKey: string): ValidationResult`
- [ ] `validateMnemonic(mnemonic: string): ValidationResult`

**Supporting Types:**
- [ ] `interface ValidationResult { isValid: boolean; error?: string; }`

**Validation Rules:**
```typescript
Address:
- Must be 40 hex characters (with or without 0x prefix)
- Optionally: checksum validation

Amount:
- Must be a valid number
- Must be positive (> 0)
- Must not exceed max supply
- Must not exceed user balance (if provided)

Gas Limit:
- Must be valid integer
- Minimum: 21,000
- Maximum: 10,000,000

Private Key:
- Must be 64 hex characters (with or without 0x prefix)
- All characters must be valid hexadecimal

Mnemonic:
- Must be 12 or 24 words
- Words separated by whitespace
- Optionally: BIP39 wordlist validation
```

**Dependencies:** None

**Estimated Time:** 2 hours

**Acceptance:**
- [ ] All 5 validation functions created
- [ ] ValidationResult interface properly typed
- [ ] All validation rules implemented correctly
- [ ] Edge cases handled (null, undefined, empty strings)
- [ ] Clear, specific error messages
- [ ] Unit tests written (see Day 5)

---

#### Task 2.2: Add Validation to Wallet Send Transaction Form (2 hours) ⏳
**File:** `gui/citrate-core/src/components/Wallet.tsx`

**Changes Required:**
- [ ] Import validation functions
- [ ] Add error state for recipient address
- [ ] Add error state for amount
- [ ] Add error state for gas limit
- [ ] Add real-time validation on input change
- [ ] Add visual error indicators (red border)
- [ ] Add error text below each field
- [ ] Disable send button when validation fails
- [ ] Add helpful placeholder text

**Form Fields to Validate:**
1. Recipient Address
2. Amount
3. Gas Limit (if manually set)

**UI Changes:**
```typescript
- Red border on invalid fields: border-color: #ef4444
- Error text: text-red-600, text-sm
- Disabled button: opacity-50, cursor-not-allowed
- Valid field indicator (optional): border-color: #ffa500
```

**Dependencies:** Task 2.1

**Estimated Time:** 2 hours

**Acceptance:**
- [ ] All fields validated in real-time
- [ ] Clear error messages displayed
- [ ] Invalid inputs prevent form submission
- [ ] Valid inputs enable form submission
- [ ] UX is smooth (validation not too aggressive)
- [ ] No console errors

---

#### Task 2.3: Add Validation to Wallet Import Account Forms (2 hours) ⏳
**File:** `gui/citrate-core/src/components/Wallet.tsx`

**Changes Required:**

**Import from Mnemonic Form:**
- [ ] Validate mnemonic phrase (12 or 24 words)
- [ ] Add error state for mnemonic input
- [ ] Add visual error indicators
- [ ] Show word count (e.g., "12/12 words")
- [ ] Disable import button when invalid

**Import from Private Key Form:**
- [ ] Validate private key format (64 hex chars)
- [ ] Add error state for private key input
- [ ] Add visual error indicators
- [ ] Add show/hide toggle for private key
- [ ] Disable import button when invalid

**Import from WIF Form (if exists):**
- [ ] Validate WIF format
- [ ] Add appropriate error handling

**Dependencies:** Task 2.1

**Estimated Time:** 2 hours

**Acceptance:**
- [ ] Mnemonic validation works correctly
- [ ] Private key validation works correctly
- [ ] Clear error messages for each format
- [ ] Import buttons disabled when invalid
- [ ] Visual feedback is clear and helpful
- [ ] No security issues (private key handling)

---

## Day 3: Input Validation Part 2 + Error Boundaries (6 hours)

### Story 2: Input Validation (continued)

#### Task 3.1: Add Validation to FirstTimeSetup Mnemonic Display (1 hour) ⏳
**File:** `gui/citrate-core/src/components/FirstTimeSetup.tsx`

**Changes Required:**
- [ ] Add mnemonic confirmation step (user types mnemonic back)
- [ ] Validate user-entered mnemonic matches generated mnemonic
- [ ] Add character-by-character or word-by-word matching
- [ ] Add visual feedback for correct/incorrect words
- [ ] Add "I have saved my recovery phrase" checkbox
- [ ] Require checkbox + validation to proceed

**UI Enhancements:**
- [ ] Display mnemonic in grid (3x4 or 6x4 for 12/24 words)
- [ ] Number each word (1. word1, 2. word2, etc.)
- [ ] Add "Copy to Clipboard" button
- [ ] Add copy confirmation message
- [ ] Warning: "Never share this phrase with anyone"

**Dependencies:** Task 2.1

**Estimated Time:** 1 hour

**Acceptance:**
- [ ] Mnemonic displayed clearly in grid
- [ ] Copy to clipboard works
- [ ] Confirmation checkbox required
- [ ] User cannot proceed without confirming
- [ ] Security warnings are prominent

---

#### Task 3.2: Add Validation to Settings Bootnode/Peer Inputs (1.5 hours) ⏳
**File:** `gui/citrate-core/src/components/Settings.tsx`

**Changes Required:**

**Bootnode Validation:**
- [ ] Create `validateBootnode(bootnode: string): ValidationResult` function
- [ ] Validate format: `/ip4/x.x.x.x/tcp/port/p2p/peerID` or multiaddr format
- [ ] Add error state for bootnode input
- [ ] Add visual error indicators
- [ ] Validate IP address format
- [ ] Validate port range (1-65535)
- [ ] Validate peer ID format

**Peer Validation:**
- [ ] Similar validation for manual peer adding
- [ ] Validate multiaddr format
- [ ] Add error handling

**Utility Function to Add:**
```typescript
// In validation.ts
export function validateBootnode(bootnode: string): ValidationResult {
  // Regex or manual parsing for multiaddr format
  // Example: /ip4/127.0.0.1/tcp/9000/p2p/12D3KooWABC...
}

export function validateIPv4(ip: string): ValidationResult {
  // Validate IPv4 address
}

export function validatePort(port: string): ValidationResult {
  // Validate port number (1-65535)
}
```

**Dependencies:** Task 2.1

**Estimated Time:** 1.5 hours

**Acceptance:**
- [ ] Bootnode format validated correctly
- [ ] Invalid bootnodes rejected with clear error
- [ ] Valid bootnodes accepted
- [ ] Settings save button disabled when invalid
- [ ] Help text explains expected format

---

### Story 3: Error Boundaries (2 points)

#### Task 3.3: Create ErrorBoundary Component (1.5 hours) ⏳
**File:** `gui/citrate-core/src/components/ErrorBoundary.tsx` (NEW FILE)

**Component Requirements:**
- [ ] Class component (Error Boundaries require class syntax)
- [ ] Implement `componentDidCatch(error, errorInfo)`
- [ ] Implement `getDerivedStateFromError(error)`
- [ ] State: `{ hasError: boolean; error: Error | null }`
- [ ] Render fallback UI when error occurs
- [ ] Log errors to console for debugging
- [ ] Provide "Reload Page" button
- [ ] Provide "Report Issue" button (optional)
- [ ] Clear error state on navigation (if applicable)

**Fallback UI Design:**
```typescript
- Centered error page
- Icon: AlertTriangle (Lucide icon)
- Heading: "Something went wrong"
- Subheading: "The application encountered an unexpected error"
- Error message (in development mode only)
- Reload button (primary action, orange)
- Report Issue button (secondary action)
- Back to Dashboard link (if applicable)
```

**Error Handling:**
- [ ] Console.error() the error and error info
- [ ] Don't show stack traces to end users (only in dev mode)
- [ ] Provide helpful, user-friendly error messages

**Dependencies:** None

**Estimated Time:** 1.5 hours

**Acceptance:**
- [ ] Component properly catches errors
- [ ] Fallback UI is clear and helpful
- [ ] Reload button works
- [ ] No infinite error loops
- [ ] Development mode shows more details
- [ ] Production mode shows minimal info

---

#### Task 3.4: Wrap App with ErrorBoundary (0.5 hours) ⏳
**File:** `gui/citrate-core/src/App.tsx`

**Changes Required:**
- [ ] Import ErrorBoundary component
- [ ] Wrap entire app content with `<ErrorBoundary>`
- [ ] Ensure router is inside error boundary
- [ ] Test that errors in any component are caught

**Code Change:**
```typescript
// Before
function App() {
  return (
    <div className="App">
      {/* content */}
    </div>
  );
}

// After
function App() {
  return (
    <ErrorBoundary>
      <div className="App">
        {/* content */}
      </div>
    </ErrorBoundary>
  );
}
```

**Dependencies:** Task 3.3

**Estimated Time:** 0.5 hours

**Acceptance:**
- [ ] ErrorBoundary wraps all app content
- [ ] Errors in any component are caught
- [ ] Navigation still works after errors
- [ ] No console errors from ErrorBoundary itself

---

#### Task 3.5: Test Error Boundary with Intentional Errors (1.5 hours) ⏳
**Testing:** Manual testing with intentional errors

**Test Cases:**
- [ ] **Test 1:** Throw error in component render (add test button)
- [ ] **Test 2:** Throw error in useEffect hook
- [ ] **Test 3:** Throw error in event handler (should NOT be caught)
- [ ] **Test 4:** Simulate network error in async operation
- [ ] **Test 5:** Test reload button functionality
- [ ] **Test 6:** Test error in Dashboard component
- [ ] **Test 7:** Test error in Wallet component
- [ ] **Test 8:** Verify error doesn't propagate to console in production build

**Testing Approach:**
```typescript
// Temporary test button to add
<button onClick={() => { throw new Error("Test error"); }}>
  Trigger Error (Test)
</button>

// Or add to a component temporarily:
useEffect(() => {
  if (testError) {
    throw new Error("Test error in useEffect");
  }
}, [testError]);
```

**Dependencies:** Task 3.4

**Estimated Time:** 1.5 hours

**Acceptance:**
- [ ] All test cases executed
- [ ] Error boundary catches render errors
- [ ] Error boundary catches lifecycle errors
- [ ] Event handler errors handled gracefully (maybe separate try-catch)
- [ ] Reload button clears error state
- [ ] User can recover from errors
- [ ] Remove all test code after testing

---

## Day 4: Loading Skeletons (6 hours)

### Story 4: Loading Skeletons (3 points)

#### Task 4.1: Create Skeleton Components (2.5 hours) ⏳

##### 4.1a: Base Skeleton Component (30 min)
**File:** `gui/citrate-core/src/components/skeletons/Skeleton.tsx` (NEW FILE)

**Component Requirements:**
- [ ] Reusable skeleton base component
- [ ] Props: `width`, `height`, `className` (optional)
- [ ] Shimmer animation effect
- [ ] Rounded corners
- [ ] Gray background (#e5e7eb)

**CSS Animation:**
```css
@keyframes shimmer {
  0% { background-position: -1000px 0; }
  100% { background-position: 1000px 0; }
}
```

**Acceptance:**
- [ ] Skeleton renders correctly
- [ ] Shimmer animation is smooth
- [ ] Props work correctly
- [ ] Reusable across all components

---

##### 4.1b: DashboardSkeleton Component (45 min)
**File:** `gui/citrate-core/src/components/skeletons/DashboardSkeleton.tsx` (NEW FILE)

**Structure to Match:**
- [ ] Header skeleton (title area)
- [ ] Stats grid skeleton (4 stat cards)
- [ ] Recent blocks table skeleton (5 rows)
- [ ] Network status skeleton
- [ ] Charts skeleton (if applicable)

**Layout:**
```typescript
- Page title: Skeleton (width: 200px, height: 32px)
- Stats grid: 4 cards (each with icon area + text lines)
  - Icon: Skeleton (width: 40px, height: 40px, rounded)
  - Label: Skeleton (width: 120px, height: 16px)
  - Value: Skeleton (width: 80px, height: 24px)
- Recent blocks: TableSkeleton (5 rows, 6 columns)
```

**Acceptance:**
- [ ] Matches Dashboard layout
- [ ] Smooth transition from skeleton to real data
- [ ] Responsive design

---

##### 4.1c: WalletSkeleton Component (45 min)
**File:** `gui/citrate-core/src/components/skeletons/WalletSkeleton.tsx` (NEW FILE)

**Structure to Match:**
- [ ] Account selector skeleton
- [ ] Balance display skeleton
- [ ] Account list skeleton (3 account cards)
- [ ] Action buttons skeleton

**Layout:**
```typescript
- Account selector: Skeleton (width: 100%, height: 48px)
- Balance:
  - Label: Skeleton (width: 60px, height: 16px)
  - Value: Skeleton (width: 150px, height: 32px)
- Account cards: 3 × CardSkeleton
  - Address: Skeleton (width: 300px, height: 20px)
  - Balance: Skeleton (width: 100px, height: 24px)
```

**Acceptance:**
- [ ] Matches Wallet layout
- [ ] Account cards look realistic
- [ ] Responsive design

---

##### 4.1d: TableSkeleton Component (45 min)
**File:** `gui/citrate-core/src/components/skeletons/TableSkeleton.tsx` (NEW FILE)

**Component Requirements:**
- [ ] Reusable table skeleton
- [ ] Props: `rows`, `columns`
- [ ] Header row skeleton
- [ ] Data rows skeleton
- [ ] Column width variation

**Structure:**
```typescript
interface TableSkeletonProps {
  rows?: number; // default: 5
  columns?: number; // default: 4
  columnWidths?: string[]; // optional custom widths
}
```

**Acceptance:**
- [ ] Renders table structure correctly
- [ ] Configurable rows and columns
- [ ] Matches table styling
- [ ] Reusable for DAG, Models, etc.

---

##### 4.1e: CardSkeleton Component (15 min)
**File:** `gui/citrate-core/src/components/skeletons/CardSkeleton.tsx` (NEW FILE)

**Component Requirements:**
- [ ] Simple card skeleton
- [ ] Props: `lines` (number of text lines)
- [ ] Rounded card border
- [ ] Padding matches real cards

**Structure:**
```typescript
interface CardSkeletonProps {
  lines?: number; // default: 3
  hasImage?: boolean; // optional image area
}
```

**Acceptance:**
- [ ] Renders card correctly
- [ ] Configurable line count
- [ ] Matches card styling

---

#### Task 4.2: Integrate Skeletons into Dashboard (1 hour) ⏳
**File:** `gui/citrate-core/src/components/Dashboard.tsx`

**Changes Required:**
- [ ] Import DashboardSkeleton
- [ ] Add loading state: `const [isLoading, setIsLoading] = useState(true)`
- [ ] Show skeleton while `isLoading === true`
- [ ] Show real content when `isLoading === false`
- [ ] Transition smoothly from skeleton to content
- [ ] Simulate loading delay in development (if needed)

**Implementation Pattern:**
```typescript
if (isLoading) {
  return <DashboardSkeleton />;
}

return (
  <div className="dashboard">
    {/* Real content */}
  </div>
);
```

**Dependencies:** Task 4.1b

**Estimated Time:** 1 hour

**Acceptance:**
- [ ] Skeleton shows while loading
- [ ] Real content appears after load
- [ ] Smooth transition (no layout shift)
- [ ] No console errors
- [ ] Loading state managed correctly

---

#### Task 4.3: Integrate Skeletons into Wallet (1 hour) ⏳
**File:** `gui/citrate-core/src/components/Wallet.tsx`

**Changes Required:**
- [ ] Import WalletSkeleton
- [ ] Add loading state for accounts
- [ ] Show skeleton while loading accounts
- [ ] Show skeleton while loading balance
- [ ] Transition to real data smoothly

**Loading States:**
```typescript
const [isLoadingAccounts, setIsLoadingAccounts] = useState(true);
const [isLoadingBalance, setIsLoadingBalance] = useState(true);
```

**Dependencies:** Task 4.1c

**Estimated Time:** 1 hour

**Acceptance:**
- [ ] Skeleton shows while loading
- [ ] Account list loads smoothly
- [ ] Balance loads smoothly
- [ ] No layout shift during transition

---

#### Task 4.4: Integrate Skeletons into DAG (1 hour) ⏳
**File:** `gui/citrate-core/src/components/DAGVisualization.tsx`

**Changes Required:**
- [ ] Import TableSkeleton
- [ ] Add loading state for block data
- [ ] Show table skeleton while loading
- [ ] Configure skeleton for 6 columns (hash, height, timestamp, etc.)
- [ ] Show 10 skeleton rows

**Dependencies:** Task 4.1d

**Estimated Time:** 1 hour

**Acceptance:**
- [ ] Table skeleton matches DAG table structure
- [ ] Smooth transition to real data
- [ ] Column widths match
- [ ] No layout shift

---

#### Task 4.5: Integrate Skeletons into Models (0.5 hours) ⏳
**File:** `gui/citrate-core/src/components/Models.tsx`

**Changes Required:**
- [ ] Import CardSkeleton or TableSkeleton
- [ ] Add loading state
- [ ] Show skeleton grid (3 columns)
- [ ] Show 6-9 card skeletons

**Dependencies:** Task 4.1e

**Estimated Time:** 0.5 hours

**Acceptance:**
- [ ] Model cards skeleton matches layout
- [ ] Grid layout maintained
- [ ] Smooth transition

---

## Day 5: Testing, Documentation & Sprint Review (7 hours)

#### Task 5.1: Write Unit Tests for Validation Functions (2 hours) ⏳
**File:** `gui/citrate-core/src/utils/validation.test.ts` (NEW FILE)

**Test Framework:** Vitest (already configured in Vite projects)

**Test Suites:**

##### validateAddress() Tests
- [ ] Valid address with 0x prefix
- [ ] Valid address without 0x prefix
- [ ] Invalid: too short
- [ ] Invalid: too long
- [ ] Invalid: non-hex characters
- [ ] Edge case: empty string
- [ ] Edge case: null/undefined

##### validateAmount() Tests
- [ ] Valid positive number
- [ ] Invalid: negative number
- [ ] Invalid: zero
- [ ] Invalid: non-numeric string
- [ ] Invalid: exceeds balance
- [ ] Invalid: exceeds max supply
- [ ] Edge case: very large number
- [ ] Edge case: very small decimal

##### validateGasLimit() Tests
- [ ] Valid gas limit (e.g., 21000)
- [ ] Invalid: too low (< 21000)
- [ ] Invalid: too high (> 10M)
- [ ] Invalid: non-integer
- [ ] Invalid: negative
- [ ] Edge case: exactly 21000
- [ ] Edge case: exactly 10M

##### validatePrivateKey() Tests
- [ ] Valid 64-char hex with 0x
- [ ] Valid 64-char hex without 0x
- [ ] Invalid: too short
- [ ] Invalid: too long
- [ ] Invalid: non-hex characters
- [ ] Edge case: empty string

##### validateMnemonic() Tests
- [ ] Valid 12-word mnemonic
- [ ] Valid 24-word mnemonic
- [ ] Invalid: 11 words
- [ ] Invalid: 13 words
- [ ] Invalid: 25 words
- [ ] Edge case: extra whitespace
- [ ] Edge case: tabs/newlines

**Test Template:**
```typescript
import { describe, it, expect } from 'vitest';
import { validateAddress, validateAmount, /* ... */ } from './validation';

describe('validateAddress', () => {
  it('should accept valid address with 0x prefix', () => {
    const result = validateAddress('0x' + '1'.repeat(40));
    expect(result.isValid).toBe(true);
    expect(result.error).toBeUndefined();
  });

  it('should reject address that is too short', () => {
    const result = validateAddress('0x1234');
    expect(result.isValid).toBe(false);
    expect(result.error).toContain('40 hexadecimal characters');
  });

  // ... more tests
});
```

**Dependencies:** Task 2.1

**Estimated Time:** 2 hours

**Acceptance:**
- [ ] All validation functions have test coverage
- [ ] Tests cover happy path and error cases
- [ ] Tests cover edge cases
- [ ] All tests pass
- [ ] Test coverage > 90% for validation.ts

**Run Tests:**
```bash
cd gui/citrate-core
npm run test
```

---

#### Task 5.2: Manual Testing of All Modified Components (2 hours) ⏳

**Testing Approach:** Systematic manual testing of all Sprint 1 changes

##### FirstTimeSetup Component Testing (30 min)
- [ ] **Test 1:** Create wallet with strong password
  - Enter strong password (12+ chars, mixed case, symbols)
  - Verify strength indicator shows "Strong"
  - Verify confirm password matches
  - Complete setup successfully
  - Verify password stored in keyring (check OS keyring manager)

- [ ] **Test 2:** Create wallet with medium password
  - Enter medium password (8+ chars, mixed case)
  - Verify strength indicator shows "Good"
  - Complete setup

- [ ] **Test 3:** Test password mismatch
  - Enter password
  - Enter different confirm password
  - Verify error message displayed
  - Verify cannot proceed

- [ ] **Test 4:** Test show/hide password toggle
  - Enter password
  - Click show/hide icon
  - Verify password visibility toggles
  - Verify icon changes (Eye ↔ EyeOff)

- [ ] **Test 5:** Test mnemonic confirmation
  - View generated mnemonic
  - Copy to clipboard
  - Verify clipboard contains mnemonic
  - Check "I have saved my recovery phrase"
  - Complete setup

##### Wallet Component Testing (45 min)
- [ ] **Test 6:** Send transaction with valid inputs
  - Enter valid recipient address
  - Enter valid amount (within balance)
  - Verify no error messages
  - Verify send button enabled
  - Send transaction successfully

- [ ] **Test 7:** Send transaction with invalid address
  - Enter invalid address (too short)
  - Verify error message displayed
  - Verify red border on field
  - Verify send button disabled

- [ ] **Test 8:** Send transaction with invalid amount
  - Enter negative amount
  - Verify error message
  - Enter zero amount
  - Verify error message
  - Enter amount exceeding balance
  - Verify error message

- [ ] **Test 9:** Import account from mnemonic
  - Enter valid 12-word mnemonic
  - Verify no errors
  - Import successfully
  - Test invalid mnemonic (11 words)
  - Verify error message

- [ ] **Test 10:** Import account from private key
  - Enter valid 64-char hex private key
  - Verify no errors
  - Import successfully
  - Test invalid private key (too short)
  - Verify error message

##### Settings Component Testing (15 min)
- [ ] **Test 11:** Add bootnode with valid format
  - Enter valid multiaddr bootnode
  - Verify no errors
  - Save settings successfully

- [ ] **Test 12:** Add bootnode with invalid format
  - Enter invalid bootnode
  - Verify error message displayed
  - Verify save button disabled

##### Dashboard Component Testing (15 min)
- [ ] **Test 13:** Dashboard loading skeleton
  - Reload dashboard
  - Verify skeleton shows immediately
  - Verify smooth transition to real data
  - Verify no layout shift

- [ ] **Test 14:** Dashboard real data display
  - Verify node status displays correctly
  - Verify block height updates
  - Verify stats are accurate

##### DAG Visualization Testing (15 min)
- [ ] **Test 15:** DAG table loading
  - Navigate to DAG tab
  - Verify table skeleton shows
  - Verify table loads with real blocks
  - Verify columns match skeleton structure

##### Models Component Testing (10 min)
- [ ] **Test 16:** Models loading
  - Navigate to Models tab
  - Verify card skeletons show
  - Verify grid layout maintained

##### Error Boundary Testing (15 min)
- [ ] **Test 17:** Trigger error in component
  - Add temporary error-throwing button
  - Click button
  - Verify error boundary catches error
  - Verify fallback UI displays
  - Click reload button
  - Verify app recovers

- [ ] **Test 18:** Normal operation (no errors)
  - Navigate through all tabs
  - Verify no unexpected errors
  - Verify error boundary doesn't interfere

**Dependencies:** All implementation tasks (Tasks 1.1-4.5)

**Estimated Time:** 2 hours

**Acceptance:**
- [ ] All 18 manual test cases executed
- [ ] All tests pass
- [ ] Issues logged for any failures
- [ ] Critical bugs fixed immediately
- [ ] Minor bugs added to backlog

---

#### Task 5.3: Fix Bugs Found During Testing (1.5 hours) ⏳

**Bug Tracking:**

**Critical Bugs (Must Fix Before Sprint Close):**
- [ ] Bug 1: _____________________ (if found)
- [ ] Bug 2: _____________________ (if found)

**High Priority Bugs:**
- [ ] Bug 3: _____________________ (if found)
- [ ] Bug 4: _____________________ (if found)

**Low Priority Bugs (Add to Backlog):**
- [ ] Bug 5: _____________________ (if found)

**Bug Fix Process:**
1. Reproduce bug
2. Identify root cause
3. Implement fix
4. Test fix
5. Verify no regressions
6. Update tests if needed

**Dependencies:** Task 5.2

**Estimated Time:** 1.5 hours (buffer for unexpected issues)

**Acceptance:**
- [ ] All critical bugs fixed
- [ ] All high priority bugs fixed or documented
- [ ] Low priority bugs added to Sprint 2 backlog
- [ ] Regression testing completed

---

#### Task 5.4: Update Documentation (0.5 hours) ⏳

**Documentation to Update:**

##### 1. Implementation Log
**File:** `docs/sprints/sprint-1/05_IMPLEMENTATION_LOG.md`
- [ ] Fill in completion dates for all tasks
- [ ] Document any challenges faced
- [ ] Document solutions implemented
- [ ] Add notes for future sprints

##### 2. Sprint Overview
**File:** `docs/sprints/sprint-1/00_SPRINT_OVERVIEW.md`
- [ ] Update sprint status to "✅ Completed"
- [ ] Check off all completed tasks
- [ ] Update success criteria checkboxes
- [ ] Add final sprint metrics (actual vs. planned)

##### 3. User Stories
**File:** `docs/sprints/sprint-1/01_USER_STORIES.md`
- [ ] Mark all user stories as "✅ Done"
- [ ] Verify all acceptance criteria met

##### 4. GUI Audit Document
**File:** `docs/GUI_AUDIT_2026_Q1.md`
- [ ] Update component completion status
- [ ] Mark Sprint 1 as "✅ Completed"
- [ ] Update overall completion percentage

##### 5. README (if needed)
**File:** `gui/citrate-core/README.md`
- [ ] Update any feature descriptions
- [ ] Add notes about new validation features
- [ ] Update security section (no hardcoded passwords)

**Dependencies:** Tasks 5.1-5.3

**Estimated Time:** 0.5 hours

**Acceptance:**
- [ ] All documentation updated
- [ ] Implementation log complete
- [ ] Sprint marked as completed
- [ ] No outdated information

---

#### Task 5.5: Sprint Review and Retrospective (1 hour) ⏳

**Sprint Review Checklist:**

##### Demo All Completed User Stories
- [ ] **Story 1:** Demonstrate secure password management
  - Show password input with strength indicator
  - Show password confirmation
  - Show successful wallet creation
  - Verify password stored in OS keyring

- [ ] **Story 2:** Demonstrate input validation
  - Show address validation (valid and invalid)
  - Show amount validation
  - Show gas limit validation
  - Show private key validation
  - Show mnemonic validation
  - Show clear error messages

- [ ] **Story 3:** Demonstrate error boundaries
  - Show error boundary catching an error
  - Show fallback UI
  - Show reload functionality
  - Show normal operation (no errors)

- [ ] **Story 4:** Demonstrate loading skeletons
  - Show Dashboard skeleton → real data
  - Show Wallet skeleton → real data
  - Show DAG skeleton → real data
  - Show smooth transitions

##### Review Acceptance Criteria Completion
- [ ] Story 1: All 6 acceptance criteria met
- [ ] Story 2: All 8 acceptance criteria met
- [ ] Story 3: All 5 acceptance criteria met
- [ ] Story 4: All 6 acceptance criteria met

##### Sprint Metrics Review
**Planned vs. Actual:**
- [ ] Story Points: 13 planned / ____ actual
- [ ] Duration: 5 days planned / ____ actual
- [ ] Hours: 30 hours planned / ____ actual
- [ ] Velocity: 13 points/week planned / ____ actual

##### Retrospective: What Went Well
- [ ] ___________________________
- [ ] ___________________________
- [ ] ___________________________

##### Retrospective: What Could Be Improved
- [ ] ___________________________
- [ ] ___________________________
- [ ] ___________________________

##### Retrospective: Action Items for Sprint 2
- [ ] ___________________________
- [ ] ___________________________
- [ ] ___________________________

##### Carry Over to Sprint 2 (if any)
- [ ] ___________________________
- [ ] ___________________________

**Dependencies:** All other tasks

**Estimated Time:** 1 hour

**Acceptance:**
- [ ] All user stories demoed
- [ ] Acceptance criteria verified
- [ ] Metrics documented
- [ ] Retrospective completed
- [ ] Action items identified
- [ ] Sprint 1 officially closed

---

## Final Sprint 1 Checklist

### Code Quality
- [ ] All code follows TypeScript best practices
- [ ] No console.log statements (except intentional logging)
- [ ] No commented-out code
- [ ] All imports organized
- [ ] No unused variables or imports
- [ ] Consistent code formatting (Prettier)
- [ ] All ESLint warnings resolved

### Security
- [ ] No hardcoded passwords anywhere
- [ ] No hardcoded private keys
- [ ] No sensitive data in console logs
- [ ] Password stored securely in OS keyring
- [ ] Input validation prevents injection attacks
- [ ] Error messages don't leak sensitive info

### Testing
- [ ] All unit tests pass
- [ ] All manual test cases pass
- [ ] No console errors in browser
- [ ] No TypeScript compilation errors
- [ ] Application builds successfully
- [ ] Application runs without crashes

### Documentation
- [ ] All sprint docs updated
- [ ] Implementation log complete
- [ ] Code comments added where needed
- [ ] README updated (if applicable)

### User Experience
- [ ] All loading states have skeletons
- [ ] All forms have validation
- [ ] All error states have clear messages
- [ ] All buttons have proper disabled states
- [ ] Orange theme consistently applied
- [ ] Keyboard navigation works
- [ ] Accessibility considerations met

### Definition of Done (from Sprint Overview)
- [ ] Code is written and follows TypeScript/React best practices
- [ ] All acceptance criteria are met
- [ ] Unit tests written and passing
- [ ] Manual testing completed
- [ ] No console errors or warnings
- [ ] Code reviewed (self-review or pair)
- [ ] Documentation updated
- [ ] Ready for Sprint 2

---

## Time Tracking Summary

| Day | Tasks | Estimated Hours | Actual Hours | Notes |
|-----|-------|----------------|--------------|-------|
| Day 1 | Password Security | 6.0 | | |
| Day 2 | Input Validation Part 1 | 6.0 | | |
| Day 3 | Input Validation Part 2 + Error Boundaries | 6.0 | | |
| Day 4 | Loading Skeletons | 6.0 | | |
| Day 5 | Testing & Review | 7.0 | | |
| **Total** | | **31.0** | | |

---

## Risk Management

### Identified Risks During Sprint
- [ ] Risk 1: _______________________ (if encountered)
- [ ] Risk 2: _______________________ (if encountered)

### Mitigation Actions Taken
- [ ] Action 1: ______________________
- [ ] Action 2: ______________________

---

## Next Steps (Transition to Sprint 2)

**Sprint 2 Preview:**
- IPFS integration for model storage
- Marketplace contract deployment
- Model upload/download functionality
- Payment processing

**Preparation for Sprint 2:**
- [ ] Review Sprint 2 objectives
- [ ] Ensure IPFS node is running
- [ ] Review marketplace smart contracts
- [ ] Set up test accounts with funds

---

**Sprint 1 Status:** ⏳ Pending → 🚧 In Progress → ✅ Completed

**Last Updated:** [To be filled during sprint]

**Total Tasks:** 23 tasks across 5 days

**Completion Rate:** ___% (to be calculated at end)
