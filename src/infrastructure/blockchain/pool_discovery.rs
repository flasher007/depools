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
use solana_account_decoder::UiAccountEncoding;
use solana_sdk::commitment_config::CommitmentConfig;

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
        let vault_reader = VaultReader::new(rpc_url.clone());
        Self {
            rpc_client: SolanaRpcClient::new(rpc_url.clone()),
            orca_parser: OrcaAccountParser::new(std::sync::Arc::new(vault_reader.clone())),
            raydium_parser: RaydiumAccountParser::new(),
            vault_reader,
        }
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
        
        // Try to get real program accounts with timeout and filters
        let timeout_duration = if dex_type == DexType::RaydiumV4 {
            tokio::time::Duration::from_secs(5) // Very short timeout for Raydium V4
        } else {
            tokio::time::Duration::from_secs(30)
        };
        
        // Add filters to limit results and avoid RPC limits
        let filters = match dex_type {
            DexType::OrcaWhirlpool => {
                // Filter for Orca Whirlpool accounts (size only to avoid RPC limits)
                vec![
                    solana_client::rpc_filter::RpcFilterType::DataSize(653), // Whirlpool account size
                ]
            },
            DexType::RaydiumV4 => {
                // For Raydium V4, use only size filter to avoid RPC errors
                vec![
                    solana_client::rpc_filter::RpcFilterType::DataSize(752), // Raydium V4 account size
                ]
            },
            _ => vec![], // No filters for other DEXes
        };
        
        let rpc_result = if dex_type == DexType::RaydiumV4 {
            // For Raydium V4, skip for now to focus on Orca Whirlpool
            println!("⚠️  Raydium V4 temporarily disabled - focusing on Orca Whirlpool");
            return Ok(Vec::new());
        } else if filters.is_empty() {
            // Use simple request if no filters
            tokio::time::timeout(
                timeout_duration,
                self.rpc_client.get_program_accounts(&program_id.to_string())
            ).await
        } else {
            // Use filtered request
            let config = solana_client::rpc_config::RpcProgramAccountsConfig {
                filters: Some(filters),
                account_config: solana_client::rpc_config::RpcAccountInfoConfig {
                    encoding: Some(solana_account_decoder::UiAccountEncoding::Base64),
                    data_slice: None,
                    commitment: Some(solana_sdk::commitment_config::CommitmentConfig::confirmed()),
                    min_context_slot: None,
                },
                with_context: None,
                sort_results: None,
            };
            
            tokio::time::timeout(
                timeout_duration,
                self.rpc_client.get_program_accounts_with_config(
                    &program_id.to_string(),
                    config
                )
            ).await
        };
        
        match rpc_result {
            Ok(Ok(accounts)) => {
                println!("✅ Found {} accounts, parsing pools...", accounts.len());
                let mut pools = Vec::new();
                
                // Limit accounts for performance (like example bot)
                let limit = 10;
                let accounts_to_process: Vec<_> = accounts.into_iter().take(limit).collect();
                
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
                            if dex_type == DexType::OrcaWhirlpool {
                                self.orca_parser.parse_pool_account(&account_data).await
                            } else {
                                parser.parse_pool_account(&account_data)
                            }
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
            self.orca_parser.parse_pool_account(&account_data).await
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
        
        // Count pools for each DEX (skip Jupiter as it's an aggregator)
        for dex_type in [DexType::OrcaWhirlpool, DexType::RaydiumV4] {
            match self.discover_dex_pools(dex_type.clone()).await {
                Ok(pools) => {
                    let count = pools.len();
                    stats.total_pools += count;
                    stats.pools_per_dex.insert(dex_type, count);
                    
                    // Calculate total liquidity
                    let total_liquidity: u128 = pools.iter()
                        .map(|pool| pool.liquidity.value as u128)
                        .sum();
                    stats.total_liquidity += total_liquidity;
                    
                    // Store pools for detailed display
                    stats.pools_by_dex.insert(dex_type, pools);
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
    pub total_liquidity: u128,
    pub pools_per_dex: HashMap<DexType, usize>,
    pub pools_by_dex: HashMap<DexType, Vec<PoolInfo>>,
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
        
        // Print detailed pool information
        println!("\n🔍 Detailed Pool Information:");
        for (dex_type, pools) in &self.pools_by_dex {
            if !pools.is_empty() {
                println!("   {} pools:", dex_type.as_str());
                for (i, pool) in pools.iter().enumerate().take(5) { // Show first 5 pools
                    println!("     Pool {}: {} <-> {} (Liquidity: {})", 
                        i + 1, 
                        pool.token_a.symbol, 
                        pool.token_b.symbol, 
                        pool.liquidity.value
                    );
                }
                if pools.len() > 5 {
                    println!("     ... and {} more pools", pools.len() - 5);
                }
            }
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
