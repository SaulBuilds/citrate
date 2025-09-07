'use client';

import { useState, useEffect } from 'react';
import { StatsOverview } from '@/components/home/stats-overview';
import { LatestBlocks } from '@/components/home/latest-blocks';
import { LatestTransactions } from '@/components/home/latest-transactions';
import { DagVisualization } from '@/components/home/dag-visualization';
import { ModelActivity } from '@/components/home/model-activity';
import { SearchBar } from '@/components/search/search-bar';
import { NetworkStatus } from '@/components/home/network-status';

export default function HomePage() {
  return (
    <div className="space-y-8">
      {/* Hero Section */}
      <div className="bg-gradient-to-r from-blue-600 to-purple-600 rounded-2xl p-8 text-white">
        <h1 className="text-4xl font-bold mb-4">Lattice Explorer</h1>
        <p className="text-lg opacity-90 mb-6">
          Explore the AI-native Layer-1 BlockDAG with GhostDAG consensus
        </p>
        <SearchBar />
      </div>

      {/* Network Status */}
      <NetworkStatus />

      {/* Stats Overview */}
      <StatsOverview />

      {/* DAG Visualization */}
      <div className="bg-white dark:bg-gray-800 rounded-xl shadow-lg p-6">
        <h2 className="text-2xl font-bold mb-4 text-gray-900 dark:text-white">
          DAG Structure
        </h2>
        <DagVisualization />
      </div>

      {/* Latest Activity */}
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-8">
        <LatestBlocks />
        <LatestTransactions />
      </div>

      {/* Model Activity */}
      <ModelActivity />
    </div>
  );
}