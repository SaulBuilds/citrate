/**
 * Citrate Model Marketplace Search Module
 *
 * Entry point for all search-related functionality:
 * - Type definitions
 * - IPFS metadata fetching
 * - Search index building
 * - FlexSearch integration
 */

// Export types
export type {
  SearchDocument,
  SearchIndex,
  SearchQuery,
  SearchFilters,
  SearchResult,
  SearchResponse,
  SearchSuggestion,
  SearchHighlights,
  SearchStatistics,
  IndexUpdateEvent,
  FlexSearchConfig,
  IndexBuilderOptions,
  SortOption
} from './types';

export {
  ModelCategory,
  ModelSize,
  CATEGORY_INFO,
  SIZE_INFO,
  categorizeModelSize,
  formatModelSize,
  formatPrice
} from './types';

// Export IPFS metadata fetcher
export {
  IPFSMetadataFetcher,
  DEFAULT_GATEWAYS,
  DEFAULT_RETRY_CONFIG,
  ipfsMetadataFetcher
} from './ipfsMetadataFetcher';

export type {
  IPFSGatewayConfig,
  RetryConfig,
  RawIPFSMetadata,
  ValidationResult
} from './ipfsMetadataFetcher';

// Export search index builder
export {
  SearchIndexBuilder,
  initializeSearchIndex,
  getSearchIndexBuilder,
  disposeSearchIndex
} from './searchIndexBuilder';

// Export search engine
export {
  SearchEngine,
  initializeSearchEngine,
  getSearchEngine,
  disposeSearchEngine
} from './searchEngine';
