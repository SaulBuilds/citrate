'use client';

import { useState, useEffect } from 'react';
import { useQuery } from '@tanstack/react-query';
import axios from 'axios';
import Link from 'next/link';
import {
  ShoppingBag,
  Search,
  Filter,
  Star,
  Download,
  DollarSign,
  Tag,
  User,
  Calendar,
  Eye,
  TrendingUp,
  Brain,
  Image,
  Code,
  Mic
} from 'lucide-react';
import { formatDistanceToNow } from 'date-fns';

interface MarketplaceModel {
  id: string;
  name: string;
  description: string;
  category: string;
  price: number;
  currency: string;
  rating: number;
  downloads: number;
  author: string;
  authorVerified: boolean;
  featured: boolean;
  modelType: 'text' | 'image' | 'audio' | 'multimodal';
  lastUpdated: string;
  tags: string[];
  ipfsCid: string;
}

const categories = [
  { id: 'all', name: 'All Models', icon: Brain },
  { id: 'text', name: 'Text Generation', icon: Code },
  { id: 'image', name: 'Image Generation', icon: Image },
  { id: 'audio', name: 'Audio Processing', icon: Mic },
  { id: 'multimodal', name: 'Multimodal', icon: Brain }
];

export default function MarketplacePage() {
  const [searchQuery, setSearchQuery] = useState('');
  const [selectedCategory, setSelectedCategory] = useState('all');
  const [sortBy, setSortBy] = useState<'popular' | 'newest' | 'rating' | 'price'>('popular');

  const { data: models, isLoading } = useQuery({
    queryKey: ['marketplace-models', selectedCategory, sortBy],
    queryFn: async () => {
      const response = await axios.get('/api/marketplace/models', {
        params: {
          category: selectedCategory === 'all' ? undefined : selectedCategory,
          sort: sortBy
        }
      });
      return response.data;
    },
    refetchInterval: 30000
  });

  const { data: stats } = useQuery({
    queryKey: ['marketplace-stats'],
    queryFn: async () => {
      const response = await axios.get('/api/marketplace/stats');
      return response.data;
    },
    refetchInterval: 60000
  });

  const filteredModels = models?.filter((model: MarketplaceModel) =>
    model.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
    model.description.toLowerCase().includes(searchQuery.toLowerCase()) ||
    model.tags.some((tag: string) => tag.toLowerCase().includes(searchQuery.toLowerCase()))
  ) || [];

  const formatPrice = (price: number, currency: string) => {
    return `${price.toFixed(3)} ${currency}`;
  };

  return (
    <div className="max-w-7xl mx-auto py-8 px-4">
      {/* Header */}
      <div className="bg-gradient-to-r from-purple-600 to-blue-600 rounded-2xl p-8 text-white mb-8">
        <h1 className="text-4xl font-bold mb-4 flex items-center gap-3">
          <ShoppingBag className="w-10 h-10" />
          AI Model Marketplace
        </h1>
        <p className="text-lg opacity-90 mb-6">
          Discover, deploy, and monetize AI models on the Lattice network
        </p>

        {/* Quick Stats */}
        <div className="grid grid-cols-2 md:grid-cols-4 gap-4 mt-6">
          <div className="bg-white/10 rounded-lg p-4">
            <p className="text-2xl font-bold">{stats?.totalModels || 0}</p>
            <p className="text-sm opacity-80">Total Models</p>
          </div>
          <div className="bg-white/10 rounded-lg p-4">
            <p className="text-2xl font-bold">{stats?.totalDownloads || 0}</p>
            <p className="text-sm opacity-80">Downloads</p>
          </div>
          <div className="bg-white/10 rounded-lg p-4">
            <p className="text-2xl font-bold">{stats?.activeAuthors || 0}</p>
            <p className="text-sm opacity-80">Developers</p>
          </div>
          <div className="bg-white/10 rounded-lg p-4">
            <p className="text-2xl font-bold">{stats?.totalVolume || '0'} LAT</p>
            <p className="text-sm opacity-80">Total Volume</p>
          </div>
        </div>
      </div>

      {/* Search and Filters */}
      <div className="bg-white dark:bg-gray-800 rounded-xl shadow-lg p-6 mb-8">
        <div className="flex flex-col lg:flex-row gap-4 items-start lg:items-center justify-between">
          {/* Search */}
          <div className="relative flex-1 max-w-md">
            <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 text-gray-400 w-5 h-5" />
            <input
              type="text"
              placeholder="Search models, tags, or authors..."
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              className="w-full pl-10 pr-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-white"
            />
          </div>

          {/* Sort */}
          <select
            value={sortBy}
            onChange={(e) => setSortBy(e.target.value as any)}
            className="px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-white"
          >
            <option value="popular">Most Popular</option>
            <option value="newest">Newest</option>
            <option value="rating">Highest Rated</option>
            <option value="price">Price: Low to High</option>
          </select>
        </div>

        {/* Categories */}
        <div className="flex flex-wrap gap-2 mt-4">
          {categories.map((category) => {
            const IconComponent = category.icon;
            return (
              <button
                key={category.id}
                onClick={() => setSelectedCategory(category.id)}
                className={`flex items-center gap-2 px-4 py-2 rounded-lg transition-colors ${
                  selectedCategory === category.id
                    ? 'bg-blue-500 text-white'
                    : 'bg-gray-100 dark:bg-gray-700 text-gray-700 dark:text-gray-300 hover:bg-gray-200 dark:hover:bg-gray-600'
                }`}
              >
                <IconComponent className="w-4 h-4" />
                {category.name}
              </button>
            );
          })}
        </div>
      </div>

      {/* Models Grid */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
        {isLoading ? (
          [...Array(6)].map((_, i) => (
            <div key={i} className="animate-pulse">
              <div className="bg-gray-200 dark:bg-gray-700 rounded-xl h-80"></div>
            </div>
          ))
        ) : (
          filteredModels.map((model: MarketplaceModel) => (
            <ModelCard key={model.id} model={model} />
          ))
        )}
      </div>

      {filteredModels.length === 0 && !isLoading && (
        <div className="text-center py-12">
          <ShoppingBag className="w-16 h-16 text-gray-400 mx-auto mb-4" />
          <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-2">
            No models found
          </h3>
          <p className="text-gray-600 dark:text-gray-400">
            Try adjusting your search criteria or browse different categories
          </p>
        </div>
      )}
    </div>
  );
}

function ModelCard({ model }: { model: MarketplaceModel }) {
  const getModelTypeIcon = (type: string) => {
    switch (type) {
      case 'text': return Code;
      case 'image': return Image;
      case 'audio': return Mic;
      default: return Brain;
    }
  };

  const IconComponent = getModelTypeIcon(model.modelType);

  return (
    <div className="bg-white dark:bg-gray-800 rounded-xl shadow-lg overflow-hidden hover:shadow-xl transition-shadow duration-300">
      {model.featured && (
        <div className="bg-gradient-to-r from-yellow-400 to-orange-500 text-white text-center py-1">
          <Star className="w-4 h-4 inline mr-1" />
          Featured
        </div>
      )}

      <div className="p-6">
        {/* Header */}
        <div className="flex items-start justify-between mb-4">
          <div className="flex items-center gap-3">
            <div className="p-2 bg-blue-100 dark:bg-blue-900 rounded-lg">
              <IconComponent className="w-6 h-6 text-blue-600 dark:text-blue-400" />
            </div>
            <div>
              <h3 className="font-semibold text-gray-900 dark:text-white text-lg">
                {model.name}
              </h3>
              <div className="flex items-center gap-2 mt-1">
                <span className="text-xs px-2 py-1 bg-gray-100 dark:bg-gray-700 text-gray-600 dark:text-gray-400 rounded-full">
                  {model.modelType}
                </span>
                <span className="text-xs px-2 py-1 bg-green-100 dark:bg-green-900 text-green-600 dark:text-green-400 rounded-full">
                  {model.category}
                </span>
              </div>
            </div>
          </div>

          <div className="text-right">
            <div className="flex items-center gap-1 text-green-600 dark:text-green-400 font-semibold">
              <DollarSign className="w-4 h-4" />
              {model.price.toFixed(3)} {model.currency}
            </div>
          </div>
        </div>

        {/* Description */}
        <p className="text-gray-600 dark:text-gray-400 text-sm mb-4 line-clamp-2">
          {model.description}
        </p>

        {/* Stats */}
        <div className="flex items-center gap-4 mb-4 text-sm text-gray-500 dark:text-gray-400">
          <div className="flex items-center gap-1">
            <Star className="w-4 h-4 text-yellow-500" />
            {model.rating}
          </div>
          <div className="flex items-center gap-1">
            <Download className="w-4 h-4" />
            {model.downloads.toLocaleString()}
          </div>
          <div className="flex items-center gap-1">
            <Calendar className="w-4 h-4" />
            {formatDistanceToNow(new Date(model.lastUpdated), { addSuffix: true })}
          </div>
        </div>

        {/* Author */}
        <div className="flex items-center gap-2 mb-4">
          <User className="w-4 h-4 text-gray-400" />
          <Link
            href={`/profile/${model.author}`}
            className="text-sm text-blue-500 hover:text-blue-600 font-medium"
          >
            {model.author.slice(0, 8)}...{model.author.slice(-6)}
          </Link>
          {model.authorVerified && (
            <div className="w-4 h-4 bg-blue-500 rounded-full flex items-center justify-center">
              <span className="text-white text-xs">✓</span>
            </div>
          )}
        </div>

        {/* Tags */}
        <div className="flex flex-wrap gap-1 mb-4">
          {model.tags.slice(0, 3).map((tag) => (
            <span
              key={tag}
              className="text-xs px-2 py-1 bg-gray-100 dark:bg-gray-700 text-gray-600 dark:text-gray-400 rounded"
            >
              {tag}
            </span>
          ))}
          {model.tags.length > 3 && (
            <span className="text-xs px-2 py-1 bg-gray-100 dark:bg-gray-700 text-gray-600 dark:text-gray-400 rounded">
              +{model.tags.length - 3}
            </span>
          )}
        </div>

        {/* Actions */}
        <div className="flex gap-2">
          <Link
            href={`/model/${model.id}`}
            className="flex-1 bg-blue-500 hover:bg-blue-600 text-white px-4 py-2 rounded-lg text-center transition-colors flex items-center justify-center gap-2"
          >
            <Eye className="w-4 h-4" />
            View Details
          </Link>
          <button className="bg-green-500 hover:bg-green-600 text-white px-4 py-2 rounded-lg transition-colors flex items-center justify-center gap-2">
            <Download className="w-4 h-4" />
            Deploy
          </button>
        </div>

        {/* IPFS CID */}
        <div className="mt-3 pt-3 border-t border-gray-200 dark:border-gray-700">
          <div className="flex items-center gap-2 text-xs text-gray-500 dark:text-gray-400">
            <Tag className="w-3 h-3" />
            <span className="font-mono">{model.ipfsCid.slice(0, 20)}...</span>
          </div>
        </div>
      </div>
    </div>
  );
}