import { NextRequest, NextResponse } from 'next/server';
import { PrismaClient } from '@prisma/client';

const prisma = new PrismaClient();

export async function GET(request: NextRequest) {
  try {
    const searchParams = request.nextUrl.searchParams;
    const page = parseInt(searchParams.get('page') || '1');
    const limit = parseInt(searchParams.get('limit') || '20');
    const skip = (page - 1) * limit;

    const [blocks, total] = await Promise.all([
      prisma.block.findMany({
        skip,
        take: limit,
        orderBy: { number: 'desc' },
        include: {
          _count: {
            select: { transactions: true }
          }
        }
      }),
      prisma.block.count()
    ]);

    return NextResponse.json({
      blocks,
      pagination: {
        page,
        limit,
        total,
        pages: Math.ceil(total / limit)
      }
    });
  } catch (error) {
    console.error('Error fetching blocks:', error);
    return NextResponse.json(
      { error: 'Failed to fetch blocks' },
      { status: 500 }
    );
  }
}