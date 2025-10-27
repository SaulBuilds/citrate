import { AxiosInstance } from 'axios';
import { ethers } from 'ethers';
import { CitrateConfig } from './types';

export class AccountManager {
  private wallet?: ethers.Wallet | ethers.HDNodeWallet;
  
  constructor(
    private rpcClient: AxiosInstance,
    private config: CitrateConfig
  ) {}
  
  /**
   * Create a new account
   */
  createAccount(): {
    address: string;
    privateKey: string;
    mnemonic?: string;
  } {
    const wallet = ethers.Wallet.createRandom();
    
    return {
      address: wallet.address,
      privateKey: wallet.privateKey,
      mnemonic: wallet.mnemonic?.phrase
    };
  }
  
  /**
   * Import account from private key
   */
  importAccount(privateKey: string): string {
    const wallet = new ethers.Wallet(privateKey);
    this.wallet = wallet;
    this.config.defaultAccount = wallet.address;
    
    return wallet.address;
  }
  
  /**
   * Import account from mnemonic
   */
  importFromMnemonic(
    mnemonic: string,
    path: string = "m/44'/60'/0'/0/0"
  ): string {
    // In ethers v6, derive via HDNodeWallet and then derivePath
    const hd = ethers.HDNodeWallet.fromPhrase(mnemonic);
    const wallet = path ? hd.derivePath(path) : hd;
    this.wallet = wallet as ethers.HDNodeWallet;
    this.config.defaultAccount = wallet.address;
    
    return wallet.address;
  }
  
  /**
   * Get account balance
   */
  async getBalance(address?: string): Promise<bigint> {
    const addr = address || this.config.defaultAccount;
    if (!addr) {
      throw new Error('No address specified');
    }
    
    const response = await this.rpcCall('eth_getBalance', [addr, 'latest']);
    return BigInt(response);
  }
  
  /**
   * Get account nonce
   */
  async getNonce(address?: string): Promise<number> {
    const addr = address || this.config.defaultAccount;
    if (!addr) {
      throw new Error('No address specified');
    }
    
    const response = await this.rpcCall('eth_getTransactionCount', [addr, 'latest']);
    return parseInt(response, 16);
  }
  
  /**
   * Send transaction
   */
  async sendTransaction(tx: {
    to: string;
    value?: string;
    data?: string;
    gasLimit?: number;
    gasPrice?: string;
  }): Promise<string> {
    if (!this.config.defaultAccount) {
      throw new Error('No default account set');
    }
    
    const nonce = await this.getNonce();
    
    const txData = {
      from: this.config.defaultAccount,
      to: tx.to,
      value: tx.value || '0x0',
      data: tx.data || '0x',
      gas: `0x${(tx.gasLimit || this.config.gasLimit || 21000).toString(16)}`,
      gasPrice: tx.gasPrice || this.config.gasPrice || '0x3b9aca00',
      nonce: `0x${nonce.toString(16)}`
    };
    
    // If we have a wallet, sign the transaction
    if (this.wallet) {
      return await this.sendSignedTransaction(txData);
    }
    
    // Otherwise, send unsigned (requires unlocked account on node)
    return await this.rpcCall('eth_sendTransaction', [txData]);
  }
  
  /**
   * Sign and send transaction
   */
  private async sendSignedTransaction(txData: any): Promise<string> {
    if (!this.wallet) {
      throw new Error('No wallet available for signing');
    }
    
    // Create transaction object for ethers
    const tx = {
      to: txData.to,
      value: txData.value,
      data: txData.data,
      gasLimit: txData.gas,
      gasPrice: txData.gasPrice,
      nonce: parseInt(txData.nonce, 16),
      chainId: this.config.chainId
    };
    
    // Sign transaction
    const signedTx = await this.wallet.signTransaction(tx);
    
    // Send raw transaction
    return await this.rpcCall('eth_sendRawTransaction', [signedTx]);
  }
  
  /**
   * Sign message
   */
  async signMessage(message: string): Promise<string> {
    if (!this.wallet) {
      throw new Error('No wallet available for signing');
    }
    
    return await this.wallet.signMessage(message);
  }
  
  /**
   * Verify message signature
   */
  verifyMessage(
    message: string,
    signature: string,
    address: string
  ): boolean {
    const recoveredAddress = ethers.verifyMessage(message, signature);
    return recoveredAddress.toLowerCase() === address.toLowerCase();
  }
  
  /**
   * List accounts (from node)
   */
  async listAccounts(): Promise<string[]> {
    return await this.rpcCall('eth_accounts');
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
}
