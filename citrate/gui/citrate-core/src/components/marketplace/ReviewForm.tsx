/**
 * ReviewForm Component
 *
 * Form for submitting a new review with:
 * - Interactive star rating selector
 * - Review text textarea with character counter
 * - Validation
 * - Verified purchase check
 * - Success/error messages
 */

import React, { useState } from 'react';
import { ReviewSubmission } from '../../utils/types/reviews';
import { validateReview, sanitizeUserInput } from '../../utils/metadata/validation';

export interface ReviewFormProps {
  modelId: string;
  userAddress: string;
  hasPurchased: boolean;
  onSubmit: (review: Omit<ReviewSubmission, 'reviewer'>, password: string) => Promise<void>;
  className?: string;
}

const MIN_CHARS = 50;
const MAX_CHARS = 1000;

export const ReviewForm: React.FC<ReviewFormProps> = ({
  modelId,
  userAddress,
  hasPurchased,
  onSubmit,
  className = '',
}) => {
  const [rating, setRating] = useState(0);
  const [hoverRating, setHoverRating] = useState(0);
  const [text, setText] = useState('');
  const [password, setPassword] = useState('');
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState(false);

  const charCount = text.length;
  const isValid = rating > 0 && charCount >= MIN_CHARS && charCount <= MAX_CHARS && password.length > 0;

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError(null);
    setSuccess(false);

    const review: Omit<ReviewSubmission, 'reviewer'> = {
      modelId,
      rating,
      text: sanitizeUserInput(text),
      verifiedPurchase: hasPurchased,
    };

    // Validate
    const validation = validateReview({ ...review, reviewer: userAddress });
    if (!validation.valid) {
      setError(validation.errors.map((e) => e.message).join('. '));
      return;
    }

    setIsSubmitting(true);

    try {
      await onSubmit(review, password);
      setSuccess(true);
      setRating(0);
      setText('');
      setPassword('');

      setTimeout(() => setSuccess(false), 5000);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to submit review');
    } finally {
      setIsSubmitting(false);
    }
  };

  const renderStars = () => {
    return (
      <div className="star-selector">
        {[1, 2, 3, 4, 5].map((star) => (
          <button
            key={star}
            type="button"
            className={`star ${(hoverRating || rating) >= star ? 'active' : ''}`}
            onClick={() => setRating(star)}
            onMouseEnter={() => setHoverRating(star)}
            onMouseLeave={() => setHoverRating(0)}
            aria-label={`Rate ${star} stars`}
          >
            ★
          </button>
        ))}
        <span className="rating-text">
          {rating > 0 ? `${rating} star${rating !== 1 ? 's' : ''}` : 'Select rating'}
        </span>
      </div>
    );
  };

  return (
    <form className={`review-form ${className}`} onSubmit={handleSubmit}>
      <h3>Write a Review</h3>

      {!hasPurchased && (
        <div className="info-banner">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg">
            <path d="M12 16V12M12 8H12.01M22 12C22 17.5228 17.5228 22 12 22C6.47715 22 2 17.5228 2 12C2 6.47715 6.47715 2 12 2C17.5228 2 22 6.47715 22 12Z" stroke="currentColor" strokeWidth="2" strokeLinecap="round"/>
          </svg>
          You haven't purchased this model. Your review won't be marked as verified.
        </div>
      )}

      <div className="form-group">
        <label>Rating *</label>
        {renderStars()}
      </div>

      <div className="form-group">
        <label htmlFor="review-text">
          Review * <span className="char-count">{charCount}/{MAX_CHARS}</span>
        </label>
        <textarea
          id="review-text"
          value={text}
          onChange={(e) => setText(e.target.value)}
          placeholder={`Share your experience with this model (minimum ${MIN_CHARS} characters)...`}
          maxLength={MAX_CHARS}
          rows={6}
          className={charCount < MIN_CHARS && text.length > 0 ? 'invalid' : ''}
        />
        {charCount > 0 && charCount < MIN_CHARS && (
          <span className="validation-hint">
            {MIN_CHARS - charCount} more character{MIN_CHARS - charCount !== 1 ? 's' : ''} required
          </span>
        )}
      </div>

      <div className="form-group">
        <label htmlFor="password">Password *</label>
        <input
          id="password"
          type="password"
          value={password}
          onChange={(e) => setPassword(e.target.value)}
          placeholder="Enter your wallet password"
          required
        />
      </div>

      {error && (
        <div className="error-message">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg">
            <path d="M12 8V12M12 16H12.01M22 12C22 17.5228 17.5228 22 12 22C6.47715 22 2 17.5228 2 12C2 6.47715 6.47715 2 12 2C17.5228 2 22 6.47715 22 12Z" stroke="currentColor" strokeWidth="2" strokeLinecap="round"/>
          </svg>
          {error}
        </div>
      )}

      {success && (
        <div className="success-message">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg">
            <path d="M9 12L11 14L15 10M21 12C21 16.9706 16.9706 21 12 21C7.02944 21 3 16.9706 3 12C3 7.02944 7.02944 3 12 3C16.9706 3 21 7.02944 21 12Z" stroke="currentColor" strokeWidth="2" strokeLinecap="round"/>
          </svg>
          Review submitted successfully!
        </div>
      )}

      <button
        type="submit"
        className="submit-btn"
        disabled={!isValid || isSubmitting}
      >
        {isSubmitting ? 'Submitting...' : 'Submit Review'}
      </button>

      <style jsx>{`
        .review-form {
          background: white;
          border: 1px solid #e5e7eb;
          border-radius: 12px;
          padding: 24px;
        }

        .review-form h3 {
          margin: 0 0 20px 0;
          font-size: 20px;
          font-weight: 700;
          color: #111827;
        }

        .info-banner {
          display: flex;
          align-items: center;
          gap: 10px;
          background: #fffbeb;
          border: 1px solid #fde68a;
          border-radius: 8px;
          padding: 12px 16px;
          margin-bottom: 20px;
          color: #92400e;
          font-size: 14px;
        }

        .info-banner svg {
          flex-shrink: 0;
          color: #f59e0b;
        }

        .form-group {
          margin-bottom: 20px;
        }

        .form-group label {
          display: flex;
          justify-content: space-between;
          align-items: center;
          margin-bottom: 8px;
          font-weight: 600;
          color: #374151;
          font-size: 14px;
        }

        .char-count {
          font-weight: 400;
          color: #9ca3af;
          font-size: 13px;
        }

        .star-selector {
          display: flex;
          align-items: center;
          gap: 8px;
        }

        .star {
          background: none;
          border: none;
          font-size: 32px;
          color: #d1d5db;
          cursor: pointer;
          padding: 4px;
          transition: all 0.2s ease;
        }

        .star:hover,
        .star.active {
          color: #fbbf24;
          transform: scale(1.1);
        }

        .rating-text {
          margin-left: 8px;
          color: #6b7280;
          font-size: 14px;
          font-weight: 500;
        }

        textarea {
          width: 100%;
          padding: 12px;
          border: 1px solid #e5e7eb;
          border-radius: 8px;
          font-family: inherit;
          font-size: 14px;
          color: #111827;
          resize: vertical;
          transition: border-color 0.2s ease;
        }

        textarea:focus {
          outline: none;
          border-color: #3b82f6;
        }

        textarea.invalid {
          border-color: #fbbf24;
        }

        .validation-hint {
          display: block;
          margin-top: 6px;
          color: #f59e0b;
          font-size: 13px;
        }

        input[type="password"] {
          width: 100%;
          padding: 12px;
          border: 1px solid #e5e7eb;
          border-radius: 8px;
          font-family: inherit;
          font-size: 14px;
          color: #111827;
        }

        input[type="password"]:focus {
          outline: none;
          border-color: #3b82f6;
        }

        .error-message {
          display: flex;
          align-items: center;
          gap: 10px;
          background: #fef2f2;
          border: 1px solid #fecaca;
          border-radius: 8px;
          padding: 12px 16px;
          margin-bottom: 16px;
          color: #991b1b;
          font-size: 14px;
        }

        .error-message svg {
          flex-shrink: 0;
          color: #ef4444;
        }

        .success-message {
          display: flex;
          align-items: center;
          gap: 10px;
          background: #f0fdf4;
          border: 1px solid #bbf7d0;
          border-radius: 8px;
          padding: 12px 16px;
          margin-bottom: 16px;
          color: #166534;
          font-size: 14px;
        }

        .success-message svg {
          flex-shrink: 0;
          color: #22c55e;
        }

        .submit-btn {
          width: 100%;
          padding: 12px 24px;
          background: #3b82f6;
          color: white;
          border: none;
          border-radius: 8px;
          font-weight: 600;
          font-size: 15px;
          cursor: pointer;
          transition: all 0.2s ease;
        }

        .submit-btn:hover:not(:disabled) {
          background: #2563eb;
        }

        .submit-btn:disabled {
          background: #d1d5db;
          cursor: not-allowed;
        }
      `}</style>
    </form>
  );
};

export default ReviewForm;
