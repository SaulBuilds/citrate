# Sprint 1: User Stories - Detailed Breakdown

---

## Story 1: Secure Password Management

**Story ID:** S1-01
**Priority:** P0 (Critical - Security Issue)
**Story Points:** 3
**Assignee:** TBD

### User Story
```
As a new user
I want to set my own secure password during wallet setup
So that my funds are protected with a password only I know
```

### Current State (Problem)
**File:** `gui/citrate-core/src/components/FirstTimeSetup.tsx`
**Line:** 52
**Issue:**
```typescript
const result = await invoke<FirstTimeSetupResult>('perform_first_time_setup', {
  password: 'user_secure_password'  // ❌ HARDCODED PASSWORD
});
```

**Security Risk:** Every user's wallet created with the same password. Anyone with access to the source code knows all wallets' passwords.

### Desired State (Solution)
```typescript
const result = await invoke<FirstTimeSetupResult>('perform_first_time_setup', {
  password: userEnteredPassword  // ✅ User-provided password
});
```

---

### Acceptance Criteria

#### AC1: Password Input Fields
- [ ] Password input field added to "Setup" step (between lines 124-156)
- [ ] Password confirmation field added
- [ ] Both fields have `type="password"` attribute
- [ ] "Show/Hide" toggle button for each field (eye icon)
- [ ] Fields are clearly labeled ("Password", "Confirm Password")

#### AC2: Password Validation
- [ ] Minimum 8 characters enforced
- [ ] Error message shown if less than 8 characters
- [ ] Error message shown if passwords don't match
- [ ] "Create Secure Wallet" button disabled until validation passes

#### AC3: Password Strength Indicator
- [ ] Visual indicator shows strength (Weak/Medium/Strong)
- [ ] Red color for weak passwords
- [ ] Orange for medium passwords
- [ ] Green for strong passwords
- [ ] Criteria shown: length, uppercase, lowercase, numbers, symbols

#### AC4: Backend Integration
- [ ] Password passed to `perform_first_time_setup` command
- [ ] No hardcoded password in code
- [ ] Password stored securely in OS keyring (Tauri keyring plugin)

#### AC5: User Experience
- [ ] User can see password while typing (toggle)
- [ ] Clear guidance on password requirements
- [ ] Cannot proceed without meeting requirements
- [ ] Password not logged or exposed in debug output

---

### Technical Tasks

#### Task 1.1: Update FirstTimeSetup.tsx UI
**Estimated Time:** 1.5 hours

**Changes:**
1. Add state for password fields:
```typescript
const [password, setPassword] = useState('');
const [confirmPassword, setConfirmPassword] = useState('');
const [showPassword, setShowPassword] = useState(false);
const [showConfirmPassword, setShowConfirmPassword] = useState(false);
const [passwordError, setPasswordError] = useState('');
```

2. Add password input fields (line ~140):
```typescript
<div className="mb-4">
  <label className="block text-sm font-medium text-gray-700 mb-2">
    Create Password
  </label>
  <div className="relative">
    <input
      type={showPassword ? "text" : "password"}
      value={password}
      onChange={(e) => setPassword(e.target.value)}
      className="w-full border rounded-lg p-3 pr-10"
      placeholder="Minimum 8 characters"
      minLength={8}
      required
    />
    <button
      type="button"
      onClick={() => setShowPassword(!showPassword)}
      className="absolute right-3 top-3 text-gray-500"
    >
      {showPassword ? <EyeOff size={20} /> : <Eye size={20} />}
    </button>
  </div>
</div>

<div className="mb-4">
  <label className="block text-sm font-medium text-gray-700 mb-2">
    Confirm Password
  </label>
  <div className="relative">
    <input
      type={showConfirmPassword ? "text" : "password"}
      value={confirmPassword}
      onChange={(e) => setConfirmPassword(e.target.value)}
      className="w-full border rounded-lg p-3 pr-10"
      placeholder="Re-enter password"
      minLength={8}
      required
    />
    <button
      type="button"
      onClick={() => setShowConfirmPassword(!showConfirmPassword)}
      className="absolute right-3 top-3 text-gray-500"
    >
      {showConfirmPassword ? <EyeOff size={20} /> : <Eye size={20} />}
    </button>
  </div>
  {passwordError && (
    <p className="text-red-600 text-sm mt-1">{passwordError}</p>
  )}
</div>
```

3. Add password strength indicator:
```typescript
<div className="mb-4">
  <div className="flex items-center justify-between text-sm">
    <span className="text-gray-600">Password Strength:</span>
    <span className={`font-medium ${getStrengthColor(passwordStrength)}`}>
      {getStrengthLabel(passwordStrength)}
    </span>
  </div>
  <div className="w-full bg-gray-200 rounded-full h-2 mt-1">
    <div
      className={`h-2 rounded-full transition-all ${getStrengthColor(passwordStrength)}`}
      style={{ width: `${(passwordStrength / 4) * 100}%` }}
    />
  </div>
</div>
```

4. Add strength calculation function:
```typescript
const calculatePasswordStrength = (pwd: string): number => {
  let strength = 0;
  if (pwd.length >= 8) strength++;
  if (pwd.length >= 12) strength++;
  if (/[a-z]/.test(pwd) && /[A-Z]/.test(pwd)) strength++;
  if (/\d/.test(pwd)) strength++;
  if (/[^a-zA-Z0-9]/.test(pwd)) strength++;
  return Math.min(strength, 4);
};

const getStrengthLabel = (strength: number): string => {
  if (strength === 0) return 'Very Weak';
  if (strength === 1) return 'Weak';
  if (strength === 2) return 'Fair';
  if (strength === 3) return 'Strong';
  return 'Very Strong';
};

const getStrengthColor = (strength: number): string => {
  if (strength <= 1) return 'text-red-600 bg-red-600';
  if (strength === 2) return 'text-orange-500 bg-orange-500';
  if (strength === 3) return 'text-yellow-500 bg-yellow-500';
  return 'text-green-600 bg-green-600';
};
```

5. Update button onClick:
```typescript
<button
  onClick={handleSetupWallet}
  disabled={loading || password.length < 8 || password !== confirmPassword}
  className="..."
>
  {loading ? 'Creating Wallet...' : 'Create Secure Wallet'}
</button>
```

6. Update handleSetupWallet:
```typescript
const handleSetupWallet = async () => {
  // Validate passwords match
  if (password !== confirmPassword) {
    setPasswordError('Passwords do not match');
    return;
  }

  // Validate minimum length
  if (password.length < 8) {
    setPasswordError('Password must be at least 8 characters');
    return;
  }

  setLoading(true);
  setPasswordError('');

  try {
    const result = await invoke<FirstTimeSetupResult>('perform_first_time_setup', {
      password: password  // ✅ User-provided password
    });

    setSetupResult(result);
    setStep('reveal');
  } catch (error) {
    console.error('Error setting up wallet:', error);
    setPasswordError('Failed to create wallet. Please try again.');
  } finally {
    setLoading(false);
  }
};
```

**Files Changed:**
- `gui/citrate-core/src/components/FirstTimeSetup.tsx`

---

#### Task 1.2: Update Backend (Optional - already working)
**Estimated Time:** 0.5 hours

**Verify:** The backend `perform_first_time_setup` command already accepts a password parameter:

```rust
// gui/citrate-core/src-tauri/src/wallet/mod.rs
#[tauri::command]
pub fn perform_first_time_setup(password: String) -> Result<FirstTimeSetupResult, String> {
    // Already implements password parameter
    // No changes needed - just verify it works
}
```

**Action:** Test that passing user password works correctly. If issues, debug and fix.

---

#### Task 1.3: Add Keyring Storage (Future Enhancement)
**Estimated Time:** Not included in this sprint

**Note:** For Sprint 1, we're focused on removing the hardcoded password. Secure keyring storage can be added in Sprint 6 (Advanced Features).

**Placeholder for Future:**
- Use `tauri-plugin-keyring` to store password
- Implement auto-lock after inactivity
- Add "Remember password" option

---

### Testing Plan

#### Unit Tests
```typescript
describe('FirstTimeSetup Password Validation', () => {
  it('should require minimum 8 characters', () => {
    const strength = calculatePasswordStrength('abc123');
    expect(strength).toBeLessThan(2);
  });

  it('should detect strong passwords', () => {
    const strength = calculatePasswordStrength('MyP@ssw0rd123!');
    expect(strength).toBeGreaterThanOrEqual(3);
  });

  it('should show error when passwords don\'t match', () => {
    // Test component state
  });

  it('should disable button until validation passes', () => {
    // Test button disabled state
  });
});
```

#### Manual Testing Checklist
- [ ] Can enter password in field
- [ ] Can toggle password visibility
- [ ] Strength indicator updates as typing
- [ ] Error shown if passwords don't match
- [ ] Error shown if less than 8 characters
- [ ] Button disabled until validation passes
- [ ] Wallet created successfully with user password
- [ ] Can unlock wallet with same password later
- [ ] No hardcoded password in source code

---

### Definition of Done
- [ ] All acceptance criteria met
- [ ] Code reviewed and approved
- [ ] Manual testing completed
- [ ] No hardcoded passwords in codebase
- [ ] Password strength indicator working
- [ ] User can create wallet with custom password
- [ ] Documentation updated

---

## Story 2: Input Validation

**Story ID:** S1-02
**Priority:** P0 (Critical - Data Integrity)
**Story Points:** 5
**Assignee:** TBD

### User Story
```
As a user
I want to receive clear feedback when I enter invalid data
So that I don't make mistakes that could result in lost funds
```

### Current State (Problem)
- No validation on send transaction form (amount, recipient address, gas)
- Invalid addresses can be entered
- Negative amounts allowed
- Invalid private keys can be imported
- Invalid mnemonics can be imported
- No visual feedback when input is invalid

### Desired State (Solution)
- Real-time validation with visual feedback
- Clear, specific error messages
- Red border on invalid fields
- Prevents submission of invalid data

---

### Acceptance Criteria

#### AC1: Address Validation
- [ ] Ethereum address checksum validation
- [ ] Shows error if address format invalid
- [ ] Shows error if checksum doesn't match
- [ ] Accepts both checksummed and non-checksummed (converts)
- [ ] Error message: "Invalid address format"

#### AC2: Amount Validation
- [ ] Must be positive number
- [ ] Cannot exceed max supply (1,000,000,000 LATT)
- [ ] Cannot exceed sender's balance
- [ ] Supports decimal places (up to 18)
- [ ] Error messages:
  - "Amount must be positive"
  - "Amount exceeds max supply"
  - "Insufficient balance"

#### AC3: Gas Limit Validation
- [ ] Must be positive integer
- [ ] Minimum 21,000 gas
- [ ] Maximum 10,000,000 gas
- [ ] Error messages:
  - "Gas limit too low (minimum 21,000)"
  - "Gas limit too high (maximum 10,000,000)"

#### AC4: Private Key Validation
- [ ] Must be 64 hexadecimal characters (or 66 with 0x prefix)
- [ ] Must be valid hex string
- [ ] Error message: "Invalid private key format"

#### AC5: Mnemonic Validation
- [ ] Must be 12 or 24 words
- [ ] All words must be in BIP39 wordlist
- [ ] Error messages:
  - "Mnemonic must be 12 or 24 words"
  - "Invalid word: [word]"

#### AC6: Visual Feedback
- [ ] Input border turns red when invalid
- [ ] Input border turns green when valid
- [ ] Error text appears below field
- [ ] Submit button disabled until all fields valid

---

### Technical Tasks

#### Task 2.1: Create Validation Utilities
**Estimated Time:** 2 hours

**Create file:** `gui/citrate-core/src/utils/validation.ts`

```typescript
/**
 * Validation utilities for Citrate GUI
 */

export interface ValidationResult {
  isValid: boolean;
  error?: string;
}

/**
 * Validate Ethereum address with checksum
 */
export function validateAddress(address: string): ValidationResult {
  // Remove 0x prefix if present
  const addr = address.startsWith('0x') ? address.slice(2) : address;

  // Check length
  if (addr.length !== 40) {
    return {
      isValid: false,
      error: 'Address must be 40 hexadecimal characters'
    };
  }

  // Check hex format
  if (!/^[0-9a-fA-F]{40}$/.test(addr)) {
    return {
      isValid: false,
      error: 'Address must contain only hexadecimal characters (0-9, a-f)'
    };
  }

  // TODO: Add checksum validation (EIP-55)
  // For now, accept any valid hex address

  return { isValid: true };
}

/**
 * Validate transaction amount
 */
export function validateAmount(
  amount: string,
  balance?: string,
  maxSupply: string = '1000000000'
): ValidationResult {
  const num = parseFloat(amount);

  if (isNaN(num)) {
    return {
      isValid: false,
      error: 'Amount must be a valid number'
    };
  }

  if (num <= 0) {
    return {
      isValid: false,
      error: 'Amount must be positive'
    };
  }

  if (num > parseFloat(maxSupply)) {
    return {
      isValid: false,
      error: `Amount exceeds maximum supply (${maxSupply} LATT)`
    };
  }

  if (balance && num > parseFloat(balance)) {
    return {
      isValid: false,
      error: 'Insufficient balance'
    };
  }

  return { isValid: true };
}

/**
 * Validate gas limit
 */
export function validateGasLimit(gasLimit: string): ValidationResult {
  const num = parseInt(gasLimit, 10);

  if (isNaN(num)) {
    return {
      isValid: false,
      error: 'Gas limit must be a valid integer'
    };
  }

  if (num < 21000) {
    return {
      isValid: false,
      error: 'Gas limit too low (minimum 21,000)'
    };
  }

  if (num > 10000000) {
    return {
      isValid: false,
      error: 'Gas limit too high (maximum 10,000,000)'
    };
  }

  return { isValid: true };
}

/**
 * Validate private key format
 */
export function validatePrivateKey(privateKey: string): ValidationResult {
  // Remove 0x prefix if present
  const key = privateKey.startsWith('0x') ? privateKey.slice(2) : privateKey;

  if (key.length !== 64) {
    return {
      isValid: false,
      error: 'Private key must be 64 hexadecimal characters'
    };
  }

  if (!/^[0-9a-fA-F]{64}$/.test(key)) {
    return {
      isValid: false,
      error: 'Private key must contain only hexadecimal characters'
    };
  }

  return { isValid: true };
}

/**
 * Validate BIP39 mnemonic phrase
 */
export function validateMnemonic(mnemonic: string): ValidationResult {
  const words = mnemonic.trim().split(/\s+/);

  if (words.length !== 12 && words.length !== 24) {
    return {
      isValid: false,
      error: 'Mnemonic must be 12 or 24 words'
    };
  }

  // Basic wordlist check (simplified - in production use full BIP39 wordlist)
  const invalidWords = words.filter(word =>
    word.length < 3 || !/^[a-z]+$/.test(word)
  );

  if (invalidWords.length > 0) {
    return {
      isValid: false,
      error: `Invalid words: ${invalidWords.join(', ')}`
    };
  }

  return { isValid: true };
}

/**
 * Validate bootnode/peer address
 */
export function validatePeerAddress(address: string): ValidationResult {
  // Format: ip:port or domain:port
  const pattern = /^([a-zA-Z0-9.-]+):(\d+)$/;
  const match = address.match(pattern);

  if (!match) {
    return {
      isValid: false,
      error: 'Address must be in format: host:port (e.g., 127.0.0.1:30303)'
    };
  }

  const port = parseInt(match[2], 10);
  if (port < 1 || port > 65535) {
    return {
      isValid: false,
      error: 'Port must be between 1 and 65535'
    };
  }

  return { isValid: true };
}
```

**Files Created:**
- `gui/citrate-core/src/utils/validation.ts`

---

#### Task 2.2: Add Validation to Wallet Component
**Estimated Time:** 2 hours

**File:** `gui/citrate-core/src/components/Wallet.tsx`

**Changes:**

1. Import validation functions:
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

2. Add validation state:
```typescript
const [recipientError, setRecipientError] = useState('');
const [amountError, setAmountError] = useState('');
const [gasError, setGasError] = useState('');
const [importKeyError, setImportKeyError] = useState('');
const [importMnemonicError, setImportMnemonicError] = useState('');
```

3. Add validation functions:
```typescript
const handleRecipientChange = (value: string) => {
  setRecipient(value);
  const result = validateAddress(value);
  setRecipientError(result.error || '');
};

const handleAmountChange = (value: string) => {
  setAmount(value);
  const balance = selectedAccount?.balance || '0';
  const result = validateAmount(value, balance);
  setAmountError(result.error || '');
};

const handleGasChange = (value: string) => {
  setGasLimit(value);
  const result = validateGasLimit(value);
  setGasError(result.error || '');
};
```

4. Update input fields with validation (line ~200):
```typescript
<input
  type="text"
  value={recipient}
  onChange={(e) => handleRecipientChange(e.target.value)}
  className={`w-full border rounded p-2 ${
    recipientError ? 'border-red-500' : recipient ? 'border-green-500' : 'border-gray-300'
  }`}
  placeholder="0x..."
/>
{recipientError && (
  <p className="text-red-500 text-sm mt-1">{recipientError}</p>
)}
```

5. Disable send button until valid:
```typescript
<button
  onClick={handleSendTransaction}
  disabled={!recipientError && !amountError && !gasError && recipient && amount && gasLimit}
  className="..."
>
  Send Transaction
</button>
```

**Files Changed:**
- `gui/citrate-core/src/components/Wallet.tsx`

---

#### Task 2.3: Add Validation to Settings Component
**Estimated Time:** 1 hour

**File:** `gui/citrate-core/src/components/Settings.tsx`

**Changes:**

1. Import validation:
```typescript
import { validatePeerAddress } from '../utils/validation';
```

2. Add validation state:
```typescript
const [bootnodeError, setBootnodeError] = useState('');
const [peerError, setPeerError] = useState('');
```

3. Add validation handlers:
```typescript
const handleBootnodeChange = (value: string) => {
  setNewBootnode(value);
  const result = validatePeerAddress(value);
  setBootnodeError(result.error || '');
};
```

4. Update inputs with visual feedback and error messages

**Files Changed:**
- `gui/citrate-core/src/components/Settings.tsx`

---

### Testing Plan

#### Unit Tests
```typescript
describe('Validation Functions', () => {
  describe('validateAddress', () => {
    it('should accept valid addresses', () => {
      const result = validateAddress('0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb');
      expect(result.isValid).toBe(true);
    });

    it('should reject invalid addresses', () => {
      const result = validateAddress('invalid');
      expect(result.isValid).toBe(false);
    });
  });

  describe('validateAmount', () => {
    it('should accept positive amounts', () => {
      const result = validateAmount('100.50');
      expect(result.isValid).toBe(true);
    });

    it('should reject negative amounts', () => {
      const result = validateAmount('-50');
      expect(result.isValid).toBe(false);
    });

    it('should reject amounts exceeding balance', () => {
      const result = validateAmount('200', '100');
      expect(result.isValid).toBe(false);
      expect(result.error).toContain('Insufficient balance');
    });
  });
});
```

#### Manual Testing Checklist
- [ ] Invalid address shows red border and error
- [ ] Valid address shows green border
- [ ] Negative amount shows error
- [ ] Amount > balance shows error
- [ ] Gas limit < 21000 shows error
- [ ] Gas limit > 10M shows error
- [ ] Invalid private key rejected
- [ ] Invalid mnemonic rejected
- [ ] Send button disabled when validation fails

---

### Definition of Done
- [ ] All validation functions implemented
- [ ] Visual feedback (red/green borders) working
- [ ] Error messages clear and helpful
- [ ] Submit buttons disabled when invalid
- [ ] Unit tests passing
- [ ] Manual testing completed
- [ ] Code reviewed

---

## Story 3: Error Boundaries

**Story ID:** S1-03
**Priority:** P1 (High - Stability)
**Story Points:** 2
**Assignee:** TBD

### User Story
```
As a user
I want to see helpful error messages when something goes wrong
So that I can understand what happened and potentially recover
```

### Current State (Problem)
- React errors cause white screen of death
- No user-friendly error messages
- Stack traces exposed to users
- App becomes unusable after error

### Desired State (Solution)
- Error boundary catches errors gracefully
- User sees friendly error page with "Reload" option
- Errors logged for debugging
- App can recover from errors

---

### Acceptance Criteria

#### AC1: Error Boundary Component
- [ ] ErrorBoundary component created
- [ ] Catches all React errors in tree
- [ ] Shows fallback UI when error occurs
- [ ] Logs error details to console

#### AC2: Fallback UI
- [ ] User-friendly error message
- [ ] "Reload Page" button
- [ ] "Report Issue" link (GitHub)
- [ ] Does NOT show stack traces to users
- [ ] Citrate branding maintained

#### AC3: Error Recovery
- [ ] Clicking "Reload" refreshes page
- [ ] Error state clears on navigation
- [ ] Local storage preserved

---

### Technical Tasks

#### Task 3.1: Create ErrorBoundary Component
**Estimated Time:** 1.5 hours

**Create file:** `gui/citrate-core/src/components/ErrorBoundary.tsx`

```typescript
import React, { Component, ErrorInfo, ReactNode } from 'react';
import { AlertTriangle } from 'lucide-react';

interface Props {
  children: ReactNode;
}

interface State {
  hasError: boolean;
  error?: Error;
  errorInfo?: ErrorInfo;
}

export class ErrorBoundary extends Component<Props, State> {
  constructor(props: Props) {
    super(props);
    this.state = { hasError: false };
  }

  static getDerivedStateFromError(error: Error): State {
    return { hasError: true, error };
  }

  componentDidCatch(error: Error, errorInfo: ErrorInfo) {
    console.error('Error caught by boundary:', error, errorInfo);
    this.setState({ error, errorInfo });
  }

  handleReload = () => {
    window.location.reload();
  };

  handleReportIssue = () => {
    const issueTitle = encodeURIComponent(`GUI Error: ${this.state.error?.message || 'Unknown'}`);
    const issueBody = encodeURIComponent(`
**Error Message:**
${this.state.error?.message}

**Stack Trace:**
\`\`\`
${this.state.error?.stack}
\`\`\`

**Component Stack:**
\`\`\`
${this.state.errorInfo?.componentStack}
\`\`\`

**User Agent:**
${navigator.userAgent}
    `);

    window.open(
      `https://github.com/saulbuilds/citrate/issues/new?title=${issueTitle}&body=${issueBody}`,
      '_blank'
    );
  };

  render() {
    if (this.state.hasError) {
      return (
        <div className="min-h-screen bg-white flex items-center justify-center p-4">
          <div className="max-w-md w-full">
            <div className="text-center mb-6">
              <div className="inline-flex items-center justify-center w-16 h-16 bg-red-100 rounded-full mb-4">
                <AlertTriangle className="w-8 h-8 text-red-600" />
              </div>
              <h1 className="text-2xl font-bold text-gray-900 mb-2">
                Something went wrong
              </h1>
              <p className="text-gray-600">
                The application encountered an unexpected error.
                Please try reloading the page.
              </p>
            </div>

            {process.env.NODE_ENV === 'development' && this.state.error && (
              <div className="bg-red-50 border border-red-200 rounded-lg p-4 mb-4">
                <p className="text-sm font-mono text-red-800 break-all">
                  {this.state.error.message}
                </p>
              </div>
            )}

            <div className="space-y-3">
              <button
                onClick={this.handleReload}
                className="w-full bg-orange-500 hover:bg-orange-600 text-white py-3 rounded-lg font-medium transition-colors"
              >
                Reload Page
              </button>

              <button
                onClick={this.handleReportIssue}
                className="w-full bg-gray-100 hover:bg-gray-200 text-gray-700 py-3 rounded-lg font-medium transition-colors"
              >
                Report Issue
              </button>
            </div>

            <p className="text-center text-sm text-gray-500 mt-4">
              Error ID: {Date.now()}
            </p>
          </div>
        </div>
      );
    }

    return this.props.children;
  }
}
```

**Files Created:**
- `gui/citrate-core/src/components/ErrorBoundary.tsx`

---

#### Task 3.2: Wrap App with ErrorBoundary
**Estimated Time:** 0.5 hours

**File:** `gui/citrate-core/src/App.tsx`

**Changes:**

1. Import ErrorBoundary:
```typescript
import { ErrorBoundary } from './components/ErrorBoundary';
```

2. Wrap entire app (line ~104):
```typescript
return (
  <ErrorBoundary>
    <div className="app">
      {/* existing app content */}
    </div>

    <FirstTimeSetup onSetupComplete={() => {
      setCurrentView('dashboard');
    }} />
  </ErrorBoundary>
);
```

**Files Changed:**
- `gui/citrate-core/src/App.tsx`

---

### Testing Plan

#### Unit Tests
```typescript
describe('ErrorBoundary', () => {
  it('should render children when no error', () => {
    const { getByText } = render(
      <ErrorBoundary>
        <div>Test Content</div>
      </ErrorBoundary>
    );
    expect(getByText('Test Content')).toBeInTheDocument();
  });

  it('should show fallback UI when error occurs', () => {
    // Test error state
  });
});
```

#### Manual Testing Checklist
- [ ] Intentionally throw error in component
- [ ] Error boundary catches error
- [ ] Fallback UI displayed
- [ ] "Reload Page" button works
- [ ] "Report Issue" opens GitHub with pre-filled details
- [ ] Error logged to console (development)
- [ ] No stack trace shown to users (production)

---

### Definition of Done
- [ ] ErrorBoundary component created
- [ ] App wrapped with ErrorBoundary
- [ ] Fallback UI matches design
- [ ] Reload functionality works
- [ ] Report issue functionality works
- [ ] Manual testing completed
- [ ] Code reviewed

---

## Story 4: Loading Skeletons

**Story ID:** S1-04
**Priority:** P2 (Medium - UX Enhancement)
**Story Points:** 3
**Assignee:** TBD

### User Story
```
As a user
I want to see content placeholders while data loads
So that the app feels faster and I know what to expect
```

### Current State (Problem)
- Generic loading spinners used everywhere
- No indication of what content is loading
- Feels slower than it actually is
- Content "pops in" abruptly

### Desired State (Solution)
- Content-shaped skeleton loaders
- Smooth transition from skeleton to real content
- Better perceived performance
- Professional, polished feel

---

### Acceptance Criteria

#### AC1: Skeleton Components Created
- [ ] DashboardSkeleton component
- [ ] WalletSkeleton component
- [ ] TableSkeleton component
- [ ] CardSkeleton component
- [ ] All use consistent shimmer animation

#### AC2: Dashboard Skeletons
- [ ] Node status box skeleton
- [ ] Stats grid skeleton (4 boxes)
- [ ] Recent blocks skeleton
- [ ] Shown while data loading
- [ ] Smooth fade transition

#### AC3: Wallet Skeletons
- [ ] Account list skeleton (3 rows)
- [ ] Balance skeleton
- [ ] Transaction history skeleton

#### AC4: DAG Skeletons
- [ ] Block table skeleton (10 rows)
- [ ] Statistics panel skeleton

#### AC5: Models Skeletons
- [ ] Model card skeleton (grid)
- [ ] Model details skeleton

---

### Technical Tasks

#### Task 4.1: Create Skeleton Components
**Estimated Time:** 2.5 hours

**Create file:** `gui/citrate-core/src/components/skeletons/Skeleton.tsx` (Base component)

```typescript
import React from 'react';

interface SkeletonProps {
  width?: string;
  height?: string;
  className?: string;
  circle?: boolean;
}

export const Skeleton: React.FC<SkeletonProps> = ({
  width = '100%',
  height = '20px',
  className = '',
  circle = false
}) => {
  return (
    <div
      className={`animate-pulse bg-gray-200 ${circle ? 'rounded-full' : 'rounded'} ${className}`}
      style={{ width, height }}
    />
  );
};
```

**Create:** `DashboardSkeleton.tsx`
```typescript
import React from 'react';
import { Skeleton } from './Skeleton';

export const DashboardSkeleton: React.FC = () => {
  return (
    <div className="p-6 space-y-6">
      {/* Node Status Card */}
      <div className="bg-white rounded-lg p-6 space-y-4">
        <Skeleton width="150px" height="24px" />
        <div className="grid grid-cols-2 gap-4">
          <Skeleton height="60px" />
          <Skeleton height="60px" />
        </div>
      </div>

      {/* Stats Grid */}
      <div className="grid grid-cols-4 gap-4">
        <Skeleton height="100px" />
        <Skeleton height="100px" />
        <Skeleton height="100px" />
        <Skeleton height="100px" />
      </div>

      {/* Recent Blocks */}
      <div className="bg-white rounded-lg p-6 space-y-4">
        <Skeleton width="150px" height="24px" />
        {[...Array(5)].map((_, i) => (
          <Skeleton key={i} height="60px" />
        ))}
      </div>
    </div>
  );
};
```

**Create:** `WalletSkeleton.tsx`, `TableSkeleton.tsx`, `CardSkeleton.tsx`

**Files Created:**
- `gui/citrate-core/src/components/skeletons/Skeleton.tsx`
- `gui/citrate-core/src/components/skeletons/DashboardSkeleton.tsx`
- `gui/citrate-core/src/components/skeletons/WalletSkeleton.tsx`
- `gui/citrate-core/src/components/skeletons/TableSkeleton.tsx`
- `gui/citrate-core/src/components/skeletons/CardSkeleton.tsx`

---

#### Task 4.2-4.5: Integrate Skeletons into Components
**Estimated Time:** 3 hours total (0.75 hours each)

**Pattern for all components:**

```typescript
import { DashboardSkeleton } from './skeletons/DashboardSkeleton';

export const Dashboard = () => {
  const [loading, setLoading] = useState(true);
  const [data, setData] = useState(null);

  if (loading) {
    return <DashboardSkeleton />;
  }

  return (
    // actual content
  );
};
```

**Files Changed:**
- `gui/citrate-core/src/components/Dashboard.tsx`
- `gui/citrate-core/src/components/Wallet.tsx`
- `gui/citrate-core/src/components/DAGVisualization.tsx`
- `gui/citrate-core/src/components/Models.tsx`

---

### Testing Plan

#### Manual Testing Checklist
- [ ] Dashboard shows skeleton while loading
- [ ] Skeleton matches actual content shape
- [ ] Smooth transition from skeleton to content
- [ ] Shimmer animation smooth
- [ ] Works on all supported browsers
- [ ] Mobile responsive

---

### Definition of Done
- [ ] All skeleton components created
- [ ] Integrated into 4 main components
- [ ] Shimmer animation working
- [ ] Smooth transitions
- [ ] Manual testing completed
- [ ] Code reviewed

---

**Total Stories:** 4
**Total Story Points:** 13
**Estimated Duration:** 5 days
