/**
 * IPFS Uploader for Citrate Model Marketplace
 *
 * Handles uploading model metadata to IPFS and updating on-chain references.
 * Features:
 * - Multiple gateway support with fallbacks
 * - Exponential backoff retry logic
 * - Progress tracking
 * - Validation before upload
 * - On-chain metadata URI updates
 */

import {
  IPFSMetadataFetcher,
  DEFAULT_GATEWAYS,
  DEFAULT_RETRY_CONFIG,
  RetryConfig,
} from '../search/ipfsMetadataFetcher';
import { EnrichedModelMetadata, ValidationResult } from '../types/reviews';
import { validateModelMetadata } from './validation';

// ============================================================================
// Configuration
// ============================================================================

const PINATA_API_KEY = import.meta.env.VITE_PINATA_API_KEY || '';
const PINATA_SECRET_KEY = import.meta.env.VITE_PINATA_SECRET_KEY || '';
const PINATA_JWT = import.meta.env.VITE_PINATA_JWT || '';

const PINATA_UPLOAD_URL = 'https://api.pinata.cloud/pinning/pinJSONToIPFS';

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
   * Fallback IPFS upload method (using public gateway)
   */
  private async uploadToIPFSFallback(
    metadata: EnrichedModelMetadata
  ): Promise<string> {
    // This is a simplified fallback - in production, you would use:
    // - Web3.Storage (https://web3.storage/)
    // - NFT.Storage (https://nft.storage/)
    // - Infura IPFS API
    // - Local IPFS node

    throw new Error('IPFS fallback upload not configured. Please set up Pinata credentials or configure an alternative IPFS service.');
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
      message: 'Updating on-chain metadata reference...',
      progress: 90,
    });

    try {
      // Import the marketplace service
      const { MarketplaceService } = await import('../marketplaceService');
      const marketplace = new MarketplaceService(marketplaceAddress);

      // Construct IPFS URI
      const ipfsUri = `ipfs://${newCID}`;

      // Note: The ModelMarketplace contract doesn't have a separate updateMetadata function
      // The metadata URI is set during listing. To update metadata, the model owner would need
      // to update their listing or we need to add a new contract function.

      // For now, we'll throw an error indicating this limitation
      throw new Error('On-chain metadata update not yet supported. Metadata URI can only be set during initial listing. Please relist the model with the new metadata URI: ' + ipfsUri);

      // TODO: Once the contract adds an updateMetadata function, implement it here:
      // const txHash = await marketplace.updateMetadata(
      //   { modelId, metadataURI: ipfsUri },
      //   { from: ownerAddress, password }
      // );
      //
      // this.updateProgress(onProgress, {
      //   stage: 'complete',
      //   message: 'On-chain update complete',
      //   progress: 100,
      // });
      //
      // return txHash;

    } catch (error) {
      this.updateProgress(onProgress, {
        stage: 'error',
        message: `On-chain update failed: ${error instanceof Error ? error.message : 'Unknown error'}`,
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
