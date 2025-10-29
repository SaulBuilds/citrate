/**
 * Search Types for Citrate Model Marketplace
 *
 * Defines the schema and types for model search and discovery functionality.
 * These types support full-text search, filtering, sorting, and indexing.
 */

export enum ModelCategory {
  LANGUAGE_MODELS = 'language-models',
  CODE_MODELS = 'code-models',
  VISION_MODELS = 'vision-models',
  EMBEDDING_MODELS = 'embedding-models',
  MULTIMODAL_MODELS = 'multimodal-models',
  GENERATIVE_MODELS = 'generative-models',
  AUDIO_MODELS = 'audio-models',
  REINFORCEMENT_LEARNING = 'reinforcement-learning',
  TIME_SERIES = 'time-series',
  TABULAR_MODELS = 'tabular-models',
  OTHER = 'other'
}

export enum ModelSize {
  TINY = 'tiny',           // < 1GB
  SMALL = 'small',         // 1-5GB
  MEDIUM = 'medium',       // 5-20GB
  LARGE = 'large',         // 20-100GB
  XLARGE = 'xlarge'        // > 100GB
}

/**
 * Search Document - The core indexed document for each model
 * Weight indicates importance for text search relevance
 */
export interface SearchDocument {
  // Core identifiers
  modelId: string;           // Unique identifier (modelHash from contract)

  // Searchable text fields (with weights)
  name: string;              // Weight: 10 (highest)
  description: string;       // Weight: 5
  tags: string[];            // Weight: 7
  category: ModelCategory;   // Weight: 3
  framework: string;         // Weight: 2 (e.g., "PyTorch", "TensorFlow")

  // Creator information
  creatorAddress: string;    // Exact match only
  creatorName?: string;      // Weight: 1 (optional ENS/display name)

  // Pricing metadata
  basePrice: number;         // In wei
  discountPrice: number;     // In wei

  // Quality indicators
  averageRating: number;     // 0-500 (stored as rating * 100)
  reviewCount: number;
  totalSales: number;        // Number of purchases
  totalInferences: number;   // Usage count

  // Status flags
  isActive: boolean;
  featured: boolean;

  // Timestamps
  listedAt: number;          // Unix timestamp
  lastSaleAt?: number;       // Unix timestamp

  // Computed metrics
  qualityScore: number;      // 0-100 (pre-computed composite score)

  // IPFS references
  metadataURI: string;       // IPFS URI for rich metadata
  ipfsCID?: string;          // Extracted CID for direct access

  // Model technical details
  sizeBytes?: number;        // Model size in bytes
  modelSize?: ModelSize;     // Categorized size
  version?: string;          // Model version string
}

/**
 * Search Index - In-memory search index structure
 */
export interface SearchIndex {
  documents: Map<string, SearchDocument>;
  lastUpdated: number;       // Unix timestamp
  version: string;           // Semantic version of index format
  totalDocuments: number;
}

/**
 * Search Query - User search request
 */
export interface SearchQuery {
  text?: string;             // Full-text search query
  filters?: SearchFilters;
  sort?: SortOption;
  page?: number;             // Zero-indexed
  pageSize?: number;         // Results per page (default: 20)
}

/**
 * Search Filters - Advanced filtering options
 */
export interface SearchFilters {
  categories?: ModelCategory[];
  priceMin?: number;         // In wei
  priceMax?: number;         // In wei
  ratingMin?: number;        // 0-5
  frameworks?: string[];     // e.g., ["PyTorch", "TensorFlow"]
  modelSizes?: ModelSize[];
  activeOnly?: boolean;      // Default: true
  featuredOnly?: boolean;
  creatorAddress?: string;   // Filter by specific creator
  minSales?: number;         // Minimum number of sales
  minInferences?: number;    // Minimum usage count
}

/**
 * Sort Options - Available sorting methods
 */
export type SortOption =
  | 'relevance'        // Default for text search (FlexSearch score)
  | 'rating_desc'      // Highest rated first
  | 'rating_asc'       // Lowest rated first
  | 'price_desc'       // Most expensive first
  | 'price_asc'        // Cheapest first
  | 'popularity'       // Most sales + inferences
  | 'recent'           // Recently listed
  | 'trending';        // Recent sales velocity

/**
 * Search Result - Single result from search
 */
export interface SearchResult {
  document: SearchDocument;
  score?: number;            // Relevance score (0-1) for text search
  highlights?: SearchHighlights;
}

/**
 * Search Highlights - Matched text snippets
 */
export interface SearchHighlights {
  name?: string[];
  description?: string[];
  tags?: string[];
}

/**
 * Search Response - Complete search results
 */
export interface SearchResponse {
  results: SearchResult[];
  total: number;             // Total matching documents
  page: number;
  pageSize: number;
  query: SearchQuery;
  executionTimeMs: number;
}

/**
 * Search Suggestions - Autocomplete results
 */
export interface SearchSuggestion {
  text: string;
  type: 'model' | 'tag' | 'framework' | 'creator';
  modelId?: string;          // If type === 'model'
  count?: number;            // Frequency for tags/frameworks
}

/**
 * Index Update Event - Emitted when index changes
 */
export interface IndexUpdateEvent {
  type: 'added' | 'updated' | 'removed';
  modelId: string;
  timestamp: number;
}

/**
 * Search Statistics - Metrics about the search index
 */
export interface SearchStatistics {
  totalModels: number;
  activeModels: number;
  featuredModels: number;
  categoryCounts: Record<ModelCategory, number>;
  frameworkCounts: Record<string, number>;
  avgQualityScore: number;
  indexSize: number;         // Bytes
  lastRebuildTime: number;   // Unix timestamp
}

/**
 * FlexSearch Configuration
 */
export interface FlexSearchConfig {
  tokenize: 'forward' | 'strict' | 'full';
  threshold: number;         // Fuzzy matching threshold (0-9)
  resolution: number;        // Context resolution (1-9)
  depth: number;             // Contextual depth (0-9)
}

/**
 * Search Index Builder Options
 */
export interface IndexBuilderOptions {
  batchSize?: number;        // Models to process per batch
  debounceMs?: number;       // Debounce time for updates
  maxRetries?: number;       // IPFS fetch retries
  ipfsTimeout?: number;      // Timeout in ms for IPFS requests
}

/**
 * Model Category Display Info
 */
export const CATEGORY_INFO: Record<ModelCategory, { label: string; icon: string; description: string }> = {
  [ModelCategory.LANGUAGE_MODELS]: {
    label: 'Language Models',
    icon: '💬',
    description: 'Text generation, completion, and understanding'
  },
  [ModelCategory.CODE_MODELS]: {
    label: 'Code Models',
    icon: '💻',
    description: 'Code generation, completion, and analysis'
  },
  [ModelCategory.VISION_MODELS]: {
    label: 'Vision Models',
    icon: '👁️',
    description: 'Image classification, detection, and segmentation'
  },
  [ModelCategory.EMBEDDING_MODELS]: {
    label: 'Embedding Models',
    icon: '🔢',
    description: 'Vector embeddings for similarity search'
  },
  [ModelCategory.MULTIMODAL_MODELS]: {
    label: 'Multimodal Models',
    icon: '🎭',
    description: 'Vision-language and cross-modal understanding'
  },
  [ModelCategory.GENERATIVE_MODELS]: {
    label: 'Generative Models',
    icon: '🎨',
    description: 'Image, audio, and content generation'
  },
  [ModelCategory.AUDIO_MODELS]: {
    label: 'Audio Models',
    icon: '🔊',
    description: 'Speech recognition, synthesis, and audio processing'
  },
  [ModelCategory.REINFORCEMENT_LEARNING]: {
    label: 'Reinforcement Learning',
    icon: '🎮',
    description: 'Policy networks and decision making'
  },
  [ModelCategory.TIME_SERIES]: {
    label: 'Time Series',
    icon: '📈',
    description: 'Forecasting and temporal pattern recognition'
  },
  [ModelCategory.TABULAR_MODELS]: {
    label: 'Tabular Models',
    icon: '📊',
    description: 'Structured data analysis and prediction'
  },
  [ModelCategory.OTHER]: {
    label: 'Other',
    icon: '🔬',
    description: 'Specialized and experimental models'
  }
};

/**
 * Model Size Display Info
 */
export const SIZE_INFO: Record<ModelSize, { label: string; range: string; color: string }> = {
  [ModelSize.TINY]: {
    label: 'Tiny',
    range: '< 1GB',
    color: '#10b981' // green
  },
  [ModelSize.SMALL]: {
    label: 'Small',
    range: '1-5GB',
    color: '#3b82f6' // blue
  },
  [ModelSize.MEDIUM]: {
    label: 'Medium',
    range: '5-20GB',
    color: '#f59e0b' // amber
  },
  [ModelSize.LARGE]: {
    label: 'Large',
    range: '20-100GB',
    color: '#ef4444' // red
  },
  [ModelSize.XLARGE]: {
    label: 'X-Large',
    range: '> 100GB',
    color: '#7c3aed' // purple
  }
};

/**
 * Helper function to categorize model size
 */
export function categorizeModelSize(sizeBytes: number): ModelSize {
  const GB = 1024 * 1024 * 1024;

  if (sizeBytes < GB) return ModelSize.TINY;
  if (sizeBytes < 5 * GB) return ModelSize.SMALL;
  if (sizeBytes < 20 * GB) return ModelSize.MEDIUM;
  if (sizeBytes < 100 * GB) return ModelSize.LARGE;
  return ModelSize.XLARGE;
}

/**
 * Helper function to format model size for display
 */
export function formatModelSize(sizeBytes: number): string {
  const KB = 1024;
  const MB = KB * 1024;
  const GB = MB * 1024;
  const TB = GB * 1024;

  if (sizeBytes < KB) return `${sizeBytes} B`;
  if (sizeBytes < MB) return `${(sizeBytes / KB).toFixed(2)} KB`;
  if (sizeBytes < GB) return `${(sizeBytes / MB).toFixed(2)} MB`;
  if (sizeBytes < TB) return `${(sizeBytes / GB).toFixed(2)} GB`;
  return `${(sizeBytes / TB).toFixed(2)} TB`;
}

/**
 * Helper function to format price in ETH
 */
export function formatPrice(priceWei: number): string {
  const eth = priceWei / 1e18;
  if (eth < 0.001) return `${(priceWei / 1e15).toFixed(2)} Finney`;
  if (eth < 1) return `${eth.toFixed(4)} ETH`;
  return `${eth.toFixed(2)} ETH`;
}
