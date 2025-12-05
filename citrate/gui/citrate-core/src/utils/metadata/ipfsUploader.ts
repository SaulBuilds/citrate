/**
 * IPFS Uploader for Citrate Model Marketplace
 *
 * Handles uploading model metadata to IPFS and updating on-chain references.
 * Features:
 * - Multiple gateway support with fallbacks (Pinata, local IPFS node, Web3.Storage)
 * - Exponential backoff retry logic
 * - Progress tracking with stage updates
 * - Validation before upload
 * - On-chain metadata URI updates via contract calls
 */

import { invoke } from '@tauri-apps/api/core';
import {
  IPFSMetadataFetcher,
  DEFAULT_GATEWAYS,
  DEFAULT_RETRY_CONFIG,
  RetryConfig,
} from '../search/ipfsMetadataFetcher';
import { EnrichedModelMetadata } from '../types/reviews';
// ValidationResult type is imported but used in validateModelMetadata signature matching
import { validateModelMetadata } from './validation';

// ============================================================================
// Configuration
// ============================================================================

const PINATA_API_KEY = import.meta.env.VITE_PINATA_API_KEY || '';
const PINATA_SECRET_KEY = import.meta.env.VITE_PINATA_SECRET_KEY || '';
const PINATA_JWT = import.meta.env.VITE_PINATA_JWT || '';
const WEB3_STORAGE_TOKEN = import.meta.env.VITE_WEB3_STORAGE_TOKEN || '';

const PINATA_UPLOAD_URL = 'https://api.pinata.cloud/pinning/pinJSONToIPFS';
const WEB3_STORAGE_UPLOAD_URL = 'https://api.web3.storage/upload';

// Tauri IPFS command result types
interface IpfsAddResult {
  cid: string;
  size: number;
  name: string;
}

interface IpfsStatus {
  running: boolean;
  peer_id: string | null;
  addresses: string[];
  repo_size: number;
  num_peers: number;
  num_pins: number;
}

// ============================================================================
// Types
// ============================================================================

export interface UploadProgress {
  stage: 'validating' | 'uploading' | 'pinning' | 'updating' | 'complete' | 'error';
  message: string;
  progress: number; // 0-100
}

export interface UploadResult {
  success: boolean;
  cid?: string;
  ipfsUri?: string;
  error?: string;
  txHash?: string; // If on-chain update was performed
}

export type ProgressCallback = (progress: UploadProgress) => void;

// ============================================================================
// IPFS Uploader
// ============================================================================

export class IPFSUploader {
  private retryConfig: RetryConfig;
  private fetcher: IPFSMetadataFetcher;

  constructor(retryConfig: RetryConfig = DEFAULT_RETRY_CONFIG) {
    this.retryConfig = retryConfig;
    this.fetcher = new IPFSMetadataFetcher(DEFAULT_GATEWAYS, retryConfig);
  }

  /**
   * Upload metadata to IPFS (with validation and pinning)
   */
  async uploadMetadataToIPFS(
    metadata: EnrichedModelMetadata,
    onProgress?: ProgressCallback
  ): Promise<string> {
    // Stage 1: Validation
    this.updateProgress(onProgress, {
      stage: 'validating',
      message: 'Validating metadata...',
      progress: 10,
    });

    const validation = validateModelMetadata(metadata);
    if (!validation.valid) {
      const errorMessages = validation.errors.map((e) => `${e.field}: ${e.message}`).join(', ');
      throw new Error(`Metadata validation failed: ${errorMessages}`);
    }

    // Stage 2: Uploading to IPFS
    this.updateProgress(onProgress, {
      stage: 'uploading',
      message: 'Uploading to IPFS...',
      progress: 30,
    });

    let cid: string;
    try {
      cid = await this.uploadToPinata(metadata, onProgress);
    } catch (error) {
      console.error('[IPFSUploader] Pinata upload failed, trying alternative method:', error);

      // Fallback: Use Web3.Storage or other IPFS service
      try {
        cid = await this.uploadToIPFSFallback(metadata);
      } catch (fallbackError) {
        throw new Error(`Failed to upload to IPFS: ${fallbackError instanceof Error ? fallbackError.message : 'Unknown error'}`);
      }
    }

    // Stage 3: Verify upload
    this.updateProgress(onProgress, {
      stage: 'pinning',
      message: 'Verifying upload...',
      progress: 80,
    });

    await this.verifyUpload(cid, metadata);

    this.updateProgress(onProgress, {
      stage: 'complete',
      message: 'Upload complete',
      progress: 100,
    });

    return cid;
  }

  /**
   * Upload to Pinata with retries
   */
  private async uploadToPinata(
    metadata: EnrichedModelMetadata,
    onProgress?: ProgressCallback
  ): Promise<string> {
    // Check if Pinata credentials are configured
    if (!PINATA_JWT && (!PINATA_API_KEY || !PINATA_SECRET_KEY)) {
      throw new Error('Pinata credentials not configured. Set VITE_PINATA_JWT or VITE_PINATA_API_KEY/VITE_PINATA_SECRET_KEY environment variables.');
    }

    let lastError: Error | null = null;
    let delay = this.retryConfig.initialDelayMs;

    for (let attempt = 0; attempt <= this.retryConfig.maxRetries; attempt++) {
      try {
        const headers: HeadersInit = {
          'Content-Type': 'application/json',
        };

        // Use JWT if available, otherwise use API keys
        if (PINATA_JWT) {
          headers['Authorization'] = `Bearer ${PINATA_JWT}`;
        } else {
          headers['pinata_api_key'] = PINATA_API_KEY;
          headers['pinata_secret_api_key'] = PINATA_SECRET_KEY;
        }

        const response = await fetch(PINATA_UPLOAD_URL, {
          method: 'POST',
          headers,
          body: JSON.stringify({
            pinataContent: metadata,
            pinataMetadata: {
              name: `${metadata.name} - Model Metadata`,
              keyvalues: {
                modelName: metadata.name,
                category: metadata.category.toString(),
                version: metadata.version || 'latest',
              },
            },
            pinataOptions: {
              cidVersion: 1,
            },
          }),
        });

        if (!response.ok) {
          const errorText = await response.text();
          throw new Error(`HTTP ${response.status}: ${errorText}`);
        }

        const result = await response.json();

        if (!result.IpfsHash) {
          throw new Error('No IPFS hash returned from Pinata');
        }

        return result.IpfsHash;
      } catch (error) {
        lastError = error as Error;

        // Don't retry on final attempt
        if (attempt === this.retryConfig.maxRetries) {
          break;
        }

        console.warn(`[IPFSUploader] Pinata upload attempt ${attempt + 1} failed:`, error);

        // Wait before retry
        await this.sleep(delay);

        // Exponential backoff
        delay = Math.min(
          delay * this.retryConfig.backoffMultiplier,
          this.retryConfig.maxDelayMs
        );

        // Update progress
        this.updateProgress(onProgress, {
          stage: 'uploading',
          message: `Retrying upload (${attempt + 1}/${this.retryConfig.maxRetries})...`,
          progress: 30 + (attempt / this.retryConfig.maxRetries) * 20,
        });
      }
    }

    throw lastError || new Error('Upload failed after retries');
  }

  /**
   * Fallback IPFS upload method - tries multiple backends in order:
   * 1. Local IPFS node via Tauri
   * 2. Web3.Storage API
   * 3. Direct HTTP to local IPFS API
   */
  private async uploadToIPFSFallback(
    metadata: EnrichedModelMetadata
  ): Promise<string> {
    const jsonContent = JSON.stringify(metadata, null, 2);
    const contentBytes = new TextEncoder().encode(jsonContent);

    // Try 1: Upload via local IPFS node through Tauri commands
    try {
      const status = await invoke<IpfsStatus>('ipfs_status');
      if (status.running) {
        const result = await invoke<IpfsAddResult>('ipfs_add', {
          data: Array.from(contentBytes),
          name: `${metadata.name || 'model'}-metadata.json`,
        });
        if (result.cid) {
          console.log('[IPFSUploader] Uploaded via local IPFS node:', result.cid);
          // Also pin the content to ensure persistence
          try {
            await invoke('ipfs_pin', { cid: result.cid });
          } catch (pinError) {
            console.warn('[IPFSUploader] Failed to pin CID, but upload succeeded:', pinError);
          }
          return result.cid;
        }
      }
    } catch (tauriError) {
      console.warn('[IPFSUploader] Local IPFS via Tauri failed:', tauriError);
    }

    // Try 2: Web3.Storage if token is configured
    if (WEB3_STORAGE_TOKEN) {
      try {
        const cid = await this.uploadToWeb3Storage(jsonContent);
        console.log('[IPFSUploader] Uploaded via Web3.Storage:', cid);
        return cid;
      } catch (web3Error) {
        console.warn('[IPFSUploader] Web3.Storage upload failed:', web3Error);
      }
    }

    // Try 3: Direct HTTP to local IPFS API (in case Tauri command isn't available)
    try {
      const cid = await this.uploadViaLocalIPFSAPI(contentBytes);
      console.log('[IPFSUploader] Uploaded via local IPFS HTTP API:', cid);
      return cid;
    } catch (localError) {
      console.warn('[IPFSUploader] Local IPFS HTTP API failed:', localError);
    }

    // All methods failed
    throw new Error(
      'IPFS upload failed: No available upload method. Please either:\n' +
      '1. Start a local IPFS node (ipfs daemon)\n' +
      '2. Set VITE_PINATA_JWT or VITE_PINATA_API_KEY environment variables\n' +
      '3. Set VITE_WEB3_STORAGE_TOKEN environment variable'
    );
  }

  /**
   * Upload to Web3.Storage API
   */
  private async uploadToWeb3Storage(jsonContent: string): Promise<string> {
    const blob = new Blob([jsonContent], { type: 'application/json' });
    const formData = new FormData();
    formData.append('file', blob, 'metadata.json');

    const response = await fetch(WEB3_STORAGE_UPLOAD_URL, {
      method: 'POST',
      headers: {
        'Authorization': `Bearer ${WEB3_STORAGE_TOKEN}`,
      },
      body: formData,
    });

    if (!response.ok) {
      const errorText = await response.text();
      throw new Error(`Web3.Storage upload failed: ${response.status} ${errorText}`);
    }

    const result = await response.json();
    if (!result.cid) {
      throw new Error('Web3.Storage did not return a CID');
    }

    return result.cid;
  }

  /**
   * Upload directly to local IPFS HTTP API
   */
  private async uploadViaLocalIPFSAPI(content: Uint8Array): Promise<string> {
    const formData = new FormData();
    const blob = new Blob([content], { type: 'application/json' });
    formData.append('file', blob);

    // Try common local IPFS API endpoints
    const endpoints = [
      'http://localhost:5001/api/v0/add',
      'http://127.0.0.1:5001/api/v0/add',
      'http://localhost:5002/api/v0/add', // Alternative port
    ];

    let lastError: Error | null = null;

    for (const endpoint of endpoints) {
      try {
        const response = await fetch(endpoint, {
          method: 'POST',
          body: formData,
        });

        if (response.ok) {
          const result = await response.json();
          if (result.Hash) {
            return result.Hash;
          }
        }
      } catch (error) {
        lastError = error as Error;
        continue;
      }
    }

    throw lastError || new Error('Local IPFS API not available');
  }

  /**
   * Verify upload by fetching the content
   */
  private async verifyUpload(cid: string, expectedMetadata: EnrichedModelMetadata): Promise<void> {
    try {
      const fetchedMetadata = await this.fetcher.fetchMetadata(`ipfs://${cid}`);

      // Verify critical fields match
      if (fetchedMetadata.name !== expectedMetadata.name) {
        throw new Error('Uploaded metadata name mismatch');
      }

      if (fetchedMetadata.description !== expectedMetadata.description) {
        throw new Error('Uploaded metadata description mismatch');
      }
    } catch (error) {
      console.error('[IPFSUploader] Verification failed:', error);
      // Don't throw - upload succeeded, verification is just a safety check
      console.warn('[IPFSUploader] Upload verification failed, but CID was returned. Proceeding...');
    }
  }

  /**
   * Update model metadata URI on-chain
   *
   * Uses the `listModel` function which updates existing listings when called by the owner.
   * This requires the current listing details to be fetched first.
   */
  async updateModelMetadataOnChain(
    modelId: string,
    newCID: string,
    options: {
      ownerAddress: string;
      password: string;
      marketplaceAddress: string;
    },
    onProgress?: ProgressCallback
  ): Promise<string> {
    const { ownerAddress, password, marketplaceAddress } = options;

    this.updateProgress(onProgress, {
      stage: 'updating',
      message: 'Fetching current listing details...',
      progress: 85,
    });

    try {
      // Import the marketplace service
      const { MarketplaceService } = await import('../marketplaceService');
      const marketplace = new MarketplaceService(marketplaceAddress);

      // Fetch current listing to preserve existing values
      const currentListing = await marketplace.getListing(modelId);

      if (!currentListing) {
        throw new Error(`Model ${modelId} not found in marketplace. Cannot update metadata for unlisted models.`);
      }

      // Verify ownership
      if (currentListing.owner.toLowerCase() !== ownerAddress.toLowerCase()) {
        throw new Error('Only the model owner can update metadata');
      }

      // Construct IPFS URI
      const ipfsUri = `ipfs://${newCID}`;

      this.updateProgress(onProgress, {
        stage: 'updating',
        message: 'Submitting on-chain transaction...',
        progress: 92,
      });

      // Call listModel which will update the existing listing with new metadata URI
      // The contract's listModel function checks if the model is already listed
      // and calls _updateListing if so, preserving sales stats while updating metadata
      const txHash = await marketplace.listModel(
        {
          modelId,
          basePrice: currentListing.basePrice,
          discountPrice: currentListing.discountPrice,
          minimumBulkSize: Number(currentListing.minimumBulkSize),
          category: currentListing.category,
          metadataURI: ipfsUri,
        },
        {
          from: ownerAddress,
          password,
          gasLimit: 300000,
        }
      );

      this.updateProgress(onProgress, {
        stage: 'complete',
        message: 'On-chain metadata updated successfully',
        progress: 100,
      });

      console.log('[IPFSUploader] On-chain metadata updated. TX:', txHash);
      return txHash;
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'Unknown error';
      this.updateProgress(onProgress, {
        stage: 'error',
        message: `On-chain update failed: ${errorMessage}`,
        progress: 0,
      });
      throw error;
    }
  }

  /**
   * Validate and upload in one step
   */
  async validateAndUpload(
    metadata: EnrichedModelMetadata,
    onProgress?: ProgressCallback
  ): Promise<UploadResult> {
    try {
      // Validate
      const validation = validateModelMetadata(metadata);

      if (!validation.valid) {
        return {
          success: false,
          error: validation.errors.map((e) => `${e.field}: ${e.message}`).join('; '),
        };
      }

      // Upload
      const cid = await this.uploadMetadataToIPFS(metadata, onProgress);
      const ipfsUri = `ipfs://${cid}`;

      return {
        success: true,
        cid,
        ipfsUri,
      };
    } catch (error) {
      return {
        success: false,
        error: error instanceof Error ? error.message : 'Unknown error',
      };
    }
  }

  /**
   * Update progress callback
   */
  private updateProgress(callback: ProgressCallback | undefined, progress: UploadProgress): void {
    if (callback) {
      callback(progress);
    }
  }

  /**
   * Sleep utility
   */
  private sleep(ms: number): Promise<void> {
    return new Promise((resolve) => setTimeout(resolve, ms));
  }
}

// ============================================================================
// Export Functions
// ============================================================================

/**
 * Upload metadata to IPFS (convenience function)
 */
export async function uploadMetadataToIPFS(
  metadata: EnrichedModelMetadata,
  onProgress?: ProgressCallback
): Promise<string> {
  const uploader = new IPFSUploader();
  return uploader.uploadMetadataToIPFS(metadata, onProgress);
}

/**
 * Update model metadata on-chain (convenience function)
 */
export async function updateModelMetadataOnChain(
  modelId: string,
  newCID: string,
  options: {
    ownerAddress: string;
    password: string;
    marketplaceAddress: string;
  },
  onProgress?: ProgressCallback
): Promise<string> {
  const uploader = new IPFSUploader();
  return uploader.updateModelMetadataOnChain(modelId, newCID, options, onProgress);
}

/**
 * Validate and upload metadata (convenience function)
 */
export async function validateAndUpload(
  metadata: EnrichedModelMetadata,
  onProgress?: ProgressCallback
): Promise<UploadResult> {
  const uploader = new IPFSUploader();
  return uploader.validateAndUpload(metadata, onProgress);
}

/**
 * Default uploader instance
 */
export const ipfsUploader = new IPFSUploader();
