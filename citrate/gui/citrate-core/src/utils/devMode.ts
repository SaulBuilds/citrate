/**
 * Development Mode Utilities
 *
 * Provides functions for dev mode gating to ensure mock/demo code
 * is never executed in production builds.
 *
 * Usage:
 * ```typescript
 * import { isDevMode, devOnly, DevModeIndicator } from './utils/devMode';
 *
 * // Check dev mode
 * if (isDevMode()) {
 *   console.log('Dev mode is enabled');
 * }
 *
 * // Execute only in dev mode
 * devOnly(() => {
 *   enableMockData();
 * });
 *
 * // Show dev mode indicator in UI
 * <DevModeIndicator />
 * ```
 */

/**
 * Check if the app is running in development mode.
 * This is determined at build time by Vite.
 */
export function isDevMode(): boolean {
  return __DEV_MODE__;
}

/**
 * Check if the app is running in production mode.
 */
export function isProdMode(): boolean {
  return !__DEV_MODE__;
}

/**
 * Get the current application version.
 */
export function getVersion(): string {
  return __CITRATE_VERSION__;
}

/**
 * Get the build timestamp.
 */
export function getBuildTime(): string {
  return __BUILD_TIME__;
}

/**
 * Execute a function only in development mode.
 * In production, this is a no-op and the code will be tree-shaken.
 *
 * @param fn - Function to execute in dev mode
 * @returns The result of the function, or undefined in production
 */
export function devOnly<T>(fn: () => T): T | undefined {
  if (__DEV_MODE__) {
    return fn();
  }
  return undefined;
}

/**
 * Execute different functions based on dev/prod mode.
 *
 * @param devFn - Function to execute in development mode
 * @param prodFn - Function to execute in production mode
 * @returns The result of the appropriate function
 */
export function devOrProd<T>(devFn: () => T, prodFn: () => T): T {
  if (__DEV_MODE__) {
    return devFn();
  }
  return prodFn();
}

/**
 * Log a message only in development mode.
 * In production, this is stripped out by esbuild.
 */
export function devLog(message: string, ...args: unknown[]): void {
  if (__DEV_MODE__) {
    console.log(`[DEV] ${message}`, ...args);
  }
}

/**
 * Log a warning only in development mode.
 */
export function devWarn(message: string, ...args: unknown[]): void {
  if (__DEV_MODE__) {
    console.warn(`[DEV] ${message}`, ...args);
  }
}

/**
 * Log an error only in development mode.
 * Note: In production, you should use proper error reporting.
 */
export function devError(message: string, ...args: unknown[]): void {
  if (__DEV_MODE__) {
    console.error(`[DEV] ${message}`, ...args);
  }
}

/**
 * Assert a condition only in development mode.
 * In production, this is a no-op.
 *
 * @param condition - Condition to assert
 * @param message - Error message if assertion fails
 */
export function devAssert(condition: boolean, message: string): void {
  if (__DEV_MODE__ && !condition) {
    throw new Error(`[DEV ASSERTION FAILED] ${message}`);
  }
}

/**
 * Create a mock value for development.
 * In production, returns the fallback value.
 *
 * @param mockValue - Value to use in development
 * @param fallbackValue - Value to use in production
 * @returns The appropriate value based on mode
 */
export function mockValue<T>(mockValue: T, fallbackValue: T): T {
  if (__DEV_MODE__) {
    return mockValue;
  }
  return fallbackValue;
}

/**
 * Wrap a function to add dev mode validation.
 * If called in production with mock data, logs a warning.
 *
 * @param fn - Function to wrap
 * @param fnName - Name of the function for logging
 * @returns Wrapped function
 */
export function wrapWithDevCheck<TArgs extends unknown[], TReturn>(
  fn: (...args: TArgs) => TReturn,
  fnName: string
): (...args: TArgs) => TReturn {
  return (...args: TArgs): TReturn => {
    if (!__DEV_MODE__) {
      devWarn(`Function ${fnName} called in production mode`);
    }
    return fn(...args);
  };
}

/**
 * Build info object containing version and build metadata.
 */
export interface BuildInfo {
  version: string;
  buildTime: string;
  isDevMode: boolean;
  environment: 'development' | 'production';
}

/**
 * Get complete build information.
 */
export function getBuildInfo(): BuildInfo {
  return {
    version: __CITRATE_VERSION__,
    buildTime: __BUILD_TIME__,
    isDevMode: __DEV_MODE__,
    environment: __DEV_MODE__ ? 'development' : 'production',
  };
}

/**
 * Print build info to console (dev mode only).
 */
export function printBuildInfo(): void {
  if (__DEV_MODE__) {
    const info = getBuildInfo();
    console.group('🔧 Citrate Build Info');
    console.log(`Version: ${info.version}`);
    console.log(`Build Time: ${info.buildTime}`);
    console.log(`Environment: ${info.environment}`);
    console.log(`Dev Mode: ${info.isDevMode}`);
    console.groupEnd();
  }
}

/**
 * Feature flags for development features.
 * Add new flags here as needed.
 */
export const DEV_FEATURES = {
  /** Enable mock blockchain data */
  MOCK_BLOCKCHAIN: __DEV_MODE__,
  /** Enable mock AI responses */
  MOCK_AI: __DEV_MODE__,
  /** Enable mock IPFS operations */
  MOCK_IPFS: __DEV_MODE__,
  /** Enable verbose logging */
  VERBOSE_LOGGING: __DEV_MODE__,
  /** Enable performance profiling */
  PROFILING: __DEV_MODE__,
  /** Enable experimental features */
  EXPERIMENTAL: __DEV_MODE__,
} as const;

/**
 * Check if a specific dev feature is enabled.
 *
 * @param feature - Feature flag name
 * @returns Whether the feature is enabled
 */
export function isDevFeatureEnabled(feature: keyof typeof DEV_FEATURES): boolean {
  return DEV_FEATURES[feature];
}

export default {
  isDevMode,
  isProdMode,
  getVersion,
  getBuildTime,
  devOnly,
  devOrProd,
  devLog,
  devWarn,
  devError,
  devAssert,
  mockValue,
  wrapWithDevCheck,
  getBuildInfo,
  printBuildInfo,
  DEV_FEATURES,
  isDevFeatureEnabled,
};
