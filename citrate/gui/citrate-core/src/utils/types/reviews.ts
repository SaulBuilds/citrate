/**
 * Review Types for Citrate Model Marketplace
 *
 * Defines comprehensive types for the review and rating system.
 * Supports verified purchases, vote tracking, moderation, and reporting.
 */

export interface Review {
  reviewId: string;
  modelId: string;
  reviewer: string;
  reviewerName?: string;
  rating: number; // 1-5
  text: string;
  timestamp: number;
  verifiedPurchase: boolean;
  helpfulVotes: number;
  unhelpfulVotes: number;
  reported: boolean;
  reportReason?: string;
}

export type ReviewSortOption =
  | 'mostHelpful'
  | 'recent'
  | 'highestRating'
  | 'lowestRating';

export interface ReviewVote {
  reviewId: string;
  voter: string;
  isHelpful: boolean;
}

export interface ReviewReport {
  reviewId: string;
  reporter: string;
  reason: string;
  timestamp: number;
  status: 'pending' | 'approved' | 'removed';
}

export interface ReviewSubmission {
  modelId: string;
  rating: number;
  text: string;
  reviewer: string;
  verifiedPurchase: boolean;
}

export interface ReviewStats {
  totalReviews: number;
  averageRating: number;
  ratingDistribution: {
    5: number;
    4: number;
    3: number;
    2: number;
    1: number;
  };
  verifiedPurchaseCount: number;
}

/**
 * Enhanced Model Metadata Types
 */

export enum ModelCategory {
  LANGUAGE_MODELS = 0,
  IMAGE_GENERATION = 1,
  COMPUTER_VISION = 2,
  AUDIO_PROCESSING = 3,
  CODE_GENERATION = 4,
  EMBEDDINGS = 5,
  CLASSIFICATION = 6,
  TRANSLATION = 7,
  SUMMARIZATION = 8,
  QUESTION_ANSWERING = 9,
  OTHER = 10,
}

export enum ModelSize {
  TINY = 'tiny',
  SMALL = 'small',
  MEDIUM = 'medium',
  LARGE = 'large',
  XLARGE = 'xlarge',
}

export interface ModelExample {
  title: string;
  input: string;
  output: string;
  description?: string;
}

export interface PerformanceBenchmark {
  metric: string;
  value: number;
  unit: string;
  dataset?: string;
}

export interface SupportedFormats {
  input: string[];
  output: string[];
}

export interface EnrichedModelMetadata {
  // Core fields
  name: string;
  description: string;
  category: ModelCategory;
  tags: string[];

  // Technical details
  framework?: string;
  modelSize?: ModelSize;
  sizeBytes?: number;
  architecture?: string;
  version?: string;

  // Model capabilities
  examples?: ModelExample[];
  supportedFormats?: SupportedFormats;

  // Training information
  trainingDataset?: string;
  pretrainedOn?: string;
  finetuneMethod?: string;

  // Legal and licensing
  license?: string;

  // Performance metrics
  benchmarks?: PerformanceBenchmark[];
  performanceMetrics?: {
    avgLatency?: number;
    throughput?: number;
    accuracy?: number;
  };

  // Metadata
  lastUpdated?: number;
  creator?: {
    address: string;
    name?: string;
    ens?: string;
  };
}

export interface ValidationResult {
  valid: boolean;
  errors: { field: string; message: string }[];
  warnings: { field: string; message: string }[];
}

/**
 * License options for models
 */
export const LICENSE_OPTIONS = [
  { value: 'MIT', label: 'MIT License' },
  { value: 'Apache-2.0', label: 'Apache License 2.0' },
  { value: 'GPL-3.0', label: 'GNU GPL v3' },
  { value: 'BSD-3-Clause', label: 'BSD 3-Clause License' },
  { value: 'CC-BY-4.0', label: 'Creative Commons Attribution 4.0' },
  { value: 'CC-BY-SA-4.0', label: 'Creative Commons Attribution-ShareAlike 4.0' },
  { value: 'Commercial', label: 'Commercial License' },
  { value: 'Custom', label: 'Custom License' },
  { value: 'Other', label: 'Other' },
] as const;

/**
 * Category display information
 */
export const CATEGORY_LABELS: Record<ModelCategory, string> = {
  [ModelCategory.LANGUAGE_MODELS]: 'Language Models',
  [ModelCategory.IMAGE_GENERATION]: 'Image Generation',
  [ModelCategory.COMPUTER_VISION]: 'Computer Vision',
  [ModelCategory.AUDIO_PROCESSING]: 'Audio Processing',
  [ModelCategory.CODE_GENERATION]: 'Code Generation',
  [ModelCategory.EMBEDDINGS]: 'Embeddings',
  [ModelCategory.CLASSIFICATION]: 'Classification',
  [ModelCategory.TRANSLATION]: 'Translation',
  [ModelCategory.SUMMARIZATION]: 'Summarization',
  [ModelCategory.QUESTION_ANSWERING]: 'Question Answering',
  [ModelCategory.OTHER]: 'Other',
};

/**
 * Model size display information
 */
export const MODEL_SIZE_INFO: Record<ModelSize, { label: string; range: string }> = {
  [ModelSize.TINY]: { label: 'Tiny', range: '< 1GB' },
  [ModelSize.SMALL]: { label: 'Small', range: '1-5GB' },
  [ModelSize.MEDIUM]: { label: 'Medium', range: '5-20GB' },
  [ModelSize.LARGE]: { label: 'Large', range: '20-100GB' },
  [ModelSize.XLARGE]: { label: 'X-Large', range: '> 100GB' },
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
 * Report reason options
 */
export const REPORT_REASONS = [
  'Spam or misleading content',
  'Offensive language',
  'False information',
  'Duplicate review',
  'Not relevant to the model',
  'Other',
] as const;

export type ReportReason = typeof REPORT_REASONS[number];
