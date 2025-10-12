// lattice-v3/core/execution/src/address_utils.rs

/// Address utilities for handling both Ethereum-style addresses and public keys
use crate::types::Address;
use lattice_consensus::types::PublicKey;

/// Convert various input formats to a proper 20-byte Ethereum address
/// This function delegates to Address::from_public_key to ensure consistency
pub fn normalize_address(input: &PublicKey) -> Address {
    Address::from_public_key(input)
}


/// Convert a hex string (with or without 0x prefix) to Address
pub fn address_from_hex(hex: &str) -> Result<Address, String> {
    let hex = hex.trim_start_matches("0x").trim_start_matches("0X");

    if hex.len() != 40 {
        return Err(format!(
            "Invalid address length: expected 40 hex chars, got {}",
            hex.len()
        ));
    }

    let bytes = hex::decode(hex).map_err(|e| format!("Invalid hex: {}", e))?;

    if bytes.len() != 20 {
        return Err(format!(
            "Invalid decoded length: expected 20 bytes, got {}",
            bytes.len()
        ));
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
    fn test_address_derivation_consistency() {
        // Test with embedded EVM address (first 20 bytes non-zero, last 12 bytes zero)
        let mut padded = [0u8; 32];
        padded[0..20].copy_from_slice(&[
            0x74, 0x2d, 0x35, 0xcc, 0x66, 0x34, 0xc0, 0x53, 0x29, 0x25, 0xa3, 0xb8, 0x44, 0xbc,
            0x9e, 0x75, 0x95, 0xf0, 0xbe, 0xb1,
        ]);

        let pubkey = PublicKey::new(padded);
        let addr_normalize = normalize_address(&pubkey);
        let addr_direct = Address::from_public_key(&pubkey);

        // Both methods should produce the same result
        assert_eq!(addr_normalize, addr_direct);
        assert_eq!(&addr_normalize.0[..], &padded[0..20]);
    }

    #[test]
    fn test_real_pubkey_consistency() {
        // Test with non-zero bytes in last 12 positions (real pubkey)
        let pubkey_bytes = [0x55u8; 32]; // All non-zero
        let pubkey = PublicKey::new(pubkey_bytes);

        let addr_normalize = normalize_address(&pubkey);
        let addr_direct = Address::from_public_key(&pubkey);

        // Both methods should produce the same result
        assert_eq!(addr_normalize, addr_direct);
        // Should derive address from pubkey, not just take first 20 bytes
        assert_ne!(&addr_normalize.0[..], &pubkey_bytes[0..20]);
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
