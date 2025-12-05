/**
 * Marketplace Helpers
 *
 * Utility functions for converting between contract data and UI data formats.
 * Handles data transformation and IPFS metadata fetching.
 */

import {
  MarketplaceService,
  ModelListing,
  CATEGORIES,
} from './marketplaceService';

// UI data format (used by the Marketplace component)
export interface MarketplaceModel {
  id: string;
  name: string;
  description: string;
  category: string;
  architecture: string;
  version: string;
  price: number; // in ETH
  currency: 'LAT' | 'ETH';
  rating: number;
  reviews: number;
  downloads: number;
  size: string;
  lastUpdated: string;
  author: string;
  authorVerified: boolean;
  tags: string[];
  featured: boolean;
  preview?: string;
  modelType: 'text' | 'image' | 'audio' | 'multimodal';
  license: 'MIT' | 'Apache-2.0' | 'Commercial' | 'Custom';
  ipfsCid: string;
}

// Extended metadata from IPFS
export interface ModelMetadata {
  name: string;
  description: string;
  architecture?: string;
  version?: string;
  size?: string;
  tags?: string[];
  modelType?: 'text' | 'image' | 'audio' | 'multimodal';
  license?: 'MIT' | 'Apache-2.0' | 'Commercial' | 'Custom';
  preview?: string;
  additionalInfo?: Record<string, any>;
}

/**
 * Convert contract ModelListing to UI MarketplaceModel format
 */
export function convertListingToModel(
  listing: ModelListing,
  metadata?: ModelMetadata
): MarketplaceModel {
  // Convert category number to string name
  const categoryName = CATEGORIES[listing.category]?.name || 'Other';

  // Convert prices from wei to ETH
  const basePrice = Number(listing.basePrice) / 1e18;

  // Convert rating (stored as percentage, e.g., 480 = 4.8 stars)
  const rating = Number(listing.averageRating) / 100;

  // Extract IPFS CID from metadataURI
  const ipfsCid = extractIPFSCid(listing.metadataURI);

  // Use metadata if available, otherwise use defaults
  const name = metadata?.name || `Model ${listing.modelId.slice(0, 8)}`;
  const description =
    metadata?.description || 'No description available';
  const architecture = metadata?.architecture || 'Unknown';
  const version = metadata?.version || '1.0.0';
  const size = metadata?.size || 'Unknown';
  const tags = metadata?.tags || [];
  const modelType = metadata?.modelType || 'text';
  const license = metadata?.license || 'Custom';
  const preview = metadata?.preview;

  return {
    id: listing.modelId,
    name,
    description,
    category: categoryName.toLowerCase().replace(/\s+/g, '-'),
    architecture,
    version,
    price: basePrice,
    currency: 'ETH',
    rating,
    reviews: Number(listing.reviewCount),
    downloads: Number(listing.totalSales), // Use total sales as downloads
    size,
    lastUpdated: new Date(Number(listing.listedAt) * 1000).toISOString().split('T')[0],
    author: listing.owner,
    authorVerified: listing.featured, // Featured models are considered verified (admin-curated)
    tags,
    featured: listing.featured,
    preview,
    modelType,
    license,
    ipfsCid,
  };
}

/**
 * Extract IPFS CID from metadataURI
 */
function extractIPFSCid(metadataURI: string): string {
  if (metadataURI.startsWith('ipfs://')) {
    return metadataURI.replace('ipfs://', '');
  }
  if (metadataURI.startsWith('Qm') || metadataURI.startsWith('bafy')) {
    return metadataURI;
  }
  return '';
}

// IPFS gateway configuration with fallbacks
const IPFS_GATEWAYS = [
  'https://gateway.pinata.cloud/ipfs/',
  'https://cloudflare-ipfs.com/ipfs/',
  'https://ipfs.io/ipfs/',
  'https://dweb.link/ipfs/',
  'https://w3s.link/ipfs/',
];

// Timeout for IPFS gateway requests (10 seconds)
const IPFS_FETCH_TIMEOUT = 10000;

/**
 * Fetch metadata from IPFS using multiple gateways with fallback
 * Tries each gateway in order until successful
 */
export async function fetchModelMetadata(
  metadataURI: string
): Promise<ModelMetadata | null> {
  try {
    console.log('[MarketplaceHelpers] Fetching metadata from:', metadataURI);

    // Extract CID from URI
    const cid = extractIPFSCid(metadataURI);
    if (!cid) {
      console.error('[MarketplaceHelpers] Invalid IPFS URI:', metadataURI);
      return null;
    }

    // Try each gateway until successful
    for (const gateway of IPFS_GATEWAYS) {
      try {
        const url = `${gateway}${cid}`;
        console.log('[MarketplaceHelpers] Trying gateway:', url);

        const controller = new AbortController();
        const timeoutId = setTimeout(() => controller.abort(), IPFS_FETCH_TIMEOUT);

        const response = await fetch(url, {
          signal: controller.signal,
          headers: {
            'Accept': 'application/json',
          },
        });

        clearTimeout(timeoutId);

        if (!response.ok) {
          console.warn(`[MarketplaceHelpers] Gateway ${gateway} returned ${response.status}`);
          continue;
        }

        const contentType = response.headers.get('content-type') || '';
        if (!contentType.includes('json') && !contentType.includes('text')) {
          console.warn(`[MarketplaceHelpers] Gateway ${gateway} returned non-JSON content type: ${contentType}`);
          continue;
        }

        const data = await response.json();
        console.log('[MarketplaceHelpers] Successfully fetched metadata from:', gateway);

        // Validate and parse metadata
        return parseModelMetadata(data);
      } catch (error) {
        if (error instanceof Error && error.name === 'AbortError') {
          console.warn(`[MarketplaceHelpers] Gateway ${gateway} timed out`);
        } else {
          console.warn(`[MarketplaceHelpers] Gateway ${gateway} failed:`, error);
        }
        continue;
      }
    }

    // All gateways failed
    console.error('[MarketplaceHelpers] All IPFS gateways failed for CID:', cid);
    return null;
  } catch (error) {
    console.error('[MarketplaceHelpers] Failed to fetch metadata:', error);
    return null;
  }
}

/**
 * Parse and validate raw IPFS metadata into ModelMetadata format
 */
function parseModelMetadata(rawData: any): ModelMetadata | null {
  if (!rawData || typeof rawData !== 'object') {
    return null;
  }

  // Handle various metadata formats (OpenAI model card, HuggingFace, custom)
  const metadata: ModelMetadata = {
    name: rawData.name || rawData.modelName || rawData.title || '',
    description: rawData.description || rawData.summary || rawData.about || '',
    architecture: rawData.architecture || rawData.model_architecture || rawData.type || undefined,
    version: rawData.version || rawData.model_version || undefined,
    size: rawData.size || rawData.model_size || rawData.sizeBytes
      ? formatFileSize(Number(rawData.sizeBytes))
      : undefined,
    tags: Array.isArray(rawData.tags) ? rawData.tags
      : typeof rawData.tags === 'string' ? rawData.tags.split(',').map((t: string) => t.trim())
      : rawData.keywords || [],
    modelType: detectModelType(rawData),
    license: detectLicense(rawData.license || rawData.licenseName),
    preview: rawData.preview || rawData.image || rawData.thumbnail || rawData.cover_image || undefined,
    additionalInfo: extractAdditionalInfo(rawData),
  };

  // Validate that at least name and description exist
  if (!metadata.name && !metadata.description) {
    console.warn('[MarketplaceHelpers] Metadata missing name and description');
    return null;
  }

  return metadata;
}

/**
 * Detect model type from metadata fields
 */
function detectModelType(data: any): 'text' | 'image' | 'audio' | 'multimodal' {
  const typeHints = [
    data.modelType?.toLowerCase(),
    data.type?.toLowerCase(),
    data.task?.toLowerCase(),
    data.category?.toLowerCase(),
    ...(data.tags || []).map((t: string) => t.toLowerCase()),
  ].filter(Boolean);

  if (typeHints.some(h => h.includes('image') || h.includes('vision') || h.includes('diffusion'))) {
    return 'image';
  }
  if (typeHints.some(h => h.includes('audio') || h.includes('speech') || h.includes('whisper'))) {
    return 'audio';
  }
  if (typeHints.some(h => h.includes('multimodal') || h.includes('multi-modal'))) {
    return 'multimodal';
  }
  return 'text';
}

/**
 * Detect and normalize license type
 */
function detectLicense(license: string | undefined): 'MIT' | 'Apache-2.0' | 'Commercial' | 'Custom' {
  if (!license) return 'Custom';

  const lowerLicense = license.toLowerCase();
  if (lowerLicense.includes('mit')) return 'MIT';
  if (lowerLicense.includes('apache')) return 'Apache-2.0';
  if (lowerLicense.includes('commercial') || lowerLicense.includes('proprietary')) return 'Commercial';
  return 'Custom';
}

/**
 * Extract additional info fields that aren't core metadata
 */
function extractAdditionalInfo(data: any): Record<string, any> | undefined {
  const coreFields = ['name', 'description', 'architecture', 'version', 'size', 'tags', 'modelType', 'license', 'preview'];
  const additional: Record<string, any> = {};

  for (const [key, value] of Object.entries(data)) {
    if (!coreFields.includes(key) && value !== undefined && value !== null) {
      additional[key] = value;
    }
  }

  return Object.keys(additional).length > 0 ? additional : undefined;
}

/**
 * Format file size in human-readable format
 */
function formatFileSize(bytes: number): string {
  if (bytes === 0) return '0 B';
  const k = 1024;
  const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
}

/**
 * Load marketplace models from the blockchain
 */
export async function loadMarketplaceModels(
  marketplaceService: MarketplaceService,
  options?: {
    category?: number;
    featured?: boolean;
    topRated?: boolean;
    limit?: number;
  }
): Promise<MarketplaceModel[]> {
  try {
    let modelIds: string[] = [];

    // Fetch model IDs based on options
    if (options?.featured) {
      modelIds = await marketplaceService.getFeaturedModels();
    } else if (options?.topRated) {
      modelIds = await marketplaceService.getTopRatedModels(options.limit || 10);
    } else if (options?.category !== undefined) {
      modelIds = await marketplaceService.getModelsByCategory(options.category);
    } else {
      // Get all listings by querying each category and combining results
      // This is the most complete way to enumerate all models without a dedicated "all" query
      const allModelIds = new Set<string>();

      // First get featured and top rated for quick results
      const featured = await marketplaceService.getFeaturedModels();
      const topRated = await marketplaceService.getTopRatedModels(options?.limit || 50);
      featured.forEach(id => allModelIds.add(id));
      topRated.forEach(id => allModelIds.add(id));

      // Then query each category (0-10)
      for (let category = 0; category <= 10; category++) {
        try {
          const categoryModels = await marketplaceService.getModelsByCategory(category);
          categoryModels.forEach(id => allModelIds.add(id));
        } catch (err) {
          console.warn(`[MarketplaceHelpers] Failed to get category ${category}:`, err);
        }
      }

      modelIds = Array.from(allModelIds);
    }

    // Fetch listing details for each model
    const models: MarketplaceModel[] = [];
    for (const modelId of modelIds) {
      const listing = await marketplaceService.getListing(modelId);
      if (listing && listing.active) {
        // Try to fetch metadata (currently returns null)
        const metadata = await fetchModelMetadata(listing.metadataURI);
        const model = convertListingToModel(listing, metadata || undefined);
        models.push(model);
      }
    }

    return models;
  } catch (error) {
    console.error('[MarketplaceHelpers] Failed to load models:', error);
    return [];
  }
}

/**
 * Load models for a specific owner
 */
export async function loadOwnerModels(
  marketplaceService: MarketplaceService,
  ownerAddress: string
): Promise<MarketplaceModel[]> {
  try {
    const modelIds = await marketplaceService.getModelsByOwner(ownerAddress);
    const models: MarketplaceModel[] = [];

    for (const modelId of modelIds) {
      const listing = await marketplaceService.getListing(modelId);
      if (listing) {
        const metadata = await fetchModelMetadata(listing.metadataURI);
        const model = convertListingToModel(listing, metadata || undefined);
        models.push(model);
      }
    }

    return models;
  } catch (error) {
    console.error('[MarketplaceHelpers] Failed to load owner models:', error);
    return [];
  }
}

/**
 * Get marketplace statistics
 */
export async function getMarketplaceStatistics(
  marketplaceService: MarketplaceService
): Promise<{
  totalListings: number;
  totalSales: number;
  totalVolume: string; // in ETH
}> {
  try {
    const stats = await marketplaceService.getMarketplaceStats();
    return {
      totalListings: Number(stats.totalListings),
      totalSales: Number(stats.totalSales),
      totalVolume: (Number(stats.totalVolume) / 1e18).toFixed(4),
    };
  } catch (error) {
    console.error('[MarketplaceHelpers] Failed to get stats:', error);
    return {
      totalListings: 0,
      totalSales: 0,
      totalVolume: '0',
    };
  }
}

/**
 * Mock data generator for development/testing
 */
export function getMockMarketplaceModels(): MarketplaceModel[] {
  return [
    {
      id: '0x' + '1'.repeat(64),
      name: 'GPT-4 Fine-tuned for Code',
      description:
        'Advanced language model fine-tuned specifically for code generation and debugging',
      category: 'text-generation',
      architecture: 'transformer',
      version: '1.2.0',
      price: 0.05,
      currency: 'ETH',
      rating: 4.8,
      reviews: 1247,
      downloads: 15689,
      size: '13.5 GB',
      lastUpdated: '2024-01-15',
      author: '0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb',
      authorVerified: true,
      tags: ['coding', 'debugging', 'typescript', 'python'],
      featured: true,
      modelType: 'text',
      license: 'MIT',
      ipfsCid: 'QmXoYpizjW3WknFiJnKLwHCnL72vedxjQkDDP1mXWo6uco',
    },
    {
      id: '0x' + '2'.repeat(64),
      name: 'Stable Diffusion v3.0',
      description:
        'High-quality image generation model with improved coherence and style control',
      category: 'image-generation',
      architecture: 'diffusion',
      version: '3.0.1',
      price: 0.15,
      currency: 'ETH',
      rating: 4.9,
      reviews: 2891,
      downloads: 45231,
      size: '8.2 GB',
      lastUpdated: '2024-01-20',
      author: '0x123d35Cc6634C0532925a3b844Bc9e7595f0bEb',
      authorVerified: true,
      tags: ['art', 'design', 'creative', 'high-res'],
      featured: true,
      modelType: 'image',
      license: 'Commercial',
      ipfsCid: 'QmYoZpizjW3WknFiJnKLwHCnL72vedxjQkDDP1mXWo6use',
    },
    {
      id: '0x' + '3'.repeat(64),
      name: 'Whisper Multilingual ASR',
      description:
        'Automatic speech recognition supporting 100+ languages with high accuracy',
      category: 'audio-processing',
      architecture: 'transformer',
      version: '2.1.0',
      price: 0.02,
      currency: 'ETH',
      rating: 4.7,
      reviews: 892,
      downloads: 8934,
      size: '1.8 GB',
      lastUpdated: '2024-01-10',
      author: '0x456d35Cc6634C0532925a3b844Bc9e7595f0bEb',
      authorVerified: false,
      tags: ['speech', 'multilingual', 'transcription'],
      featured: false,
      modelType: 'audio',
      license: 'Apache-2.0',
      ipfsCid: 'QmZoApizjW3WknFiJnKLwHCnL72vedxjQkDDP1mXWo6usr',
    },
  ];
}

/**
 * Check if marketplace contract is available
 */
export async function isMarketplaceAvailable(
  marketplaceService: MarketplaceService
): Promise<boolean> {
  try {
    // Try to fetch stats as a health check
    await marketplaceService.getMarketplaceStats();
    return true;
  } catch (error) {
    console.log('[MarketplaceHelpers] Marketplace contract not available:', error);
    return false;
  }
}
