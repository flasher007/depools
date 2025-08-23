//! Vault balance reader for real token reserves

use solana_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::program_pack::Pack;
use spl_token::state::Account as TokenAccount;
use spl_token::state::Mint;
use crate::shared::errors::AppError;
use crate::shared::types::Amount;
use super::token_metadata::TokenMetadataReader;

/// Service for reading real vault balances
pub struct VaultReader {
    rpc_client: RpcClient,
    token_metadata: TokenMetadataReader,
}

impl VaultReader {
    /// Create new vault reader
    pub fn new(rpc_url: String) -> Self {
        Self {
            rpc_client: RpcClient::new(rpc_url.clone()),
            token_metadata: TokenMetadataReader::new(rpc_url),
        }
    }
    
    /// Create new vault reader with default mainnet RPC
    pub fn new_default() -> Self {
        Self::new_mainnet()
    }
    
    /// Create with default mainnet RPC
    pub fn new_mainnet() -> Self {
        Self::new("https://api.mainnet-beta.solana.com".to_string())
    }
    
    /// Create with devnet RPC
    pub fn new_devnet() -> Self {
        Self::new("https://api.devnet.solana.com".to_string())
    }
    
    /// Get token account balance
    pub async fn get_token_account_balance(&self, token_account: &str) -> Result<u64, AppError> {
        let pubkey = Pubkey::from_str(token_account)
            .map_err(|e| AppError::BlockchainError(format!("Invalid token account: {}", e)))?;
            
        let account = self.rpc_client.get_account(&pubkey)
            .map_err(|e| AppError::BlockchainError(format!("Failed to get token account: {}", e)))?;
            
        // Parse token account data
        let token_account_data = TokenAccount::unpack(&account.data)
            .map_err(|e| AppError::BlockchainError(format!("Failed to parse token account: {}", e)))?;
            
        Ok(token_account_data.amount)
    }
    
    /// Get token balance with metadata
    pub async fn get_token_balance(&self, mint: &str) -> Result<TokenBalance, AppError> {
        // For now, return a simulated balance
        // In real implementation, this would query actual token accounts
        let balance = TokenBalance {
            value: 1000000000, // 1 SOL in lamports
            decimals: 9,
            symbol: "SOL".to_string(),
        };
        
        Ok(balance)
    }
    
    /// Get multiple token account balances
    pub async fn get_multiple_token_balances(&self, token_accounts: &[String]) -> Result<Vec<u64>, AppError> {
        let mut balances = Vec::new();
        
        for account in token_accounts {
            match self.get_token_account_balance(account).await {
                Ok(balance) => balances.push(balance),
                Err(e) => {
                    eprintln!("Failed to get balance for {}: {}", account, e);
                    balances.push(0); // Default to 0 on error
                }
            }
        }
        
        Ok(balances)
    }
    
    /// Get vault balances for a pool
    pub async fn get_pool_vault_balances(
        &self,
        vault_a: &str,
        vault_b: &str,
    ) -> Result<(u64, u64), AppError> {
        let balance_a = self.get_token_account_balance(vault_a).await?;
        let balance_b = self.get_token_account_balance(vault_b).await?;
        
        Ok((balance_a, balance_b))
    }
    
    /// Get token metadata (decimals, symbol, name)
    pub async fn get_token_metadata(&self, mint: &str) -> Result<TokenMetadata, AppError> {
        // Get mint account for decimals
        let pubkey = Pubkey::from_str(mint)
            .map_err(|e| AppError::BlockchainError(format!("Invalid mint: {}", e)))?;
            
        let mint_account = self.rpc_client.get_account(&pubkey)
            .map_err(|e| AppError::BlockchainError(format!("Failed to get mint account: {}", e)))?;
            
        let mint_data = Mint::unpack(&mint_account.data)
            .map_err(|e| AppError::BlockchainError(format!("Failed to parse mint: {}", e)))?;
            
        // Get full metadata from Token Metadata Program
        match self.token_metadata.get_token_metadata(mint).await {
            Ok(metadata) => {
                Ok(TokenMetadata {
                    decimals: mint_data.decimals,
                    symbol: metadata.symbol,
                    name: metadata.name,
                })
            }
            Err(_) => {
                // Fallback to basic metadata
                Ok(TokenMetadata {
                    decimals: mint_data.decimals,
                    symbol: "UNKNOWN".to_string(),
                    name: "Unknown Token".to_string(),
                })
            }
        }
    }
}

/// Token metadata structure
#[derive(Debug, Clone)]
pub struct TokenMetadata {
    pub decimals: u8,
    pub symbol: String,
    pub name: String,
}

/// Token balance structure
#[derive(Debug, Clone)]
pub struct TokenBalance {
    pub value: u64,
    pub decimals: u8,
    pub symbol: String,
}

use std::str::FromStr;
