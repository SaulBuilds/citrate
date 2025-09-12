#!/usr/bin/env node

const { ethers } = require('ethers');

async function main() {
    // Connect to local Lattice node (use IPv4 explicitly)
    const provider = new ethers.JsonRpcProvider('http://127.0.0.1:8545');
    
    // Get initial state
    console.log('=== Initial Chain State ===');
    const initialBlock = await provider.getBlock('latest');
    console.log('Block number:', initialBlock.number);
    console.log('State root:', initialBlock.stateRoot);
    console.log('Block hash:', initialBlock.hash);
    
    // Create a simple transaction with AI opcode data
    // Using MODEL_LOAD opcode (0xf1) as a test
    const aiData = ethers.concat([
        '0xf1',  // MODEL_LOAD opcode
        ethers.hexlify(ethers.randomBytes(32)),  // Model hash
    ]);
    
    console.log('\n=== Sending AI Transaction ===');
    console.log('AI operation data:', aiData);
    
    // Use a test private key
    const privateKey = '0x0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef';
    const wallet = new ethers.Wallet(privateKey, provider);
    
    try {
        // Send transaction with AI data
        const tx = await wallet.sendTransaction({
            to: '0x3333333333333333333333333333333333333333',  // Test recipient
            value: ethers.parseEther('0.1'),
            data: aiData,
            gasLimit: 100000,
            gasPrice: ethers.parseUnits('1', 'gwei')
        });
        
        console.log('Transaction hash:', tx.hash);
        console.log('Waiting for confirmation...');
        
        const receipt = await tx.wait();
        console.log('Confirmed in block:', receipt.blockNumber);
        
        // Get new state
        console.log('\n=== New Chain State ===');
        const newBlock = await provider.getBlock(receipt.blockNumber);
        console.log('Block number:', newBlock.number);
        console.log('State root:', newBlock.stateRoot);
        console.log('Block hash:', newBlock.hash);
        
        // Compare state roots
        console.log('\n=== State Root Comparison ===');
        console.log('Initial state root:', initialBlock.stateRoot);
        console.log('New state root:    ', newBlock.stateRoot);
        console.log('State root changed:', initialBlock.stateRoot !== newBlock.stateRoot);
        
    } catch (error) {
        console.error('Transaction failed:', error.message);
        
        // Try a simpler transaction without AI data
        console.log('\n=== Trying simple transfer ===');
        const simpleTx = await wallet.sendTransaction({
            to: '0x3333333333333333333333333333333333333333',
            value: ethers.parseEther('0.1'),
            gasLimit: 21000,
            gasPrice: ethers.parseUnits('1', 'gwei')
        });
        
        console.log('Simple tx hash:', simpleTx.hash);
        const simpleReceipt = await simpleTx.wait();
        console.log('Confirmed in block:', simpleReceipt.blockNumber);
        
        const finalBlock = await provider.getBlock(simpleReceipt.blockNumber);
        console.log('Final state root:', finalBlock.stateRoot);
    }
}

main().catch(console.error);