//! Pool discovery and scanning

use crate::shared::types::{Amount, Token};
use crate::domain::dex::DexPool;

/// Discovers and scans liquidity pools
pub struct PoolDiscovery {
    min_liquidity: Amount,
}

impl PoolDiscovery {
    pub fn new(min_liquidity: Amount) -> Self {
        Self { min_liquidity }
    }

    pub fn discover_pools(&self) -> Vec<DexPool> {
        // TODO: Implement pool discovery
        Vec::new()
    }
}
