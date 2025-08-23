//! Token metadata reader using Token Metadata Program

use solana_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use crate::shared::errors::AppError;
use std::str::FromStr;

/// Token Metadata Program ID
pub const TOKEN_METADATA_PROGRAM_ID: &str = "metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s";

/// Token metadata structure
#[derive(Debug, Clone)]
pub struct TokenMetadata {
    pub symbol: String,
    pub name: String,
    pub decimals: u8,
    pub uri: Option<String>,
    pub seller_fee_basis_points: Option<u16>,
    pub creators: Option<Vec<Creator>>,
}

/// Creator information
#[derive(Debug, Clone)]
pub struct Creator {
    pub address: Pubkey,
    pub verified: bool,
    pub share: u8,
}

/// Token metadata reader
pub struct TokenMetadataReader {
    rpc_client: RpcClient,
}

impl TokenMetadataReader {
    /// Create new token metadata reader
    pub fn new(rpc_url: String) -> Self {
        Self {
            rpc_client: RpcClient::new(rpc_url),
        }
    }
    
    /// Create new token metadata reader with default mainnet RPC
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
    
    /// Get token metadata by mint address
    pub async fn get_token_metadata(&self, mint: &str) -> Result<TokenMetadata, AppError> {
        let mint_pubkey = Pubkey::from_str(mint)
            .map_err(|e| AppError::BlockchainError(format!("Invalid mint: {}", e)))?;
            
        // Derive metadata account address
        let metadata_account = self.derive_metadata_account(&mint_pubkey)?;
        
        // Get metadata account data
        let account = self.rpc_client.get_account(&metadata_account)
            .map_err(|e| AppError::BlockchainError(format!("Failed to get metadata account: {}", e)))?;
            
        // Parse metadata (simplified for now)
        let metadata = self.parse_metadata_account(&account.data)?;
        
        Ok(metadata)
    }
    
    /// Derive metadata account address for a mint
    fn derive_metadata_account(&self, mint: &Pubkey) -> Result<Pubkey, AppError> {
        let metadata_program = Pubkey::from_str(TOKEN_METADATA_PROGRAM_ID)
            .map_err(|e| AppError::BlockchainError(format!("Invalid metadata program ID: {}", e)))?;
            
        let seeds = &[
            b"metadata".as_ref(),
            metadata_program.as_ref(),
            mint.as_ref(),
        ];
        
        let (metadata_account, _bump) = Pubkey::find_program_address(seeds, &metadata_program);
            
        Ok(metadata_account)
    }
    
    /// Parse metadata account data
    fn parse_metadata_account(&self, data: &[u8]) -> Result<TokenMetadata, AppError> {
        // Simplified parsing - in production, use proper borsh deserialization
        if data.len() < 1 {
            return Err(AppError::BlockchainError("Invalid metadata account data".to_string()));
        }
        
        // For now, return basic metadata
        // TODO: Implement proper borsh deserialization
        Ok(TokenMetadata {
            symbol: "TOKEN".to_string(),
            name: "Unknown Token".to_string(),
            decimals: 9, // Default
            uri: None,
            seller_fee_basis_points: None,
            creators: None,
        })
    }
    
    /// Get multiple token metadata
    pub async fn get_multiple_token_metadata(&self, mints: &[String]) -> Result<Vec<TokenMetadata>, AppError> {
        let mut metadata_list = Vec::new();
        
        for mint in mints {
            match self.get_token_metadata(mint).await {
                Ok(metadata) => metadata_list.push(metadata),
                Err(e) => {
                    eprintln!("Failed to get metadata for {}: {}", mint, e);
                    // Add default metadata on error
                    metadata_list.push(TokenMetadata {
                        symbol: "UNKNOWN".to_string(),
                        name: "Unknown Token".to_string(),
                        decimals: 9,
                        uri: None,
                        seller_fee_basis_points: None,
                        creators: None,
                    });
                }
            }
        }
        
        Ok(metadata_list)
    }
}
