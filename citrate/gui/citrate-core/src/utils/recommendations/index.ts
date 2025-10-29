/**
 * Recommendation System - Exports
 *
 * Unified exports for the recommendation engine and utilities.
 */

export * from './types';
export * from './engine';
export * from './userTracking';

// Re-export commonly used functions
export { RecommendationEngine } from './engine';
export {
  trackModelView,
  trackModelPurchase,
  trackModelInference,
  getUserHistory,
  getRecentlyViewed,
  getPurchaseHistory,
  clearHistory,
  buildUserProfile
} from './userTracking';
