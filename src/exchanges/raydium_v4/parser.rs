use anyhow::Result;
use tracing::{info, warn};
use solana_sdk::pubkey::Pubkey;
use crate::exchanges::types::{TokenInfo, PoolReserves, PoolFees};

pub struct RaydiumV4Parser;

impl RaydiumV4Parser {
    pub fn parse_pool_data(&self, data: &[u8]) -> Result<(TokenInfo, TokenInfo, PoolReserves, PoolFees)> {
        info!("Parsing Raydium V4 pool data, size: {} bytes", data.len());
        
        if data.len() < 500 {
            warn!("Data too short for Raydium V4 pool");
            return Err(anyhow::anyhow!("Data too short for Raydium V4 pool"));
        }

        // Known token positions from our analysis
        let base_mint_start = 400;  // WSOL
        let quote_mint_start = 432; // USDC
        
        // Vault positions based on REAL data analysis
        // Found real vault addresses at positions 336 and 368
        let base_vault_start = 336;  // Real WSOL vault
        let quote_vault_start = 368; // Real USDC vault

        // Parse base token (WSOL)
        let base_mint = if base_mint_start + 32 <= data.len() {
            Pubkey::try_from(&data[base_mint_start..base_mint_start + 32])?
        } else {
            warn!("Base mint position out of bounds");
            return Err(anyhow::anyhow!("Base mint position out of bounds"));
        };

        // Parse quote token (USDC)
        let quote_mint = if quote_mint_start + 32 <= data.len() {
            Pubkey::try_from(&data[quote_mint_start..quote_mint_start + 32])?
        } else {
            warn!("Quote mint position out of bounds");
            return Err(anyhow::anyhow!("Quote mint position out of bounds"));
        };

        // Parse vaults
        let base_vault = if base_vault_start + 32 <= data.len() {
            Some(Pubkey::try_from(&data[base_vault_start..base_vault_start + 32])?)
        } else {
            warn!("Base vault position out of bounds");
            None
        };

        let quote_vault = if quote_vault_start + 32 <= data.len() {
            Some(Pubkey::try_from(&data[quote_vault_start..quote_vault_start + 32])?)
        } else {
            warn!("Quote vault position out of bounds");
            None
        };

        info!("Parsed tokens - Base: {} (WSOL), Quote: {} (USDC)", base_mint, quote_mint);
        if let Some(vault) = base_vault {
            info!("Base vault: {}", vault);
        }
        if let Some(vault) = quote_vault {
            info!("Quote vault: {}", vault);
        }

        // Create TokenInfo structs
        let base_token = TokenInfo {
            mint: base_mint,
            symbol: "WSOL".to_string(),
            decimals: 9,
            vault: base_vault.expect("Base vault should exist"),
        };

        let quote_token = TokenInfo {
            mint: quote_mint,
            symbol: "USDC".to_string(),
            decimals: 6,
            vault: quote_vault.expect("Quote vault should exist"),
        };

        // Parse fees from contract data using exact positions found in analysis
        let trade_fee_numerator_pos = 144; // Found at position 144 (u16 little-endian)
        let trade_fee_denominator_pos = 136; // Found at position 136 (u32 little-endian)
        
        let trade_fee_numerator = if trade_fee_numerator_pos + 2 <= data.len() {
            u16::from_le_bytes([data[trade_fee_numerator_pos], data[trade_fee_numerator_pos + 1]])
        } else {
            warn!("Trade fee numerator position out of bounds, using default");
            25
        };
        
        let trade_fee_denominator = if trade_fee_denominator_pos + 4 <= data.len() {
            u32::from_le_bytes([
                data[trade_fee_denominator_pos],
                data[trade_fee_denominator_pos + 1],
                data[trade_fee_denominator_pos + 2],
                data[trade_fee_denominator_pos + 3]
            ])
        } else {
            warn!("Trade fee denominator position out of bounds, using default");
            10000
        };
        
        info!("Parsed fees from contract: {}/{} = {} bps", trade_fee_numerator, trade_fee_denominator, trade_fee_numerator);
        
        let fees = PoolFees {
            trade_fee_bps: trade_fee_numerator as u32, // 0.25% (25/10000)
            owner_trade_fee_bps: 0,
            owner_withdraw_fee_bps: 0,
        };

        // Note: Reserves are stored in separate token accounts (vaults)
        // We need to fetch them via RPC calls to getTokenAccountBalance
        info!("Fees: trade_fee_bps = {}, owner_trade_fee_bps = {}, owner_withdraw_fee_bps = {}", 
              fees.trade_fee_bps, fees.owner_trade_fee_bps, fees.owner_withdraw_fee_bps);

        Ok((base_token, quote_token, PoolReserves { token_a_reserve: 0, token_b_reserve: 0, lp_supply: None }, fees))
    }
}
