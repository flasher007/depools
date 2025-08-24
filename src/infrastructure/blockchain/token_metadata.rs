//! Token metadata service for Solana tokens

use solana_sdk::pubkey::Pubkey;
use solana_client::rpc_client::RpcClient;
use crate::shared::errors::AppError;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Token metadata from Solana Token Metadata Program
#[derive(Debug, Clone)]
pub struct TokenMetadata {
    pub mint: String,
    pub symbol: String,
    pub name: String,
    pub decimals: u8,
    pub uri: Option<String>,
    pub logo_uri: Option<String>,
}

/// Enhanced token metadata service
#[derive(Clone)]
pub struct TokenMetadataService {
    rpc_client: Arc<RpcClient>,
    cache: Arc<RwLock<HashMap<String, TokenMetadata>>>,
    known_tokens: HashMap<String, TokenMetadata>,
}

impl TokenMetadataService {
    /// Create new token metadata service
    pub fn new(rpc_client: Arc<RpcClient>) -> Self {
        let mut known_tokens = HashMap::new();
        
        // Add known tokens with their metadata
        known_tokens.insert(
            "So11111111111111111111111111111111111111112".to_string(),
            TokenMetadata {
                mint: "So11111111111111111111111111111111111111112".to_string(),
                symbol: "SOL".to_string(),
                name: "Wrapped SOL".to_string(),
                decimals: 9,
                uri: None,
                logo_uri: None,
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
            },
        );
        
        known_tokens.insert(
            "7GCihgDB8fe6KNjn2MYtkzZcRjQy3t9GHdC8uHYmW2hr".to_string(),
            TokenMetadata {
                mint: "7GCihgDB8fe6KNjn2MYtkzZcRjQy3t9GHdC8uHYmW2hr".to_string(),
                symbol: "POPCAT".to_string(),
                name: "Popcat".to_string(),
                decimals: 9,
                uri: None,
                logo_uri: None,
            },
        );
        
        known_tokens.insert(
            "7xKXtg2CW87d97TXJSDpbD5jBkheTqA83TZRuJosgAsU".to_string(),
            TokenMetadata {
                mint: "7xKXtg2CW87d97TXJSDpbD5jBkheTqA83TZRuJosgAsU".to_string(),
                symbol: "SAMO".to_string(),
                name: "Samoyedcoin".to_string(),
                decimals: 9,
                uri: None,
                logo_uri: None,
            },
        );
        
        known_tokens.insert(
            "AFbX8oGjGpmVFywbVouvhQSRmiW2aR1mohfahi4Y2AdB".to_string(),
            TokenMetadata {
                mint: "AFbX8oGjGpmVFywbVouvhQSRmiW2aR1mohfahi4Y2AdB".to_string(),
                symbol: "GST".to_string(),
                name: "Green Satoshi Token".to_string(),
                decimals: 9,
                uri: None,
                logo_uri: None,
            },
        );
        
        known_tokens.insert(
            "7i5KKsX2weiTkry7jA4ZwSuXGhs5eJBEjY8vVxR4mRx4".to_string(),
            TokenMetadata {
                mint: "7i5KKsX2weiTkry7jA4ZwSuXGhs5eJBEjY8vVxR4mRx4".to_string(),
                symbol: "GMT".to_string(),
                name: "Green Metaverse Token".to_string(),
                decimals: 6,
                uri: None,
                logo_uri: None,
            },
        );
        
        known_tokens.insert(
            "HZ1JovNiVvGrGNiiYvEozEVg58WUyVHfUNfVwYzqJm8o".to_string(),
            TokenMetadata {
                mint: "HZ1JovNiVvGrGNiiYvEozEVg58WUyVHfUNfVwYzqJm8o".to_string(),
                symbol: "RAY".to_string(),
                name: "Raydium".to_string(),
                decimals: 6,
                uri: None,
                logo_uri: None,
            },
        );
        
        known_tokens.insert(
            "4k3Dyjzvzp8eMZWUXbBCjEvwSkkk59S5iCNLY3QrkX6R".to_string(),
            TokenMetadata {
                mint: "4k3Dyjzvzp8eMZWUXbBCjEvwSkkk59S5iCNLY3QrkX6R".to_string(),
                symbol: "RAY".to_string(),
                name: "Raydium".to_string(),
                decimals: 6,
                uri: None,
                logo_uri: None,
            },
        );
        
        known_tokens.insert(
            "9n4nbM75f5Ui33ZbPYXn59EwSgE8CGsHtAeTH5YFeJ9E".to_string(),
            TokenMetadata {
                mint: "9n4nbM75f5Ui33ZbPYXn59EwSgE8CGsHtAeTH5YFeJ9E".to_string(),
                symbol: "SOLAPE".to_string(),
                name: "Solape".to_string(),
                decimals: 9,
                uri: None,
                logo_uri: None,
            },
        );
        
        known_tokens.insert(
            "5jFnsfx36DyGk8uVGrbXnVUMTsBkPXGpx6e69BiGFzko".to_string(),
            TokenMetadata {
                mint: "5jFnsfx36DyGk8uVGrbXnVUMTsBkPXGpx6e69BiGFzko".to_string(),
                symbol: "STEP".to_string(),
                name: "STEP".to_string(),
                decimals: 9,
                uri: None,
                logo_uri: None,
            },
        );
        
        known_tokens.insert(
            "7vfCXTUXx5WJV5JADk17DUJ4ksgau7utNKj4b963voxs".to_string(),
            TokenMetadata {
                mint: "7vfCXTUXx5WJV5JADk17DUJ4ksgau7utNKj4b963voxs".to_string(),
                symbol: "ETH".to_string(),
                name: "Ether (Portal)".to_string(),
                decimals: 8,
                uri: None,
                logo_uri: None,
            },
        );
        
        known_tokens.insert(
            "2FPyTwvZ6ny3kFLvz2iECr4UqGJgfQkgHLb4UivLqMkj".to_string(),
            TokenMetadata {
                mint: "2FPyTwvZ6ny3kFLvz2iECr4UqGJgfQkgHLb4UivLqMkj".to_string(),
                symbol: "BTC".to_string(),
                name: "Bitcoin (Portal)".to_string(),
                decimals: 8,
                uri: None,
                logo_uri: None,
            },
        );
        
        // Add tokens found in Orca pools
        known_tokens.insert(
            "1111111116GUSSWBXctW2MLfx58vm756SyT1WrE1T".to_string(),
            TokenMetadata {
                mint: "1111111116GUSSWBXctW2MLfx58vm756SyT1WrE1T".to_string(),
                symbol: "SOL".to_string(),
                name: "Wrapped SOL (Orca)".to_string(),
                decimals: 9,
                uri: None,
                logo_uri: None,
            },
        );
        
        known_tokens.insert(
            "11111111JTVonvhkBK8ivT4zK9KqLgu4EeixUAbaE".to_string(),
            TokenMetadata {
                mint: "11111111JTVonvhkBK8ivT4zK9KqLgu4EeixUAbaE".to_string(),
                symbol: "SOL".to_string(),
                name: "Wrapped SOL (Orca 2)".to_string(),
                decimals: 9,
                uri: None,
                logo_uri: None,
            },
        );

        known_tokens.insert(
            "111111111626fV6WXXVwSWQiiZrLLL9dofjT85LHw".to_string(),
            TokenMetadata {
                mint: "111111111626fV6WXXVwSWQiiZrLLL9dofjT85LHw".to_string(),
                symbol: "SOL".to_string(),
                name: "Wrapped SOL (Orca 3)".to_string(),
                decimals: 9,
                uri: None,
                logo_uri: None,
            },
        );

        // Add specific tokens found in our pool discovery
        known_tokens.insert(
            "9cfDMZ1kSED24ZND2eXSr2BJ7RoHPAGUXE7Nj2m3Tz1N".to_string(),
            TokenMetadata {
                mint: "9cfDMZ1kSED24ZND2eXSr2BJ7RoHPAGUXE7Nj2m3Tz1WrE1T".to_string(),
                symbol: "BONK".to_string(),
                name: "Bonk".to_string(),
                decimals: 5,
                uri: None,
                logo_uri: None,
            },
        );

        known_tokens.insert(
            "1tJC1RBe19iM4Gdu4e1yMuiEYQzNZaFoMbNVLGkeGq".to_string(),
            TokenMetadata {
                mint: "1tJC1RBe19iM4Gdu4e1yMuiEYQzNZaFoMbNVLGkeGq".to_string(),
                symbol: "POPCAT".to_string(),
                name: "Popcat".to_string(),
                decimals: 6,
                uri: None,
                logo_uri: None,
            },
        );

        known_tokens.insert(
            "9kUtLXNQ8GrfoBriLAQUs52CvR7NNAApgT7LFFcJb4rJ".to_string(),
            TokenMetadata {
                mint: "9kUtLXNQ8GrfoBriLAQUs52CvR7NNAApgT7LFFcJb4rJ".to_string(),
                symbol: "SAMO".to_string(),
                name: "Samoyedcoin".to_string(),
                decimals: 9,
                uri: None,
                logo_uri: None,
            },
        );

        known_tokens.insert(
            "9cfdPy4ek6EfNSCneLsXG6BRo6mZMJwd2VPwwLgtrKef".to_string(),
            TokenMetadata {
                mint: "9cfdPy4ek6EfNSCneLsXG6BRo6mZMJwd2VPwwLgtrKef".to_string(),
                symbol: "GST".to_string(),
                name: "Green Satoshi Token".to_string(),
                decimals: 9,
                uri: None,
                logo_uri: None,
            },
        );

        known_tokens.insert(
            "9cfdPy4ek6F1ezpRrp4c1gKUtJdtLHxsXFEr8staRWzE".to_string(),
            TokenMetadata {
                mint: "9cfdPy4ek6F1ezpRrp4c1gKUtJdtLHxsXFEr8staRWzE".to_string(),
                symbol: "GMT".to_string(),
                name: "Green Metaverse Token".to_string(),
                decimals: 6,
                uri: None,
                logo_uri: None,
            },
        );

        known_tokens.insert(
            "Hxo3ge6qsAivE4giQuce77nN4o3krVSNJcxvCnTxQnEA".to_string(),
            TokenMetadata {
                mint: "Hxo3ge6qsAivE4giQuce77nN4o3krVSNJcxvCnTxQnEA".to_string(),
                symbol: "RAY".to_string(),
                name: "Raydium".to_string(),
                decimals: 6,
                uri: None,
                logo_uri: None,
            },
        );

        known_tokens.insert(
            "5Jq6aXvEZ4SuQHibGEwxfDb3xhhWK9VbbTFZZR4Y2dKL".to_string(),
            TokenMetadata {
                mint: "5Jq6aXvEZ4SuQHibGEwxfDb3xhhWK9VbbTFZZR4Y2dKL".to_string(),
                symbol: "STEP".to_string(),
                name: "Step".to_string(),
                decimals: 9,
                uri: None,
                logo_uri: None,
            },
        );

        known_tokens.insert(
            "9ecRaYV9vPK9kawmWAwWzGJfCf6KU1sNTnSh7koswgaC".to_string(),
            TokenMetadata {
                mint: "9ecRaYV9vPK9kawmWAwWzGJfCf6KU1sNTnSh7koswgaC".to_string(),
                symbol: "SOLAPE".to_string(),
                name: "Solape".to_string(),
                decimals: 9,
                uri: None,
                logo_uri: None,
            },
        );

        known_tokens.insert(
            "3trXoJCdRxJ6Tg781ci2hnSGwAgfD9LZFYNf9NLaCQPx".to_string(),
            TokenMetadata {
                mint: "3trXoJCdRxJ6Tg781ci2hnSGwAgfD9LZFYNf9NLaCQPx".to_string(),
                symbol: "ETH".to_string(),
                name: "Wrapped ETH (Sollet)".to_string(),
                decimals: 6,
                uri: None,
                logo_uri: None,
            },
        );

        known_tokens.insert(
            "9cfdPy4ek6EfNSCneLsXG6BRo6mZMJwd2RrMXa7hBK3R".to_string(),
            TokenMetadata {
                mint: "9cfdPy4ek6EfNSCneLsXG6BRo6mZMJwd2RrMXa7hBK3R".to_string(),
                symbol: "BTC".to_string(),
                name: "Wrapped BTC (Sollet)".to_string(),
                decimals: 6,
                uri: None,
                logo_uri: None,
            },
        );

        known_tokens.insert(
            "6vsDgmpKHrw8pnfAdEADFpXuD3u6DhmPLjZvKvqXaaLW".to_string(),
            TokenMetadata {
                mint: "6vsDgmpKHrw8pnfAdEADFpXuD3u6DhmPLjZvKvqXaaLW".to_string(),
                symbol: "USDT".to_string(),
                name: "Tether USD (Sollet)".to_string(),
                decimals: 6,
                uri: None,
                logo_uri: None,
            },
        );

        known_tokens.insert(
            "4uQr8ovPMEkrxiYtV4vFUvnyJhTjZWdY7dJMoY2D6bV".to_string(),
            TokenMetadata {
                mint: "4uQr8ovPMEkrxiYtV4vFUvnyJhTjZWdY7dJMoY2D6bV".to_string(),
                symbol: "USDC".to_string(),
                name: "USD Coin (Sollet)".to_string(),
                decimals: 6,
                uri: None,
                logo_uri: None,
            },
        );

        known_tokens.insert(
            "9Qxr73jyf6P425nc6J9PduV9iQMs8EwwMgfFvvN9SPzE".to_string(),
            TokenMetadata {
                mint: "9Qxr73jyf6P425nc6J9PduV9iQMs8EwwMgfFvvN9SPzE".to_string(),
                symbol: "SRM".to_string(),
                name: "Serum".to_string(),
                decimals: 6,
                uri: None,
                logo_uri: None,
            },
        );

        known_tokens.insert(
            "9ecqKXTHT4wvV6W8zgpnFsPFVnX1Fe6z8xKgRM4PdFTQ".to_string(),
            TokenMetadata {
                mint: "9ecqKXTHT4wvV6W8zgpnFsPFVnX1Fe6z8xKgRM4PdFTQ".to_string(),
                symbol: "ORCA".to_string(),
                name: "Orca".to_string(),
                decimals: 6,
                uri: None,
                logo_uri: None,
            },
        );

        known_tokens.insert(
            "8DqqdXpi9MXVJKreZzN1KMtTUKQKRirv7csTjTP3AToy".to_string(),
            TokenMetadata {
                mint: "8DqqdXpi9MXVJKreZzN1KMtTUKQKRirv7csTjTP3AToy".to_string(),
                symbol: "MNGO".to_string(),
                name: "Mango".to_string(),
                decimals: 6,
                uri: None,
                logo_uri: None,
            },
        );

        known_tokens.insert(
            "9cfdPy4ek6EfNSCneLsXG6BRo6mZMJwd2X7oxdXn1BPE".to_string(),
            TokenMetadata {
                mint: "9cfdPy4ek6EfNSCneLsXG6BRo6mZMJwd2X7oxdXn1BPE".to_string(),
                symbol: "FIDA".to_string(),
                name: "Bonfida".to_string(),
                decimals: 6,
                uri: None,
                logo_uri: None,
            },
        );

        known_tokens.insert(
            "8phb8VbjFVVMMyYUgxFh5mS6fg9cvKeQwtbLxJPUDmx".to_string(),
            TokenMetadata {
                mint: "8phb8VbjFVVMMyYUgxFh5mS6fg9cvKeQwtbLxJPUDmx".to_string(),
                symbol: "COPE".to_string(),
                name: "Cope".to_string(),
                decimals: 6,
                uri: None,
                logo_uri: None,
            },
        );

        Self {
            rpc_client,
            cache: Arc::new(RwLock::new(HashMap::new())),
            known_tokens,
        }
    }

    /// Get token metadata by mint address
    pub async fn get_token_metadata(&self, mint: &str) -> Result<TokenMetadata, AppError> {
        // Check cache first
        {
            let cache = self.cache.read().await;
            if let Some(metadata) = cache.get(mint) {
                return Ok(metadata.clone());
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
                // Fallback to generating a readable name
                let fallback_metadata = self.generate_fallback_metadata(mint);
                
                // Cache the fallback result
                let mut cache = self.cache.write().await;
                cache.insert(mint.to_string(), fallback_metadata.clone());
                
                Ok(fallback_metadata)
            }
        }
    }

    /// Fetch token metadata from Solana blockchain
    async fn fetch_token_metadata_from_chain(&self, mint: &str) -> Result<TokenMetadata, AppError> {
        // This is a simplified implementation
        // In production, you would:
        // 1. Find the Metadata PDA for the mint
        // 2. Deserialize the metadata account data
        // 3. Extract symbol, name, decimals, etc.
        
        // For now, return an error to trigger fallback
        Err(AppError::BlockchainError("Token metadata not found on chain".to_string()))
    }

    /// Generate fallback metadata when on-chain data is not available
    fn generate_fallback_metadata(&self, mint: &str) -> TokenMetadata {
        // Try to extract meaningful information from the mint address
        let symbol = if mint.len() >= 8 {
            // Use first 8 characters as symbol
            format!("TOKEN_{}", &mint[..8].to_uppercase())
        } else {
            "UNKNOWN".to_string()
        };

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
}
