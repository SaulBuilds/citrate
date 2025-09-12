const { ethers } = require('ethers');

async function sendTestTransaction() {
    // Connect to the local node
    const provider = new ethers.JsonRpcProvider('http://localhost:8545');
    
    // Create a wallet with a test private key
    const privateKey = '0x0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef';
    const wallet = new ethers.Wallet(privateKey, provider);
    
    console.log('Wallet address:', wallet.address);
    
    // Create transaction
    const tx = {
        to: '0x3333333333333333333333333333333333333333',
        value: ethers.parseEther('0.1'),
        gasLimit: 21000,
        gasPrice: ethers.parseUnits('10', 'gwei'),
        nonce: 0,
        chainId: 1337
    };
    
    try {
        // Sign and send transaction
        const signedTx = await wallet.signTransaction(tx);
        console.log('Signed transaction:', signedTx);
        
        // Send raw transaction
        const response = await provider.send('eth_sendRawTransaction', [signedTx]);
        console.log('Transaction hash:', response);
    } catch (error) {
        console.error('Error:', error.message);
    }
}

sendTestTransaction();