use thiserror::Error;

#[derive(Error, Debug)]
pub enum WalletError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] bincode::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Hex decode error: {0}")]
    HexDecode(#[from] hex::FromHexError),

    #[error("Encryption error: {0}")]
    Encryption(String),

    #[error("Decryption error: {0}")]
    Decryption(String),

    #[error("Invalid password")]
    InvalidPassword,

    #[error("Account not found: {0}")]
    AccountNotFound(String),

    #[error("Insufficient balance: need {need} LATT, have {have} LATT")]
    InsufficientBalance { need: String, have: String },

    #[error("RPC error: {0}")]
    Rpc(String),

    #[error("Transaction failed: {0}")]
    TransactionFailed(String),

    #[error("Invalid address: {0}")]
    InvalidAddress(String),

    #[error("Wallet locked")]
    WalletLocked,

    #[error("Wallet already exists at path")]
    WalletExists,

    #[error("Other error: {0}")]
    Other(String),
}
