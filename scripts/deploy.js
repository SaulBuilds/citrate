const { ethers } = require('ethers');
const fs = require('fs');

async function deploy() {
    // Use explicit IPv4 address to avoid IPv6 issues
    const provider = new ethers.JsonRpcProvider('http://127.0.0.1:8545');
    
    // Test private key (DO NOT USE IN PRODUCTION)
    const privateKey = '0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80';
    const wallet = new ethers.Wallet(privateKey, provider);
    
    console.log('Deployer address:', wallet.address);
    
    try {
        // Get balance
        const balance = await provider.getBalance(wallet.address);
        console.log('Deployer balance:', ethers.formatEther(balance), 'ETH');
        
        // SimpleStorage bytecode (simplified example)
        const bytecode = '0x608060405234801561001057600080fd5b5060405161016f38038061016f8339818101604052810190610032919061007a565b80600081905550336000806101000a81548173ffffffffffffffffffffffffffffffffffffffff021916908373ffffffffffffffffffffffffffffffffffffffff16021790555050610007565b600080fd5b6000819050919050565b61008f81610086565b811461009a57600080fd5b50565b6000815190506100ac81610090565b92915050565b6000602082840312156100c8576100c7610081565b5b60006100d68482850161009d565b9150509291505056fe';
        
        // Deploy transaction
        const deployTx = {
            data: bytecode,
            gasLimit: 1000000,
            gasPrice: ethers.parseUnits('10', 'gwei')
        };
        
        console.log('\nSending deployment transaction...');
        const tx = await wallet.sendTransaction(deployTx);
        console.log('Transaction hash:', tx.hash);
        
        const receipt = await tx.wait();
        console.log('Contract deployed at:', receipt.contractAddress);
        
    } catch (error) {
        console.error('Deployment error:', error.message);
        console.log('\nNote: Transaction validation is still being implemented.');
        console.log('The network is generating proper hashes but signature validation needs work.');
    }
}

deploy();
