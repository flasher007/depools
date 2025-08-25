//! Error handling for the application

use thiserror::Error;

/// Arbitrage-related errors
#[derive(Error, Debug, Clone)]
pub enum ArbitrageError {
    #[error("Insufficient profit margin: {0}%")]
    InsufficientProfit(f64),
    
    #[error("Route not found for tokens")]
    RouteNotFound,
    
    #[error("Invalid arbitrage route")]
    InvalidRoute,
    
    #[error("Price data outdated")]
    PriceOutdated,
    
    #[error("Insufficient liquidity: {0}")]
    InsufficientLiquidity(String),
}

/// DEX-related errors
#[derive(Error, Debug)]
pub enum DexError {
    #[error("DEX not supported: {0}")]
    UnsupportedDex(String),
    
    #[error("Pool not found: {0}")]
    PoolNotFound(String),
    
    #[error("Invalid swap parameters")]
    InvalidSwapParams,
    
    #[error("Slippage tolerance exceeded: {0}%")]
    SlippageExceeded(f64),
    
    #[error("API request failed: {0}")]
    ApiError(String),
}

/// Execution-related errors
#[derive(Error, Debug, Clone)]
pub enum ExecutionError {
    #[error("Transaction failed: {0}")]
    TransactionFailed(String),
    
    #[error("Insufficient balance")]
    InsufficientBalance,
    
    #[error("Transaction timeout")]
    Timeout,
    
    #[error("Invalid signature")]
    InvalidSignature,
    
    #[error("Network error: {0}")]
    NetworkError(String),
}

/// Pool-related errors
#[derive(Error, Debug)]
pub enum PoolError {
    #[error("Pool discovery failed")]
    DiscoveryFailed,
    
    #[error("Invalid pool data")]
    InvalidPoolData,
    
    #[error("Pool analysis failed")]
    AnalysisFailed,
}

/// Price-related errors
#[derive(Error, Debug)]
pub enum PriceError {
    #[error("Price feed unavailable")]
    FeedUnavailable,
    
    #[error("Invalid price data")]
    InvalidPriceData,
    
    #[error("Price calculation failed")]
    CalculationFailed,
}

/// General application error
#[derive(Error, Debug)]
pub enum AppError {
    #[error("Configuration error: {0}")]
    ConfigError(String),
    
    #[error("Database error: {0}")]
    DatabaseError(String),
    
    #[error("Blockchain error: {0}")]
    BlockchainError(String),
    
    #[error("Execution error: {0}")]
    ExecutionError(String),
    
    #[error("Unknown error: {0}")]
    Unknown(String),
}

impl From<ArbitrageError> for AppError {
    fn from(err: ArbitrageError) -> Self {
        AppError::Unknown(err.to_string())
    }
}

impl From<DexError> for AppError {
    fn from(err: DexError) -> Self {
        AppError::Unknown(err.to_string())
    }
}

impl From<ExecutionError> for AppError {
    fn from(err: ExecutionError) -> Self {
        AppError::Unknown(err.to_string())
    }
}

impl From<PoolError> for AppError {
    fn from(err: PoolError) -> Self {
        AppError::Unknown(err.to_string())
    }
}

impl From<PriceError> for AppError {
    fn from(err: PriceError) -> Self {
        AppError::Unknown(err.to_string())
    }
}
