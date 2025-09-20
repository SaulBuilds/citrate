use crate::types::{Transaction, PublicKey, Signature};
use ed25519_dalek::{Verifier, SigningKey, VerifyingKey, Signature as DalekSignature, Signer};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CryptoError {
    #[error("Invalid public key")]
    InvalidPublicKey,
    
    #[error("Invalid signature")]
    InvalidSignature,
    
    #[error("Signature verification failed")]
    VerificationFailed,
    
    #[error("Serialization error: {0}")]
    SerializationError(String),
}

/// Verify a transaction's signature
pub fn verify_transaction(tx: &Transaction) -> Result<bool, CryptoError> {
    // Get canonical bytes to verify (everything except signature)
    let message = canonical_tx_bytes(tx)?;
    
    // Convert our types to ed25519-dalek types
    let public_key = VerifyingKey::from_bytes(tx.from.as_bytes())
        .map_err(|_| CryptoError::InvalidPublicKey)?;
    
    let signature = DalekSignature::from_bytes(tx.signature.as_bytes());
    
    // Verify the signature
    match public_key.verify(&message, &signature) {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
}

/// Sign a transaction (for testing and dev tools)
pub fn sign_transaction(tx: &mut Transaction, signing_key: &SigningKey) -> Result<(), CryptoError> {
    // Ensure `from` matches the signing key before computing canonical bytes
    tx.from = PublicKey::new(signing_key.verifying_key().to_bytes());

    // Get canonical bytes to sign (now includes correct `from`)
    let message = canonical_tx_bytes(tx)?;

    // Sign the message
    let signature: DalekSignature = signing_key.sign(&message);

    // Update signature in transaction
    tx.signature = Signature::new(signature.to_bytes());
    
    Ok(())
}

/// Get canonical bytes for transaction signing/verification
/// This excludes the signature field and uses a deterministic encoding
fn canonical_tx_bytes(tx: &Transaction) -> Result<Vec<u8>, CryptoError> {
    let mut data = Vec::new();
    
    // Fixed-size fields first (exclude tx.hash to avoid circular dependency)
    data.extend_from_slice(&tx.nonce.to_le_bytes());
    data.extend_from_slice(tx.from.as_bytes());
    
    // Optional to field
    if let Some(to) = &tx.to {
        data.push(1); // Present flag
        data.extend_from_slice(to.as_bytes());
    } else {
        data.push(0); // Absent flag
    }
    
    // Value and gas fields
    data.extend_from_slice(&tx.value.to_le_bytes());
    data.extend_from_slice(&tx.gas_limit.to_le_bytes());
    data.extend_from_slice(&tx.gas_price.to_le_bytes());
    
    // Variable-length data field
    data.extend_from_slice(&(tx.data.len() as u32).to_le_bytes());
    data.extend_from_slice(&tx.data);
    
    Ok(data)
}

/// Generate a new keypair for testing
pub fn generate_keypair() -> SigningKey {
    SigningKey::from_bytes(&rand::random())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Hash;
    
    #[test]
    fn test_transaction_signing_and_verification() {
        // Generate a keypair
        let signing_key = generate_keypair();
        
        // Create a transaction
        let mut tx = Transaction {
            hash: Hash::new([1; 32]),
            nonce: 1,
            from: PublicKey::new([0; 32]), // Will be updated by sign
            to: Some(PublicKey::new([2; 32])),
            value: 1000,
            gas_limit: 21000,
            gas_price: 1_000_000_000,
            data: vec![1, 2, 3],
            signature: Signature::new([0; 64]), // Will be updated by sign
            tx_type: None,
        };
        
        // Sign it
        sign_transaction(&mut tx, &signing_key).unwrap();
        
        // Verify it
        assert!(verify_transaction(&tx).unwrap());
        
        // Tamper with it
        tx.value = 2000;
        
        // Should fail verification
        assert!(!verify_transaction(&tx).unwrap());
    }
    
    #[test]
    fn test_canonical_bytes_deterministic() {
        let tx = Transaction {
            hash: Hash::new([1; 32]),
            nonce: 42,
            from: PublicKey::new([3; 32]),
            to: Some(PublicKey::new([4; 32])),
            value: 1000,
            gas_limit: 21000,
            gas_price: 1_000_000_000,
            data: vec![5, 6, 7],
            signature: Signature::new([8; 64]),
            tx_type: None,
        };
        
        // Should produce same bytes every time
        let bytes1 = canonical_tx_bytes(&tx).unwrap();
        let bytes2 = canonical_tx_bytes(&tx).unwrap();
        assert_eq!(bytes1, bytes2);
    }
}
