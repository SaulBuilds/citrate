'use client';

import { useQuery } from '@tanstack/react-query';
import axios from 'axios';
import Link from 'next/link';
import { formatDistanceToNow } from 'date-fns';
import { Blocks } from 'lucide-react';

export function LatestBlocks() {
  const { data, isLoading } = useQuery({
    queryKey: ['latest-blocks'],
    queryFn: async () => {
      const response = await axios.get('/api/blocks?limit=10');
      return response.data;
    },
    refetchInterval: 5000
  });

  return (
    <div className="bg-white dark:bg-gray-800 rounded-xl shadow-lg p-6">
      <div className="flex items-center justify-between mb-6">
        <h2 className="text-xl font-bold text-gray-900 dark:text-white flex items-center gap-2">
          <Blocks className="w-5 h-5" />
          Latest Blocks
        </h2>
        <Link
          href="/blocks"
          className="text-sm text-blue-500 hover:text-blue-600 transition-colors"
        >
          View all →
        </Link>
      </div>

      <div className="space-y-4">
        {isLoading ? (
          [...Array(5)].map((_, i) => (
            <div key={i} className="animate-pulse">
              <div className="h-16 bg-gray-100 dark:bg-gray-700 rounded-lg"></div>
            </div>
          ))
        ) : (
          data?.blocks.map((block: any) => (
            <div
              key={block.hash}
              className="flex items-center justify-between p-4 bg-gray-50 dark:bg-gray-700 rounded-lg hover:bg-gray-100 dark:hover:bg-gray-600 transition-colors"
            >
              <div className="flex items-center gap-4">
                <div className="bg-blue-500 bg-opacity-10 p-2 rounded-lg">
                  <Blocks className="w-5 h-5 text-blue-500" />
                </div>
                <div>
                  <Link
                    href={`/block/${block.hash}`}
                    className="font-mono text-sm text-blue-500 hover:text-blue-600"
                  >
                    #{block.number.toString()}
                  </Link>
                  <p className="text-xs text-gray-500 dark:text-gray-400 mt-1">
                    {formatDistanceToNow(new Date(block.timestamp), { addSuffix: true })}
                  </p>
                </div>
              </div>
              
              <div className="text-right">
                <p className="text-sm text-gray-900 dark:text-white">
                  {block._count.transactions} txns
                </p>
                <p className="text-xs text-gray-500 dark:text-gray-400 mt-1">
                  Miner: {block.miner.slice(0, 6)}...{block.miner.slice(-4)}
                </p>
              </div>
              
              {block.isBlue && (
                <div className="ml-2">
                  <span className="px-2 py-1 text-xs bg-blue-100 text-blue-600 rounded-full">
                    Blue
                  </span>
                </div>
              )}
            </div>
          ))
        )}
      </div>
    </div>
  );
}