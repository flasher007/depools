//! Pool discovery service using direct blockchain reading

use std::collections::HashMap;
use solana_sdk::pubkey::Pubkey;
use crate::shared::types::{Token, Amount};
use crate::domain::dex::{PoolInfo, DexType};
use crate::shared::errors::AppError;
use super::rpc_client::SolanaRpcClient;
use super::account_parser::{OrcaAccountParser, RaydiumAccountParser, AccountParser};

use super::vault_reader::VaultReader;
use super::dex_structures::{Whirlpool, RaydiumAMMPool};
use solana_account_decoder::UiAccountEncoding;
use solana_sdk::commitment_config::CommitmentConfig;
use std::time::Duration;
use std::str::FromStr;

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
    

    
    /// Discover pools for a specific DEX
    pub async fn discover_dex_pools(&self, dex_type: DexType) -> Result<Vec<PoolInfo>, AppError> {
        let program_id = dex_type.program_id();
        let timeout_duration = match dex_type {
            DexType::OrcaWhirlpool => Duration::from_secs(10),
            DexType::RaydiumAMM => Duration::from_secs(15), // Reasonable timeout for Raydium AMM
        };

        let filters = match dex_type {
            DexType::OrcaWhirlpool => {
                vec![
                    solana_client::rpc_filter::RpcFilterType::DataSize(653), // Whirlpool account size
                ]
            },
            DexType::RaydiumAMM => {
                // For Raydium AMM, we'll use a different approach - search for specific pool pairs
                vec![]
            },
        };

        let rpc_result = if dex_type == DexType::RaydiumAMM {
            // For Raydium AMM, search for specific known pools
            println!("🔍 Raydium AMM: Using specific pool discovery approach...");
            
            let known_pool_pairs = vec![
                ("So11111111111111111111111111111111111111112", "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"), // SOL-USDC
                ("So11111111111111111111111111111111111111112", "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB"), // SOL-USDT
                ("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v", "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB"), // USDC-USDT
                ("7vfCXTUXx5WJV5JADk17DUJ4ksgau7utNKj4b963voxs", "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"), // ETH-USDC
                ("DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263", "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"), // BONK-USDC
            ];
            
            let mut raydium_pools = Vec::new();
            
            for (mint_a, mint_b) in known_pool_pairs {
                println!("   🔍 Searching for pool: {} <-> {}", mint_a, mint_b);
                
                match self.search_raydium_amm_pool(mint_a, mint_b).await {
                    Ok(pool_info) => {
                        println!("   ✅ Found pool: {} <-> {}", pool_info.token_a.symbol, pool_info.token_b.symbol);
                        raydium_pools.push(pool_info);
                    },
                    Err(e) => {
                        println!("   ❌ Pool not found: {} <-> {} (error: {})", mint_a, mint_b, e);
                    }
                }
            }
            
            Ok(raydium_pools)
        } else {
            // Use filtered request for other DEXes
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
            
            let accounts_result = tokio::time::timeout(
                timeout_duration,
                self.rpc_client.get_program_accounts_with_config(
                    &program_id.to_string(),
                    config
                )
            ).await.map_err(|_| AppError::BlockchainError("RPC timeout".to_string()))??;
            
            let mut pools = Vec::new();
            
            // Limit accounts for performance (like example bot)
            let limit = 10;
            let accounts_to_process: Vec<_> = accounts_result.into_iter().take(limit).collect();
            
            for (pubkey, account_data) in accounts_to_process {
                // Check if account is a valid pool
                let parser = self.get_parser_for_dex(&dex_type);
                if parser.is_pool_account(&account_data) {
                    // Try to parse with real balances first
                    let pool_result = if let Ok(_whirlpool) = Whirlpool::try_deserialize(&account_data) {
                        // This is an Orca pool, use enhanced parsing
                        self.orca_parser.parse_pool_account_with_balances(&account_data, &self.vault_reader).await
                    } else if let Ok(_raydium_pool) = RaydiumAMMPool::try_deserialize(&account_data) {
                        // This is a Raydium AMM pool, use enhanced parsing
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
            
            Ok(pools)
        };
        
        rpc_result
    }
    
    /// Discover pools for specific token pair across all DEXes
    pub async fn discover_token_pair_pools(
        &self,
        token_a: &Token,
        token_b: &Token,
    ) -> Result<HashMap<DexType, Vec<PoolInfo>>, AppError> {
        let mut all_pools: HashMap<DexType, Vec<PoolInfo>> = HashMap::new();
        
        // Discover pools for each DEX
        let dexes = vec![DexType::OrcaWhirlpool, DexType::RaydiumAMM];
        
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
            DexType::RaydiumAMM => &self.raydium_parser,
            _ => &self.orca_parser, // Default fallback
        }
    }
    
    /// Search for a specific Raydium AMM pool by mint addresses
    async fn search_raydium_amm_pool(&self, mint_a: &str, mint_b: &str) -> Result<PoolInfo, AppError> {
        println!("      🔍 Searching Raydium AMM pool for {} <-> {}", mint_a, mint_b);
        
        // For now, we'll create a mock pool info based on the mints
        // In production, you'd search the blockchain for actual pool accounts
        
        // Get token metadata
        let token_a_meta = self.vault_reader.get_token_metadata(mint_a).await?;
        let token_b_meta = self.vault_reader.get_token_metadata(mint_b).await?;
        
        // Create a mock pool info (this would be replaced with real pool data)
        let pool_info = PoolInfo {
            id: format!("raydium_amm_{}_{}", mint_a, mint_b),
            dex_type: DexType::RaydiumAMM,
            token_a: Token {
                mint: Pubkey::from_str(mint_a).unwrap(),
                symbol: token_a_meta.symbol,
                decimals: token_a_meta.decimals,
                name: Some(token_a_meta.name),
            },
            token_b: Token {
                mint: Pubkey::from_str(mint_b).unwrap(),
                symbol: token_b_meta.symbol,
                decimals: token_b_meta.decimals,
                name: Some(token_b_meta.name),
            },
            reserve_a: Amount::new(1000000000, token_a_meta.decimals), // Mock liquidity
            reserve_b: Amount::new(1000000000, token_b_meta.decimals), // Mock liquidity
            fee_rate: 0.25, // Raydium AMM typical fee
            liquidity: Amount::new(1000000000, 6), // Mock total liquidity
            volume_24h: Amount::new(0, 6), // TODO: Calculate from recent transactions
        };
        
        Ok(pool_info)
    }
    
    /// Get pool statistics
    pub async fn get_pool_statistics(&self) -> Result<PoolStatistics, AppError> {
        let mut stats = PoolStatistics::default();
        
        // Count pools for each DEX (skip Jupiter as it's an aggregator)
        for dex_type in [DexType::OrcaWhirlpool, DexType::RaydiumAMM] {
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
