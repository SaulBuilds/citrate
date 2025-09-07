'use client';

import { useQuery } from '@tanstack/react-query';
import axios from 'axios';
import { Wifi, WifiOff, AlertCircle } from 'lucide-react';

export function NetworkStatus() {
  const { data: status, isError } = useQuery({
    queryKey: ['network-status'],
    queryFn: async () => {
      const response = await axios.get('/api/status');
      return response.data;
    },
    refetchInterval: 10000,
    retry: 1
  });

  const isOnline = !isError && status?.syncing === false;
  const isSyncing = status?.syncing === true;

  return (
    <div className={`rounded-lg p-4 ${
      isOnline ? 'bg-green-50 dark:bg-green-900/20' :
      isSyncing ? 'bg-yellow-50 dark:bg-yellow-900/20' :
      'bg-red-50 dark:bg-red-900/20'
    }`}>
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-3">
          {isOnline ? (
            <Wifi className="w-5 h-5 text-green-500" />
          ) : isSyncing ? (
            <AlertCircle className="w-5 h-5 text-yellow-500 animate-pulse" />
          ) : (
            <WifiOff className="w-5 h-5 text-red-500" />
          )}
          
          <div>
            <p className={`font-semibold ${
              isOnline ? 'text-green-700 dark:text-green-400' :
              isSyncing ? 'text-yellow-700 dark:text-yellow-400' :
              'text-red-700 dark:text-red-400'
            }`}>
              {isOnline ? 'Network Online' :
               isSyncing ? 'Syncing...' :
               'Network Offline'}
            </p>
            <p className="text-sm text-gray-600 dark:text-gray-400">
              {isOnline ? `Connected to ${status?.peers || 0} peers` :
               isSyncing ? `Progress: ${status?.syncProgress || 0}%` :
               'Unable to connect to network'}
            </p>
          </div>
        </div>
        
        {status && (
          <div className="text-right">
            <p className="text-sm text-gray-600 dark:text-gray-400">
              Chain ID: {status.chainId}
            </p>
            <p className="text-xs text-gray-500 dark:text-gray-500">
              Network: {status.network || 'Lattice v3'}
            </p>
          </div>
        )}
      </div>
    </div>
  );
}