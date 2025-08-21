pub mod raydium_v4;
pub mod orca_whirlpool;
pub mod types;
pub mod utils;
pub mod common;
pub mod compute_budget;
pub mod transaction_builder;

use anyhow::Result;
use solana_sdk::pubkey::Pubkey;
use crate::exchanges::types::{DexLabel, PoolInfo, SwapQuote};

#[async_trait::async_trait]
pub trait DexAdapter: Send + Sync {
    fn get_label(&self) -> DexLabel;
    async fn get_pool_info(&self, pool_address: &Pubkey) -> Result<PoolInfo>;
    async fn get_swap_quote(&self, pool_address: &Pubkey, amount_in: u64, token_in: &Pubkey) -> Result<SwapQuote>;
    
    /// Create swap instruction with slippage protection
    fn create_swap_instruction(
        &self, 
        quote: &SwapQuote, 
        user_pubkey: &Pubkey,
        min_amount_out: u64,
    ) -> Result<solana_sdk::instruction::Instruction>;
    
    /// Get real-time reserves for a pool
    async fn get_pool_reserves(&self, pool_address: &Pubkey) -> Result<crate::exchanges::types::PoolReserves>;
}

pub mod factory {
    use super::*;
    use crate::exchanges::{raydium_v4::RaydiumV4Adapter, orca_whirlpool::OrcaWhirlpoolAdapter};
    use crate::config::Config;

    pub fn create_adapter(dex_label: DexLabel, config: Config) -> Result<Box<dyn DexAdapter>> {
        match dex_label {
            DexLabel::RaydiumV4 => Ok(Box::new(RaydiumV4Adapter::new(config)?)),
            DexLabel::OrcaWhirlpool => Ok(Box::new(OrcaWhirlpoolAdapter::new(config)?)),
        }
    }
}
