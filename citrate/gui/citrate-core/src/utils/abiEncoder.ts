/**
 * ABI Encoder/Decoder Utility
 *
 * Provides basic ABI encoding and decoding for contract function calls.
 * This is a simplified implementation that handles common types.
 * For production use with complex types (arrays, structs, tuples), use ethers.js
 *
 * Supported types:
 * - uint256, uint*, int256, int*
 * - address
 * - bool
 * - bytes32, bytes*
 * - string (basic support)
 */

import { keccak_256 } from '@noble/hashes/sha3';
import { bytesToHex, hexToBytes } from '@noble/hashes/utils';

/**
 * Encode function call data
 *
 * @param functionName - Function name
 * @param inputs - Function parameter types
 * @param args - Actual argument values
 * @returns Hex-encoded function call data
 *
 * @example
 * ```typescript
 * const data = encodeFunctionCall(
 *   'transfer',
 *   [{ type: 'address' }, { type: 'uint256' }],
 *   ['0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb', '1000000000000000000']
 * );
 * // Returns: 0xa9059cbb...
 * ```
 */
export function encodeFunctionCall(
  functionName: string,
  inputs: Array<{ type: string; name?: string }>,
  args: any[]
): string {
  if (inputs.length !== args.length) {
    throw new Error(
      `Expected ${inputs.length} arguments, got ${args.length}`
    );
  }

  // Calculate function selector (first 4 bytes of keccak256 hash of signature)
  const signature = `${functionName}(${inputs.map(i => i.type).join(',')})`;
  const signatureHash = keccak_256(new TextEncoder().encode(signature));
  const selector = bytesToHex(signatureHash.slice(0, 4));

  // Encode arguments
  const encodedArgs = encodeParameters(inputs.map(i => i.type), args);

  return '0x' + selector + encodedArgs;
}

/**
 * Encode constructor parameters
 *
 * @param types - Parameter types
 * @param values - Parameter values
 * @returns Hex-encoded parameters (without 0x prefix)
 */
export function encodeConstructorParams(
  types: string[],
  values: any[]
): string {
  return encodeParameters(types, values);
}

/**
 * Encode parameters for a function call or constructor
 *
 * @param types - Array of parameter types
 * @param values - Array of parameter values
 * @returns Hex-encoded parameters (without 0x prefix)
 */
export function encodeParameters(types: string[], values: any[]): string {
  if (types.length !== values.length) {
    throw new Error(`Type/value count mismatch: ${types.length} types, ${values.length} values`);
  }

  let encoded = '';
  let dynamicData = '';
  let dynamicOffset = types.length * 32; // Each static param is 32 bytes

  for (let i = 0; i < types.length; i++) {
    const type = types[i];
    const value = values[i];

    if (isStaticType(type)) {
      // Static types are encoded directly
      encoded += encodeValue(type, value);
    } else {
      // Dynamic types: encode offset, then add data at the end
      encoded += encodeUint(dynamicOffset);
      const dynamicEncoded = encodeDynamicValue(type, value);
      dynamicData += dynamicEncoded;
      dynamicOffset += dynamicEncoded.length / 2;
    }
  }

  return encoded + dynamicData;
}

/**
 * Check if a type is static (fixed size)
 */
function isStaticType(type: string): boolean {
  // Static types: uint*, int*, address, bool, bytes1-32
  if (type.startsWith('uint') || type.startsWith('int')) return true;
  if (type === 'address') return true;
  if (type === 'bool') return true;
  if (type.match(/^bytes\d+$/)) return true; // bytes1 to bytes32

  // Dynamic types: string, bytes, arrays
  return false;
}

/**
 * Encode a single static value
 */
function encodeValue(type: string, value: any): string {
  // uint256, uint*, int256, int*
  if (type.startsWith('uint') || type.startsWith('int')) {
    return encodeUint(value);
  }

  // address
  if (type === 'address') {
    return encodeAddress(value);
  }

  // bool
  if (type === 'bool') {
    return encodeBool(value);
  }

  // bytes32, bytes*, etc.
  if (type.match(/^bytes\d+$/)) {
    return encodeFixedBytes(value);
  }

  throw new Error(`Unsupported type: ${type}`);
}

/**
 * Encode a dynamic value (string, bytes, arrays)
 */
function encodeDynamicValue(type: string, value: any): string {
  if (type === 'string') {
    return encodeString(value);
  }

  if (type === 'bytes') {
    return encodeDynamicBytes(value);
  }

  // Arrays: type[]
  if (type.endsWith('[]')) {
    const baseType = type.slice(0, -2);
    return encodeArray(baseType, value);
  }

  throw new Error(`Unsupported dynamic type: ${type}`);
}

/**
 * Encode uint256 or other uint types
 */
function encodeUint(value: number | string | bigint): string {
  const bigValue = typeof value === 'bigint' ? value : BigInt(value.toString());
  const hex = bigValue.toString(16).padStart(64, '0');
  return hex;
}

/**
 * Encode address (20 bytes)
 */
function encodeAddress(value: string): string {
  const addr = value.toLowerCase().replace('0x', '');
  if (addr.length !== 40) {
    throw new Error(`Invalid address: ${value}`);
  }
  return addr.padStart(64, '0');
}

/**
 * Encode bool
 */
function encodeBool(value: boolean | string): string {
  const boolValue = typeof value === 'string' ? value === 'true' : value;
  return boolValue ? '0'.repeat(63) + '1' : '0'.repeat(64);
}

/**
 * Encode fixed-size bytes (bytes32, etc.)
 */
function encodeFixedBytes(value: string): string {
  const hex = value.replace('0x', '');
  return hex.padEnd(64, '0');
}

/**
 * Encode string (dynamic type)
 */
function encodeString(value: string): string {
  const bytes = new TextEncoder().encode(value);
  const length = encodeUint(bytes.length);
  const data = bytesToHex(bytes).padEnd(Math.ceil(bytes.length / 32) * 64, '0');
  return length + data;
}

/**
 * Encode dynamic bytes
 */
function encodeDynamicBytes(value: string): string {
  const hex = value.replace('0x', '');
  const length = encodeUint(hex.length / 2);
  const data = hex.padEnd(Math.ceil(hex.length / 64) * 64, '0');
  return length + data;
}

/**
 * Encode array
 */
function encodeArray(baseType: string, values: any[]): string {
  const length = encodeUint(values.length);
  const encoded = encodeParameters(
    Array(values.length).fill(baseType),
    values
  );
  return length + encoded;
}

/**
 * Decode function return value
 *
 * @param types - Return value types
 * @param data - Hex-encoded return data
 * @returns Decoded values
 *
 * @example
 * ```typescript
 * const result = decodeFunctionResult(
 *   ['uint256'],
 *   '0x000000000000000000000000000000000000000000000000000000000000002a'
 * );
 * // Returns: [42n]
 * ```
 */
export function decodeFunctionResult(types: string[], data: string): any[] {
  const cleanData = data.replace('0x', '');

  if (types.length === 0) return [];
  if (cleanData.length === 0) return [];

  const results: any[] = [];
  let offset = 0;

  for (const type of types) {
    if (isStaticType(type)) {
      const value = decodeValue(type, cleanData.slice(offset, offset + 64));
      results.push(value);
      offset += 64;
    } else {
      // Dynamic type: read offset, then decode from that position
      const dynamicOffset = parseInt(cleanData.slice(offset, offset + 64), 16) * 2;
      const value = decodeDynamicValue(type, cleanData.slice(dynamicOffset));
      results.push(value);
      offset += 64;
    }
  }

  return results;
}

/**
 * Decode a static value
 */
function decodeValue(type: string, hex: string): any {
  if (type.startsWith('uint') || type.startsWith('int')) {
    return BigInt('0x' + hex);
  }

  if (type === 'address') {
    return '0x' + hex.slice(-40);
  }

  if (type === 'bool') {
    return hex.slice(-1) === '1';
  }

  if (type.match(/^bytes\d+$/)) {
    return '0x' + hex;
  }

  throw new Error(`Unsupported decode type: ${type}`);
}

/**
 * Decode a dynamic value
 */
function decodeDynamicValue(type: string, hex: string): any {
  if (type === 'string') {
    const length = parseInt(hex.slice(0, 64), 16);
    const data = hex.slice(64, 64 + length * 2);
    return new TextDecoder().decode(hexToBytes(data));
  }

  if (type === 'bytes') {
    const length = parseInt(hex.slice(0, 64), 16);
    return '0x' + hex.slice(64, 64 + length * 2);
  }

  // Arrays
  if (type.endsWith('[]')) {
    const baseType = type.slice(0, -2);
    const arrayLength = parseInt(hex.slice(0, 64), 16);
    const values: any[] = [];

    let offset = 64;
    for (let i = 0; i < arrayLength; i++) {
      if (isStaticType(baseType)) {
        values.push(decodeValue(baseType, hex.slice(offset, offset + 64)));
        offset += 64;
      } else {
        // Complex dynamic types in arrays not fully supported yet
        throw new Error(`Decoding dynamic type arrays not fully supported: ${type}`);
      }
    }

    return values;
  }

  throw new Error(`Unsupported dynamic decode type: ${type}`);
}

/**
 * Get function selector (first 4 bytes of function signature hash)
 *
 * @param functionName - Function name
 * @param inputs - Parameter types
 * @returns Function selector (0x...)
 */
export function getFunctionSelector(
  functionName: string,
  inputs: string[]
): string {
  const signature = `${functionName}(${inputs.join(',')})`;
  const signatureHash = keccak_256(new TextEncoder().encode(signature));
  return '0x' + bytesToHex(signatureHash.slice(0, 4));
}
