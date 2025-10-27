import { PrismaClient } from '@prisma/client';
import { ethers } from 'ethers';
import axios from 'axios';

const prisma = new PrismaClient();

interface CitrateBlock {
  number: string;
  hash: string;
  parentHash: string;
  selectedParent?: string;
  mergeParents?: string[];
  timestamp: string;
  miner: string;
  gasUsed: string;
  gasLimit: string;
  blueScore?: string;
  difficulty?: string;
  totalDifficulty?: string;
  size: string;
  stateRoot: string;
  transactionsRoot: string;
  receiptsRoot: string;
  transactions: any[];
}

class CitrateIndexer {
  private provider: ethers.JsonRpcProvider;
  private rpcClient = axios.create({
    baseURL: process.env.RPC_ENDPOINT || 'http://localhost:8545',
    headers: { 'Content-Type': 'application/json' }
  });
  private isRunning = false;
  private lastBlockNumber = 0n;

  constructor() {
    this.provider = new ethers.JsonRpcProvider(
      process.env.RPC_ENDPOINT || 'http://localhost:8545'
    );
  }

  async start() {
    console.log('🚀 Starting Citrate Explorer Indexer...');
    this.isRunning = true;

    // Get last indexed block
    const lastBlock = await prisma.block.findFirst({
      orderBy: { number: 'desc' }
    });

    if (lastBlock) {
      this.lastBlockNumber = lastBlock.number;
      console.log(`📦 Resuming from block ${this.lastBlockNumber}`);
    } else {
      console.log('📦 Starting fresh indexing from genesis');
    }

    // Start indexing loop
    this.indexLoop();

    // Start stats collection
    this.statsLoop();
  }

  private async indexLoop() {
    while (this.isRunning) {
      try {
        const latestBlockNumber = await this.getLatestBlockNumber();
        
        if (latestBlockNumber > this.lastBlockNumber) {
          console.log(`📊 Indexing blocks ${this.lastBlockNumber + 1n} to ${latestBlockNumber}`);
          
          for (let i = this.lastBlockNumber + 1n; i <= latestBlockNumber; i++) {
            await this.indexBlock(i);
            this.lastBlockNumber = i;
          }
        }

        // Wait before next check
        await new Promise(resolve => setTimeout(resolve, 2000));
      } catch (error) {
        console.error('❌ Indexing error:', error);
        await new Promise(resolve => setTimeout(resolve, 5000));
      }
    }
  }

  private async getLatestBlockNumber(): Promise<bigint> {
    const response = await this.rpcCall('eth_blockNumber');
    return BigInt(response);
  }

  private async indexBlock(blockNumber: bigint) {
    try {
      // Fetch block with transactions
      const block = await this.getBlock(blockNumber);
      
      // Store block
      await prisma.block.create({
        data: {
          id: `block-${block.number}`,
          number: BigInt(block.number),
          hash: block.hash,
          parentHash: block.parentHash,
          selectedParent: block.selectedParent,
          mergeParents: block.mergeParents || [],
          timestamp: new Date(parseInt(block.timestamp, 16) * 1000),
          miner: block.miner,
          gasUsed: BigInt(block.gasUsed),
          gasLimit: BigInt(block.gasLimit),
          blueScore: block.blueScore ? BigInt(block.blueScore) : null,
          isBlue: true, // Will be calculated based on DAG
          difficulty: block.difficulty ? BigInt(block.difficulty) : null,
          totalDifficulty: block.totalDifficulty ? BigInt(block.totalDifficulty) : null,
          size: parseInt(block.size, 16),
          stateRoot: block.stateRoot,
          txRoot: block.transactionsRoot,
          receiptRoot: block.receiptsRoot,
        }
      });

      // Index transactions
      for (const tx of block.transactions) {
        await this.indexTransaction(tx, block);
      }

      // Update search index
      await this.updateSearchIndex('block', block.hash, {
        number: block.number,
        hash: block.hash,
        timestamp: block.timestamp,
        transactionCount: block.transactions.length
      });

      console.log(`✅ Indexed block ${blockNumber} with ${block.transactions.length} transactions`);
    } catch (error: any) {
      if (error.code === 'P2002') {
        console.log(`⏭️ Block ${blockNumber} already indexed`);
      } else {
        throw error;
      }
    }
  }

  private async indexTransaction(tx: any, block: CitrateBlock) {
    try {
      // Get transaction receipt
      const receipt = await this.getTransactionReceipt(tx.hash);
      
      // Determine transaction type
      const txType = this.getTransactionType(tx, receipt);
      
      // Store transaction
      await prisma.transaction.create({
        data: {
          id: `tx-${tx.hash}`,
          hash: tx.hash,
          blockHash: block.hash,
          blockNumber: BigInt(block.number),
          from: tx.from.toLowerCase(),
          to: tx.to ? tx.to.toLowerCase() : null,
          value: tx.value,
          gas: BigInt(tx.gas),
          gasPrice: tx.gasPrice,
          nonce: BigInt(tx.nonce),
          data: tx.input,
          status: receipt.status === '0x1',
          contractAddress: receipt.contractAddress,
          type: txType,
        }
      });

      // Index logs
      for (const log of receipt.logs) {
        await this.indexLog(log, tx.hash);
      }

      // Handle special transaction types
      if (txType === 'model') {
        await this.indexModelDeployment(tx, receipt, block);
      } else if (txType === 'inference') {
        await this.indexInference(tx, receipt, block);
      }

      // Update account info
      await this.updateAccount(tx.from);
      if (tx.to) {
        await this.updateAccount(tx.to);
      }

      // Update search index
      await this.updateSearchIndex('transaction', tx.hash, {
        hash: tx.hash,
        from: tx.from,
        to: tx.to,
        value: tx.value,
        blockNumber: block.number
      });
    } catch (error: any) {
      if (error.code !== 'P2002') {
        console.error(`Failed to index transaction ${tx.hash}:`, error);
      }
    }
  }

  private async indexLog(log: any, txHash: string) {
    try {
      const eventName = this.decodeEventName(log.topics[0]);
      
      await prisma.log.create({
        data: {
          id: `log-${txHash}-${log.logIndex}`,
          transactionHash: txHash,
          logIndex: parseInt(log.logIndex, 16),
          address: log.address.toLowerCase(),
          topics: log.topics,
          data: log.data,
          eventName,
        }
      });
    } catch (error: any) {
      if (error.code !== 'P2002') {
        console.error(`Failed to index log:`, error);
      }
    }
  }

  private async indexModelDeployment(tx: any, receipt: any, block: CitrateBlock) {
    try {
      // Parse model deployment data
      const modelData = this.parseModelDeployment(tx.input);
      
      if (modelData) {
        const modelId = `model-${receipt.contractAddress || tx.hash}`;
        
        await prisma.model.create({
          data: {
            id: `model-${modelId}`,
            modelId,
            owner: tx.from.toLowerCase(),
            name: modelData.name || 'Unnamed Model',
            version: modelData.version || '1.0.0',
            format: modelData.format || 'unknown',
            dataHash: modelData.dataHash || tx.hash,
            metadata: modelData.metadata || {},
            blockNumber: BigInt(block.number),
            blockHash: block.hash,
            transactionHash: tx.hash,
            timestamp: new Date(parseInt(block.timestamp, 16) * 1000),
            size: modelData.size ? BigInt(modelData.size) : null,
            permissions: [],
          }
        });

        await prisma.modelOperation.create({
          data: {
            id: `op-${tx.hash}`,
            modelId,
            operationType: 'deploy',
            transactionHash: tx.hash,
            blockNumber: BigInt(block.number),
            timestamp: new Date(parseInt(block.timestamp, 16) * 1000),
            details: modelData,
          }
        });

        console.log(`🤖 Indexed model deployment: ${modelId}`);
      }
    } catch (error: any) {
      if (error.code !== 'P2002') {
        console.error(`Failed to index model deployment:`, error);
      }
    }
  }

  private async indexInference(tx: any, receipt: any, block: CitrateBlock) {
    try {
      // Parse inference data
      const inferenceData = this.parseInference(tx.input);
      
      if (inferenceData) {
        const inferenceId = `inference-${tx.hash}`;
        
        await prisma.inference.create({
          data: {
            id: inferenceId,
            inferenceId,
            modelId: inferenceData.modelId,
            executorAddress: tx.from.toLowerCase(),
            inputHash: inferenceData.inputHash || '',
            outputHash: inferenceData.outputHash || '',
            proofId: inferenceData.proofId,
            gasUsed: BigInt(receipt.gasUsed),
            executionTime: inferenceData.executionTime || 0,
            timestamp: new Date(parseInt(block.timestamp, 16) * 1000),
            blockNumber: BigInt(block.number),
            transactionHash: tx.hash,
          }
        });

        console.log(`🧠 Indexed inference: ${inferenceId}`);
      }
    } catch (error: any) {
      if (error.code !== 'P2002') {
        console.error(`Failed to index inference:`, error);
      }
    }
  }

  private async updateAccount(address: string) {
    try {
      const balance = await this.provider.getBalance(address);
      const nonce = await this.provider.getTransactionCount(address);
      const code = await this.provider.getCode(address);
      
      const isContract = code !== '0x';
      const now = new Date();
      
      await prisma.account.upsert({
        where: { address: address.toLowerCase() },
        update: {
          balance: balance.toString(),
          nonce: BigInt(nonce),
          lastSeen: now,
          transactionCount: { increment: 1 }
        },
        create: {
          id: `account-${address.toLowerCase()}`,
          address: address.toLowerCase(),
          balance: balance.toString(),
          nonce: BigInt(nonce),
          isContract,
          firstSeen: now,
          lastSeen: now,
          transactionCount: 1
        }
      });

      await this.updateSearchIndex('address', address, {
        address: address.toLowerCase(),
        isContract,
        balance: balance.toString()
      });
    } catch (error) {
      console.error(`Failed to update account ${address}:`, error);
    }
  }

  private async updateSearchIndex(type: string, identifier: string, data: any) {
    try {
      await prisma.searchIndex.upsert({
        where: {
          type_identifier: { type, identifier }
        },
        update: { data },
        create: {
          type,
          identifier,
          data
        }
      });
    } catch (error) {
      console.error(`Failed to update search index:`, error);
    }
  }

  private async statsLoop() {
    while (this.isRunning) {
      try {
        await this.collectStats();
        await new Promise(resolve => setTimeout(resolve, 60000)); // Every minute
      } catch (error) {
        console.error('Stats collection error:', error);
      }
    }
  }

  private async collectStats() {
    try {
      // Get DAG stats from custom RPC
      const dagStats = await this.rpcCall('citrate_getDagStats').catch(() => null);
      
      // Calculate from DB if RPC not available
      const totalBlocks = await prisma.block.count();
      const blueBlocks = await prisma.block.count({ where: { isBlue: true } });
      const redBlocks = totalBlocks - blueBlocks;
      
      // Get tips (blocks without children)
      const tips = await prisma.$queryRaw<any[]>`
        SELECT COUNT(*) as count FROM "Block" b1
        WHERE NOT EXISTS (
          SELECT 1 FROM "Block" b2 
          WHERE b2."parentHash" = b1.hash 
          OR b1.hash = ANY(b2."mergeParents")
        )
      `;
      
      const tipsCount = parseInt(tips[0]?.count || '0');
      
      // Calculate average block time
      const recentBlocks = await prisma.block.findMany({
        orderBy: { number: 'desc' },
        take: 100,
        select: { timestamp: true }
      });
      
      let avgBlockTime = 0;
      if (recentBlocks.length > 1) {
        const timeDiffs = [];
        for (let i = 1; i < recentBlocks.length; i++) {
          const diff = recentBlocks[i - 1].timestamp.getTime() - recentBlocks[i].timestamp.getTime();
          timeDiffs.push(diff / 1000);
        }
        avgBlockTime = timeDiffs.reduce((a, b) => a + b, 0) / timeDiffs.length;
      }
      
      // Calculate TPS
      const lastHourTxCount = await prisma.transaction.count({
        where: {
          createdAt: { gte: new Date(Date.now() - 3600000) }
        }
      });
      const tps = lastHourTxCount / 3600;
      
      // Count active models and inferences
      const activeModels = await prisma.model.count();
      const totalInferences = await prisma.inference.count();
      
      // Store stats
      await prisma.dagStats.create({
        data: {
          totalBlocks: BigInt(totalBlocks),
          blueBlocks: BigInt(blueBlocks),
          redBlocks: BigInt(redBlocks),
          tipsCount,
          maxBlueScore: dagStats?.maxBlueScore ? BigInt(dagStats.maxBlueScore) : 0n,
          avgBlockTime,
          tps,
          activeModels,
          totalInferences: BigInt(totalInferences)
        }
      });
      
      console.log(`📈 Stats: ${totalBlocks} blocks, ${tipsCount} tips, ${tps.toFixed(2)} TPS`);
    } catch (error) {
      console.error('Failed to collect stats:', error);
    }
  }

  private async getBlock(blockNumber: bigint): Promise<CitrateBlock> {
    const response = await this.rpcCall('eth_getBlockByNumber', [
      `0x${blockNumber.toString(16)}`,
      true
    ]);
    return response;
  }

  private async getTransactionReceipt(txHash: string) {
    return await this.rpcCall('eth_getTransactionReceipt', [txHash]);
  }

  private async rpcCall(method: string, params: any[] = []): Promise<any> {
    const response = await this.rpcClient.post('', {
      jsonrpc: '2.0',
      method,
      params,
      id: Date.now()
    });
    
    if (response.data.error) {
      throw new Error(response.data.error.message);
    }
    
    return response.data.result;
  }

  private getTransactionType(tx: any, receipt: any): string {
    // Check if contract creation
    if (!tx.to) {
      return 'contract';
    }
    
    // Check input data for method signatures
    if (tx.input && tx.input !== '0x') {
      const methodId = tx.input.slice(0, 10);
      
      // Model registry methods
      if (methodId === '0xdeployModel') return 'model';
      if (methodId === '0xrunInference') return 'inference';
      
      // Check logs for model events
      for (const log of receipt.logs) {
        if (log.topics[0] === this.getEventHash('ModelDeployed')) return 'model';
        if (log.topics[0] === this.getEventHash('InferenceExecuted')) return 'inference';
      }
      
      return 'contract';
    }
    
    return 'transfer';
  }

  private parseModelDeployment(input: string): any {
    // Parse model deployment data from transaction input
    // This would decode the actual method call
    try {
      // Simplified parsing - in production use ethers ABI decoder
      return {
        name: 'Model',
        version: '1.0.0',
        format: 'onnx',
        dataHash: input.slice(0, 66),
        metadata: {},
        size: 1000000
      };
    } catch {
      return null;
    }
  }

  private parseInference(input: string): any {
    // Parse inference data from transaction input
    try {
      return {
        modelId: 'model-0x123',
        inputHash: input.slice(0, 66),
        outputHash: '',
        executionTime: 100
      };
    } catch {
      return null;
    }
  }

  private decodeEventName(topic: string): string | null {
    const events: Record<string, string> = {
      [this.getEventHash('Transfer')]: 'Transfer',
      [this.getEventHash('ModelDeployed')]: 'ModelDeployed',
      [this.getEventHash('InferenceExecuted')]: 'InferenceExecuted',
      [this.getEventHash('PermissionGranted')]: 'PermissionGranted',
    };
    
    return events[topic] || null;
  }

  private getEventHash(eventName: string): string {
    // Generate event signature hash
    const hash = ethers.id(`${eventName}()`);
    return hash;
  }

  async stop() {
    this.isRunning = false;
    await prisma.$disconnect();
    console.log('🛑 Indexer stopped');
  }
}

// Start indexer
const indexer = new CitrateIndexer();

indexer.start().catch(error => {
  console.error('Fatal error:', error);
  process.exit(1);
});

// Graceful shutdown
process.on('SIGINT', async () => {
  console.log('\n📊 Shutting down indexer...');
  await indexer.stop();
  process.exit(0);
});

process.on('SIGTERM', async () => {
  await indexer.stop();
  process.exit(0);
});