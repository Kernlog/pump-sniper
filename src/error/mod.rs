//! Error types

use thiserror::Error;

/// Custom err
#[derive(Error, Debug)]
pub enum SniperError {
    #[error("gRPC connection failed: {0}")]
    GrpcConnectionFailed(String),
    
    #[error("Failed to parse transaction: {0}")]
    TransactionParseError(String),
    
    #[error("Bonding curve is complete")]
    BondingCurveComplete,
    
    #[error("Insufficient funds")]
    InsufficientFunds,
    
    #[error("Transaction failed: {0}")]
    TransactionFailed(String),
    
    #[error("RPC error: {0}")]
    RpcError(String),
    
    #[error("Serialization error: {0}")]
    SerializationError(String),
    
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
    
    #[error("Token not found: {0}")]
    TokenNotFound(String),
    
    #[error("Market cap calculation failed")]
    MarketCapCalculationFailed,
    
    #[error("Slippage exceeded")]
    SlippageExceeded,
}