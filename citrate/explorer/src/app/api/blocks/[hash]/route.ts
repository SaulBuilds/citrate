import { NextRequest, NextResponse } from 'next/server';
import { PrismaClient } from '@prisma/client';

const prisma = new PrismaClient();

export async function GET(
  request: NextRequest,
  { params }: { params: { hash: string } }
) {
  try {
    const block = await prisma.block.findFirst({
      where: {
        OR: [
          { hash: params.hash },
          { number: BigInt(params.hash) }
        ]
      },
      include: {
        transactions: {
          include: {
            logs: true
          }
        }
      }
    });

    if (!block) {
      return NextResponse.json(
        { error: 'Block not found' },
        { status: 404 }
      );
    }

    return NextResponse.json(block);
  } catch (error) {
    console.error('Error fetching block:', error);
    return NextResponse.json(
      { error: 'Failed to fetch block' },
      { status: 500 }
    );
  }
}