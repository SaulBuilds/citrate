import { NextResponse } from 'next/server';
import { PrismaClient } from '@prisma/client';
import axios from 'axios';

const prisma = new PrismaClient();

export async function GET() {
  try {
    // Try to get network status from RPC
    let networkStatus: any = {};
    
    try {
      const rpcClient = axios.create({
        baseURL: process.env.RPC_ENDPOINT || 'http://localhost:8545',
      });
      
      const [syncingRes, peersRes, chainIdRes] = await Promise.all([
        rpcClient.post('', {
          jsonrpc: '2.0',
          method: 'eth_syncing',
          params: [],
          id: 1
        }),
        rpcClient.post('', {
          jsonrpc: '2.0',
          method: 'net_peerCount',
          params: [],
          id: 2
        }),
        rpcClient.post('', {
          jsonrpc: '2.0',
          method: 'eth_chainId',
          params: [],
          id: 3
        })
      ]);
      
      networkStatus = {
        syncing: syncingRes.data.result !== false,
        syncProgress: syncingRes.data.result ? 
          ((syncingRes.data.result.currentBlock / syncingRes.data.result.highestBlock) * 100).toFixed(2) : 
          100,
        peers: parseInt(peersRes.data.result, 16),
        chainId: parseInt(chainIdRes.data.result, 16),
        network: process.env.NETWORK_NAME || 'Lattice v3'
      };
    } catch (error) {
      console.error('RPC connection error:', error);
      networkStatus = {
        syncing: false,
        peers: 0,
        chainId: parseInt(process.env.CHAIN_ID || '1337'),
        network: process.env.NETWORK_NAME || 'Lattice v3',
        error: 'RPC connection failed'
      };
    }
    
    return NextResponse.json(networkStatus);
  } catch (error) {
    console.error('Error fetching status:', error);
    return NextResponse.json(
      { error: 'Failed to fetch status' },
      { status: 500 }
    );
  }
}