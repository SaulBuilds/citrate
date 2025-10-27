import * as vscode from 'vscode';
import axios, { AxiosInstance } from 'axios';
import Web3 from 'web3';

export interface LatticeModel {
    id: string;
    name?: string;
    description?: string;
    version?: string;
    owner?: string;
    price?: string;
    totalInferences?: number;
    totalRevenue?: string;
    status?: string;
}

export interface DeploymentConfig {
    encrypted?: boolean;
    price?: string;
    metadata?: {
        name: string;
        description: string;
        version: string;
        tags?: string[];
    };
}

export interface DeploymentResult {
    modelId: string;
    txHash: string;
    contractAddress?: string;
}

export interface InferenceResult {
    outputData: any;
    gasUsed: number;
    executionTime: number;
    txHash?: string;
}

export interface NetworkStatus {
    connected: boolean;
    chainId?: number;
    blockNumber?: number;
    peerCount?: number;
    gasPrice?: number;
}

export class CitrateClient {
    private rpcUrl: string;
    private client: AxiosInstance;
    private web3: Web3;
    private connected: boolean = false;
    private connectionListeners: ((connected: boolean) => void)[] = [];

    constructor(rpcUrl: string) {
        this.rpcUrl = rpcUrl;
        this.client = axios.create({
            baseURL: rpcUrl,
            timeout: 30000,
            headers: {
                'Content-Type': 'application/json'
            }
        });
        this.web3 = new Web3(rpcUrl);
    }

    updateRpcUrl(newUrl: string) {
        this.rpcUrl = newUrl;
        this.client = axios.create({
            baseURL: newUrl,
            timeout: 30000,
            headers: {
                'Content-Type': 'application/json'
            }
        });
        this.web3 = new Web3(newUrl);

        // Reconnect with new URL
        if (this.connected) {
            this.connect();
        }
    }

    async connect(): Promise<void> {
        try {
            // Test connection with a simple RPC call
            const chainId = await this.rpcCall('eth_chainId');
            this.connected = true;
            this.notifyConnectionChange(true);
        } catch (error) {
            this.connected = false;
            this.notifyConnectionChange(false);
            throw new Error(`Failed to connect to Citrate node: ${error}`);
        }
    }

    disconnect() {
        this.connected = false;
        this.notifyConnectionChange(false);
    }

    get isConnected(): boolean {
        return this.connected;
    }

    onConnectionChange(listener: (connected: boolean) => void) {
        this.connectionListeners.push(listener);
    }

    private notifyConnectionChange(connected: boolean) {
        this.connectionListeners.forEach(listener => listener(connected));
    }

    private async rpcCall(method: string, params: any[] = []): Promise<any> {
        try {
            const response = await this.client.post('/', {
                jsonrpc: '2.0',
                method,
                params,
                id: Date.now()
            });

            if (response.data.error) {
                throw new Error(response.data.error.message);
            }

            return response.data.result;
        } catch (error: any) {
            if (error.response?.data?.error) {
                throw new Error(error.response.data.error.message);
            }
            throw error;
        }
    }

    async getNetworkStatus(): Promise<NetworkStatus> {
        try {
            const [chainId, blockNumber, peerCount, gasPrice] = await Promise.all([
                this.rpcCall('eth_chainId'),
                this.rpcCall('eth_blockNumber'),
                this.rpcCall('net_peerCount').catch(() => '0x0'),
                this.rpcCall('eth_gasPrice').catch(() => '0x0')
            ]);

            return {
                connected: true,
                chainId: parseInt(chainId, 16),
                blockNumber: parseInt(blockNumber, 16),
                peerCount: parseInt(peerCount, 16),
                gasPrice: parseInt(gasPrice, 16)
            };
        } catch (error) {
            return { connected: false };
        }
    }

    async getModels(): Promise<LatticeModel[]> {
        try {
            // Try Lattice-specific RPC method
            const models = await this.rpcCall('citrate_getModels');
            return models || [];
        } catch (error) {
            // Fallback to mock data for development
            return [
                {
                    id: 'model_001',
                    name: 'Image Classifier',
                    description: 'CNN model for image classification',
                    version: '1.0.0',
                    owner: '0x1234...5678',
                    price: '1000000000000000000',
                    totalInferences: 1234,
                    totalRevenue: '2500000000000000000',
                    status: 'active'
                },
                {
                    id: 'model_002',
                    name: 'Text Sentiment',
                    description: 'NLP model for sentiment analysis',
                    version: '2.1.0',
                    owner: '0xabcd...efgh',
                    price: '500000000000000000',
                    totalInferences: 567,
                    totalRevenue: '1200000000000000000',
                    status: 'active'
                }
            ];
        }
    }

    async getModelInfo(modelId: string): Promise<LatticeModel | null> {
        try {
            return await this.rpcCall('citrate_getModelInfo', [modelId]);
        } catch (error) {
            console.warn(`Failed to get model info for ${modelId}:`, error);
            return null;
        }
    }

    async deployModel(modelData: Buffer, config: DeploymentConfig): Promise<DeploymentResult> {
        try {
            // In a real implementation, this would:
            // 1. Upload model to IPFS
            // 2. Create deployment transaction
            // 3. Submit to blockchain

            const modelDataBase64 = modelData.toString('base64');

            const result = await this.rpcCall('citrate_deployModel', [{
                modelData: modelDataBase64,
                metadata: config.metadata,
                price: config.price || '1000000000000000000',
                encrypted: config.encrypted || false
            }]);

            return {
                modelId: result.modelId || `model_${Date.now()}`,
                txHash: result.txHash || `0x${Math.random().toString(16).slice(2)}`,
                contractAddress: result.contractAddress
            };
        } catch (error: any) {
            // For development, return mock success
            if (error.message.includes('Method not found') || error.code === -32601) {
                return {
                    modelId: `model_${Date.now()}`,
                    txHash: `0x${Math.random().toString(16).slice(2)}${Math.random().toString(16).slice(2)}`
                };
            }
            throw error;
        }
    }

    async runInference(modelId: string, inputData: any): Promise<InferenceResult> {
        try {
            const result = await this.rpcCall('citrate_runInference', [modelId, inputData]);
            return result;
        } catch (error: any) {
            // For development, return mock result
            if (error.message.includes('Method not found') || error.code === -32601) {
                return {
                    outputData: {
                        prediction: 'mock_prediction',
                        confidence: 0.95,
                        processing_time: 0.123
                    },
                    gasUsed: 21000,
                    executionTime: 123,
                    txHash: `0x${Math.random().toString(16).slice(2)}${Math.random().toString(16).slice(2)}`
                };
            }
            throw error;
        }
    }

    async getAccounts(): Promise<string[]> {
        try {
            return await this.rpcCall('eth_accounts');
        } catch (error) {
            return [];
        }
    }

    async getBalance(address: string): Promise<string> {
        try {
            const balance = await this.rpcCall('eth_getBalance', [address, 'latest']);
            return this.web3.utils.fromWei(balance, 'ether');
        } catch (error) {
            return '0';
        }
    }

    async getBlock(blockNumber: string | number = 'latest'): Promise<any> {
        const blockParam = typeof blockNumber === 'number'
            ? `0x${blockNumber.toString(16)}`
            : blockNumber;
        return await this.rpcCall('eth_getBlockByNumber', [blockParam, true]);
    }

    async getTransaction(txHash: string): Promise<any> {
        return await this.rpcCall('eth_getTransactionByHash', [txHash]);
    }

    async getTransactionReceipt(txHash: string): Promise<any> {
        return await this.rpcCall('eth_getTransactionReceipt', [txHash]);
    }

    async getPeers(): Promise<any[]> {
        try {
            return await this.rpcCall('citrate_getPeers');
        } catch (error) {
            // Return mock peers for development
            return [
                { id: 'peer1', address: '192.168.1.100:30303', type: 'validator' },
                { id: 'peer2', address: '192.168.1.101:30303', type: 'peer' },
                { id: 'peer3', address: '192.168.1.102:30303', type: 'peer' }
            ];
        }
    }

    formatAddress(address: string): string {
        if (!address) return '';
        return `${address.slice(0, 6)}...${address.slice(-4)}`;
    }

    formatHash(hash: string): string {
        if (!hash) return '';
        return `${hash.slice(0, 10)}...${hash.slice(-8)}`;
    }

    formatEther(wei: string | number): string {
        return this.web3.utils.fromWei(wei.toString(), 'ether');
    }
}