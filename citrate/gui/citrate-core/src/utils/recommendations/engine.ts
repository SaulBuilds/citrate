/**
 * Recommendation Engine - Multi-algorithm recommendation system
 *
 * Implements content-based filtering, collaborative filtering, trending,
 * and personalized recommendations for the model marketplace.
 */

import { SearchDocument, ModelCategory, ModelSize } from '../search/types';
import {
  RecommendationScore,
  RecommendationAlgorithm,
  SimilarityWeights,
  TrendingScore,
  CoPurchaseData,
  UserProfile,
  RecommendationContext,
  RecommendationResult,
  JaccardSimilarity,
  TimeWindow,
  RecommendationEngineConfig,
  DEFAULT_RECOMMENDATION_CONFIG,
  TIME_WINDOW_MS,
  RecommendationCacheEntry
} from './types';
import {
  getUserHistory,
  getPurchaseHistory,
  buildUserProfile,
  getCoViewedModels
} from './userTracking';

/**
 * Recommendation Engine Class
 */
export class RecommendationEngine {
  private models: Map<string, SearchDocument>;
  private config: Required<RecommendationEngineConfig>;
  private cache: Map<string, RecommendationCacheEntry>;

  constructor(
    models: SearchDocument[] | Map<string, SearchDocument>,
    config?: RecommendationEngineConfig
  ) {
    // Convert array to map if needed
    this.models = Array.isArray(models)
      ? new Map(models.map(m => [m.modelId, m]))
      : models;

    this.config = { ...DEFAULT_RECOMMENDATION_CONFIG, ...config };
    this.cache = new Map();
  }

  /**
   * Get recommendations based on context
   */
  public getRecommendations(context: RecommendationContext): RecommendationResult {
    const startTime = Date.now();
    const cacheKey = this.getCacheKey(context);

    // Check cache
    const cached = this.getFromCache(cacheKey);
    if (cached) {
      return cached;
    }

    const scores: RecommendationScore[] = [];
    const algorithmsUsed: Set<RecommendationAlgorithm> = new Set();

    // Apply requested algorithms or defaults
    const algorithms = context.algorithms || ['content-based', 'collaborative', 'trending', 'personalized'];

    for (const algorithm of algorithms) {
      let algorithmScores: RecommendationScore[] = [];

      switch (algorithm) {
        case 'content-based':
          if (context.modelId) {
            algorithmScores = this.getSimilarModels(context.modelId, context.limit || 10)
              .map(doc => ({
                modelId: doc.modelId,
                score: 85,
                reason: 'Similar content and features',
                algorithm: 'content-based' as RecommendationAlgorithm
              }));
          }
          break;

        case 'collaborative':
          if (context.modelId) {
            algorithmScores = this.getUsersWhoBoughtAlsoBought(context.modelId, context.limit || 10)
              .map(doc => ({
                modelId: doc.modelId,
                score: 80,
                reason: 'Users who bought this also bought',
                algorithm: 'collaborative' as RecommendationAlgorithm
              }));
          }
          break;

        case 'trending':
          algorithmScores = this.getTrendingModels(this.config.trendingTimeWindow, context.limit || 10)
            .map(doc => ({
              modelId: doc.modelId,
              score: 75,
              reason: 'Trending in marketplace',
              algorithm: 'trending' as RecommendationAlgorithm
            }));
          break;

        case 'personalized':
          if (context.userAddress) {
            algorithmScores = this.getPersonalizedRecommendations(context.userAddress, context.limit || 10)
              .map(doc => ({
                modelId: doc.modelId,
                score: 90,
                reason: 'Personalized for you',
                algorithm: 'personalized' as RecommendationAlgorithm
              }));
          }
          break;
      }

      if (algorithmScores.length > 0) {
        scores.push(...algorithmScores);
        algorithmsUsed.add(algorithm);
      }
    }

    // Remove excluded models
    const filtered = scores.filter(s =>
      !context.excludeModelIds?.includes(s.modelId) &&
      (context.minScore === undefined || s.score >= context.minScore)
    );

    // Deduplicate and merge scores
    const merged = this.mergeAndDiversify(filtered, context.limit || 10);

    const result: RecommendationResult = {
      recommendations: merged,
      context,
      totalCandidates: scores.length,
      executionTimeMs: Date.now() - startTime,
      algorithmsUsed: Array.from(algorithmsUsed)
    };

    // Cache result
    this.addToCache(cacheKey, result);

    return result;
  }

  /**
   * Content-Based Filtering: Get similar models
   */
  public getSimilarModels(modelId: string, limit: number = 10): SearchDocument[] {
    const targetModel = this.models.get(modelId);
    if (!targetModel) return [];

    const scores: Array<{ model: SearchDocument; score: number }> = [];

    for (const [id, model] of this.models) {
      // Skip the target model itself and inactive models
      if (id === modelId || !model.isActive) continue;

      const similarity = this.calculateSimilarity(targetModel, model, this.config.similarityWeights);
      scores.push({ model, score: similarity });
    }

    // Sort by similarity and return top N
    return scores
      .sort((a, b) => b.score - a.score)
      .slice(0, limit)
      .map(s => s.model);
  }

  /**
   * Collaborative Filtering: Users who bought this also bought
   */
  public getUsersWhoBoughtAlsoBought(modelId: string, limit: number = 10): SearchDocument[] {
    const history = getUserHistory();

    // Find users who purchased this model
    const purchasers = new Set(
      history
        .filter(i => i.modelId === modelId && i.type === 'purchase')
        .map(i => i.userAddress)
    );

    if (purchasers.size === 0) {
      // Fall back to co-viewed models
      const coViewed = getCoViewedModels(modelId);
      const results: SearchDocument[] = [];

      for (const [coModelId, count] of coViewed) {
        const model = this.models.get(coModelId);
        if (model && model.isActive) {
          results.push(model);
        }
      }

      return results
        .sort((a, b) => (coViewed.get(b.modelId) || 0) - (coViewed.get(a.modelId) || 0))
        .slice(0, limit);
    }

    // Find other models these users purchased
    const coPurchases = new Map<string, number>();

    for (const interaction of history) {
      if (
        interaction.type === 'purchase' &&
        interaction.modelId !== modelId &&
        purchasers.has(interaction.userAddress)
      ) {
        coPurchases.set(
          interaction.modelId,
          (coPurchases.get(interaction.modelId) || 0) + 1
        );
      }
    }

    // Sort by co-purchase frequency
    const sorted = Array.from(coPurchases.entries())
      .sort((a, b) => b[1] - a[1])
      .slice(0, limit);

    const results: SearchDocument[] = [];
    for (const [coModelId] of sorted) {
      const model = this.models.get(coModelId);
      if (model && model.isActive) {
        results.push(model);
      }
    }

    return results;
  }

  /**
   * Trending Algorithm: Hot models based on recent activity
   */
  public getTrendingModels(timeWindow: TimeWindow, limit: number = 10): SearchDocument[] {
    const windowMs = TIME_WINDOW_MS[timeWindow];
    const cutoff = Date.now() - windowMs;
    const history = getUserHistory().filter(i => i.timestamp >= cutoff);

    const trendingScores = new Map<string, TrendingScore>();

    // Calculate trending scores for all models
    for (const model of this.models.values()) {
      if (!model.isActive) continue;

      const modelHistory = history.filter(i => i.modelId === model.modelId);
      const sales = modelHistory.filter(i => i.type === 'purchase').length;
      const inferences = modelHistory.filter(i => i.type === 'inference').length;

      const daysSinceListing = (Date.now() - model.listedAt) / (24 * 60 * 60 * 1000);

      // Trending score formula: (sales * 2 + inferences) / max(daysSinceListing, 1)
      // Apply time decay for recent activity
      const recentWeight = 1 + (modelHistory.filter(i => i.timestamp >= Date.now() - 24 * 60 * 60 * 1000).length * 0.5);
      const trendScore = ((sales * 2 + inferences) / Math.max(daysSinceListing, 1)) * recentWeight;

      // Calculate velocity (activity rate)
      const velocity = (sales + inferences) / Math.max(daysSinceListing, 1);

      // Calculate momentum (recent vs overall)
      const recentActivity = modelHistory.filter(i => i.timestamp >= Date.now() - 7 * 24 * 60 * 60 * 1000).length;
      const momentum = recentActivity / Math.max(modelHistory.length, 1);

      trendingScores.set(model.modelId, {
        modelId: model.modelId,
        sales,
        inferences,
        daysSinceListing,
        trendScore,
        velocity,
        momentum
      });
    }

    // Filter by minimum threshold and sort
    const results = Array.from(trendingScores.values())
      .filter(ts => (ts.sales + ts.inferences) >= this.config.minTrendingThreshold)
      .sort((a, b) => b.trendScore - a.trendScore)
      .slice(0, limit);

    return results
      .map(ts => this.models.get(ts.modelId))
      .filter((m): m is SearchDocument => m !== undefined);
  }

  /**
   * Category-Based Recommendations: Popular models in same category
   */
  public getCategoryRecommendations(category: ModelCategory, limit: number = 10): SearchDocument[] {
    const categoryModels = Array.from(this.models.values())
      .filter(m => m.category === category && m.isActive);

    // Sort by quality score and sales
    return categoryModels
      .sort((a, b) => {
        const scoreA = a.qualityScore * 0.6 + (a.totalSales / 100) * 0.4;
        const scoreB = b.qualityScore * 0.6 + (b.totalSales / 100) * 0.4;
        return scoreB - scoreA;
      })
      .slice(0, limit);
  }

  /**
   * Personalized Recommendations: Based on user history
   */
  public getPersonalizedRecommendations(userAddress: string, limit: number = 10): SearchDocument[] {
    // Build user profile
    const modelMetadata = new Map(
      Array.from(this.models.values()).map(m => [
        m.modelId,
        {
          category: m.category,
          tags: m.tags,
          framework: m.framework,
          basePrice: m.basePrice
        }
      ])
    );

    const profile = buildUserProfile(userAddress, modelMetadata);
    const purchasedModelIds = new Set(getPurchaseHistory(userAddress));

    const scores: Array<{ model: SearchDocument; score: number }> = [];

    for (const model of this.models.values()) {
      if (!model.isActive || purchasedModelIds.has(model.modelId)) continue;

      let score = 0;

      // Category preference
      const categoryScore = profile.favoriteCategories.get(model.category) || 0;
      score += categoryScore * 40;

      // Tag overlap
      const tagScore = model.tags.reduce((sum, tag) => {
        return sum + (profile.favoriteTags.get(tag) || 0);
      }, 0);
      score += Math.min(tagScore * 20, 30); // Cap at 30

      // Framework preference
      const frameworkScore = profile.favoriteFrameworks.get(model.framework) || 0;
      score += frameworkScore * 15;

      // Price range compatibility
      const priceInRange = model.basePrice >= profile.avgPriceRange.min * 0.5 &&
                           model.basePrice <= profile.avgPriceRange.max * 2;
      if (priceInRange) score += 15;

      scores.push({ model, score });
    }

    // Apply diversity factor
    const diverse = this.applyDiversityFactor(scores, this.config.diversityFactor);

    return diverse
      .sort((a, b) => b.score - a.score)
      .slice(0, limit)
      .map(s => s.model);
  }

  /**
   * Calculate similarity between two models
   */
  private calculateSimilarity(
    modelA: SearchDocument,
    modelB: SearchDocument,
    weights: SimilarityWeights
  ): number {
    let similarity = 0;

    // Category match (40%)
    if (modelA.category === modelB.category) {
      similarity += weights.category * 100;
    }

    // Tag overlap using Jaccard similarity (30%)
    const tagSimilarity = this.jaccardSimilarity(modelA.tags, modelB.tags);
    similarity += tagSimilarity.similarity * weights.tags * 100;

    // Framework match (15%)
    if (modelA.framework === modelB.framework) {
      similarity += weights.framework * 100;
    }

    // Model size similarity (15%)
    const sizeSimilarity = this.modelSizeSimilarity(modelA.modelSize, modelB.modelSize);
    similarity += sizeSimilarity * weights.modelSize * 100;

    return similarity;
  }

  /**
   * Jaccard similarity for sets
   */
  private jaccardSimilarity(setA: string[], setB: string[]): JaccardSimilarity {
    const a = new Set(setA);
    const b = new Set(setB);

    const intersection = new Set([...a].filter(x => b.has(x)));
    const union = new Set([...a, ...b]);

    const similarity = union.size > 0 ? intersection.size / union.size : 0;

    return {
      similarity,
      intersection: intersection.size,
      union: union.size
    };
  }

  /**
   * Model size similarity (0-1)
   */
  private modelSizeSimilarity(sizeA?: ModelSize, sizeB?: ModelSize): number {
    if (!sizeA || !sizeB) return 0;
    if (sizeA === sizeB) return 1;

    const sizes = [ModelSize.TINY, ModelSize.SMALL, ModelSize.MEDIUM, ModelSize.LARGE, ModelSize.XLARGE];
    const indexA = sizes.indexOf(sizeA);
    const indexB = sizes.indexOf(sizeB);

    const distance = Math.abs(indexA - indexB);
    return Math.max(0, 1 - distance * 0.25); // Adjacent sizes = 0.75 similarity
  }

  /**
   * Merge and diversify recommendations
   */
  private mergeAndDiversify(
    scores: RecommendationScore[],
    limit: number
  ): RecommendationScore[] {
    // Group by modelId and take highest score
    const merged = new Map<string, RecommendationScore>();

    for (const score of scores) {
      const existing = merged.get(score.modelId);
      if (!existing || score.score > existing.score) {
        merged.set(score.modelId, score);
      }
    }

    // Convert to array and sort
    const sorted = Array.from(merged.values())
      .sort((a, b) => b.score - a.score);

    // Apply diversity (ensure varied categories)
    const diverse: RecommendationScore[] = [];
    const categoryCounts = new Map<ModelCategory, number>();

    for (const score of sorted) {
      const model = this.models.get(score.modelId);
      if (!model) continue;

      const categoryCount = categoryCounts.get(model.category) || 0;

      // Allow some repetition but prefer diversity
      if (categoryCount < 3 || diverse.length < limit * 0.7) {
        diverse.push(score);
        categoryCounts.set(model.category, categoryCount + 1);

        if (diverse.length >= limit) break;
      }
    }

    // Fill remaining slots if needed
    if (diverse.length < limit) {
      for (const score of sorted) {
        if (!diverse.find(d => d.modelId === score.modelId)) {
          diverse.push(score);
          if (diverse.length >= limit) break;
        }
      }
    }

    return diverse;
  }

  /**
   * Apply diversity factor to reduce similar results
   */
  private applyDiversityFactor(
    scores: Array<{ model: SearchDocument; score: number }>,
    diversityFactor: number
  ): Array<{ model: SearchDocument; score: number }> {
    if (diversityFactor === 0) return scores;

    const adjusted = [...scores];
    const categoryCounts = new Map<ModelCategory, number>();

    for (const item of adjusted) {
      const count = categoryCounts.get(item.model.category) || 0;
      categoryCounts.set(item.model.category, count + 1);

      // Penalize repeated categories
      item.score *= Math.pow(0.9, count * diversityFactor);
    }

    return adjusted;
  }

  /**
   * Generate cache key from context
   */
  private getCacheKey(context: RecommendationContext): string {
    return JSON.stringify({
      modelId: context.modelId,
      userAddress: context.userAddress,
      algorithms: context.algorithms?.sort(),
      limit: context.limit
    });
  }

  /**
   * Get from cache if valid
   */
  private getFromCache(key: string): RecommendationResult | null {
    const entry = this.cache.get(key);
    if (!entry) return null;

    if (Date.now() > entry.expiresAt) {
      this.cache.delete(key);
      return null;
    }

    return entry.result;
  }

  /**
   * Add to cache with LRU eviction
   */
  private addToCache(key: string, result: RecommendationResult): void {
    const entry: RecommendationCacheEntry = {
      key,
      result,
      timestamp: Date.now(),
      expiresAt: Date.now() + this.config.cacheExpirationMs
    };

    this.cache.set(key, entry);

    // LRU eviction
    if (this.cache.size > this.config.maxCacheSize) {
      const oldest = Array.from(this.cache.entries())
        .sort((a, b) => a[1].timestamp - b[1].timestamp)[0];

      if (oldest) {
        this.cache.delete(oldest[0]);
      }
    }
  }

  /**
   * Clear cache
   */
  public clearCache(): void {
    this.cache.clear();
  }

  /**
   * Update models in engine
   */
  public updateModels(models: SearchDocument[]): void {
    this.models = new Map(models.map(m => [m.modelId, m]));
    this.clearCache();
  }

  /**
   * Get engine statistics
   */
  public getStatistics(): {
    totalModels: number;
    activeModels: number;
    cacheSize: number;
    cacheHitRate: number;
  } {
    const activeModels = Array.from(this.models.values()).filter(m => m.isActive).length;

    return {
      totalModels: this.models.size,
      activeModels,
      cacheSize: this.cache.size,
      cacheHitRate: 0 // Would need to track hits/misses
    };
  }
}
