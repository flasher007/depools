pub mod raydium_v4;
pub mod orca_whirlpool;
pub mod compute_budget;
pub mod transaction_builder;
pub mod api_clients; // Новый модуль для API клиентов
pub mod types;
pub mod utils;
pub mod common;

use async_trait::async_trait;
use anyhow::Result;
use solana_sdk::pubkey::Pubkey;
use std::any::Any;
use crate::exchanges::types::{PoolInfo, SwapQuote, DexLabel};

#[async_trait]
pub trait DexAdapter: Send + Sync {
    async fn get_pool_info(&self, pool_pubkey: &Pubkey) -> Result<PoolInfo>;
    async fn get_swap_quote(&self, pool_pubkey: &Pubkey, amount_in: u64) -> Result<SwapQuote>;
    async fn create_swap_instruction(&self, pool_pubkey: &Pubkey, amount_in: u64, min_amount_out: u64) -> Result<solana_sdk::instruction::Instruction>;
    
    /// Метод для downcasting к конкретному типу адаптера
    fn as_any(&self) -> &dyn Any;
}

pub fn create_adapter(dex_label: DexLabel, config: crate::config::Config) -> Result<Box<dyn DexAdapter>> {
    match dex_label {
        DexLabel::RaydiumV4 => Ok(Box::new(raydium_v4::adapter::RaydiumV4Adapter::new(config)?)),
        DexLabel::OrcaWhirlpool => Ok(Box::new(orca_whirlpool::adapter::OrcaWhirlpoolAdapter::new(config)?)),
    }
}
