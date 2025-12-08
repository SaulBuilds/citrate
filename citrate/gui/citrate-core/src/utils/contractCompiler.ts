/**
 * Contract Compiler Utility
 *
 * Compiles Solidity smart contracts using solc-js WASM.
 * Supports version selection, optimizer configuration, and import resolution.
 *
 * NOTE: solc is loaded dynamically to avoid blocking app startup.
 * The solc package uses Node.js internals (util.inherits) that don't work
 * in browser environments if imported at module level.
 */

// Lazy-loaded solc instance
let solcInstance: typeof import('solc') | null = null;

/**
 * Lazily load the solc compiler
 * This avoids the Node.js compatibility error at startup
 */
async function getSolc(): Promise<typeof import('solc')> {
  if (!solcInstance) {
    // Dynamic import to defer loading until needed
    solcInstance = await import('solc');
  }
  return solcInstance;
}

export interface CompilationError {
  severity: 'error' | 'warning' | 'info';
  message: string;
  line?: number;
  column?: number;
  formattedMessage?: string;
  sourceLocation?: {
    file: string;
    start: number;
    end: number;
  };
}

export interface CompilationResult {
  success: boolean;
  bytecode?: string;
  deployedBytecode?: string;
  abi?: any[];
  errors?: CompilationError[];
  warnings?: string[];
  gasEstimate?: number;
  contractSize?: number;
  metadata?: {
    compiler: string;
    language: string;
    output: any;
    settings: any;
  };
}

export interface CompilerOptions {
  /** Solidity version (e.g., '0.8.28') */
  version?: string;
  /** Enable optimizer */
  optimize?: boolean;
  /** Optimizer runs (default: 200) */
  optimizerRuns?: number;
  /** EVM version target */
  evmVersion?: 'london' | 'paris' | 'shanghai' | 'cancun';
  /** Import remappings for resolving imports */
  remappings?: string[];
  /** Additional source files for imports */
  sources?: { [filename: string]: string };
}

/** Default compiler options */
const DEFAULT_OPTIONS: Required<Omit<CompilerOptions, 'remappings' | 'sources'>> = {
  version: '0.8.28',
  optimize: true,
  optimizerRuns: 200,
  evmVersion: 'shanghai',
};

/** Common OpenZeppelin imports that we can provide */
const COMMON_IMPORTS: { [path: string]: string } = {
  // These would be populated from node_modules or a CDN in a full implementation
  // For now, we handle them via remappings or external resolution
};

/**
 * Compile a Solidity contract using solc-js
 *
 * @param source - Solidity source code
 * @param contractName - Name of the contract to compile
 * @param options - Compiler options
 * @returns Compilation result with bytecode, ABI, or errors
 *
 * @example
 * ```typescript
 * const result = await compileContract(source, 'MyToken', {
 *   optimize: true,
 *   optimizerRuns: 200,
 * });
 * if (result.success) {
 *   console.log('Bytecode:', result.bytecode);
 *   console.log('ABI:', result.abi);
 * }
 * ```
 */
export async function compileContract(
  source: string,
  contractName: string,
  options: CompilerOptions = {}
): Promise<CompilationResult> {
  console.log(`[Compiler] Compiling contract: ${contractName}`);

  // Validate input
  if (!source || !source.trim()) {
    return {
      success: false,
      errors: [{
        severity: 'error',
        message: 'Source code is empty',
      }],
    };
  }

  if (!contractName || !contractName.trim()) {
    return {
      success: false,
      errors: [{
        severity: 'error',
        message: 'Contract name is required',
      }],
    };
  }

  // Merge with defaults
  const opts = {
    ...DEFAULT_OPTIONS,
    ...options,
  };

  try {
    // Build solc input JSON
    const sources: { [filename: string]: { content: string } } = {
      'Contract.sol': { content: source },
    };

    // Add any additional source files
    if (options.sources) {
      for (const [filename, content] of Object.entries(options.sources)) {
        sources[filename] = { content };
      }
    }

    const input = {
      language: 'Solidity',
      sources,
      settings: {
        outputSelection: {
          '*': {
            '*': [
              'abi',
              'evm.bytecode',
              'evm.deployedBytecode',
              'evm.gasEstimates',
              'metadata',
              'devdoc',
              'userdoc',
            ],
          },
        },
        optimizer: {
          enabled: opts.optimize,
          runs: opts.optimizerRuns,
        },
        evmVersion: opts.evmVersion,
        remappings: options.remappings || [],
      },
    };

    console.log('[Compiler] Compiler settings:', {
      optimize: opts.optimize,
      runs: opts.optimizerRuns,
      evmVersion: opts.evmVersion,
    });

    // Import callback for resolving imports
    const findImports = (importPath: string): { contents?: string; error?: string } => {
      console.log('[Compiler] Resolving import:', importPath);

      // Check if we have the import in our common imports
      if (COMMON_IMPORTS[importPath]) {
        return { contents: COMMON_IMPORTS[importPath] };
      }

      // Check additional sources
      if (options.sources && options.sources[importPath]) {
        return { contents: options.sources[importPath] };
      }

      // For external imports (OpenZeppelin etc.), return error with helpful message
      // In a browser environment, we'd need to fetch these from a CDN or have them bundled
      return {
        error: `Import "${importPath}" not found. For OpenZeppelin contracts, please include the full source or use Foundry CLI mode.`,
      };
    };

    // Compile
    console.log('[Compiler] Starting compilation...');
    const solc = await getSolc();
    const outputJson = solc.compile(JSON.stringify(input), { import: findImports });
    const output = JSON.parse(outputJson);

    // Process errors and warnings
    const errors: CompilationError[] = [];
    const warnings: string[] = [];

    if (output.errors) {
      for (const err of output.errors) {
        const compilationError: CompilationError = {
          severity: err.severity,
          message: err.message,
          formattedMessage: err.formattedMessage,
        };

        // Extract line/column from source location
        if (err.sourceLocation) {
          compilationError.sourceLocation = err.sourceLocation;
          // Parse line number from formatted message
          const lineMatch = err.formattedMessage?.match(/:(\d+):/);
          if (lineMatch) {
            compilationError.line = parseInt(lineMatch[1], 10);
          }
        }

        if (err.severity === 'error') {
          errors.push(compilationError);
        } else if (err.severity === 'warning') {
          warnings.push(err.message);
        }
      }
    }

    // Check for fatal errors
    if (errors.length > 0) {
      console.error('[Compiler] Compilation failed with errors:', errors);
      return {
        success: false,
        errors,
        warnings,
      };
    }

    // Find the compiled contract
    const contracts = output.contracts?.['Contract.sol'];
    if (!contracts) {
      return {
        success: false,
        errors: [{
          severity: 'error',
          message: 'No contracts compiled. Check your source code.',
        }],
        warnings,
      };
    }

    const contract = contracts[contractName];
    if (!contract) {
      // List available contracts
      const available = Object.keys(contracts);
      return {
        success: false,
        errors: [{
          severity: 'error',
          message: `Contract "${contractName}" not found in compilation output. Available: ${available.join(', ')}`,
        }],
        warnings,
      };
    }

    // Extract bytecode
    const bytecode = contract.evm?.bytecode?.object
      ? '0x' + contract.evm.bytecode.object
      : undefined;
    const deployedBytecode = contract.evm?.deployedBytecode?.object
      ? '0x' + contract.evm.deployedBytecode.object
      : undefined;

    if (!bytecode) {
      return {
        success: false,
        errors: [{
          severity: 'error',
          message: 'Compilation succeeded but no bytecode was generated. The contract may be abstract or an interface.',
        }],
        warnings,
      };
    }

    // Calculate sizes
    const contractSize = (bytecode.length - 2) / 2; // Remove '0x' and convert hex to bytes
    const gasEstimate = estimateDeploymentGas(bytecode);

    // Get gas estimates from compiler if available
    let creationGasEstimate: number | undefined;
    if (contract.evm?.gasEstimates?.creation) {
      const creation = contract.evm.gasEstimates.creation;
      creationGasEstimate = (creation.codeDepositCost || 0) + (creation.executionCost || 0);
    }

    console.log('[Compiler] Compilation successful:', {
      contractName,
      bytecodeSize: contractSize,
      abiEntries: contract.abi?.length || 0,
      gasEstimate: creationGasEstimate || gasEstimate,
    });

    return {
      success: true,
      bytecode,
      deployedBytecode,
      abi: contract.abi,
      gasEstimate: creationGasEstimate || gasEstimate,
      contractSize,
      warnings: warnings.length > 0 ? warnings : undefined,
      metadata: {
        compiler: `solc-js@${solc.version ? solc.version() : 'unknown'}`,
        language: 'Solidity',
        output: contract.metadata,
        settings: {
          optimizer: opts.optimize,
          runs: opts.optimizerRuns,
          evmVersion: opts.evmVersion,
        },
      },
    };

  } catch (error: any) {
    console.error('[Compiler] Compilation failed:', error);
    return {
      success: false,
      errors: [{
        severity: 'error',
        message: `Compilation error: ${error.message || 'Unknown error'}`,
      }],
    };
  }
}

/**
 * Compile multiple contracts from a single source
 *
 * @param source - Solidity source code with multiple contracts
 * @param options - Compiler options
 * @returns Map of contract names to compilation results
 */
export async function compileAllContracts(
  source: string,
  options: CompilerOptions = {}
): Promise<{ [contractName: string]: CompilationResult }> {
  const results: { [contractName: string]: CompilationResult } = {};

  // Extract contract names from source
  const contractMatches = source.matchAll(/contract\s+(\w+)/g);
  const contractNames = Array.from(contractMatches, m => m[1]);

  if (contractNames.length === 0) {
    return {
      '__error__': {
        success: false,
        errors: [{
          severity: 'error',
          message: 'No contracts found in source code',
        }],
      },
    };
  }

  for (const name of contractNames) {
    results[name] = await compileContract(source, name, options);
  }

  return results;
}

/**
 * Get the current solc-js version
 */
export async function getCompilerVersion(): Promise<string> {
  const solc = await getSolc();
  return solc.version();
}

/**
 * Estimate gas required for contract deployment
 *
 * Formula:
 * - 21,000 gas base transaction cost
 * - 32,000 gas contract creation cost
 * - 200 gas per byte of code for storage
 * - 16 gas per non-zero byte / 4 gas per zero byte for calldata
 *
 * @param bytecode - Contract bytecode (with or without '0x' prefix)
 * @returns Estimated gas
 */
export function estimateDeploymentGas(bytecode: string): number {
  const cleanBytecode = bytecode.startsWith('0x') ? bytecode.slice(2) : bytecode;
  const bytes = cleanBytecode.match(/.{2}/g) || [];

  let calldataGas = 0;
  for (const byte of bytes) {
    calldataGas += byte === '00' ? 4 : 16;
  }

  const baseGas = 21000; // Transaction base cost
  const creationGas = 32000; // Contract creation cost
  const codeDepositGas = bytes.length * 200; // Code storage cost

  const totalGas = baseGas + creationGas + codeDepositGas + calldataGas;

  // Add 20% buffer for execution costs
  return Math.ceil(totalGas * 1.2);
}

/**
 * Validate contract bytecode size
 *
 * EIP-170 limits contract size to 24KB (24,576 bytes)
 *
 * @param bytecode - Contract bytecode
 * @returns Object with isValid flag and size in bytes
 */
export function validateContractSize(bytecode: string): {
  isValid: boolean;
  size: number;
  maxSize: number;
  utilizationPercent: number;
} {
  const cleanBytecode = bytecode.startsWith('0x') ? bytecode.slice(2) : bytecode;
  const size = cleanBytecode.length / 2;
  const maxSize = 24576; // 24KB limit

  return {
    isValid: size <= maxSize,
    size,
    maxSize,
    utilizationPercent: Math.round((size / maxSize) * 100),
  };
}

/**
 * Format bytecode size for display
 *
 * @param bytes - Size in bytes
 * @returns Formatted string (e.g., "12.5 KB")
 */
export function formatBytecodeSize(bytes: number): string {
  if (bytes < 1024) {
    return `${bytes} B`;
  } else {
    const kb = bytes / 1024;
    return `${kb.toFixed(2)} KB`;
  }
}

/**
 * Check if compilation result has errors
 */
export function hasCompilationErrors(result: CompilationResult): boolean {
  return !result.success || (result.errors !== undefined && result.errors.length > 0);
}

/**
 * Extract error messages from compilation result
 */
export function getErrorMessages(result: CompilationResult): string[] {
  if (!result.errors) return [];
  return result.errors.map(e => e.formattedMessage || e.message);
}

/**
 * Extract pragma version from source code
 */
export function extractPragmaVersion(source: string): string | null {
  const match = source.match(/pragma\s+solidity\s+([^;]+);/);
  if (match) {
    return match[1].trim();
  }
  return null;
}

/**
 * Check if a specific compiler version satisfies a pragma constraint
 */
export function versionSatisfiesPragma(version: string, pragma: string): boolean {
  // Simple implementation - would need semver library for full support
  // For now, assume version matches if it starts with same major.minor
  const versionParts = version.split('.');
  const pragmaClean = pragma.replace(/[\^~>=<]/g, '').trim();
  const pragmaParts = pragmaClean.split('.');

  if (pragmaParts.length >= 2 && versionParts.length >= 2) {
    return versionParts[0] === pragmaParts[0] && versionParts[1] === pragmaParts[1];
  }
  return true; // Default to allowing if we can't parse
}
