/**
 * Validation utilities for Citrate GUI
 *
 * Provides comprehensive input validation for:
 * - Ethereum-compatible addresses
 * - Transaction amounts
 * - Gas limits
 * - Private keys (ed25519 and ECDSA)
 * - BIP39 mnemonic phrases
 * - Network bootnodes (multiaddr format)
 */

export interface ValidationResult {
  isValid: boolean;
  error?: string;
}

/**
 * Validate Ethereum-compatible address (20 bytes hex)
 * Accepts with or without '0x' prefix
 */
export function validateAddress(address: string): ValidationResult {
  if (!address || address.trim() === '') {
    return { isValid: false, error: 'Address is required' };
  }

  const addr = address.trim().startsWith('0x')
    ? address.trim().slice(2)
    : address.trim();

  if (addr.length !== 40) {
    return {
      isValid: false,
      error: 'Address must be 40 hexadecimal characters (20 bytes)'
    };
  }

  if (!/^[0-9a-fA-F]{40}$/.test(addr)) {
    return {
      isValid: false,
      error: 'Address must contain only hexadecimal characters (0-9, a-f)'
    };
  }

  // Optional: Add checksum validation here if needed
  // For now, basic format validation is sufficient

  return { isValid: true };
}

/**
 * Validate transaction amount
 *
 * @param amount - Amount as string
 * @param balance - Optional user balance to check against
 * @param maxSupply - Optional maximum supply to check against (default: 1 billion)
 */
export function validateAmount(
  amount: string,
  balance?: string,
  maxSupply: string = '1000000000'
): ValidationResult {
  if (!amount || amount.trim() === '') {
    return { isValid: false, error: 'Amount is required' };
  }

  const num = parseFloat(amount);

  if (isNaN(num)) {
    return { isValid: false, error: 'Amount must be a valid number' };
  }

  if (num <= 0) {
    return { isValid: false, error: 'Amount must be greater than zero' };
  }

  if (!isFinite(num)) {
    return { isValid: false, error: 'Amount must be a finite number' };
  }

  // Check against maximum supply
  const maxSupplyNum = parseFloat(maxSupply);
  if (num > maxSupplyNum) {
    return {
      isValid: false,
      error: `Amount exceeds maximum supply (${maxSupply})`
    };
  }

  // Check against user balance if provided
  if (balance) {
    const balanceNum = parseFloat(balance);
    if (!isNaN(balanceNum) && num > balanceNum) {
      return {
        isValid: false,
        error: `Insufficient balance (available: ${balance})`
      };
    }
  }

  // Check for reasonable decimal places (max 18 for EVM compatibility)
  const decimalPlaces = (amount.split('.')[1] || '').length;
  if (decimalPlaces > 18) {
    return {
      isValid: false,
      error: 'Amount has too many decimal places (maximum: 18)'
    };
  }

  return { isValid: true };
}

/**
 * Validate gas limit for EVM transactions
 *
 * Reasonable bounds:
 * - Minimum: 21,000 (simple transfer)
 * - Maximum: 10,000,000 (complex contract interaction)
 */
export function validateGasLimit(gasLimit: string): ValidationResult {
  if (!gasLimit || gasLimit.trim() === '') {
    return { isValid: false, error: 'Gas limit is required' };
  }

  const num = parseInt(gasLimit, 10);

  if (isNaN(num)) {
    return { isValid: false, error: 'Gas limit must be a valid integer' };
  }

  if (num < 0) {
    return { isValid: false, error: 'Gas limit cannot be negative' };
  }

  if (num < 21000) {
    return {
      isValid: false,
      error: 'Gas limit too low (minimum: 21,000 for simple transfer)'
    };
  }

  if (num > 10000000) {
    return {
      isValid: false,
      error: 'Gas limit too high (maximum: 10,000,000)'
    };
  }

  // Ensure it's an integer (no decimals)
  if (gasLimit.includes('.')) {
    return { isValid: false, error: 'Gas limit must be a whole number' };
  }

  return { isValid: true };
}

/**
 * Validate private key (32 bytes hex for ed25519 or secp256k1)
 * Accepts with or without '0x' prefix
 */
export function validatePrivateKey(privateKey: string): ValidationResult {
  if (!privateKey || privateKey.trim() === '') {
    return { isValid: false, error: 'Private key is required' };
  }

  const key = privateKey.trim().startsWith('0x')
    ? privateKey.trim().slice(2)
    : privateKey.trim();

  if (key.length !== 64) {
    return {
      isValid: false,
      error: 'Private key must be 64 hexadecimal characters (32 bytes)'
    };
  }

  if (!/^[0-9a-fA-F]{64}$/.test(key)) {
    return {
      isValid: false,
      error: 'Private key must contain only hexadecimal characters (0-9, a-f)'
    };
  }

  // Check for all-zeros private key (invalid)
  if (/^0+$/.test(key)) {
    return {
      isValid: false,
      error: 'Private key cannot be all zeros'
    };
  }

  return { isValid: true };
}

/**
 * Validate BIP39 mnemonic phrase
 *
 * Supports 12-word and 24-word mnemonics
 * Validates word count but not BIP39 wordlist (can be added later)
 */
export function validateMnemonic(mnemonic: string): ValidationResult {
  if (!mnemonic || mnemonic.trim() === '') {
    return { isValid: false, error: 'Mnemonic phrase is required' };
  }

  // Normalize whitespace (tabs, newlines, multiple spaces)
  const normalized = mnemonic.trim().replace(/\s+/g, ' ');
  const words = normalized.split(' ');

  // Count non-empty words
  const wordCount = words.filter(w => w.length > 0).length;

  if (wordCount !== 12 && wordCount !== 24) {
    return {
      isValid: false,
      error: `Mnemonic must be 12 or 24 words (found: ${wordCount})`
    };
  }

  // Check that all words are lowercase alphanumeric (basic validation)
  const invalidWords = words.filter(w => !/^[a-z]+$/.test(w));
  if (invalidWords.length > 0) {
    return {
      isValid: false,
      error: 'Mnemonic words must contain only lowercase letters'
    };
  }

  // Optional: Add BIP39 wordlist validation here
  // For now, basic format validation is sufficient

  return { isValid: true };
}

/**
 * Validate bootnode multiaddr format
 *
 * Expected formats:
 * - /ip4/127.0.0.1/tcp/9000/p2p/12D3KooW...
 * - /ip6/::1/tcp/9000/p2p/12D3KooW...
 * - /dns/example.com/tcp/9000/p2p/12D3KooW...
 */
export function validateBootnode(bootnode: string): ValidationResult {
  if (!bootnode || bootnode.trim() === '') {
    return { isValid: false, error: 'Bootnode address is required' };
  }

  const addr = bootnode.trim();

  // Basic multiaddr format check (starts with /)
  if (!addr.startsWith('/')) {
    return {
      isValid: false,
      error: 'Bootnode must be in multiaddr format (e.g., /ip4/127.0.0.1/tcp/9000/p2p/...)'
    };
  }

  // Check for required components: ip4/ip6/dns, tcp, and p2p
  const hasIP = /\/(ip4|ip6|dns)\//.test(addr);
  const hasTCP = /\/tcp\//.test(addr);
  const hasP2P = /\/p2p\//.test(addr);

  if (!hasIP) {
    return {
      isValid: false,
      error: 'Bootnode must include IP address (/ip4/, /ip6/, or /dns/)'
    };
  }

  if (!hasTCP) {
    return {
      isValid: false,
      error: 'Bootnode must include TCP port (/tcp/PORT)'
    };
  }

  if (!hasP2P) {
    return {
      isValid: false,
      error: 'Bootnode must include peer ID (/p2p/PEER_ID)'
    };
  }

  // Validate port number (extract and check)
  const portMatch = addr.match(/\/tcp\/(\d+)/);
  if (portMatch) {
    const port = parseInt(portMatch[1], 10);
    if (port < 1 || port > 65535) {
      return {
        isValid: false,
        error: 'TCP port must be between 1 and 65535'
      };
    }
  }

  // Validate IP address if present
  const ip4Match = addr.match(/\/ip4\/([^\/]+)/);
  if (ip4Match) {
    const ipValidation = validateIPv4(ip4Match[1]);
    if (!ipValidation.isValid) {
      return ipValidation;
    }
  }

  return { isValid: true };
}

/**
 * Validate IPv4 address
 */
export function validateIPv4(ip: string): ValidationResult {
  if (!ip || ip.trim() === '') {
    return { isValid: false, error: 'IP address is required' };
  }

  const parts = ip.trim().split('.');

  if (parts.length !== 4) {
    return {
      isValid: false,
      error: 'IPv4 address must have 4 octets (e.g., 192.168.1.1)'
    };
  }

  for (const part of parts) {
    const num = parseInt(part, 10);

    if (isNaN(num)) {
      return {
        isValid: false,
        error: 'IPv4 octets must be numbers'
      };
    }

    if (num < 0 || num > 255) {
      return {
        isValid: false,
        error: 'IPv4 octets must be between 0 and 255'
      };
    }

    // Check for leading zeros (invalid in strict parsing)
    if (part.length > 1 && part.startsWith('0')) {
      return {
        isValid: false,
        error: 'IPv4 octets cannot have leading zeros'
      };
    }
  }

  return { isValid: true };
}

/**
 * Validate port number (standalone)
 */
export function validatePort(port: string): ValidationResult {
  if (!port || port.trim() === '') {
    return { isValid: false, error: 'Port is required' };
  }

  const num = parseInt(port, 10);

  if (isNaN(num)) {
    return { isValid: false, error: 'Port must be a valid number' };
  }

  if (num < 1 || num > 65535) {
    return {
      isValid: false,
      error: 'Port must be between 1 and 65535'
    };
  }

  return { isValid: true };
}

/**
 * Validate gas price (in wei or gwei)
 */
export function validateGasPrice(gasPrice: string): ValidationResult {
  if (!gasPrice || gasPrice.trim() === '') {
    return { isValid: false, error: 'Gas price is required' };
  }

  const num = parseFloat(gasPrice);

  if (isNaN(num)) {
    return { isValid: false, error: 'Gas price must be a valid number' };
  }

  if (num < 0) {
    return { isValid: false, error: 'Gas price cannot be negative' };
  }

  if (!isFinite(num)) {
    return { isValid: false, error: 'Gas price must be a finite number' };
  }

  return { isValid: true };
}

/**
 * Helper: Sanitize string input (remove leading/trailing whitespace)
 */
export function sanitizeInput(input: string): string {
  return input.trim();
}

/**
 * Helper: Format address for display (0x... + last 6 chars)
 */
export function formatAddressShort(address: string): string {
  if (!address || address.length < 10) return address;
  const addr = address.startsWith('0x') ? address : `0x${address}`;
  return `${addr.slice(0, 8)}...${addr.slice(-6)}`;
}

/**
 * Helper: Format amount with commas for display
 */
export function formatAmount(amount: string | number): string {
  const num = typeof amount === 'string' ? parseFloat(amount) : amount;
  if (isNaN(num)) return '0';

  return num.toLocaleString('en-US', {
    minimumFractionDigits: 0,
    maximumFractionDigits: 6
  });
}
