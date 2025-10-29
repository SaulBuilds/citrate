/**
 * useReviews Hook
 *
 * React hook for managing model reviews and ratings.
 * Features:
 * - Fetch reviews for a model
 * - Submit new reviews
 * - Vote on reviews (helpful/unhelpful)
 * - Report reviews
 * - Sort and filter reviews
 */

import { useState, useEffect, useCallback } from 'react';
import {
  Review,
  ReviewSortOption,
  ReviewSubmission,
  ReviewStats,
} from '../utils/types/reviews';
import { getMarketplaceService, Review as ContractReview } from '../utils/marketplaceService';

// ============================================================================
// Types
// ============================================================================

export interface UseReviewsOptions {
  modelId: string;
  autoFetch?: boolean;
  sortBy?: ReviewSortOption;
  ratingFilter?: number; // Filter by specific rating (1-5)
}

export interface UseReviewsReturn {
  reviews: Review[];
  isLoading: boolean;
  error: string | null;
  stats: ReviewStats;
  sortBy: ReviewSortOption;
  ratingFilter: number | null;

  // Actions
  setSortBy: (sortBy: ReviewSortOption) => void;
  setRatingFilter: (rating: number | null) => void;
  submitReview: (review: Omit<ReviewSubmission, 'reviewer'>, password: string) => Promise<void>;
  voteOnReview: (reviewId: string, isHelpful: boolean) => Promise<void>;
  reportReview: (reviewId: string, reason: string) => Promise<void>;
  refetch: () => Promise<void>;
}

// ============================================================================
// Hook Implementation
// ============================================================================

export function useReviews(options: UseReviewsOptions): UseReviewsReturn {
  const { modelId, autoFetch = true, sortBy: initialSortBy = 'mostHelpful' } = options;

  const [reviews, setReviews] = useState<Review[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [sortBy, setSortBy] = useState<ReviewSortOption>(initialSortBy);
  const [ratingFilter, setRatingFilter] = useState<number | null>(null);

  /**
   * Fetch reviews from contract
   */
  const fetchReviews = useCallback(async () => {
    if (!modelId) return;

    setIsLoading(true);
    setError(null);

    try {
      const marketplace = getMarketplaceService();
      const contractReviews = await marketplace.getModelReviews(modelId);

      // Convert contract reviews to enhanced Review type
      const enhancedReviews: Review[] = contractReviews.map((r, index) => ({
        reviewId: `${r.modelId}-${r.reviewer}-${r.timestamp}`,
        modelId: r.modelId,
        reviewer: r.reviewer,
        reviewerName: undefined, // Could be fetched from ENS or user profile
        rating: r.rating,
        text: r.comment,
        timestamp: parseInt(r.timestamp),
        verifiedPurchase: r.verified,
        helpfulVotes: 0, // Not tracked on-chain in current contract
        unhelpfulVotes: 0, // Not tracked on-chain in current contract
        reported: false, // Not tracked on-chain in current contract
        reportReason: undefined,
      }));

      setReviews(enhancedReviews);
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to fetch reviews';
      setError(errorMessage);
      console.error('[useReviews] Failed to fetch reviews:', err);
    } finally {
      setIsLoading(false);
    }
  }, [modelId]);

  /**
   * Submit a new review
   */
  const submitReview = useCallback(
    async (review: Omit<ReviewSubmission, 'reviewer'>, password: string) => {
      try {
        const marketplace = getMarketplaceService();

        // Get current user address (would come from wallet context in real app)
        // For now, we'll need it passed in or use a placeholder
        const userAddress = '0x0000000000000000000000000000000000000000'; // TODO: Get from wallet

        const txHash = await marketplace.addReview(
          {
            modelId: review.modelId,
            rating: review.rating,
            comment: review.text,
          },
          {
            from: userAddress,
            password,
          }
        );

        console.log('[useReviews] Review submitted:', txHash);

        // Refresh reviews
        await fetchReviews();
      } catch (err) {
        const errorMessage = err instanceof Error ? err.message : 'Failed to submit review';
        throw new Error(errorMessage);
      }
    },
    [fetchReviews]
  );

  /**
   * Vote on review (helpful/unhelpful)
   * Note: Not implemented in current contract, would need contract upgrade
   */
  const voteOnReview = useCallback(
    async (reviewId: string, isHelpful: boolean) => {
      try {
        // TODO: Implement when contract supports review voting
        console.log('[useReviews] Vote on review:', reviewId, isHelpful);

        // For now, update local state optimistically
        setReviews((prev) =>
          prev.map((review) => {
            if (review.reviewId === reviewId) {
              return {
                ...review,
                helpfulVotes: isHelpful
                  ? review.helpfulVotes + 1
                  : review.helpfulVotes,
                unhelpfulVotes: !isHelpful
                  ? review.unhelpfulVotes + 1
                  : review.unhelpfulVotes,
              };
            }
            return review;
          })
        );
      } catch (err) {
        const errorMessage = err instanceof Error ? err.message : 'Failed to vote on review';
        throw new Error(errorMessage);
      }
    },
    []
  );

  /**
   * Report a review
   * Note: Not implemented in current contract, would need contract upgrade
   */
  const reportReview = useCallback(
    async (reviewId: string, reason: string) => {
      try {
        // TODO: Implement when contract supports review reporting
        console.log('[useReviews] Report review:', reviewId, reason);

        // For now, update local state
        setReviews((prev) =>
          prev.map((review) => {
            if (review.reviewId === reviewId) {
              return {
                ...review,
                reported: true,
                reportReason: reason,
              };
            }
            return review;
          })
        );
      } catch (err) {
        const errorMessage = err instanceof Error ? err.message : 'Failed to report review';
        throw new Error(errorMessage);
      }
    },
    []
  );

  /**
   * Sort reviews
   */
  const sortedReviews = useCallback(() => {
    let sorted = [...reviews];

    // Apply rating filter
    if (ratingFilter !== null) {
      sorted = sorted.filter((review) => review.rating === ratingFilter);
    }

    // Apply sorting
    switch (sortBy) {
      case 'mostHelpful':
        sorted.sort((a, b) => {
          const aScore = a.helpfulVotes - a.unhelpfulVotes;
          const bScore = b.helpfulVotes - b.unhelpfulVotes;
          return bScore - aScore;
        });
        break;

      case 'recent':
        sorted.sort((a, b) => b.timestamp - a.timestamp);
        break;

      case 'highestRating':
        sorted.sort((a, b) => b.rating - a.rating);
        break;

      case 'lowestRating':
        sorted.sort((a, b) => a.rating - b.rating);
        break;
    }

    return sorted;
  }, [reviews, sortBy, ratingFilter]);

  /**
   * Calculate review statistics
   */
  const calculateStats = useCallback((): ReviewStats => {
    if (reviews.length === 0) {
      return {
        totalReviews: 0,
        averageRating: 0,
        ratingDistribution: { 5: 0, 4: 0, 3: 0, 2: 0, 1: 0 },
        verifiedPurchaseCount: 0,
      };
    }

    const ratingDistribution = { 5: 0, 4: 0, 3: 0, 2: 0, 1: 0 };
    let totalRating = 0;
    let verifiedCount = 0;

    reviews.forEach((review) => {
      totalRating += review.rating;
      ratingDistribution[review.rating as keyof typeof ratingDistribution]++;
      if (review.verifiedPurchase) {
        verifiedCount++;
      }
    });

    return {
      totalReviews: reviews.length,
      averageRating: totalRating / reviews.length,
      ratingDistribution,
      verifiedPurchaseCount: verifiedCount,
    };
  }, [reviews]);

  // Auto-fetch on mount
  useEffect(() => {
    if (autoFetch) {
      fetchReviews();
    }
  }, [autoFetch, fetchReviews]);

  return {
    reviews: sortedReviews(),
    isLoading,
    error,
    stats: calculateStats(),
    sortBy,
    ratingFilter,
    setSortBy,
    setRatingFilter,
    submitReview,
    voteOnReview,
    reportReview,
    refetch: fetchReviews,
  };
}

export default useReviews;
