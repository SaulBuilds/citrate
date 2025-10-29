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
    authorVerified: false, // TODO: Implement verification system
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

/**
 * Fetch metadata from IPFS
 * TODO: Implement actual IPFS fetching
 */
export async function fetchModelMetadata(
  metadataURI: string
): Promise<ModelMetadata | null> {
  try {
    console.log('[MarketplaceHelpers] Fetching metadata from:', metadataURI);

    // TODO: Implement IPFS gateway fetching
    // For now, return null to use default values
    // In production, this would fetch from:
    // - IPFS gateway (https://ipfs.io/ipfs/{cid})
    // - Pinata gateway
    // - Local IPFS node
    // - Arweave (if using AR instead of IPFS)

    return null;
  } catch (error) {
    console.error('[MarketplaceHelpers] Failed to fetch metadata:', error);
    return null;
  }
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
      // TODO: Implement a way to get all listings
      // For now, try featured + top rated
      const featured = await marketplaceService.getFeaturedModels();
      const topRated = await marketplaceService.getTopRatedModels(20);
      modelIds = [...new Set([...featured, ...topRated])];
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
