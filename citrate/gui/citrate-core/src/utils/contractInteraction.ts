/**
 * Contract Interaction Utility
 *
 * Provides high-level functions for interacting with smart contracts:
 * - Call read functions (eth_call)
 * - Execute write functions (eth_sendTransaction)
 * - Encode/decode function calls
 */

import { invoke } from '@tauri-apps/api/core';
import {
  encodeFunctionCall,
  decodeFunctionResult,
  encodeConstructorParams,
} from './abiEncoder';

export interface CallFunctionRequest {
  contractAddress: string;
  functionName: string;
  inputs: Array<{ name: string; type: string }>;
  outputs: Array<{ name: string; type: string }>;
  args: any[];
}

export interface SendTransactionRequest {
  from: string;
  contractAddress: string;
  functionName: string;
  inputs: Array<{ name: string; type: string }>;
  args: any[];
  value?: string;
  gasLimit?: number;
  gasPrice?: string;
  password: string;
}

export interface CallFunctionResult {
  success: boolean;
  outputs: any[];
  decodedOutputs?: Record<string, any>;
}

export interface SendTransactionResult {
  success: boolean;
  txHash: string;
}

/**
 * Call a read-only contract function (view/pure)
 *
 * Uses eth_call to execute without sending a transaction
 *
 * @param request - Function call parameters
 * @returns Decoded function outputs
 *
 * @example
 * ```typescript
 * const result = await callContractFunction({
 *   contractAddress: '0x...',
 *   functionName: 'balanceOf',
 *   inputs: [{ name: 'account', type: 'address' }],
 *   outputs: [{ name: 'balance', type: 'uint256' }],
 *   args: ['0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb'],
 * });
 * console.log('Balance:', result.outputs[0]);
 * ```
 */
export async function callContractFunction(
  request: CallFunctionRequest
): Promise<CallFunctionResult> {
  try {
    console.log('[ContractInteraction] Calling function:', request.functionName);

    // Encode function call
    const data = encodeFunctionCall(
      request.functionName,
      request.inputs,
      request.args
    );

    console.log('[ContractInteraction] Encoded call data:', data);

    // Make eth_call via Tauri backend
    const returnData = await invoke<string>('eth_call', {
      request: {
        to: request.contractAddress,
        data,
        from: null // Optional: could pass from address if needed
      }
    });

    console.log('[ContractInteraction] eth_call returned:', returnData);

    // Decode return data
    const outputTypes = request.outputs.map(o => o.type);
    const decodedOutputs = outputTypes.length > 0
      ? decodeFunctionResult(outputTypes, returnData)
      : [];

    // Create named outputs object
    const namedOutputs: Record<string, any> = {};
    request.outputs.forEach((output, index) => {
      if (output.name) {
        namedOutputs[output.name] = decodedOutputs[index];
      }
    });

    return {
      success: true,
      outputs: decodedOutputs,
      decodedOutputs: namedOutputs,
    };
  } catch (error: any) {
    console.error('[ContractInteraction] Call failed:', error);
    throw new Error(`Function call failed: ${error.message}`);
  }
}

/**
 * Send a transaction to execute a contract function
 *
 * Uses eth_sendTransaction to modify blockchain state
 *
 * @param request - Transaction parameters
 * @returns Transaction hash
 *
 * @example
 * ```typescript
 * const result = await sendContractTransaction({
 *   from: '0x...',
 *   contractAddress: '0x...',
 *   functionName: 'transfer',
 *   inputs: [
 *     { name: 'to', type: 'address' },
 *     { name: 'amount', type: 'uint256' }
 *   ],
 *   args: ['0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb', '1000000000000000000'],
 *   password: 'user-password',
 * });
 * console.log('Transaction sent:', result.txHash);
 * ```
 */
export async function sendContractTransaction(
  request: SendTransactionRequest
): Promise<SendTransactionResult> {
  try {
    console.log('[ContractInteraction] Sending transaction:', request.functionName);

    // Encode function call
    const data = encodeFunctionCall(
      request.functionName,
      request.inputs,
      request.args
    );

    console.log('[ContractInteraction] Encoded transaction data:', data);

    // Prepare transaction request
    const txRequest = {
      from: request.from,
      to: request.contractAddress,
      value: request.value || '0',
      gasLimit: request.gasLimit || 500000,
      gasPrice: request.gasPrice || '1000000000',
      data,
    };

    // Send transaction via Tauri
    const txHash = await invoke<string>('send_transaction', {
      request: txRequest,
      password: request.password,
    });

    console.log('[ContractInteraction] Transaction sent:', txHash);

    return {
      success: true,
      txHash,
    };
  } catch (error: any) {
    console.error('[ContractInteraction] Transaction failed:', error);
    throw new Error(`Transaction failed: ${error.message}`);
  }
}

/**
 * Encode constructor parameters for contract deployment
 *
 * @param inputs - Constructor parameter types
 * @param args - Constructor argument values
 * @returns Hex-encoded parameters
 */
export function encodeConstructorArguments(
  inputs: Array<{ type: string }>,
  args: any[]
): string {
  const types = inputs.map(i => i.type);
  return encodeConstructorParams(types, args);
}

/**
 * Format function output for display
 *
 * @param outputs - Function output definitions
 * @param values - Decoded output values
 * @returns Formatted output string
 */
export function formatFunctionOutput(
  outputs: Array<{ name: string; type: string }>,
  values: any[]
): string {
  if (outputs.length === 0) return 'Success (no return value)';
  if (outputs.length === 1 && !outputs[0].name) {
    return formatValue(outputs[0].type, values[0]);
  }

  const formatted = outputs.map((output, index) => {
    const name = output.name || `output${index}`;
    const value = formatValue(output.type, values[index]);
    return `${name}: ${value}`;
  });

  return formatted.join('\n');
}

/**
 * Format a single value for display
 */
function formatValue(type: string, value: any): string {
  if (type.startsWith('uint') || type.startsWith('int')) {
    return value.toString();
  }

  if (type === 'address') {
    return value.toString();
  }

  if (type === 'bool') {
    return value ? 'true' : 'false';
  }

  if (type === 'string') {
    return `"${value}"`;
  }

  if (type === 'bytes' || type.startsWith('bytes')) {
    return value.toString();
  }

  if (Array.isArray(value)) {
    return `[${value.map(v => formatValue(type.replace('[]', ''), v)).join(', ')}]`;
  }

  return JSON.stringify(value);
}

/**
 * Parse user input for a specific type
 *
 * @param type - Solidity type
 * @param input - User input string
 * @returns Parsed value
 */
export function parseUserInput(type: string, input: string): any {
  if (!input || input.trim() === '') {
    throw new Error(`Value required for type ${type}`);
  }

  if (type.startsWith('uint') || type.startsWith('int')) {
    // Handle large numbers
    return BigInt(input);
  }

  if (type === 'address') {
    if (!input.startsWith('0x') || input.length !== 42) {
      throw new Error('Invalid address format');
    }
    return input.toLowerCase();
  }

  if (type === 'bool') {
    const lower = input.toLowerCase().trim();
    if (lower === 'true' || lower === '1') return true;
    if (lower === 'false' || lower === '0') return false;
    throw new Error('Invalid boolean value (use true/false or 1/0)');
  }

  if (type === 'string') {
    return input;
  }

  if (type === 'bytes' || type.startsWith('bytes')) {
    if (!input.startsWith('0x')) {
      throw new Error('Bytes value must start with 0x');
    }
    return input;
  }

  // Arrays: parse JSON
  if (type.endsWith('[]')) {
    try {
      return JSON.parse(input);
    } catch {
      throw new Error('Invalid array format (use JSON array syntax)');
    }
  }

  return input;
}
