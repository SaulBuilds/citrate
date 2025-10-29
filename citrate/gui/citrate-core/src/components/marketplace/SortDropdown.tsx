/**
 * SortDropdown Component
 *
 * Dropdown for selecting sort options in the model marketplace.
 * Features:
 * - Sort by relevance, rating, price, popularity, trending, recent
 * - Icon indicators for each sort type
 * - Keyboard accessible
 * - Click-outside to close
 * - Active sort highlighting
 */

import React, { useState, useRef, useEffect } from 'react';
import { SortOption } from '../../utils/search';

export interface SortDropdownProps {
  value: SortOption;
  onChange: (sort: SortOption) => void;
  className?: string;
  disabled?: boolean;
}

interface SortOptionInfo {
  value: SortOption;
  label: string;
  icon: string;
  description: string;
}

const SORT_OPTIONS: SortOptionInfo[] = [
  {
    value: 'relevance',
    label: 'Relevance',
    icon: '🎯',
    description: 'Best match for your search'
  },
  {
    value: 'rating_desc',
    label: 'Highest Rated',
    icon: '⭐',
    description: 'Top rated models first'
  },
  {
    value: 'price_asc',
    label: 'Lowest Price',
    icon: '💰',
    description: 'Cheapest models first'
  },
  {
    value: 'price_desc',
    label: 'Highest Price',
    icon: '💎',
    description: 'Most expensive models first'
  },
  {
    value: 'popularity',
    label: 'Most Popular',
    icon: '🔥',
    description: 'Most sales and usage'
  },
  {
    value: 'trending',
    label: 'Trending',
    icon: '📈',
    description: 'Rising in popularity'
  },
  {
    value: 'recent',
    label: 'Recently Listed',
    icon: '🆕',
    description: 'Newest models first'
  }
];

export const SortDropdown: React.FC<SortDropdownProps> = ({
  value,
  onChange,
  className = '',
  disabled = false
}) => {
  const [isOpen, setIsOpen] = useState(false);
  const dropdownRef = useRef<HTMLDivElement>(null);

  const selectedOption = SORT_OPTIONS.find(opt => opt.value === value) || SORT_OPTIONS[0];

  // Click outside to close
  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (dropdownRef.current && !dropdownRef.current.contains(event.target as Node)) {
        setIsOpen(false);
      }
    };

    if (isOpen) {
      document.addEventListener('mousedown', handleClickOutside);
      return () => {
        document.removeEventListener('mousedown', handleClickOutside);
      };
    }
  }, [isOpen]);

  // Close on escape key
  useEffect(() => {
    const handleEscape = (event: KeyboardEvent) => {
      if (event.key === 'Escape' && isOpen) {
        setIsOpen(false);
      }
    };

    if (isOpen) {
      document.addEventListener('keydown', handleEscape);
      return () => {
        document.removeEventListener('keydown', handleEscape);
      };
    }
  }, [isOpen]);

  const handleToggle = () => {
    if (!disabled) {
      setIsOpen(!isOpen);
    }
  };

  const handleSelect = (option: SortOption) => {
    onChange(option);
    setIsOpen(false);
  };

  return (
    <div ref={dropdownRef} className={`sort-dropdown ${className}`}>
      <button
        type="button"
        onClick={handleToggle}
        className={`sort-button ${isOpen ? 'open' : ''}`}
        disabled={disabled}
        aria-haspopup="listbox"
        aria-expanded={isOpen}
      >
        <span className="sort-icon">{selectedOption.icon}</span>
        <span className="sort-label">Sort: {selectedOption.label}</span>
        <svg
          className={`sort-chevron ${isOpen ? 'rotate' : ''}`}
          xmlns="http://www.w3.org/2000/svg"
          width="16"
          height="16"
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

      {isOpen && (
        <div className="sort-menu" role="listbox">
          {SORT_OPTIONS.map((option) => (
            <button
              key={option.value}
              type="button"
              onClick={() => handleSelect(option.value)}
              className={`sort-option ${value === option.value ? 'active' : ''}`}
              role="option"
              aria-selected={value === option.value}
            >
              <span className="option-icon">{option.icon}</span>
              <div className="option-content">
                <span className="option-label">{option.label}</span>
                <span className="option-description">{option.description}</span>
              </div>
              {value === option.value && (
                <svg
                  className="option-check"
                  xmlns="http://www.w3.org/2000/svg"
                  width="18"
                  height="18"
                  viewBox="0 0 24 24"
                  fill="none"
                  stroke="currentColor"
                  strokeWidth="3"
                  strokeLinecap="round"
                  strokeLinejoin="round"
                >
                  <polyline points="20 6 9 17 4 12"></polyline>
                </svg>
              )}
            </button>
          ))}
        </div>
      )}

      <style jsx>{`
        .sort-dropdown {
          position: relative;
          display: inline-block;
        }

        .sort-button {
          display: flex;
          align-items: center;
          gap: 8px;
          padding: 10px 16px;
          background: white;
          border: 1px solid #e5e7eb;
          border-radius: 8px;
          font-size: 14px;
          font-weight: 500;
          color: #374151;
          cursor: pointer;
          transition: all 0.2s;
          min-width: 200px;
        }

        .sort-button:hover:not(:disabled) {
          border-color: #3b82f6;
          background: #f9fafb;
        }

        .sort-button.open {
          border-color: #3b82f6;
          box-shadow: 0 0 0 3px rgba(59, 130, 246, 0.1);
        }

        .sort-button:disabled {
          opacity: 0.5;
          cursor: not-allowed;
        }

        .sort-icon {
          font-size: 16px;
          flex-shrink: 0;
        }

        .sort-label {
          flex: 1;
          text-align: left;
          white-space: nowrap;
        }

        .sort-chevron {
          flex-shrink: 0;
          transition: transform 0.2s;
          color: #6b7280;
        }

        .sort-chevron.rotate {
          transform: rotate(180deg);
        }

        .sort-menu {
          position: absolute;
          top: calc(100% + 4px);
          left: 0;
          right: 0;
          background: white;
          border: 1px solid #e5e7eb;
          border-radius: 8px;
          box-shadow: 0 10px 25px rgba(0, 0, 0, 0.1);
          z-index: 1000;
          overflow: hidden;
          animation: slideDown 0.2s ease-out;
        }

        @keyframes slideDown {
          from {
            opacity: 0;
            transform: translateY(-8px);
          }
          to {
            opacity: 1;
            transform: translateY(0);
          }
        }

        .sort-option {
          width: 100%;
          display: flex;
          align-items: center;
          gap: 12px;
          padding: 12px 16px;
          border: none;
          background: white;
          text-align: left;
          cursor: pointer;
          transition: background 0.15s;
          border-bottom: 1px solid #f3f4f6;
        }

        .sort-option:last-child {
          border-bottom: none;
        }

        .sort-option:hover {
          background: #f9fafb;
        }

        .sort-option.active {
          background: #eff6ff;
        }

        .option-icon {
          font-size: 18px;
          flex-shrink: 0;
        }

        .option-content {
          flex: 1;
          display: flex;
          flex-direction: column;
          gap: 2px;
        }

        .option-label {
          font-size: 14px;
          font-weight: 500;
          color: #111827;
        }

        .option-description {
          font-size: 12px;
          color: #6b7280;
        }

        .option-check {
          flex-shrink: 0;
          color: #3b82f6;
        }

        @media (max-width: 640px) {
          .sort-button {
            min-width: unset;
            width: 100%;
          }

          .sort-menu {
            left: 0;
            right: 0;
          }
        }
      `}</style>
    </div>
  );
};

export default SortDropdown;
