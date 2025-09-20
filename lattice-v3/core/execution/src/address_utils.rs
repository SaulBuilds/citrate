/// Address utilities for handling both Ethereum-style addresses and public keys
use crate::types::Address;
use lattice_consensus::types::PublicKey;

/// Convert various input formats to a proper 20-byte Ethereum address
pub fn normalize_address(input: &PublicKey) -> Address {
    let bytes = input.as_bytes();
    
    // Check if this is a 20-byte address padded with zeros
    // (common when sending to Ethereum addresses from wallets)
    if is_padded_address(bytes) {
        // Extract the 20-byte address from the first 20 bytes
        let mut addr = [0u8; 20];
        addr.copy_from_slice(&bytes[0..20]);
        return Address(addr);
    }
    
    // Otherwise, derive address from public key using standard Ethereum method
    derive_address_from_pubkey(input)
}

/// Check if a 32-byte array contains a 20-byte address padded with zeros
fn is_padded_address(bytes: &[u8; 32]) -> bool {
    // Treat 32-byte input as an embedded 20-byte EVM address when
    // the last 12 bytes are zero and the first 20 are not all zero.
    bytes[20..].iter().all(|&b| b == 0) && !bytes[..20].iter().all(|&b| b == 0)
}

/// Derive Ethereum address from public key using keccak256
fn derive_address_from_pubkey(pubkey: &PublicKey) -> Address {
    use sha3::{Digest, Keccak256};
    
    // For actual public keys, we need the uncompressed form
    // This is simplified - in production, handle key formats properly
    let mut hasher = Keccak256::new();
    hasher.update(pubkey.as_bytes());
    let hash = hasher.finalize();
    
    // Take last 20 bytes of hash as address
    let mut addr = [0u8; 20];
    addr.copy_from_slice(&hash[12..32]);
    Address(addr)
}

/// Convert a hex string (with or without 0x prefix) to Address
pub fn address_from_hex(hex: &str) -> Result<Address, String> {
    let hex = hex.trim_start_matches("0x").trim_start_matches("0X");
    
    if hex.len() != 40 {
        return Err(format!("Invalid address length: expected 40 hex chars, got {}", hex.len()));
    }
    
    let bytes = hex::decode(hex)
        .map_err(|e| format!("Invalid hex: {}", e))?;
    
    if bytes.len() != 20 {
        return Err(format!("Invalid decoded length: expected 20 bytes, got {}", bytes.len()));
    }
    
    let mut addr = [0u8; 20];
    addr.copy_from_slice(&bytes);
    Ok(Address(addr))
}

/// Convert Address to a 32-byte PublicKey format for compatibility
pub fn address_to_pubkey_format(addr: &Address) -> PublicKey {
    let mut bytes = [0u8; 32];
    bytes[0..20].copy_from_slice(&addr.0);
    // Last 12 bytes remain zero to indicate this is an address, not a pubkey
    PublicKey::new(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_padded_address_detection() {
        // Test with padded address
        let mut padded = [0u8; 32];
        padded[0..20].copy_from_slice(&[
            0x74, 0x2d, 0x35, 0xcc, 0x66, 0x34, 0xc0, 0x53,
            0x29, 0x25, 0xa3, 0xb8, 0x44, 0xbc, 0x9e, 0x75,
            0x95, 0xf0, 0xbe, 0xb1
        ]);
        
        let pubkey = PublicKey::new(padded);
        let addr = normalize_address(&pubkey);
        
        assert_eq!(&addr.0[..], &padded[0..20]);
    }
    
    #[test]
    fn test_real_pubkey_handling() {
        // Test with non-zero bytes in last 12 positions (real pubkey)
        let pubkey_bytes = [0x55u8; 32]; // All non-zero
        let pubkey = PublicKey::new(pubkey_bytes);
        let addr = normalize_address(&pubkey);
        
        // Should derive address from pubkey, not just take first 20 bytes
        assert_ne!(&addr.0[..], &pubkey_bytes[0..20]);
    }
    
    #[test]
    fn test_hex_conversion() {
        let hex_with_prefix = "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb1";
        let hex_without = "742d35Cc6634C0532925a3b844Bc9e7595f0bEb1";
        
        let addr1 = address_from_hex(hex_with_prefix).unwrap();
        let addr2 = address_from_hex(hex_without).unwrap();
        
        assert_eq!(addr1, addr2);
    }
}
