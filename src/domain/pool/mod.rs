//! Pool domain - liquidity pool management

mod pool_manager;
mod pool_discovery;
mod pool_analyzer;

pub use pool_manager::PoolManager;
pub use pool_discovery::PoolDiscovery;
pub use pool_analyzer::PoolAnalyzer;

use crate::shared::types::{Token, Amount, Price};
use crate::domain::dex::DexPool;

/// Pool discovery criteria
#[derive(Debug, Clone)]
pub struct PoolDiscoveryCriteria {
    pub min_liquidity: Amount,
    pub min_volume_24h: Amount,
    pub supported_tokens: Vec<Token>,
    pub dex_types: Vec<crate::domain::dex::DexType>,
}

/// Pool analysis result
#[derive(Debug, Clone)]
pub struct PoolAnalysis {
    pub pool: DexPool,
    pub volume_24h: Amount,
    pub price_volatility: f64,
    pub liquidity_depth: Vec<(Price, Amount)>,
    pub risk_score: f64,
}
