/**
 * Marketplace Service
 *
 * High-level service layer for interacting with the ModelMarketplace smart contract.
 * Provides type-safe wrappers for all marketplace operations.
 */

import {
  callContractFunction,
  sendContractTransaction,
} from './contractInteraction';

// Type definitions matching the contract interface
export interface ModelListing {
  modelId: string;
  owner: string;
  basePrice: string;
  discountPrice: string;
  minimumBulkSize: string;
  totalSales: string;
  totalRevenue: string;
  category: number;
  metadataURI: string;
  featured: boolean;
  active: boolean;
  listedAt: string;
  lastSaleAt: string;
  totalRating: string;
  reviewCount: string;
  averageRating: string;
}

export interface Purchase {
  modelId: string;
  buyer: string;
  price: string;
  quantity: string;
  timestamp: string;
  bulkDiscount: boolean;
}

export interface Review {
  reviewer: string;
  modelId: string;
  rating: number;
  comment: string;
  timestamp: string;
  verified: boolean;
}

export interface MarketplaceStats {
  totalListings: string;
  totalSales: string;
  totalVolume: string;
}

/**
 * MarketplaceService class
 * Provides methods for interacting with the ModelMarketplace contract
 */
export class MarketplaceService {
  private contractAddress: string;

  constructor(contractAddress: string) {
    this.contractAddress = contractAddress;
  }

  // ============================================================================
  // Write Functions (State-changing transactions)
  // ============================================================================

  /**
   * List a model in the marketplace
   */
  async listModel(
    params: {
      modelId: string;
      basePrice: string; // in wei
      discountPrice: string; // in wei
      minimumBulkSize: number;
      category: number; // 0-10
      metadataURI: string; // IPFS URI
    },
    options: {
      from: string;
      password: string;
      gasLimit?: number;
    }
  ): Promise<string> {
    const result = await sendContractTransaction({
      from: options.from,
      contractAddress: this.contractAddress,
      functionName: 'listModel',
      inputs: [
        { name: 'modelId', type: 'bytes32' },
        { name: 'basePrice', type: 'uint256' },
        { name: 'discountPrice', type: 'uint256' },
        { name: 'minimumBulkSize', type: 'uint256' },
        { name: 'category', type: 'uint8' },
        { name: 'metadataURI', type: 'string' },
      ],
      args: [
        params.modelId,
        params.basePrice,
        params.discountPrice,
        params.minimumBulkSize.toString(),
        params.category,
        params.metadataURI,
      ],
      password: options.password,
      gasLimit: options.gasLimit || 500000,
    });

    return result.txHash;
  }

  /**
   * Purchase access to a model
   */
  async purchaseAccess(
    params: {
      modelId: string;
      quantity: number;
    },
    options: {
      from: string;
      password: string;
      value: string; // ETH amount in wei
      gasLimit?: number;
    }
  ): Promise<string> {
    const result = await sendContractTransaction({
      from: options.from,
      contractAddress: this.contractAddress,
      functionName: 'purchaseAccess',
      inputs: [
        { name: 'modelId', type: 'bytes32' },
        { name: 'quantity', type: 'uint256' },
      ],
      args: [params.modelId, params.quantity.toString()],
      value: options.value,
      password: options.password,
      gasLimit: options.gasLimit || 300000,
    });

    return result.txHash;
  }

  /**
   * Update pricing for a listed model
   */
  async updatePricing(
    params: {
      modelId: string;
      newBasePrice: string;
      newDiscountPrice: string;
      newMinimumBulkSize: number;
    },
    options: {
      from: string;
      password: string;
      gasLimit?: number;
    }
  ): Promise<string> {
    const result = await sendContractTransaction({
      from: options.from,
      contractAddress: this.contractAddress,
      functionName: 'updatePricing',
      inputs: [
        { name: 'modelId', type: 'bytes32' },
        { name: 'newBasePrice', type: 'uint256' },
        { name: 'newDiscountPrice', type: 'uint256' },
        { name: 'newMinimumBulkSize', type: 'uint256' },
      ],
      args: [
        params.modelId,
        params.newBasePrice,
        params.newDiscountPrice,
        params.newMinimumBulkSize.toString(),
      ],
      password: options.password,
      gasLimit: options.gasLimit || 200000,
    });

    return result.txHash;
  }

  /**
   * Update category for a model
   */
  async updateCategory(
    params: {
      modelId: string;
      newCategory: number;
    },
    options: {
      from: string;
      password: string;
      gasLimit?: number;
    }
  ): Promise<string> {
    const result = await sendContractTransaction({
      from: options.from,
      contractAddress: this.contractAddress,
      functionName: 'updateCategory',
      inputs: [
        { name: 'modelId', type: 'bytes32' },
        { name: 'newCategory', type: 'uint8' },
      ],
      args: [params.modelId, params.newCategory],
      password: options.password,
      gasLimit: options.gasLimit || 200000,
    });

    return result.txHash;
  }

  /**
   * Feature a model (requires payment or admin role)
   */
  async featureModel(
    params: {
      modelId: string;
    },
    options: {
      from: string;
      password: string;
      value?: string; // FEATURED_FEE (1 ETH) if not admin
      gasLimit?: number;
    }
  ): Promise<string> {
    const result = await sendContractTransaction({
      from: options.from,
      contractAddress: this.contractAddress,
      functionName: 'featureModel',
      inputs: [{ name: 'modelId', type: 'bytes32' }],
      args: [params.modelId],
      value: options.value || '0',
      password: options.password,
      gasLimit: options.gasLimit || 200000,
    });

    return result.txHash;
  }

  /**
   * Add a review for a model
   */
  async addReview(
    params: {
      modelId: string;
      rating: number; // 1-5
      comment: string;
    },
    options: {
      from: string;
      password: string;
      gasLimit?: number;
    }
  ): Promise<string> {
    if (params.rating < 1 || params.rating > 5) {
      throw new Error('Rating must be between 1 and 5');
    }

    if (params.comment.length > 500) {
      throw new Error('Comment must be 500 characters or less');
    }

    const result = await sendContractTransaction({
      from: options.from,
      contractAddress: this.contractAddress,
      functionName: 'addReview',
      inputs: [
        { name: 'modelId', type: 'bytes32' },
        { name: 'rating', type: 'uint8' },
        { name: 'comment', type: 'string' },
      ],
      args: [params.modelId, params.rating, params.comment],
      password: options.password,
      gasLimit: options.gasLimit || 300000,
    });

    return result.txHash;
  }

  /**
   * Deactivate a model listing
   */
  async deactivateListing(
    params: {
      modelId: string;
    },
    options: {
      from: string;
      password: string;
      gasLimit?: number;
    }
  ): Promise<string> {
    const result = await sendContractTransaction({
      from: options.from,
      contractAddress: this.contractAddress,
      functionName: 'deactivateListing',
      inputs: [{ name: 'modelId', type: 'bytes32' }],
      args: [params.modelId],
      password: options.password,
      gasLimit: options.gasLimit || 100000,
    });

    return result.txHash;
  }

  /**
   * Activate a model listing
   */
  async activateListing(
    params: {
      modelId: string;
    },
    options: {
      from: string;
      password: string;
      gasLimit?: number;
    }
  ): Promise<string> {
    const result = await sendContractTransaction({
      from: options.from,
      contractAddress: this.contractAddress,
      functionName: 'activateListing',
      inputs: [{ name: 'modelId', type: 'bytes32' }],
      args: [params.modelId],
      password: options.password,
      gasLimit: options.gasLimit || 100000,
    });

    return result.txHash;
  }

  // ============================================================================
  // Read Functions (View/Pure, no gas required)
  // ============================================================================

  /**
   * Get listing details for a model
   */
  async getListing(modelId: string): Promise<ModelListing | null> {
    try {
      const result = await callContractFunction({
        contractAddress: this.contractAddress,
        functionName: 'getListing',
        inputs: [{ name: 'modelId', type: 'bytes32' }],
        outputs: [
          { name: 'listing', type: 'tuple' }, // Full ModelListing struct
        ],
        args: [modelId],
      });

      if (!result.success || result.outputs.length === 0) {
        return null;
      }

      // Parse the tuple into ModelListing
      const listing = result.outputs[0];
      return this.parseListing(listing);
    } catch (error) {
      console.error('[MarketplaceService] Failed to get listing:', error);
      return null;
    }
  }

  /**
   * Get models by category
   */
  async getModelsByCategory(category: number): Promise<string[]> {
    try {
      const result = await callContractFunction({
        contractAddress: this.contractAddress,
        functionName: 'getModelsByCategory',
        inputs: [{ name: 'category', type: 'uint8' }],
        outputs: [{ name: 'modelIds', type: 'bytes32[]' }],
        args: [category],
      });

      if (!result.success || result.outputs.length === 0) {
        return [];
      }

      return result.outputs[0] as string[];
    } catch (error) {
      console.error('[MarketplaceService] Failed to get models by category:', error);
      return [];
    }
  }

  /**
   * Get featured models
   */
  async getFeaturedModels(): Promise<string[]> {
    try {
      const result = await callContractFunction({
        contractAddress: this.contractAddress,
        functionName: 'getFeaturedModels',
        inputs: [],
        outputs: [{ name: 'modelIds', type: 'bytes32[]' }],
        args: [],
      });

      if (!result.success || result.outputs.length === 0) {
        return [];
      }

      return result.outputs[0] as string[];
    } catch (error) {
      console.error('[MarketplaceService] Failed to get featured models:', error);
      return [];
    }
  }

  /**
   * Get top-rated models
   */
  async getTopRatedModels(limit: number = 10): Promise<string[]> {
    try {
      const result = await callContractFunction({
        contractAddress: this.contractAddress,
        functionName: 'getTopRatedModels',
        inputs: [{ name: 'limit', type: 'uint256' }],
        outputs: [{ name: 'modelIds', type: 'bytes32[]' }],
        args: [limit.toString()],
      });

      if (!result.success || result.outputs.length === 0) {
        return [];
      }

      return result.outputs[0] as string[];
    } catch (error) {
      console.error('[MarketplaceService] Failed to get top rated models:', error);
      return [];
    }
  }

  /**
   * Get models by owner
   */
  async getModelsByOwner(ownerAddress: string): Promise<string[]> {
    try {
      const result = await callContractFunction({
        contractAddress: this.contractAddress,
        functionName: 'getModelsByOwner',
        inputs: [{ name: 'owner', type: 'address' }],
        outputs: [{ name: 'modelIds', type: 'bytes32[]' }],
        args: [ownerAddress],
      });

      if (!result.success || result.outputs.length === 0) {
        return [];
      }

      return result.outputs[0] as string[];
    } catch (error) {
      console.error('[MarketplaceService] Failed to get models by owner:', error);
      return [];
    }
  }

  /**
   * Get purchase history for a model
   */
  async getPurchaseHistory(modelId: string): Promise<Purchase[]> {
    try {
      const result = await callContractFunction({
        contractAddress: this.contractAddress,
        functionName: 'getPurchaseHistory',
        inputs: [{ name: 'modelId', type: 'bytes32' }],
        outputs: [{ name: 'purchases', type: 'tuple[]' }],
        args: [modelId],
      });

      if (!result.success || result.outputs.length === 0) {
        return [];
      }

      // Parse array of Purchase structs
      return (result.outputs[0] as any[]).map((p) => this.parsePurchase(p));
    } catch (error) {
      console.error('[MarketplaceService] Failed to get purchase history:', error);
      return [];
    }
  }

  /**
   * Get reviews for a model
   */
  async getModelReviews(modelId: string): Promise<Review[]> {
    try {
      const result = await callContractFunction({
        contractAddress: this.contractAddress,
        functionName: 'getModelReviews',
        inputs: [{ name: 'modelId', type: 'bytes32' }],
        outputs: [{ name: 'reviews', type: 'tuple[]' }],
        args: [modelId],
      });

      if (!result.success || result.outputs.length === 0) {
        return [];
      }

      // Parse array of Review structs
      return (result.outputs[0] as any[]).map((r) => this.parseReview(r));
    } catch (error) {
      console.error('[MarketplaceService] Failed to get model reviews:', error);
      return [];
    }
  }

  /**
   * Get marketplace statistics
   */
  async getMarketplaceStats(): Promise<MarketplaceStats> {
    try {
      const result = await callContractFunction({
        contractAddress: this.contractAddress,
        functionName: 'getMarketplaceStats',
        inputs: [],
        outputs: [
          { name: 'totalListings', type: 'uint256' },
          { name: 'totalSales', type: 'uint256' },
          { name: 'totalVolume', type: 'uint256' },
        ],
        args: [],
      });

      if (!result.success || result.outputs.length < 3) {
        return {
          totalListings: '0',
          totalSales: '0',
          totalVolume: '0',
        };
      }

      return {
        totalListings: result.outputs[0].toString(),
        totalSales: result.outputs[1].toString(),
        totalVolume: result.outputs[2].toString(),
      };
    } catch (error) {
      console.error('[MarketplaceService] Failed to get marketplace stats:', error);
      return {
        totalListings: '0',
        totalSales: '0',
        totalVolume: '0',
      };
    }
  }

  // ============================================================================
  // Helper Methods
  // ============================================================================

  /**
   * Parse ModelListing struct from contract response
   */
  private parseListing(data: any): ModelListing {
    return {
      modelId: data[0],
      owner: data[1],
      basePrice: data[2].toString(),
      discountPrice: data[3].toString(),
      minimumBulkSize: data[4].toString(),
      totalSales: data[5].toString(),
      totalRevenue: data[6].toString(),
      category: Number(data[7]),
      metadataURI: data[8],
      featured: data[9],
      active: data[10],
      listedAt: data[11].toString(),
      lastSaleAt: data[12].toString(),
      totalRating: data[13].toString(),
      reviewCount: data[14].toString(),
      averageRating: data[15].toString(),
    };
  }

  /**
   * Parse Purchase struct from contract response
   */
  private parsePurchase(data: any): Purchase {
    return {
      modelId: data[0],
      buyer: data[1],
      price: data[2].toString(),
      quantity: data[3].toString(),
      timestamp: data[4].toString(),
      bulkDiscount: data[5],
    };
  }

  /**
   * Parse Review struct from contract response
   */
  private parseReview(data: any): Review {
    return {
      reviewer: data[0],
      modelId: data[1],
      rating: Number(data[2]),
      comment: data[3],
      timestamp: data[4].toString(),
      verified: data[5],
    };
  }

  /**
   * Calculate total price for purchasing model access
   */
  calculatePurchasePrice(
    listing: ModelListing,
    quantity: number
  ): { pricePerInference: bigint; totalPrice: bigint; bulkDiscount: boolean } {
    const bulkDiscount = quantity >= Number(listing.minimumBulkSize);
    const pricePerInference = bulkDiscount
      ? BigInt(listing.discountPrice)
      : BigInt(listing.basePrice);
    const totalPrice = pricePerInference * BigInt(quantity);

    return {
      pricePerInference,
      totalPrice,
      bulkDiscount,
    };
  }

  /**
   * Format average rating (stored as percentage, e.g., 480 = 4.8 stars)
   */
  formatAverageRating(averageRating: string): number {
    return Number(averageRating) / 100;
  }
}

/**
 * Create a marketplace service instance
 */
export function createMarketplaceService(contractAddress: string): MarketplaceService {
  return new MarketplaceService(contractAddress);
}

/**
 * Deployed contract addresses on testnet (Chain ID: 1337)
 *
 * ModelMarketplace: 0xC9F9A1e0C2822663e31c0fCdF46aF0dc10081423
 * ModelRegistry: 0x2785033dd812eEd5815BD7B8Dcbf43337C192DED
 * Deployer: 0xFCAd0B19bB29D4674531d6f115237E16AfCE377c
 */
export const DEPLOYED_MARKETPLACE_ADDRESS = '0xC9F9A1e0C2822663e31c0fCdF46aF0dc10081423';

/**
 * Default marketplace service (will be initialized with deployed contract address)
 */
let defaultMarketplaceService: MarketplaceService | null = null;

/**
 * Initialize the default marketplace service
 * If no address provided, uses the deployed testnet address
 */
export function initMarketplaceService(contractAddress?: string): MarketplaceService {
  const address = contractAddress || DEPLOYED_MARKETPLACE_ADDRESS;
  const service = new MarketplaceService(address);
  defaultMarketplaceService = service;
  return service;
}

/**
 * Get the default marketplace service instance
 */
export function getMarketplaceService(): MarketplaceService {
  if (!defaultMarketplaceService) {
    throw new Error(
      'Marketplace service not initialized. Call initMarketplaceService() first.'
    );
  }
  return defaultMarketplaceService;
}

// Export constants from the contract
export const MARKETPLACE_FEE_BASIS_POINTS = 250; // 2.5%
export const MIN_PRICE = '1000000000000000'; // 0.001 ETH in wei
export const MAX_PRICE = '1000000000000000000000'; // 1000 ETH in wei
export const FEATURED_FEE = '1000000000000000000'; // 1 ETH in wei
export const BASIS_POINTS_DENOMINATOR = 10000;

export const CATEGORIES = [
  { id: 0, name: 'Language Models' },
  { id: 1, name: 'Image Generation' },
  { id: 2, name: 'Computer Vision' },
  { id: 3, name: 'Audio Processing' },
  { id: 4, name: 'Code Generation' },
  { id: 5, name: 'Embeddings' },
  { id: 6, name: 'Classification' },
  { id: 7, name: 'Translation' },
  { id: 8, name: 'Summarization' },
  { id: 9, name: 'Question Answering' },
  { id: 10, name: 'Other' },
];
