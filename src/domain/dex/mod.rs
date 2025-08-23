//! DEX domain - decentralized exchange integrations

mod dex_registry;

pub use dex_registry::{DexRegistry, DexType, DexInfo, PoolInfo};

use crate::shared::types::{Token, Amount, Price};
use crate::shared::errors::DexError;

/// DEX pool information
#[derive(Debug, Clone)]
pub struct DexPool {
    pub id: String,
    pub dex_type: DexType,
    pub token_a: Token,
    pub token_b: Token,
    pub reserve_a: Amount,
    pub reserve_b: Amount,
    pub fee_rate: f64,
    pub liquidity: Amount,
}

/// DEX swap quote
#[derive(Debug, Clone)]
pub struct SwapQuote {
    pub pool_id: String,
    pub dex_type: DexType,
    pub token_in: Token,
    pub token_out: Token,
    pub amount_in: Amount,
    pub amount_out: Amount,
    pub fee: Amount,
    pub price_impact: f64,
    pub slippage: f64,
}
