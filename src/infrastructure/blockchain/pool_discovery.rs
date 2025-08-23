//! Pool discovery service using direct blockchain reading

use std::collections::HashMap;
use solana_sdk::pubkey::Pubkey;
use crate::shared::types::{Token, Amount};
use crate::domain::dex::{PoolInfo, DexType};
use crate::shared::errors::AppError;
use super::rpc_client::SolanaRpcClient;
use super::account_parser::{OrcaAccountParser, RaydiumAccountParser, AccountParser};

use super::vault_reader::VaultReader;
use super::dex_structures::{Whirlpool, RaydiumV4Pool};

/// Pool discovery service
pub struct PoolDiscoveryService {
    rpc_client: SolanaRpcClient,
    orca_parser: OrcaAccountParser,
    raydium_parser: RaydiumAccountParser,
    vault_reader: VaultReader,
}

impl PoolDiscoveryService {
    /// Create new pool discovery service
    pub fn new(rpc_url: String) -> Self {
        Self {
            rpc_client: SolanaRpcClient::new(rpc_url.clone()),
            orca_parser: OrcaAccountParser::new(),
            raydium_parser: RaydiumAccountParser::new(),
            vault_reader: VaultReader::new(rpc_url),
        }
    }
    
    /// Create new pool discovery service with default mainnet RPC
    pub fn new() -> Self {
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
    
    /// Discover all pools for a specific DEX
    pub async fn discover_dex_pools(&self, dex_type: DexType) -> Result<Vec<PoolInfo>, AppError> {
        let program_id = dex_type.program_id();
        let parser = self.get_parser_for_dex(&dex_type);
        
        println!("🔍 Discovering {} pools...", dex_type.as_str());
        
        // Try to get real program accounts with timeout
        let timeout_duration = tokio::time::Duration::from_secs(10);
        let rpc_result = tokio::time::timeout(
            timeout_duration,
            self.rpc_client.get_program_accounts(&program_id.to_string())
        ).await;
        
        match rpc_result {
            Ok(Ok(accounts)) => {
                println!("✅ Found {} accounts, parsing pools...", accounts.len());
                let mut pools = Vec::new();
                
                // Limit to first 10 accounts for performance
                let accounts_to_process: Vec<_> = accounts.into_iter().take(10).collect();
                
                for (pubkey, account_data) in accounts_to_process {
                    // Check if account is a valid pool
                    if parser.is_pool_account(&account_data) {
                        // Try to parse with real balances first
                        let pool_result = if let Ok(_whirlpool) = Whirlpool::try_deserialize(&account_data) {
                            // This is an Orca pool, use enhanced parsing
                            self.orca_parser.parse_pool_account_with_balances(&account_data, &self.vault_reader).await
                        } else if let Ok(_raydium_pool) = RaydiumV4Pool::try_deserialize(&account_data) {
                            // This is a Raydium pool, use enhanced parsing
                            self.raydium_parser.parse_pool_account_with_balances(&account_data, &self.vault_reader).await
                        } else {
                            // Fallback to legacy parsing
                            parser.parse_pool_account(&account_data)
                        };
                        
                        match pool_result {
                            Ok(mut pool) => {
                                // Update pool ID with actual pubkey
                                pool.id = pubkey.to_string();
                                pools.push(pool);
                            }
                            Err(e) => {
                                // Log parsing error but continue
                                eprintln!("Failed to parse pool account {}: {}", pubkey, e);
                            }
                        }
                    }
                }
                
                if pools.is_empty() {
                    println!("⚠️  No valid pools found for {}", dex_type.as_str());
                    Ok(Vec::new())
                } else {
                    println!("✅ Successfully parsed {} pools", pools.len());
                    Ok(pools)
                }
            }
            Ok(Err(e)) => {
                println!("❌ RPC error: {} for {}", e, dex_type.as_str());
                Ok(Vec::new())
            }
            Err(_) => {
                println!("⏰ RPC timeout after 10s for {}", dex_type.as_str());
                Ok(Vec::new())
            }
        }
    }
    
    /// Discover pools for specific token pair across all DEXes
    pub async fn discover_token_pair_pools(
        &self,
        token_a: &Token,
        token_b: &Token,
    ) -> Result<HashMap<DexType, Vec<PoolInfo>>, AppError> {
        let mut all_pools: HashMap<DexType, Vec<PoolInfo>> = HashMap::new();
        
        // Discover pools for each DEX
        let dexes = vec![DexType::OrcaWhirlpool, DexType::RaydiumV4];
        
        for dex_type in dexes {
            match self.discover_dex_pools(dex_type.clone()).await {
                Ok(pools) => {
                    // Filter pools for specific token pair
                    let filtered_pools: Vec<PoolInfo> = pools
                        .into_iter()
                        .filter(|pool| {
                            (pool.token_a.mint == token_a.mint && pool.token_b.mint == token_b.mint) ||
                            (pool.token_a.mint == token_b.mint && pool.token_b.mint == token_a.mint)
                        })
                        .collect();
                    
                    if !filtered_pools.is_empty() {
                        all_pools.insert(dex_type, filtered_pools);
                    }
                }
                Err(e) => {
                    eprintln!("Failed to discover pools for {}: {}", dex_type.as_str(), e);
                }
            }
        }
        
        Ok(all_pools)
    }
    
    /// Get pool information by address
    pub async fn get_pool_by_address(&self, pool_address: &str) -> Result<PoolInfo, AppError> {
        let account_data = self.rpc_client.get_account_data(pool_address).await?;
        
        // Try to determine DEX type and parse accordingly
        if self.orca_parser.is_pool_account(&account_data) {
            self.orca_parser.parse_pool_account(&account_data)
        } else if self.raydium_parser.is_pool_account(&account_data) {
            self.raydium_parser.parse_pool_account(&account_data)
        } else {
            Err(AppError::BlockchainError("Unknown pool account format".to_string()))
        }
    }
    
    /// Get parser for specific DEX type
    fn get_parser_for_dex(&self, dex_type: &DexType) -> &dyn AccountParser {
        match dex_type {
            DexType::OrcaWhirlpool => &self.orca_parser,
            DexType::RaydiumV4 => &self.raydium_parser,
            _ => &self.orca_parser, // Default fallback
        }
    }
    
    /// Get pool statistics
    pub async fn get_pool_statistics(&self) -> Result<PoolStatistics, AppError> {
        let mut stats = PoolStatistics::default();
        
        // Count pools for each DEX
        for dex_type in [DexType::OrcaWhirlpool, DexType::RaydiumV4] {
            match self.discover_dex_pools(dex_type.clone()).await {
                Ok(pools) => {
                    let count = pools.len();
                    stats.total_pools += count;
                    stats.pools_per_dex.insert(dex_type, count);
                    
                    // Calculate total liquidity
                    let total_liquidity: u64 = pools.iter()
                        .map(|pool| pool.liquidity.value)
                        .sum();
                    stats.total_liquidity += total_liquidity;
                }
                Err(e) => {
                    eprintln!("Failed to get statistics for {}: {}", dex_type.as_str(), e);
                }
            }
        }
        
        Ok(stats)
    }
}

/// Pool statistics
#[derive(Debug, Default)]
pub struct PoolStatistics {
    pub total_pools: usize,
    pub total_liquidity: u64,
    pub pools_per_dex: HashMap<DexType, usize>,
}

impl PoolStatistics {
    pub fn print_summary(&self) {
        println!("📊 Pool Discovery Statistics:");
        println!("   Total pools: {}", self.total_pools);
        println!("   Total liquidity: {} USDC", self.total_liquidity);
        println!("   Pools per DEX:");
        for (dex_type, count) in &self.pools_per_dex {
            println!("     {}: {} pools", dex_type.as_str(), count);
        }
    }
    
    /// Get pool prices for monitoring
    pub fn get_pool_prices(&self) -> HashMap<String, f64> {
        let mut prices = HashMap::new();
        
        // For now, return simulated prices based on pool count
        // In real implementation, this would return actual pool prices
        for (dex_type, count) in &self.pools_per_dex {
            for i in 0..*count {
                let pool_id = format!("{}_{}", dex_type.as_str(), i);
                let price = 100.0 + (i as f64 * 0.1); // Simulated price
                prices.insert(pool_id, price);
            }
        }
        
        prices
    }
}
