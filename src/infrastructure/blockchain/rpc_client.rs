//! Solana RPC client for direct blockchain reading

use solana_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::commitment_config::CommitmentConfig;
use std::str::FromStr;
use crate::shared::errors::AppError;

/// Solana RPC client wrapper
pub struct SolanaRpcClient {
    client: RpcClient,
}

impl SolanaRpcClient {
    /// Create new RPC client
    pub fn new(rpc_url: String) -> Self {
        Self {
            client: RpcClient::new(rpc_url),
        }
    }
    
    /// Create new RPC client with default mainnet RPC
    pub fn new() -> Self {
        Self::new_mainnet()
    }
    
    /// Create default client (mainnet)
    pub fn new_mainnet() -> Self {
        Self::new("https://api.mainnet-beta.solana.com".to_string())
    }
    
    /// Create devnet client
    pub fn new_devnet() -> Self {
        Self::new("https://api.devnet.solana.com".to_string())
    }
    
    /// Get all accounts owned by a program
    pub async fn get_program_accounts(&self, program_id: &str) -> Result<Vec<(Pubkey, Vec<u8>)>, AppError> {
        let pubkey = Pubkey::from_str(program_id)
            .map_err(|e| AppError::BlockchainError(format!("Invalid program ID: {}", e)))?;
            
        // Get all accounts owned by the program
        let accounts = self.client.get_program_accounts(&pubkey)
            .map_err(|e| AppError::BlockchainError(format!("Failed to get program accounts: {}", e)))?;
            
        // Convert to our expected format
        let result: Vec<(Pubkey, Vec<u8>)> = accounts
            .into_iter()
            .map(|(pubkey, account)| (pubkey, account.data))
            .collect();
            
        Ok(result)
    }
    
    /// Get account data by address
    pub async fn get_account_data(&self, address: &str) -> Result<Vec<u8>, AppError> {
        let pubkey = Pubkey::from_str(address)
            .map_err(|e| AppError::BlockchainError(format!("Invalid address: {}", e)))?;
            
        let account = self.client.get_account(&pubkey)
            .map_err(|e| AppError::BlockchainError(format!("Failed to get account: {}", e)))?;
            
        Ok(account.data)
    }
    
    /// Get multiple accounts by addresses
    pub async fn get_multiple_accounts(&self, addresses: &[String]) -> Result<Vec<Vec<u8>>, AppError> {
        let pubkeys: Result<Vec<Pubkey>, _> = addresses
            .iter()
            .map(|addr| Pubkey::from_str(addr))
            .collect();
            
        let pubkeys = pubkeys
            .map_err(|e| AppError::BlockchainError(format!("Invalid address in list: {}", e)))?;
            
        let accounts = self.client.get_multiple_accounts(&pubkeys)
            .map_err(|e| AppError::BlockchainError(format!("Failed to get multiple accounts: {}", e)))?;
            
        Ok(accounts.into_iter()
            .filter_map(|opt| opt.map(|acc| acc.data))
            .collect())
    }
    
    /// Get latest blockhash
    pub async fn get_latest_blockhash(&self) -> Result<solana_sdk::hash::Hash, AppError> {
        self.client.get_latest_blockhash()
            .map_err(|e| AppError::BlockchainError(format!("Failed to get latest blockhash: {}", e)))
    }
    
    /// Get slot
    pub async fn get_slot(&self) -> Result<u64, AppError> {
        self.client.get_slot()
            .map_err(|e| AppError::BlockchainError(format!("Failed to get slot: {}", e)))
    }
}
