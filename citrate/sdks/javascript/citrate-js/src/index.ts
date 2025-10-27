/**
 * Citrate JavaScript/TypeScript SDK
 *
 * A comprehensive SDK for interacting with the Citrate AI blockchain platform.
 * Provides easy-to-use interfaces for model deployment, inference execution,
 * encryption, access control, and payment systems.
 */

export { CitrateClient } from './client/CitrateClient';
export { WebSocketClient } from './client/WebSocketClient';

// Types and interfaces
export * from './types/Model';
export * from './types/Inference';
export * from './types/Crypto';
export * from './types/Transaction';
export * from './types/Client';

// Crypto utilities
export { CryptoManager } from './crypto/CryptoManager';
export { KeyManager } from './crypto/KeyManager';
export { splitSecretBytes, reconstructSecretBytes, GF256, ShamirSecretSharing } from './crypto/FiniteField';

// Error classes
export * from './errors/CitrateError';

// React hooks (if React is available)
export * from './react/hooks';

// Utils
export * from './utils/constants';
export * from './utils/validation';

// Version
export const VERSION = '0.1.0';