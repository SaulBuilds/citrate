// citrate/core/api/src/eip1559_decoder.rs

use ethereum_types::{H160, H256, U256};
use citrate_consensus::types::{Hash, PublicKey, Signature, Transaction};
use rlp::{DecoderError, Rlp, RlpStream};
use secp256k1::{ecdsa::RecoverableSignature, ecdsa::RecoveryId, Message, Secp256k1};
use sha3::{Digest, Keccak256};
use std::collections::HashMap;
use thiserror::Error;
use tracing::{debug, info, warn};

/// Enhanced EIP-1559 transaction decoder with comprehensive validation
#[derive(Debug, Clone)]
pub struct Eip1559Decoder {
    supported_chain_ids: Vec<u64>,
    max_fee_per_gas_cap: U256,
    max_priority_fee_cap: U256,
    secp: Secp256k1<secp256k1::All>,
}

/// EIP-1559 transaction structure
#[derive(Debug, Clone)]
pub struct Eip1559Transaction {
    pub chain_id: u64,
    pub nonce: u64,
    pub max_priority_fee_per_gas: U256,
    pub max_fee_per_gas: U256,
    pub gas_limit: u64,
    pub to: Option<H160>,
    pub value: U256,
    pub data: Vec<u8>,
    pub access_list: Vec<AccessListEntry>,
    pub y_parity: u8,
    pub r: H256,
    pub s: H256,
}

/// Access list entry for EIP-2930/1559 transactions
#[derive(Debug, Clone)]
pub struct AccessListEntry {
    pub address: H160,
    pub storage_keys: Vec<H256>,
}

/// Transaction validation result
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<ValidationWarning>,
    pub gas_estimation: GasEstimation,
}

#[derive(Debug, Clone)]
pub struct GasEstimation {
    pub base_fee_recommended: U256,
    pub priority_fee_recommended: U256,
    pub total_cost_estimate: U256,
    pub access_list_gas_saved: u64,
}

#[derive(Debug, Clone)]
pub enum ValidationError {
    InvalidChainId(u64),
    InvalidSignature,
    ExcessiveGasFees,
    MalformedAccessList,
    InvalidRecoveryId,
    GasLimitTooLow,
    GasLimitTooHigh,
    ZeroGasPrice,
    InvalidNonce,
}

#[derive(Debug, Clone)]
pub enum ValidationWarning {
    HighGasFees,
    LargeAccessList,
    LargeTransactionData,
    UnusualGasLimit,
}

#[derive(Error, Debug)]
pub enum Eip1559Error {
    #[error("RLP decoding error: {0}")]
    RlpError(#[from] DecoderError),

    #[error("Invalid transaction format: {0}")]
    InvalidFormat(String),

    #[error("Signature recovery failed: {0}")]
    SignatureError(String),

    #[error("Chain ID not supported: {0}")]
    UnsupportedChainId(u64),

    #[error("Gas fees exceed maximum: max_fee={0}, cap={1}")]
    ExcessiveGasFees(U256, U256),

    #[error("Invalid access list: {0}")]
    InvalidAccessList(String),

    #[error("Validation failed: {0:?}")]
    ValidationFailed(Vec<ValidationError>),
}

impl Eip1559Decoder {
    /// Create a new EIP-1559 decoder with configuration
    pub fn new(supported_chain_ids: Vec<u64>) -> Self {
        Self {
            supported_chain_ids,
            max_fee_per_gas_cap: U256::from(1000u64) * U256::exp10(9), // 1000 gwei
            max_priority_fee_cap: U256::from(100u64) * U256::exp10(9), // 100 gwei
            secp: Secp256k1::new(),
        }
    }

    /// Decode and validate EIP-1559 transaction
    pub fn decode_transaction(&self, tx_bytes: &[u8]) -> Result<Transaction, Eip1559Error> {
        // Verify transaction type prefix
        if tx_bytes.is_empty() || tx_bytes[0] != 0x02 {
            return Err(Eip1559Error::InvalidFormat(
                "Transaction must start with 0x02 for EIP-1559".to_string()
            ));
        }

        // Parse RLP payload
        let rlp_payload = &tx_bytes[1..];
        let eip1559_tx = self.parse_eip1559_rlp(rlp_payload)?;

        // Validate transaction
        let validation = self.validate_transaction(&eip1559_tx)?;
        if !validation.is_valid {
            return Err(Eip1559Error::ValidationFailed(validation.errors));
        }

        // Log warnings if any
        for warning in &validation.warnings {
            warn!("EIP-1559 transaction warning: {:?}", warning);
        }

        // Recover sender address
        let sender = self.recover_sender(&eip1559_tx, rlp_payload)?;

        // Convert to Citrate transaction format
        let citrate_tx = self.convert_to_citrate_transaction(eip1559_tx, sender, tx_bytes)?;

        info!(
            "Successfully decoded EIP-1559 transaction: hash=0x{}, from=0x{}, nonce={}",
            hex::encode(citrate_tx.hash.as_bytes()),
            hex::encode(&sender.as_bytes()),
            citrate_tx.nonce
        );

        Ok(citrate_tx)
    }

    /// Parse EIP-1559 RLP structure
    fn parse_eip1559_rlp(&self, rlp_bytes: &[u8]) -> Result<Eip1559Transaction, Eip1559Error> {
        let rlp = Rlp::new(rlp_bytes);

        if !rlp.is_list() || rlp.item_count()? != 12 {
            return Err(Eip1559Error::InvalidFormat(
                "EIP-1559 transaction must have exactly 12 fields".to_string()
            ));
        }

        let chain_id_u256: U256 = rlp.val_at(0)?;
        let chain_id = if chain_id_u256 > U256::from(u64::MAX) {
            return Err(Eip1559Error::InvalidFormat(
                "Chain ID too large for u64".to_string()
            ));
        } else {
            chain_id_u256.as_u64()
        };

        let nonce: u64 = rlp.val_at(1)?;
        let max_priority_fee_per_gas: U256 = rlp.val_at(2)?;
        let max_fee_per_gas: U256 = rlp.val_at(3)?;
        let gas_limit: u64 = rlp.val_at(4)?;

        // Parse 'to' field
        let to = {
            let to_bytes: Vec<u8> = rlp.val_at(5)?;
            if to_bytes.is_empty() {
                None
            } else if to_bytes.len() != 20 {
                return Err(Eip1559Error::InvalidFormat(
                    "Invalid 'to' address length".to_string()
                ));
            } else {
                Some(H160::from_slice(&to_bytes))
            }
        };

        let value: U256 = rlp.val_at(6)?;
        let data: Vec<u8> = rlp.val_at(7)?;

        // Parse access list
        let access_list = self.parse_access_list_from_rlp(&rlp, 8)?;

        let y_parity_u64: u64 = rlp.val_at(9)?;
        let y_parity = if y_parity_u64 > 1 {
            return Err(Eip1559Error::InvalidFormat(
                "y_parity must be 0 or 1".to_string()
            ));
        } else {
            y_parity_u64 as u8
        };

        let r: H256 = rlp.val_at(10)?;
        let s: H256 = rlp.val_at(11)?;

        Ok(Eip1559Transaction {
            chain_id,
            nonce,
            max_priority_fee_per_gas,
            max_fee_per_gas,
            gas_limit,
            to,
            value,
            data,
            access_list,
            y_parity,
            r,
            s,
        })
    }

    /// Parse access list from RLP
    fn parse_access_list_from_rlp(&self, rlp: &Rlp, index: usize) -> Result<Vec<AccessListEntry>, Eip1559Error> {
        let access_list_rlp = rlp.at(index)?;
        let mut access_list = Vec::new();

        for i in 0..access_list_rlp.item_count().unwrap_or(0) {
            let entry_rlp = access_list_rlp.at(i)?;

            if entry_rlp.item_count()? != 2 {
                return Err(Eip1559Error::InvalidAccessList(
                    format!("Access list entry {} must have exactly 2 fields", i)
                ));
            }

            let address_bytes: Vec<u8> = entry_rlp.val_at(0)?;
            if address_bytes.len() != 20 {
                return Err(Eip1559Error::InvalidAccessList(
                    format!("Invalid address length in access list entry {}", i)
                ));
            }
            let address = H160::from_slice(&address_bytes);

            let storage_keys_rlp = entry_rlp.at(1)?;
            let mut storage_keys = Vec::new();

            for j in 0..storage_keys_rlp.item_count().unwrap_or(0) {
                let key_bytes: Vec<u8> = storage_keys_rlp.val_at(j)?;
                if key_bytes.len() != 32 {
                    return Err(Eip1559Error::InvalidAccessList(
                        format!("Invalid storage key length in access list entry {}, key {}", i, j)
                    ));
                }
                storage_keys.push(H256::from_slice(&key_bytes));
            }

            access_list.push(AccessListEntry {
                address,
                storage_keys,
            });
        }

        Ok(access_list)
    }

    /// Validate EIP-1559 transaction
    fn validate_transaction(&self, tx: &Eip1559Transaction) -> Result<ValidationResult, Eip1559Error> {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        // Validate chain ID
        if !self.supported_chain_ids.contains(&tx.chain_id) {
            errors.push(ValidationError::InvalidChainId(tx.chain_id));
        }

        // Validate gas fees
        if tx.max_fee_per_gas > self.max_fee_per_gas_cap {
            errors.push(ValidationError::ExcessiveGasFees);
        } else if tx.max_fee_per_gas > U256::from(50u64) * U256::exp10(9) { // 50 gwei
            warnings.push(ValidationWarning::HighGasFees);
        }

        if tx.max_priority_fee_per_gas > self.max_priority_fee_cap {
            errors.push(ValidationError::ExcessiveGasFees);
        }

        if tx.max_fee_per_gas < tx.max_priority_fee_per_gas {
            errors.push(ValidationError::ExcessiveGasFees);
        }

        if tx.max_fee_per_gas.is_zero() {
            errors.push(ValidationError::ZeroGasPrice);
        }

        // Validate gas limit
        if tx.gas_limit < 21000 {
            errors.push(ValidationError::GasLimitTooLow);
        }

        if tx.gas_limit > 30_000_000 {
            errors.push(ValidationError::GasLimitTooHigh);
        } else if tx.gas_limit > 10_000_000 {
            warnings.push(ValidationWarning::UnusualGasLimit);
        }

        // Validate signature components
        if tx.r.is_zero() || tx.s.is_zero() {
            errors.push(ValidationError::InvalidSignature);
        }

        // Validate access list
        if tx.access_list.len() > 100 {
            warnings.push(ValidationWarning::LargeAccessList);
        }

        // Validate transaction data size
        if tx.data.len() > 1_000_000 {
            warnings.push(ValidationWarning::LargeTransactionData);
        }

        // Estimate gas costs
        let gas_estimation = self.estimate_gas_costs(tx);

        Ok(ValidationResult {
            is_valid: errors.is_empty(),
            errors,
            warnings,
            gas_estimation,
        })
    }

    /// Estimate gas costs for transaction
    fn estimate_gas_costs(&self, tx: &Eip1559Transaction) -> GasEstimation {
        // Simplified gas estimation
        let base_fee_recommended = U256::from(20u64) * U256::exp10(9); // 20 gwei
        let priority_fee_recommended = U256::from(2u64) * U256::exp10(9); // 2 gwei

        let access_list_gas_saved = tx.access_list.iter()
            .map(|entry| 2400 + entry.storage_keys.len() as u64 * 1900)
            .sum::<u64>();

        let total_cost_estimate = tx.max_fee_per_gas * U256::from(tx.gas_limit);

        GasEstimation {
            base_fee_recommended,
            priority_fee_recommended,
            total_cost_estimate,
            access_list_gas_saved,
        }
    }

    /// Recover sender address from signature
    fn recover_sender(&self, tx: &Eip1559Transaction, _rlp_payload: &[u8]) -> Result<H160, Eip1559Error> {
        // Build signing payload (transaction without signature)
        let signing_payload = self.build_signing_payload(tx)?;

        // Calculate typed transaction hash
        let mut hasher = Keccak256::new();
        hasher.update([0x02]); // EIP-1559 type prefix
        hasher.update(&signing_payload);
        let sighash = hasher.finalize();

        // Recover public key from signature
        let recovery_id = RecoveryId::from_i32(tx.y_parity as i32)
            .map_err(|e| Eip1559Error::SignatureError(format!("Invalid recovery ID: {}", e)))?;

        let mut rs_bytes = [0u8; 64];
        rs_bytes[..32].copy_from_slice(tx.r.as_bytes());
        rs_bytes[32..].copy_from_slice(tx.s.as_bytes());

        let recoverable_sig = RecoverableSignature::from_compact(&rs_bytes, recovery_id)
            .map_err(|e| Eip1559Error::SignatureError(format!("Invalid signature: {}", e)))?;

        let message = Message::from_slice(&sighash)
            .map_err(|e| Eip1559Error::SignatureError(format!("Invalid message: {}", e)))?;

        let public_key = self.secp.recover_ecdsa(&message, &recoverable_sig)
            .map_err(|e| Eip1559Error::SignatureError(format!("Recovery failed: {}", e)))?;

        // Convert public key to Ethereum address
        let uncompressed = public_key.serialize_uncompressed();
        let mut addr_hasher = Keccak256::new();
        addr_hasher.update(&uncompressed[1..]); // Skip 0x04 prefix
        let addr_hash = addr_hasher.finalize();

        let mut addr_bytes = [0u8; 20];
        addr_bytes.copy_from_slice(&addr_hash[12..]);

        Ok(H160::from_slice(&addr_bytes))
    }

    /// Build signing payload for EIP-1559 transaction
    fn build_signing_payload(&self, tx: &Eip1559Transaction) -> Result<Vec<u8>, Eip1559Error> {
        let mut stream = RlpStream::new_list(9);

        stream.append(&tx.chain_id);
        stream.append(&tx.nonce);
        stream.append(&tx.max_priority_fee_per_gas);
        stream.append(&tx.max_fee_per_gas);
        stream.append(&tx.gas_limit);

        // Handle 'to' field
        if let Some(to) = tx.to {
            stream.append(&to.as_bytes());
        } else {
            stream.append_empty_data();
        }

        stream.append(&tx.value);
        stream.append(&tx.data.as_slice());

        // Encode access list
        stream.begin_list(tx.access_list.len());
        for entry in &tx.access_list {
            stream.begin_list(2);
            stream.append(&entry.address.as_bytes());
            stream.begin_list(entry.storage_keys.len());
            for key in &entry.storage_keys {
                stream.append(&key.as_bytes());
            }
        }

        Ok(stream.out().to_vec())
    }

    /// Convert EIP-1559 transaction to Citrate transaction format
    fn convert_to_citrate_transaction(
        &self,
        tx: Eip1559Transaction,
        sender: H160,
        original_bytes: &[u8],
    ) -> Result<Transaction, Eip1559Error> {
        // Calculate transaction hash from original bytes
        let mut hasher = Keccak256::new();
        hasher.update(original_bytes);
        let mut hash_bytes = [0u8; 32];
        hash_bytes.copy_from_slice(&hasher.finalize());

        // Convert addresses to Citrate PublicKey format (embed 20 bytes in 32-byte field)
        let mut from_pk_bytes = [0u8; 32];
        from_pk_bytes[..20].copy_from_slice(sender.as_bytes());
        let from_pk = PublicKey::new(from_pk_bytes);

        let to_pk = tx.to.map(|addr| {
            let mut pk_bytes = [0u8; 32];
            pk_bytes[..20].copy_from_slice(addr.as_bytes());
            PublicKey::new(pk_bytes)
        });

        // Convert gas price (use max_fee_per_gas)
        let gas_price = if tx.max_fee_per_gas > U256::from(u64::MAX) {
            u64::MAX
        } else {
            tx.max_fee_per_gas.as_u64()
        };

        // Convert value
        let value = if tx.value > U256::from(u128::MAX) {
            u128::MAX
        } else {
            tx.value.as_u128()
        };

        // Create signature from r, s components
        let mut sig_bytes = [0u8; 64];
        sig_bytes[..32].copy_from_slice(tx.r.as_bytes());
        sig_bytes[32..].copy_from_slice(tx.s.as_bytes());

        let mut citrate_tx = Transaction {
            hash: Hash::new(hash_bytes),
            from: from_pk,
            to: to_pk,
            value,
            data: tx.data,
            nonce: tx.nonce,
            gas_price,
            gas_limit: tx.gas_limit,
            signature: Signature::new(sig_bytes),
            tx_type: None,
        };

        // Determine transaction type from data
        citrate_tx.determine_type();

        debug!(
            "Converted EIP-1559 transaction: hash=0x{}, gas_price={}, gas_limit={}",
            hex::encode(citrate_tx.hash.as_bytes()),
            citrate_tx.gas_price,
            citrate_tx.gas_limit
        );

        Ok(citrate_tx)
    }
}

/// Transaction statistics for monitoring
#[derive(Debug, Clone, Default)]
pub struct TransactionStats {
    pub total_decoded: u64,
    pub eip1559_count: u64,
    pub eip2930_count: u64,
    pub legacy_count: u64,
    pub failed_decodes: u64,
    pub invalid_signatures: u64,
    pub chain_id_distribution: HashMap<u64, u64>,
}

impl TransactionStats {
    pub fn record_successful_decode(&mut self, chain_id: u64, tx_type: &str) {
        self.total_decoded += 1;
        match tx_type {
            "eip1559" => self.eip1559_count += 1,
            "eip2930" => self.eip2930_count += 1,
            "legacy" => self.legacy_count += 1,
            _ => {}
        }
        *self.chain_id_distribution.entry(chain_id).or_insert(0) += 1;
    }

    pub fn record_failed_decode(&mut self, reason: &str) {
        self.failed_decodes += 1;
        if reason.contains("signature") {
            self.invalid_signatures += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_eip1559_decoder_creation() {
        let decoder = Eip1559Decoder::new(vec![1, 5, 1337]);
        assert_eq!(decoder.supported_chain_ids, vec![1, 5, 1337]);
    }

    #[test]
    fn test_invalid_transaction_type() {
        let decoder = Eip1559Decoder::new(vec![1]);
        let result = decoder.decode_transaction(&[0x01, 0x02, 0x03]); // Wrong type prefix
        assert!(result.is_err());
    }

    #[test]
    fn test_empty_transaction() {
        let decoder = Eip1559Decoder::new(vec![1]);
        let result = decoder.decode_transaction(&[]);
        assert!(result.is_err());
    }
}