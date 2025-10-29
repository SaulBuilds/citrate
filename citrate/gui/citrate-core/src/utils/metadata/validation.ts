/**
 * Metadata Validation for Citrate Model Marketplace
 *
 * Provides comprehensive validation for model metadata and reviews.
 * Includes schema validation, sanitization, and security checks.
 */

import {
  EnrichedModelMetadata,
  ValidationResult,
  ReviewSubmission,
  ModelCategory,
  ModelSize,
  ModelExample,
  PerformanceBenchmark,
  CATEGORY_LABELS,
} from '../types/reviews';

// ============================================================================
// Constants
// ============================================================================

const MIN_NAME_LENGTH = 3;
const MAX_NAME_LENGTH = 100;
const MIN_DESCRIPTION_LENGTH = 50;
const MAX_DESCRIPTION_LENGTH = 2000;
const MIN_REVIEW_LENGTH = 50;
const MAX_REVIEW_LENGTH = 1000;
const MAX_TAGS = 20;
const MAX_TAG_LENGTH = 30;
const MAX_EXAMPLES = 10;
const MAX_BENCHMARKS = 20;

// ============================================================================
// Model Metadata Validation
// ============================================================================

/**
 * Validate complete model metadata
 */
export function validateModelMetadata(
  metadata: Partial<EnrichedModelMetadata>
): ValidationResult {
  const errors: { field: string; message: string }[] = [];
  const warnings: { field: string; message: string }[] = [];

  // Required fields
  if (!metadata.name) {
    errors.push({ field: 'name', message: 'Model name is required' });
  } else if (metadata.name.length < MIN_NAME_LENGTH) {
    errors.push({
      field: 'name',
      message: `Model name must be at least ${MIN_NAME_LENGTH} characters`,
    });
  } else if (metadata.name.length > MAX_NAME_LENGTH) {
    errors.push({
      field: 'name',
      message: `Model name must not exceed ${MAX_NAME_LENGTH} characters`,
    });
  }

  if (!metadata.description) {
    errors.push({ field: 'description', message: 'Description is required' });
  } else if (metadata.description.length < MIN_DESCRIPTION_LENGTH) {
    errors.push({
      field: 'description',
      message: `Description must be at least ${MIN_DESCRIPTION_LENGTH} characters`,
    });
  } else if (metadata.description.length > MAX_DESCRIPTION_LENGTH) {
    errors.push({
      field: 'description',
      message: `Description must not exceed ${MAX_DESCRIPTION_LENGTH} characters`,
    });
  }

  if (metadata.category === undefined || metadata.category === null) {
    errors.push({ field: 'category', message: 'Category is required' });
  } else if (!isValidCategory(metadata.category)) {
    errors.push({ field: 'category', message: 'Invalid category' });
  }

  if (!metadata.tags || metadata.tags.length === 0) {
    warnings.push({ field: 'tags', message: 'Tags help with discoverability' });
  } else {
    if (metadata.tags.length > MAX_TAGS) {
      errors.push({
        field: 'tags',
        message: `Maximum ${MAX_TAGS} tags allowed`,
      });
    }

    metadata.tags.forEach((tag, index) => {
      if (tag.length > MAX_TAG_LENGTH) {
        errors.push({
          field: `tags[${index}]`,
          message: `Tag "${tag}" exceeds ${MAX_TAG_LENGTH} characters`,
        });
      }
      if (tag.trim() === '') {
        errors.push({
          field: `tags[${index}]`,
          message: 'Empty tags are not allowed',
        });
      }
    });
  }

  // Optional but recommended fields
  if (!metadata.framework) {
    warnings.push({
      field: 'framework',
      message: 'Specifying a framework helps users understand compatibility',
    });
  }

  if (!metadata.license) {
    warnings.push({
      field: 'license',
      message: 'License information is important for users',
    });
  }

  if (!metadata.examples || metadata.examples.length === 0) {
    warnings.push({
      field: 'examples',
      message: 'Adding examples helps users understand model capabilities',
    });
  } else if (metadata.examples.length > MAX_EXAMPLES) {
    errors.push({
      field: 'examples',
      message: `Maximum ${MAX_EXAMPLES} examples allowed`,
    });
  } else {
    metadata.examples.forEach((example, index) => {
      const exampleErrors = validateModelExample(example);
      exampleErrors.forEach((error) => {
        errors.push({
          field: `examples[${index}].${error.field}`,
          message: error.message,
        });
      });
    });
  }

  // Validate benchmarks
  if (metadata.benchmarks && metadata.benchmarks.length > MAX_BENCHMARKS) {
    errors.push({
      field: 'benchmarks',
      message: `Maximum ${MAX_BENCHMARKS} benchmarks allowed`,
    });
  } else if (metadata.benchmarks) {
    metadata.benchmarks.forEach((benchmark, index) => {
      const benchmarkErrors = validateBenchmark(benchmark);
      benchmarkErrors.forEach((error) => {
        errors.push({
          field: `benchmarks[${index}].${error.field}`,
          message: error.message,
        });
      });
    });
  }

  // Validate supported formats
  if (metadata.supportedFormats) {
    if (
      !metadata.supportedFormats.input ||
      metadata.supportedFormats.input.length === 0
    ) {
      warnings.push({
        field: 'supportedFormats.input',
        message: 'Specify supported input formats',
      });
    }
    if (
      !metadata.supportedFormats.output ||
      metadata.supportedFormats.output.length === 0
    ) {
      warnings.push({
        field: 'supportedFormats.output',
        message: 'Specify supported output formats',
      });
    }
  }

  // Validate model size
  if (metadata.modelSize && !isValidModelSize(metadata.modelSize)) {
    errors.push({ field: 'modelSize', message: 'Invalid model size category' });
  }

  // Validate size bytes
  if (metadata.sizeBytes !== undefined) {
    if (metadata.sizeBytes < 0) {
      errors.push({
        field: 'sizeBytes',
        message: 'Model size cannot be negative',
      });
    }
    if (metadata.sizeBytes > 1024 * 1024 * 1024 * 1024) {
      // 1TB
      warnings.push({
        field: 'sizeBytes',
        message: 'Model size seems unusually large',
      });
    }
  }

  // Validate version format
  if (metadata.version) {
    const versionRegex = /^v?\d+\.\d+(\.\d+)?(-[a-zA-Z0-9]+)?$/;
    if (!versionRegex.test(metadata.version)) {
      warnings.push({
        field: 'version',
        message: 'Version should follow semantic versioning (e.g., 1.0.0)',
      });
    }
  }

  return {
    valid: errors.length === 0,
    errors,
    warnings,
  };
}

/**
 * Validate a model example
 */
function validateModelExample(
  example: Partial<ModelExample>
): { field: string; message: string }[] {
  const errors: { field: string; message: string }[] = [];

  if (!example.title || example.title.trim() === '') {
    errors.push({ field: 'title', message: 'Example title is required' });
  }

  if (!example.input || example.input.trim() === '') {
    errors.push({ field: 'input', message: 'Example input is required' });
  }

  if (!example.output || example.output.trim() === '') {
    errors.push({ field: 'output', message: 'Example output is required' });
  }

  return errors;
}

/**
 * Validate a performance benchmark
 */
function validateBenchmark(
  benchmark: Partial<PerformanceBenchmark>
): { field: string; message: string }[] {
  const errors: { field: string; message: string }[] = [];

  if (!benchmark.metric || benchmark.metric.trim() === '') {
    errors.push({ field: 'metric', message: 'Benchmark metric is required' });
  }

  if (benchmark.value === undefined || benchmark.value === null) {
    errors.push({ field: 'value', message: 'Benchmark value is required' });
  } else if (typeof benchmark.value !== 'number') {
    errors.push({ field: 'value', message: 'Benchmark value must be a number' });
  }

  if (!benchmark.unit || benchmark.unit.trim() === '') {
    errors.push({ field: 'unit', message: 'Benchmark unit is required' });
  }

  return errors;
}

/**
 * Check if category is valid
 */
function isValidCategory(category: any): boolean {
  return Object.values(ModelCategory).includes(category);
}

/**
 * Check if model size is valid
 */
function isValidModelSize(size: any): boolean {
  return Object.values(ModelSize).includes(size);
}

// ============================================================================
// Review Validation
// ============================================================================

/**
 * Validate a review submission
 */
export function validateReview(
  review: Partial<ReviewSubmission>
): ValidationResult {
  const errors: { field: string; message: string }[] = [];
  const warnings: { field: string; message: string }[] = [];

  if (!review.modelId) {
    errors.push({ field: 'modelId', message: 'Model ID is required' });
  }

  if (!review.reviewer) {
    errors.push({ field: 'reviewer', message: 'Reviewer address is required' });
  }

  if (review.rating === undefined || review.rating === null) {
    errors.push({ field: 'rating', message: 'Rating is required' });
  } else if (review.rating < 1 || review.rating > 5) {
    errors.push({ field: 'rating', message: 'Rating must be between 1 and 5' });
  }

  if (!review.text || review.text.trim() === '') {
    errors.push({ field: 'text', message: 'Review text is required' });
  } else {
    const sanitizedText = sanitizeUserInput(review.text);

    if (sanitizedText.length < MIN_REVIEW_LENGTH) {
      errors.push({
        field: 'text',
        message: `Review must be at least ${MIN_REVIEW_LENGTH} characters`,
      });
    }

    if (sanitizedText.length > MAX_REVIEW_LENGTH) {
      errors.push({
        field: 'text',
        message: `Review must not exceed ${MAX_REVIEW_LENGTH} characters`,
      });
    }

    // Check for spam patterns
    if (isSpammy(sanitizedText)) {
      errors.push({
        field: 'text',
        message: 'Review appears to contain spam or suspicious content',
      });
    }

    // Check for excessive caps
    if (hasExcessiveCaps(sanitizedText)) {
      warnings.push({
        field: 'text',
        message: 'Avoid excessive use of capital letters',
      });
    }
  }

  return {
    valid: errors.length === 0,
    errors,
    warnings,
  };
}

// ============================================================================
// Input Sanitization
// ============================================================================

/**
 * Sanitize user input to prevent XSS and other attacks
 */
export function sanitizeUserInput(input: string): string {
  if (!input) return '';

  let sanitized = input.trim();

  // Remove null bytes
  sanitized = sanitized.replace(/\0/g, '');

  // Remove script tags
  sanitized = sanitized.replace(/<script\b[^<]*(?:(?!<\/script>)<[^<]*)*<\/script>/gi, '');

  // Remove potentially dangerous HTML tags
  const dangerousTags = [
    'iframe',
    'embed',
    'object',
    'applet',
    'meta',
    'link',
    'style',
  ];
  dangerousTags.forEach((tag) => {
    const regex = new RegExp(`<${tag}\\b[^<]*(?:(?!<\\/${tag}>)<[^<]*)*<\\/${tag}>`, 'gi');
    sanitized = sanitized.replace(regex, '');
  });

  // Encode HTML entities to prevent XSS
  sanitized = sanitized
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&#x27;')
    .replace(/\//g, '&#x2F;');

  // Limit consecutive newlines
  sanitized = sanitized.replace(/\n{3,}/g, '\n\n');

  return sanitized;
}

/**
 * Sanitize markdown input (allows some formatting)
 */
export function sanitizeMarkdown(input: string): string {
  if (!input) return '';

  let sanitized = input.trim();

  // Remove null bytes
  sanitized = sanitized.replace(/\0/g, '');

  // Remove script tags and dangerous HTML
  sanitized = sanitized.replace(/<script\b[^<]*(?:(?!<\/script>)<[^<]*)*<\/script>/gi, '');

  const dangerousTags = [
    'iframe',
    'embed',
    'object',
    'applet',
    'meta',
    'link',
    'style',
  ];
  dangerousTags.forEach((tag) => {
    const regex = new RegExp(`<${tag}\\b[^<]*(?:(?!<\\/${tag}>)<[^<]*)*<\\/${tag}>`, 'gi');
    sanitized = sanitized.replace(regex, '');
  });

  // Sanitize javascript: and data: URLs
  sanitized = sanitized.replace(/javascript:/gi, '');
  sanitized = sanitized.replace(/data:text\/html/gi, '');

  return sanitized;
}

// ============================================================================
// Spam and Content Detection
// ============================================================================

/**
 * Check if text appears to be spam
 */
function isSpammy(text: string): boolean {
  const spamPatterns = [
    /click here/gi,
    /buy now/gi,
    /limited time/gi,
    /act now/gi,
    /visit.*\.com/gi,
    /https?:\/\/[^\s]{50,}/g, // Very long URLs
    /(.)\1{10,}/, // Excessive character repetition
    /🎁|💰|💵|💴|💶|💷/g, // Spam emojis
  ];

  return spamPatterns.some((pattern) => pattern.test(text));
}

/**
 * Check for excessive capitalization
 */
function hasExcessiveCaps(text: string): boolean {
  if (text.length < 20) return false;

  const words = text.split(/\s+/);
  const capsWords = words.filter((word) => {
    return word.length > 2 && word === word.toUpperCase();
  });

  // More than 30% of words are all caps
  return capsWords.length / words.length > 0.3;
}

// ============================================================================
// Validation Helpers
// ============================================================================

/**
 * Validate Ethereum address
 */
export function isValidAddress(address: string): boolean {
  return /^0x[a-fA-F0-9]{40}$/.test(address);
}

/**
 * Validate IPFS CID
 */
export function isValidIPFSCID(cid: string): boolean {
  // CIDv0 (Qm...) or CIDv1 (bafy...)
  return /^(Qm[1-9A-HJ-NP-Za-km-z]{44}|bafy[0-9a-z]{55})$/.test(cid);
}

/**
 * Validate URL
 */
export function isValidURL(url: string): boolean {
  try {
    const parsed = new URL(url);
    return parsed.protocol === 'http:' || parsed.protocol === 'https:';
  } catch {
    return false;
  }
}

/**
 * Get field-specific validation message
 */
export function getValidationMessage(
  field: string,
  errors: { field: string; message: string }[]
): string | null {
  const error = errors.find((e) => e.field === field);
  return error ? error.message : null;
}

/**
 * Check if field has error
 */
export function hasFieldError(
  field: string,
  errors: { field: string; message: string }[]
): boolean {
  return errors.some((e) => e.field === field);
}
