'use client';

import { useQuery } from '@tanstack/react-query';
import axios from 'axios';
import Link from 'next/link';
import { Brain, Cpu, Zap, TrendingUp } from 'lucide-react';
import { formatDistanceToNow } from 'date-fns';

export function ModelActivity() {
  const { data, isLoading } = useQuery({
    queryKey: ['model-activity'],
    queryFn: async () => {
      const response = await axios.get('/api/models?limit=5');
      return response.data;
    },
    refetchInterval: 10000
  });

  const { data: inferenceStats } = useQuery({
    queryKey: ['inference-stats'],
    queryFn: async () => {
      const response = await axios.get('/api/inferences/stats');
      return response.data;
    },
    refetchInterval: 30000
  });

  return (
    <div className="bg-white dark:bg-gray-800 rounded-xl shadow-lg p-6">
      <div className="flex items-center justify-between mb-6">
        <h2 className="text-2xl font-bold text-gray-900 dark:text-white flex items-center gap-2">
          <Brain className="w-6 h-6" />
          AI Model Activity
        </h2>
        <Link
          href="/models"
          className="text-sm text-blue-500 hover:text-blue-600 transition-colors"
        >
          View all models →
        </Link>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
        {/* Recent Models */}
        <div className="lg:col-span-2">
          <h3 className="text-sm font-semibold text-gray-600 dark:text-gray-400 mb-4">
            Recently Deployed Models
          </h3>
          
          <div className="space-y-3">
            {isLoading ? (
              [...Array(3)].map((_, i) => (
                <div key={i} className="animate-pulse">
                  <div className="h-20 bg-gray-100 dark:bg-gray-700 rounded-lg"></div>
                </div>
              ))
            ) : (
              data?.models.map((model: any) => (
                <div
                  key={model.id}
                  className="p-4 bg-gray-50 dark:bg-gray-700 rounded-lg hover:bg-gray-100 dark:hover:bg-gray-600 transition-colors"
                >
                  <div className="flex items-start justify-between">
                    <div className="flex items-start gap-3">
                      <div className="p-2 bg-purple-100 dark:bg-purple-900 rounded-lg">
                        <Cpu className="w-5 h-5 text-purple-500" />
                      </div>
                      
                      <div>
                        <Link
                          href={`/model/${model.modelId}`}
                          className="font-medium text-gray-900 dark:text-white hover:text-blue-500"
                        >
                          {model.name}
                        </Link>
                        
                        <div className="flex items-center gap-4 mt-1">
                          <span className="text-xs text-gray-500 dark:text-gray-400">
                            v{model.version}
                          </span>
                          <span className="text-xs px-2 py-0.5 bg-blue-100 dark:bg-blue-900 text-blue-600 dark:text-blue-400 rounded-full">
                            {model.format}
                          </span>
                          <span className="text-xs text-gray-500 dark:text-gray-400">
                            {formatDistanceToNow(new Date(model.timestamp), { addSuffix: true })}
                          </span>
                        </div>
                        
                        <div className="flex items-center gap-4 mt-2">
                          <div className="flex items-center gap-1">
                            <Zap className="w-3 h-3 text-yellow-500" />
                            <span className="text-xs text-gray-600 dark:text-gray-400">
                              {model._count.inferences} inferences
                            </span>
                          </div>
                          <div className="flex items-center gap-1">
                            <TrendingUp className="w-3 h-3 text-green-500" />
                            <span className="text-xs text-gray-600 dark:text-gray-400">
                              {model._count.operations} ops
                            </span>
                          </div>
                        </div>
                      </div>
                    </div>
                    
                    <div className="text-right">
                      <p className="text-xs text-gray-500 dark:text-gray-400">
                        Owner
                      </p>
                      <Link
                        href={`/address/${model.owner}`}
                        className="font-mono text-xs text-blue-500 hover:text-blue-600"
                      >
                        {model.owner.slice(0, 6)}...{model.owner.slice(-4)}
                      </Link>
                    </div>
                  </div>
                </div>
              ))
            )}
          </div>
        </div>

        {/* Inference Statistics */}
        <div>
          <h3 className="text-sm font-semibold text-gray-600 dark:text-gray-400 mb-4">
            Inference Statistics
          </h3>
          
          <div className="space-y-4">
            <div className="p-4 bg-gradient-to-r from-purple-50 to-blue-50 dark:from-purple-900/20 dark:to-blue-900/20 rounded-lg">
              <div className="flex items-center justify-between mb-2">
                <span className="text-sm text-gray-600 dark:text-gray-400">
                  Total Inferences
                </span>
                <Zap className="w-4 h-4 text-yellow-500" />
              </div>
              <p className="text-2xl font-bold text-gray-900 dark:text-white">
                {inferenceStats?.total?.toLocaleString() || '0'}
              </p>
              <p className="text-xs text-green-500 mt-1">
                +{inferenceStats?.last24h || 0} today
              </p>
            </div>
            
            <div className="p-4 bg-gray-50 dark:bg-gray-700 rounded-lg">
              <div className="flex items-center justify-between mb-2">
                <span className="text-sm text-gray-600 dark:text-gray-400">
                  Avg Response Time
                </span>
              </div>
              <p className="text-xl font-bold text-gray-900 dark:text-white">
                {inferenceStats?.avgTime || '0'}ms
              </p>
            </div>
            
            <div className="p-4 bg-gray-50 dark:bg-gray-700 rounded-lg">
              <div className="flex items-center justify-between mb-2">
                <span className="text-sm text-gray-600 dark:text-gray-400">
                  Active Models
                </span>
              </div>
              <p className="text-xl font-bold text-gray-900 dark:text-white">
                {inferenceStats?.activeModels || '0'}
              </p>
            </div>
            
            <div className="p-4 bg-gray-50 dark:bg-gray-700 rounded-lg">
              <div className="flex items-center justify-between mb-2">
                <span className="text-sm text-gray-600 dark:text-gray-400">
                  Proofs Generated
                </span>
              </div>
              <p className="text-xl font-bold text-gray-900 dark:text-white">
                {inferenceStats?.proofsGenerated || '0'}
              </p>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}