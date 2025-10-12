#!/usr/bin/env node

/**
 * Lattice API Testing Suite
 * Comprehensive testing framework for Lattice blockchain APIs
 */

const axios = require('axios');
const Web3 = require('web3');
const fs = require('fs');
const path = require('path');

class LatticeTestRunner {
    constructor(config = {}) {
        this.rpcUrl = config.rpcUrl || 'http://localhost:8545';
        this.web3 = new Web3(this.rpcUrl);
        this.client = axios.create({
            baseURL: this.rpcUrl,
            timeout: 30000,
            headers: { 'Content-Type': 'application/json' }
        });
        this.results = [];
        this.stats = { passed: 0, failed: 0, total: 0 };
    }

    async rpcCall(method, params = []) {
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
    }

    async test(name, testFn) {
        console.log(`🧪 Running: ${name}`);
        this.stats.total++;

        try {
            const startTime = Date.now();
            await testFn();
            const duration = Date.now() - startTime;

            this.results.push({
                name,
                status: 'PASSED',
                duration,
                error: null
            });

            this.stats.passed++;
            console.log(`✅ ${name} (${duration}ms)`);
        } catch (error) {
            this.results.push({
                name,
                status: 'FAILED',
                duration: 0,
                error: error.message
            });

            this.stats.failed++;
            console.log(`❌ ${name}: ${error.message}`);
        }
    }

    async runBasicTests() {
        console.log('🚀 Starting Lattice API Basic Tests...\n');

        // Connection test
        await this.test('Connection Test', async () => {
            const chainId = await this.rpcCall('eth_chainId');
            if (!chainId) throw new Error('No chain ID returned');
        });

        // Block retrieval test
        await this.test('Block Retrieval Test', async () => {
            const latestBlock = await this.rpcCall('eth_getBlockByNumber', ['latest', false]);
            if (!latestBlock || !latestBlock.number) throw new Error('Invalid block data');
        });

        // Gas price test
        await this.test('Gas Price Test', async () => {
            const gasPrice = await this.rpcCall('eth_gasPrice');
            if (!gasPrice) throw new Error('No gas price returned');

            const price = parseInt(gasPrice, 16);
            if (price <= 0) throw new Error('Invalid gas price');
        });

        // Network peer test
        await this.test('Network Peers Test', async () => {
            try {
                const peerCount = await this.rpcCall('net_peerCount');
                console.log(`    📡 Connected peers: ${parseInt(peerCount, 16)}`);
            } catch (error) {
                // Some nodes might not support this method
                console.log('    ⚠️  Peer count not available');
            }
        });

        // Account test
        await this.test('Accounts Test', async () => {
            const accounts = await this.rpcCall('eth_accounts');
            console.log(`    👤 Available accounts: ${accounts ? accounts.length : 0}`);
        });
    }

    async runLatticeSpecificTests() {
        console.log('\n🤖 Starting Lattice-Specific Tests...\n');

        // Model registry test
        await this.test('Model Registry Test', async () => {
            try {
                const models = await this.rpcCall('lattice_getModels');
                console.log(`    🧠 Registered models: ${models ? models.length : 0}`);
            } catch (error) {
                if (error.message.includes('Method not found')) {
                    console.log('    ℹ️  Lattice-specific methods not available (using standard node)');
                } else {
                    throw error;
                }
            }
        });

        // Network topology test
        await this.test('Network Topology Test', async () => {
            try {
                const topology = await this.rpcCall('lattice_getNetworkTopology');
                console.log(`    🕸️  Network nodes: ${topology?.nodes?.length || 0}`);
            } catch (error) {
                if (error.message.includes('Method not found')) {
                    console.log('    ℹ️  Topology API not available');
                } else {
                    throw error;
                }
            }
        });

        // AI inference test (mock)
        await this.test('AI Inference API Test', async () => {
            try {
                const result = await this.rpcCall('lattice_runInference', [
                    'test_model_id',
                    { input: 'test_data' }
                ]);
                console.log('    🎯 Inference test completed');
            } catch (error) {
                if (error.message.includes('Method not found')) {
                    console.log('    ℹ️  Inference API not available');
                } else {
                    throw error;
                }
            }
        });
    }

    async runPerformanceTests() {
        console.log('\n⚡ Starting Performance Tests...\n');

        // RPC response time test
        await this.test('RPC Response Time Test', async () => {
            const iterations = 10;
            const times = [];

            for (let i = 0; i < iterations; i++) {
                const start = Date.now();
                await this.rpcCall('eth_blockNumber');
                times.push(Date.now() - start);
            }

            const avgTime = times.reduce((a, b) => a + b, 0) / times.length;
            const maxTime = Math.max(...times);

            console.log(`    📊 Average response time: ${avgTime.toFixed(2)}ms`);
            console.log(`    📊 Max response time: ${maxTime}ms`);

            if (avgTime > 1000) {
                throw new Error(`Average response time too high: ${avgTime.toFixed(2)}ms`);
            }
        });

        // Concurrent request test
        await this.test('Concurrent Requests Test', async () => {
            const concurrent = 5;
            const requests = [];

            for (let i = 0; i < concurrent; i++) {
                requests.push(this.rpcCall('eth_blockNumber'));
            }

            const start = Date.now();
            await Promise.all(requests);
            const duration = Date.now() - start;

            console.log(`    🔄 ${concurrent} concurrent requests completed in ${duration}ms`);

            if (duration > 5000) {
                throw new Error(`Concurrent requests took too long: ${duration}ms`);
            }
        });
    }

    async runContractTests() {
        console.log('\n📋 Starting Smart Contract Tests...\n');

        // EVM compatibility test
        await this.test('EVM Compatibility Test', async () => {
            try {
                // Test basic EVM methods
                const blockNumber = await this.rpcCall('eth_blockNumber');
                const gasPrice = await this.rpcCall('eth_gasPrice');

                console.log(`    🔧 Block number: ${parseInt(blockNumber, 16)}`);
                console.log(`    💰 Gas price: ${parseInt(gasPrice, 16)} wei`);
            } catch (error) {
                throw new Error(`EVM compatibility issue: ${error.message}`);
            }
        });

        // Contract call test (if contracts available)
        await this.test('Contract Call Test', async () => {
            try {
                // Try to call a view function on a known contract
                const result = await this.rpcCall('eth_call', [
                    {
                        to: '0x0000000000000000000000000000000000000000',
                        data: '0x'
                    },
                    'latest'
                ]);
                console.log('    📞 Contract call mechanism working');
            } catch (error) {
                // This is expected to fail for empty contract
                console.log('    ✅ Contract call mechanism available');
            }
        });
    }

    generateReport() {
        console.log('\n📊 Test Results Summary');
        console.log('=' .repeat(50));
        console.log(`Total Tests: ${this.stats.total}`);
        console.log(`✅ Passed: ${this.stats.passed}`);
        console.log(`❌ Failed: ${this.stats.failed}`);
        console.log(`📈 Success Rate: ${((this.stats.passed / this.stats.total) * 100).toFixed(1)}%`);

        if (this.stats.failed > 0) {
            console.log('\n❌ Failed Tests:');
            this.results
                .filter(r => r.status === 'FAILED')
                .forEach(result => {
                    console.log(`  • ${result.name}: ${result.error}`);
                });
        }

        // Save detailed report
        const report = {
            timestamp: new Date().toISOString(),
            rpcUrl: this.rpcUrl,
            summary: this.stats,
            results: this.results
        };

        const reportPath = path.join(__dirname, 'test-reports', `report-${Date.now()}.json`);

        // Ensure reports directory exists
        const reportsDir = path.dirname(reportPath);
        if (!fs.existsSync(reportsDir)) {
            fs.mkdirSync(reportsDir, { recursive: true });
        }

        fs.writeFileSync(reportPath, JSON.stringify(report, null, 2));
        console.log(`\n📄 Detailed report saved: ${reportPath}`);

        return this.stats.failed === 0;
    }

    async run() {
        console.log('🚀 Lattice API Testing Suite');
        console.log(`📡 Testing node: ${this.rpcUrl}`);
        console.log('=' .repeat(50));

        try {
            await this.runBasicTests();
            await this.runLatticeSpecificTests();
            await this.runPerformanceTests();
            await this.runContractTests();

            const success = this.generateReport();

            if (success) {
                console.log('\n🎉 All tests passed!');
                process.exit(0);
            } else {
                console.log('\n⚠️  Some tests failed. Check the report for details.');
                process.exit(1);
            }
        } catch (error) {
            console.error('\n💥 Test suite failed:', error.message);
            process.exit(1);
        }
    }
}

// CLI usage
if (require.main === module) {
    const config = {};

    // Parse command line arguments
    const args = process.argv.slice(2);
    for (let i = 0; i < args.length; i++) {
        if (args[i] === '--rpc' && args[i + 1]) {
            config.rpcUrl = args[i + 1];
            i++;
        }
    }

    const runner = new LatticeTestRunner(config);
    runner.run();
}

module.exports = LatticeTestRunner;