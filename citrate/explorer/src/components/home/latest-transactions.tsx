'use client';

import { useQuery } from '@tanstack/react-query';
import axios from 'axios';
import Link from 'next/link';
import { formatDistanceToNow } from 'date-fns';
import { Activity, ArrowRight } from 'lucide-react';

export function LatestTransactions() {
  const { data, isLoading } = useQuery({
    queryKey: ['latest-transactions'],
    queryFn: async () => {
      const response = await axios.get('/api/transactions?limit=10');
      return response.data;
    },
    refetchInterval: 5000
  });

  const formatValue = (value: string) => {
    const eth = parseFloat(value) / 1e18;
    if (eth === 0) return '0 ETH';
    if (eth < 0.001) return '<0.001 ETH';
    return `${eth.toFixed(4)} ETH`;
  };

  return (
    <div className="bg-white dark:bg-gray-800 rounded-xl shadow-lg p-6">
      <div className="flex items-center justify-between mb-6">
        <h2 className="text-xl font-bold text-gray-900 dark:text-white flex items-center gap-2">
          <Activity className="w-5 h-5" />
          Latest Transactions
        </h2>
        <Link
          href="/transactions"
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
          data?.transactions.map((tx: any) => (
            <div
              key={tx.hash}
              className="p-4 bg-gray-50 dark:bg-gray-700 rounded-lg hover:bg-gray-100 dark:hover:bg-gray-600 transition-colors"
            >
              <div className="flex items-center justify-between">
                <div className="flex items-center gap-3">
                  <div className="bg-green-500 bg-opacity-10 p-2 rounded-lg">
                    <Activity className="w-5 h-5 text-green-500" />
                  </div>
                  <div>
                    <Link
                      href={`/tx/${tx.hash}`}
                      className="font-mono text-xs text-blue-500 hover:text-blue-600"
                    >
                      {tx.hash.slice(0, 10)}...{tx.hash.slice(-8)}
                    </Link>
                    <div className="flex items-center gap-2 mt-1">
                      <span className="text-xs text-gray-500">From</span>
                      <Link
                        href={`/address/${tx.from}`}
                        className="font-mono text-xs text-gray-600 dark:text-gray-400 hover:text-blue-500"
                      >
                        {tx.from.slice(0, 6)}...{tx.from.slice(-4)}
                      </Link>
                      <ArrowRight className="w-3 h-3 text-gray-400" />
                      <span className="text-xs text-gray-500">To</span>
                      <Link
                        href={`/address/${tx.to || 'contract'}`}
                        className="font-mono text-xs text-gray-600 dark:text-gray-400 hover:text-blue-500"
                      >
                        {tx.to ? `${tx.to.slice(0, 6)}...${tx.to.slice(-4)}` : 'Contract Creation'}
                      </Link>
                    </div>
                  </div>
                </div>
                
                <div className="text-right">
                  <p className="text-sm font-semibold text-gray-900 dark:text-white">
                    {formatValue(tx.value)}
                  </p>
                  <p className="text-xs text-gray-500 dark:text-gray-400 mt-1">
                    {formatDistanceToNow(new Date(tx.createdAt), { addSuffix: true })}
                  </p>
                </div>
              </div>
            </div>
          ))
        )}
      </div>
    </div>
  );
}