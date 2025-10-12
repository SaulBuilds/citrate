#!/usr/bin/env node

const { ethers } = require('ethers');

// AI Operation Types (matching our executor opcodes)
const AI_OPS = {
    MODEL_DEPLOY: 0x01000000,  // Register a new model
    MODEL_UPDATE: 0x02000000,  // Update model weights
    INFERENCE_REQUEST: 0x03000000,  // Request inference
    TRAINING_JOB: 0x04000000,  // Create training job
};

async function main() {
    // Connect to local Lattice node
    const provider = new ethers.JsonRpcProvider('http://localhost:8545');
    
    // Test account with funds from genesis
    const privateKey = '0x0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef';
    const wallet = new ethers.Wallet(privateKey, provider);
    
    console.log('Wallet address:', wallet.address);
    
    // Check balance
    const balance = await provider.getBalance(wallet.address);
    console.log('Balance:', ethers.formatEther(balance), 'LATT');
    
    // Get current block
    const currentBlock = await provider.getBlock('latest');
    console.log('Current block:', currentBlock.number);
    console.log('State root:', currentBlock.stateRoot);
    
    // Test 1: Register AI Model
    console.log('\n=== Test 1: Register AI Model ===');
    
    // Create model registration data
    // Format: [opcode(4)] [model_hash(32)] [metadata_len(4)] [metadata]
    const modelHash = ethers.hexlify(ethers.randomBytes(32));
    const metadata = {
        name: "GPT-Test",
        version: "1.0",
        framework: "PyTorch",
        size_bytes: 1000000
    };
    const metadataBytes = ethers.toUtf8Bytes(JSON.stringify(metadata));
    
    // Build transaction data
    const txData = ethers.concat([
        ethers.toBeHex(AI_OPS.MODEL_DEPLOY, 4),  // Opcode
        modelHash,  // Model hash
        ethers.toBeHex(metadataBytes.length, 4),  // Metadata length
        metadataBytes  // Metadata
    ]);
    
    console.log('Sending model registration transaction...');
    console.log('Model hash:', modelHash);
    console.log('Transaction data length:', txData.length, 'bytes');
    
    try {
        const tx = await wallet.sendTransaction({
            to: '0x0000000000000000000000000000000000000001',  // AI precompile address
            data: txData,
            gasLimit: 100000,
            gasPrice: ethers.parseUnits('1', 'gwei')
        });
        
        console.log('Transaction hash:', tx.hash);
        
        // Wait for confirmation
        const receipt = await tx.wait();
        console.log('Transaction confirmed in block:', receipt.blockNumber);
        console.log('Gas used:', receipt.gasUsed.toString());
        
        // Check new state root
        const newBlock = await provider.getBlock(receipt.blockNumber);
        console.log('New state root:', newBlock.stateRoot);
        
    } catch (error) {
        console.error('Transaction failed:', error.message);
    }
    
    // Test 2: Inference Request
    console.log('\n=== Test 2: Inference Request ===');
    
    // Create inference request data
    // Format: [opcode(4)] [model_hash(32)] [input_len(4)] [input]
    const input = ethers.toUtf8Bytes("Hello, AI!");
    const inferenceData = ethers.concat([
        ethers.toBeHex(AI_OPS.INFERENCE_REQUEST, 4),  // Opcode
        modelHash,  // Model hash (use same model)
        ethers.toBeHex(input.length, 4),  // Input length
        input  // Input data
    ]);
    
    console.log('Sending inference request...');
    
    try {
        const tx2 = await wallet.sendTransaction({
            to: '0x0000000000000000000000000000000000000001',  // AI precompile address
            data: inferenceData,
            gasLimit: 50000,
            gasPrice: ethers.parseUnits('1', 'gwei')
        });
        
        console.log('Transaction hash:', tx2.hash);
        
        const receipt2 = await tx2.wait();
        console.log('Transaction confirmed in block:', receipt2.blockNumber);
        console.log('Gas used:', receipt2.gasUsed.toString());
        
        // Check state root change
        const finalBlock = await provider.getBlock(receipt2.blockNumber);
        console.log('Final state root:', finalBlock.stateRoot);
        
    } catch (error) {
        console.error('Inference request failed:', error.message);
    }
    
    // Test 3: Check DAG structure
    console.log('\n=== Test 3: Check DAG Structure ===');
    
    const latestBlock = await provider.getBlock('latest');
    console.log('Latest block number:', latestBlock.number);
    console.log('Latest block hash:', latestBlock.hash);
    console.log('Parent hash:', latestBlock.parentHash);
    
    // Get chain of blocks
    console.log('\nBlock chain (last 5 blocks):');
    for (let i = 0; i < 5 && latestBlock.number - i >= 0; i++) {
        const block = await provider.getBlock(latestBlock.number - i);
        console.log(`  Block ${block.number}: ${block.hash.substring(0, 10)}... (state: ${block.stateRoot.substring(0, 10)}...)`);
    }
}

main().catch(console.error);