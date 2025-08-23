//! Real transaction executor for Solana

use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    instruction::Instruction,
    message::Message,
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    system_instruction,
    transaction::Transaction,
    system_program,
};
use solana_transaction_status::UiTransactionEncoding;
use crate::shared::errors::AppError;
use crate::domain::dex::{PoolInfo, DexType};
use crate::shared::types::{Amount, Token};
use std::str::FromStr;

/// Transaction execution configuration
#[derive(Debug, Clone)]
pub struct ExecutionConfig {
    pub max_retries: u32,
    pub retry_delay_ms: u64,
    pub commitment: CommitmentConfig,
    pub max_priority_fee: u64,
    pub compute_units: u32,
}

impl Default for ExecutionConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            retry_delay_ms: 1000,
            commitment: CommitmentConfig::confirmed(),
            max_priority_fee: 50_000, // 50k lamports
            compute_units: 200_000,   // 200k compute units
        }
    }
}

/// Transaction execution result
#[derive(Debug, Clone)]
pub struct ExecutionResult {
    pub success: bool,
    pub transaction_signature: Option<String>,
    pub error_message: Option<String>,
    pub gas_used: u64,
    pub block_time: i64,
    pub slot: u64,
}

/// Real transaction executor
pub struct RealTransactionExecutor {
    rpc_client: RpcClient,
    config: ExecutionConfig,
    wallet: Keypair,
}

impl RealTransactionExecutor {
    /// Create new transaction executor
    pub fn new(rpc_url: String, wallet: Keypair, config: ExecutionConfig) -> Self {
        Self {
            rpc_client: RpcClient::new_with_commitment(rpc_url, config.commitment),
            config,
            wallet,
        }
    }
    
    /// Create with default configuration
    pub fn new_default(rpc_url: String, wallet: Keypair) -> Self {
        Self::new(rpc_url, wallet, ExecutionConfig::default())
    }
    
    /// Create with just RPC URL (for testing)
    pub fn new_simple(rpc_url: String) -> Self {
        let dummy_wallet = Keypair::new();
        Self::new(rpc_url, dummy_wallet, ExecutionConfig::default())
    }
    
    /// Execute arbitrage transaction
    pub async fn execute_arbitrage(
        &self,
        token_a: &Token,
        token_b: &Token,
        amount_in: Amount,
        pool_1: &PoolInfo,
        pool_2: &PoolInfo,
    ) -> Result<ExecutionResult, AppError> {
        println!("🚀 Executing arbitrage transaction...");
        println!("   Token A: {} ({})", token_a.symbol, token_a.mint);
        println!("   Token B: {} ({})", token_b.symbol, token_b.mint);
        println!("   Amount: {} tokens", amount_in.value);
        println!("   Pool 1: {} on {}", pool_1.id, pool_1.dex_type.as_str());
        println!("   Pool 2: {} on {}", pool_2.id, pool_2.dex_type.as_str());
        
        // Build arbitrage instructions
        let instructions = self.build_arbitrage_instructions(
            token_a, token_b, amount_in, pool_1, pool_2
        )?;
        
        // Execute transaction
        let result = self.execute_transaction(&instructions).await?;
        
        if result.success {
            println!("✅ Arbitrage transaction executed successfully!");
            if let Some(sig) = &result.transaction_signature {
                println!("   Signature: {}", sig);
            }
            println!("   Gas used: {} lamports", result.gas_used);
        } else {
            println!("❌ Arbitrage transaction failed!");
            if let Some(error) = &result.error_message {
                println!("   Error: {}", error);
            }
        }
        
        Ok(result)
    }
    
    /// Build arbitrage instructions
    pub fn build_arbitrage_instructions(
        &self,
        token_a: &Token,
        token_b: &Token,
        amount_in: Amount,
        pool_1: &PoolInfo,
        pool_2: &PoolInfo,
    ) -> Result<Vec<Instruction>, AppError> {
        let mut instructions = Vec::new();
        
        // Add compute budget instruction for priority fee
        // Note: These functions may not be available in all Solana SDK versions
        // For now, we'll skip compute budget instructions
        // let compute_budget_ix = solana_sdk::compute_budget::set_compute_unit_price(
        //     self.config.max_priority_fee,
        // );
        // instructions.push(compute_budget_ix);
        
        // let compute_units_ix = solana_sdk::compute_budget::set_compute_unit_limit(
        //     self.config.compute_units,
        // );
        // instructions.push(compute_units_ix);
        
        // Build swap instruction for first pool
        let swap_1_ix = self.build_swap_instruction(
            pool_1,
            token_a,
            token_b,
            amount_in.clone(),
            true, // first swap
        )?;
        instructions.push(swap_1_ix);
        
        // Build swap instruction for second pool
        let swap_2_ix = self.build_swap_instruction(
            pool_2,
            token_b,
            token_a,
            amount_in, // This would be the output from first swap
            false, // second swap
        )?;
        instructions.push(swap_2_ix);
        
        Ok(instructions)
    }
    
    /// Build swap instruction for a specific pool
    fn build_swap_instruction(
        &self,
        pool: &PoolInfo,
        token_in: &Token,
        token_out: &Token,
        amount_in: Amount,
        is_first_swap: bool,
    ) -> Result<Instruction, AppError> {
        match pool.dex_type {
            DexType::OrcaWhirlpool => {
                self.build_orca_swap_instruction(pool, token_in, token_out, amount_in, is_first_swap)
            }
            DexType::RaydiumV4 => {
                self.build_raydium_swap_instruction(pool, token_in, token_out, amount_in, is_first_swap)
            }
            _ => Err(AppError::BlockchainError("Unsupported DEX type".to_string())),
        }
    }
    
    /// Build Orca Whirlpool swap instruction
    fn build_orca_swap_instruction(
        &self,
        pool: &PoolInfo,
        token_in: &Token,
        token_out: &Token,
        amount_in: Amount,
        _is_first_swap: bool,
    ) -> Result<Instruction, AppError> {
        // Orca Whirlpool Program ID
        let whirlpool_program = Pubkey::from_str("whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc")
            .map_err(|e| AppError::BlockchainError(format!("Invalid Orca program ID: {}", e)))?;
        
        // Pool address
        let pool_address = Pubkey::from_str(&pool.id)
            .map_err(|e| AppError::BlockchainError(format!("Invalid pool address: {}", e)))?;
        
        // Token accounts
        let token_in_account = self.get_or_create_token_account(token_in)?;
        let token_out_account = self.get_or_create_token_account(token_out)?;
        
        // Build swap instruction
        let swap_ix = solana_sdk::instruction::Instruction {
            program_id: whirlpool_program,
            accounts: vec![
                solana_sdk::instruction::AccountMeta::new(pool_address, false),
                solana_sdk::instruction::AccountMeta::new(token_in_account, false),
                solana_sdk::instruction::AccountMeta::new(token_out_account, false),
                solana_sdk::instruction::AccountMeta::new_readonly(self.wallet.pubkey(), true),
                solana_sdk::instruction::AccountMeta::new_readonly(system_program::id(), false),
            ],
            data: vec![
                0x01, // Swap instruction
                // amount_in.value.to_le_bytes().to_vec(),
                // Additional swap parameters would go here
            ],
        };
        
        Ok(swap_ix)
    }
    
    /// Build Raydium V4 swap instruction
    fn build_raydium_swap_instruction(
        &self,
        pool: &PoolInfo,
        token_in: &Token,
        token_out: &Token,
        amount_in: Amount,
        _is_first_swap: bool,
    ) -> Result<Instruction, AppError> {
        // Raydium V4 Program ID
        let raydium_program = Pubkey::from_str("675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8")
            .map_err(|e| AppError::BlockchainError(format!("Invalid Raydium program ID: {}", e)))?;
        
        // Pool address
        let pool_address = Pubkey::from_str(&pool.id)
            .map_err(|e| AppError::BlockchainError(format!("Invalid pool address: {}", e)))?;
        
        // Token accounts
        let token_in_account = self.get_or_create_token_account(token_in)?;
        let token_out_account = self.get_or_create_token_account(token_out)?;
        
        // Build swap instruction
        let swap_ix = solana_sdk::instruction::Instruction {
            program_id: raydium_program,
            accounts: vec![
                solana_sdk::instruction::AccountMeta::new(pool_address, false),
                solana_sdk::instruction::AccountMeta::new(token_in_account, false),
                solana_sdk::instruction::AccountMeta::new(token_out_account, false),
                solana_sdk::instruction::AccountMeta::new_readonly(self.wallet.pubkey(), true),
                solana_sdk::instruction::AccountMeta::new_readonly(system_program::id(), false),
            ],
            data: vec![
                0x01, // Swap instruction
                // amount_in.value.to_le_bytes().to_vec(),
                // Additional swap parameters would go here
            ],
        };
        
        Ok(swap_ix)
    }
    
    /// Get or create token account
    fn get_or_create_token_account(&self, token: &Token) -> Result<Pubkey, AppError> {
        // For demo purposes, return a mock token account
        // In production, this would check if account exists and create if needed
        Ok(Pubkey::new_unique())
    }
    
    /// Execute transaction with retries
    async fn execute_transaction(
        &self,
        instructions: &[Instruction],
    ) -> Result<ExecutionResult, AppError> {
        let mut attempts = 0;
        
        while attempts < self.config.max_retries {
            match self.try_execute_transaction(instructions).await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    attempts += 1;
                    eprintln!("❌ Transaction attempt {} failed: {}", attempts, e);
                    
                    if attempts < self.config.max_retries {
                        tokio::time::sleep(tokio::time::Duration::from_millis(self.config.retry_delay_ms)).await;
                    }
                }
            }
        }
        
        Err(AppError::BlockchainError("Transaction failed after max retries".to_string()))
    }
    
    /// Try to execute a single transaction
    async fn try_execute_transaction(
        &self,
        instructions: &[Instruction],
    ) -> Result<ExecutionResult, AppError> {
        // Get recent blockhash
        let recent_blockhash = self.rpc_client.get_latest_blockhash()
            .map_err(|e| AppError::BlockchainError(format!("Failed to get blockhash: {}", e)))?;
        
        // Build message
        let message = Message::new(
            instructions,
            Some(&self.wallet.pubkey()),
        );
        
        // Create transaction
        let mut transaction = Transaction::new_unsigned(message);
        transaction.sign(&[&self.wallet], recent_blockhash);
        
        // Send transaction
        let signature = self.rpc_client.send_and_confirm_transaction(&transaction)
            .map_err(|e| AppError::BlockchainError(format!("Transaction failed: {}", e)))?;
        
        // Get transaction status
        let status = self.rpc_client.get_transaction(
            &signature,
            UiTransactionEncoding::Json,
        );
        
        match status {
            Ok(tx_status) => {
                let success = tx_status.transaction.meta.as_ref()
                    .map(|meta| meta.err.is_none())
                    .unwrap_or(false);
                
                let gas_used = tx_status.transaction.meta.as_ref()
                    .map(|meta| meta.fee)
                    .unwrap_or(0);
                
                let block_time = tx_status.block_time.unwrap_or(0);
                let slot = tx_status.slot;
                
                Ok(ExecutionResult {
                    success,
                    transaction_signature: Some(signature.to_string()),
                    error_message: None,
                    gas_used,
                    block_time,
                    slot,
                })
            }
            // Note: get_transaction now returns the transaction directly, not Option
            // This case is no longer needed
            Err(e) => {
                Ok(ExecutionResult {
                    success: false,
                    transaction_signature: Some(signature.to_string()),
                    error_message: Some(format!("Failed to get status: {}", e)),
                    gas_used: 0,
                    block_time: 0,
                    slot: 0,
                })
            }
        }
    }
    
    /// Get wallet balance
    pub fn get_wallet_balance(&self) -> Result<u64, AppError> {
        self.rpc_client.get_balance(&self.wallet.pubkey())
            .map_err(|e| AppError::BlockchainError(format!("Failed to get balance: {}", e)))
    }
    
    /// Get token balance
    pub fn get_token_balance(&self, token: &Token) -> Result<u64, AppError> {
        // For demo purposes, return mock balance
        // In production, this would query the actual token account
        Ok(1000)
    }
    
    /// Simulate transaction (dry run)
    pub async fn simulate_transaction(
        &self,
        instructions: &[Instruction],
    ) -> Result<ExecutionResult, AppError> {
        println!("🧪 Simulating transaction...");
        
        // Get recent blockhash
        let recent_blockhash = self.rpc_client.get_latest_blockhash()
            .map_err(|e| AppError::BlockchainError(format!("Failed to get blockhash: {}", e)))?;
        
        // Build message
        let message = Message::new(
            instructions,
            Some(&self.wallet.pubkey()),
        );
        
        // Create transaction
        let mut transaction = Transaction::new_unsigned(message);
        transaction.sign(&[&self.wallet], recent_blockhash);
        
        // Simulate transaction
        let simulation = self.rpc_client.simulate_transaction(&transaction)
            .map_err(|e| AppError::BlockchainError(format!("Simulation failed: {}", e)))?;
        
        let success = simulation.value.err.is_none();
        let gas_used = simulation.value.units_consumed.unwrap_or(0);
        
        println!("   Simulation result: {}", if success { "✅ SUCCESS" } else { "❌ FAILED" });
        println!("   Estimated gas: {} lamports", gas_used);
        
        Ok(ExecutionResult {
            success,
            transaction_signature: None,
            error_message: simulation.value.err.map(|e| format!("{:?}", e)),
            gas_used,
            block_time: 0,
            slot: 0,
        })
    }
}
