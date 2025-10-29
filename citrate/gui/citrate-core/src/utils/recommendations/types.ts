/**
 * Recommendation System Types
 *
 * Defines interfaces for recommendation algorithms, user tracking,
 * and similarity calculations for the model marketplace.
 */

import { SearchDocument } from '../search/types';

/**
 * User Interaction - Tracks user behavior for personalized recommendations
 */
export interface UserInteraction {
  userAddress: string;
  modelId: string;
  type: 'view' | 'purchase' | 'inference';
  timestamp: number;
  metadata?: {
    duration?: number;      // Time spent viewing (seconds)
    fromSearch?: boolean;   // Came from search?
    searchQuery?: string;   // What they searched for
  };
}

/**
 * Recommendation Score - Individual model recommendation with reasoning
 */
export interface RecommendationScore {
  modelId: string;
  score: number;          // 0-100
  reason: string;         // Human-readable explanation
  algorithm: RecommendationAlgorithm;
  metadata?: {
    similarityScore?: number;
    coOccurrenceCount?: number;
    categoryMatch?: boolean;
  };
}

/**
 * Recommendation algorithms available
 */
export type RecommendationAlgorithm =
  | 'content-based'      // Similar models based on attributes
  | 'collaborative'      // Users who bought X also bought Y
  | 'trending'           // Hot models right now
  | 'personalized'       // Based on user's history
  | 'category'           // Popular in same category
  | 'creator';           // Other models by same creator

/**
 * Similarity Weights - Configurable weights for content-based filtering
 */
export interface SimilarityWeights {
  category: number;      // Default: 0.40 (40%)
  tags: number;          // Default: 0.30 (30%)
  framework: number;     // Default: 0.15 (15%)
  modelSize: number;     // Default: 0.15 (15%)
}

/**
 * Default similarity weights
 */
export const DEFAULT_SIMILARITY_WEIGHTS: SimilarityWeights = {
  category: 0.40,
  tags: 0.30,
  framework: 0.15,
  modelSize: 0.15
};

/**
 * Trending Score - Calculated trending metrics
 */
export interface TrendingScore {
  modelId: string;
  sales: number;
  inferences: number;
  daysSinceListing: number;
  trendScore: number;    // Composite score
  velocity: number;      // Recent activity rate
  momentum: number;      // Acceleration of activity
}

/**
 * Co-purchase Data - Models bought together
 */
export interface CoPurchaseData {
  modelIdA: string;
  modelIdB: string;
  count: number;         // Times purchased together
  percentage: number;    // % of A buyers who also bought B
  confidence: number;    // Statistical confidence score
}

/**
 * User Profile - Derived preferences from history
 */
export interface UserProfile {
  userAddress: string;
  favoriteCategories: Map<string, number>;  // Category -> frequency
  favoriteTags: Map<string, number>;        // Tag -> frequency
  favoriteFrameworks: Map<string, number>;  // Framework -> frequency
  avgPriceRange: { min: number; max: number };
  recentActivity: UserInteraction[];
  totalPurchases: number;
  totalInferences: number;
  firstSeen: number;     // Unix timestamp
  lastSeen: number;      // Unix timestamp
}

/**
 * Recommendation Context - Input parameters for recommendations
 */
export interface RecommendationContext {
  modelId?: string;           // For similar/related recommendations
  userAddress?: string;       // For personalized recommendations
  excludeModelIds?: string[]; // Models to exclude from results
  limit?: number;             // Max results to return
  minScore?: number;          // Minimum recommendation score
  algorithms?: RecommendationAlgorithm[]; // Specific algorithms to use
}

/**
 * Recommendation Result - Full recommendation response
 */
export interface RecommendationResult {
  recommendations: RecommendationScore[];
  context: RecommendationContext;
  totalCandidates: number;
  executionTimeMs: number;
  algorithmsUsed: RecommendationAlgorithm[];
}

/**
 * Jaccard Similarity Result
 */
export interface JaccardSimilarity {
  similarity: number;    // 0-1
  intersection: number;  // Count of common elements
  union: number;         // Count of total unique elements
}

/**
 * Time Window - For trending calculations
 */
export type TimeWindow = '24h' | '7d' | '30d' | '90d';

/**
 * Time window in milliseconds
 */
export const TIME_WINDOW_MS: Record<TimeWindow, number> = {
  '24h': 24 * 60 * 60 * 1000,
  '7d': 7 * 24 * 60 * 60 * 1000,
  '30d': 30 * 24 * 60 * 60 * 1000,
  '90d': 90 * 24 * 60 * 60 * 1000
};

/**
 * Recommendation Cache Entry
 */
export interface RecommendationCacheEntry {
  key: string;
  result: RecommendationResult;
  timestamp: number;
  expiresAt: number;
}

/**
 * Recommendation Engine Config
 */
export interface RecommendationEngineConfig {
  similarityWeights?: SimilarityWeights;
  trendingTimeWindow?: TimeWindow;
  minTrendingThreshold?: number;  // Min sales+inferences for trending
  cacheExpirationMs?: number;     // How long to cache results
  maxCacheSize?: number;          // Max cached results
  diversityFactor?: number;       // 0-1, higher = more diverse results
}

/**
 * Default recommendation engine config
 */
export const DEFAULT_RECOMMENDATION_CONFIG: Required<RecommendationEngineConfig> = {
  similarityWeights: DEFAULT_SIMILARITY_WEIGHTS,
  trendingTimeWindow: '7d',
  minTrendingThreshold: 5,
  cacheExpirationMs: 5 * 60 * 1000, // 5 minutes
  maxCacheSize: 100,
  diversityFactor: 0.3
};
