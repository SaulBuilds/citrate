/**
 * User Tracking - LocalStorage-based interaction tracking
 *
 * Tracks user interactions with models for personalized recommendations.
 * Privacy-conscious: all data stored locally, never sent to server.
 */

import { UserInteraction, UserProfile } from './types';
import { ModelCategory } from '../search/types';

const STORAGE_KEY = 'citrate_user_interactions';
const MAX_INTERACTIONS = 100; // LRU cache limit

/**
 * Track a model view
 */
export function trackModelView(
  modelId: string,
  userAddress: string = 'anonymous',
  metadata?: UserInteraction['metadata']
): void {
  const interaction: UserInteraction = {
    userAddress,
    modelId,
    type: 'view',
    timestamp: Date.now(),
    metadata
  };

  storeInteraction(interaction);
}

/**
 * Track a model purchase
 */
export function trackModelPurchase(
  modelId: string,
  userAddress: string
): void {
  const interaction: UserInteraction = {
    userAddress,
    modelId,
    type: 'purchase',
    timestamp: Date.now()
  };

  storeInteraction(interaction);
}

/**
 * Track a model inference
 */
export function trackModelInference(
  modelId: string,
  userAddress: string
): void {
  const interaction: UserInteraction = {
    userAddress,
    modelId,
    type: 'inference',
    timestamp: Date.now()
  };

  storeInteraction(interaction);
}

/**
 * Store interaction in localStorage with LRU eviction
 */
function storeInteraction(interaction: UserInteraction): void {
  try {
    const history = getUserHistory();

    // Add new interaction
    history.push(interaction);

    // Apply LRU eviction if needed
    if (history.length > MAX_INTERACTIONS) {
      // Remove oldest interactions
      history.splice(0, history.length - MAX_INTERACTIONS);
    }

    localStorage.setItem(STORAGE_KEY, JSON.stringify(history));
  } catch (error) {
    console.error('Failed to store user interaction:', error);
    // LocalStorage might be full or disabled - fail silently
  }
}

/**
 * Get full user interaction history
 */
export function getUserHistory(): UserInteraction[] {
  try {
    const data = localStorage.getItem(STORAGE_KEY);
    if (!data) return [];

    const history = JSON.parse(data) as UserInteraction[];

    // Validate and filter
    return history.filter(interaction =>
      interaction.modelId &&
      interaction.type &&
      interaction.timestamp
    );
  } catch (error) {
    console.error('Failed to load user history:', error);
    return [];
  }
}

/**
 * Get recently viewed model IDs
 */
export function getRecentlyViewed(limit: number = 10): string[] {
  const history = getUserHistory();

  // Filter views only, reverse to get most recent first
  const views = history
    .filter(i => i.type === 'view')
    .reverse();

  // Remove duplicates (keep most recent)
  const seen = new Set<string>();
  const unique: string[] = [];

  for (const view of views) {
    if (!seen.has(view.modelId)) {
      seen.add(view.modelId);
      unique.push(view.modelId);

      if (unique.length >= limit) break;
    }
  }

  return unique;
}

/**
 * Get user's purchase history
 */
export function getPurchaseHistory(userAddress?: string): string[] {
  const history = getUserHistory();

  const purchases = history.filter(i =>
    i.type === 'purchase' &&
    (!userAddress || i.userAddress === userAddress)
  );

  // Return unique model IDs
  return [...new Set(purchases.map(p => p.modelId))];
}

/**
 * Get user's inference history
 */
export function getInferenceHistory(userAddress?: string): string[] {
  const history = getUserHistory();

  const inferences = history.filter(i =>
    i.type === 'inference' &&
    (!userAddress || i.userAddress === userAddress)
  );

  // Return model IDs (can include duplicates for frequency analysis)
  return inferences.map(i => i.modelId);
}

/**
 * Clear all user history
 */
export function clearHistory(): void {
  try {
    localStorage.removeItem(STORAGE_KEY);
  } catch (error) {
    console.error('Failed to clear user history:', error);
  }
}

/**
 * Get interactions within a time window
 */
export function getRecentInteractions(
  windowMs: number,
  type?: UserInteraction['type']
): UserInteraction[] {
  const history = getUserHistory();
  const cutoff = Date.now() - windowMs;

  return history.filter(i =>
    i.timestamp >= cutoff &&
    (!type || i.type === type)
  );
}

/**
 * Build user profile from interaction history
 */
export function buildUserProfile(
  userAddress: string,
  modelMetadata?: Map<string, { category: ModelCategory; tags: string[]; framework: string; basePrice: number }>
): UserProfile {
  const history = getUserHistory().filter(i => i.userAddress === userAddress);

  const favoriteCategories = new Map<string, number>();
  const favoriteTags = new Map<string, number>();
  const favoriteFrameworks = new Map<string, number>();
  const prices: number[] = [];

  // Analyze interactions
  for (const interaction of history) {
    const metadata = modelMetadata?.get(interaction.modelId);
    if (!metadata) continue;

    // Count categories
    favoriteCategories.set(
      metadata.category,
      (favoriteCategories.get(metadata.category) || 0) + 1
    );

    // Count tags
    for (const tag of metadata.tags) {
      favoriteTags.set(tag, (favoriteTags.get(tag) || 0) + 1);
    }

    // Count frameworks
    favoriteFrameworks.set(
      metadata.framework,
      (favoriteFrameworks.get(metadata.framework) || 0) + 1
    );

    // Track prices
    if (interaction.type === 'purchase') {
      prices.push(metadata.basePrice);
    }
  }

  // Calculate price range
  const avgPriceRange = prices.length > 0
    ? {
        min: Math.min(...prices),
        max: Math.max(...prices)
      }
    : { min: 0, max: 0 };

  const purchases = history.filter(i => i.type === 'purchase');
  const inferences = history.filter(i => i.type === 'inference');

  const timestamps = history.map(i => i.timestamp);
  const firstSeen = timestamps.length > 0 ? Math.min(...timestamps) : Date.now();
  const lastSeen = timestamps.length > 0 ? Math.max(...timestamps) : Date.now();

  return {
    userAddress,
    favoriteCategories,
    favoriteTags,
    favoriteFrameworks,
    avgPriceRange,
    recentActivity: history.slice(-20), // Last 20 interactions
    totalPurchases: purchases.length,
    totalInferences: inferences.length,
    firstSeen,
    lastSeen
  };
}

/**
 * Get interaction count by model
 */
export function getModelInteractionCounts(): Map<string, { views: number; purchases: number; inferences: number }> {
  const history = getUserHistory();
  const counts = new Map<string, { views: number; purchases: number; inferences: number }>();

  for (const interaction of history) {
    const current = counts.get(interaction.modelId) || { views: 0, purchases: 0, inferences: 0 };

    if (interaction.type === 'view') current.views++;
    else if (interaction.type === 'purchase') current.purchases++;
    else if (interaction.type === 'inference') current.inferences++;

    counts.set(interaction.modelId, current);
  }

  return counts;
}

/**
 * Check if user has interacted with a model
 */
export function hasInteractedWithModel(
  modelId: string,
  userAddress?: string,
  type?: UserInteraction['type']
): boolean {
  const history = getUserHistory();

  return history.some(i =>
    i.modelId === modelId &&
    (!userAddress || i.userAddress === userAddress) &&
    (!type || i.type === type)
  );
}

/**
 * Get co-viewed models (models viewed in same session)
 */
export function getCoViewedModels(modelId: string, sessionWindowMs: number = 30 * 60 * 1000): Map<string, number> {
  const history = getUserHistory().filter(i => i.type === 'view');
  const coViewed = new Map<string, number>();

  // Find views of the target model
  const targetViews = history.filter(i => i.modelId === modelId);

  for (const targetView of targetViews) {
    // Find other views within session window
    const sessionViews = history.filter(i =>
      i.modelId !== modelId &&
      Math.abs(i.timestamp - targetView.timestamp) <= sessionWindowMs
    );

    for (const view of sessionViews) {
      coViewed.set(view.modelId, (coViewed.get(view.modelId) || 0) + 1);
    }
  }

  return coViewed;
}

/**
 * Export user data (for GDPR compliance)
 */
export function exportUserData(): string {
  const history = getUserHistory();
  return JSON.stringify({
    version: '1.0',
    exportedAt: new Date().toISOString(),
    totalInteractions: history.length,
    interactions: history
  }, null, 2);
}

/**
 * Import user data
 */
export function importUserData(jsonData: string): boolean {
  try {
    const data = JSON.parse(jsonData);
    if (data.interactions && Array.isArray(data.interactions)) {
      localStorage.setItem(STORAGE_KEY, JSON.stringify(data.interactions));
      return true;
    }
    return false;
  } catch (error) {
    console.error('Failed to import user data:', error);
    return false;
  }
}
