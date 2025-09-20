use crate::errors::WalletError;
use ed25519_dalek::SigningKey;
use lattice_consensus::types::{Hash, PublicKey, Signature, Transaction};
use lattice_consensus::crypto as consensus_crypto;
use lattice_execution::types::Address;
use primitive_types::U256;
use sha3::{Digest, Keccak256};
use serde::{Deserialize, Serialize};

/// Signed transaction ready for broadcast
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignedTransaction {
    pub transaction: Transaction,
    pub raw: Vec<u8>,
}

/// Transaction builder
pub struct TransactionBuilder {
    from: Option<PublicKey>,
    to: Option<Address>,
    value: U256,
    data: Vec<u8>,
    nonce: u64,
    gas_price: u64,
    gas_limit: u64,
    chain_id: u64,
}

impl TransactionBuilder {
    /// Create new transaction builder
    pub fn new() -> Self {
        Self {
            from: None,
            to: None,
            value: U256::zero(),
            data: Vec::new(),
            nonce: 0,
            gas_price: 1_000_000_000, // 1 gwei default
            gas_limit: 21_000,        // Standard transfer
            chain_id: 1337,           // Default testnet
        }
    }
    
    /// Set sender
    pub fn from(mut self, from: PublicKey) -> Self {
        self.from = Some(from);
        self
    }
    
    /// Set recipient
    pub fn to(mut self, to: Option<Address>) -> Self {
        self.to = to;
        self
    }
    
    /// Set value in wei
    pub fn value(mut self, value: U256) -> Self {
        self.value = value;
        self
    }
    
    /// Set data
    pub fn data(mut self, data: Vec<u8>) -> Self {
        self.data = data;
        self
    }
    
    /// Set nonce
    pub fn nonce(mut self, nonce: u64) -> Self {
        self.nonce = nonce;
        self
    }
    
    /// Set gas price
    pub fn gas_price(mut self, gas_price: u64) -> Self {
        self.gas_price = gas_price;
        self
    }
    
    /// Set gas limit
    pub fn gas_limit(mut self, gas_limit: u64) -> Self {
        self.gas_limit = gas_limit;
        self
    }
    
    /// Set chain ID
    pub fn chain_id(mut self, chain_id: u64) -> Self {
        self.chain_id = chain_id;
        self
    }
    
    /// Build and sign transaction
    pub fn build_and_sign(self, signing_key: &SigningKey) -> Result<SignedTransaction, WalletError> {
        let from = self.from.ok_or_else(|| WalletError::Other("From address not set".to_string()))?;
        
        // Convert to address to PublicKey if set (for transaction format)
        let to_pubkey = self.to.map(|addr| {
            // Create a pseudo public key from address for compatibility
            // In production, this would be resolved from address book or chain state
            let mut pk_bytes = [0u8; 32];
            pk_bytes[..20].copy_from_slice(&addr.0);
            PublicKey::new(pk_bytes)
        });
        
        // Create unsigned transaction
        let mut tx = Transaction {
            hash: Hash::default(), // Will be calculated
            from,
            to: to_pubkey,
            value: value_to_u128(self.value),
            data: self.data,
            nonce: self.nonce,
            gas_price: self.gas_price,
            gas_limit: self.gas_limit,
            signature: Signature::new([0; 64]), // Will be replaced
            tx_type: None, // Will be determined if needed
        };

        // Calculate transaction hash (UI/display). Consensus verification uses canonical bytes.
        tx.hash = calculate_tx_hash(&tx, self.chain_id);

        // Sign canonical transaction bytes using consensus crypto so mempool verification passes
        consensus_crypto::sign_transaction(&mut tx, signing_key)
            .map_err(|e| WalletError::Other(format!("Transaction signing failed: {}", e)))?;

        // Serialize for raw format
        let raw = bincode::serialize(&tx)?;
        
        Ok(SignedTransaction {
            transaction: tx,
            raw,
        })
    }
}

impl Default for TransactionBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Calculate transaction hash including chain ID (EIP-155 style)
fn calculate_tx_hash(tx: &Transaction, chain_id: u64) -> Hash {
    // Use Keccak-256 to align with Ethereum-style hashing across the stack
    let mut hasher = Keccak256::new();
    
    // Hash transaction fields
    hasher.update(tx.nonce.to_le_bytes());
    hasher.update(tx.gas_price.to_le_bytes());
    hasher.update(tx.gas_limit.to_le_bytes());
    
    if let Some(to) = &tx.to {
        hasher.update(to.as_bytes());
    }
    
    hasher.update(tx.value.to_le_bytes());
    hasher.update(&tx.data);
    
    // Include chain ID for replay protection
    hasher.update(chain_id.to_le_bytes());
    hasher.update([0u8; 8]); // r placeholder
    hasher.update([0u8; 8]); // s placeholder
    
    let hash_bytes = hasher.finalize();
    let mut hash_array = [0u8; 32];
    hash_array.copy_from_slice(&hash_bytes);
    
    Hash::new(hash_array)
}

/// Convert U256 to u128 (with overflow check)
fn value_to_u128(value: U256) -> u128 {
    // Check if value fits in u128
    if value > U256::from(u128::MAX) {
        // Saturate at max
        u128::MAX
    } else {
        value.as_u128()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::SigningKey;
    
    #[test]
    fn test_transaction_builder() {
        use rand::RngCore;
        let mut secret_bytes = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut secret_bytes);
        let signing_key = SigningKey::from_bytes(&secret_bytes);
        let public_key = PublicKey::new(signing_key.verifying_key().to_bytes());
        
        let tx = TransactionBuilder::new()
            .from(public_key)
            .to(Some(Address([0x11; 20])))
            .value(U256::from(1000))
            .nonce(0)
            .gas_price(1_000_000_000)
            .gas_limit(21_000)
            .chain_id(1337)
            .build_and_sign(&signing_key)
            .unwrap();
        
        assert_eq!(tx.transaction.from, public_key);
        assert_eq!(tx.transaction.value, 1000);
        assert_eq!(tx.transaction.nonce, 0);
    }
}
