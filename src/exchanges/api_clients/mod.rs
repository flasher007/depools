pub mod orca_quote_client;
pub mod raydium_quote_client;

use async_trait::async_trait;
use anyhow::Result;
use solana_sdk::pubkey::Pubkey;
use crate::exchanges::types::{SwapQuote, PoolInfo};

/// Базовый trait для API клиентов получения котировок
#[async_trait]
pub trait QuoteApiClient: Send + Sync {
    /// Получить котировку для свапа
    async fn get_quote(&self, pool_pubkey: &Pubkey, amount_in: u64) -> Result<SwapQuote>;
    
    /// Получить информацию о пуле через API
    async fn get_pool_info(&self, pool_pubkey: &Pubkey) -> Result<PoolInfo>;
    
    /// Проверить доступность API
    async fn is_available(&self) -> bool;
}
