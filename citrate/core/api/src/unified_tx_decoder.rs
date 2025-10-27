// citrate/core/api/src/unified_tx_decoder.rs

use crate::enhanced_tx_decoder::{EnhancedTransactionDecoder, DecodedTransaction, DecoderConfig, TransactionDecoderError};
use citrate_consensus::types::Transaction;
use std::sync::Arc;
use tracing::{debug, error, info, warn};

/// Unified transaction decoder that replaces the legacy eth_tx_decoder
/// This provides a single entry point for all transaction decoding
pub struct UnifiedTransactionDecoder {
    enhanced_decoder: EnhancedTransactionDecoder,
    fallback_enabled: bool,
}

impl UnifiedTransactionDecoder {
    /// Create a new unified transaction decoder
    pub fn new(supported_chain_ids: Vec<u64>, fallback_enabled: bool) -> Self {
        let config = DecoderConfig {
            enable_legacy_support: true,
            enable_eip2930_support: true,
            enable_eip1559_support: true,
            strict_validation: false, // Allow some flexibility for compatibility
            max_transaction_size: 10_485_760, // 10MB for large contract deployments
            default_chain_id: supported_chain_ids.first().copied().unwrap_or(1),
        };

        let enhanced_decoder = EnhancedTransactionDecoder::new(supported_chain_ids, config);

        Self {
            enhanced_decoder,
            fallback_enabled,
        }
    }

    /// Decode transaction from raw bytes with comprehensive error handling
    pub fn decode_eth_transaction(&self, tx_bytes: &[u8]) -> Result<Transaction, String> {
        info!(
            "Decoding transaction: {} bytes, first_byte=0x{:02x}",
            tx_bytes.len(),
            tx_bytes.first().unwrap_or(&0)
        );

        // Use the enhanced decoder first
        match self.enhanced_decoder.decode_transaction(tx_bytes) {
            Ok(decoded_tx) => {
                info!(
                    "Successfully decoded {} transaction: hash=0x{}, from=0x{}, nonce={}",
                    format!("{:?}", decoded_tx.tx_type).to_lowercase(),
                    hex::encode(decoded_tx.transaction.hash.as_bytes()),
                    hex::encode(decoded_tx.sender.as_bytes()),
                    decoded_tx.transaction.nonce
                );

                // Log any validation warnings
                for warning in &decoded_tx.validation_warnings {
                    warn!("Transaction validation warning: {}", warning);
                }

                // Log transaction details for debugging
                debug!(
                    "Transaction details: gas_price={}, gas_limit={}, value={}, data_size={}",
                    decoded_tx.effective_gas_price,
                    decoded_tx.transaction.gas_limit,
                    decoded_tx.transaction.value,
                    decoded_tx.transaction.data.len()
                );

                Ok(decoded_tx.transaction)
            }
            Err(e) => {
                error!("Enhanced decoder failed: {}", e);

                // If fallback is enabled, try the legacy decoder
                if self.fallback_enabled {
                    warn!("Attempting fallback to legacy decoder");
                    match self.fallback_decode(tx_bytes) {
                        Ok(tx) => {
                            warn!("Fallback decoder succeeded for transaction: 0x{}", hex::encode(tx.hash.as_bytes()));
                            Ok(tx)
                        }
                        Err(fallback_err) => {
                            error!("Both enhanced and fallback decoders failed");
                            Err(format!("Primary: {}; Fallback: {}", e, fallback_err))
                        }
                    }
                } else {
                    Err(e.to_string())
                }
            }
        }
    }

    /// Get detailed transaction information including metadata
    pub fn decode_transaction_with_metadata(&self, tx_bytes: &[u8]) -> Result<DecodedTransaction, TransactionDecoderError> {
        self.enhanced_decoder.decode_transaction(tx_bytes)
    }

    /// Fallback decoder using the original implementation for compatibility
    fn fallback_decode(&self, tx_bytes: &[u8]) -> Result<Transaction, String> {
        // Import the legacy decoder function
        crate::eth_tx_decoder::decode_eth_transaction(tx_bytes)
    }

    /// Get decoder statistics
    pub fn get_stats(&self) -> crate::eip1559_decoder::TransactionStats {
        self.enhanced_decoder.get_stats()
    }

    /// Reset decoder statistics
    pub fn reset_stats(&self) {
        self.enhanced_decoder.reset_stats();
    }

    /// Validate transaction without full decoding (for quick checks)
    pub fn quick_validate(&self, tx_bytes: &[u8]) -> Result<TransactionValidationResult, String> {
        if tx_bytes.is_empty() {
            return Err("Empty transaction data".to_string());
        }

        if tx_bytes.len() > 10_485_760 {
            return Err("Transaction too large".to_string());
        }

        // Check transaction type
        let tx_type = match tx_bytes[0] {
            0x02 => "EIP-1559",
            0x01 => "EIP-2930",
            _ => "Legacy",
        };

        // Try to detect if it's a valid RLP structure for legacy transactions
        if tx_type == "Legacy" {
            let rlp = rlp::Rlp::new(tx_bytes);
            if !rlp.is_list() {
                // Could be Citrate native format
                match bincode::deserialize::<Transaction>(tx_bytes) {
                    Ok(_) => return Ok(TransactionValidationResult {
                        is_valid: true,
                        tx_type: "Citrate Native".to_string(),
                        estimated_gas: None,
                        warnings: Vec::new(),
                    }),
                    Err(_) => return Err("Invalid transaction format".to_string()),
                }
            }
        }

        Ok(TransactionValidationResult {
            is_valid: true,
            tx_type: tx_type.to_string(),
            estimated_gas: None,
            warnings: Vec::new(),
        })
    }
}

/// Quick validation result for fast transaction checks
#[derive(Debug, Clone)]
pub struct TransactionValidationResult {
    pub is_valid: bool,
    pub tx_type: String,
    pub estimated_gas: Option<u64>,
    pub warnings: Vec<String>,
}

/// Global transaction decoder instance for easy access
pub struct GlobalTransactionDecoder {
    decoder: Arc<UnifiedTransactionDecoder>,
}

impl GlobalTransactionDecoder {
    /// Initialize the global decoder with configuration
    pub fn init(supported_chain_ids: Vec<u64>, fallback_enabled: bool) -> Self {
        let decoder = Arc::new(UnifiedTransactionDecoder::new(supported_chain_ids, fallback_enabled));

        info!(
            "Initialized global transaction decoder with chain IDs: {:?}, fallback: {}",
            decoder.enhanced_decoder.get_supported_chain_ids(),
            fallback_enabled
        );

        Self { decoder }
    }

    /// Get a reference to the decoder
    pub fn decoder(&self) -> Arc<UnifiedTransactionDecoder> {
        Arc::clone(&self.decoder)
    }

    /// Decode transaction (convenience method)
    pub fn decode(&self, tx_bytes: &[u8]) -> Result<Transaction, String> {
        self.decoder.decode_eth_transaction(tx_bytes)
    }
}

/// Decoder factory for creating decoders with different configurations
pub struct DecoderFactory;

impl DecoderFactory {
    /// Create a production decoder with comprehensive validation
    pub fn production(supported_chain_ids: Vec<u64>) -> UnifiedTransactionDecoder {
        UnifiedTransactionDecoder::new(supported_chain_ids, false)
    }

    /// Create a development decoder with fallback support
    pub fn development(supported_chain_ids: Vec<u64>) -> UnifiedTransactionDecoder {
        UnifiedTransactionDecoder::new(supported_chain_ids, true)
    }

    /// Create a testing decoder with relaxed validation
    pub fn testing() -> UnifiedTransactionDecoder {
        let config = DecoderConfig {
            enable_legacy_support: true,
            enable_eip2930_support: true,
            enable_eip1559_support: true,
            strict_validation: false,
            max_transaction_size: 50_000_000, // 50MB for testing
            default_chain_id: 31337, // Common test chain ID
        };

        let enhanced_decoder = EnhancedTransactionDecoder::new(vec![1, 31337, 1337], config);

        UnifiedTransactionDecoder {
            enhanced_decoder,
            fallback_enabled: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decoder_creation() {
        let decoder = DecoderFactory::production(vec![1, 5]);
        assert!(!decoder.fallback_enabled);
    }

    #[test]
    fn test_development_decoder() {
        let decoder = DecoderFactory::development(vec![1, 5, 1337]);
        assert!(decoder.fallback_enabled);
    }

    #[test]
    fn test_quick_validation_empty() {
        let decoder = DecoderFactory::testing();
        let result = decoder.quick_validate(&[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_quick_validation_too_large() {
        let decoder = DecoderFactory::testing();
        let large_tx = vec![0u8; 60_000_000];
        let result = decoder.quick_validate(&large_tx);
        assert!(result.is_err());
    }

    #[test]
    fn test_global_decoder_init() {
        let global = GlobalTransactionDecoder::init(vec![1, 31337], true);
        assert!(global.decoder.fallback_enabled);
    }
}