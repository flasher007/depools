use anyhow::Result;
use crate::exchanges::types::{ArbitrageOpportunity, RiskScore};
use crate::exchanges::utils::{lamports_to_sol, format_sol};
use crate::opportunity::scanner::{OpportunityScanner, AsyncOpportunityScanner, CrossDexScanner};
use crate::exchanges::transaction_builder::TransactionBuilder;
use std::sync::Arc;
use solana_sdk::{
    signature::{Keypair, Signature},
    hash::Hash,
    transaction::Transaction,
};
use solana_client::rpc_client::RpcClient;
use tracing::{info, error};

pub struct ArbitrageEngine {
    scanner: Arc<CrossDexScanner>,
    min_profit_bps: i32,
    max_risk_score: RiskScore,
}

impl ArbitrageEngine {
    pub fn new(scanner: Arc<CrossDexScanner>, min_profit_bps: i32) -> Self {
        Self {
            scanner,
            min_profit_bps,
            max_risk_score: RiskScore::Medium,
        }
    }

    pub fn get_scanner(&self) -> Arc<CrossDexScanner> {
        self.scanner.clone()
    }

    pub async fn find_opportunities(&self, pool_addresses: &[String]) -> Result<Vec<ArbitrageOpportunity>> {
        let opportunities = self.scanner.scan_opportunities(pool_addresses)?;
        
        // Фильтруем по минимальной прибыльности и риску
        let filtered = opportunities
            .into_iter()
            .filter(|opp| {
                opp.profit_bps >= self.min_profit_bps &&
                opp.risk_score as u8 <= self.max_risk_score as u8
            })
            .collect();
        
        Ok(filtered)
    }

    pub async fn find_opportunities_async(&self, pool_addresses: &[String]) -> Result<Vec<ArbitrageOpportunity>> {
        // Use default parameters for now - these should come from configuration
        let amount_in = 1_000_000_000; // 1 SOL in lamports (much more readable)
        let spread_threshold_bps = 50; // 0.5%
        let slippage_bps = 100; // 1%
        let priority_fee = 1000; // 1000 lamports per compute unit
        
        let opportunities = self.scanner.scan_opportunities_async(
            pool_addresses,
            amount_in,
            spread_threshold_bps,
            slippage_bps,
            priority_fee,
        ).await?;
        
        // Фильтруем по минимальной прибыльности и риску
        let filtered = opportunities
            .into_iter()
            .filter(|opp| {
                opp.profit_bps >= self.min_profit_bps &&
                opp.risk_score as u8 <= self.max_risk_score as u8
            })
            .collect();
        
        Ok(filtered)
    }

    pub fn set_min_profit_threshold(&mut self, bps: i32) {
        self.min_profit_bps = bps;
    }

    pub fn set_max_risk_score(&mut self, risk_score: RiskScore) {
        self.max_risk_score = risk_score;
    }
    
    /// Execute arbitrage opportunity
    pub async fn execute_arbitrage(
        &self,
        opportunity: &ArbitrageOpportunity,
        user_keypair: &Keypair,
        rpc_client: &RpcClient,
        slippage_bps: u32,
        priority_fee: u64,
        simulate_only: bool,
    ) -> Result<Option<Signature>> {
        info!("🚀 Executing arbitrage opportunity: {}", opportunity.id);
        let profit_sol = lamports_to_sol(opportunity.profit_amount);
        info!("💰 Expected profit: {} bps ({})", opportunity.profit_bps, format_sol(profit_sol));
        
        // Get recent blockhash
        let recent_blockhash = rpc_client.get_latest_blockhash()?;
        info!("📋 Got recent blockhash: {}", recent_blockhash);
        
        // Get adapters from scanner (we need to access them for transaction building)
        let adapters = self.get_adapters_for_transaction()?;
        
        // Build atomic transaction
        let transaction_builder = TransactionBuilder;
        let transaction = transaction_builder.build_arbitrage_transaction(
            opportunity,
            user_keypair,
            recent_blockhash,
            &adapters,
            slippage_bps,
            priority_fee,
        )?;
        
        // Validate transaction
        transaction_builder.validate_transaction(&transaction)?;
        
        if simulate_only {
            // Simulate transaction
            self.simulate_transaction(&transaction, rpc_client).await?;
            info!("✅ Transaction simulation successful");
            return Ok(None);
        }
        
        // Execute transaction
        info!("📤 Sending transaction to network...");
        match rpc_client.send_and_confirm_transaction(&transaction) {
            Ok(signature) => {
                info!("🎉 Arbitrage executed successfully! Signature: {}", signature);
                Ok(Some(signature))
            }
            Err(e) => {
                error!("❌ Failed to execute arbitrage: {}", e);
                Err(anyhow::anyhow!("Transaction failed: {}", e))
            }
        }
    }
    
    /// Simulate transaction before execution
    async fn simulate_transaction(&self, transaction: &Transaction, rpc_client: &RpcClient) -> Result<()> {
        info!("🔍 Simulating transaction...");
        
        match rpc_client.simulate_transaction(transaction) {
            Ok(result) => {
                if let Some(err) = result.value.err {
                    error!("❌ Simulation failed: {:?}", err);
                    return Err(anyhow::anyhow!("Simulation failed: {:?}", err));
                }
                
                if let Some(logs) = &result.value.logs {
                    info!("📝 Simulation logs:");
                    for log in logs {
                        info!("  {}", log);
                    }
                }
                
                if let Some(units_consumed) = result.value.units_consumed {
                    info!("⚡ Compute units consumed: {}", units_consumed);
                }
                
                info!("✅ Transaction simulation successful");
                Ok(())
            }
            Err(e) => {
                error!("❌ Failed to simulate transaction: {}", e);
                Err(anyhow::anyhow!("Simulation failed: {}", e))
            }
        }
    }
    
    /// Get adapters for transaction building (placeholder - needs proper implementation)
    fn get_adapters_for_transaction(&self) -> Result<Vec<Box<dyn crate::exchanges::DexAdapter>>> {
        // TODO: This is a placeholder. In a real implementation, we would need to:
        // 1. Extract adapters from the scanner
        // 2. Or recreate them based on configuration
        // For now, return empty vec - this will need to be fixed
        
        error!("⚠️ get_adapters_for_transaction not fully implemented");
        Err(anyhow::anyhow!("Adapter access not implemented - requires architecture refactoring"))
    }
}
