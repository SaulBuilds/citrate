/**
 * SearchBar Component
 *
 * Search input with autocomplete suggestions for the model marketplace.
 * Features:
 * - Debounced text input
 * - Real-time autocomplete suggestions
 * - Keyboard navigation (Arrow keys, Enter, Escape)
 * - Loading states
 * - Click-outside to close
 */

import React, { useState, useEffect, useRef, useCallback } from 'react';
import { SearchSuggestion } from '../../utils/search';
import { getSearchEngine } from '../../utils/search';

export interface SearchBarProps {
  onSearch: (query: string) => void;
  onSuggestionSelect?: (suggestion: SearchSuggestion) => void;
  placeholder?: string;
  className?: string;
  autoFocus?: boolean;
  debounceMs?: number;
}

export const SearchBar: React.FC<SearchBarProps> = ({
  onSearch,
  onSuggestionSelect,
  placeholder = 'Search models by name, tags, framework...',
  className = '',
  autoFocus = false,
  debounceMs = 300
}) => {
  const [query, setQuery] = useState('');
  const [suggestions, setSuggestions] = useState<SearchSuggestion[]>([]);
  const [showSuggestions, setShowSuggestions] = useState(false);
  const [selectedIndex, setSelectedIndex] = useState(-1);
  const [isLoading, setIsLoading] = useState(false);

  const inputRef = useRef<HTMLInputElement>(null);
  const suggestionsRef = useRef<HTMLDivElement>(null);
  const debounceTimerRef = useRef<NodeJS.Timeout | null>(null);

  // Focus input on mount if autoFocus
  useEffect(() => {
    if (autoFocus && inputRef.current) {
      inputRef.current.focus();
    }
  }, [autoFocus]);

  // Fetch suggestions when query changes (debounced)
  useEffect(() => {
    if (debounceTimerRef.current) {
      clearTimeout(debounceTimerRef.current);
    }

    if (query.trim().length < 2) {
      setSuggestions([]);
      setShowSuggestions(false);
      setIsLoading(false);
      return;
    }

    setIsLoading(true);

    debounceTimerRef.current = setTimeout(async () => {
      try {
        const searchEngine = getSearchEngine();
        if (searchEngine) {
          const results = await searchEngine.getSuggestions(query, 10);
          setSuggestions(results);
          setShowSuggestions(true);
          setSelectedIndex(-1);
        }
      } catch (error) {
        console.error('Failed to fetch suggestions:', error);
        setSuggestions([]);
      } finally {
        setIsLoading(false);
      }
    }, debounceMs);

    return () => {
      if (debounceTimerRef.current) {
        clearTimeout(debounceTimerRef.current);
      }
    };
  }, [query, debounceMs]);

  // Click outside to close suggestions
  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (
        suggestionsRef.current &&
        !suggestionsRef.current.contains(event.target as Node) &&
        inputRef.current &&
        !inputRef.current.contains(event.target as Node)
      ) {
        setShowSuggestions(false);
      }
    };

    document.addEventListener('mousedown', handleClickOutside);
    return () => {
      document.removeEventListener('mousedown', handleClickOutside);
    };
  }, []);

  const handleInputChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    setQuery(e.target.value);
  };

  const handleSubmit = (e?: React.FormEvent) => {
    e?.preventDefault();
    setShowSuggestions(false);
    onSearch(query);
  };

  const handleSuggestionClick = (suggestion: SearchSuggestion) => {
    setQuery(suggestion.text);
    setShowSuggestions(false);

    if (onSuggestionSelect) {
      onSuggestionSelect(suggestion);
    } else {
      onSearch(suggestion.text);
    }
  };

  const handleKeyDown = (e: React.KeyboardEvent<HTMLInputElement>) => {
    if (!showSuggestions || suggestions.length === 0) {
      if (e.key === 'Enter') {
        handleSubmit();
      }
      return;
    }

    switch (e.key) {
      case 'ArrowDown':
        e.preventDefault();
        setSelectedIndex(prev =>
          prev < suggestions.length - 1 ? prev + 1 : 0
        );
        break;

      case 'ArrowUp':
        e.preventDefault();
        setSelectedIndex(prev =>
          prev > 0 ? prev - 1 : suggestions.length - 1
        );
        break;

      case 'Enter':
        e.preventDefault();
        if (selectedIndex >= 0 && selectedIndex < suggestions.length) {
          handleSuggestionClick(suggestions[selectedIndex]);
        } else {
          handleSubmit();
        }
        break;

      case 'Escape':
        e.preventDefault();
        setShowSuggestions(false);
        setSelectedIndex(-1);
        break;
    }
  };

  const getSuggestionIcon = (type: SearchSuggestion['type']): string => {
    switch (type) {
      case 'model':
        return '🤖';
      case 'tag':
        return '🏷️';
      case 'framework':
        return '⚙️';
      case 'creator':
        return '👤';
      default:
        return '🔍';
    }
  };

  const getSuggestionTypeLabel = (type: SearchSuggestion['type']): string => {
    switch (type) {
      case 'model':
        return 'Model';
      case 'tag':
        return 'Tag';
      case 'framework':
        return 'Framework';
      case 'creator':
        return 'Creator';
      default:
        return '';
    }
  };

  return (
    <div className={`search-bar-container ${className}`}>
      <form onSubmit={handleSubmit} className="search-bar-form">
        <div className="search-input-wrapper">
          <svg
            className="search-icon"
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
            <circle cx="11" cy="11" r="8"></circle>
            <path d="m21 21-4.35-4.35"></path>
          </svg>

          <input
            ref={inputRef}
            type="text"
            value={query}
            onChange={handleInputChange}
            onKeyDown={handleKeyDown}
            placeholder={placeholder}
            className="search-input"
            aria-label="Search models"
            aria-autocomplete="list"
            aria-expanded={showSuggestions}
            aria-controls="search-suggestions"
          />

          {isLoading && (
            <div className="search-loading-spinner">
              <svg className="spinner" viewBox="0 0 50 50">
                <circle
                  className="spinner-path"
                  cx="25"
                  cy="25"
                  r="20"
                  fill="none"
                  strokeWidth="5"
                ></circle>
              </svg>
            </div>
          )}

          {query && (
            <button
              type="button"
              onClick={() => {
                setQuery('');
                setSuggestions([]);
                setShowSuggestions(false);
                inputRef.current?.focus();
              }}
              className="search-clear-button"
              aria-label="Clear search"
            >
              <svg
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
                <line x1="18" y1="6" x2="6" y2="18"></line>
                <line x1="6" y1="6" x2="18" y2="18"></line>
              </svg>
            </button>
          )}
        </div>

        <button type="submit" className="search-submit-button">
          Search
        </button>
      </form>

      {showSuggestions && suggestions.length > 0 && (
        <div
          ref={suggestionsRef}
          id="search-suggestions"
          className="search-suggestions"
          role="listbox"
        >
          {suggestions.map((suggestion, index) => (
            <button
              key={`${suggestion.type}-${suggestion.text}-${index}`}
              type="button"
              onClick={() => handleSuggestionClick(suggestion)}
              className={`search-suggestion-item ${
                index === selectedIndex ? 'selected' : ''
              }`}
              role="option"
              aria-selected={index === selectedIndex}
            >
              <span className="suggestion-icon">
                {getSuggestionIcon(suggestion.type)}
              </span>
              <div className="suggestion-content">
                <span className="suggestion-text">{suggestion.text}</span>
                <span className="suggestion-type">
                  {getSuggestionTypeLabel(suggestion.type)}
                  {suggestion.count !== undefined && ` (${suggestion.count})`}
                </span>
              </div>
            </button>
          ))}
        </div>
      )}

      <style jsx>{`
        .search-bar-container {
          position: relative;
          width: 100%;
          max-width: 600px;
        }

        .search-bar-form {
          display: flex;
          gap: 8px;
        }

        .search-input-wrapper {
          position: relative;
          flex: 1;
          display: flex;
          align-items: center;
        }

        .search-icon {
          position: absolute;
          left: 12px;
          color: #6b7280;
          pointer-events: none;
        }

        .search-input {
          width: 100%;
          padding: 12px 40px 12px 44px;
          font-size: 16px;
          border: 2px solid #e5e7eb;
          border-radius: 8px;
          outline: none;
          transition: all 0.2s;
          background: #ffffff;
        }

        .search-input:focus {
          border-color: #3b82f6;
          box-shadow: 0 0 0 3px rgba(59, 130, 246, 0.1);
        }

        .search-loading-spinner {
          position: absolute;
          right: 44px;
          width: 20px;
          height: 20px;
        }

        .spinner {
          animation: rotate 2s linear infinite;
          width: 20px;
          height: 20px;
        }

        .spinner-path {
          stroke: #3b82f6;
          stroke-linecap: round;
          animation: dash 1.5s ease-in-out infinite;
        }

        @keyframes rotate {
          100% {
            transform: rotate(360deg);
          }
        }

        @keyframes dash {
          0% {
            stroke-dasharray: 1, 150;
            stroke-dashoffset: 0;
          }
          50% {
            stroke-dasharray: 90, 150;
            stroke-dashoffset: -35;
          }
          100% {
            stroke-dasharray: 90, 150;
            stroke-dashoffset: -124;
          }
        }

        .search-clear-button {
          position: absolute;
          right: 12px;
          padding: 4px;
          background: transparent;
          border: none;
          cursor: pointer;
          color: #6b7280;
          border-radius: 4px;
          transition: all 0.2s;
        }

        .search-clear-button:hover {
          background: #f3f4f6;
          color: #374151;
        }

        .search-submit-button {
          padding: 12px 24px;
          background: #3b82f6;
          color: white;
          border: none;
          border-radius: 8px;
          font-size: 16px;
          font-weight: 500;
          cursor: pointer;
          transition: all 0.2s;
        }

        .search-submit-button:hover {
          background: #2563eb;
        }

        .search-submit-button:active {
          transform: scale(0.98);
        }

        .search-suggestions {
          position: absolute;
          top: calc(100% + 8px);
          left: 0;
          right: 0;
          background: white;
          border: 1px solid #e5e7eb;
          border-radius: 8px;
          box-shadow: 0 10px 25px rgba(0, 0, 0, 0.1);
          max-height: 400px;
          overflow-y: auto;
          z-index: 1000;
        }

        .search-suggestion-item {
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

        .search-suggestion-item:last-child {
          border-bottom: none;
        }

        .search-suggestion-item:hover,
        .search-suggestion-item.selected {
          background: #f9fafb;
        }

        .suggestion-icon {
          font-size: 20px;
          flex-shrink: 0;
        }

        .suggestion-content {
          flex: 1;
          display: flex;
          flex-direction: column;
          gap: 2px;
        }

        .suggestion-text {
          font-size: 15px;
          font-weight: 500;
          color: #111827;
        }

        .suggestion-type {
          font-size: 13px;
          color: #6b7280;
        }

        @media (max-width: 640px) {
          .search-bar-form {
            flex-direction: column;
          }

          .search-submit-button {
            width: 100%;
          }
        }
      `}</style>
    </div>
  );
};

export default SearchBar;
