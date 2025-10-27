'use client';

import { useState, useCallback } from 'react';
import { Search, Sparkles, Brain, Database } from 'lucide-react';
import axios from 'axios';
import { useQuery } from '@tanstack/react-query';
import debounce from 'lodash/debounce';

interface SearchResult {
  type: 'block' | 'transaction' | 'address' | 'model' | 'inference';
  id: string;
  title: string;
  description: string;
  relevance: number;
  metadata?: any;
}

export function SemanticSearch() {
  const [query, setQuery] = useState('');
  const [isSearching, setIsSearching] = useState(false);
  const [results, setResults] = useState<SearchResult[]>([]);
  const [searchMode, setSearchMode] = useState<'semantic' | 'traditional'>('semantic');

  // Get genesis model for embeddings
  const { data: genesisModel } = useQuery({
    queryKey: ['genesis-model'],
    queryFn: async () => {
      const response = await axios.get('/api/models/genesis');
      return response.data;
    }
  });

  // Semantic search using the native chain model
  const performSemanticSearch = useCallback(
    debounce(async (searchQuery: string) => {
      if (searchQuery.length < 3) {
        setResults([]);
        return;
      }

      setIsSearching(true);
      try {
        // Use genesis model for embedding generation
        const embeddingResponse = await axios.post('/api/search/embed', {
          query: searchQuery,
          modelId: genesisModel?.modelId || 'genesis-bert-tiny'
        });

        const queryEmbedding = embeddingResponse.data.embedding;

        // Search using vector similarity
        const searchResponse = await axios.post('/api/search/semantic', {
          embedding: queryEmbedding,
          limit: 10,
          threshold: 0.7
        });

        const semanticResults = searchResponse.data.results.map((r: any) => ({
          ...r,
          relevance: r.similarity
        }));

        // Enhance with traditional search for hybrid approach
        const traditionalResponse = await axios.get('/api/search', {
          params: { q: searchQuery }
        });

        // Merge and rank results
        const mergedResults = mergeSearchResults(
          semanticResults,
          traditionalResponse.data.results
        );

        setResults(mergedResults);
      } catch (error) {
        console.error('Search error:', error);
      } finally {
        setIsSearching(false);
      }
    }, 500),
    [genesisModel]
  );

  const mergeSearchResults = (semantic: SearchResult[], traditional: SearchResult[]) => {
    const resultMap = new Map<string, SearchResult>();
    
    // Add semantic results with boost
    semantic.forEach(r => {
      resultMap.set(r.id, { ...r, relevance: r.relevance * 1.2 });
    });
    
    // Add traditional results
    traditional.forEach(r => {
      if (resultMap.has(r.id)) {
        // Boost if found in both
        const existing = resultMap.get(r.id)!;
        existing.relevance = (existing.relevance + r.relevance) / 1.5;
      } else {
        resultMap.set(r.id, r);
      }
    });
    
    // Sort by relevance
    return Array.from(resultMap.values())
      .sort((a, b) => b.relevance - a.relevance)
      .slice(0, 10);
  };

  const handleSearch = (e: React.ChangeEvent<HTMLInputElement>) => {
    const value = e.target.value;
    setQuery(value);
    
    if (searchMode === 'semantic') {
      performSemanticSearch(value);
    } else {
      // Traditional search
      performTraditionalSearch(value);
    }
  };

  const performTraditionalSearch = async (searchQuery: string) => {
    if (searchQuery.length < 3) {
      setResults([]);
      return;
    }

    setIsSearching(true);
    try {
      const response = await axios.get('/api/search', {
        params: { q: searchQuery }
      });
      setResults(response.data.results);
    } catch (error) {
      console.error('Search error:', error);
    } finally {
      setIsSearching(false);
    }
  };

  const getResultIcon = (type: string) => {
    switch (type) {
      case 'model':
        return <Brain className="w-4 h-4" />;
      case 'block':
        return <Database className="w-4 h-4" />;
      default:
        return <Search className="w-4 h-4" />;
    }
  };

  const getResultColor = (type: string) => {
    switch (type) {
      case 'model':
        return 'text-purple-500';
      case 'block':
        return 'text-blue-500';
      case 'transaction':
        return 'text-green-500';
      case 'address':
        return 'text-orange-500';
      default:
        return 'text-gray-500';
    }
  };

  return (
    <div className="relative w-full max-w-2xl mx-auto">
      {/* Search Input */}
      <div className="relative">
        <div className="absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none">
          {searchMode === 'semantic' ? (
            <Sparkles className="h-5 w-5 text-purple-400" />
          ) : (
            <Search className="h-5 w-5 text-gray-400" />
          )}
        </div>
        
        <input
          type="text"
          value={query}
          onChange={handleSearch}
          placeholder={
            searchMode === 'semantic'
              ? "Ask anything about the chain... (AI-powered)"
              : "Search blocks, transactions, addresses..."
          }
          className="block w-full pl-10 pr-20 py-3 border border-gray-300 dark:border-gray-600 rounded-xl bg-white dark:bg-gray-800 text-gray-900 dark:text-white placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
        />
        
        {/* Mode Toggle */}
        <div className="absolute inset-y-0 right-0 flex items-center pr-2">
          <button
            onClick={() => setSearchMode(searchMode === 'semantic' ? 'traditional' : 'semantic')}
            className={`px-3 py-1 rounded-lg text-xs font-medium transition-colors ${
              searchMode === 'semantic'
                ? 'bg-purple-100 text-purple-700 dark:bg-purple-900 dark:text-purple-300'
                : 'bg-gray-100 text-gray-700 dark:bg-gray-700 dark:text-gray-300'
            }`}
          >
            {searchMode === 'semantic' ? 'AI' : 'Classic'}
          </button>
        </div>
      </div>

      {/* Search Results */}
      {(isSearching || results.length > 0) && (
        <div className="absolute top-full mt-2 w-full bg-white dark:bg-gray-800 rounded-xl shadow-2xl border border-gray-200 dark:border-gray-700 overflow-hidden z-50">
          {isSearching ? (
            <div className="p-4 text-center">
              <div className="animate-spin rounded-full h-8 w-8 border-t-2 border-b-2 border-purple-500 mx-auto mb-2"></div>
              <p className="text-sm text-gray-500 dark:text-gray-400">
                {searchMode === 'semantic' ? 'AI is thinking...' : 'Searching...'}
              </p>
            </div>
          ) : (
            <div className="max-h-96 overflow-y-auto">
              {results.map((result) => (
                <a
                  key={result.id}
                  href={`/${result.type}/${result.id}`}
                  className="block px-4 py-3 hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors border-b border-gray-100 dark:border-gray-700 last:border-0"
                >
                  <div className="flex items-start gap-3">
                    <div className={`mt-1 ${getResultColor(result.type)}`}>
                      {getResultIcon(result.type)}
                    </div>
                    
                    <div className="flex-1 min-w-0">
                      <div className="flex items-center gap-2">
                        <h4 className="text-sm font-medium text-gray-900 dark:text-white truncate">
                          {result.title}
                        </h4>
                        <span className="text-xs px-2 py-0.5 bg-gray-100 dark:bg-gray-700 rounded-full text-gray-600 dark:text-gray-400">
                          {result.type}
                        </span>
                      </div>
                      
                      <p className="text-xs text-gray-500 dark:text-gray-400 mt-1 line-clamp-2">
                        {result.description}
                      </p>
                      
                      {searchMode === 'semantic' && (
                        <div className="flex items-center gap-2 mt-2">
                          <div className="flex-1 h-1 bg-gray-200 dark:bg-gray-700 rounded-full overflow-hidden">
                            <div
                              className="h-full bg-gradient-to-r from-purple-500 to-blue-500"
                              style={{ width: `${result.relevance * 100}%` }}
                            />
                          </div>
                          <span className="text-xs text-gray-500 dark:text-gray-400">
                            {(result.relevance * 100).toFixed(0)}% match
                          </span>
                        </div>
                      )}
                    </div>
                  </div>
                </a>
              ))}
              
              {results.length === 0 && !isSearching && query.length >= 3 && (
                <div className="p-4 text-center text-gray-500 dark:text-gray-400">
                  No results found
                </div>
              )}
            </div>
          )}
        </div>
      )}

      {/* Genesis Model Info */}
      {searchMode === 'semantic' && genesisModel && (
        <div className="mt-2 text-xs text-gray-500 dark:text-gray-400 text-center">
          Powered by {genesisModel.name} (Genesis Model)
        </div>
      )}
    </div>
  );
}