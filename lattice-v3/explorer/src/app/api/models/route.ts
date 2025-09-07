import { NextRequest, NextResponse } from 'next/server';
import { PrismaClient } from '@prisma/client';

const prisma = new PrismaClient();

export async function GET(request: NextRequest) {
  try {
    const searchParams = request.nextUrl.searchParams;
    const page = parseInt(searchParams.get('page') || '1');
    const limit = parseInt(searchParams.get('limit') || '20');
    const owner = searchParams.get('owner');
    const format = searchParams.get('format');
    const skip = (page - 1) * limit;

    const where: any = {};
    if (owner) where.owner = owner.toLowerCase();
    if (format) where.format = format;

    const [models, total] = await Promise.all([
      prisma.model.findMany({
        where,
        skip,
        take: limit,
        orderBy: { timestamp: 'desc' },
        include: {
          _count: {
            select: {
              operations: true,
              inferences: true
            }
          }
        }
      }),
      prisma.model.count({ where })
    ]);

    return NextResponse.json({
      models,
      pagination: {
        page,
        limit,
        total,
        pages: Math.ceil(total / limit)
      }
    });
  } catch (error) {
    console.error('Error fetching models:', error);
    return NextResponse.json(
      { error: 'Failed to fetch models' },
      { status: 500 }
    );
  }
}