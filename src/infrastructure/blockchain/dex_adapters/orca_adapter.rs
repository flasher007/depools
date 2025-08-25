use async_trait::async_trait;
use solana_client::rpc_client::RpcClient;
use solana_client::rpc_filter::RpcFilterType;
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;
use std::sync::Arc;
use tokio::time::Duration;

use super::traits::DexAdapter;
use crate::domain::dex::{DexType, PoolInfo};
use crate::shared::errors::AppError;
use crate::shared::types::{Token, Amount};
use crate::infrastructure::blockchain::{
    OrcaAccountParser, VaultReader, dex_structures::Whirlpool
};

/// Orca Whirlpool DEX adapter
pub struct OrcaAdapter {
    rpc_client: Arc<RpcClient>,
    vault_reader: Arc<VaultReader>,
    account_parser: OrcaAccountParser,
}

impl OrcaAdapter {
    pub fn new(
        rpc_client: Arc<RpcClient>,
        vault_reader: Arc<VaultReader>,
        account_parser: OrcaAccountParser,
    ) -> Self {
        Self {
            rpc_client,
            vault_reader,
            account_parser,
        }
    }
}

#[async_trait]
impl DexAdapter for OrcaAdapter {
    fn dex_type(&self) -> DexType {
        DexType::OrcaWhirlpool
    }
    
    async fn discover_pools(&self) -> Result<Vec<PoolInfo>, AppError> {
        let program_id = DexType::OrcaWhirlpool.program_id();
        
        // Use filters to limit results and avoid RPC limits
        let filters = vec![
            RpcFilterType::DataSize(653), // Whirlpool account size
        ];
        
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
        
        let accounts_result = self.rpc_client.get_program_accounts_with_config(
            &program_id,
            config
        ).map_err(|e| AppError::BlockchainError(format!("RPC error: {}", e)))?;
        
        let mut pools = Vec::new();
        
        // Limit accounts for performance
        let limit = 10;
        let accounts_to_process: Vec<_> = accounts_result.into_iter().take(limit).collect();
        
        for (pubkey, account) in accounts_to_process {
            if self.is_pool_account(&account.data) {
                match self.parse_pool_account(&account.data).await {
                    Ok(mut pool) => {
                        pool.id = pubkey.to_string();
                        pools.push(pool);
                    }
                    Err(e) => {
                        eprintln!("Failed to parse Orca pool account {}: {}", pubkey, e);
                    }
                }
            }
        }
        
        Ok(pools)
    }
    
    async fn get_pool_by_tokens(&self, _token_a: &str, _token_b: &str) -> Result<Option<PoolInfo>, AppError> {
        // Orca doesn't support direct token pair lookup like Raydium
        // We need to scan all pools to find matching pairs
        Ok(None)
    }
    
    fn is_pool_account(&self, account_data: &[u8]) -> bool {
        self.account_parser.is_pool_account(account_data)
    }
    
    async fn parse_pool_account(&self, account_data: &[u8]) -> Result<PoolInfo, AppError> {
        // Try to parse with real balances first
        if let Ok(_whirlpool) = Whirlpool::try_deserialize(account_data) {
            // This is an Orca pool, use enhanced parsing
            self.account_parser.parse_pool_account_with_balances(account_data, &self.vault_reader).await
        } else {
            // Fallback to legacy parsing
            self.account_parser.parse_pool_account(account_data).await
        }
    }
    
    async fn get_pool_stats(&self, pool_id: &str) -> Result<PoolInfo, AppError> {
        // Get pool account data and parse it
        let pubkey = Pubkey::from_str(pool_id)
            .map_err(|e| AppError::BlockchainError(format!("Invalid pubkey: {}", e)))?;
        
        let account = self.rpc_client.get_account(&pubkey)
            .map_err(|e| AppError::BlockchainError(format!("Failed to get account: {}", e)))?;
        
        self.parse_pool_account(&account.data).await
    }
}
