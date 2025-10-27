import { NextRequest, NextResponse } from 'next/server';
import { PrismaClient } from '@prisma/client';

const prisma = new PrismaClient();

export async function GET(request: NextRequest) {
  try {
    const searchParams = request.nextUrl.searchParams;
    const depth = parseInt(searchParams.get('depth') || '10');
    const startBlock = searchParams.get('block');

    // Get recent blocks for DAG visualization
    const blocks = await prisma.block.findMany({
      take: Math.min(depth, 100),
      orderBy: { number: 'desc' },
      select: {
        hash: true,
        number: true,
        parentHash: true,
        selectedParent: true,
        mergeParents: true,
        timestamp: true,
        blueScore: true,
        isBlue: true,
        _count: {
          select: { transactions: true }
        }
      }
    });

    // Build nodes and links for D3 visualization
    const nodes = blocks.map(block => ({
      id: block.hash,
      number: block.number.toString(),
      timestamp: block.timestamp.toISOString(),
      blueScore: block.blueScore?.toString() || '0',
      isBlue: block.isBlue,
      txCount: block._count.transactions
    }));

    const links: any[] = [];
    
    blocks.forEach(block => {
      // Add parent link
      if (block.parentHash) {
        links.push({
          source: block.parentHash,
          target: block.hash,
          type: 'parent'
        });
      }
      
      // Add selected parent link if different
      if (block.selectedParent && block.selectedParent !== block.parentHash) {
        links.push({
          source: block.selectedParent,
          target: block.hash,
          type: 'selected'
        });
      }
      
      // Add merge parent links
      block.mergeParents.forEach(mergeParent => {
        links.push({
          source: mergeParent,
          target: block.hash,
          type: 'merge'
        });
      });
    });

    return NextResponse.json({
      nodes,
      links,
      stats: {
        totalNodes: nodes.length,
        blueNodes: nodes.filter(n => n.isBlue).length,
        redNodes: nodes.filter(n => !n.isBlue).length
      }
    });
  } catch (error) {
    console.error('Error fetching DAG data:', error);
    return NextResponse.json(
      { error: 'Failed to fetch DAG data' },
      { status: 500 }
    );
  }
}