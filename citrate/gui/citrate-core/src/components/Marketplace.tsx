import React, { useState, useEffect } from 'react';
import {
  ShoppingBag,
  Search,
  Star,
  Download,
  Upload,
  Eye,
  DollarSign,
  Tag,
  User,
  CheckCircle,
  AlertCircle
} from 'lucide-react';
import { SkeletonCard } from './Skeleton';
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
import { useWallet } from '../context/WalletContext';

export const Marketplace: React.FC = () => {
  const wallet = useWallet();
  const [models, setModels] = useState<MarketplaceModel[]>([]);
  const [searchQuery, setSearchQuery] = useState('');
  const [selectedCategory, setSelectedCategory] = useState('all');
  const [sortBy, setSortBy] = useState<'popular' | 'newest' | 'rating' | 'price'>('popular');
  const [viewMode, setViewMode] = useState<'grid' | 'list'>('grid');
  const [selectedModel, setSelectedModel] = useState<MarketplaceModel | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [marketplaceService, setMarketplaceService] = useState<MarketplaceService | null>(null);
  const [useMockData, setUseMockData] = useState(true);

  // Convert CATEGORIES to UI-friendly format with 'all' option
  const categories = [
    'all',
    ...CATEGORIES.map(cat => cat.name.toLowerCase().replace(/\s+/g, '-'))
  ];

  // Initialize marketplace service on mount
  useEffect(() => {
    const initMarketplace = async () => {
      try {
        // Initialize marketplace service
        const service = await initMarketplaceService();
        setMarketplaceService(service);

        // Check if marketplace contract is available
        const available = await isMarketplaceAvailable(service);

        if (!available) {
          console.warn('[Marketplace] Contract not available, using mock data');
          setUseMockData(true);
          setError('Marketplace contract not deployed. Showing example models.');
        } else {
          console.log('[Marketplace] Contract available, loading real data');
          setUseMockData(false);
          setError(null);
        }
      } catch (err: any) {
        console.error('[Marketplace] Initialization failed:', err);
        setUseMockData(true);
        setError('Failed to initialize marketplace. Showing example models.');
      }

      // Load initial data
      await loadModels();
    };

    initMarketplace();
  }, []);

  const loadModels = async () => {
    setLoading(true);
    try {
      if (useMockData || !marketplaceService) {
        // Use mock data for development/demo
        console.log('[Marketplace] Loading mock data');
        const mockModels = getMockMarketplaceModels();
        setModels(mockModels);
      } else {
        // Load real data from contract
        console.log('[Marketplace] Loading models from contract');
        const loadedModels = await loadMarketplaceModels(marketplaceService, {
          featured: sortBy === 'popular',
          topRated: sortBy === 'rating',
          limit: 50
        });

        if (loadedModels.length === 0) {
          console.warn('[Marketplace] No models found, falling back to mock data');
          setModels(getMockMarketplaceModels());
          setUseMockData(true);
        } else {
          setModels(loadedModels);
        }
      }
      setError(null);
    } catch (err: any) {
      console.error('[Marketplace] Failed to load models:', err);
      setError(err.message || 'Failed to load marketplace models');
      // Fallback to mock data on error
      setModels(getMockMarketplaceModels());
    } finally {
      setLoading(false);
    }
  };

  // Reload models when sort order changes
  useEffect(() => {
    if (marketplaceService && !useMockData) {
      loadModels();
    }
  }, [sortBy]);

  const filteredModels = models
    .filter(model =>
      (selectedCategory === 'all' || model.category === selectedCategory) &&
      (searchQuery === '' ||
        model.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
        model.description.toLowerCase().includes(searchQuery.toLowerCase()) ||
        model.tags.some(tag => tag.toLowerCase().includes(searchQuery.toLowerCase()))
      )
    )
    .sort((a, b) => {
      switch (sortBy) {
        case 'newest':
          return new Date(b.lastUpdated).getTime() - new Date(a.lastUpdated).getTime();
        case 'rating':
          return b.rating - a.rating;
        case 'price':
          return a.price - b.price;
        case 'popular':
        default:
          return b.downloads - a.downloads;
      }
    });

  const formatPrice = (price: number, currency: string) => {
    return `${price.toFixed(3)} ${currency}`;
  };

  const handlePurchase = async (model: MarketplaceModel) => {
    if (!marketplaceService || useMockData) {
      alert(`Marketplace demo: Would purchase "${model.name}" for ${model.price} ${model.currency}\n\nIPFS CID: ${model.ipfsCid}\n\nThis feature requires a deployed marketplace contract.`);
      return;
    }

    if (!wallet.selectedAddress) {
      alert('Please select a wallet address first');
      return;
    }

    try {
      console.log(`[Marketplace] Purchasing access to: ${model.name}`);

      // Get password for transaction signing
      const password = prompt('Enter wallet password to purchase model access:');
      if (!password) {
        console.log('[Marketplace] Purchase cancelled');
        return;
      }

      // Purchase access via marketplace contract
      const txHash = await marketplaceService.purchaseAccess(
        {
          modelId: model.id,
          buyer: wallet.selectedAddress
        },
        {
          from: wallet.selectedAddress,
          value: (BigInt(Math.floor(model.price * 1e18))).toString(), // Convert ETH to wei
          password
        }
      );

      console.log('[Marketplace] Purchase transaction sent:', txHash);
      alert(`Purchase successful!\n\nTransaction: ${txHash}\n\nYou can now download the model from IPFS: ${model.ipfsCid}`);

      // TODO: Implement actual IPFS download
      // await downloadFromIPFS(model.ipfsCid);
    } catch (error: any) {
      console.error('[Marketplace] Purchase failed:', error);
      alert(`Purchase failed: ${error.message || error}`);
    }
  };

  return (
    <div className="marketplace">
      {error && (
        <div className="error-banner">
          <AlertCircle size={20} />
          <span>{error}</span>
        </div>
      )}

      <div className="marketplace-header">
        <h2>AI Model Marketplace</h2>
        <button className="btn btn-primary">
          <Upload size={16} />
          Publish Model
        </button>
      </div>

      <div className="marketplace-controls">
        <div className="search-bar">
          <Search size={20} />
          <input
            type="text"
            placeholder="Search models, categories, or tags..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
          />
        </div>

        <div className="filters">
          <select
            value={selectedCategory}
            onChange={(e) => setSelectedCategory(e.target.value)}
          >
            {categories.map(cat => (
              <option key={cat} value={cat}>
                {cat.replace('-', ' ').replace(/\b\w/g, l => l.toUpperCase())}
              </option>
            ))}
          </select>

          <select
            value={sortBy}
            onChange={(e) => setSortBy(e.target.value as any)}
          >
            <option value="popular">Most Popular</option>
            <option value="newest">Newest</option>
            <option value="rating">Highest Rated</option>
            <option value="price">Price: Low to High</option>
          </select>

          <div className="view-mode">
            <button
              className={`btn-view ${viewMode === 'grid' ? 'active' : ''}`}
              onClick={() => setViewMode('grid')}
            >
              Grid
            </button>
            <button
              className={`btn-view ${viewMode === 'list' ? 'active' : ''}`}
              onClick={() => setViewMode('list')}
            >
              List
            </button>
          </div>
        </div>
      </div>

      <div className={`models-container ${viewMode}`}>
        {loading ? (
          <>
            <SkeletonCard height="380px" />
            <SkeletonCard height="380px" />
            <SkeletonCard height="380px" />
            <SkeletonCard height="380px" />
            <SkeletonCard height="380px" />
            <SkeletonCard height="380px" />
          </>
        ) : (
          <>
            {filteredModels.map(model => (
              <div
                key={model.id}
                className={`model-card ${model.featured ? 'featured' : ''}`}
                onClick={() => setSelectedModel(model)}
              >
                {model.featured && (
                  <div className="featured-badge">
                    <Star size={14} />
                    Featured
                  </div>
                )}

                <div className="model-header">
                  <div className="model-type-badge">{model.modelType}</div>
                  <div className="model-price">
                    <DollarSign size={14} />
                    {formatPrice(model.price, model.currency)}
                  </div>
                </div>

                <h3>{model.name}</h3>
                <p className="model-description">{model.description}</p>

                <div className="model-stats">
                  <div className="stat">
                    <Star size={14} />
                    <span>{model.rating} ({model.reviews})</span>
                  </div>
                  <div className="stat">
                    <Download size={14} />
                    <span>{model.downloads.toLocaleString()}</span>
                  </div>
                  <div className="stat">
                    <Tag size={14} />
                    <span>{model.size}</span>
                  </div>
                </div>

                <div className="model-author">
                  <User size={14} />
                  <span>{model.author}</span>
                  {model.authorVerified && <CheckCircle size={14} className="verified" />}
                </div>

                <div className="model-tags">
                  {model.tags.slice(0, 3).map(tag => (
                    <span key={tag} className="tag">{tag}</span>
                  ))}
                  {model.tags.length > 3 && <span className="tag-more">+{model.tags.length - 3}</span>}
                </div>

                <div className="model-actions">
                  <button
                    className="btn btn-primary"
                    onClick={(e) => {
                      e.stopPropagation();
                      handlePurchase(model);
                    }}
                  >
                    <ShoppingBag size={16} />
                    Purchase
                  </button>
                  <button className="btn btn-secondary">
                    <Eye size={16} />
                    Preview
                  </button>
                </div>
              </div>
            ))}

            {filteredModels.length === 0 && !loading && (
              <div className="empty-state">
                <ShoppingBag size={48} />
                <p>No models found</p>
                <p className="text-muted">Try adjusting your search or filters</p>
              </div>
            )}
          </>
        )}
      </div>

      {selectedModel && (
        <ModelDetailsModal
          model={selectedModel}
          onClose={() => setSelectedModel(null)}
          onPurchase={handlePurchase}
        />
      )}

      <style jsx>{`
        .marketplace {
          padding: 2rem;
          background: #f9fafb;
          min-height: 100vh;
        }

        .error-banner {
          background: #fef3c7;
          border: 1px solid #fbbf24;
          border-radius: 0.75rem;
          padding: 1rem 1.5rem;
          margin-bottom: 1.5rem;
          display: flex;
          align-items: center;
          gap: 0.75rem;
          color: #92400e;
          font-size: 0.9375rem;
        }

        .error-banner svg {
          flex-shrink: 0;
          color: #f59e0b;
        }

        .marketplace-header {
          display: flex;
          justify-content: space-between;
          align-items: center;
          margin-bottom: 2rem;
        }

        .marketplace-header h2 {
          margin: 0;
          font-size: 1.75rem;
          font-weight: 600;
          color: #111827;
        }

        .marketplace-controls {
          background: white;
          padding: 1.5rem;
          border-radius: 1rem;
          box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
          margin-bottom: 2rem;
        }

        .search-bar {
          display: flex;
          align-items: center;
          gap: 0.75rem;
          margin-bottom: 1rem;
          position: relative;
        }

        .search-bar input {
          flex: 1;
          padding: 0.75rem 1rem;
          border: 1px solid #e5e7eb;
          border-radius: 0.5rem;
          font-size: 1rem;
          padding-left: 3rem;
        }

        .search-bar svg {
          position: absolute;
          left: 0.75rem;
          color: #9ca3af;
        }

        .filters {
          display: flex;
          align-items: center;
          gap: 1rem;
          flex-wrap: wrap;
        }

        .filters select {
          padding: 0.5rem 1rem;
          border: 1px solid #e5e7eb;
          border-radius: 0.5rem;
          background: white;
          font-size: 0.875rem;
        }

        .view-mode {
          display: flex;
          border: 1px solid #e5e7eb;
          border-radius: 0.5rem;
          overflow: hidden;
        }

        .btn-view {
          padding: 0.5rem 1rem;
          border: none;
          background: white;
          font-size: 0.875rem;
          cursor: pointer;
          transition: all 0.2s;
        }

        .btn-view.active {
          background: #667eea;
          color: white;
        }

        .models-container {
          display: grid;
          gap: 1.5rem;
        }

        .models-container.grid {
          grid-template-columns: repeat(auto-fill, minmax(350px, 1fr));
        }

        .models-container.list {
          grid-template-columns: 1fr;
        }

        .model-card {
          background: white;
          border-radius: 1rem;
          padding: 1.5rem;
          box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
          cursor: pointer;
          transition: all 0.2s;
          position: relative;
          border: 1px solid #e5e7eb;
        }

        .model-card:hover {
          transform: translateY(-2px);
          box-shadow: 0 8px 16px rgba(0, 0, 0, 0.15);
        }

        .model-card.featured {
          border: 2px solid #fbbf24;
          box-shadow: 0 4px 8px rgba(251, 191, 36, 0.2);
        }

        .featured-badge {
          position: absolute;
          top: -1px;
          right: -1px;
          background: linear-gradient(135deg, #fbbf24 0%, #f59e0b 100%);
          color: white;
          padding: 0.25rem 0.75rem;
          border-radius: 0 0.75rem 0 0.75rem;
          font-size: 0.75rem;
          font-weight: 600;
          display: flex;
          align-items: center;
          gap: 0.25rem;
        }

        .model-header {
          display: flex;
          justify-content: space-between;
          align-items: flex-start;
          margin-bottom: 1rem;
        }

        .model-type-badge {
          background: #e0e7ff;
          color: #4338ca;
          padding: 0.25rem 0.75rem;
          border-radius: 1rem;
          font-size: 0.75rem;
          font-weight: 500;
          text-transform: capitalize;
        }

        .model-price {
          display: flex;
          align-items: center;
          gap: 0.25rem;
          color: #059669;
          font-weight: 600;
          font-size: 0.875rem;
        }

        .model-card h3 {
          margin: 0 0 0.5rem 0;
          font-size: 1.25rem;
          font-weight: 600;
          color: #111827;
        }

        .model-description {
          color: #6b7280;
          margin: 0 0 1rem 0;
          line-height: 1.5;
          font-size: 0.9375rem;
        }

        .model-stats {
          display: flex;
          gap: 1rem;
          margin-bottom: 1rem;
          flex-wrap: wrap;
        }

        .stat {
          display: flex;
          align-items: center;
          gap: 0.5rem;
          color: #6b7280;
          font-size: 0.875rem;
        }

        .model-author {
          display: flex;
          align-items: center;
          gap: 0.5rem;
          color: #6b7280;
          font-size: 0.875rem;
          margin-bottom: 1rem;
        }

        .verified {
          color: #059669;
        }

        .model-tags {
          display: flex;
          gap: 0.5rem;
          margin-bottom: 1rem;
          flex-wrap: wrap;
        }

        .tag {
          background: #f3f4f6;
          color: #374151;
          padding: 0.25rem 0.75rem;
          border-radius: 1rem;
          font-size: 0.75rem;
          font-weight: 500;
        }

        .tag-more {
          background: #e5e7eb;
          color: #6b7280;
          padding: 0.25rem 0.75rem;
          border-radius: 1rem;
          font-size: 0.75rem;
          font-weight: 500;
        }

        .model-actions {
          display: flex;
          gap: 0.75rem;
        }

        .btn {
          display: flex;
          align-items: center;
          gap: 0.5rem;
          padding: 0.5rem 1rem;
          border: none;
          border-radius: 0.5rem;
          font-size: 0.875rem;
          font-weight: 500;
          cursor: pointer;
          transition: all 0.2s;
          text-decoration: none;
        }

        .btn-primary {
          background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
          color: white;
        }

        .btn-primary:hover {
          transform: translateY(-1px);
          box-shadow: 0 4px 8px rgba(102, 126, 234, 0.3);
        }

        .btn-secondary {
          background: #f3f4f6;
          color: #374151;
          border: 1px solid #e5e7eb;
        }

        .btn-secondary:hover {
          background: #e5e7eb;
        }

        .loading, .empty-state {
          grid-column: 1 / -1;
          text-align: center;
          padding: 3rem;
          color: #6b7280;
        }

        .empty-state svg {
          margin-bottom: 1rem;
          color: #9ca3af;
        }

        .text-muted {
          color: #9ca3af;
          margin-top: 0.5rem;
        }
      `}</style>
    </div>
  );
};

// Model Details Modal Component
const ModelDetailsModal: React.FC<{
  model: MarketplaceModel;
  onClose: () => void;
  onPurchase: (model: MarketplaceModel) => void;
}> = ({ model, onClose, onPurchase }) => {
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
                <span className="label">Version:</span>
                <span className="value">{model.version}</span>
              </div>
              <div className="detail-item">
                <span className="label">Architecture:</span>
                <span className="value">{model.architecture}</span>
              </div>
              <div className="detail-item">
                <span className="label">Size:</span>
                <span className="value">{model.size}</span>
              </div>
              <div className="detail-item">
                <span className="label">License:</span>
                <span className="value">{model.license}</span>
              </div>
              <div className="detail-item">
                <span className="label">Last Updated:</span>
                <span className="value">{new Date(model.lastUpdated).toLocaleDateString()}</span>
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
                <Star size={24} className="filled" />
                <span className="score">{model.rating}</span>
                <span className="reviews">({model.reviews} reviews)</span>
              </div>
              <div className="downloads">
                <Download size={16} />
                <span>{model.downloads.toLocaleString()} downloads</span>
              </div>
            </div>
          </div>

          <div className="detail-section">
            <h4>Tags</h4>
            <div className="tags-list">
              {model.tags.map(tag => (
                <span key={tag} className="tag">{tag}</span>
              ))}
            </div>
          </div>
        </div>

        <div className="modal-actions">
          <div className="price-display">
            <DollarSign size={20} />
            <span className="price">{model.price.toFixed(3)} {model.currency}</span>
          </div>
          <button className="btn btn-secondary" onClick={onClose}>
            Close
          </button>
          <button
            className="btn btn-primary"
            onClick={() => onPurchase(model)}
          >
            <ShoppingBag size={16} />
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
        }

        .rating-score {
          display: flex;
          align-items: center;
          gap: 0.5rem;
        }

        .filled {
          color: #fbbf24;
        }

        .score {
          font-size: 1.25rem;
          font-weight: 600;
        }

        .reviews {
          color: #6b7280;
        }

        .downloads {
          display: flex;
          align-items: center;
          gap: 0.5rem;
          color: #6b7280;
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
          align-items: center;
          gap: 0.5rem;
          color: #059669;
          font-weight: 600;
          font-size: 1.125rem;
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