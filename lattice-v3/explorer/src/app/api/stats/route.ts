import { NextResponse } from 'next/server';
import { PrismaClient } from '@prisma/client';

const prisma = new PrismaClient();

export async function GET() {
  try {
    const [
      latestBlock,
      totalBlocks,
      totalTransactions,
      totalModels,
      totalInferences,
      latestStats,
      last24hTransactions
    ] = await Promise.all([
      prisma.block.findFirst({
        orderBy: { number: 'desc' }
      }),
      prisma.block.count(),
      prisma.transaction.count(),
      prisma.model.count(),
      prisma.inference.count(),
      prisma.dagStats.findFirst({
        orderBy: { timestamp: 'desc' }
      }),
      prisma.transaction.count({
        where: {
          createdAt: {
            gte: new Date(Date.now() - 24 * 60 * 60 * 1000)
          }
        }
      })
    ]);

    const tps = last24hTransactions / (24 * 60 * 60);

    return NextResponse.json({
      blockHeight: latestBlock?.number.toString() || '0',
      totalBlocks,
      totalTransactions,
      totalModels,
      totalInferences,
      tps: tps.toFixed(2),
      blueBlocks: latestStats?.blueBlocks.toString() || '0',
      redBlocks: latestStats?.redBlocks.toString() || '0',
      tipsCount: latestStats?.tipsCount || 0,
      avgBlockTime: latestStats?.avgBlockTime || 0,
      maxBlueScore: latestStats?.maxBlueScore.toString() || '0'
    });
  } catch (error) {
    console.error('Error fetching stats:', error);
    return NextResponse.json(
      { error: 'Failed to fetch stats' },
      { status: 500 }
    );
  }
}