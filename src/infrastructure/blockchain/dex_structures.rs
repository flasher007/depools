//! Real DEX structures for Solana mainnet

use solana_sdk::pubkey::Pubkey;
use borsh::{BorshDeserialize, BorshSerialize};
use bytemuck::{Pod, Zeroable};
use crate::shared::errors::AppError;

/// Orca Whirlpool structure
#[derive(Debug, Clone, BorshDeserialize, BorshSerialize)]
pub struct Whirlpool {
    pub whirlpools_config: Pubkey,
    pub whirlpool_bump: [u8; 1],
    pub tick_spacing: u16,
    pub tick_spacing_seed: [u8; 2],
    pub fee_rate: u16,
    pub protocol_fee_rate: u16,
    pub liquidity: u128,
    pub sqrt_price: u128,
    pub tick_current_index: i32,
    pub protocol_fee_owed_a: u64,
    pub protocol_fee_owed_b: u64,
    pub token_vault_a: Pubkey,
    pub token_vault_b: Pubkey,
    pub fee_growth_global_a: u128,
    pub fee_growth_global_b: u128,
    pub fee_growth_checkpoint_a: u128,
    pub fee_growth_checkpoint_b: u128,
    pub reward_infos: [RewardInfo; 3],
    pub reward_last_updated_timestamp: u64,
    pub tick_arrays: [Pubkey; 2],
}

/// Raydium V4 CLMM structure
#[derive(Debug, Clone, BorshDeserialize, BorshSerialize)]
pub struct RaydiumV4Pool {
    pub status: u64,
    pub nonce: u64,
    pub order_num: u64,
    pub depth: u64,
    pub base_decimals: u64,
    pub quote_decimals: u64,
    pub state: u64,
    pub reset_flag: u64,
    pub min_size: u64,
    pub vol_max_cut_ratio: u64,
    pub amount_wave_ratio: u64,
    pub coin_lot_size: u64,
    pub pc_lot_size: u64,
    pub min_price_multiplier: u64,
    pub max_price_multiplier: u64,
    pub system_decimals_value: u64,
    pub min_separate_numerator: u64,
    pub min_separate_denominator: u64,
    pub trade_fee_numerator: u64,
    pub trade_fee_denominator: u64,
    pub pnl_numerator: u64,
    pub pnl_denominator: u64,
    pub swap_fee_numerator: u64,
    pub swap_fee_denominator: u64,
    pub need_take_pnl_coin: u64,
    pub need_take_pnl_pc: u64,
    pub total_pnl_pc: u64,
    pub total_pnl_coin: u64,
    pub pool_total_deposit_pc: u128,
    pub pool_total_deposit_coin: u128,
    pub swap_coin_in_amount: u128,
    pub swap_pc_out_amount: u128,
    pub swap_coin_to_pc_fee: u128,
    pub swap_pc_in_amount: u128,
    pub swap_coin_out_amount: u128,
    pub swap_pc_to_coin_fee: u128,
    pub token_coin: Pubkey,
    pub token_pc: Pubkey,
    pub coin_vault: Pubkey,
    pub pc_vault: Pubkey,
    pub lp_vault: Pubkey,
    pub open_orders: Pubkey,
    pub market: Pubkey,
    pub serum_dex: Pubkey,
    pub target_orders: Pubkey,
    pub withdraw_queue: Pubkey,
    pub lp_mint: Pubkey,
    pub owner: Pubkey,
    pub lp_mint_authority: Pubkey,
    pub base_mint: Pubkey,
    pub quote_mint: Pubkey,
    pub base_vault: Pubkey,
    pub quote_vault: Pubkey,
    pub market_base_vault: Pubkey,
    pub market_quote_vault: Pubkey,
    pub market_authority: Pubkey,
    pub target_orders_authority: Pubkey,
    pub withdraw_queue_authority: Pubkey,
}

/// Reward information for Orca Whirlpool
#[derive(Debug, Clone, BorshDeserialize, BorshSerialize)]
pub struct RewardInfo {
    pub mint: Pubkey,
    pub vault: Pubkey,
    pub authority: Pubkey,
    pub emissions_per_second_x64: u128,
    pub growth_global_x64: u128,
}

/// Meteora DLMM structure
#[derive(Debug, Clone, BorshDeserialize, BorshSerialize)]
pub struct MeteoraDLMM {
    pub status: u64,
    pub nonce: u64,
    pub max_order: u64,
    pub depth: u64,
    pub base_decimals: u64,
    pub quote_decimals: u64,
    pub state: u64,
    pub reset_flag: u64,
    pub min_size: u64,
    pub vol_max_cut_ratio: u64,
    pub amount_wave_ratio: u64,
    pub coin_lot_size: u64,
    pub pc_lot_size: u64,
    pub min_price_multiplier: u64,
    pub max_price_multiplier: u64,
    pub system_decimals_value: u64,
    pub min_separate_numerator: u64,
    pub min_separate_denominator: u64,
    pub trade_fee_numerator: u64,
    pub trade_fee_denominator: u64,
    pub pnl_numerator: u64,
    pub pnl_denominator: u64,
    pub swap_fee_numerator: u64,
    pub swap_fee_denominator: u64,
    pub need_take_pnl_coin: u64,
    pub need_take_pnl_pc: u64,
    pub total_pnl_pc: u64,
    pub total_pnl_coin: u64,
    pub pool_total_deposit_pc: u128,
    pub pool_total_deposit_coin: u128,
    pub swap_coin_in_amount: u128,
    pub swap_pc_out_amount: u128,
    pub swap_coin_to_pc_fee: u128,
    pub swap_pc_in_amount: u128,
    pub swap_coin_out_amount: u128,
    pub swap_pc_to_coin_fee: u128,
    pub token_coin: Pubkey,
    pub token_pc: Pubkey,
    pub coin_vault: Pubkey,
    pub pc_vault: Pubkey,
    pub lp_vault: Pubkey,
    pub open_orders: Pubkey,
    pub market: Pubkey,
    pub serum_dex: Pubkey,
    pub target_orders: Pubkey,
    pub withdraw_queue: Pubkey,
    pub lp_mint: Pubkey,
    pub owner: Pubkey,
    pub lp_mint_authority: Pubkey,
    pub base_mint: Pubkey,
    pub quote_mint: Pubkey,
    pub base_vault: Pubkey,
    pub quote_vault: Pubkey,
    pub market_base_vault: Pubkey,
    pub market_quote_vault: Pubkey,
    pub market_authority: Pubkey,
    pub target_orders_authority: Pubkey,
    pub withdraw_queue_authority: Pubkey,
}

impl Whirlpool {
    /// Try to deserialize from account data
    pub fn try_deserialize(data: &[u8]) -> Result<Self, AppError> {
        if data.len() < 653 {
            return Err(AppError::BlockchainError("Invalid Whirlpool account data length".to_string()));
        }
        
        BorshDeserialize::try_from_slice(data)
            .map_err(|e| AppError::BlockchainError(format!("Failed to deserialize Whirlpool: {}", e)))
    }
    
    /// Get pool fee rate as percentage
    pub fn fee_rate_percentage(&self) -> f64 {
        self.fee_rate as f64 / 10000.0
    }
    
    /// Get current sqrt price
    pub fn current_sqrt_price(&self) -> u128 {
        self.sqrt_price
    }
    
    /// Get current tick index
    pub fn current_tick_index(&self) -> i32 {
        self.tick_current_index
    }
}

impl RaydiumV4Pool {
    /// Try to deserialize from account data
    pub fn try_deserialize(data: &[u8]) -> Result<Self, AppError> {
        if data.len() < 752 {
            return Err(AppError::BlockchainError("Invalid Raydium V4 pool account data length".to_string()));
        }
        
        BorshDeserialize::try_from_slice(data)
            .map_err(|e| AppError::BlockchainError(format!("Failed to deserialize Raydium V4 pool: {}", e)))
    }
    
    /// Get trade fee as percentage
    pub fn trade_fee_percentage(&self) -> f64 {
        if self.trade_fee_denominator == 0 {
            return 0.0;
        }
        (self.trade_fee_numerator as f64 / self.trade_fee_denominator as f64) * 100.0
    }
    
    /// Get swap fee as percentage
    pub fn swap_fee_percentage(&self) -> f64 {
        if self.swap_fee_denominator == 0 {
            return 0.0;
        }
        (self.swap_fee_numerator as f64 / self.swap_fee_denominator as f64) * 100.0
    }
}

impl MeteoraDLMM {
    /// Try to deserialize from account data
    pub fn try_deserialize(data: &[u8]) -> Result<Self, AppError> {
        if data.len() < 752 {
            return Err(AppError::BlockchainError("Invalid Meteora DLMM account data length".to_string()));
        }
        
        BorshDeserialize::try_from_slice(data)
            .map_err(|e| AppError::BlockchainError(format!("Failed to deserialize Meteora DLMM: {}", e)))
    }
    
    /// Get trade fee as percentage
    pub fn trade_fee_percentage(&self) -> f64 {
        if self.trade_fee_denominator == 0 {
            return 0.0;
        }
        (self.trade_fee_numerator as f64 / self.trade_fee_denominator as f64) * 100.0
    }
}
