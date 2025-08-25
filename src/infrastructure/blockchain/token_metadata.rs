//! Token metadata service for Solana tokens

use solana_sdk::pubkey::Pubkey;
use solana_client::rpc_client::RpcClient;
use crate::shared::errors::AppError;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use solana_sdk::account::Account;
use solana_sdk::commitment_config::CommitmentConfig;
use std::str::FromStr;

/// Token metadata from Solana Token Metadata Program
#[derive(Debug, Clone)]
pub struct TokenMetadata {
    pub mint: String,
    pub symbol: String,
    pub name: String,
    pub decimals: u8,
    pub uri: Option<String>,
    pub logo_uri: Option<String>,
    pub verified: bool,
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

/// Enhanced token metadata service with real blockchain integration
#[derive(Clone)]
pub struct TokenMetadataService {
    rpc_client: Arc<RpcClient>,
    cache: Arc<RwLock<HashMap<String, TokenMetadata>>>,
    known_tokens: HashMap<String, TokenMetadata>,
    cache_ttl: std::time::Duration,
}

impl TokenMetadataService {
    /// Create new token metadata service
    pub fn new(rpc_client: Arc<RpcClient>) -> Self {
        let mut known_tokens = HashMap::new();
        
        // Add known tokens with their metadata
        known_tokens.insert(
            "So111111111111111111111111111111111111111111111111111111111111111111".to_string(),
            TokenMetadata {
                mint: "So111111111111111111111111111111111111111111111111111111111111111111".to_string(),
                symbol: "SOL".to_string(),
                name: "Wrapped SOL".to_string(),
                decimals: 9,
                uri: None,
                logo_uri: None,
                verified: true,
                last_updated: chrono::Utc::now(),
            },
        );
        
        // Add system tokens
        known_tokens.insert(
            "111111111111111111111111111111111111111111111111111111111111111111".to_string(),
            TokenMetadata {
                mint: "111111111111111111111111111111111111111111111111111111111111111111".to_string(),
                symbol: "SOL".to_string(),
                name: "Native SOL".to_string(),
                decimals: 9,
                uri: None,
                logo_uri: None,
                verified: true,
                last_updated: chrono::Utc::now(),
            },
        );
        
        known_tokens.insert(
            "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
            TokenMetadata {
                mint: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
                symbol: "USDC".to_string(),
                name: "USD Coin".to_string(),
                decimals: 6,
                uri: None,
                logo_uri: None,
                verified: true,
                last_updated: chrono::Utc::now(),
            },
        );
        
        known_tokens.insert(
            "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB".to_string(),
            TokenMetadata {
                mint: "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB".to_string(),
                symbol: "USDT".to_string(),
                name: "Tether USD".to_string(),
                decimals: 6,
                uri: None,
                logo_uri: None,
                verified: true,
                last_updated: chrono::Utc::now(),
            },
        );
        
        known_tokens.insert(
            "mSoLzYCxHdYgdzU16g5QSh3i5K3z3KZK7ytfqcJm7So".to_string(),
            TokenMetadata {
                mint: "mSoLzYCxHdYgdzU16g5QSh3i5K3z3KZK7ytfqcJm7So".to_string(),
                symbol: "mSOL".to_string(),
                name: "Marinade Staked SOL".to_string(),
                decimals: 9,
                uri: None,
                logo_uri: None,
                verified: true,
                last_updated: chrono::Utc::now(),
            },
        );
        
        known_tokens.insert(
            "7dHbWXmci3dT8UFYWYZweBLXgycu7Y3iL6trKn1Y7ARj".to_string(),
            TokenMetadata {
                mint: "7dHbWXmci3dT8UFYWYZweBLXgycu7Y3iL6trKn1Y7ARj".to_string(),
                symbol: "stSOL".to_string(),
                name: "Lido Staked SOL".to_string(),
                decimals: 9,
                uri: None,
                logo_uri: None,
                verified: true,
                last_updated: chrono::Utc::now(),
            },
        );
        
        known_tokens.insert(
            "DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263".to_string(),
            TokenMetadata {
                mint: "DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263".to_string(),
                symbol: "BONK".to_string(),
                name: "Bonk".to_string(),
                decimals: 5,
                uri: None,
                logo_uri: None,
                verified: true,
                last_updated: chrono::Utc::now(),
            },
        );

        Self {
            rpc_client,
            cache: Arc::new(RwLock::new(HashMap::new())),
            known_tokens,
            cache_ttl: std::time::Duration::from_secs(3600), // 1 hour TTL
        }
    }

    /// Get token metadata by mint address with enhanced caching
    pub async fn get_token_metadata(&self, mint: &str) -> Result<TokenMetadata, AppError> {
        // Check cache first
        {
            let cache = self.cache.read().await;
            if let Some(metadata) = cache.get(mint) {
                // Check if cache is still valid
                if chrono::Utc::now() - metadata.last_updated < chrono::Duration::from_std(self.cache_ttl).unwrap() {
                    return Ok(metadata.clone());
                }
            }
        }

        // Check known tokens
        if let Some(metadata) = self.known_tokens.get(mint) {
            // Cache the result
            let mut cache = self.cache.write().await;
            cache.insert(mint.to_string(), metadata.clone());
            return Ok(metadata.clone());
        }

        // Try to fetch from Token Metadata Program
        match self.fetch_token_metadata_from_chain(mint).await {
            Ok(metadata) => {
                // Cache the result
                let mut cache = self.cache.write().await;
                cache.insert(mint.to_string(), metadata.clone());
                Ok(metadata)
            }
            Err(_) => {
                // Try to fetch basic token info from SPL Token Program
                match self.fetch_basic_token_info(mint).await {
                    Ok(metadata) => {
                        // Cache the result
                        let mut cache = self.cache.write().await;
                        cache.insert(mint.to_string(), metadata.clone());
                        Ok(metadata)
                    }
                    Err(_) => {
                        // Fallback to generating a readable name
                        let fallback_metadata = self.generate_fallback_metadata(mint);
                        
                        // Cache the fallback result
                        let mut cache = self.cache.write().await;
                        cache.insert(mint.to_string(), fallback_metadata.clone());
                        
                        Ok(fallback_metadata)
                    }
                }
            }
        }
    }

    /// Fetch token metadata from Solana Token Metadata Program
    async fn fetch_token_metadata_from_chain(&self, mint: &str) -> Result<TokenMetadata, AppError> {
        // Token Metadata Program ID
        let metadata_program_id = "metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s";
        
        // Find the Metadata PDA for the mint
        let mint_pubkey = Pubkey::from_str(mint)
            .map_err(|e| AppError::BlockchainError(format!("Invalid mint address: {}", e)))?;
        
        let metadata_program_pubkey = Pubkey::from_str(metadata_program_id)
            .map_err(|e| AppError::BlockchainError(format!("Invalid metadata program ID: {}", e)))?;
        
        // Derive metadata account address
        let (metadata_account, _) = Pubkey::find_program_address(
            &[
                b"metadata",
                metadata_program_pubkey.as_ref(),
                mint_pubkey.as_ref(),
            ],
            &metadata_program_pubkey,
        );

        // Fetch metadata account
        let account = self.rpc_client
            .get_account_with_commitment(&metadata_account, CommitmentConfig::confirmed())
            .map_err(|e| AppError::BlockchainError(format!("Failed to fetch metadata account: {}", e)))?
            .value
            .ok_or_else(|| AppError::BlockchainError("Metadata account not found".to_string()))?;

        // Parse metadata (simplified - in production you'd use proper deserialization)
        let metadata = self.parse_metadata_account(&account.data)?;
        
        Ok(metadata)
    }

    /// Fetch basic token info from SPL Token Program
    async fn fetch_basic_token_info(&self, mint: &str) -> Result<TokenMetadata, AppError> {
        let mint_pubkey = Pubkey::from_str(mint)
            .map_err(|e| AppError::BlockchainError(format!("Invalid mint address: {}", e)))?;

        // Fetch mint account
        let account = self.rpc_client
            .get_account_with_commitment(&mint_pubkey, CommitmentConfig::confirmed())
            .map_err(|e| AppError::BlockchainError(format!("Failed to fetch mint account: {}", e)))?
            .value
            .ok_or_else(|| AppError::BlockchainError("Mint account not found".to_string()))?;

        // Parse mint account data
        let decimals = if account.data.len() >= 45 {
            account.data[44] // decimals field in SPL Token mint account
        } else {
            6 // fallback
        };

        // Try to get symbol from mint address (common pattern)
        let symbol = self.extract_symbol_from_mint(mint);

        Ok(TokenMetadata {
            mint: mint.to_string(),
            symbol,
            name: format!("Token {}", &mint[..8]),
            decimals,
            uri: None,
            logo_uri: None,
            verified: false,
            last_updated: chrono::Utc::now(),
        })
    }

    /// Parse metadata account data
    fn parse_metadata_account(&self, data: &[u8]) -> Result<TokenMetadata, AppError> {
        // This is a simplified parser for Token Metadata Program
        // In production, you'd use proper Anchor or Borsh deserialization
        
        if data.len() < 1 {
            return Err(AppError::BlockchainError("Invalid metadata account data".to_string()));
        }

        // Extract basic info (this is a simplified version)
        let symbol = if data.len() >= 9 {
            let symbol_len = data[8] as usize;
            if data.len() >= 9 + symbol_len {
                String::from_utf8_lossy(&data[9..9+symbol_len]).to_string()
            } else {
                "UNKNOWN".to_string()
            }
        } else {
            "UNKNOWN".to_string()
        };

        let name = if data.len() >= 9 + symbol.len() + 1 {
            let name_len = data[9 + symbol.len()] as usize;
            if data.len() >= 9 + symbol.len() + 1 + name_len {
                String::from_utf8_lossy(&data[9 + symbol.len() + 1..9 + symbol.len() + 1 + name_len]).to_string()
            } else {
                format!("Token {}", symbol)
            }
        } else {
            format!("Token {}", symbol)
        };

        Ok(TokenMetadata {
            mint: "".to_string(), // Will be set by caller
            symbol,
            name,
            decimals: 6, // Default
            uri: None,
            logo_uri: None,
            verified: true,
            last_updated: chrono::Utc::now(),
        })
    }

    /// Extract symbol from mint address using common patterns
    fn extract_symbol_from_mint(&self, mint: &str) -> String {
        // Handle system tokens first
        if mint == "111111111111111111111111111111111111111111111111111111111111111111" {
            return "SOL".to_string();
        }
        
        // Common token patterns
        if mint.len() >= 8 {
            let prefix = &mint[..8];
            
            // Check for known patterns
            match prefix {
                "EPjFWdd" => "USDC".to_string(),
                "Es9vMFr" => "USDT".to_string(),
                "So11111" => "SOL".to_string(),
                "mSoLzYC" => "mSOL".to_string(),
                "7dHbWXm" => "stSOL".to_string(),
                "DezXAZ8" => "BONK".to_string(),
                "11111111" => "SOL".to_string(), // System token prefix
                _ => {
                    // Try to extract a meaningful symbol from the mint
                    if mint.len() >= 12 {
                        let short_prefix = &mint[..12];
                        format!("TOKEN_{}", short_prefix.to_uppercase())
                    } else {
                        format!("TOKEN_{}", prefix.to_uppercase())
                    }
                }
            }
        } else {
            "UNKNOWN".to_string()
        }
    }

    /// Generate fallback metadata when on-chain data is not available
    fn generate_fallback_metadata(&self, mint: &str) -> TokenMetadata {
        let symbol = self.extract_symbol_from_mint(mint);
        let name = if mint.len() >= 8 {
            format!("Token {}", &mint[..8])
        } else {
            "Unknown Token".to_string()
        };

        TokenMetadata {
            mint: mint.to_string(),
            symbol,
            name,
            decimals: 6, // Default to 6 decimals
            uri: None,
            logo_uri: None,
            verified: false,
            last_updated: chrono::Utc::now(),
        }
    }

    /// Get cached token metadata
    pub async fn get_cached_metadata(&self, mint: &str) -> Option<TokenMetadata> {
        let cache = self.cache.read().await;
        cache.get(mint).cloned()
    }

    /// Clear cache
    pub async fn clear_cache(&self) {
        let mut cache = self.cache.write().await;
        cache.clear();
    }

    /// Get cache size
    pub async fn get_cache_size(&self) -> usize {
        let cache = self.cache.read().await;
        cache.len()
    }

    /// Refresh token metadata from blockchain
    pub async fn refresh_token_metadata(&self, mint: &str) -> Result<TokenMetadata, AppError> {
        // Remove from cache to force refresh
        {
            let mut cache = self.cache.write().await;
            cache.remove(mint);
        }
        
        // Fetch fresh data
        self.get_token_metadata(mint).await
    }

    /// Batch fetch multiple token metadata
    pub async fn batch_get_token_metadata(&self, mints: &[String]) -> Result<Vec<TokenMetadata>, AppError> {
        let mut results = Vec::new();
        
        for mint in mints {
            match self.get_token_metadata(mint).await {
                Ok(metadata) => results.push(metadata),
                Err(e) => {
                    // Log error but continue with other tokens
                    eprintln!("Failed to fetch metadata for {}: {}", mint, e);
                    // Add fallback metadata
                    results.push(self.generate_fallback_metadata(mint));
                }
            }
        }
        
        Ok(results)
    }

    /// Get token symbol with fallback
    pub async fn get_token_symbol(&self, mint: &str) -> String {
        match self.get_token_metadata(mint).await {
            Ok(metadata) => metadata.symbol,
            Err(_) => self.extract_symbol_from_mint(mint),
        }
    }

    /// Get token name with fallback
    pub async fn get_token_name(&self, mint: &str) -> String {
        match self.get_token_metadata(mint).await {
            Ok(metadata) => metadata.name,
            Err(_) => {
                let symbol = self.extract_symbol_from_mint(mint);
                format!("Token {}", symbol)
            }
        }
    }
}
