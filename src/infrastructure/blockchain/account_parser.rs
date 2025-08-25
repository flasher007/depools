//! Account parsers for DEX programs

use solana_sdk::pubkey::Pubkey;
use crate::shared::types::{Token, Amount};
use crate::domain::dex::{PoolInfo, DexType};
use crate::shared::errors::AppError;
use super::{orca_structures::Whirlpool, dex_structures::RaydiumAMMPool, VaultReader};
use super::vault_reader::TokenMetadata;
use std::sync::Arc;

/// Orca Whirlpool account parser
#[derive(Clone)]
pub struct OrcaAccountParser {
    vault_reader: Arc<VaultReader>,
}

impl OrcaAccountParser {
    /// Create new Orca account parser
    pub fn new(vault_reader: Arc<VaultReader>) -> Self {
        Self { vault_reader }
    }
    /// Parse Orca Whirlpool pool account with real vault balances
    pub async fn parse_pool_account_with_balances(
        &self,
        account_data: &[u8],
        vault_reader: &VaultReader,
    ) -> Result<PoolInfo, AppError> {
        // Try to deserialize as Whirlpool
        let whirlpool = Whirlpool::try_deserialize(account_data)
            .map_err(|e| AppError::BlockchainError(format!("Failed to deserialize Whirlpool: {}", e)))?;
        
        // Get real vault balances
        let (balance_a, balance_b) = vault_reader
            .get_pool_vault_balances(
                &whirlpool.token_vault_a.to_string(),
                &whirlpool.token_vault_b.to_string(),
            )
            .await?;
        
        // Get token metadata
        let token_a_meta = vault_reader.get_token_metadata(&whirlpool.token_mint_a.to_string()).await?;
        let token_b_meta = vault_reader.get_token_metadata(&whirlpool.token_mint_b.to_string()).await?;
        
        // Create pool info from real data
        let pool_info = PoolInfo {
            id: "orca_pool".to_string(), // Will be updated with actual pubkey
            dex_type: DexType::OrcaWhirlpool,
            token_a: Token {
                mint: whirlpool.token_mint_a,
                symbol: token_a_meta.symbol,
                decimals: token_a_meta.decimals,
                name: Some(token_a_meta.name),
            },
            token_b: Token {
                mint: whirlpool.token_mint_b,
                symbol: token_b_meta.symbol,
                decimals: token_b_meta.decimals,
                name: Some(token_b_meta.name),
            },
            reserve_a: Amount::new(balance_a, token_a_meta.decimals),
            reserve_b: Amount::new(balance_b, token_b_meta.decimals),
            fee_rate: whirlpool.get_fee_rate_percentage(),
            liquidity: Amount::new(whirlpool.liquidity as u64, 6),
            volume_24h: Amount::new(0, 6), // TODO: Calculate from recent transactions
        };
        
        Ok(pool_info)
    }
    
    /// Parse Orca Whirlpool pool account (legacy method)
    pub async fn parse_pool_account(&self, account_data: &[u8]) -> Result<PoolInfo, AppError> {

        
        // Try to deserialize as Whirlpool
        let whirlpool = Whirlpool::try_deserialize(account_data)
            .map_err(|e| AppError::BlockchainError(format!("Failed to deserialize Whirlpool: {}", e)))?;
        
        // Get token metadata for real symbols
        let token_a_mint = whirlpool.token_mint_a.to_string();
        let token_b_mint = whirlpool.token_mint_b.to_string();
        
        let token_a_meta = self.vault_reader.get_token_metadata(&token_a_mint).await
            .unwrap_or_else(|_| {
                // Use fallback symbol from known tokens
                let symbol = self.vault_reader.get_known_token_symbol(&token_a_mint);
                TokenMetadata {
                    symbol,
                    decimals: 9,
                    name: "Unknown Token A".to_string(),
                }
            });
        
        let token_b_meta = self.vault_reader.get_token_metadata(&token_b_mint).await
            .unwrap_or_else(|_| {
                // Use fallback symbol from known tokens
                let symbol = self.vault_reader.get_known_token_symbol(&token_b_mint);
                TokenMetadata {
                    symbol,
                    decimals: 6,
                    name: "Unknown Token B".to_string(),
                }
            });
        
        // Get real vault balances
        let (balance_a, balance_b) = self.vault_reader
            .get_pool_vault_balances(
                &whirlpool.token_vault_a.to_string(),
                &whirlpool.token_vault_b.to_string(),
            )
            .await
            .unwrap_or((0, 0)); // Fallback to 0 if vault reading fails
        
        // Create pool info from real data
        let pool_info = PoolInfo {
            id: "orca_pool".to_string(), // Will be updated with actual pubkey
            dex_type: DexType::OrcaWhirlpool,
            token_a: Token {
                mint: whirlpool.token_mint_a,
                symbol: token_a_meta.symbol,
                decimals: token_a_meta.decimals,
                name: Some(token_a_meta.name),
            },
            token_b: Token {
                mint: whirlpool.token_mint_b,
                symbol: token_b_meta.symbol,
                decimals: token_b_meta.decimals,
                name: Some(token_b_meta.name),
            },
            reserve_a: Amount::new(balance_a, token_a_meta.decimals),
            reserve_b: Amount::new(balance_b, token_b_meta.decimals),
            fee_rate: whirlpool.get_fee_rate_percentage(),
            liquidity: Amount::new(whirlpool.liquidity as u64, 6),
            volume_24h: Amount::new(0, 6), // TODO: Calculate from recent transactions
        };
        
        Ok(pool_info)
    }
    
    /// Check if account is a valid Orca pool
    pub fn is_pool_account(&self, account_data: &[u8]) -> bool {
        let is_valid = Whirlpool::is_valid_whirlpool(account_data);
        if !is_valid && account_data.len() >= 8 {
            let discriminator: [u8; 8] = account_data[0..8].try_into().unwrap_or([0; 8]);
            println!("🔍 Debug: Account discriminator: {:02x?} (expected: {:02x?})", discriminator, [0x0b, 0x0b, 0x0b, 0x0b, 0x0b, 0x0b, 0x0b, 0x0b]);
        }
        is_valid
    }
}

/// Raydium V4 account parser
pub struct RaydiumAccountParser;

impl RaydiumAccountParser {
    /// Create new Raydium account parser
    pub fn new() -> Self {
        Self
    }
}

impl RaydiumAccountParser {
    /// Parse Raydium AMM pool account with real vault balances
    pub async fn parse_pool_account_with_balances(
        &self,
        account_data: &[u8],
        vault_reader: &VaultReader,
    ) -> Result<PoolInfo, AppError> {
        // Try to deserialize as RaydiumAMMPool
        let raydium_pool = RaydiumAMMPool::try_deserialize(account_data)
            .map_err(|e| AppError::BlockchainError(format!("Failed to deserialize Raydium AMM pool: {}", e)))?;
        
        // Get real vault balances
        let (balance_a, balance_b) = vault_reader
            .get_pool_vault_balances(
                &raydium_pool.base_vault.to_string(),
                &raydium_pool.quote_vault.to_string(),
            )
            .await?;
        
        // Get token metadata
        let token_a_meta = vault_reader.get_token_metadata(&raydium_pool.base_mint.to_string()).await?;
        let token_b_meta = vault_reader.get_token_metadata(&raydium_pool.quote_mint.to_string()).await?;
        
        // Create pool info from real data
        let pool_info = PoolInfo {
            id: "raydium_pool".to_string(), // Will be updated with actual pubkey
            dex_type: DexType::RaydiumAMM,
            token_a: Token {
                mint: raydium_pool.base_mint,
                symbol: token_a_meta.symbol,
                decimals: token_a_meta.decimals,
                name: Some(token_a_meta.name),
            },
            token_b: Token {
                mint: raydium_pool.quote_mint,
                symbol: token_b_meta.symbol,
                decimals: token_b_meta.decimals,
                name: Some(token_b_meta.name),
            },
            reserve_a: Amount::new(balance_a, token_a_meta.decimals),
            reserve_b: Amount::new(balance_b, token_b_meta.decimals),
            fee_rate: raydium_pool.trade_fee_percentage(),
            liquidity: Amount::new(raydium_pool.total_liquidity() as u64, 6),
            volume_24h: Amount::new(0, 6), // TODO: Calculate from recent transactions
        };
        
        Ok(pool_info)
    }
    
    /// Parse Raydium AMM pool account (legacy method)
    pub fn parse_pool_account(&self, account_data: &[u8]) -> Result<PoolInfo, AppError> {
        // Try to deserialize as RaydiumAMMPool
        let raydium_pool = RaydiumAMMPool::try_deserialize(account_data)
            .map_err(|e| AppError::BlockchainError(format!("Failed to deserialize Raydium AMM pool: {}", e)))?;
        
        // Create pool info from real data
        let pool_info = PoolInfo {
            id: "raydium_pool".to_string(), // Will be updated with actual pubkey
            dex_type: DexType::RaydiumAMM,
            token_a: Token {
                mint: raydium_pool.base_mint,
                symbol: "TOKEN_A".to_string(), // TODO: Get from token metadata
                decimals: raydium_pool.base_decimals as u8,
                name: None,
            },
            token_b: Token {
                mint: raydium_pool.quote_mint,
                symbol: "TOKEN_B".to_string(), // TODO: Get from token metadata
                decimals: raydium_pool.quote_decimals as u8,
                name: None,
            },
            reserve_a: Amount::new(0, 0), // TODO: Get from vault balance
            reserve_b: Amount::new(0, 0), // TODO: Get from vault balance
            fee_rate: raydium_pool.trade_fee_percentage(),
            liquidity: Amount::new(raydium_pool.total_liquidity() as u64, 6),
            volume_24h: Amount::new(0, 6), // TODO: Calculate from recent transactions
        };
        
        Ok(pool_info)
    }
    
    /// Check if account is a valid Raydium pool
    pub fn is_pool_account(&self, account_data: &[u8]) -> bool {
        RaydiumAMMPool::try_deserialize(account_data).is_ok()
    }
}

/// Generic account parser trait
pub trait AccountParser {
    fn parse_pool_account(&self, account_data: &[u8]) -> Result<PoolInfo, AppError>;
    fn is_pool_account(&self, account_data: &[u8]) -> bool;
}

impl AccountParser for OrcaAccountParser {
    fn parse_pool_account(&self, account_data: &[u8]) -> Result<PoolInfo, AppError> {
        // This is a sync wrapper around the async method
        // In production, this should be handled differently
        Err(AppError::BlockchainError("Use parse_pool_account_async for Orca".to_string()))
    }
    
    fn is_pool_account(&self, account_data: &[u8]) -> bool {
        // Call the actual implementation
        OrcaAccountParser::is_pool_account(self, account_data)
    }
}

impl AccountParser for RaydiumAccountParser {
    fn parse_pool_account(&self, account_data: &[u8]) -> Result<PoolInfo, AppError> {
        // Call the actual implementation
        RaydiumAccountParser::parse_pool_account(self, account_data)
    }
    
    fn is_pool_account(&self, account_data: &[u8]) -> bool {
        // Call the actual implementation
        RaydiumAccountParser::is_pool_account(self, account_data)
    }
}
