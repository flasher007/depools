use async_trait::async_trait;
use crate::domain::dex::{DexType, PoolInfo};
use crate::shared::errors::AppError;

/// Trait for DEX-specific adapters
/// This provides a unified interface for different DEX implementations
#[async_trait]
pub trait DexAdapter: Send + Sync {
    /// Get the DEX type this adapter handles
    fn dex_type(&self) -> DexType;
    
    /// Discover pools for this DEX
    async fn discover_pools(&self) -> Result<Vec<PoolInfo>, AppError>;
    
    /// Get pool by specific token pair (if supported)
    async fn get_pool_by_tokens(&self, token_a: &str, token_b: &str) -> Result<Option<PoolInfo>, AppError>;
    
    /// Check if an account is a valid pool for this DEX
    fn is_pool_account(&self, account_data: &[u8]) -> bool;
    
    /// Parse pool account data into PoolInfo
    async fn parse_pool_account(&self, account_data: &[u8]) -> Result<PoolInfo, AppError>;
    
    /// Get pool statistics (liquidity, volume, etc.)
    async fn get_pool_stats(&self, pool_id: &str) -> Result<PoolInfo, AppError>;
}
