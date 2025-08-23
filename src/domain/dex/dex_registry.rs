//! DEX Registry for Solana mainnet

use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;
use crate::shared::errors::AppError;
use crate::shared::types::{Token, Amount};

/// Supported DEX types on Solana mainnet
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DexType {
    OrcaWhirlpool,
    RaydiumV4,
    RaydiumAMM,
    RaydiumCLMM,
    MeteoraDLMM,
    MeteoraPools,
    PumpSwap,
    Jupiter,
}

/// DEX information
#[derive(Debug, Clone)]
pub struct DexInfo {
    pub name: String,
    pub program_id: Pubkey,
    pub dex_type: DexType,
    pub description: String,
    pub is_active: bool,
}

/// Pool information
#[derive(Debug, Clone)]
pub struct PoolInfo {
    pub id: String,
    pub dex_type: DexType,
    pub token_a: Token,
    pub token_b: Token,
    pub reserve_a: Amount,
    pub reserve_b: Amount,
    pub fee_rate: f64,
    pub liquidity: Amount,
    pub volume_24h: Amount,
}

/// DEX Registry for Solana mainnet
pub struct DexRegistry;

impl DexRegistry {
    /// Get all supported DEXes
    pub fn get_all_dexes() -> Vec<DexInfo> {
        vec![
            DexInfo {
                name: "orca".to_string(),
                program_id: Pubkey::from_str("whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc").unwrap(),
                dex_type: DexType::OrcaWhirlpool,
                description: "Orca Whirlpool - Concentrated Liquidity AMM".to_string(),
                is_active: true,
            },
            DexInfo {
                name: "raydium_v4".to_string(),
                program_id: Pubkey::from_str("675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8").unwrap(),
                dex_type: DexType::RaydiumV4,
                description: "Raydium V4 - Legacy AMM".to_string(),
                is_active: true,
            },
            DexInfo {
                name: "raydium_amm".to_string(),
                program_id: Pubkey::from_str("9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM").unwrap(),
                dex_type: DexType::RaydiumAMM,
                description: "Raydium AMM - Automated Market Maker".to_string(),
                is_active: true,
            },
            DexInfo {
                name: "raydium_clmm".to_string(),
                program_id: Pubkey::from_str("CAMMCzo5YL8w4VFF8KVHrK22GGUQp5VhH5bMKM3p9bkt").unwrap(),
                dex_type: DexType::RaydiumCLMM,
                description: "Raydium CLMM - Concentrated Liquidity Market Maker".to_string(),
                is_active: true,
            },
            DexInfo {
                name: "meteora_dlmm".to_string(),
                program_id: Pubkey::from_str("LBUZKhRxPF3XUpBCjp4YzTKgLccjZhTSDM9YuVaPwxo").unwrap(),
                dex_type: DexType::MeteoraDLMM,
                description: "Meteora DLMM - Dynamic Liquidity Market Maker".to_string(),
                is_active: true,
            },
            DexInfo {
                name: "meteora_pools".to_string(),
                program_id: Pubkey::from_str("Eo7WjKq67rjJQSZxS6z3YkapzY3eMj6Xy8X5EQVn5UaB").unwrap(),
                dex_type: DexType::MeteoraPools,
                description: "Meteora Pools - Traditional AMM".to_string(),
                is_active: true,
            },
            DexInfo {
                name: "pumpswap".to_string(),
                program_id: Pubkey::from_str("9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM").unwrap(),
                dex_type: DexType::PumpSwap,
                description: "PumpSwap - AMM with pump mechanics".to_string(),
                is_active: true,
            },
            DexInfo {
                name: "jupiter".to_string(),
                program_id: Pubkey::from_str("JUP4Fb2cqiRUcaTHdrPC8h2gNsA2ETXiPDD33WcGuJB").unwrap(),
                dex_type: DexType::Jupiter,
                description: "Jupiter - Aggregator and DEX".to_string(),
                is_active: true,
            },
        ]
    }
    
    /// Get DEX by type
    pub fn get_dex_by_type(dex_type: &DexType) -> Option<DexInfo> {
        Self::get_all_dexes()
            .into_iter()
            .find(|dex| dex.dex_type == *dex_type)
    }
    
    /// Get DEX by program ID
    pub fn get_dex_by_program_id(program_id: &Pubkey) -> Option<DexInfo> {
        Self::get_all_dexes()
            .into_iter()
            .find(|dex| dex.program_id == *program_id)
    }
    
    /// Get active DEXes only
    pub fn get_active_dexes() -> Vec<DexInfo> {
        Self::get_all_dexes()
            .into_iter()
            .filter(|dex| dex.is_active)
            .collect()
    }
    
    /// Check if DEX is supported
    pub fn is_dex_supported(program_id: &Pubkey) -> bool {
        Self::get_dex_by_program_id(program_id).is_some()
    }
    
    /// Get DEX type by program ID
    pub fn get_dex_type_by_program_id(program_id: &Pubkey) -> Option<DexType> {
        Self::get_dex_by_program_id(program_id)
            .map(|dex| dex.dex_type)
    }
}

impl DexType {
    /// Get string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            DexType::OrcaWhirlpool => "Orca Whirlpool",
            DexType::RaydiumV4 => "Raydium V4",
            DexType::RaydiumAMM => "Raydium AMM",
            DexType::RaydiumCLMM => "Raydium CLMM",
            DexType::MeteoraDLMM => "Meteora DLMM",
            DexType::MeteoraPools => "Meteora Pools",
            DexType::PumpSwap => "PumpSwap",
            DexType::Jupiter => "Jupiter",
        }
    }
    
    /// Get program ID
    pub fn program_id(&self) -> Pubkey {
        match self {
            DexType::OrcaWhirlpool => Pubkey::from_str("whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc").unwrap(),
            DexType::RaydiumV4 => Pubkey::from_str("675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8").unwrap(),
            DexType::RaydiumAMM => Pubkey::from_str("9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM").unwrap(),
            DexType::RaydiumCLMM => Pubkey::from_str("CAMMCzo5YL8w4VFF8KVHrK22GGUQp5VhH5bMKM3p9bkt").unwrap(),
            DexType::MeteoraDLMM => Pubkey::from_str("LBUZKhRxPF3XUpBCjp4YzTKgLccjZhTSDM9YuVaPwxo").unwrap(),
            DexType::MeteoraPools => Pubkey::from_str("Eo7WjKq67rjJQSZxS6z3YkapzY3eMj6Xy8X5EQVn5UaB").unwrap(),
            DexType::PumpSwap => Pubkey::from_str("9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM").unwrap(),
            DexType::Jupiter => Pubkey::from_str("JUP4Fb2cqiRUcaTHdrPC8h2gNsA2ETXiPDD33WcGuJB").unwrap(),
        }
    }
}


