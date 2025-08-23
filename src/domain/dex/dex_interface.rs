//! DEX interface trait

use async_trait::async_trait;
use crate::shared::types::{Token, Amount};
use crate::shared::errors::DexError;
use super::{DexType, PoolInfo, SwapQuote};

/// Common interface for all DEX implementations
#[async_trait]
pub trait DexInterface: Send + Sync {
    fn dex_type(&self) -> DexType;
    
    async fn get_pools(&self) -> Result<Vec<PoolInfo>, DexError>;
    
    async fn get_quote(
        &self,
        token_in: &Token,
        token_out: &Token,
        amount_in: Amount,
    ) -> Result<SwapQuote, DexError>;
    
    async fn get_pool_info(&self, pool_id: &str) -> Result<PoolInfo, DexError>;
    
    async fn get_token_price(&self, token: &Token) -> Result<f64, DexError>;
}
