/**
 * Metadata Utilities Index
 *
 * Central export for all metadata-related utilities.
 */

// Validation
export {
  validateModelMetadata,
  validateReview,
  sanitizeUserInput,
  sanitizeMarkdown,
  isValidAddress,
  isValidIPFSCID,
  isValidURL,
  getValidationMessage,
  hasFieldError,
} from './validation';

// IPFS Upload
export {
  IPFSUploader,
  uploadMetadataToIPFS,
  updateModelMetadataOnChain,
  validateAndUpload,
  ipfsUploader,
} from './ipfsUploader';

export type {
  UploadProgress,
  UploadResult,
  ProgressCallback,
} from './ipfsUploader';

// Re-export types
export type {
  EnrichedModelMetadata,
  ValidationResult,
  ModelExample,
  PerformanceBenchmark,
  SupportedFormats,
} from '../types/reviews';

export {
  ModelCategory,
  ModelSize,
  categorizeModelSize,
  CATEGORY_LABELS,
  MODEL_SIZE_INFO,
  LICENSE_OPTIONS,
} from '../types/reviews';
