const { ethers } = require('ethers');

async function testContracts() {
    // Use explicit IPv4 address to avoid IPv6 issues
    const provider = new ethers.JsonRpcProvider('http://127.0.0.1:8545');
    
    // Check network
    const network = await provider.getNetwork();
    console.log('Connected to network:', network.chainId);
    
    // Get latest block
    const block = await provider.getBlock('latest');
    console.log('Latest block:', block.number);
    console.log('Block hash:', block.hash);
    
    // Test RPC methods
    const accounts = ['0x3333333333333333333333333333333333333333'];
    for (const account of accounts) {
        const balance = await provider.getBalance(account);
        console.log(`Balance of ${account}: ${ethers.formatEther(balance)} ETH`);
    }
}

testContracts().catch(console.error);
