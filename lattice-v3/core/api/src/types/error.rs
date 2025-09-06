use jsonrpc_core::{Error, ErrorCode};
use thiserror::Error;

/// API Error types
#[derive(Debug, Error)]
pub enum ApiError {
    #[error("Block not found: {0}")]
    BlockNotFound(String),
    
    #[error("Transaction not found: {0}")]
    TransactionNotFound(String),
    
    #[error("Account not found: {0}")]
    AccountNotFound(String),
    
    #[error("Model not found: {0}")]
    ModelNotFound(String),
    
    #[error("Invalid parameters: {0}")]
    InvalidParams(String),
    
    #[error("Internal error: {0}")]
    InternalError(String),
    
    #[error("Method not found")]
    MethodNotFound,
    
    #[error("Invalid transaction: {0}")]
    InvalidTransaction(String),
    
    #[error("Mempool full")]
    MempoolFull,
    
    #[error("Execution failed: {0}")]
    ExecutionFailed(String),
}

impl From<ApiError> for Error {
    fn from(err: ApiError) -> Self {
        match err {
            ApiError::BlockNotFound(_) |
            ApiError::TransactionNotFound(_) |
            ApiError::AccountNotFound(_) |
            ApiError::ModelNotFound(_) => {
                Error {
                    code: ErrorCode::InvalidParams,
                    message: err.to_string(),
                    data: None,
                }
            }
            ApiError::InvalidParams(_) |
            ApiError::InvalidTransaction(_) => {
                Error {
                    code: ErrorCode::InvalidParams,
                    message: err.to_string(),
                    data: None,
                }
            }
            ApiError::MethodNotFound => {
                Error {
                    code: ErrorCode::MethodNotFound,
                    message: err.to_string(),
                    data: None,
                }
            }
            ApiError::MempoolFull |
            ApiError::ExecutionFailed(_) |
            ApiError::InternalError(_) => {
                Error {
                    code: ErrorCode::InternalError,
                    message: err.to_string(),
                    data: None,
                }
            }
        }
    }
}