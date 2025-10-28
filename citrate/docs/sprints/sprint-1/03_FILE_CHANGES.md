# Sprint 1: File Changes Tracking

**Sprint Goal:** Fix critical security issues and establish quality baseline

**Total Files to Modify:** 11 files
**Total Files to Create:** 11 new files
**Total Files:** 22 files

---

## File Modification Status

### ✅ Completed | 🚧 In Progress | ⏳ Pending

---

## 1. Files to Modify

### 1.1 FirstTimeSetup.tsx ⏳
**Path:** `gui/citrate-core/src/components/FirstTimeSetup.tsx`

**Purpose:** Remove hardcoded password, add password input UI with strength indicator

**Current State:**
- Hardcoded password on line 52: `password: 'user_secure_password'`
- No password input fields
- No password validation

**Changes Required:**

#### State Additions
```typescript
// Add password management state
const [password, setPassword] = useState('');
const [confirmPassword, setConfirmPassword] = useState('');
const [showPassword, setShowPassword] = useState(false);
const [passwordError, setPasswordError] = useState('');
const [passwordStrength, setPasswordStrength] = useState(0);
```

#### New Functions
```typescript
// Password strength calculator
const calculatePasswordStrength = (pwd: string): number => {
  let strength = 0;
  if (pwd.length >= 8) strength++;
  if (pwd.length >= 12) strength++;
  if (/[a-z]/.test(pwd) && /[A-Z]/.test(pwd)) strength++;
  if (/\d/.test(pwd)) strength++;
  if (/[^a-zA-Z0-9]/.test(pwd)) strength++;
  return Math.min(strength, 4);
};

// Password validation
const validatePasswords = (): boolean => {
  if (password.length < 8) {
    setPasswordError('Password must be at least 8 characters');
    return false;
  }
  if (password !== confirmPassword) {
    setPasswordError('Passwords do not match');
    return false;
  }
  setPasswordError('');
  return true;
};
```

#### UI Changes (Password Step)
```typescript
{/* Password input field */}
<div className="mb-4">
  <label className="block text-sm font-medium mb-2">
    Choose a Strong Password
  </label>
  <div className="relative">
    <input
      type={showPassword ? 'text' : 'password'}
      value={password}
      onChange={(e) => {
        setPassword(e.target.value);
        setPasswordStrength(calculatePasswordStrength(e.target.value));
      }}
      className="w-full px-4 py-2 border rounded"
      placeholder="Enter password"
    />
    <button
      type="button"
      onClick={() => setShowPassword(!showPassword)}
      className="absolute right-2 top-2"
    >
      {showPassword ? <EyeOff size={20} /> : <Eye size={20} />}
    </button>
  </div>
</div>

{/* Confirm password field */}
<div className="mb-4">
  <label className="block text-sm font-medium mb-2">
    Confirm Password
  </label>
  <input
    type={showPassword ? 'text' : 'password'}
    value={confirmPassword}
    onChange={(e) => setConfirmPassword(e.target.value)}
    className="w-full px-4 py-2 border rounded"
    placeholder="Re-enter password"
  />
</div>

{/* Password strength indicator */}
<div className="mb-4">
  <div className="flex items-center justify-between mb-1">
    <span className="text-sm">Password Strength</span>
    <span className={`text-sm font-medium ${
      passwordStrength === 0 ? 'text-red-600' :
      passwordStrength <= 2 ? 'text-yellow-600' :
      passwordStrength === 3 ? 'text-orange-500' :
      'text-green-600'
    }`}>
      {passwordStrength === 0 ? 'Too Weak' :
       passwordStrength <= 2 ? 'Weak' :
       passwordStrength === 3 ? 'Good' :
       'Strong'}
    </span>
  </div>
  <div className="h-2 bg-gray-200 rounded">
    <div
      className={`h-full rounded transition-all ${
        passwordStrength === 0 ? 'bg-red-500 w-1/4' :
        passwordStrength <= 2 ? 'bg-yellow-500 w-2/4' :
        passwordStrength === 3 ? 'bg-orange-500 w-3/4' :
        'bg-green-500 w-full'
      }`}
    />
  </div>
</div>

{/* Password requirements */}
<div className="text-xs text-gray-600 mb-4">
  <p>Password should contain:</p>
  <ul className="list-disc list-inside">
    <li className={password.length >= 8 ? 'text-green-600' : ''}>
      At least 8 characters
    </li>
    <li className={/[A-Z]/.test(password) && /[a-z]/.test(password) ? 'text-green-600' : ''}>
      Upper and lowercase letters
    </li>
    <li className={/\d/.test(password) ? 'text-green-600' : ''}>
      At least one number
    </li>
    <li className={/[^a-zA-Z0-9]/.test(password) ? 'text-green-600' : ''}>
      At least one special character
    </li>
  </ul>
</div>

{/* Error message */}
{passwordError && (
  <div className="text-red-600 text-sm mb-4">{passwordError}</div>
)}
```

#### Invoke Call Update
```typescript
// Line 52 - BEFORE
const result = await invoke<FirstTimeSetupResult>('perform_first_time_setup', {
  password: 'user_secure_password'  // ❌ HARDCODED
});

// Line 52 - AFTER
if (!validatePasswords()) {
  return;
}

const result = await invoke<FirstTimeSetupResult>('perform_first_time_setup', {
  password: password  // ✅ User-provided
});
```

#### Mnemonic Confirmation Enhancements
```typescript
{/* Add checkbox for mnemonic confirmation */}
const [confirmedMnemonic, setConfirmedMnemonic] = useState(false);

{/* In mnemonic display step */}
<div className="mb-4">
  <label className="flex items-center">
    <input
      type="checkbox"
      checked={confirmedMnemonic}
      onChange={(e) => setConfirmedMnemonic(e.target.checked)}
      className="mr-2"
    />
    <span className="text-sm">
      I have securely saved my recovery phrase
    </span>
  </label>
</div>

{/* Update next button to require confirmation */}
<button
  disabled={!confirmedMnemonic}
  style={{
    backgroundColor: confirmedMnemonic ? '#ffa500' : '#9ca3af',
    cursor: confirmedMnemonic ? 'pointer' : 'not-allowed'
  }}
>
  Continue
</button>
```

**Lines Modified:** ~50-200 (password step additions)

**Estimated Changes:** 150+ lines added/modified

**Story:** Story 1 (Secure Password Management)

**Status:** ⏳ Pending

---

### 1.2 Wallet.tsx ⏳
**Path:** `gui/citrate-core/src/components/Wallet.tsx`

**Purpose:** Add input validation to all wallet forms

**Changes Required:**

#### Imports
```typescript
import {
  validateAddress,
  validateAmount,
  validateGasLimit,
  validatePrivateKey,
  validateMnemonic,
  ValidationResult
} from '../utils/validation';
```

#### State Additions
```typescript
// Send transaction form validation
const [recipientError, setRecipientError] = useState('');
const [amountError, setAmountError] = useState('');
const [gasLimitError, setGasLimitError] = useState('');

// Import account validation
const [mnemonicError, setMnemonicError] = useState('');
const [privateKeyError, setPrivateKeyError] = useState('');
```

#### Send Transaction Form Updates
```typescript
{/* Recipient address field */}
<div className="mb-4">
  <label>Recipient Address</label>
  <input
    type="text"
    value={recipient}
    onChange={(e) => {
      setRecipient(e.target.value);
      const validation = validateAddress(e.target.value);
      setRecipientError(validation.error || '');
    }}
    className={`w-full px-4 py-2 border rounded ${
      recipientError ? 'border-red-500' : ''
    }`}
    placeholder="0x..."
  />
  {recipientError && (
    <p className="text-red-600 text-sm mt-1">{recipientError}</p>
  )}
</div>

{/* Amount field */}
<div className="mb-4">
  <label>Amount (CITRATE)</label>
  <input
    type="text"
    value={amount}
    onChange={(e) => {
      setAmount(e.target.value);
      const validation = validateAmount(e.target.value, selectedAccount?.balance);
      setAmountError(validation.error || '');
    }}
    className={`w-full px-4 py-2 border rounded ${
      amountError ? 'border-red-500' : ''
    }`}
    placeholder="0.0"
  />
  {amountError && (
    <p className="text-red-600 text-sm mt-1">{amountError}</p>
  )}
</div>

{/* Gas limit field (if exposed) */}
<div className="mb-4">
  <label>Gas Limit</label>
  <input
    type="text"
    value={gasLimit}
    onChange={(e) => {
      setGasLimit(e.target.value);
      const validation = validateGasLimit(e.target.value);
      setGasLimitError(validation.error || '');
    }}
    className={`w-full px-4 py-2 border rounded ${
      gasLimitError ? 'border-red-500' : ''
    }`}
    placeholder="21000"
  />
  {gasLimitError && (
    <p className="text-red-600 text-sm mt-1">{gasLimitError}</p>
  )}
</div>

{/* Send button with validation */}
<button
  onClick={handleSend}
  disabled={!!recipientError || !!amountError || !!gasLimitError || !recipient || !amount}
  style={{
    backgroundColor: (!recipientError && !amountError && !gasLimitError && recipient && amount) ? '#ffa500' : '#9ca3af',
    cursor: (!recipientError && !amountError && !gasLimitError && recipient && amount) ? 'pointer' : 'not-allowed',
    opacity: (!recipientError && !amountError && !gasLimitError && recipient && amount) ? 1 : 0.5
  }}
>
  Send Transaction
</button>
```

#### Import from Mnemonic Updates
```typescript
{/* Mnemonic input */}
<div className="mb-4">
  <label>Recovery Phrase (12 or 24 words)</label>
  <textarea
    value={importMnemonic}
    onChange={(e) => {
      setImportMnemonic(e.target.value);
      const validation = validateMnemonic(e.target.value);
      setMnemonicError(validation.error || '');
    }}
    className={`w-full px-4 py-2 border rounded ${
      mnemonicError ? 'border-red-500' : ''
    }`}
    rows={3}
    placeholder="word1 word2 word3 ..."
  />
  {mnemonicError && (
    <p className="text-red-600 text-sm mt-1">{mnemonicError}</p>
  )}
  <p className="text-sm text-gray-600 mt-1">
    {importMnemonic.trim().split(/\s+/).filter(w => w).length} / {12 or 24} words
  </p>
</div>

<button
  onClick={handleImportFromMnemonic}
  disabled={!!mnemonicError || !importMnemonic}
  style={{
    backgroundColor: (!mnemonicError && importMnemonic) ? '#ffa500' : '#9ca3af',
    cursor: (!mnemonicError && importMnemonic) ? 'pointer' : 'not-allowed'
  }}
>
  Import Account
</button>
```

#### Import from Private Key Updates
```typescript
{/* Private key input */}
<div className="mb-4">
  <label>Private Key</label>
  <div className="relative">
    <input
      type={showPrivateKey ? 'text' : 'password'}
      value={importPrivateKey}
      onChange={(e) => {
        setImportPrivateKey(e.target.value);
        const validation = validatePrivateKey(e.target.value);
        setPrivateKeyError(validation.error || '');
      }}
      className={`w-full px-4 py-2 border rounded pr-10 ${
        privateKeyError ? 'border-red-500' : ''
      }`}
      placeholder="0x..."
    />
    <button
      type="button"
      onClick={() => setShowPrivateKey(!showPrivateKey)}
      className="absolute right-2 top-2"
    >
      {showPrivateKey ? <EyeOff size={20} /> : <Eye size={20} />}
    </button>
  </div>
  {privateKeyError && (
    <p className="text-red-600 text-sm mt-1">{privateKeyError}</p>
  )}
</div>

<button
  onClick={handleImportFromPrivateKey}
  disabled={!!privateKeyError || !importPrivateKey}
  style={{
    backgroundColor: (!privateKeyError && importPrivateKey) ? '#ffa500' : '#9ca3af',
    cursor: (!privateKeyError && importPrivateKey) ? 'pointer' : 'not-allowed'
  }}
>
  Import Account
</button>
```

**Lines Modified:** ~100-300

**Estimated Changes:** 100+ lines added/modified

**Story:** Story 2 (Input Validation)

**Status:** ⏳ Pending

---

### 1.3 Settings.tsx ⏳
**Path:** `gui/citrate-core/src/components/Settings.tsx`

**Purpose:** Add validation for bootnode and peer inputs

**Changes Required:**

#### Imports
```typescript
import { validateBootnode, ValidationResult } from '../utils/validation';
```

#### State Additions
```typescript
const [bootnodeError, setBootnodeError] = useState('');
```

#### Bootnode Input Updates
```typescript
{/* Bootnode input */}
<div className="mb-4">
  <label>Add Bootnode</label>
  <input
    type="text"
    value={newBootnode}
    onChange={(e) => {
      setNewBootnode(e.target.value);
      const validation = validateBootnode(e.target.value);
      setBootnodeError(validation.error || '');
    }}
    className={`w-full px-4 py-2 border rounded ${
      bootnodeError ? 'border-red-500' : ''
    }`}
    placeholder="/ip4/127.0.0.1/tcp/9000/p2p/12D3KooW..."
  />
  {bootnodeError && (
    <p className="text-red-600 text-sm mt-1">{bootnodeError}</p>
  )}
  <p className="text-xs text-gray-600 mt-1">
    Format: /ip4/IP/tcp/PORT/p2p/PEER_ID
  </p>
</div>

<button
  onClick={handleAddBootnode}
  disabled={!!bootnodeError || !newBootnode}
  style={{
    backgroundColor: (!bootnodeError && newBootnode) ? '#ffa500' : '#9ca3af',
    cursor: (!bootnodeError && newBootnode) ? 'pointer' : 'not-allowed'
  }}
>
  Add Bootnode
</button>
```

**Lines Modified:** ~50-100

**Estimated Changes:** 30+ lines added/modified

**Story:** Story 2 (Input Validation)

**Status:** ⏳ Pending

---

### 1.4 App.tsx ⏳
**Path:** `gui/citrate-core/src/App.tsx`

**Purpose:** Wrap app with ErrorBoundary component

**Changes Required:**

#### Imports
```typescript
import { ErrorBoundary } from './components/ErrorBoundary';
```

#### Component Wrapper
```typescript
// BEFORE
function App() {
  return (
    <div className="App">
      {/* Navigation and content */}
    </div>
  );
}

// AFTER
function App() {
  return (
    <ErrorBoundary>
      <div className="App">
        {/* Navigation and content */}
      </div>
    </ErrorBoundary>
  );
}
```

**Lines Modified:** ~5-10

**Estimated Changes:** 3 lines added

**Story:** Story 3 (Error Boundaries)

**Status:** ⏳ Pending

---

### 1.5 Dashboard.tsx ⏳
**Path:** `gui/citrate-core/src/components/Dashboard.tsx`

**Purpose:** Integrate loading skeleton

**Changes Required:**

#### Imports
```typescript
import { DashboardSkeleton } from './skeletons/DashboardSkeleton';
```

#### State Addition
```typescript
const [isLoading, setIsLoading] = useState(true);
```

#### Loading Logic
```typescript
useEffect(() => {
  const loadData = async () => {
    setIsLoading(true);
    try {
      // Existing data loading logic
      await Promise.all([
        // ... existing data fetches
      ]);
    } finally {
      setIsLoading(false);
    }
  };
  loadData();
}, []);
```

#### Render Logic
```typescript
if (isLoading) {
  return <DashboardSkeleton />;
}

return (
  <div className="dashboard">
    {/* Existing content */}
  </div>
);
```

**Lines Modified:** ~20-30

**Estimated Changes:** 15+ lines added

**Story:** Story 4 (Loading Skeletons)

**Status:** ⏳ Pending

---

### 1.6 Wallet.tsx (Skeletons) ⏳
**Path:** `gui/citrate-core/src/components/Wallet.tsx`

**Purpose:** Integrate wallet loading skeleton

**Changes Required:**

#### Imports
```typescript
import { WalletSkeleton } from './skeletons/WalletSkeleton';
```

#### State Additions
```typescript
const [isLoadingAccounts, setIsLoadingAccounts] = useState(true);
const [isLoadingBalance, setIsLoadingBalance] = useState(true);
```

#### Render Logic
```typescript
if (isLoadingAccounts) {
  return <WalletSkeleton />;
}

return (
  <div className="wallet">
    {/* Existing content */}
  </div>
);
```

**Lines Modified:** ~15-20

**Estimated Changes:** 10+ lines added

**Story:** Story 4 (Loading Skeletons)

**Status:** ⏳ Pending

---

### 1.7 DAGVisualization.tsx ⏳
**Path:** `gui/citrate-core/src/components/DAGVisualization.tsx`

**Purpose:** Integrate table loading skeleton

**Changes Required:**

#### Imports
```typescript
import { TableSkeleton } from './skeletons/TableSkeleton';
```

#### State Addition
```typescript
const [isLoadingBlocks, setIsLoadingBlocks] = useState(true);
```

#### Render Logic
```typescript
if (isLoadingBlocks) {
  return <TableSkeleton rows={10} columns={6} />;
}

return (
  <div className="dag-visualization">
    {/* Existing table */}
  </div>
);
```

**Lines Modified:** ~10-15

**Estimated Changes:** 8+ lines added

**Story:** Story 4 (Loading Skeletons)

**Status:** ⏳ Pending

---

### 1.8 Models.tsx ⏳
**Path:** `gui/citrate-core/src/components/Models.tsx`

**Purpose:** Integrate card loading skeletons

**Changes Required:**

#### Imports
```typescript
import { CardSkeleton } from './skeletons/CardSkeleton';
```

#### State Addition
```typescript
const [isLoadingModels, setIsLoadingModels] = useState(true);
```

#### Render Logic
```typescript
if (isLoadingModels) {
  return (
    <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
      {[...Array(6)].map((_, i) => (
        <CardSkeleton key={i} lines={4} />
      ))}
    </div>
  );
}

return (
  <div className="models">
    {/* Existing content */}
  </div>
);
```

**Lines Modified:** ~10-15

**Estimated Changes:** 10+ lines added

**Story:** Story 4 (Loading Skeletons)

**Status:** ⏳ Pending

---

### 1.9 mod.rs (Wallet Backend) ⏳
**Path:** `gui/citrate-core/src-tauri/src/wallet/mod.rs`

**Purpose:** Ensure password parameter is properly handled

**Current State:** Already accepts password parameter (needs verification)

**Changes Required:**
- Verify password parameter type
- Add password strength validation (minimum 8 characters)
- Ensure secure keyring storage
- Add error handling

**Example:**
```rust
#[tauri::command]
pub async fn perform_first_time_setup(password: String) -> Result<FirstTimeSetupResult, String> {
    // Validate password
    if password.len() < 8 {
        return Err("Password must be at least 8 characters".to_string());
    }

    // Store password securely in OS keyring
    // ... existing logic
}
```

**Lines Modified:** ~5-10

**Estimated Changes:** Minor validation additions

**Story:** Story 1 (Secure Password Management)

**Status:** ⏳ Pending

---

### 1.10 package.json ⏳
**Path:** `gui/citrate-core/package.json`

**Purpose:** Add test script and dependencies (if needed)

**Changes Required:**

#### Add vitest if not present
```json
{
  "devDependencies": {
    "vitest": "^1.0.0",
    "@testing-library/react": "^14.0.0",
    "@testing-library/user-event": "^14.0.0"
  },
  "scripts": {
    "test": "vitest",
    "test:ui": "vitest --ui",
    "test:coverage": "vitest --coverage"
  }
}
```

**Lines Modified:** ~5-10

**Story:** Story 2 (Input Validation - Testing)

**Status:** ⏳ Pending

---

### 1.11 tsconfig.json ⏳
**Path:** `gui/citrate-core/tsconfig.json`

**Purpose:** Ensure test files are included in TypeScript compilation

**Changes Required:**

#### Update include array
```json
{
  "include": [
    "src",
    "src/**/*.test.ts",
    "src/**/*.test.tsx"
  ]
}
```

**Lines Modified:** ~2-3

**Story:** Story 2 (Input Validation - Testing)

**Status:** ⏳ Pending

---

## 2. New Files to Create

### 2.1 validation.ts ⏳
**Path:** `gui/citrate-core/src/utils/validation.ts` (NEW FILE)

**Purpose:** Centralized validation utility functions

**Contents:**
- ValidationResult interface
- validateAddress()
- validateAmount()
- validateGasLimit()
- validatePrivateKey()
- validateMnemonic()
- validateBootnode()
- validateIPv4()
- validatePort()

**Lines of Code:** ~200 lines

**Story:** Story 2 (Input Validation)

**Status:** ⏳ Pending

**Full Implementation:** See 01_USER_STORIES.md for complete code

---

### 2.2 validation.test.ts ⏳
**Path:** `gui/citrate-core/src/utils/validation.test.ts` (NEW FILE)

**Purpose:** Unit tests for validation functions

**Contents:**
- Test suites for all validation functions
- Happy path tests
- Error case tests
- Edge case tests

**Lines of Code:** ~300 lines

**Story:** Story 2 (Input Validation)

**Status:** ⏳ Pending

---

### 2.3 ErrorBoundary.tsx ⏳
**Path:** `gui/citrate-core/src/components/ErrorBoundary.tsx` (NEW FILE)

**Purpose:** React error boundary component

**Contents:**
- Class component with error handling
- componentDidCatch implementation
- getDerivedStateFromError implementation
- Fallback UI
- Reload functionality

**Lines of Code:** ~150 lines

**Story:** Story 3 (Error Boundaries)

**Status:** ⏳ Pending

**Full Implementation:** See 01_USER_STORIES.md for complete code

---

### 2.4 Skeleton.tsx ⏳
**Path:** `gui/citrate-core/src/components/skeletons/Skeleton.tsx` (NEW FILE)

**Purpose:** Base skeleton component with shimmer animation

**Contents:**
- Reusable skeleton primitive
- Shimmer animation CSS
- Width/height props

**Lines of Code:** ~50 lines

**Story:** Story 4 (Loading Skeletons)

**Status:** ⏳ Pending

---

### 2.5 DashboardSkeleton.tsx ⏳
**Path:** `gui/citrate-core/src/components/skeletons/DashboardSkeleton.tsx` (NEW FILE)

**Purpose:** Dashboard loading skeleton

**Contents:**
- Header skeleton
- Stats grid skeleton
- Recent blocks table skeleton
- Network status skeleton

**Lines of Code:** ~100 lines

**Story:** Story 4 (Loading Skeletons)

**Status:** ⏳ Pending

---

### 2.6 WalletSkeleton.tsx ⏳
**Path:** `gui/citrate-core/src/components/skeletons/WalletSkeleton.tsx` (NEW FILE)

**Purpose:** Wallet loading skeleton

**Contents:**
- Account selector skeleton
- Balance display skeleton
- Account list skeleton

**Lines of Code:** ~80 lines

**Story:** Story 4 (Loading Skeletons)

**Status:** ⏳ Pending

---

### 2.7 TableSkeleton.tsx ⏳
**Path:** `gui/citrate-core/src/components/skeletons/TableSkeleton.tsx` (NEW FILE)

**Purpose:** Reusable table skeleton

**Contents:**
- Configurable rows/columns
- Header row skeleton
- Data rows skeleton

**Lines of Code:** ~60 lines

**Story:** Story 4 (Loading Skeletons)

**Status:** ⏳ Pending

---

### 2.8 CardSkeleton.tsx ⏳
**Path:** `gui/citrate-core/src/components/skeletons/CardSkeleton.tsx` (NEW FILE)

**Purpose:** Reusable card skeleton

**Contents:**
- Simple card skeleton
- Configurable line count

**Lines of Code:** ~40 lines

**Story:** Story 4 (Loading Skeletons)

**Status:** ⏳ Pending

---

### 2.9 skeletons/index.ts ⏳
**Path:** `gui/citrate-core/src/components/skeletons/index.ts` (NEW FILE)

**Purpose:** Barrel export for all skeleton components

**Contents:**
```typescript
export { Skeleton } from './Skeleton';
export { DashboardSkeleton } from './DashboardSkeleton';
export { WalletSkeleton } from './WalletSkeleton';
export { TableSkeleton } from './TableSkeleton';
export { CardSkeleton } from './CardSkeleton';
```

**Lines of Code:** ~5 lines

**Story:** Story 4 (Loading Skeletons)

**Status:** ⏳ Pending

---

### 2.10 vitest.config.ts ⏳
**Path:** `gui/citrate-core/vitest.config.ts` (NEW FILE)

**Purpose:** Vitest configuration for testing

**Contents:**
```typescript
import { defineConfig } from 'vitest/config';
import react from '@vitejs/plugin-react';

export default defineConfig({
  plugins: [react()],
  test: {
    globals: true,
    environment: 'jsdom',
    setupFiles: './src/test/setup.ts',
    coverage: {
      provider: 'v8',
      reporter: ['text', 'json', 'html'],
      exclude: [
        'node_modules/',
        'src/test/',
      ],
    },
  },
});
```

**Lines of Code:** ~20 lines

**Story:** Story 2 (Input Validation - Testing)

**Status:** ⏳ Pending

---

### 2.11 test/setup.ts ⏳
**Path:** `gui/citrate-core/src/test/setup.ts` (NEW FILE)

**Purpose:** Test environment setup

**Contents:**
```typescript
import '@testing-library/jest-dom';
```

**Lines of Code:** ~5 lines

**Story:** Story 2 (Input Validation - Testing)

**Status:** ⏳ Pending

---

## 3. File Change Summary

### By Story

#### Story 1: Secure Password Management
**Files Modified:** 2
- FirstTimeSetup.tsx (major changes)
- wallet/mod.rs (minor validation)

**Files Created:** 0

**Total Files:** 2

---

#### Story 2: Input Validation
**Files Modified:** 3
- Wallet.tsx (validation integration)
- Settings.tsx (bootnode validation)
- package.json (test dependencies)

**Files Created:** 3
- validation.ts (utility functions)
- validation.test.ts (unit tests)
- vitest.config.ts (test config)
- test/setup.ts (test setup)

**Total Files:** 7

---

#### Story 3: Error Boundaries
**Files Modified:** 1
- App.tsx (wrap with ErrorBoundary)

**Files Created:** 1
- ErrorBoundary.tsx (error boundary component)

**Total Files:** 2

---

#### Story 4: Loading Skeletons
**Files Modified:** 4
- Dashboard.tsx (integrate skeleton)
- Wallet.tsx (integrate skeleton)
- DAGVisualization.tsx (integrate skeleton)
- Models.tsx (integrate skeleton)

**Files Created:** 6
- skeletons/Skeleton.tsx (base component)
- skeletons/DashboardSkeleton.tsx
- skeletons/WalletSkeleton.tsx
- skeletons/TableSkeleton.tsx
- skeletons/CardSkeleton.tsx
- skeletons/index.ts (barrel export)

**Total Files:** 10

---

### Grand Total

**Files to Modify:** 11 files
**Files to Create:** 11 files
**Total Files Affected:** 22 files

**Lines of Code:**
- Modified: ~500 lines
- Created: ~1,000 lines
- Total: ~1,500 lines

---

## 4. Merge Conflict Prevention

### Files with Potential Conflicts
- ✅ FirstTimeSetup.tsx - Single developer, low risk
- ✅ Wallet.tsx - Single developer, low risk
- ✅ App.tsx - Single developer, low risk

### Best Practices
- Commit frequently after each task
- Use descriptive commit messages
- Test before committing
- Keep branches short-lived

---

## 5. Backup Strategy

### Before Starting Each Day
```bash
# Create backup branch
git checkout -b sprint-1-day-X-backup

# Return to main sprint branch
git checkout sprint-1
```

### Rollback Strategy
```bash
# If something goes wrong
git checkout sprint-1-day-X-backup
git checkout -b sprint-1-recovery
# Fix issues, then merge back
```

---

## 6. Commit Strategy

### Recommended Commits

#### Day 1 (Password Security)
```bash
git commit -m "feat: add password input fields to FirstTimeSetup"
git commit -m "feat: implement password strength indicator"
git commit -m "feat: remove hardcoded password, use user input"
git commit -m "fix: update backend password validation"
git commit -m "test: verify password creation flow"
```

#### Day 2 (Input Validation Part 1)
```bash
git commit -m "feat: create validation utility functions"
git commit -m "feat: add validation to wallet send form"
git commit -m "feat: add validation to import account forms"
```

#### Day 3 (Input Validation Part 2 + Error Boundaries)
```bash
git commit -m "feat: add mnemonic confirmation to setup"
git commit -m "feat: add bootnode validation to settings"
git commit -m "feat: create ErrorBoundary component"
git commit -m "feat: wrap app with ErrorBoundary"
```

#### Day 4 (Loading Skeletons)
```bash
git commit -m "feat: create base skeleton components"
git commit -m "feat: integrate skeletons into Dashboard"
git commit -m "feat: integrate skeletons into Wallet"
git commit -m "feat: integrate skeletons into DAG and Models"
```

#### Day 5 (Testing & Review)
```bash
git commit -m "test: add validation unit tests"
git commit -m "fix: resolve bugs from manual testing"
git commit -m "docs: update sprint documentation"
git commit -m "chore: sprint 1 complete"
```

---

## 7. File Dependency Graph

```
validation.ts (create first)
  ↓
Wallet.tsx (validation integration)
Settings.tsx (validation integration)
FirstTimeSetup.tsx (password validation)
  ↓
ErrorBoundary.tsx (create)
  ↓
App.tsx (wrap with boundary)
  ↓
Skeleton.tsx (create base)
  ↓
DashboardSkeleton.tsx (depends on Skeleton)
WalletSkeleton.tsx (depends on Skeleton)
TableSkeleton.tsx (depends on Skeleton)
CardSkeleton.tsx (depends on Skeleton)
  ↓
Dashboard.tsx (integrate skeleton)
Wallet.tsx (integrate skeleton)
DAGVisualization.tsx (integrate skeleton)
Models.tsx (integrate skeleton)
```

**Recommended Order:**
1. Create validation.ts
2. Update forms with validation
3. Create ErrorBoundary.tsx
4. Wrap App.tsx
5. Create skeleton components
6. Integrate skeletons
7. Write tests
8. Manual testing
9. Bug fixes

---

**Last Updated:** [To be updated during sprint]

**Status:** ⏳ Ready to Start

**Next Update:** After Day 1 completion
