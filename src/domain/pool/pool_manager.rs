//! Pool manager for liquidity pool operations

use crate::shared::types::{Amount, Token};
use crate::domain::dex::DexPool;

/// Manages liquidity pool operations
pub struct PoolManager {
    pools: Vec<DexPool>,
}

impl PoolManager {
    pub fn new() -> Self {
        Self { pools: Vec::new() }
    }

    pub fn add_pool(&mut self, pool: DexPool) {
        self.pools.push(pool);
    }

    pub fn get_pools(&self) -> &[DexPool] {
        &self.pools
    }
}
