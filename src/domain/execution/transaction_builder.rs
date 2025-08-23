//! Transaction building and construction

use solana_sdk::transaction::Transaction;
use crate::shared::types::{Amount, Token};
use super::ExecutionRequest;

/// Builds Solana transactions
pub struct TransactionBuilder;

impl TransactionBuilder {
    pub fn new() -> Self {
        Self
    }

    pub fn build_arbitrage_transaction(
        &self,
        request: &ExecutionRequest,
    ) -> Result<Transaction, String> {
        // TODO: Implement transaction building
        Err("Not implemented yet".to_string())
    }
}
