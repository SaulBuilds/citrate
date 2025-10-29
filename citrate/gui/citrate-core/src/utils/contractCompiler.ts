/**
 * Contract Compiler Utility
 *
 * Compiles Solidity smart contracts and returns bytecode, ABI, and metadata.
 *
 * IMPLEMENTATION OPTIONS:
 * 1. Browser: Use solc-js (requires npm install solc)
 * 2. Backend: Call Foundry CLI via Tauri (requires forge installed)
 *
 * Current: Placeholder implementation for development
 * TODO: Implement actual compilation using solc-js or Foundry
 */

export interface CompilationError {
  severity: 'error' | 'warning' | 'info';
  message: string;
  line?: number;
  column?: number;
  formattedMessage?: string;
}

export interface CompilationResult {
  success: boolean;
  bytecode?: string;
  abi?: any[];
  errors?: CompilationError[];
  warnings?: string[];
  gasEstimate?: number;
  contractSize?: number;
  metadata?: {
    compiler: string;
    language: string;
    output: any;
  };
}

/**
 * Compile a Solidity contract
 *
 * @param source - Solidity source code
 * @param contractName - Name of the contract to compile
 * @returns Compilation result with bytecode, ABI, or errors
 *
 * @example
 * ```typescript
 * const result = await compileContract(source, 'MyContract');
 * if (result.success) {
 *   console.log('Bytecode:', result.bytecode);
 *   console.log('ABI:', result.abi);
 * } else {
 *   console.error('Errors:', result.errors);
 * }
 * ```
 */
export async function compileContract(
  source: string,
  contractName: string
): Promise<CompilationResult> {
  console.log(`[Compiler] Compiling contract: ${contractName}`);

  try {
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

    // OPTION A: Use solc-js (browser compilation)
    // Uncomment and implement when solc is installed:
    /*
    const solc = await import('solc');
    const input = {
      language: 'Solidity',
      sources: {
        'Contract.sol': { content: source }
      },
      settings: {
        outputSelection: {
          '*': {
            '*': ['abi', 'evm.bytecode', 'evm.deployedBytecode', 'metadata']
          }
        },
        optimizer: {
          enabled: true,
          runs: 200
        }
      }
    };

    const output = JSON.parse(solc.compile(JSON.stringify(input)));

    if (output.errors && output.errors.length > 0) {
      const errors = output.errors.filter((e: any) => e.severity === 'error');
      const warnings = output.errors.filter((e: any) => e.severity === 'warning');

      if (errors.length > 0) {
        return {
          success: false,
          errors: errors.map((e: any) => ({
            severity: e.severity,
            message: e.message,
            formattedMessage: e.formattedMessage,
          })),
          warnings: warnings.map((w: any) => w.message),
        };
      }
    }

    const contract = output.contracts['Contract.sol'][contractName];
    const bytecode = '0x' + contract.evm.bytecode.object;
    const abi = contract.abi;
    const contractSize = bytecode.length / 2; // bytes

    return {
      success: true,
      bytecode,
      abi,
      gasEstimate: estimateDeploymentGas(bytecode),
      contractSize,
      metadata: {
        compiler: solc.version(),
        language: 'Solidity',
        output: contract.metadata,
      },
    };
    */

    // OPTION B: Call Tauri backend with Foundry
    // Uncomment and implement when Tauri command is ready:
    /*
    const { invoke } = await import('@tauri-apps/api/core');
    const result = await invoke<CompilationResult>('compile_contract', {
      source,
      contractName,
    });
    return result;
    */

    // PLACEHOLDER: Mock successful compilation for development
    // This allows UI development while compiler integration is pending
    console.warn('[Compiler] Using mock compilation (not real compilation!)');

    // Simulate compilation delay
    await new Promise(resolve => setTimeout(resolve, 500));

    // Check for obvious syntax errors (basic validation)
    const hasContract = source.includes('contract ' + contractName);
    if (!hasContract) {
      return {
        success: false,
        errors: [{
          severity: 'error',
          message: `Contract "${contractName}" not found in source code`,
          line: 1,
        }],
      };
    }

    // Mock successful compilation
    const mockBytecode = '0x608060405234801561001057600080fd5b50610150806100206000396000f3fe608060405234801561001057600080fd5b50600436106100365760003560e01c80632e64cec11461003b5780636057361d14610059575b600080fd5b610043610075565b60405161005091906100a1565b60405180910390f35b610073600480360381019061006e91906100ed565b61007e565b005b60008054905090565b8060008190555050565b6000819050919050565b61009b81610088565b82525050565b60006020820190506100b66000830184610092565b92915050565b600080fd5b6100ca81610088565b81146100d557600080fd5b50565b6000813590506100e7816100c1565b92915050565b600060208284031215610103576101026100bc565b5b6000610111848285016100d8565b9150509291505056fea264697066735822122064c33e7907e55b6b33c0c47e6a6f60e5f4c7b7c0d8e3c82c52d8c9c8e5e8e8f64736f6c63430008110033';

    const mockAbi = [
      {
        inputs: [],
        name: contractName,
        outputs: [],
        stateMutability: 'nonpayable',
        type: 'constructor',
      },
    ];

    const contractSize = mockBytecode.length / 2;

    return {
      success: true,
      bytecode: mockBytecode,
      abi: mockAbi,
      gasEstimate: estimateDeploymentGas(mockBytecode),
      contractSize,
      warnings: [
        '⚠️  Mock compilation - not real Solidity compilation!',
        'Install solc-js or configure Foundry backend for actual compilation.',
      ],
      metadata: {
        compiler: 'mock-0.8.17',
        language: 'Solidity',
        output: {},
      },
    };

  } catch (error: any) {
    console.error('[Compiler] Compilation failed:', error);
    return {
      success: false,
      errors: [{
        severity: 'error',
        message: error.message || 'Unknown compilation error',
      }],
    };
  }
}

/**
 * Estimate gas required for contract deployment
 *
 * Formula:
 * - 32,000 gas base cost
 * - 200 gas per byte of bytecode
 * - Additional 68 gas per byte for calldata (approx)
 *
 * @param bytecode - Contract bytecode (with or without '0x' prefix)
 * @returns Estimated gas
 */
export function estimateDeploymentGas(bytecode: string): number {
  // Remove '0x' prefix if present
  const cleanBytecode = bytecode.startsWith('0x') ? bytecode.slice(2) : bytecode;

  // Each byte is 2 hex characters
  const byteCount = cleanBytecode.length / 2;

  // Gas calculation
  const baseGas = 32000; // Transaction intrinsic cost
  const creationGas = 32000; // Contract creation cost
  const codeDepositGas = byteCount * 200; // Cost to store code
  const calldataGas = byteCount * 68; // Cost to send bytecode as calldata

  const totalGas = baseGas + creationGas + codeDepositGas + calldataGas;

  return Math.ceil(totalGas);
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
} {
  const cleanBytecode = bytecode.startsWith('0x') ? bytecode.slice(2) : bytecode;
  const size = cleanBytecode.length / 2; // Convert hex to bytes
  const maxSize = 24576; // 24KB limit

  return {
    isValid: size <= maxSize,
    size,
    maxSize,
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
 *
 * @param result - Compilation result
 * @returns True if there are errors
 */
export function hasCompilationErrors(result: CompilationResult): boolean {
  return !result.success || (result.errors !== undefined && result.errors.length > 0);
}

/**
 * Extract error messages from compilation result
 *
 * @param result - Compilation result
 * @returns Array of error messages
 */
export function getErrorMessages(result: CompilationResult): string[] {
  if (!result.errors) return [];
  return result.errors.map(e => e.formattedMessage || e.message);
}
