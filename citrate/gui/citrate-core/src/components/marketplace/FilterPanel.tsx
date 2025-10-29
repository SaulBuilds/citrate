/**
 * FilterPanel Component
 *
 * Advanced filtering interface for model marketplace search.
 * Features:
 * - Category multi-select
 * - Price range slider
 * - Rating filter
 * - Framework multi-select
 * - Model size multi-select
 * - Featured/Active toggles
 * - Clear all filters
 * - Active filter count badge
 */

import React, { useState, useEffect } from 'react';
import {
  SearchFilters,
  ModelCategory,
  ModelSize,
  CATEGORY_INFO,
  SIZE_INFO,
  formatPrice
} from '../../utils/search';

export interface FilterPanelProps {
  filters: SearchFilters;
  onFiltersChange: (filters: SearchFilters) => void;
  availableFrameworks?: string[];
  className?: string;
  collapsible?: boolean;
}

export const FilterPanel: React.FC<FilterPanelProps> = ({
  filters,
  onFiltersChange,
  availableFrameworks = ['PyTorch', 'TensorFlow', 'JAX', 'ONNX', 'Transformers'],
  className = '',
  collapsible = true
}) => {
  const [isExpanded, setIsExpanded] = useState(!collapsible);
  const [localFilters, setLocalFilters] = useState<SearchFilters>(filters);

  // Sync local filters with props
  useEffect(() => {
    setLocalFilters(filters);
  }, [filters]);

  const updateFilters = (updates: Partial<SearchFilters>) => {
    const newFilters = { ...localFilters, ...updates };
    setLocalFilters(newFilters);
    onFiltersChange(newFilters);
  };

  const handleCategoryToggle = (category: ModelCategory) => {
    const categories = localFilters.categories || [];
    const newCategories = categories.includes(category)
      ? categories.filter(c => c !== category)
      : [...categories, category];

    updateFilters({
      categories: newCategories.length > 0 ? newCategories : undefined
    });
  };

  const handleFrameworkToggle = (framework: string) => {
    const frameworks = localFilters.frameworks || [];
    const newFrameworks = frameworks.includes(framework)
      ? frameworks.filter(f => f !== framework)
      : [...frameworks, framework];

    updateFilters({
      frameworks: newFrameworks.length > 0 ? newFrameworks : undefined
    });
  };

  const handleModelSizeToggle = (size: ModelSize) => {
    const sizes = localFilters.modelSizes || [];
    const newSizes = sizes.includes(size)
      ? sizes.filter(s => s !== size)
      : [...sizes, size];

    updateFilters({
      modelSizes: newSizes.length > 0 ? newSizes : undefined
    });
  };

  const handlePriceChange = (min?: number, max?: number) => {
    updateFilters({
      priceMin: min,
      priceMax: max
    });
  };

  const handleRatingChange = (rating?: number) => {
    updateFilters({ ratingMin: rating });
  };

  const handleClearAll = () => {
    const emptyFilters: SearchFilters = {
      activeOnly: true  // Keep activeOnly as default
    };
    setLocalFilters(emptyFilters);
    onFiltersChange(emptyFilters);
  };

  const getActiveFilterCount = (): number => {
    let count = 0;
    if (localFilters.categories && localFilters.categories.length > 0) count++;
    if (localFilters.priceMin !== undefined || localFilters.priceMax !== undefined) count++;
    if (localFilters.ratingMin !== undefined) count++;
    if (localFilters.frameworks && localFilters.frameworks.length > 0) count++;
    if (localFilters.modelSizes && localFilters.modelSizes.length > 0) count++;
    if (localFilters.featuredOnly) count++;
    if (localFilters.activeOnly === false) count++;
    return count;
  };

  const activeFilterCount = getActiveFilterCount();

  return (
    <div className={`filter-panel ${className}`}>
      {collapsible && (
        <button
          onClick={() => setIsExpanded(!isExpanded)}
          className="filter-panel-header"
          aria-expanded={isExpanded}
        >
          <div className="filter-header-content">
            <svg
              className={`filter-icon ${isExpanded ? 'expanded' : ''}`}
              xmlns="http://www.w3.org/2000/svg"
              width="20"
              height="20"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              strokeWidth="2"
              strokeLinecap="round"
              strokeLinejoin="round"
            >
              <polygon points="22 3 2 3 10 12.46 10 19 14 21 14 12.46 22 3"></polygon>
            </svg>
            <span className="filter-title">Filters</span>
            {activeFilterCount > 0 && (
              <span className="filter-badge">{activeFilterCount}</span>
            )}
          </div>
          <svg
            className={`chevron ${isExpanded ? 'expanded' : ''}`}
            xmlns="http://www.w3.org/2000/svg"
            width="20"
            height="20"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            strokeWidth="2"
            strokeLinecap="round"
            strokeLinejoin="round"
          >
            <polyline points="6 9 12 15 18 9"></polyline>
          </svg>
        </button>
      )}

      <div className={`filter-panel-content ${isExpanded ? 'expanded' : 'collapsed'}`}>
        {activeFilterCount > 0 && (
          <div className="filter-clear-section">
            <button onClick={handleClearAll} className="clear-all-button">
              Clear all filters
            </button>
          </div>
        )}

        {/* Categories */}
        <div className="filter-section">
          <h3 className="filter-section-title">Categories</h3>
          <div className="filter-options">
            {Object.values(ModelCategory).map(category => (
              <label key={category} className="filter-checkbox-label">
                <input
                  type="checkbox"
                  checked={localFilters.categories?.includes(category) || false}
                  onChange={() => handleCategoryToggle(category)}
                  className="filter-checkbox"
                />
                <span className="category-icon">{CATEGORY_INFO[category].icon}</span>
                <span className="filter-label-text">{CATEGORY_INFO[category].label}</span>
              </label>
            ))}
          </div>
        </div>

        {/* Price Range */}
        <div className="filter-section">
          <h3 className="filter-section-title">Price Range</h3>
          <div className="price-inputs">
            <div className="price-input-group">
              <label className="price-label">Min</label>
              <input
                type="number"
                value={localFilters.priceMin || ''}
                onChange={(e) => handlePriceChange(
                  e.target.value ? Number(e.target.value) : undefined,
                  localFilters.priceMax
                )}
                placeholder="0 wei"
                className="price-input"
                min="0"
              />
            </div>
            <div className="price-separator">-</div>
            <div className="price-input-group">
              <label className="price-label">Max</label>
              <input
                type="number"
                value={localFilters.priceMax || ''}
                onChange={(e) => handlePriceChange(
                  localFilters.priceMin,
                  e.target.value ? Number(e.target.value) : undefined
                )}
                placeholder="No limit"
                className="price-input"
                min="0"
              />
            </div>
          </div>
          {(localFilters.priceMin || localFilters.priceMax) && (
            <div className="price-display">
              {localFilters.priceMin && formatPrice(localFilters.priceMin)}
              {localFilters.priceMin && localFilters.priceMax && ' - '}
              {localFilters.priceMax && formatPrice(localFilters.priceMax)}
            </div>
          )}
        </div>

        {/* Rating */}
        <div className="filter-section">
          <h3 className="filter-section-title">Minimum Rating</h3>
          <div className="rating-options">
            {[5, 4, 3, 2, 1].map(rating => (
              <button
                key={rating}
                onClick={() => handleRatingChange(
                  localFilters.ratingMin === rating ? undefined : rating
                )}
                className={`rating-button ${
                  localFilters.ratingMin === rating ? 'selected' : ''
                }`}
              >
                {Array.from({ length: 5 }).map((_, i) => (
                  <span key={i} className={i < rating ? 'star filled' : 'star'}>
                    ★
                  </span>
                ))}
                <span className="rating-text">& up</span>
              </button>
            ))}
          </div>
        </div>

        {/* Frameworks */}
        <div className="filter-section">
          <h3 className="filter-section-title">Frameworks</h3>
          <div className="filter-options">
            {availableFrameworks.map(framework => (
              <label key={framework} className="filter-checkbox-label">
                <input
                  type="checkbox"
                  checked={localFilters.frameworks?.includes(framework) || false}
                  onChange={() => handleFrameworkToggle(framework)}
                  className="filter-checkbox"
                />
                <span className="filter-label-text">{framework}</span>
              </label>
            ))}
          </div>
        </div>

        {/* Model Sizes */}
        <div className="filter-section">
          <h3 className="filter-section-title">Model Size</h3>
          <div className="filter-options">
            {Object.values(ModelSize).map(size => (
              <label key={size} className="filter-checkbox-label">
                <input
                  type="checkbox"
                  checked={localFilters.modelSizes?.includes(size) || false}
                  onChange={() => handleModelSizeToggle(size)}
                  className="filter-checkbox"
                />
                <span
                  className="size-badge"
                  style={{ backgroundColor: SIZE_INFO[size].color }}
                >
                  {SIZE_INFO[size].label}
                </span>
                <span className="filter-label-text size-range">
                  {SIZE_INFO[size].range}
                </span>
              </label>
            ))}
          </div>
        </div>

        {/* Toggle Options */}
        <div className="filter-section">
          <h3 className="filter-section-title">Options</h3>
          <div className="filter-toggles">
            <label className="filter-toggle-label">
              <input
                type="checkbox"
                checked={localFilters.featuredOnly || false}
                onChange={(e) => updateFilters({ featuredOnly: e.target.checked || undefined })}
                className="filter-checkbox"
              />
              <span className="filter-label-text">Featured only</span>
            </label>
            <label className="filter-toggle-label">
              <input
                type="checkbox"
                checked={localFilters.activeOnly !== false}
                onChange={(e) => updateFilters({ activeOnly: e.target.checked })}
                className="filter-checkbox"
              />
              <span className="filter-label-text">Active only</span>
            </label>
          </div>
        </div>
      </div>

      <style jsx>{`
        .filter-panel {
          background: white;
          border: 1px solid #e5e7eb;
          border-radius: 8px;
          overflow: hidden;
        }

        .filter-panel-header {
          width: 100%;
          display: flex;
          align-items: center;
          justify-content: space-between;
          padding: 16px;
          background: white;
          border: none;
          cursor: pointer;
          transition: background 0.2s;
        }

        .filter-panel-header:hover {
          background: #f9fafb;
        }

        .filter-header-content {
          display: flex;
          align-items: center;
          gap: 12px;
        }

        .filter-icon {
          color: #6b7280;
          transition: transform 0.2s;
        }

        .filter-icon.expanded {
          color: #3b82f6;
        }

        .filter-title {
          font-size: 16px;
          font-weight: 600;
          color: #111827;
        }

        .filter-badge {
          background: #3b82f6;
          color: white;
          font-size: 12px;
          font-weight: 600;
          padding: 2px 8px;
          border-radius: 12px;
          min-width: 20px;
          text-align: center;
        }

        .chevron {
          color: #6b7280;
          transition: transform 0.2s;
        }

        .chevron.expanded {
          transform: rotate(180deg);
        }

        .filter-panel-content {
          max-height: 0;
          overflow: hidden;
          transition: max-height 0.3s ease-out;
        }

        .filter-panel-content.expanded {
          max-height: 2000px;
        }

        .filter-panel-content.collapsed {
          max-height: 0;
        }

        .filter-clear-section {
          padding: 0 16px 12px;
          border-bottom: 1px solid #e5e7eb;
        }

        .clear-all-button {
          width: 100%;
          padding: 8px 16px;
          background: #f3f4f6;
          border: 1px solid #d1d5db;
          border-radius: 6px;
          font-size: 14px;
          font-weight: 500;
          color: #374151;
          cursor: pointer;
          transition: all 0.2s;
        }

        .clear-all-button:hover {
          background: #e5e7eb;
        }

        .filter-section {
          padding: 16px;
          border-bottom: 1px solid #e5e7eb;
        }

        .filter-section:last-child {
          border-bottom: none;
        }

        .filter-section-title {
          font-size: 14px;
          font-weight: 600;
          color: #111827;
          margin: 0 0 12px 0;
        }

        .filter-options {
          display: flex;
          flex-direction: column;
          gap: 8px;
        }

        .filter-checkbox-label {
          display: flex;
          align-items: center;
          gap: 8px;
          cursor: pointer;
          padding: 6px 8px;
          border-radius: 4px;
          transition: background 0.15s;
        }

        .filter-checkbox-label:hover {
          background: #f9fafb;
        }

        .filter-checkbox {
          width: 16px;
          height: 16px;
          cursor: pointer;
        }

        .category-icon {
          font-size: 18px;
        }

        .filter-label-text {
          font-size: 14px;
          color: #374151;
          flex: 1;
        }

        .price-inputs {
          display: flex;
          align-items: flex-end;
          gap: 8px;
        }

        .price-input-group {
          flex: 1;
          display: flex;
          flex-direction: column;
          gap: 4px;
        }

        .price-label {
          font-size: 12px;
          color: #6b7280;
          font-weight: 500;
        }

        .price-input {
          width: 100%;
          padding: 8px;
          border: 1px solid #d1d5db;
          border-radius: 4px;
          font-size: 14px;
          outline: none;
          transition: border-color 0.2s;
        }

        .price-input:focus {
          border-color: #3b82f6;
        }

        .price-separator {
          color: #6b7280;
          margin-bottom: 8px;
        }

        .price-display {
          margin-top: 8px;
          font-size: 13px;
          color: #3b82f6;
          font-weight: 500;
        }

        .rating-options {
          display: flex;
          flex-direction: column;
          gap: 6px;
        }

        .rating-button {
          display: flex;
          align-items: center;
          gap: 8px;
          padding: 8px 12px;
          background: white;
          border: 1px solid #d1d5db;
          border-radius: 6px;
          cursor: pointer;
          transition: all 0.2s;
        }

        .rating-button:hover {
          border-color: #3b82f6;
          background: #eff6ff;
        }

        .rating-button.selected {
          border-color: #3b82f6;
          background: #dbeafe;
        }

        .star {
          color: #d1d5db;
          font-size: 16px;
        }

        .star.filled {
          color: #fbbf24;
        }

        .rating-text {
          font-size: 13px;
          color: #6b7280;
          margin-left: auto;
        }

        .size-badge {
          display: inline-block;
          padding: 2px 8px;
          border-radius: 4px;
          font-size: 11px;
          font-weight: 600;
          color: white;
        }

        .size-range {
          font-size: 12px;
          color: #6b7280;
        }

        .filter-toggles {
          display: flex;
          flex-direction: column;
          gap: 8px;
        }

        .filter-toggle-label {
          display: flex;
          align-items: center;
          gap: 8px;
          cursor: pointer;
          padding: 6px 8px;
          border-radius: 4px;
          transition: background 0.15s;
        }

        .filter-toggle-label:hover {
          background: #f9fafb;
        }
      `}</style>
    </div>
  );
};

export default FilterPanel;
