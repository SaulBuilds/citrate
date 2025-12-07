/**
 * Search Engine for Citrate Model Marketplace
 *
 * Provides full-text search, filtering, and sorting using FlexSearch.
 * Integrates with SearchIndexBuilder for real-time index updates.
 */

import FlexSearch, { Document } from 'flexsearch';
import {
  SearchDocument,
  SearchQuery,
  SearchFilters,
  SearchResult,
  SearchResponse,
  SearchSuggestion,
  SortOption,
  ModelCategory,
  FlexSearchConfig
} from './types';
import { SearchIndexBuilder } from './searchIndexBuilder';

// ============================================================================
// FlexSearch Document Index Configuration
// ============================================================================

const DEFAULT_FLEXSEARCH_CONFIG: FlexSearchConfig = {
  tokenize: 'forward',
  threshold: 6,
  resolution: 9,
  depth: 3
};

// ============================================================================
// Search Engine
// ============================================================================

export class SearchEngine {
  private indexBuilder: SearchIndexBuilder;
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  private flexIndex: Document<SearchDocument, any>;
  private config: FlexSearchConfig;

  constructor(indexBuilder: SearchIndexBuilder, config?: Partial<FlexSearchConfig>) {
    this.indexBuilder = indexBuilder;
    this.config = { ...DEFAULT_FLEXSEARCH_CONFIG, ...config };

    // Initialize FlexSearch with weighted fields
    // Using 'as any' because FlexSearch types don't match runtime API
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    this.flexIndex = new (FlexSearch as any).Document({
      tokenize: this.config.tokenize,
      threshold: this.config.threshold,
      resolution: this.config.resolution,
      depth: this.config.depth,
      document: {
        id: 'modelId',
        index: ['name', 'tags', 'description', 'category', 'framework', 'creatorName']
      }
    });

    // Build initial FlexSearch index
    this.buildFlexIndex();
  }

  /**
   * Build FlexSearch index from current documents
   */
  private buildFlexIndex(): void {
    const documents = this.indexBuilder.getAllDocuments();

    console.log(`Building FlexSearch index with ${documents.length} documents...`);

    for (const doc of documents) {
      this.flexIndex.add(doc);
    }

    console.log('FlexSearch index built successfully');
  }

  /**
   * Rebuild FlexSearch index (call after index updates)
   */
  rebuildFlexIndex(): void {
    // Clear existing index
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    this.flexIndex = new (FlexSearch as any).Document({
      tokenize: this.config.tokenize,
      threshold: this.config.threshold,
      resolution: this.config.resolution,
      depth: this.config.depth,
      document: {
        id: 'modelId',
        index: ['name', 'tags', 'description', 'category', 'framework', 'creatorName']
      }
    });

    this.buildFlexIndex();
  }

  /**
   * Perform a search
   */
  async search(query: SearchQuery): Promise<SearchResponse> {
    const startTime = performance.now();

    let results: SearchDocument[] = [];

    // Full-text search if query text provided
    if (query.text && query.text.trim().length > 0) {
      results = await this.performTextSearch(query.text);
    } else {
      // No text query - return all active documents
      results = this.indexBuilder.getAllDocuments().filter(d => d.isActive);
    }

    // Apply filters
    if (query.filters) {
      results = this.applyFilters(results, query.filters);
    }

    // Calculate total before pagination
    const total = results.length;

    // Apply sorting
    const sortOption = query.sort || 'relevance';
    results = this.applySorting(results, sortOption, !!query.text);

    // Apply pagination
    const page = query.page || 0;
    const pageSize = query.pageSize || 20;
    const startIdx = page * pageSize;
    const endIdx = startIdx + pageSize;
    const paginatedResults = results.slice(startIdx, endIdx);

    // Convert to SearchResult format
    const searchResults: SearchResult[] = paginatedResults.map(doc => ({
      document: doc,
      score: undefined,  // FlexSearch doesn't expose scores directly
      highlights: undefined  // Could implement highlighting later
    }));

    const executionTimeMs = performance.now() - startTime;

    return {
      results: searchResults,
      total,
      page,
      pageSize,
      query,
      executionTimeMs
    };
  }

  /**
   * Perform full-text search using FlexSearch
   */
  private async performTextSearch(text: string): Promise<SearchDocument[]> {
    // Search across all indexed fields
    const searchResults = await this.flexIndex.search(text, {
      limit: 1000,  // Get all matches, we'll paginate later
      enrich: true
    });

    // FlexSearch returns results grouped by field
    // We need to combine them and deduplicate by modelId
    const documentMap = new Map<string, SearchDocument>();

    for (const fieldResults of searchResults) {
      if (Array.isArray(fieldResults.result)) {
        for (const item of fieldResults.result) {
          const doc = item as any as SearchDocument;
          if (!documentMap.has(doc.modelId)) {
            documentMap.set(doc.modelId, doc);
          }
        }
      }
    }

    return Array.from(documentMap.values());
  }

  /**
   * Apply filters to search results
   */
  private applyFilters(documents: SearchDocument[], filters: SearchFilters): SearchDocument[] {
    let filtered = documents;

    // Category filter
    if (filters.categories && filters.categories.length > 0) {
      filtered = filtered.filter(doc => filters.categories!.includes(doc.category));
    }

    // Price range filter
    if (filters.priceMin !== undefined) {
      filtered = filtered.filter(doc =>
        Math.min(doc.basePrice, doc.discountPrice) >= filters.priceMin!
      );
    }
    if (filters.priceMax !== undefined) {
      filtered = filtered.filter(doc =>
        Math.min(doc.basePrice, doc.discountPrice) <= filters.priceMax!
      );
    }

    // Rating filter
    if (filters.ratingMin !== undefined) {
      const minRating = filters.ratingMin * 100;  // Convert 0-5 to 0-500
      filtered = filtered.filter(doc => doc.averageRating >= minRating);
    }

    // Framework filter
    if (filters.frameworks && filters.frameworks.length > 0) {
      filtered = filtered.filter(doc =>
        filters.frameworks!.some(fw =>
          doc.framework.toLowerCase().includes(fw.toLowerCase())
        )
      );
    }

    // Model size filter
    if (filters.modelSizes && filters.modelSizes.length > 0) {
      filtered = filtered.filter(doc =>
        doc.modelSize && filters.modelSizes!.includes(doc.modelSize)
      );
    }

    // Active only filter (default: true)
    if (filters.activeOnly !== false) {
      filtered = filtered.filter(doc => doc.isActive);
    }

    // Featured only filter
    if (filters.featuredOnly) {
      filtered = filtered.filter(doc => doc.featured);
    }

    // Creator filter
    if (filters.creatorAddress) {
      filtered = filtered.filter(doc =>
        doc.creatorAddress.toLowerCase() === filters.creatorAddress!.toLowerCase()
      );
    }

    // Minimum sales filter
    if (filters.minSales !== undefined) {
      filtered = filtered.filter(doc => doc.totalSales >= filters.minSales!);
    }

    // Minimum inferences filter
    if (filters.minInferences !== undefined) {
      filtered = filtered.filter(doc => doc.totalInferences >= filters.minInferences!);
    }

    return filtered;
  }

  /**
   * Apply sorting to search results
   */
  private applySorting(
    documents: SearchDocument[],
    sortOption: SortOption,
    hasTextQuery: boolean
  ): SearchDocument[] {
    const sorted = [...documents];

    switch (sortOption) {
      case 'relevance':
        // For relevance, FlexSearch already sorted by relevance
        // If no text query, sort by quality score instead
        if (!hasTextQuery) {
          sorted.sort((a, b) => b.qualityScore - a.qualityScore);
        }
        break;

      case 'rating_desc':
        sorted.sort((a, b) => b.averageRating - a.averageRating);
        break;

      case 'rating_asc':
        sorted.sort((a, b) => a.averageRating - b.averageRating);
        break;

      case 'price_desc':
        sorted.sort((a, b) => {
          const priceA = Math.min(a.basePrice, a.discountPrice);
          const priceB = Math.min(b.basePrice, b.discountPrice);
          return priceB - priceA;
        });
        break;

      case 'price_asc':
        sorted.sort((a, b) => {
          const priceA = Math.min(a.basePrice, a.discountPrice);
          const priceB = Math.min(b.basePrice, b.discountPrice);
          return priceA - priceB;
        });
        break;

      case 'popularity':
        // Popularity = sales + inferences
        sorted.sort((a, b) => {
          const popA = a.totalSales + a.totalInferences;
          const popB = b.totalSales + b.totalInferences;
          return popB - popA;
        });
        break;

      case 'recent':
        sorted.sort((a, b) => b.listedAt - a.listedAt);
        break;

      case 'trending':
        // Trending = recent sales velocity
        // Use lastSaleAt with sales count as proxy
        sorted.sort((a, b) => {
          const trendA = (a.lastSaleAt || 0) + a.totalSales * 1000;
          const trendB = (b.lastSaleAt || 0) + b.totalSales * 1000;
          return trendB - trendA;
        });
        break;
    }

    return sorted;
  }

  /**
   * Get autocomplete suggestions
   */
  async getSuggestions(query: string, limit: number = 10): Promise<SearchSuggestion[]> {
    if (!query || query.trim().length < 2) {
      return [];
    }

    const suggestions: SearchSuggestion[] = [];

    // Search for model name matches
    const modelResults = await this.flexIndex.search(query, {
      field: 'name',
      limit: 5,
      enrich: true
    });

    for (const fieldResults of modelResults) {
      if (Array.isArray(fieldResults.result)) {
        for (const item of fieldResults.result) {
          const doc = item as any as SearchDocument;
          suggestions.push({
            text: doc.name,
            type: 'model',
            modelId: doc.modelId
          });
        }
      }
    }

    // Get tag suggestions
    const allDocs = this.indexBuilder.getAllDocuments();
    const tagCounts = new Map<string, number>();

    for (const doc of allDocs) {
      for (const tag of doc.tags) {
        if (tag.toLowerCase().includes(query.toLowerCase())) {
          tagCounts.set(tag, (tagCounts.get(tag) || 0) + 1);
        }
      }
    }

    // Add top tags
    const sortedTags = Array.from(tagCounts.entries())
      .sort((a, b) => b[1] - a[1])
      .slice(0, 3);

    for (const [tag, count] of sortedTags) {
      suggestions.push({
        text: tag,
        type: 'tag',
        count
      });
    }

    // Get framework suggestions
    const frameworkCounts = new Map<string, number>();

    for (const doc of allDocs) {
      if (doc.framework.toLowerCase().includes(query.toLowerCase())) {
        frameworkCounts.set(doc.framework, (frameworkCounts.get(doc.framework) || 0) + 1);
      }
    }

    const sortedFrameworks = Array.from(frameworkCounts.entries())
      .sort((a, b) => b[1] - a[1])
      .slice(0, 2);

    for (const [framework, count] of sortedFrameworks) {
      suggestions.push({
        text: framework,
        type: 'framework',
        count
      });
    }

    return suggestions.slice(0, limit);
  }

  /**
   * Get featured models
   */
  getFeaturedModels(limit: number = 6): SearchDocument[] {
    const featured = this.indexBuilder
      .getAllDocuments()
      .filter(doc => doc.featured && doc.isActive)
      .sort((a, b) => b.qualityScore - a.qualityScore)
      .slice(0, limit);

    return featured;
  }

  /**
   * Get trending models
   */
  getTrendingModels(limit: number = 6): SearchDocument[] {
    const now = Date.now();
    const thirtyDaysAgo = now - 30 * 24 * 60 * 60 * 1000;

    const trending = this.indexBuilder
      .getAllDocuments()
      .filter(doc => doc.isActive)
      .map(doc => {
        // Calculate trend score
        const recentSales = doc.lastSaleAt && doc.lastSaleAt > thirtyDaysAgo ? doc.totalSales : 0;
        const trendScore = recentSales * 10 + doc.totalInferences / 100 + doc.reviewCount;
        return { doc, trendScore };
      })
      .sort((a, b) => b.trendScore - a.trendScore)
      .slice(0, limit)
      .map(item => item.doc);

    return trending;
  }

  /**
   * Get new models
   */
  getNewModels(limit: number = 6): SearchDocument[] {
    const newModels = this.indexBuilder
      .getAllDocuments()
      .filter(doc => doc.isActive)
      .sort((a, b) => b.listedAt - a.listedAt)
      .slice(0, limit);

    return newModels;
  }

  /**
   * Get top rated models
   */
  getTopRatedModels(limit: number = 6, minReviews: number = 5): SearchDocument[] {
    const topRated = this.indexBuilder
      .getAllDocuments()
      .filter(doc => doc.isActive && doc.reviewCount >= minReviews)
      .sort((a, b) => {
        // Sort by rating, then by review count
        if (b.averageRating !== a.averageRating) {
          return b.averageRating - a.averageRating;
        }
        return b.reviewCount - a.reviewCount;
      })
      .slice(0, limit);

    return topRated;
  }

  /**
   * Get models by category
   */
  getModelsByCategory(category: ModelCategory, limit?: number): SearchDocument[] {
    const models = this.indexBuilder
      .getAllDocuments()
      .filter(doc => doc.isActive && doc.category === category)
      .sort((a, b) => b.qualityScore - a.qualityScore);

    return limit ? models.slice(0, limit) : models;
  }

  /**
   * Get models by creator
   */
  getModelsByCreator(creatorAddress: string, limit?: number): SearchDocument[] {
    const models = this.indexBuilder
      .getAllDocuments()
      .filter(doc =>
        doc.creatorAddress.toLowerCase() === creatorAddress.toLowerCase()
      )
      .sort((a, b) => b.listedAt - a.listedAt);

    return limit ? models.slice(0, limit) : models;
  }

  /**
   * Get recommended models based on a given model
   */
  getRecommendedModels(modelId: string, limit: number = 6): SearchDocument[] {
    const targetModel = this.indexBuilder.getDocument(modelId);
    if (!targetModel) {
      return [];
    }

    const allModels = this.indexBuilder
      .getAllDocuments()
      .filter(doc => doc.isActive && doc.modelId !== modelId);

    // Calculate similarity scores
    const scored = allModels.map(doc => {
      let score = 0;

      // Same category: +10
      if (doc.category === targetModel.category) {
        score += 10;
      }

      // Same framework: +5
      if (doc.framework === targetModel.framework) {
        score += 5;
      }

      // Shared tags: +2 per tag
      const sharedTags = targetModel.tags.filter(tag => doc.tags.includes(tag));
      score += sharedTags.length * 2;

      // Similar quality score: +0-5
      const qualityDiff = Math.abs(doc.qualityScore - targetModel.qualityScore);
      score += Math.max(0, 5 - qualityDiff / 20);

      // Similar price range: +0-3
      const targetPrice = Math.min(targetModel.basePrice, targetModel.discountPrice);
      const docPrice = Math.min(doc.basePrice, doc.discountPrice);
      const priceDiff = Math.abs(docPrice - targetPrice);
      score += Math.max(0, 3 - priceDiff / targetPrice);

      return { doc, score };
    });

    // Sort by score and return top N
    const recommended = scored
      .sort((a, b) => b.score - a.score)
      .slice(0, limit)
      .map(item => item.doc);

    return recommended;
  }

  /**
   * Get model by ID
   */
  getModelById(modelId: string): SearchDocument | undefined {
    return this.indexBuilder.getDocument(modelId);
  }

  /**
   * Get search statistics
   */
  getStatistics() {
    return this.indexBuilder.getStatistics();
  }
}

// ============================================================================
// Export singleton
// ============================================================================

let globalSearchEngine: SearchEngine | null = null;

export function initializeSearchEngine(
  indexBuilder?: SearchIndexBuilder,
  config?: Partial<FlexSearchConfig>
): SearchEngine {
  if (globalSearchEngine) {
    console.warn('Search engine already initialized, creating new instance');
  }

  // If no index builder provided, try to get the global one
  const builder = indexBuilder || (globalSearchEngine as SearchEngine | null)?.['indexBuilder'];
  if (!builder) {
    // Create a minimal index builder for standalone use
    const { getSearchIndexBuilder } = require('./searchIndexBuilder');
    const existingBuilder = getSearchIndexBuilder();
    if (existingBuilder) {
      globalSearchEngine = new SearchEngine(existingBuilder, config);
    } else {
      throw new Error('No index builder available. Call initializeSearchIndex first.');
    }
  } else {
    globalSearchEngine = new SearchEngine(builder, config);
  }
  return globalSearchEngine;
}

export function getSearchEngine(): SearchEngine | null {
  return globalSearchEngine;
}

export function disposeSearchEngine(): void {
  globalSearchEngine = null;
}
