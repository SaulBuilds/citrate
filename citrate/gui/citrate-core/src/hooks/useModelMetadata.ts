/**
 * useModelMetadata Hook
 *
 * React hook for fetching and updating model metadata.
 * Features:
 * - Fetch metadata from IPFS
 * - Update metadata (owner only)
 * - Validate metadata
 * - Track ownership
 * - Handle loading and error states
 */

import { useState, useEffect, useCallback } from 'react';
import { EnrichedModelMetadata, ValidationResult, ModelSize } from '../utils/types/reviews';
import { ipfsMetadataFetcher, RawIPFSMetadata } from '../utils/search/ipfsMetadataFetcher';
import { validateModelMetadata } from '../utils/metadata/validation';
import { uploadMetadataToIPFS, ProgressCallback } from '../utils/metadata/ipfsUploader';
import { getMarketplaceService, ModelListing } from '../utils/marketplaceService';

// ============================================================================
// Types
// ============================================================================

export interface UseModelMetadataOptions {
  modelId: string;
  userAddress?: string; // Current user's address to check ownership
  autoFetch?: boolean;
}

export interface UseModelMetadataReturn {
  metadata: EnrichedModelMetadata | null;
  listing: ModelListing | null;
  isLoading: boolean;
  error: string | null;
  isOwner: boolean;
  validation: ValidationResult | null;

  // Actions
  updateMetadata: (
    newMetadata: EnrichedModelMetadata,
    password: string,
    onProgress?: ProgressCallback
  ) => Promise<string>;
  refreshMetadata: () => Promise<void>;
  validateMetadata: (metadata: EnrichedModelMetadata) => ValidationResult;
}

// ============================================================================
// Hook Implementation
// ============================================================================

export function useModelMetadata(options: UseModelMetadataOptions): UseModelMetadataReturn {
  const { modelId, userAddress, autoFetch = true } = options;

  const [metadata, setMetadata] = useState<EnrichedModelMetadata | null>(null);
  const [listing, setListing] = useState<ModelListing | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [validation, setValidation] = useState<ValidationResult | null>(null);

  /**
   * Check if current user is the model owner
   */
  const isOwner = useCallback(() => {
    if (!userAddress || !listing) return false;
    return listing.owner.toLowerCase() === userAddress.toLowerCase();
  }, [userAddress, listing]);

  /**
   * Fetch model listing and metadata
   */
  const fetchMetadata = useCallback(async () => {
    if (!modelId) return;

    setIsLoading(true);
    setError(null);

    try {
      const marketplace = getMarketplaceService();

      // Fetch on-chain listing
      const modelListing = await marketplace.getListing(modelId);

      if (!modelListing) {
        throw new Error('Model not found');
      }

      setListing(modelListing);

      // Fetch metadata from IPFS
      if (modelListing.metadataURI) {
        try {
          const rawMetadata = await ipfsMetadataFetcher.fetchMetadata(modelListing.metadataURI);

          // Convert raw metadata to EnrichedModelMetadata
          const enrichedMetadata = convertRawMetadata(rawMetadata, modelListing);
          setMetadata(enrichedMetadata);

          // Validate fetched metadata
          const validationResult = validateModelMetadata(enrichedMetadata);
          setValidation(validationResult);

          if (!validationResult.valid) {
            console.warn('[useModelMetadata] Fetched metadata has validation issues:', validationResult.errors);
          }
        } catch (ipfsError) {
          console.error('[useModelMetadata] Failed to fetch IPFS metadata:', ipfsError);
          // Create minimal metadata from on-chain data
          setMetadata({
            name: 'Unknown Model',
            description: 'Metadata could not be loaded from IPFS',
            category: modelListing.category,
            tags: [],
          });
          setError('Failed to load metadata from IPFS');
        }
      }
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to fetch model metadata';
      setError(errorMessage);
      console.error('[useModelMetadata] Failed to fetch metadata:', err);
    } finally {
      setIsLoading(false);
    }
  }, [modelId]);

  /**
   * Update model metadata
   */
  const updateMetadata = useCallback(
    async (
      newMetadata: EnrichedModelMetadata,
      password: string,
      onProgress?: ProgressCallback
    ): Promise<string> => {
      if (!userAddress) {
        throw new Error('User address is required to update metadata');
      }

      if (!isOwner()) {
        throw new Error('Only the model owner can update metadata');
      }

      try {
        // Validate metadata
        const validationResult = validateModelMetadata(newMetadata);
        if (!validationResult.valid) {
          const errors = validationResult.errors.map((e) => `${e.field}: ${e.message}`).join(', ');
          throw new Error(`Metadata validation failed: ${errors}`);
        }

        // Upload to IPFS
        const cid = await uploadMetadataToIPFS(newMetadata, onProgress);

        // Note: The current ModelMarketplace contract doesn't have an updateMetadata function.
        // The metadata URI can only be set during initial listing.
        // To update metadata, you would need to either:
        // 1. Add an updateMetadata function to the contract
        // 2. Delist and relist the model with new metadata
        // 3. Store the new IPFS URI off-chain and reference it

        // For now, we'll update the local state and return the new CID
        setMetadata(newMetadata);
        console.log('[useModelMetadata] Metadata uploaded to IPFS:', cid);
        console.log('[useModelMetadata] New IPFS URI:', `ipfs://${cid}`);
        console.warn('[useModelMetadata] On-chain metadata update not yet supported. New URI:', `ipfs://${cid}`);

        return cid;
      } catch (err) {
        const errorMessage = err instanceof Error ? err.message : 'Failed to update metadata';
        setError(errorMessage);
        throw new Error(errorMessage);
      }
    },
    [userAddress, isOwner]
  );

  /**
   * Validate metadata without uploading
   */
  const validateMetadataFn = useCallback((metadata: EnrichedModelMetadata): ValidationResult => {
    const result = validateModelMetadata(metadata);
    setValidation(result);
    return result;
  }, []);

  // Auto-fetch on mount
  useEffect(() => {
    if (autoFetch) {
      fetchMetadata();
    }
  }, [autoFetch, fetchMetadata]);

  return {
    metadata,
    listing,
    isLoading,
    error,
    isOwner: isOwner(),
    validation,
    updateMetadata,
    refreshMetadata: fetchMetadata,
    validateMetadata: validateMetadataFn,
  };
}

// ============================================================================
// Helper Functions
// ============================================================================

/**
 * Convert raw IPFS metadata to EnrichedModelMetadata
 */
function convertRawMetadata(
  raw: RawIPFSMetadata,
  listing: ModelListing
): EnrichedModelMetadata {
  return {
    name: raw.name,
    description: raw.description,
    category: listing.category,
    tags: raw.tags || [],
    framework: raw.framework,
    modelSize: raw.modelSize !== undefined ? categorizeSize(raw.modelSize) : undefined,
    sizeBytes: raw.modelSize,
    architecture: raw.architecture,
    version: raw.version,
    examples: [],
    supportedFormats: undefined,
    trainingDataset: raw.trainingData,
    pretrainedOn: raw.pretrainedOn,
    finetuneMethod: raw.finetuneMethod,
    license: raw.license,
    benchmarks: raw.benchmarks?.map((b) => ({
      metric: b.name,
      value: b.score,
      unit: b.metric,
    })),
    performanceMetrics: raw.performanceMetrics,
    lastUpdated: Date.now(),
    creator: raw.creator || {
      address: listing.owner,
    },
  };
}

/**
 * Categorize model size
 */
function categorizeSize(sizeBytes: number): ModelSize {
  const GB = 1024 * 1024 * 1024;

  if (sizeBytes < GB) return ModelSize.TINY;
  if (sizeBytes < 5 * GB) return ModelSize.SMALL;
  if (sizeBytes < 20 * GB) return ModelSize.MEDIUM;
  if (sizeBytes < 100 * GB) return ModelSize.LARGE;
  return ModelSize.XLARGE;
}

export default useModelMetadata;
