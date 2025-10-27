'use client';

import { useState, useEffect } from 'react';
import { useQuery } from '@tanstack/react-query';
import axios from 'axios';
import Link from 'next/link';
import {
  Database,
  File,
  FolderOpen,
  Copy,
  Download,
  Globe,
  Pin,
  Unlink,
  Search,
  Filter,
  Activity,
  HardDrive,
  Users,
  Zap,
  Clock,
  Eye,
  Hash
} from 'lucide-react';
import { formatDistanceToNow } from 'date-fns';

interface IPFSFile {
  cid: string;
  name?: string;
  size: number;
  type: 'file' | 'directory';
  pinned: boolean;
  replicas: number;
  lastAccessed: string;
  addedBy?: string;
  contentType?: string;
}

interface IPFSStats {
  totalFiles: number;
  totalSize: number;
  pinnedFiles: number;
  recentActivity: number;
  topProviders: Array<{
    peerId: string;
    filesHosted: number;
    reputation: number;
  }>;
}

export default function IPFSPage() {
  const [searchQuery, setSearchQuery] = useState('');
  const [filterType, setFilterType] = useState<'all' | 'pinned' | 'files' | 'directories'>('all');
  const [selectedCid, setSelectedCid] = useState<string | null>(null);

  const { data: files, isLoading } = useQuery({
    queryKey: ['ipfs-files', filterType],
    queryFn: async () => {
      const response = await axios.get('/api/ipfs/files', {
        params: { type: filterType === 'all' ? undefined : filterType }
      });
      return response.data;
    },
    refetchInterval: 30000
  });

  const { data: stats } = useQuery({
    queryKey: ['ipfs-stats'],
    queryFn: async () => {
      const response = await axios.get('/api/ipfs/stats');
      return response.data;
    },
    refetchInterval: 60000
  });

  const { data: fileDetails } = useQuery({
    queryKey: ['ipfs-file', selectedCid],
    queryFn: async () => {
      if (!selectedCid) return null;
      const response = await axios.get(`/api/ipfs/file/${selectedCid}`);
      return response.data;
    },
    enabled: !!selectedCid
  });

  const filteredFiles = files?.filter((file: IPFSFile) => {
    const matchesSearch = file.cid.toLowerCase().includes(searchQuery.toLowerCase()) ||
                         (file.name && file.name.toLowerCase().includes(searchQuery.toLowerCase()));

    const matchesFilter = filterType === 'all' ||
                         (filterType === 'pinned' && file.pinned) ||
                         (filterType === 'files' && file.type === 'file') ||
                         (filterType === 'directories' && file.type === 'directory');

    return matchesSearch && matchesFilter;
  }) || [];

  const formatSize = (bytes: number) => {
    const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
    if (bytes === 0) return '0 B';
    const i = Math.floor(Math.log(bytes) / Math.log(1024));
    return Math.round(bytes / Math.pow(1024, i) * 100) / 100 + ' ' + sizes[i];
  };

  const copyToClipboard = (text: string) => {
    navigator.clipboard.writeText(text);
  };

  return (
    <div className="max-w-7xl mx-auto py-8 px-4">
      {/* Header */}
      <div className="bg-gradient-to-r from-indigo-600 to-purple-600 rounded-2xl p-8 text-white mb-8">
        <h1 className="text-4xl font-bold mb-4 flex items-center gap-3">
          <Database className="w-10 h-10" />
          IPFS Storage Network
        </h1>
        <p className="text-lg opacity-90 mb-6">
          Explore the distributed storage layer powering Citrate AI models and data
        </p>

        {/* Network Stats */}
        <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
          <div className="bg-white/10 rounded-lg p-4">
            <div className="flex items-center gap-2 mb-2">
              <File className="w-5 h-5" />
              <span className="text-sm opacity-80">Total Files</span>
            </div>
            <p className="text-2xl font-bold">{stats?.totalFiles?.toLocaleString() || 0}</p>
          </div>
          <div className="bg-white/10 rounded-lg p-4">
            <div className="flex items-center gap-2 mb-2">
              <HardDrive className="w-5 h-5" />
              <span className="text-sm opacity-80">Storage Used</span>
            </div>
            <p className="text-2xl font-bold">{formatSize(stats?.totalSize || 0)}</p>
          </div>
          <div className="bg-white/10 rounded-lg p-4">
            <div className="flex items-center gap-2 mb-2">
              <Pin className="w-5 h-5" />
              <span className="text-sm opacity-80">Pinned</span>
            </div>
            <p className="text-2xl font-bold">{stats?.pinnedFiles?.toLocaleString() || 0}</p>
          </div>
          <div className="bg-white/10 rounded-lg p-4">
            <div className="flex items-center gap-2 mb-2">
              <Activity className="w-5 h-5" />
              <span className="text-sm opacity-80">24h Activity</span>
            </div>
            <p className="text-2xl font-bold">{stats?.recentActivity || 0}</p>
          </div>
        </div>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-4 gap-8">
        {/* Main Content */}
        <div className="lg:col-span-3">
          {/* Search and Filters */}
          <div className="bg-white dark:bg-gray-800 rounded-xl shadow-lg p-6 mb-6">
            <div className="flex flex-col sm:flex-row gap-4 items-start sm:items-center justify-between">
              {/* Search */}
              <div className="relative flex-1 max-w-md">
                <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 text-gray-400 w-5 h-5" />
                <input
                  type="text"
                  placeholder="Search by CID or filename..."
                  value={searchQuery}
                  onChange={(e) => setSearchQuery(e.target.value)}
                  className="w-full pl-10 pr-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-white"
                />
              </div>

              {/* Filter */}
              <div className="flex gap-2">
                {['all', 'pinned', 'files', 'directories'].map((filter) => (
                  <button
                    key={filter}
                    onClick={() => setFilterType(filter as any)}
                    className={`px-4 py-2 rounded-lg capitalize transition-colors ${
                      filterType === filter
                        ? 'bg-blue-500 text-white'
                        : 'bg-gray-100 dark:bg-gray-700 text-gray-700 dark:text-gray-300 hover:bg-gray-200 dark:hover:bg-gray-600'
                    }`}
                  >
                    {filter}
                  </button>
                ))}
              </div>
            </div>
          </div>

          {/* Files List */}
          <div className="bg-white dark:bg-gray-800 rounded-xl shadow-lg overflow-hidden">
            <div className="px-6 py-4 border-b border-gray-200 dark:border-gray-700">
              <h2 className="text-xl font-semibold text-gray-900 dark:text-white">
                IPFS Files ({filteredFiles.length})
              </h2>
            </div>

            <div className="divide-y divide-gray-200 dark:divide-gray-700">
              {isLoading ? (
                [...Array(5)].map((_, i) => (
                  <div key={i} className="p-6 animate-pulse">
                    <div className="flex items-center gap-4">
                      <div className="w-8 h-8 bg-gray-200 dark:bg-gray-700 rounded"></div>
                      <div className="flex-1">
                        <div className="h-4 bg-gray-200 dark:bg-gray-700 rounded mb-2"></div>
                        <div className="h-3 bg-gray-200 dark:bg-gray-700 rounded w-1/2"></div>
                      </div>
                    </div>
                  </div>
                ))
              ) : (
                filteredFiles.map((file: IPFSFile) => (
                  <FileItem
                    key={file.cid}
                    file={file}
                    onSelect={() => setSelectedCid(file.cid)}
                    onCopy={copyToClipboard}
                  />
                ))
              )}
            </div>

            {filteredFiles.length === 0 && !isLoading && (
              <div className="text-center py-12">
                <Database className="w-16 h-16 text-gray-400 mx-auto mb-4" />
                <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-2">
                  No files found
                </h3>
                <p className="text-gray-600 dark:text-gray-400">
                  Try adjusting your search criteria or filters
                </p>
              </div>
            )}
          </div>
        </div>

        {/* Sidebar */}
        <div className="space-y-6">
          {/* Top Providers */}
          <div className="bg-white dark:bg-gray-800 rounded-xl shadow-lg p-6">
            <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-4 flex items-center gap-2">
              <Users className="w-5 h-5" />
              Top Providers
            </h3>
            <div className="space-y-3">
              {stats?.topProviders?.map((provider, index) => (
                <div key={provider.peerId} className="flex items-center justify-between">
                  <div className="flex items-center gap-3">
                    <span className="text-sm font-medium text-gray-500 dark:text-gray-400">
                      #{index + 1}
                    </span>
                    <div>
                      <Link
                        href={`/peer/${provider.peerId}`}
                        className="font-mono text-sm text-blue-500 hover:text-blue-600"
                      >
                        {provider.peerId.slice(0, 8)}...
                      </Link>
                      <p className="text-xs text-gray-500 dark:text-gray-400">
                        Rep: {provider.reputation}
                      </p>
                    </div>
                  </div>
                  <span className="text-sm font-medium text-gray-900 dark:text-white">
                    {provider.filesHosted}
                  </span>
                </div>
              )) || [...Array(3)].map((_, i) => (
                <div key={i} className="animate-pulse">
                  <div className="h-12 bg-gray-100 dark:bg-gray-700 rounded"></div>
                </div>
              ))}
            </div>
          </div>

          {/* Quick Actions */}
          <div className="bg-white dark:bg-gray-800 rounded-xl shadow-lg p-6">
            <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-4 flex items-center gap-2">
              <Zap className="w-5 h-5" />
              Quick Actions
            </h3>
            <div className="space-y-3">
              <button className="w-full bg-blue-500 hover:bg-blue-600 text-white px-4 py-2 rounded-lg transition-colors flex items-center gap-2">
                <Download className="w-4 h-4" />
                Add to IPFS
              </button>
              <button className="w-full bg-gray-100 dark:bg-gray-700 hover:bg-gray-200 dark:hover:bg-gray-600 text-gray-900 dark:text-white px-4 py-2 rounded-lg transition-colors flex items-center gap-2">
                <Pin className="w-4 h-4" />
                Pin Content
              </button>
              <button className="w-full bg-gray-100 dark:bg-gray-700 hover:bg-gray-200 dark:hover:bg-gray-600 text-gray-900 dark:text-white px-4 py-2 rounded-lg transition-colors flex items-center gap-2">
                <Globe className="w-4 h-4" />
                Gateway Access
              </button>
            </div>
          </div>
        </div>
      </div>

      {/* File Details Modal */}
      {selectedCid && fileDetails && (
        <FileDetailsModal
          file={fileDetails}
          onClose={() => setSelectedCid(null)}
          onCopy={copyToClipboard}
        />
      )}
    </div>
  );
}

function FileItem({
  file,
  onSelect,
  onCopy
}: {
  file: IPFSFile;
  onSelect: () => void;
  onCopy: (text: string) => void;
}) {
  const formatSize = (bytes: number) => {
    const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
    if (bytes === 0) return '0 B';
    const i = Math.floor(Math.log(bytes) / Math.log(1024));
    return Math.round(bytes / Math.pow(1024, i) * 100) / 100 + ' ' + sizes[i];
  };

  return (
    <div className="p-6 hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors cursor-pointer" onClick={onSelect}>
      <div className="flex items-center gap-4">
        <div className="p-2 bg-gray-100 dark:bg-gray-700 rounded-lg">
          {file.type === 'directory' ? (
            <FolderOpen className="w-6 h-6 text-blue-500" />
          ) : (
            <File className="w-6 h-6 text-gray-500" />
          )}
        </div>

        <div className="flex-1 min-w-0">
          <div className="flex items-start justify-between">
            <div className="flex-1 min-w-0">
              <h3 className="font-medium text-gray-900 dark:text-white truncate">
                {file.name || 'Unnamed'}
              </h3>
              <div className="flex items-center gap-4 mt-1">
                <div className="flex items-center gap-1 font-mono text-sm text-gray-500 dark:text-gray-400">
                  <Hash className="w-3 h-3" />
                  {file.cid.slice(0, 20)}...
                </div>
                <span className="text-sm text-gray-500 dark:text-gray-400">
                  {formatSize(file.size)}
                </span>
                <div className="flex items-center gap-1">
                  <Clock className="w-3 h-3 text-gray-400" />
                  <span className="text-sm text-gray-500 dark:text-gray-400">
                    {formatDistanceToNow(new Date(file.lastAccessed), { addSuffix: true })}
                  </span>
                </div>
              </div>
            </div>

            <div className="flex items-center gap-2 ml-4">
              {file.pinned && (
                <div className="flex items-center gap-1 text-green-500">
                  <Pin className="w-4 h-4" />
                  <span className="text-xs">Pinned</span>
                </div>
              )}
              <div className="flex items-center gap-1 text-gray-500 dark:text-gray-400">
                <Users className="w-4 h-4" />
                <span className="text-sm">{file.replicas}</span>
              </div>
            </div>
          </div>

          {file.contentType && (
            <div className="flex items-center gap-2 mt-2">
              <span className="text-xs px-2 py-1 bg-blue-100 dark:bg-blue-900 text-blue-600 dark:text-blue-400 rounded">
                {file.contentType}
              </span>
            </div>
          )}
        </div>

        <div className="flex items-center gap-2 opacity-0 group-hover:opacity-100 transition-opacity">
          <button
            onClick={(e) => {
              e.stopPropagation();
              onCopy(file.cid);
            }}
            className="p-2 text-gray-400 hover:text-gray-600 dark:hover:text-gray-300 transition-colors"
            title="Copy CID"
          >
            <Copy className="w-4 h-4" />
          </button>
          <button
            onClick={(e) => {
              e.stopPropagation();
              window.open(`https://ipfs.io/ipfs/${file.cid}`, '_blank');
            }}
            className="p-2 text-gray-400 hover:text-gray-600 dark:hover:text-gray-300 transition-colors"
            title="Open in gateway"
          >
            <Globe className="w-4 h-4" />
          </button>
        </div>
      </div>
    </div>
  );
}

function FileDetailsModal({
  file,
  onClose,
  onCopy
}: {
  file: any;
  onClose: () => void;
  onCopy: (text: string) => void;
}) {
  return (
    <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50 p-4">
      <div className="bg-white dark:bg-gray-800 rounded-xl max-w-2xl w-full max-h-[90vh] overflow-hidden">
        <div className="p-6 border-b border-gray-200 dark:border-gray-700">
          <div className="flex items-center justify-between">
            <h2 className="text-xl font-semibold text-gray-900 dark:text-white">
              File Details
            </h2>
            <button
              onClick={onClose}
              className="text-gray-400 hover:text-gray-600 dark:hover:text-gray-300"
            >
              ×
            </button>
          </div>
        </div>

        <div className="p-6 space-y-4 overflow-y-auto">
          <div className="grid grid-cols-2 gap-4">
            <div>
              <label className="text-sm font-medium text-gray-500 dark:text-gray-400">CID</label>
              <div className="flex items-center gap-2 mt-1">
                <span className="font-mono text-sm text-gray-900 dark:text-white break-all">
                  {file.cid}
                </span>
                <button
                  onClick={() => onCopy(file.cid)}
                  className="text-blue-500 hover:text-blue-600"
                >
                  <Copy className="w-4 h-4" />
                </button>
              </div>
            </div>
            <div>
              <label className="text-sm font-medium text-gray-500 dark:text-gray-400">Size</label>
              <p className="text-sm text-gray-900 dark:text-white mt-1">
                {/* formatSize(file.size) */}
              </p>
            </div>
          </div>

          <div className="flex gap-4 pt-4">
            <button
              onClick={onClose}
              className="px-4 py-2 bg-gray-100 dark:bg-gray-700 text-gray-900 dark:text-white rounded-lg hover:bg-gray-200 dark:hover:bg-gray-600 transition-colors"
            >
              Close
            </button>
            <button className="px-4 py-2 bg-blue-500 text-white rounded-lg hover:bg-blue-600 transition-colors flex items-center gap-2">
              <Globe className="w-4 h-4" />
              Open in Gateway
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}