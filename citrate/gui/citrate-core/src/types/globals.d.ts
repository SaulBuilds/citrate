/**
 * Global type declarations for Citrate build-time constants
 *
 * These constants are injected by Vite at build time and provide
 * dev mode gating, version info, and build metadata.
 */

declare global {
  /**
   * Dev mode flag - true in development, false in production
   * Use this to gate dev-only features and mock code
   */
  const __DEV_MODE__: boolean;

  /**
   * Current application version from package.json
   */
  const __CITRATE_VERSION__: string;

  /**
   * ISO timestamp of when the build was created
   */
  const __BUILD_TIME__: string;
}

export {};
