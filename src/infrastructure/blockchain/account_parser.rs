//! Account parsers for DEX programs

use solana_sdk::pubkey::Pubkey;
use crate::shared::types::{Token, Amount};
use crate::domain::dex::{PoolInfo, DexType};
use crate::shared::errors::AppError;
use super::{Whirlpool, RaydiumV4Pool, VaultReader};

/// Orca Whirlpool account parser
pub struct OrcaAccountParser;

impl OrcaAccountParser {
    /// Create new Orca account parser
    pub fn new() -> Self {
        Self
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
    pub fn parse_pool_account(&self, account_data: &[u8]) -> Result<PoolInfo, AppError> {
        // Try to deserialize as Whirlpool
        let whirlpool = Whirlpool::try_deserialize(account_data)
            .map_err(|e| AppError::BlockchainError(format!("Failed to deserialize Whirlpool: {}", e)))?;
        
        // Create pool info from real data
        let pool_info = PoolInfo {
            id: "orca_pool".to_string(), // Will be updated with actual pubkey
            dex_type: DexType::OrcaWhirlpool,
            token_a: Token {
                mint: whirlpool.token_mint_a,
                symbol: "TOKEN_A".to_string(), // TODO: Get from token metadata
                decimals: 9, // TODO: Get from token metadata
                name: None,
            },
            token_b: Token {
                mint: whirlpool.token_mint_b,
                symbol: "TOKEN_B".to_string(), // TODO: Get from token metadata
                decimals: 6, // TODO: Get from token metadata
                name: None,
            },
            reserve_a: Amount::new(0, 9), // TODO: Get from vault balance
            reserve_b: Amount::new(0, 6), // TODO: Get from vault balance
            fee_rate: whirlpool.get_fee_rate_percentage(),
            liquidity: Amount::new(whirlpool.liquidity as u64, 6),
            volume_24h: Amount::new(0, 6), // TODO: Calculate from recent transactions
        };
        
        Ok(pool_info)
    }
    
    /// Check if account is a valid Orca pool
    pub fn is_pool_account(&self, account_data: &[u8]) -> bool {
        Whirlpool::is_valid_whirlpool(account_data)
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
    /// Parse Raydium V4 pool account with real vault balances
    pub async fn parse_pool_account_with_balances(
        &self,
        account_data: &[u8],
        vault_reader: &VaultReader,
    ) -> Result<PoolInfo, AppError> {
        // Try to deserialize as RaydiumV4Pool
        let raydium_pool = RaydiumV4Pool::try_deserialize(account_data)
            .map_err(|e| AppError::BlockchainError(format!("Failed to deserialize Raydium V4 pool: {}", e)))?;
        
        // Get real vault balances
        let (balance_a, balance_b) = vault_reader
            .get_pool_vault_balances(
                &raydium_pool.token_vault_a.to_string(),
                &raydium_pool.token_vault_b.to_string(),
            )
            .await?;
        
        // Get token metadata
        let token_a_meta = vault_reader.get_token_metadata(&raydium_pool.token_mint_a.to_string()).await?;
        let token_b_meta = vault_reader.get_token_metadata(&raydium_pool.token_mint_b.to_string()).await?;
        
        // Create pool info from real data
        let pool_info = PoolInfo {
            id: "raydium_pool".to_string(), // Will be updated with actual pubkey
            dex_type: DexType::RaydiumV4,
            token_a: Token {
                mint: raydium_pool.token_mint_a,
                symbol: token_a_meta.symbol,
                decimals: token_a_meta.decimals,
                name: Some(token_a_meta.name),
            },
            token_b: Token {
                mint: raydium_pool.token_mint_b,
                symbol: token_b_meta.symbol,
                decimals: token_b_meta.decimals,
                name: Some(token_b_meta.name),
            },
            reserve_a: Amount::new(balance_a, token_a_meta.decimals),
            reserve_b: Amount::new(balance_b, token_b_meta.decimals),
            fee_rate: raydium_pool.get_fee_rate_percentage(),
            liquidity: Amount::new(raydium_pool.liquidity as u64, 6),
            volume_24h: Amount::new(0, 6), // TODO: Calculate from recent transactions
        };
        
        Ok(pool_info)
    }
    
    /// Parse Raydium V4 pool account (legacy method)
    pub fn parse_pool_account(&self, account_data: &[u8]) -> Result<PoolInfo, AppError> {
        // Try to deserialize as RaydiumV4Pool
        let raydium_pool = RaydiumV4Pool::try_deserialize(account_data)
            .map_err(|e| AppError::BlockchainError(format!("Failed to deserialize Raydium V4 pool: {}", e)))?;
        
        // Create pool info from real data
        let pool_info = PoolInfo {
            id: "raydium_pool".to_string(), // Will be updated with actual pubkey
            dex_type: DexType::RaydiumV4,
            token_a: Token {
                mint: raydium_pool.token_mint_a,
                symbol: "TOKEN_A".to_string(), // TODO: Get from token metadata
                decimals: 9, // TODO: Get from token metadata
                name: None,
            },
            token_b: Token {
                mint: raydium_pool.token_mint_b,
                symbol: "TOKEN_B".to_string(), // TODO: Get from token metadata
                decimals: 6, // TODO: Get from token metadata
                name: None,
            },
            reserve_a: Amount::new(0, 0), // TODO: Get from vault balance
            reserve_b: Amount::new(0, 0), // TODO: Get from vault balance
            fee_rate: raydium_pool.get_fee_rate_percentage(),
            liquidity: Amount::new(raydium_pool.liquidity as u64, 6),
            volume_24h: Amount::new(0, 6), // TODO: Calculate from recent transactions
        };
        
        Ok(pool_info)
    }
    
    /// Check if account is a valid Raydium pool
    pub fn is_pool_account(&self, account_data: &[u8]) -> bool {
        RaydiumV4Pool::try_deserialize(account_data).is_ok()
    }
}

/// Generic account parser trait
pub trait AccountParser {
    fn parse_pool_account(&self, account_data: &[u8]) -> Result<PoolInfo, AppError>;
    fn is_pool_account(&self, account_data: &[u8]) -> bool;
}

impl AccountParser for OrcaAccountParser {
    fn parse_pool_account(&self, account_data: &[u8]) -> Result<PoolInfo, AppError> {
        self.parse_pool_account(account_data)
    }
    
    fn is_pool_account(&self, account_data: &[u8]) -> bool {
        self.is_pool_account(account_data)
    }
}

impl AccountParser for RaydiumAccountParser {
    fn parse_pool_account(&self, account_data: &[u8]) -> Result<PoolInfo, AppError> {
        self.parse_pool_account(account_data)
    }
    
    fn is_pool_account(&self, account_data: &[u8]) -> bool {
        self.is_pool_account(account_data)
    }
}
