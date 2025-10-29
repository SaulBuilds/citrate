/**
 * useSearch Hook
 *
 * React hook for managing search state and interactions with the SearchEngine.
 * Handles:
 * - Query state management
 * - Filter state
 * - Sort option state
 * - Pagination state
 * - Search execution with debouncing
 * - Loading states
 */

import { useState, useEffect, useCallback, useRef } from 'react';
import {
  SearchQuery,
  SearchFilters,
  SearchResponse,
  SearchDocument,
  SortOption,
  getSearchEngine
} from '../utils/search';

export interface UseSearchOptions {
  initialQuery?: string;
  initialFilters?: SearchFilters;
  initialSort?: SortOption;
  initialPage?: number;
  pageSize?: number;
  debounceMs?: number;
  autoSearch?: boolean; // Automatically search on filter/query changes
}

export interface UseSearchResult {
  // State
  query: string;
  filters: SearchFilters;
  sortOption: SortOption;
  page: number;
  pageSize: number;
  results: SearchDocument[];
  total: number;
  isLoading: boolean;
  error: string | null;
  executionTimeMs: number;

  // Actions
  setQuery: (query: string) => void;
  setFilters: (filters: SearchFilters | ((prev: SearchFilters) => SearchFilters)) => void;
  setSortOption: (sort: SortOption) => void;
  setPage: (page: number) => void;
  search: () => Promise<void>;
  reset: () => void;
  nextPage: () => void;
  prevPage: () => void;
  goToPage: (page: number) => void;

  // Computed
  totalPages: number;
  hasNextPage: boolean;
  hasPrevPage: boolean;
  isEmpty: boolean;
}

const DEFAULT_PAGE_SIZE = 20;
const DEFAULT_DEBOUNCE_MS = 300;

export function useSearch(options: UseSearchOptions = {}): UseSearchResult {
  const {
    initialQuery = '',
    initialFilters = {},
    initialSort = 'relevance',
    initialPage = 0,
    pageSize = DEFAULT_PAGE_SIZE,
    debounceMs = DEFAULT_DEBOUNCE_MS,
    autoSearch = true
  } = options;

  // State
  const [query, setQuery] = useState(initialQuery);
  const [filters, setFilters] = useState<SearchFilters>(initialFilters);
  const [sortOption, setSortOption] = useState<SortOption>(initialSort);
  const [page, setPage] = useState(initialPage);
  const [results, setResults] = useState<SearchDocument[]>([]);
  const [total, setTotal] = useState(0);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [executionTimeMs, setExecutionTimeMs] = useState(0);

  // Refs
  const debounceTimerRef = useRef<NodeJS.Timeout | null>(null);
  const abortControllerRef = useRef<AbortController | null>(null);

  /**
   * Execute search with current state
   */
  const search = useCallback(async () => {
    const searchEngine = getSearchEngine();
    if (!searchEngine) {
      setError('Search engine not initialized');
      return;
    }

    // Abort previous search if still running
    if (abortControllerRef.current) {
      abortControllerRef.current.abort();
    }

    abortControllerRef.current = new AbortController();

    setIsLoading(true);
    setError(null);

    try {
      const searchQuery: SearchQuery = {
        text: query.trim() || undefined,
        filters,
        sort: sortOption,
        page,
        pageSize
      };

      const response: SearchResponse = await searchEngine.search(searchQuery);

      // Check if this search was aborted
      if (abortControllerRef.current?.signal.aborted) {
        return;
      }

      setResults(response.results.map(r => r.document));
      setTotal(response.total);
      setExecutionTimeMs(response.executionTimeMs);
      setError(null);
    } catch (err) {
      // Don't set error if aborted
      if (err instanceof Error && err.name === 'AbortError') {
        return;
      }

      const errorMessage = err instanceof Error ? err.message : 'Search failed';
      setError(errorMessage);
      console.error('Search error:', err);
      setResults([]);
      setTotal(0);
    } finally {
      setIsLoading(false);
      abortControllerRef.current = null;
    }
  }, [query, filters, sortOption, page, pageSize]);

  /**
   * Auto-search with debouncing when query/filters/sort changes
   */
  useEffect(() => {
    if (!autoSearch) {
      return;
    }

    // Clear existing timer
    if (debounceTimerRef.current) {
      clearTimeout(debounceTimerRef.current);
    }

    // Debounce search
    debounceTimerRef.current = setTimeout(() => {
      search();
    }, debounceMs);

    return () => {
      if (debounceTimerRef.current) {
        clearTimeout(debounceTimerRef.current);
      }
    };
  }, [query, filters, sortOption, page, autoSearch, debounceMs, search]);

  /**
   * Reset page to 0 when query, filters, or sort changes
   */
  useEffect(() => {
    setPage(0);
  }, [query, filters, sortOption]);

  /**
   * Reset all state to initial values
   */
  const reset = useCallback(() => {
    setQuery(initialQuery);
    setFilters(initialFilters);
    setSortOption(initialSort);
    setPage(initialPage);
    setResults([]);
    setTotal(0);
    setError(null);
    setExecutionTimeMs(0);
  }, [initialQuery, initialFilters, initialSort, initialPage]);

  /**
   * Pagination helpers
   */
  const totalPages = Math.ceil(total / pageSize);
  const hasNextPage = page < totalPages - 1;
  const hasPrevPage = page > 0;
  const isEmpty = results.length === 0 && !isLoading;

  const nextPage = useCallback(() => {
    if (hasNextPage) {
      setPage(p => p + 1);
    }
  }, [hasNextPage]);

  const prevPage = useCallback(() => {
    if (hasPrevPage) {
      setPage(p => p - 1);
    }
  }, [hasPrevPage]);

  const goToPage = useCallback((newPage: number) => {
    const clampedPage = Math.max(0, Math.min(newPage, totalPages - 1));
    setPage(clampedPage);
  }, [totalPages]);

  return {
    // State
    query,
    filters,
    sortOption,
    page,
    pageSize,
    results,
    total,
    isLoading,
    error,
    executionTimeMs,

    // Actions
    setQuery,
    setFilters,
    setSortOption,
    setPage,
    search,
    reset,
    nextPage,
    prevPage,
    goToPage,

    // Computed
    totalPages,
    hasNextPage,
    hasPrevPage,
    isEmpty
  };
}

export default useSearch;
