import { NextResponse } from 'next/server';
import { PrismaClient } from '@prisma/client';

const prisma = new PrismaClient();

export async function GET() {
  try {
    const now = new Date();
    const yesterday = new Date(now.getTime() - 24 * 60 * 60 * 1000);

    const [total, last24h, avgTime, activeModels, proofsGenerated] = await Promise.all([
      prisma.inference.count(),
      prisma.inference.count({
        where: {
          timestamp: {
            gte: yesterday
          }
        }
      }),
      prisma.inference.aggregate({
        _avg: {
          executionTime: true
        }
      }),
      prisma.model.count({
        where: {
          inferences: {
            some: {
              timestamp: {
                gte: yesterday
              }
            }
          }
        }
      }),
      prisma.inference.count({
        where: {
          proofId: {
            not: null
          }
        }
      })
    ]);

    return NextResponse.json({
      total,
      last24h,
      avgTime: avgTime._avg.executionTime?.toFixed(2) || '0',
      activeModels,
      proofsGenerated
    });
  } catch (error) {
    console.error('Error fetching inference stats:', error);
    return NextResponse.json(
      { error: 'Failed to fetch inference stats' },
      { status: 500 }
    );
  }
}