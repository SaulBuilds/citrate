/**
 * useRecommendations Hook
 *
 * React hook for fetching and managing model recommendations.
 * Integrates with the recommendation engine and search index.
 */

import { useState, useEffect, useCallback } from 'react';
import { SearchDocument } from '../utils/search/types';
import { RecommendationEngine, getRecentlyViewed } from '../utils/recommendations';
import { RecommendationContext } from '../utils/recommendations/types';

interface UseRecommendationsOptions {
  modelId?: string;
  userAddress?: string;
  models: SearchDocument[];  // All available models
  enabled?: boolean;         // Enable/disable fetching
}

interface UseRecommendationsReturn {
  similarModels: SearchDocument[];
  trendingModels: SearchDocument[];
  recentlyViewed: SearchDocument[];
  collaborative: SearchDocument[];
  personalized: SearchDocument[];
  isLoading: boolean;
  error: Error | null;
  refreshRecommendations: () => void;
}

/**
 * Hook for fetching recommendations
 */
export function useRecommendations(
  options: UseRecommendationsOptions
): UseRecommendationsReturn {
  const { modelId, userAddress, models, enabled = true } = options;

  const [similarModels, setSimilarModels] = useState<SearchDocument[]>([]);
  const [trendingModels, setTrendingModels] = useState<SearchDocument[]>([]);
  const [recentlyViewed, setRecentlyViewed] = useState<SearchDocument[]>([]);
  const [collaborative, setCollaborative] = useState<SearchDocument[]>([]);
  const [personalized, setPersonalized] = useState<SearchDocument[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<Error | null>(null);

  const fetchRecommendations = useCallback(async () => {
    if (!enabled || models.length === 0) return;

    setIsLoading(true);
    setError(null);

    try {
      const engine = new RecommendationEngine(models);

      // Fetch similar models
      if (modelId) {
        const similar = engine.getSimilarModels(modelId, 4);
        setSimilarModels(similar);

        // Fetch collaborative recommendations
        const collab = engine.getUsersWhoBoughtAlsoBought(modelId, 3);
        setCollaborative(collab);
      } else {
        setSimilarModels([]);
        setCollaborative([]);
      }

      // Fetch trending models
      const trending = engine.getTrendingModels('7d', 10);
      setTrendingModels(trending);

      // Fetch personalized recommendations
      if (userAddress) {
        const personal = engine.getPersonalizedRecommendations(userAddress, 5);
        setPersonalized(personal);
      } else {
        setPersonalized([]);
      }

      // Fetch recently viewed models
      const recentIds = getRecentlyViewed(5);
      const recent = recentIds
        .map(id => models.find(m => m.modelId === id))
        .filter((m): m is SearchDocument => m !== undefined);
      setRecentlyViewed(recent);

    } catch (err) {
      console.error('Failed to fetch recommendations:', err);
      setError(err instanceof Error ? err : new Error('Failed to fetch recommendations'));
    } finally {
      setIsLoading(false);
    }
  }, [modelId, userAddress, models, enabled]);

  // Fetch on mount and when dependencies change
  useEffect(() => {
    fetchRecommendations();
  }, [fetchRecommendations]);

  const refreshRecommendations = useCallback(() => {
    fetchRecommendations();
  }, [fetchRecommendations]);

  return {
    similarModels,
    trendingModels,
    recentlyViewed,
    collaborative,
    personalized,
    isLoading,
    error,
    refreshRecommendations
  };
}

/**
 * Hook for fetching specific recommendation type
 */
export function useSpecificRecommendations(
  context: RecommendationContext,
  models: SearchDocument[],
  enabled: boolean = true
): {
  recommendations: SearchDocument[];
  isLoading: boolean;
  error: Error | null;
  executionTimeMs: number;
} {
  const [recommendations, setRecommendations] = useState<SearchDocument[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<Error | null>(null);
  const [executionTimeMs, setExecutionTimeMs] = useState(0);

  useEffect(() => {
    if (!enabled || models.length === 0) return;

    setIsLoading(true);
    setError(null);

    try {
      const engine = new RecommendationEngine(models);
      const result = engine.getRecommendations(context);

      const recommendedModels = result.recommendations
        .map(rec => models.find(m => m.modelId === rec.modelId))
        .filter((m): m is SearchDocument => m !== undefined);

      setRecommendations(recommendedModels);
      setExecutionTimeMs(result.executionTimeMs);
    } catch (err) {
      console.error('Failed to fetch specific recommendations:', err);
      setError(err instanceof Error ? err : new Error('Failed to fetch recommendations'));
    } finally {
      setIsLoading(false);
    }
  }, [context, models, enabled]);

  return {
    recommendations,
    isLoading,
    error,
    executionTimeMs
  };
}

/**
 * Hook for tracking recommendation clicks
 */
export function useRecommendationTracking() {
  const trackClick = useCallback((
    modelId: string,
    algorithm: string,
    position: number
  ) => {
    // Track recommendation click for analytics
    console.log('Recommendation clicked:', {
      modelId,
      algorithm,
      position,
      timestamp: Date.now()
    });

    // Could send to analytics service here
  }, []);

  return { trackClick };
}
