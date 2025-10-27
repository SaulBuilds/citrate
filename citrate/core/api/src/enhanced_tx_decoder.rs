// citrate/core/api/src/enhanced_tx_decoder.rs

use crate::eip1559_decoder::{Eip1559Decoder, TransactionStats};
use ethereum_types::{H160, H256, U256 as EthU256};
use citrate_consensus::types::{Hash, PublicKey, Signature, Transaction};
use rlp::{DecoderError, Rlp, RlpStream};
use secp256k1::{ecdsa::RecoverableSignature, ecdsa::RecoveryId, Message, Secp256k1};
use sha3::{Digest, Keccak256};
use std::sync::{Arc, Mutex};
use thiserror::Error;
use tracing::{debug, error};

/// Enhanced transaction decoder supporting multiple transaction types
pub struct EnhancedTransactionDecoder {
    eip1559_decoder: Eip1559Decoder,
    supported_chain_ids: Vec<u64>,
    secp: Secp256k1<secp256k1::All>,
    stats: Arc<Mutex<TransactionStats>>,
    config: DecoderConfig,
}

/// Decoder configuration
#[derive(Debug, Clone)]
pub struct DecoderConfig {
    pub enable_legacy_support: bool,
    pub enable_eip2930_support: bool,
    pub enable_eip1559_support: bool,
    pub strict_validation: bool,
    pub max_transaction_size: usize,
    pub default_chain_id: u64,
}

impl Default for DecoderConfig {
    fn default() -> Self {
        Self {
            enable_legacy_support: true,
            enable_eip2930_support: true,
            enable_eip1559_support: true,
            strict_validation: true,
            max_transaction_size: 1_048_576, // 1MB
            default_chain_id: 1,
        }
    }
}

/// Transaction type enumeration
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransactionType {
    Legacy,
    Eip2930, // Access list transactions
    Eip1559, // Fee market transactions
    CitrateNative,
}

/// Enhanced transaction result with metadata
#[derive(Debug, Clone)]
pub struct DecodedTransaction {
    pub transaction: Transaction,
    pub tx_type: TransactionType,
    pub chain_id: Option<u64>,
    pub sender: H160,
    pub effective_gas_price: u64,
    pub access_list_size: usize,
    pub validation_warnings: Vec<String>,
}

#[derive(Error, Debug)]
pub enum TransactionDecoderError {
    #[error("Transaction too large: {size} bytes (max: {max})")]
    TransactionTooLarge { size: usize, max: usize },

    #[error("Unsupported transaction type: {type_byte:#x}")]
    UnsupportedTransactionType { type_byte: u8 },

    #[error("Transaction type disabled: {tx_type:?}")]
    TransactionTypeDisabled { tx_type: TransactionType },

    #[error("RLP decoding failed: {0}")]
    RlpDecodingFailed(#[from] DecoderError),

    #[error("Signature verification failed: {0}")]
    SignatureVerificationFailed(String),

    #[error("Invalid chain ID: {chain_id}")]
    InvalidChainId { chain_id: u64 },

    #[error("Malformed transaction: {0}")]
    MalformedTransaction(String),

    #[error("EIP-1559 decoding failed: {0}")]
    Eip1559Error(String),

    #[error("Legacy transaction decoding failed: {0}")]
    LegacyError(String),

    #[error("Validation failed: {errors:?}")]
    ValidationFailed { errors: Vec<String> },
}

/// Legacy transaction structure
#[derive(Debug, Clone)]
struct LegacyTransaction {
    nonce: u64,
    gas_price: EthU256,
    gas_limit: u64,
    to: Option<H160>,
    value: EthU256,
    data: Vec<u8>,
    v: u64,
    r: H256,
    s: H256,
}

/// EIP-2930 transaction structure
#[derive(Debug, Clone)]
struct Eip2930Transaction {
    chain_id: u64,
    nonce: u64,
    gas_price: EthU256,
    gas_limit: u64,
    to: Option<H160>,
    value: EthU256,
    data: Vec<u8>,
    access_list: Vec<AccessListEntry>,
    y_parity: u8,
    r: H256,
    s: H256,
}

#[derive(Debug, Clone)]
struct AccessListEntry {
    address: H160,
    storage_keys: Vec<H256>,
}

impl EnhancedTransactionDecoder {
    /// Create a new enhanced transaction decoder
    pub fn new(supported_chain_ids: Vec<u64>, config: DecoderConfig) -> Self {
        let eip1559_decoder = Eip1559Decoder::new(supported_chain_ids.clone());

        Self {
            eip1559_decoder,
            supported_chain_ids,
            secp: Secp256k1::new(),
            stats: Arc::new(Mutex::new(TransactionStats::default())),
            config,
        }
    }

    /// Decode transaction from raw bytes
    pub fn decode_transaction(&self, tx_bytes: &[u8]) -> Result<DecodedTransaction, TransactionDecoderError> {
        // Validate transaction size
        if tx_bytes.len() > self.config.max_transaction_size {
            return Err(TransactionDecoderError::TransactionTooLarge {
                size: tx_bytes.len(),
                max: self.config.max_transaction_size,
            });
        }

        if tx_bytes.is_empty() {
            return Err(TransactionDecoderError::MalformedTransaction(
                "Empty transaction data".to_string()
            ));
        }

        debug!("Decoding transaction: {} bytes", tx_bytes.len());

        // Try Citrate native format first
        if let Ok(tx) = bincode::deserialize::<Transaction>(tx_bytes) {
            return self.handle_citrate_native_transaction(tx);
        }

        // Determine transaction type from first byte
        let tx_type = match tx_bytes[0] {
            0x02 => TransactionType::Eip1559,
            0x01 => TransactionType::Eip2930,
            _ => TransactionType::Legacy,
        };

        // Check if transaction type is enabled
        match tx_type {
            TransactionType::Eip1559 if !self.config.enable_eip1559_support => {
                return Err(TransactionDecoderError::TransactionTypeDisabled { tx_type });
            }
            TransactionType::Eip2930 if !self.config.enable_eip2930_support => {
                return Err(TransactionDecoderError::TransactionTypeDisabled { tx_type });
            }
            TransactionType::Legacy if !self.config.enable_legacy_support => {
                return Err(TransactionDecoderError::TransactionTypeDisabled { tx_type });
            }
            _ => {}
        }

        // Decode based on transaction type
        let result = match tx_type {
            TransactionType::Eip1559 => self.decode_eip1559_transaction(tx_bytes),
            TransactionType::Eip2930 => self.decode_eip2930_transaction(tx_bytes),
            TransactionType::Legacy => self.decode_legacy_transaction(tx_bytes),
            TransactionType::CitrateNative => unreachable!("Already handled above"),
        };

        // Update statistics
        match &result {
            Ok(decoded) => {
                let mut stats = self.stats.lock().unwrap();
                let tx_type_str = match decoded.tx_type {
                    TransactionType::Eip1559 => "eip1559",
                    TransactionType::Eip2930 => "eip2930",
                    TransactionType::Legacy => "legacy",
                    TransactionType::CitrateNative => "citrate_native",
                };
                stats.record_successful_decode(
                    decoded.chain_id.unwrap_or(self.config.default_chain_id),
                    tx_type_str
                );
            }
            Err(e) => {
                let mut stats = self.stats.lock().unwrap();
                stats.record_failed_decode(&e.to_string());
            }
        }

        result
    }

    /// Decode EIP-1559 transaction
    fn decode_eip1559_transaction(&self, tx_bytes: &[u8]) -> Result<DecodedTransaction, TransactionDecoderError> {
        let citrate_tx = self.eip1559_decoder.decode_transaction(tx_bytes)
            .map_err(|e| TransactionDecoderError::Eip1559Error(e.to_string()))?;

        // Extract additional metadata for EIP-1559 transactions
        let rlp_payload = &tx_bytes[1..];
        let rlp = Rlp::new(rlp_payload);

        let chain_id: u64 = rlp.val_at(0).unwrap_or(self.config.default_chain_id);
        let max_fee_per_gas: EthU256 = rlp.val_at(3).unwrap_or_default();

        // Extract sender from lattice transaction
        let sender = self.extract_sender_from_citrate_tx(&citrate_tx);

        // Parse access list for size calculation
        let access_list_size = self.calculate_access_list_size(&rlp, 8)?;

        let mut warnings = Vec::new();
        if max_fee_per_gas > EthU256::from(100u64) * EthU256::exp10(9) {
            warnings.push("High gas fees detected".to_string());
        }

        Ok(DecodedTransaction {
            transaction: citrate_tx,
            tx_type: TransactionType::Eip1559,
            chain_id: Some(chain_id),
            sender,
            effective_gas_price: max_fee_per_gas.as_u64().min(u64::MAX),
            access_list_size,
            validation_warnings: warnings,
        })
    }

    /// Decode EIP-2930 transaction
    fn decode_eip2930_transaction(&self, tx_bytes: &[u8]) -> Result<DecodedTransaction, TransactionDecoderError> {
        let rlp_payload = &tx_bytes[1..];
        let eip2930_tx = self.parse_eip2930_rlp(rlp_payload)?;

        // Validate chain ID
        if self.config.strict_validation && !self.supported_chain_ids.contains(&eip2930_tx.chain_id) {
            return Err(TransactionDecoderError::InvalidChainId {
                chain_id: eip2930_tx.chain_id,
            });
        }

        // Recover sender
        let sender = self.recover_eip2930_sender(&eip2930_tx, rlp_payload)?;

        // Convert to Citrate transaction
        let citrate_tx = self.convert_eip2930_to_citrate(eip2930_tx.clone(), sender, tx_bytes)?;

        let mut warnings = Vec::new();
        if eip2930_tx.access_list.len() > 10 {
            warnings.push("Large access list detected".to_string());
        }

        Ok(DecodedTransaction {
            transaction: citrate_tx,
            tx_type: TransactionType::Eip2930,
            chain_id: Some(eip2930_tx.chain_id),
            sender,
            effective_gas_price: eip2930_tx.gas_price.as_u64().min(u64::MAX),
            access_list_size: eip2930_tx.access_list.len(),
            validation_warnings: warnings,
        })
    }

    /// Decode legacy transaction
    fn decode_legacy_transaction(&self, tx_bytes: &[u8]) -> Result<DecodedTransaction, TransactionDecoderError> {
        let rlp = Rlp::new(tx_bytes);
        let legacy_tx = self.parse_legacy_rlp(&rlp)?;

        // Determine chain ID and recovery ID from v value
        let (_recovery_id, chain_id) = self.extract_chain_id_from_v(legacy_tx.v)?;

        // Validate chain ID if present
        if let Some(cid) = chain_id {
            if self.config.strict_validation && !self.supported_chain_ids.contains(&cid) {
                return Err(TransactionDecoderError::InvalidChainId { chain_id: cid });
            }
        }

        // Recover sender
        let sender = self.recover_legacy_sender(&legacy_tx, chain_id)?;

        // Convert to Citrate transaction
        let citrate_tx = self.convert_legacy_to_citrate(legacy_tx.clone(), sender, tx_bytes)?;

        let mut warnings = Vec::new();
        if chain_id.is_none() {
            warnings.push("Pre-EIP-155 transaction without replay protection".to_string());
        }

        Ok(DecodedTransaction {
            transaction: citrate_tx,
            tx_type: TransactionType::Legacy,
            chain_id,
            sender,
            effective_gas_price: legacy_tx.gas_price.as_u64().min(u64::MAX),
            access_list_size: 0,
            validation_warnings: warnings,
        })
    }

    /// Handle Citrate native transaction
    fn handle_citrate_native_transaction(&self, mut tx: Transaction) -> Result<DecodedTransaction, TransactionDecoderError> {
        // Ensure transaction has a proper hash if missing
        if tx.hash == Hash::default() {
            let tx_bytes = bincode::serialize(&tx).unwrap_or_default();
            let mut hasher = Keccak256::new();
            hasher.update(&tx_bytes);
            let mut hash_bytes = [0u8; 32];
            hash_bytes.copy_from_slice(&hasher.finalize());
            tx.hash = Hash::new(hash_bytes);
        }

        // Extract sender from PublicKey
        let sender = self.extract_sender_from_citrate_tx(&tx);

        let mut stats = self.stats.lock().unwrap();
        stats.record_successful_decode(self.config.default_chain_id, "citrate_native");

        let gas_price = tx.gas_price; // Extract before moving tx
        Ok(DecodedTransaction {
            transaction: tx,
            tx_type: TransactionType::CitrateNative,
            chain_id: Some(self.config.default_chain_id),
            sender,
            effective_gas_price: gas_price,
            access_list_size: 0,
            validation_warnings: Vec::new(),
        })
    }

    // Helper methods for parsing different transaction types

    fn parse_legacy_rlp(&self, rlp: &Rlp) -> Result<LegacyTransaction, TransactionDecoderError> {
        if !rlp.is_list() || rlp.item_count()? != 9 {
            return Err(TransactionDecoderError::MalformedTransaction(
                "Legacy transaction must have exactly 9 fields".to_string()
            ));
        }

        Ok(LegacyTransaction {
            nonce: rlp.val_at(0)?,
            gas_price: rlp.val_at(1)?,
            gas_limit: rlp.val_at(2)?,
            to: {
                let to_bytes: Vec<u8> = rlp.val_at(3)?;
                if to_bytes.is_empty() {
                    None
                } else if to_bytes.len() != 20 {
                    return Err(TransactionDecoderError::MalformedTransaction(
                        "Invalid 'to' address length".to_string()
                    ));
                } else {
                    Some(H160::from_slice(&to_bytes))
                }
            },
            value: rlp.val_at(4)?,
            data: rlp.val_at(5)?,
            v: rlp.val_at(6)?,
            r: rlp.val_at(7)?,
            s: rlp.val_at(8)?,
        })
    }

    fn parse_eip2930_rlp(&self, rlp_bytes: &[u8]) -> Result<Eip2930Transaction, TransactionDecoderError> {
        let rlp = Rlp::new(rlp_bytes);

        if !rlp.is_list() || rlp.item_count()? != 11 {
            return Err(TransactionDecoderError::MalformedTransaction(
                "EIP-2930 transaction must have exactly 11 fields".to_string()
            ));
        }

        let chain_id_u256: EthU256 = rlp.val_at(0)?;
        let chain_id = if chain_id_u256 > EthU256::from(u64::MAX) {
            return Err(TransactionDecoderError::MalformedTransaction(
                "Chain ID too large".to_string()
            ));
        } else {
            chain_id_u256.as_u64()
        };

        let access_list = self.parse_access_list_from_rlp(&rlp, 7)?;

        let y_parity_u64: u64 = rlp.val_at(8)?;
        let y_parity = if y_parity_u64 > 1 {
            return Err(TransactionDecoderError::MalformedTransaction(
                "y_parity must be 0 or 1".to_string()
            ));
        } else {
            y_parity_u64 as u8
        };

        Ok(Eip2930Transaction {
            chain_id,
            nonce: rlp.val_at(1)?,
            gas_price: rlp.val_at(2)?,
            gas_limit: rlp.val_at(3)?,
            to: {
                let to_bytes: Vec<u8> = rlp.val_at(4)?;
                if to_bytes.is_empty() {
                    None
                } else if to_bytes.len() != 20 {
                    return Err(TransactionDecoderError::MalformedTransaction(
                        "Invalid 'to' address length".to_string()
                    ));
                } else {
                    Some(H160::from_slice(&to_bytes))
                }
            },
            value: rlp.val_at(5)?,
            data: rlp.val_at(6)?,
            access_list,
            y_parity,
            r: rlp.val_at(9)?,
            s: rlp.val_at(10)?,
        })
    }

    fn parse_access_list_from_rlp(&self, rlp: &Rlp, index: usize) -> Result<Vec<AccessListEntry>, TransactionDecoderError> {
        let access_list_rlp = rlp.at(index)?;
        let mut access_list = Vec::new();

        for i in 0..access_list_rlp.item_count().unwrap_or(0) {
            let entry_rlp = access_list_rlp.at(i)?;

            if entry_rlp.item_count()? != 2 {
                return Err(TransactionDecoderError::MalformedTransaction(
                    format!("Access list entry {} must have exactly 2 fields", i)
                ));
            }

            let address_bytes: Vec<u8> = entry_rlp.val_at(0)?;
            if address_bytes.len() != 20 {
                return Err(TransactionDecoderError::MalformedTransaction(
                    "Invalid address in access list".to_string()
                ));
            }

            let storage_keys_rlp = entry_rlp.at(1)?;
            let mut storage_keys = Vec::new();

            for j in 0..storage_keys_rlp.item_count().unwrap_or(0) {
                let key_bytes: Vec<u8> = storage_keys_rlp.val_at(j)?;
                if key_bytes.len() != 32 {
                    return Err(TransactionDecoderError::MalformedTransaction(
                        "Invalid storage key in access list".to_string()
                    ));
                }
                storage_keys.push(H256::from_slice(&key_bytes));
            }

            access_list.push(AccessListEntry {
                address: H160::from_slice(&address_bytes),
                storage_keys,
            });
        }

        Ok(access_list)
    }

    fn extract_chain_id_from_v(&self, v: u64) -> Result<(i32, Option<u64>), TransactionDecoderError> {
        if v >= 35 {
            // EIP-155 transaction
            let chain_id = (v - 35) / 2;
            let recovery_id = ((v - 35) % 2) as i32;
            Ok((recovery_id, Some(chain_id)))
        } else if v == 27 || v == 28 {
            // Pre-EIP-155 transaction
            let recovery_id = (v - 27) as i32;
            Ok((recovery_id, None))
        } else {
            Err(TransactionDecoderError::MalformedTransaction(
                format!("Invalid v value: {}", v)
            ))
        }
    }

    // Additional helper methods for signature recovery and conversion...

    fn recover_legacy_sender(&self, tx: &LegacyTransaction, chain_id: Option<u64>) -> Result<H160, TransactionDecoderError> {
        // Build signing payload
        let signing_payload = self.build_legacy_signing_payload(tx, chain_id)?;
        let sighash = Keccak256::digest(&signing_payload);

        // Recover address
        self.recover_address_from_signature(&tx.r, &tx.s, &sighash, ((tx.v - if chain_id.is_some() { 35 + 2 * chain_id.unwrap() } else { 27 }) % 2) as i32)
    }

    fn recover_eip2930_sender(&self, tx: &Eip2930Transaction, _rlp_payload: &[u8]) -> Result<H160, TransactionDecoderError> {
        // Build signing payload
        let signing_payload = self.build_eip2930_signing_payload(tx)?;

        // Calculate typed transaction hash
        let mut hasher = Keccak256::new();
        hasher.update([0x01]); // EIP-2930 type prefix
        hasher.update(&signing_payload);
        let sighash = hasher.finalize();

        self.recover_address_from_signature(&tx.r, &tx.s, &sighash, tx.y_parity as i32)
    }

    fn recover_address_from_signature(&self, r: &H256, s: &H256, sighash: &[u8], recovery_id: i32) -> Result<H160, TransactionDecoderError> {
        let recid = RecoveryId::from_i32(recovery_id)
            .map_err(|e| TransactionDecoderError::SignatureVerificationFailed(format!("Invalid recovery ID: {}", e)))?;

        let mut rs_bytes = [0u8; 64];
        rs_bytes[..32].copy_from_slice(r.as_bytes());
        rs_bytes[32..].copy_from_slice(s.as_bytes());

        let recoverable_sig = RecoverableSignature::from_compact(&rs_bytes, recid)
            .map_err(|e| TransactionDecoderError::SignatureVerificationFailed(format!("Invalid signature: {}", e)))?;

        let message = Message::from_slice(sighash)
            .map_err(|e| TransactionDecoderError::SignatureVerificationFailed(format!("Invalid message: {}", e)))?;

        let public_key = self.secp.recover_ecdsa(&message, &recoverable_sig)
            .map_err(|e| TransactionDecoderError::SignatureVerificationFailed(format!("Recovery failed: {}", e)))?;

        // Convert to Ethereum address
        let uncompressed = public_key.serialize_uncompressed();
        let mut addr_hasher = Keccak256::new();
        addr_hasher.update(&uncompressed[1..]);
        let addr_hash = addr_hasher.finalize();

        let mut addr_bytes = [0u8; 20];
        addr_bytes.copy_from_slice(&addr_hash[12..]);

        Ok(H160::from_slice(&addr_bytes))
    }

    fn build_legacy_signing_payload(&self, tx: &LegacyTransaction, chain_id: Option<u64>) -> Result<Vec<u8>, TransactionDecoderError> {
        let mut stream = if chain_id.is_some() {
            RlpStream::new_list(9)
        } else {
            RlpStream::new_list(6)
        };

        stream.append(&tx.nonce);
        stream.append(&tx.gas_price);
        stream.append(&tx.gas_limit);

        if let Some(to) = tx.to {
            stream.append(&to.as_bytes());
        } else {
            stream.append_empty_data();
        }

        stream.append(&tx.value);
        stream.append(&tx.data);

        if let Some(cid) = chain_id {
            stream.append(&cid);
            stream.append(&0u8);
            stream.append(&0u8);
        }

        Ok(stream.out().to_vec())
    }

    fn build_eip2930_signing_payload(&self, tx: &Eip2930Transaction) -> Result<Vec<u8>, TransactionDecoderError> {
        let mut stream = RlpStream::new_list(8);

        stream.append(&tx.chain_id);
        stream.append(&tx.nonce);
        stream.append(&tx.gas_price);
        stream.append(&tx.gas_limit);

        if let Some(to) = tx.to {
            stream.append(&to.as_bytes());
        } else {
            stream.append_empty_data();
        }

        stream.append(&tx.value);
        stream.append(&tx.data);

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

    fn convert_legacy_to_citrate(&self, tx: LegacyTransaction, sender: H160, original_bytes: &[u8]) -> Result<Transaction, TransactionDecoderError> {
        let hash_bytes = self.calculate_transaction_hash(original_bytes);

        let mut from_pk_bytes = [0u8; 32];
        from_pk_bytes[..20].copy_from_slice(sender.as_bytes());
        let from_pk = PublicKey::new(from_pk_bytes);

        let to_pk = tx.to.map(|addr| {
            let mut pk_bytes = [0u8; 32];
            pk_bytes[..20].copy_from_slice(addr.as_bytes());
            PublicKey::new(pk_bytes)
        });

        let gas_price = tx.gas_price.as_u64().min(u64::MAX);
        let value = tx.value.as_u128().min(u128::MAX);

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

        citrate_tx.determine_type();
        Ok(citrate_tx)
    }

    fn convert_eip2930_to_citrate(&self, tx: Eip2930Transaction, sender: H160, original_bytes: &[u8]) -> Result<Transaction, TransactionDecoderError> {
        let hash_bytes = self.calculate_transaction_hash(original_bytes);

        let mut from_pk_bytes = [0u8; 32];
        from_pk_bytes[..20].copy_from_slice(sender.as_bytes());
        let from_pk = PublicKey::new(from_pk_bytes);

        let to_pk = tx.to.map(|addr| {
            let mut pk_bytes = [0u8; 32];
            pk_bytes[..20].copy_from_slice(addr.as_bytes());
            PublicKey::new(pk_bytes)
        });

        let gas_price = tx.gas_price.as_u64().min(u64::MAX);
        let value = tx.value.as_u128().min(u128::MAX);

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

        citrate_tx.determine_type();
        Ok(citrate_tx)
    }

    fn calculate_transaction_hash(&self, tx_bytes: &[u8]) -> [u8; 32] {
        let mut hasher = Keccak256::new();
        hasher.update(tx_bytes);
        let mut hash_bytes = [0u8; 32];
        hash_bytes.copy_from_slice(&hasher.finalize());
        hash_bytes
    }

    fn extract_sender_from_citrate_tx(&self, tx: &Transaction) -> H160 {
        let pk_bytes = tx.from.as_bytes();
        H160::from_slice(&pk_bytes[..20])
    }

    fn calculate_access_list_size(&self, rlp: &Rlp, index: usize) -> Result<usize, TransactionDecoderError> {
        let access_list_rlp = rlp.at(index)?;
        Ok(access_list_rlp.item_count().unwrap_or(0))
    }

    /// Get decoder statistics
    pub fn get_stats(&self) -> TransactionStats {
        self.stats.lock().unwrap().clone()
    }

    /// Reset statistics
    pub fn reset_stats(&self) {
        let mut stats = self.stats.lock().unwrap();
        *stats = TransactionStats::default();
    }

    /// Get supported chain IDs
    pub fn get_supported_chain_ids(&self) -> &[u64] {
        &self.supported_chain_ids
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decoder_creation() {
        let config = DecoderConfig::default();
        let decoder = EnhancedTransactionDecoder::new(vec![1, 5], config);
        assert_eq!(decoder.supported_chain_ids, vec![1, 5]);
    }

    #[test]
    fn test_transaction_too_large() {
        let config = DecoderConfig {
            max_transaction_size: 100,
            ..Default::default()
        };
        let decoder = EnhancedTransactionDecoder::new(vec![1], config);
        let large_tx = vec![0u8; 200];

        let result = decoder.decode_transaction(&large_tx);
        assert!(matches!(result, Err(TransactionDecoderError::TransactionTooLarge { .. })));
    }

    #[test]
    fn test_empty_transaction() {
        let decoder = EnhancedTransactionDecoder::new(vec![1], DecoderConfig::default());
        let result = decoder.decode_transaction(&[]);
        assert!(result.is_err());
    }
}