/**
 * Error classes for Citrate JavaScript SDK
 */

export class CitrateError extends Error {
  public readonly code?: string;
  public readonly details?: Record<string, any>;

  constructor(message: string, code?: string, details?: Record<string, any>) {
    super(message);
    this.name = 'CitrateError';
    this.code = code;
    this.details = details;

    // Maintains proper stack trace for where our error was thrown (only available on V8)
    if (Error.captureStackTrace) {
      Error.captureStackTrace(this, CitrateError);
    }
  }
}

export class NetworkError extends CitrateError {
  constructor(message: string, details?: Record<string, any>) {
    super(message, 'NETWORK_ERROR', details);
    this.name = 'NetworkError';
  }
}

export class AuthenticationError extends CitrateError {
  constructor(message: string, details?: Record<string, any>) {
    super(message, 'AUTHENTICATION_ERROR', details);
    this.name = 'AuthenticationError';
  }
}

export class ModelNotFoundError extends CitrateError {
  constructor(message: string, details?: Record<string, any>) {
    super(message, 'MODEL_NOT_FOUND', details);
    this.name = 'ModelNotFoundError';
  }
}

export class InsufficientFundsError extends CitrateError {
  constructor(message: string, details?: Record<string, any>) {
    super(message, 'INSUFFICIENT_FUNDS', details);
    this.name = 'InsufficientFundsError';
  }
}

export class ModelDeploymentError extends CitrateError {
  constructor(message: string, details?: Record<string, any>) {
    super(message, 'MODEL_DEPLOYMENT_ERROR', details);
    this.name = 'ModelDeploymentError';
  }
}

export class InferenceError extends CitrateError {
  constructor(message: string, details?: Record<string, any>) {
    super(message, 'INFERENCE_ERROR', details);
    this.name = 'InferenceError';
  }
}

export class EncryptionError extends CitrateError {
  constructor(message: string, details?: Record<string, any>) {
    super(message, 'ENCRYPTION_ERROR', details);
    this.name = 'EncryptionError';
  }
}

export class ValidationError extends CitrateError {
  constructor(message: string, details?: Record<string, any>) {
    super(message, 'VALIDATION_ERROR', details);
    this.name = 'ValidationError';
  }
}

export class TimeoutError extends CitrateError {
  constructor(message: string, details?: Record<string, any>) {
    super(message, 'TIMEOUT_ERROR', details);
    this.name = 'TimeoutError';
  }
}

export class ConfigurationError extends CitrateError {
  constructor(message: string, details?: Record<string, any>) {
    super(message, 'CONFIGURATION_ERROR', details);
    this.name = 'ConfigurationError';
  }
}

export class IPFSError extends CitrateError {
  constructor(message: string, details?: Record<string, any>) {
    super(message, 'IPFS_ERROR', details);
    this.name = 'IPFSError';
  }
}

export class ContractError extends CitrateError {
  constructor(message: string, details?: Record<string, any>) {
    super(message, 'CONTRACT_ERROR', details);
    this.name = 'ContractError';
  }
}