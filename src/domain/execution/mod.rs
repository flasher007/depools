//! Execution domain - transaction execution and management

mod transaction_executor;
mod transaction_builder;
mod transaction_validator;
mod arbitrage_executor;

pub use transaction_executor::TransactionExecutor;
pub use transaction_builder::TransactionBuilder;
pub use transaction_validator::TransactionValidator;
pub use arbitrage_executor::{ArbitrageTransactionExecutor, RiskManagementConfig, ActiveTrade, DailyStats, TradeStatus};

use crate::shared::types::{Amount, Token};
use crate::shared::errors::ExecutionError;
use solana_sdk::transaction::Transaction;

/// Transaction execution request
#[derive(Debug, Clone)]
pub struct ExecutionRequest {
    pub route_id: String,
    pub amount_in: Amount,
    pub token_in: Token,
    pub token_out: Token,
    pub min_amount_out: Amount,
    pub slippage_tolerance: f64,
    pub deadline: chrono::DateTime<chrono::Utc>,
}

/// Transaction execution result
#[derive(Debug, Clone)]
pub struct ExecutionResult {
    pub request: ExecutionRequest,
    pub transaction: Option<Transaction>,
    pub signature: Option<String>,
    pub success: bool,
    pub error: Option<ExecutionError>,
    pub gas_used: Option<u64>,
    pub actual_amount_out: Option<Amount>,
}
