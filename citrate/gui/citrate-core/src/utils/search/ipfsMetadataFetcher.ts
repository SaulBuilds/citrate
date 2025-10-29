/**
 * IPFS Metadata Fetcher for Citrate Model Marketplace
 *
 * Fetches and parses model metadata from IPFS with:
 * - Multiple gateway support (Pinata + fallbacks)
 * - Exponential backoff retry logic
 * - Metadata validation
 * - In-memory caching
 * - Batch fetching optimization
 */

import { SearchDocument, ModelCategory, categorizeModelSize } from './types';

// ============================================================================
// Configuration
// ============================================================================

/**
 * IPFS Gateway configuration
 */
export interface IPFSGatewayConfig {
  url: string;
  timeout: number;
  priority: number;  // Lower = higher priority
}

/**
 * Default IPFS gateways (in priority order)
 */
export const DEFAULT_GATEWAYS: IPFSGatewayConfig[] = [
  {
    url: 'https://gateway.pinata.cloud/ipfs/',
    timeout: 10000,
    priority: 1
  },
  {
    url: 'https://ipfs.io/ipfs/',
    timeout: 15000,
    priority: 2
  },
  {
    url: 'https://cloudflare-ipfs.com/ipfs/',
    timeout: 15000,
    priority: 3
  },
  {
    url: 'https://dweb.link/ipfs/',
    timeout: 20000,
    priority: 4
  }
];

/**
 * Retry configuration
 */
export interface RetryConfig {
  maxRetries: number;
  initialDelayMs: number;
  maxDelayMs: number;
  backoffMultiplier: number;
}

export const DEFAULT_RETRY_CONFIG: RetryConfig = {
  maxRetries: 3,
  initialDelayMs: 1000,
  maxDelayMs: 10000,
  backoffMultiplier: 2
};

// ============================================================================
// Metadata Types (from IPFS)
// ============================================================================

/**
 * Raw metadata structure from IPFS
 */
export interface RawIPFSMetadata {
  // Core info
  name: string;
  description: string;
  category: string;
  framework: string;
  version?: string;
  license?: string;

  // Creator
  creator?: {
    address: string;
    name?: string;
    ens?: string;
  };

  // Technical details
  modelSize?: number;        // Bytes
  parameters?: number;       // Model parameter count
  inputShape?: string[];
  outputShape?: string[];

  // Tags and classification
  tags?: string[];
  useCases?: string[];
  languages?: string[];

  // Media
  thumbnail?: string;        // IPFS CID or HTTP URL
  images?: string[];
  demo?: string;

  // Quality metadata
  benchmarks?: {
    name: string;
    score: number;
    metric: string;
  }[];

  performanceMetrics?: {
    avgLatency?: number;
    throughput?: number;
    accuracy?: number;
  };

  // Additional metadata
  trainingData?: string;
  architecture?: string;
  pretrainedOn?: string;
  finetuneMethod?: string;

  // Social proof
  featured?: boolean;
  verified?: boolean;

  // Custom fields
  [key: string]: any;
}

/**
 * Validation result
 */
export interface ValidationResult {
  valid: boolean;
  errors: string[];
  warnings: string[];
}

// ============================================================================
// Cache
// ============================================================================

/**
 * Simple in-memory cache with TTL
 */
class MetadataCache {
  private cache: Map<string, { data: RawIPFSMetadata; timestamp: number }> = new Map();
  private ttlMs: number;

  constructor(ttlMs: number = 5 * 60 * 1000) { // Default 5 minutes
    this.ttlMs = ttlMs;
  }

  get(cid: string): RawIPFSMetadata | null {
    const entry = this.cache.get(cid);
    if (!entry) return null;

    const age = Date.now() - entry.timestamp;
    if (age > this.ttlMs) {
      this.cache.delete(cid);
      return null;
    }

    return entry.data;
  }

  set(cid: string, data: RawIPFSMetadata): void {
    this.cache.set(cid, {
      data,
      timestamp: Date.now()
    });
  }

  clear(): void {
    this.cache.clear();
  }

  size(): number {
    return this.cache.size;
  }
}

// ============================================================================
// IPFS Metadata Fetcher
// ============================================================================

export class IPFSMetadataFetcher {
  private gateways: IPFSGatewayConfig[];
  private retryConfig: RetryConfig;
  private cache: MetadataCache;

  constructor(
    gateways: IPFSGatewayConfig[] = DEFAULT_GATEWAYS,
    retryConfig: RetryConfig = DEFAULT_RETRY_CONFIG,
    cacheTtlMs: number = 5 * 60 * 1000
  ) {
    this.gateways = gateways.sort((a, b) => a.priority - b.priority);
    this.retryConfig = retryConfig;
    this.cache = new MetadataCache(cacheTtlMs);
  }

  /**
   * Fetch metadata for a single model
   */
  async fetchMetadata(uri: string): Promise<RawIPFSMetadata> {
    const cid = this.extractCID(uri);

    // Check cache first
    const cached = this.cache.get(cid);
    if (cached) {
      return cached;
    }

    // Try each gateway with retries
    let lastError: Error | null = null;

    for (const gateway of this.gateways) {
      try {
        const metadata = await this.fetchWithRetry(cid, gateway);
        this.cache.set(cid, metadata);
        return metadata;
      } catch (error) {
        lastError = error as Error;
        console.warn(`Gateway ${gateway.url} failed for CID ${cid}:`, error);
        // Continue to next gateway
      }
    }

    // All gateways failed
    throw new Error(
      `Failed to fetch metadata for ${cid} from all gateways: ${lastError?.message}`
    );
  }

  /**
   * Fetch metadata with exponential backoff retry
   */
  private async fetchWithRetry(
    cid: string,
    gateway: IPFSGatewayConfig
  ): Promise<RawIPFSMetadata> {
    let lastError: Error | null = null;
    let delay = this.retryConfig.initialDelayMs;

    for (let attempt = 0; attempt <= this.retryConfig.maxRetries; attempt++) {
      try {
        return await this.fetchFromGateway(cid, gateway);
      } catch (error) {
        lastError = error as Error;

        // Don't retry on final attempt
        if (attempt === this.retryConfig.maxRetries) {
          break;
        }

        // Wait before retry
        await this.sleep(delay);

        // Exponential backoff
        delay = Math.min(
          delay * this.retryConfig.backoffMultiplier,
          this.retryConfig.maxDelayMs
        );
      }
    }

    throw lastError || new Error('Unknown error during fetch');
  }

  /**
   * Fetch from a specific gateway
   */
  private async fetchFromGateway(
    cid: string,
    gateway: IPFSGatewayConfig
  ): Promise<RawIPFSMetadata> {
    const url = `${gateway.url}${cid}`;

    const controller = new AbortController();
    const timeoutId = setTimeout(() => controller.abort(), gateway.timeout);

    try {
      const response = await fetch(url, {
        signal: controller.signal,
        headers: {
          'Accept': 'application/json'
        }
      });

      clearTimeout(timeoutId);

      if (!response.ok) {
        throw new Error(`HTTP ${response.status}: ${response.statusText}`);
      }

      const data = await response.json();

      // Validate metadata structure
      const validation = this.validateMetadata(data);
      if (!validation.valid) {
        throw new Error(`Invalid metadata: ${validation.errors.join(', ')}`);
      }

      return data as RawIPFSMetadata;
    } catch (error) {
      clearTimeout(timeoutId);

      if (error instanceof Error && error.name === 'AbortError') {
        throw new Error(`Timeout after ${gateway.timeout}ms`);
      }

      throw error;
    }
  }

  /**
   * Batch fetch multiple metadata entries
   */
  async fetchBatch(uris: string[]): Promise<Map<string, RawIPFSMetadata | Error>> {
    const results = new Map<string, RawIPFSMetadata | Error>();

    // Fetch in parallel with concurrency limit
    const concurrencyLimit = 5;
    const batches: string[][] = [];

    for (let i = 0; i < uris.length; i += concurrencyLimit) {
      batches.push(uris.slice(i, i + concurrencyLimit));
    }

    for (const batch of batches) {
      const promises = batch.map(async (uri) => {
        try {
          const metadata = await this.fetchMetadata(uri);
          results.set(uri, metadata);
        } catch (error) {
          results.set(uri, error as Error);
        }
      });

      await Promise.all(promises);
    }

    return results;
  }

  /**
   * Extract CID from various URI formats
   */
  private extractCID(uri: string): string {
    // ipfs://QmXXX
    if (uri.startsWith('ipfs://')) {
      return uri.slice(7);
    }

    // https://ipfs.io/ipfs/QmXXX
    if (uri.includes('/ipfs/')) {
      const match = uri.match(/\/ipfs\/([^/?#]+)/);
      if (match) return match[1];
    }

    // Direct CID
    if (uri.startsWith('Qm') || uri.startsWith('bafy')) {
      return uri;
    }

    throw new Error(`Invalid IPFS URI: ${uri}`);
  }

  /**
   * Validate metadata structure
   */
  private validateMetadata(data: any): ValidationResult {
    const errors: string[] = [];
    const warnings: string[] = [];

    // Required fields
    if (!data.name || typeof data.name !== 'string') {
      errors.push('Missing or invalid "name" field');
    }
    if (!data.description || typeof data.description !== 'string') {
      errors.push('Missing or invalid "description" field');
    }
    if (!data.category || typeof data.category !== 'string') {
      errors.push('Missing or invalid "category" field');
    }
    if (!data.framework || typeof data.framework !== 'string') {
      errors.push('Missing or invalid "framework" field');
    }

    // Optional but recommended fields
    if (!data.tags || !Array.isArray(data.tags)) {
      warnings.push('Missing or invalid "tags" field');
    }
    if (!data.modelSize || typeof data.modelSize !== 'number') {
      warnings.push('Missing or invalid "modelSize" field');
    }

    // Validate category
    if (data.category) {
      const validCategories = Object.values(ModelCategory);
      if (!validCategories.includes(data.category)) {
        warnings.push(`Unknown category: ${data.category}`);
      }
    }

    return {
      valid: errors.length === 0,
      errors,
      warnings
    };
  }

  /**
   * Convert raw IPFS metadata to SearchDocument
   */
  convertToSearchDocument(
    modelId: string,
    metadata: RawIPFSMetadata,
    onChainData: {
      creatorAddress: string;
      basePrice: number;
      discountPrice: number;
      averageRating: number;
      reviewCount: number;
      totalSales: number;
      totalInferences: number;
      isActive: boolean;
      featured: boolean;
      listedAt: number;
      lastSaleAt?: number;
      metadataURI: string;
    }
  ): SearchDocument {
    const ipfsCID = this.extractCID(onChainData.metadataURI);

    return {
      modelId,
      name: metadata.name,
      description: metadata.description,
      tags: metadata.tags || [],
      category: this.mapCategory(metadata.category),
      framework: metadata.framework,
      creatorAddress: onChainData.creatorAddress,
      creatorName: metadata.creator?.name || metadata.creator?.ens,
      basePrice: onChainData.basePrice,
      discountPrice: onChainData.discountPrice,
      averageRating: onChainData.averageRating,
      reviewCount: onChainData.reviewCount,
      totalSales: onChainData.totalSales,
      totalInferences: onChainData.totalInferences,
      isActive: onChainData.isActive,
      featured: onChainData.featured || metadata.featured || false,
      listedAt: onChainData.listedAt,
      lastSaleAt: onChainData.lastSaleAt,
      qualityScore: this.calculateQualityScore({
        averageRating: onChainData.averageRating,
        reviewCount: onChainData.reviewCount,
        totalInferences: onChainData.totalInferences,
        performanceMetrics: metadata.performanceMetrics
      }),
      metadataURI: onChainData.metadataURI,
      ipfsCID,
      sizeBytes: metadata.modelSize,
      modelSize: metadata.modelSize ? categorizeModelSize(metadata.modelSize) : undefined,
      version: metadata.version
    };
  }

  /**
   * Map category string to ModelCategory enum
   */
  private mapCategory(category: string): ModelCategory {
    const normalized = category.toLowerCase().replace(/[_\s]/g, '-');
    const categoryMap: Record<string, ModelCategory> = {
      'language-models': ModelCategory.LANGUAGE_MODELS,
      'language': ModelCategory.LANGUAGE_MODELS,
      'llm': ModelCategory.LANGUAGE_MODELS,
      'code-models': ModelCategory.CODE_MODELS,
      'code': ModelCategory.CODE_MODELS,
      'vision-models': ModelCategory.VISION_MODELS,
      'vision': ModelCategory.VISION_MODELS,
      'image': ModelCategory.VISION_MODELS,
      'embedding-models': ModelCategory.EMBEDDING_MODELS,
      'embedding': ModelCategory.EMBEDDING_MODELS,
      'embeddings': ModelCategory.EMBEDDING_MODELS,
      'multimodal-models': ModelCategory.MULTIMODAL_MODELS,
      'multimodal': ModelCategory.MULTIMODAL_MODELS,
      'generative-models': ModelCategory.GENERATIVE_MODELS,
      'generative': ModelCategory.GENERATIVE_MODELS,
      'generation': ModelCategory.GENERATIVE_MODELS,
      'audio-models': ModelCategory.AUDIO_MODELS,
      'audio': ModelCategory.AUDIO_MODELS,
      'speech': ModelCategory.AUDIO_MODELS,
      'reinforcement-learning': ModelCategory.REINFORCEMENT_LEARNING,
      'rl': ModelCategory.REINFORCEMENT_LEARNING,
      'time-series': ModelCategory.TIME_SERIES,
      'timeseries': ModelCategory.TIME_SERIES,
      'forecasting': ModelCategory.TIME_SERIES,
      'tabular-models': ModelCategory.TABULAR_MODELS,
      'tabular': ModelCategory.TABULAR_MODELS
    };

    return categoryMap[normalized] || ModelCategory.OTHER;
  }

  /**
   * Calculate quality score (0-100)
   */
  private calculateQualityScore(params: {
    averageRating: number;
    reviewCount: number;
    totalInferences: number;
    performanceMetrics?: {
      avgLatency?: number;
      throughput?: number;
      accuracy?: number;
    };
  }): number {
    // Rating component (40%)
    const ratingScore = (params.averageRating / 500) * 100; // 0-500 → 0-100
    const reviewWeight = Math.min(params.reviewCount / 50, 1); // Weight by review count
    const weightedRating = ratingScore * (0.5 + 0.5 * reviewWeight);

    // Performance component (30%)
    let performanceScore = 50; // Default neutral score
    if (params.performanceMetrics) {
      const { avgLatency, throughput, accuracy } = params.performanceMetrics;

      if (avgLatency) {
        // Lower latency is better: <100ms = 100, >1000ms = 0
        const latencyScore = Math.max(0, Math.min(100, (1000 - avgLatency) / 10));
        performanceScore = latencyScore;
      }

      if (accuracy) {
        // Direct accuracy percentage
        performanceScore = (performanceScore + accuracy) / 2;
      }
    }

    // Reliability component (20%)
    const reliabilityScore = params.totalInferences > 0 ?
      Math.min(100, (params.totalInferences / 1000) * 100) : 0;

    // Engagement component (10%)
    const engagementScore = Math.min(100, (params.reviewCount / 100) * 100);

    // Weighted sum
    const finalScore =
      weightedRating * 0.4 +
      performanceScore * 0.3 +
      reliabilityScore * 0.2 +
      engagementScore * 0.1;

    return Math.round(Math.max(0, Math.min(100, finalScore)));
  }

  /**
   * Sleep utility
   */
  private sleep(ms: number): Promise<void> {
    return new Promise(resolve => setTimeout(resolve, ms));
  }

  /**
   * Get cache statistics
   */
  getCacheStats(): { size: number; ttlMs: number } {
    return {
      size: this.cache.size(),
      ttlMs: this.cache['ttlMs']
    };
  }

  /**
   * Clear cache
   */
  clearCache(): void {
    this.cache.clear();
  }
}

// ============================================================================
// Export default instance
// ============================================================================

export const ipfsMetadataFetcher = new IPFSMetadataFetcher();
