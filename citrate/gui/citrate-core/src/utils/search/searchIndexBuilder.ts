/**
 * Search Index Builder for Citrate Model Marketplace
 *
 * Builds and maintains an in-memory search index by:
 * - Fetching all marketplace listings from the blockchain
 * - Retrieving IPFS metadata for each model
 * - Listening for contract events (ModelListed, ModelUpdated, ModelDelisted)
 * - Updating the index in real-time with debouncing
 * - Providing index statistics and health metrics
 */

import { ethers } from 'ethers';
import {
  SearchDocument,
  SearchIndex,
  IndexUpdateEvent,
  SearchStatistics,
  ModelCategory,
  IndexBuilderOptions
} from './types';
import { IPFSMetadataFetcher, RawIPFSMetadata } from './ipfsMetadataFetcher';

// ============================================================================
// Contract ABIs (simplified - only events/methods we need)
// ============================================================================

const MARKETPLACE_ABI = [
  'event ModelListed(bytes32 indexed modelHash, address indexed owner, uint256 basePrice, uint256 discountPrice, uint8 category)',
  'event ModelPriceUpdated(bytes32 indexed modelHash, uint256 newBasePrice, uint256 newDiscountPrice)',
  'event ModelDelisted(bytes32 indexed modelHash)',
  'event ModelFeatured(bytes32 indexed modelHash, address indexed sponsor)',
  'event ModelPurchased(bytes32 indexed modelHash, address indexed buyer, uint256 price)',
  'event ReviewSubmitted(bytes32 indexed modelHash, address indexed reviewer, uint8 rating, string reviewText)',

  'function getListingInfo(bytes32 modelHash) view returns (tuple(address owner, uint256 basePrice, uint256 discountPrice, uint256 minBulkSize, uint8 category, bool isActive, bool isFeatured, uint256 totalSales, string metadataURI))',
  'function getReviewStats(bytes32 indexed modelHash) view returns (tuple(uint256 totalRatings, uint256 sumRatings, uint256 reviewCount))',
  'function getAllListings() view returns (bytes32[] memory)'
];

const REGISTRY_ABI = [
  'function getModelInfo(bytes32 modelHash) view returns (tuple(string name, string framework, string version, uint256 sizeBytes, address owner, uint256 timestamp, bool isActive))',
  'function getModelMetadataURI(bytes32 modelHash) view returns (string)',
  'function totalModels() view returns (uint256)'
];

// ============================================================================
// On-chain data types
// ============================================================================

interface ListingInfo {
  owner: string;
  basePrice: bigint;
  discountPrice: bigint;
  minBulkSize: bigint;
  category: number;
  isActive: boolean;
  isFeatured: boolean;
  totalSales: bigint;
  metadataURI: string;
}

interface ReviewStats {
  totalRatings: bigint;
  sumRatings: bigint;
  reviewCount: bigint;
}

interface ModelInfo {
  name: string;
  framework: string;
  version: string;
  sizeBytes: bigint;
  owner: string;
  timestamp: bigint;
  isActive: boolean;
}

// ============================================================================
// Search Index Builder
// ============================================================================

export class SearchIndexBuilder {
  private provider: ethers.Provider;
  private marketplaceContract: ethers.Contract;
  private registryContract: ethers.Contract;
  private metadataFetcher: IPFSMetadataFetcher;
  private options: Required<IndexBuilderOptions>;

  private index: SearchIndex;
  private isInitialized: boolean = false;
  private isBuilding: boolean = false;
  private updateQueue: Set<string> = new Set();
  private debounceTimer: NodeJS.Timeout | null = null;
  private eventListeners: Map<string, ethers.ContractEventName> = new Map();

  constructor(
    provider: ethers.Provider,
    marketplaceAddress: string,
    registryAddress: string,
    metadataFetcher?: IPFSMetadataFetcher,
    options?: IndexBuilderOptions
  ) {
    this.provider = provider;
    this.marketplaceContract = new ethers.Contract(
      marketplaceAddress,
      MARKETPLACE_ABI,
      provider
    );
    this.registryContract = new ethers.Contract(
      registryAddress,
      REGISTRY_ABI,
      provider
    );
    this.metadataFetcher = metadataFetcher || new IPFSMetadataFetcher();

    // Default options
    this.options = {
      batchSize: options?.batchSize || 10,
      debounceMs: options?.debounceMs || 2000,
      maxRetries: options?.maxRetries || 3,
      ipfsTimeout: options?.ipfsTimeout || 10000
    };

    // Initialize empty index
    this.index = {
      documents: new Map(),
      lastUpdated: 0,
      version: '1.0.0',
      totalDocuments: 0
    };
  }

  /**
   * Initialize the search index
   */
  async initialize(): Promise<void> {
    if (this.isInitialized) {
      console.warn('Search index already initialized');
      return;
    }

    console.log('Initializing search index...');
    this.isBuilding = true;

    try {
      // Build initial index from all marketplace listings
      await this.buildFullIndex();

      // Subscribe to contract events
      this.subscribeToEvents();

      this.isInitialized = true;
      console.log(`Search index initialized with ${this.index.totalDocuments} models`);
    } catch (error) {
      console.error('Failed to initialize search index:', error);
      throw error;
    } finally {
      this.isBuilding = false;
    }
  }

  /**
   * Build full index from all marketplace listings
   */
  private async buildFullIndex(): Promise<void> {
    console.log('Building full search index...');
    const startTime = Date.now();

    try {
      // Get all listed model hashes
      const modelHashes: string[] = await this.marketplaceContract.getAllListings();
      console.log(`Found ${modelHashes.length} marketplace listings`);

      // Process in batches
      const batches: string[][] = [];
      for (let i = 0; i < modelHashes.length; i += this.options.batchSize) {
        batches.push(modelHashes.slice(i, i + this.options.batchSize));
      }

      let processed = 0;
      for (const batch of batches) {
        await Promise.all(
          batch.map(async (modelHash) => {
            try {
              const document = await this.buildSearchDocument(modelHash);
              if (document) {
                this.index.documents.set(modelHash, document);
              }
              processed++;
            } catch (error) {
              console.error(`Failed to index model ${modelHash}:`, error);
            }
          })
        );

        console.log(`Indexed ${processed}/${modelHashes.length} models...`);
      }

      this.index.totalDocuments = this.index.documents.size;
      this.index.lastUpdated = Date.now();

      const duration = Date.now() - startTime;
      console.log(`Full index built in ${duration}ms (${processed} models)`);
    } catch (error) {
      console.error('Failed to build full index:', error);
      throw error;
    }
  }

  /**
   * Build a search document for a single model
   */
  private async buildSearchDocument(modelHash: string): Promise<SearchDocument | null> {
    try {
      // Fetch on-chain data in parallel
      const [listingInfo, reviewStats, modelInfo, metadataURI] = await Promise.all([
        this.getListingInfo(modelHash),
        this.getReviewStats(modelHash),
        this.getModelInfo(modelHash),
        this.getMetadataURI(modelHash)
      ]);

      // Skip if model is not active
      if (!listingInfo.isActive || !modelInfo.isActive) {
        return null;
      }

      // Fetch IPFS metadata
      let ipfsMetadata: RawIPFSMetadata;
      try {
        ipfsMetadata = await this.metadataFetcher.fetchMetadata(metadataURI);
      } catch (error) {
        console.warn(`Failed to fetch IPFS metadata for ${modelHash}:`, error);
        // Create minimal metadata from on-chain data
        ipfsMetadata = {
          name: modelInfo.name,
          description: `${modelInfo.framework} model`,
          category: this.categoryNumberToString(listingInfo.category),
          framework: modelInfo.framework,
          tags: [],
          modelSize: Number(modelInfo.sizeBytes),
          version: modelInfo.version
        };
      }

      // Calculate average rating
      const averageRating = reviewStats.reviewCount > 0n
        ? Number(reviewStats.sumRatings) / Number(reviewStats.reviewCount)
        : 0;

      // Get total inferences - using totalSales as a proxy for inference count
      // In production with InferenceRouter, this would be fetched separately
      // For now, totalSales represents completed inference purchases
      const totalInferences = Number(listingInfo.totalSales);

      // Convert to search document
      const document = this.metadataFetcher.convertToSearchDocument(
        modelHash,
        ipfsMetadata,
        {
          creatorAddress: listingInfo.owner,
          basePrice: Number(listingInfo.basePrice),
          discountPrice: Number(listingInfo.discountPrice),
          averageRating: Math.round(averageRating * 100), // Store as 0-500
          reviewCount: Number(reviewStats.reviewCount),
          totalSales: Number(listingInfo.totalSales),
          totalInferences,
          isActive: listingInfo.isActive,
          featured: listingInfo.isFeatured,
          listedAt: Number(modelInfo.timestamp),
          metadataURI
        }
      );

      return document;
    } catch (error) {
      console.error(`Error building search document for ${modelHash}:`, error);
      return null;
    }
  }

  /**
   * Subscribe to marketplace contract events
   */
  private subscribeToEvents(): void {
    console.log('Subscribing to marketplace events...');

    // ModelListed - Add new model to index
    this.marketplaceContract.on('ModelListed', (modelHash: string) => {
      console.log(`Event: ModelListed ${modelHash}`);
      this.queueUpdate(modelHash, 'added');
    });

    // ModelPriceUpdated - Update existing model
    this.marketplaceContract.on('ModelPriceUpdated', (modelHash: string) => {
      console.log(`Event: ModelPriceUpdated ${modelHash}`);
      this.queueUpdate(modelHash, 'updated');
    });

    // ModelDelisted - Remove from index
    this.marketplaceContract.on('ModelDelisted', (modelHash: string) => {
      console.log(`Event: ModelDelisted ${modelHash}`);
      this.removeFromIndex(modelHash);
    });

    // ModelFeatured - Update featured status
    this.marketplaceContract.on('ModelFeatured', (modelHash: string) => {
      console.log(`Event: ModelFeatured ${modelHash}`);
      this.queueUpdate(modelHash, 'updated');
    });

    // ModelPurchased - Update sales count
    this.marketplaceContract.on('ModelPurchased', (modelHash: string) => {
      console.log(`Event: ModelPurchased ${modelHash}`);
      this.queueUpdate(modelHash, 'updated');
    });

    // ReviewSubmitted - Update ratings
    this.marketplaceContract.on('ReviewSubmitted', (modelHash: string) => {
      console.log(`Event: ReviewSubmitted ${modelHash}`);
      this.queueUpdate(modelHash, 'updated');
    });

    console.log('Event subscriptions active');
  }

  /**
   * Queue an index update with debouncing
   */
  private queueUpdate(modelHash: string, type: 'added' | 'updated'): void {
    this.updateQueue.add(modelHash);

    // Clear existing debounce timer
    if (this.debounceTimer) {
      clearTimeout(this.debounceTimer);
    }

    // Set new debounce timer
    this.debounceTimer = setTimeout(() => {
      this.processUpdateQueue();
    }, this.options.debounceMs);
  }

  /**
   * Process queued updates
   */
  private async processUpdateQueue(): Promise<void> {
    if (this.updateQueue.size === 0 || this.isBuilding) {
      return;
    }

    const modelHashes = Array.from(this.updateQueue);
    this.updateQueue.clear();

    console.log(`Processing ${modelHashes.length} queued index updates...`);

    for (const modelHash of modelHashes) {
      try {
        const document = await this.buildSearchDocument(modelHash);
        if (document) {
          const wasNew = !this.index.documents.has(modelHash);
          this.index.documents.set(modelHash, document);

          if (wasNew) {
            this.index.totalDocuments++;
          }

          console.log(`Updated index for ${modelHash}`);
        }
      } catch (error) {
        console.error(`Failed to update index for ${modelHash}:`, error);
      }
    }

    this.index.lastUpdated = Date.now();
  }

  /**
   * Remove model from index
   */
  private removeFromIndex(modelHash: string): void {
    const existed = this.index.documents.delete(modelHash);
    if (existed) {
      this.index.totalDocuments--;
      this.index.lastUpdated = Date.now();
      console.log(`Removed ${modelHash} from index`);
    }
  }

  /**
   * Get the current search index
   */
  getIndex(): SearchIndex {
    return this.index;
  }

  /**
   * Get all documents
   */
  getAllDocuments(): SearchDocument[] {
    return Array.from(this.index.documents.values());
  }

  /**
   * Get document by model ID
   */
  getDocument(modelId: string): SearchDocument | undefined {
    return this.index.documents.get(modelId);
  }

  /**
   * Get index statistics
   */
  getStatistics(): SearchStatistics {
    const documents = this.getAllDocuments();
    const activeModels = documents.filter(d => d.isActive);
    const featuredModels = documents.filter(d => d.featured);

    // Count by category
    const categoryCounts: Record<ModelCategory, number> = {} as any;
    for (const category of Object.values(ModelCategory)) {
      categoryCounts[category] = documents.filter(d => d.category === category).length;
    }

    // Count by framework
    const frameworkCounts: Record<string, number> = {};
    for (const doc of documents) {
      frameworkCounts[doc.framework] = (frameworkCounts[doc.framework] || 0) + 1;
    }

    // Average quality score
    const avgQualityScore = documents.length > 0
      ? documents.reduce((sum, d) => sum + d.qualityScore, 0) / documents.length
      : 0;

    // Estimate index size (rough calculation)
    const indexSize = JSON.stringify(Array.from(this.index.documents.entries())).length;

    return {
      totalModels: documents.length,
      activeModels: activeModels.length,
      featuredModels: featuredModels.length,
      categoryCounts,
      frameworkCounts,
      avgQualityScore,
      indexSize,
      lastRebuildTime: this.index.lastUpdated
    };
  }

  /**
   * Force a full rebuild of the index
   */
  async rebuild(): Promise<void> {
    console.log('Forcing full index rebuild...');
    this.isBuilding = true;

    try {
      // Clear existing index
      this.index.documents.clear();
      this.index.totalDocuments = 0;

      // Rebuild
      await this.buildFullIndex();
    } finally {
      this.isBuilding = false;
    }
  }

  /**
   * Check if index is ready
   */
  isReady(): boolean {
    return this.isInitialized && !this.isBuilding;
  }

  /**
   * Dispose of resources (unsubscribe from events)
   */
  dispose(): void {
    console.log('Disposing search index builder...');

    // Clear debounce timer
    if (this.debounceTimer) {
      clearTimeout(this.debounceTimer);
    }

    // Remove all event listeners
    this.marketplaceContract.removeAllListeners();

    this.isInitialized = false;
  }

  // ============================================================================
  // Contract call helpers
  // ============================================================================

  private async getListingInfo(modelHash: string): Promise<ListingInfo> {
    const result = await this.marketplaceContract.getListingInfo(modelHash);
    return {
      owner: result.owner,
      basePrice: result.basePrice,
      discountPrice: result.discountPrice,
      minBulkSize: result.minBulkSize,
      category: result.category,
      isActive: result.isActive,
      isFeatured: result.isFeatured,
      totalSales: result.totalSales,
      metadataURI: result.metadataURI
    };
  }

  private async getReviewStats(modelHash: string): Promise<ReviewStats> {
    try {
      const result = await this.marketplaceContract.getReviewStats(modelHash);
      return {
        totalRatings: result.totalRatings,
        sumRatings: result.sumRatings,
        reviewCount: result.reviewCount
      };
    } catch (error) {
      // Review stats might not exist for all models
      return {
        totalRatings: 0n,
        sumRatings: 0n,
        reviewCount: 0n
      };
    }
  }

  private async getModelInfo(modelHash: string): Promise<ModelInfo> {
    const result = await this.registryContract.getModelInfo(modelHash);
    return {
      name: result.name,
      framework: result.framework,
      version: result.version,
      sizeBytes: result.sizeBytes,
      owner: result.owner,
      timestamp: result.timestamp,
      isActive: result.isActive
    };
  }

  private async getMetadataURI(modelHash: string): Promise<string> {
    return await this.registryContract.getModelMetadataURI(modelHash);
  }

  private categoryNumberToString(category: number): string {
    const categoryMap: Record<number, string> = {
      0: 'language-models',
      1: 'code-models',
      2: 'vision-models',
      3: 'embedding-models',
      4: 'multimodal-models',
      5: 'generative-models',
      6: 'audio-models',
      7: 'reinforcement-learning',
      8: 'time-series',
      9: 'tabular-models',
      10: 'other'
    };

    return categoryMap[category] || 'other';
  }
}

// ============================================================================
// Export singleton factory
// ============================================================================

let globalIndexBuilder: SearchIndexBuilder | null = null;

export async function initializeSearchIndex(
  provider: ethers.Provider,
  marketplaceAddress: string,
  registryAddress: string,
  options?: IndexBuilderOptions
): Promise<SearchIndexBuilder> {
  if (globalIndexBuilder) {
    console.warn('Search index already initialized, disposing old instance');
    globalIndexBuilder.dispose();
  }

  globalIndexBuilder = new SearchIndexBuilder(
    provider,
    marketplaceAddress,
    registryAddress,
    undefined,
    options
  );

  await globalIndexBuilder.initialize();
  return globalIndexBuilder;
}

export function getSearchIndexBuilder(): SearchIndexBuilder | null {
  return globalIndexBuilder;
}

export function disposeSearchIndex(): void {
  if (globalIndexBuilder) {
    globalIndexBuilder.dispose();
    globalIndexBuilder = null;
  }
}
