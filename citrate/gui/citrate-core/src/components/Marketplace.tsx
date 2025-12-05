import React, { useState, useEffect, useCallback } from 'react';
import { Upload, AlertCircle, Sparkles } from 'lucide-react';
import { SearchBar } from './marketplace/SearchBar';
import { FilterPanel } from './marketplace/FilterPanel';
import { SortDropdown } from './marketplace/SortDropdown';
import { SearchResults } from './marketplace/SearchResults';
import { Pagination } from './marketplace/Pagination';
import { useSearch } from '../hooks/useSearch';
import {
  initializeSearchIndex,
  initializeSearchEngine,
  getSearchEngine,
  SearchFilters,
  SortOption,
  SearchDocument
} from '../utils/search';
import {
  MarketplaceService,
  initMarketplaceService,
  CATEGORIES
} from '../utils/marketplaceService';
import {
  MarketplaceModel,
  loadMarketplaceModels,
  getMockMarketplaceModels,
  isMarketplaceAvailable
} from '../utils/marketplaceHelpers';

export const Marketplace: React.FC = () => {
  // Search state using the useSearch hook
  const {
    query,
    filters,
    sortOption,
    page,
    pageSize,
    results,
    total,
    isLoading: searchLoading,
    error: searchError,
    executionTimeMs,
    setQuery,
    setFilters,
    setSortOption,
    setPage,
    totalPages
  } = useSearch({
    initialSort: 'relevance',
    pageSize: 20,
    autoSearch: true,
    debounceMs: 300
  });

  // Marketplace initialization state
  const [initLoading, setInitLoading] = useState(true);
  const [initError, setInitError] = useState<string | null>(null);
  const [marketplaceService, setMarketplaceService] = useState<MarketplaceService | null>(null);
  const [selectedModel, setSelectedModel] = useState<SearchDocument | null>(null);
  const [availableFrameworks, setAvailableFrameworks] = useState<string[]>([
    'PyTorch',
    'TensorFlow',
    'JAX',
    'ONNX',
    'Transformers'
  ]);

  /**
   * Initialize marketplace and search system on mount
   */
  useEffect(() => {
    const initializeMarketplace = async () => {
      setInitLoading(true);
      setInitError(null);

      try {
        console.log('[Marketplace] Starting initialization...');

        // Step 1: Initialize marketplace service
        const service = await initMarketplaceService();
        setMarketplaceService(service);
        console.log('[Marketplace] Service initialized');

        // Step 2: Check if marketplace contract is available
        const available = await isMarketplaceAvailable(service);

        if (!available) {
          console.warn('[Marketplace] Contract not available, will use mock data for search index');
          setInitError('Marketplace contract not deployed. Showing example models.');
        } else {
          console.log('[Marketplace] Contract available');
        }

        // Step 3: Get ethers provider from service
        // Note: The service should expose a provider or we create one from RPC
        const provider = (service as any).provider || null;

        // Step 4: Get contract addresses from service
        const modelRegistryAddress = (service as any).modelRegistryAddress || '';
        const marketplaceAddress = (service as any).marketplaceAddress || '';

        if (!provider) {
          console.warn('[Marketplace] No provider available, search index will use fallback');
        }

        // Step 5: Initialize search index with contract data
        console.log('[Marketplace] Initializing search index...');

        // Skip contract initialization if addresses are empty - use mock data instead
        if (!marketplaceAddress || !modelRegistryAddress) {
          console.warn('[Marketplace] Contract addresses not configured, using mock data');
          setInitError('Marketplace contracts not deployed. Showing example models.');
          // Continue without contract-based indexing
        } else if (provider) {
          try {
            await initializeSearchIndex(provider, marketplaceAddress, modelRegistryAddress, {
              ipfsGateways: [
                'https://gateway.pinata.cloud/ipfs/',
                'https://cloudflare-ipfs.com/ipfs/',
                'https://ipfs.io/ipfs/'
              ],
              maxConcurrentFetches: 10,
              fetchTimeoutMs: 10000
            });
          } catch (indexError) {
            console.warn('[Marketplace] Search index initialization failed:', indexError);
            // Continue with search engine only
          }
        }

        // Step 6: Initialize search engine
        console.log('[Marketplace] Initializing search engine...');
        await initializeSearchEngine();

        const searchEngine = getSearchEngine();
        if (searchEngine) {
          const stats = searchEngine.getStatistics();
          console.log('[Marketplace] Search engine ready:', stats);

          // Extract unique frameworks from indexed documents
          const uniqueFrameworks = new Set<string>();
          const allDocs = await searchEngine.search({ text: '', pageSize: 1000 });
          allDocs.results.forEach(result => {
            if (result.document.framework) {
              uniqueFrameworks.add(result.document.framework);
            }
          });
          if (uniqueFrameworks.size > 0) {
            setAvailableFrameworks(Array.from(uniqueFrameworks).sort());
          }
        }

        console.log('[Marketplace] Initialization complete');
        setInitError(null);
      } catch (err: any) {
        console.error('[Marketplace] Initialization failed:', err);
        setInitError(
          err.message || 'Failed to initialize marketplace search system. Please refresh the page.'
        );
      } finally {
        setInitLoading(false);
      }
    };

    initializeMarketplace();
  }, []);

  /**
   * Handle search query changes from SearchBar
   */
  const handleSearch = useCallback((searchQuery: string) => {
    setQuery(searchQuery);
  }, [setQuery]);

  /**
   * Handle filter changes from FilterPanel
   */
  const handleFiltersChange = useCallback((newFilters: SearchFilters) => {
    setFilters(newFilters);
  }, [setFilters]);

  /**
   * Handle sort changes from SortDropdown
   */
  const handleSortChange = useCallback((sort: SortOption) => {
    setSortOption(sort);
  }, [setSortOption]);

  /**
   * Handle model click to show detail view
   */
  const handleModelClick = useCallback((modelId: string) => {
    const model = results.find(m => m.modelId === modelId);
    if (model) {
      setSelectedModel(model);
    }
  }, [results]);

  /**
   * Handle page changes from Pagination
   */
  const handlePageChange = useCallback((newPage: number) => {
    setPage(newPage);
    // Scroll to top when page changes
    window.scrollTo({ top: 0, behavior: 'smooth' });
  }, [setPage]);

  // Show initialization loading screen
  if (initLoading) {
    return (
      <div className="marketplace">
        <div className="init-loading">
          <div className="loading-spinner">
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
          <h3>Initializing Marketplace</h3>
          <p>Loading search index and contract data...</p>
        </div>

        <style jsx>{`
          .marketplace {
            padding: 2rem;
            min-height: 100vh;
            background: #f9fafb;
          }

          .init-loading {
            display: flex;
            flex-direction: column;
            align-items: center;
            justify-content: center;
            min-height: 60vh;
            gap: 24px;
          }

          .loading-spinner {
            width: 60px;
            height: 60px;
          }

          .spinner {
            animation: rotate 2s linear infinite;
            width: 60px;
            height: 60px;
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

          .init-loading h3 {
            margin: 0;
            font-size: 24px;
            font-weight: 600;
            color: #111827;
          }

          .init-loading p {
            margin: 0;
            font-size: 16px;
            color: #6b7280;
          }
        `}</style>
      </div>
    );
  }

  // Show initialization error screen
  if (initError) {
    return (
      <div className="marketplace">
        <div className="init-error">
          <div className="error-icon">
            <AlertCircle size={64} />
          </div>
          <h3>Initialization Failed</h3>
          <p>{initError}</p>
          <button
            onClick={() => window.location.reload()}
            className="retry-button"
          >
            Retry
          </button>
        </div>

        <style jsx>{`
          .marketplace {
            padding: 2rem;
            min-height: 100vh;
            background: #f9fafb;
          }

          .init-error {
            display: flex;
            flex-direction: column;
            align-items: center;
            justify-content: center;
            min-height: 60vh;
            gap: 24px;
          }

          .error-icon {
            color: #ef4444;
          }

          .init-error h3 {
            margin: 0;
            font-size: 24px;
            font-weight: 600;
            color: #111827;
          }

          .init-error p {
            margin: 0;
            font-size: 16px;
            color: #6b7280;
            text-align: center;
            max-width: 500px;
          }

          .retry-button {
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

          .retry-button:hover {
            background: #2563eb;
            transform: translateY(-2px);
            box-shadow: 0 4px 12px rgba(59, 130, 246, 0.3);
          }
        `}</style>
      </div>
    );
  }

  return (
    <div className="marketplace">
      {/* Header with logo and publish button */}
      <div className="marketplace-header">
        <div className="header-content">
          <Sparkles className="logo-icon" size={32} />
          <div>
            <h1>AI Model Marketplace</h1>
            <p>Discover and deploy cutting-edge AI models</p>
          </div>
        </div>
        <button className="publish-button">
          <Upload size={18} />
          Publish Model
        </button>
      </div>

      {/* Search bar - centered, full width */}
      <div className="search-section">
        <SearchBar
          onSearch={handleSearch}
          placeholder="Search models by name, tags, framework, creator..."
          autoFocus
        />
      </div>

      {/* Main content area - two column layout */}
      <div className="marketplace-content">
        {/* Left sidebar - Filter panel */}
        <aside className="filter-sidebar">
          <FilterPanel
            filters={filters}
            onFiltersChange={handleFiltersChange}
            availableFrameworks={availableFrameworks}
            collapsible={false}
          />
        </aside>

        {/* Right main content - Results and controls */}
        <main className="results-main">
          {/* Results header with count and sort */}
          <div className="results-header">
            <div className="results-info">
              {!searchLoading && (
                <span className="results-count">
                  {total.toLocaleString()} model{total !== 1 ? 's' : ''} found
                  {executionTimeMs > 0 && (
                    <span className="execution-time"> in {executionTimeMs.toFixed(0)}ms</span>
                  )}
                </span>
              )}
            </div>
            <SortDropdown
              value={sortOption}
              onChange={handleSortChange}
            />
          </div>

          {/* Search results grid */}
          <SearchResults
            results={results}
            isLoading={searchLoading}
            error={searchError}
            onModelClick={handleModelClick}
            executionTimeMs={executionTimeMs}
            showExecutionTime={false}
          />

          {/* Pagination */}
          {totalPages > 1 && (
            <Pagination
              currentPage={page}
              totalPages={totalPages}
              onPageChange={handlePageChange}
              totalResults={total}
              resultsPerPage={pageSize}
            />
          )}
        </main>
      </div>

      {/* Model details modal */}
      {selectedModel && (
        <ModelDetailsModal
          model={selectedModel}
          onClose={() => setSelectedModel(null)}
        />
      )}

      <style jsx>{`
        .marketplace {
          padding: 2rem;
          background: #f9fafb;
          min-height: 100vh;
          max-width: 1600px;
          margin: 0 auto;
        }

        .marketplace-header {
          display: flex;
          justify-content: space-between;
          align-items: center;
          margin-bottom: 2rem;
          padding-bottom: 2rem;
          border-bottom: 1px solid #e5e7eb;
        }

        .header-content {
          display: flex;
          align-items: center;
          gap: 1.5rem;
        }

        .logo-icon {
          color: #3b82f6;
        }

        .marketplace-header h1 {
          margin: 0;
          font-size: 2rem;
          font-weight: 700;
          color: #111827;
          background: linear-gradient(135deg, #3b82f6 0%, #8b5cf6 100%);
          -webkit-background-clip: text;
          -webkit-text-fill-color: transparent;
          background-clip: text;
        }

        .marketplace-header p {
          margin: 0.25rem 0 0 0;
          font-size: 1rem;
          color: #6b7280;
        }

        .publish-button {
          display: flex;
          align-items: center;
          gap: 0.5rem;
          padding: 0.75rem 1.5rem;
          background: linear-gradient(135deg, #3b82f6 0%, #2563eb 100%);
          color: white;
          border: none;
          border-radius: 0.5rem;
          font-size: 1rem;
          font-weight: 600;
          cursor: pointer;
          transition: all 0.2s;
          box-shadow: 0 4px 12px rgba(59, 130, 246, 0.3);
        }

        .publish-button:hover {
          transform: translateY(-2px);
          box-shadow: 0 8px 20px rgba(59, 130, 246, 0.4);
        }

        .publish-button:active {
          transform: translateY(0);
        }

        .search-section {
          display: flex;
          justify-content: center;
          margin-bottom: 2rem;
        }

        .marketplace-content {
          display: grid;
          grid-template-columns: 280px 1fr;
          gap: 2rem;
          align-items: start;
        }

        .filter-sidebar {
          position: sticky;
          top: 2rem;
        }

        .results-main {
          display: flex;
          flex-direction: column;
          gap: 1.5rem;
        }

        .results-header {
          display: flex;
          justify-content: space-between;
          align-items: center;
          gap: 1rem;
          flex-wrap: wrap;
        }

        .results-info {
          flex: 1;
        }

        .results-count {
          font-size: 1rem;
          font-weight: 600;
          color: #111827;
        }

        .execution-time {
          font-size: 0.875rem;
          font-weight: 400;
          color: #6b7280;
        }

        /* Mobile responsive */
        @media (max-width: 1024px) {
          .marketplace-content {
            grid-template-columns: 1fr;
          }

          .filter-sidebar {
            position: static;
          }
        }

        @media (max-width: 768px) {
          .marketplace {
            padding: 1rem;
          }

          .marketplace-header {
            flex-direction: column;
            align-items: flex-start;
            gap: 1rem;
          }

          .header-content {
            flex-direction: column;
            align-items: flex-start;
            gap: 0.75rem;
          }

          .marketplace-header h1 {
            font-size: 1.5rem;
          }

          .publish-button {
            width: 100%;
            justify-content: center;
          }

          .results-header {
            flex-direction: column;
            align-items: stretch;
          }
        }
      `}</style>
    </div>
  );
};

// Model Details Modal Component
const ModelDetailsModal: React.FC<{
  model: SearchDocument;
  onClose: () => void;
}> = ({ model, onClose }) => {
  const effectivePrice = Math.min(model.basePrice, model.discountPrice);
  const hasDiscount = model.discountPrice < model.basePrice;

  return (
    <div className="modal-overlay" onClick={onClose}>
      <div className="modal large" onClick={e => e.stopPropagation()}>
        <div className="modal-header">
          <h3>{model.name}</h3>
          <button className="close-btn" onClick={onClose}>×</button>
        </div>

        <div className="model-details-content">
          <div className="detail-section">
            <h4>Overview</h4>
            <p>{model.description}</p>

            <div className="detail-grid">
              <div className="detail-item">
                <span className="label">Creator:</span>
                <span className="value">
                  {model.creatorName || `${model.creatorAddress.slice(0, 6)}...${model.creatorAddress.slice(-4)}`}
                </span>
              </div>
              <div className="detail-item">
                <span className="label">Model ID:</span>
                <span className="value mono">{model.modelId}</span>
              </div>
              {model.framework && (
                <div className="detail-item">
                  <span className="label">Framework:</span>
                  <span className="value">{model.framework}</span>
                </div>
              )}
              {model.modelSize && (
                <div className="detail-item">
                  <span className="label">Model Size:</span>
                  <span className="value">{model.modelSize}</span>
                </div>
              )}
              <div className="detail-item">
                <span className="label">Active:</span>
                <span className="value">{model.active ? 'Yes' : 'No'}</span>
              </div>
              <div className="detail-item">
                <span className="label">Featured:</span>
                <span className="value">{model.featured ? 'Yes' : 'No'}</span>
              </div>
              <div className="detail-item">
                <span className="label">IPFS CID:</span>
                <span className="value mono">{model.ipfsCid}</span>
              </div>
            </div>
          </div>

          <div className="detail-section">
            <h4>Performance & Reviews</h4>
            <div className="rating-display">
              <div className="rating-score">
                <span className="score">{(model.averageRating / 100).toFixed(1)}</span>
                <span className="reviews">({model.reviewCount} reviews)</span>
              </div>
              <div className="sales">
                <span>{model.totalSales.toLocaleString()} sales</span>
              </div>
              <div className="quality">
                <span>Quality Score: {model.qualityScore}</span>
              </div>
            </div>
          </div>

          {model.tags.length > 0 && (
            <div className="detail-section">
              <h4>Tags</h4>
              <div className="tags-list">
                {model.tags.map((tag, index) => (
                  <span key={index} className="tag">{tag}</span>
                ))}
              </div>
            </div>
          )}
        </div>

        <div className="modal-actions">
          <div className="price-display">
            {hasDiscount && (
              <span className="original-price">{effectivePrice} wei</span>
            )}
            <span className="price">{effectivePrice} wei</span>
            <span className="price-label">per inference</span>
          </div>
          <button className="btn btn-secondary" onClick={onClose}>
            Close
          </button>
          <button className="btn btn-primary">
            Purchase Access
          </button>
        </div>
      </div>

      <style jsx>{`
        .modal-overlay {
          position: fixed;
          top: 0;
          left: 0;
          right: 0;
          bottom: 0;
          background: rgba(0, 0, 0, 0.5);
          display: flex;
          align-items: center;
          justify-content: center;
          z-index: 1000;
        }

        .modal {
          background: white;
          border-radius: 1rem;
          width: 90%;
          max-width: 700px;
          max-height: 90vh;
          overflow: hidden;
          display: flex;
          flex-direction: column;
        }

        .modal-header {
          display: flex;
          justify-content: space-between;
          align-items: center;
          padding: 1.5rem;
          border-bottom: 1px solid #e5e7eb;
        }

        .modal-header h3 {
          margin: 0;
          font-size: 1.5rem;
          font-weight: 600;
        }

        .close-btn {
          background: none;
          border: none;
          font-size: 1.5rem;
          cursor: pointer;
          color: #6b7280;
          padding: 0.5rem;
        }

        .model-details-content {
          padding: 1.5rem;
          overflow-y: auto;
          flex: 1;
        }

        .detail-section {
          margin-bottom: 2rem;
        }

        .detail-section h4 {
          margin: 0 0 1rem 0;
          font-size: 1.125rem;
          font-weight: 600;
          color: #111827;
        }

        .detail-grid {
          display: grid;
          grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
          gap: 1rem;
        }

        .detail-item {
          display: flex;
          justify-content: space-between;
          padding: 0.75rem 0;
          border-bottom: 1px solid #f3f4f6;
        }

        .label {
          color: #6b7280;
          font-weight: 500;
        }

        .value {
          font-weight: 600;
          text-align: right;
        }

        .mono {
          font-family: monospace;
          font-size: 0.875rem;
        }

        .rating-display {
          display: flex;
          align-items: center;
          justify-content: space-between;
          flex-wrap: wrap;
          gap: 1rem;
          padding: 1rem;
          background: #f9fafb;
          border-radius: 8px;
        }

        .rating-score {
          display: flex;
          align-items: center;
          gap: 0.5rem;
        }

        .score {
          font-size: 1.5rem;
          font-weight: 700;
          color: #111827;
        }

        .reviews {
          color: #6b7280;
          font-size: 0.875rem;
        }

        .sales,
        .quality {
          display: flex;
          align-items: center;
          gap: 0.5rem;
          color: #6b7280;
          font-size: 0.875rem;
        }

        .tags-list {
          display: flex;
          gap: 0.5rem;
          flex-wrap: wrap;
        }

        .tag {
          background: #f3f4f6;
          color: #374151;
          padding: 0.5rem 1rem;
          border-radius: 1rem;
          font-size: 0.875rem;
          font-weight: 500;
        }

        .modal-actions {
          display: flex;
          justify-content: space-between;
          align-items: center;
          padding: 1.5rem;
          border-top: 1px solid #e5e7eb;
          gap: 1rem;
        }

        .price-display {
          display: flex;
          flex-direction: column;
          gap: 0.25rem;
        }

        .original-price {
          font-size: 0.875rem;
          color: #9ca3af;
          text-decoration: line-through;
        }

        .price {
          font-size: 1.25rem;
          font-weight: 700;
          color: #059669;
        }

        .price-label {
          font-size: 0.75rem;
          color: #6b7280;
        }

        .btn {
          display: flex;
          align-items: center;
          gap: 0.5rem;
          padding: 0.75rem 1.5rem;
          border: none;
          border-radius: 0.5rem;
          font-size: 1rem;
          font-weight: 500;
          cursor: pointer;
          transition: all 0.2s;
        }

        .btn-primary {
          background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
          color: white;
        }

        .btn-secondary {
          background: #f3f4f6;
          color: #374151;
        }
      `}</style>
    </div>
  );
};