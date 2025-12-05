/**
 * Type declarations for solc (Solidity Compiler)
 */

declare module 'solc' {
  interface ImportCallback {
    (path: string): { contents?: string; error?: string };
  }

  interface CompileOptions {
    import?: ImportCallback;
  }

  /**
   * Compile Solidity source code
   * @param input - JSON stringified input
   * @param options - Compilation options including import callback
   * @returns JSON stringified output
   */
  export function compile(input: string, options?: CompileOptions): string;

  /**
   * Get the compiler version
   */
  export function version(): string;

  /**
   * Load a specific version of the compiler
   * @param version - Version string (e.g., 'v0.8.28+commit.7893614a')
   */
  export function loadRemoteVersion(
    version: string,
    callback: (err: Error | null, solc: typeof import('solc')) => void
  ): void;

  /**
   * Setup methods for compiler
   */
  export function setupMethods(soljson: any): typeof import('solc');

  // Default export
  const solc: {
    compile: typeof compile;
    version: typeof version;
    loadRemoteVersion: typeof loadRemoteVersion;
    setupMethods: typeof setupMethods;
  };

  export default solc;
}
