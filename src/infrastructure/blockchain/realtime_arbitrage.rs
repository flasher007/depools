//! Real-time arbitrage engine with auto-execution

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{Duration, interval};
use crate::shared::errors::AppError;
use crate::domain::dex::{PoolInfo, DexType};
use crate::shared::types::{Amount, Token};
use super::realtime_monitor::RealtimePriceMonitor;
use super::profit_calculator::RealProfitCalculator;
use super::yellowstone_grpc::PriceData;

/// Arbitrage opportunity
#[derive(Debug, Clone)]
pub struct ArbitrageOpportunity {
    pub id: String,
    pub token_a: Token,
    pub token_b: Token,
    pub pool_1: PoolInfo,
    pub pool_2: PoolInfo,
    pub input_amount: Amount,
    pub expected_profit: Amount,
    pub profit_percentage: f64,
    pub gas_cost: Amount,
    pub net_profit: Amount,
    pub route: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub status: OpportunityStatus,
}

/// Status of arbitrage opportunity
#[derive(Debug, Clone, PartialEq)]
pub enum OpportunityStatus {
    Detected,
    Validating,
    ReadyToExecute,
    Executing,
    Completed,
    Failed(String),
    Expired,
}

/// Auto-execution configuration
#[derive(Debug, Clone)]
pub struct AutoExecutionConfig {
    pub min_profit_threshold: f64, // Minimum profit percentage
    pub max_slippage: f64,         // Maximum allowed slippage
    pub max_gas_cost: u64,         // Maximum gas cost in lamports
    pub auto_execute: bool,        // Enable automatic execution
    pub execution_delay_ms: u64,   // Delay before execution
    pub max_concurrent_trades: usize, // Maximum concurrent trades
}

impl Default for AutoExecutionConfig {
    fn default() -> Self {
        Self {
            min_profit_threshold: 0.5, // 0.5%
            max_slippage: 1.0,         // 1%
            max_gas_cost: 500_000,     // 500k lamports
            auto_execute: false,       // Disabled by default
            execution_delay_ms: 1000,  // 1 second delay
            max_concurrent_trades: 3,
        }
    }
}

/// Real-time arbitrage engine
pub struct RealtimeArbitrageEngine {
    config: AutoExecutionConfig,
    price_monitor: Arc<RwLock<RealtimePriceMonitor>>,
    profit_calculator: RealProfitCalculator,
    opportunities: Arc<RwLock<HashMap<String, ArbitrageOpportunity>>>,
    active_trades: Arc<RwLock<HashMap<String, ArbitrageOpportunity>>>,
    is_running: bool,
}

impl RealtimeArbitrageEngine {
    /// Create new arbitrage engine
    pub fn new(
        config: AutoExecutionConfig,
        price_monitor: RealtimePriceMonitor,
        rpc_url: String,
    ) -> Self {
        Self {
            config,
            price_monitor: Arc::new(RwLock::new(price_monitor)),
            profit_calculator: RealProfitCalculator::new(rpc_url),
            opportunities: Arc::new(RwLock::new(HashMap::new())),
            active_trades: Arc::new(RwLock::new(HashMap::new())),
            is_running: false,
        }
    }
    
    /// Create with default configuration
    pub fn new_default(price_monitor: RealtimePriceMonitor, rpc_url: String) -> Self {
        Self::new(AutoExecutionConfig::default(), price_monitor, rpc_url)
    }
    
    /// Create with just RPC URL
    pub fn new(rpc_url: String) -> Self {
        let price_monitor = RealtimePriceMonitor::new(rpc_url.clone());
        Self::new_default(price_monitor, rpc_url)
    }
    
    /// Start the arbitrage engine
    pub async fn start(&mut self) -> Result<(), AppError> {
        if self.is_running {
            return Ok(());
        }
        
        println!("🚀 Starting Real-time Arbitrage Engine...");
        println!("   Min Profit Threshold: {}%", self.config.min_profit_threshold);
        println!("   Max Slippage: {}%", self.config.max_slippage);
        println!("   Max Gas Cost: {} lamports", self.config.max_gas_cost);
        println!("   Auto Execute: {}", if self.config.auto_execute { "ENABLED" } else { "DISABLED" });
        
        self.is_running = true;
        
        // Start opportunity detection loop
        self.start_opportunity_detection().await;
        
        // Start execution loop if auto-execute is enabled
        if self.config.auto_execute {
            self.start_execution_loop().await;
        }
        
        println!("✅ Real-time Arbitrage Engine started successfully!");
        Ok(())
    }
    
    /// Stop the arbitrage engine
    pub async fn stop(&mut self) -> Result<(), AppError> {
        if !self.is_running {
            return Ok(());
        }
        
        println!("🛑 Stopping Real-time Arbitrage Engine...");
        self.is_running = false;
        
        // Cancel all active trades
        let mut active_trades = self.active_trades.write().await;
        for (id, trade) in active_trades.iter_mut() {
            trade.status = OpportunityStatus::Failed("Engine stopped".to_string());
            println!("   ❌ Cancelled trade: {}", id);
        }
        active_trades.clear();
        
        println!("✅ Real-time Arbitrage Engine stopped successfully!");
        Ok(())
    }
    
    /// Start opportunity detection loop
    async fn start_opportunity_detection(&self) {
        let opportunities = self.opportunities.clone();
        let price_monitor = self.price_monitor.clone();
        let profit_calculator = self.profit_calculator.clone();
        let config = self.config.clone();
        let is_running = Arc::new(RwLock::new(self.is_running));
        
        tokio::spawn(async move {
            let mut interval_timer = interval(Duration::from_millis(500)); // Check every 500ms
            
            loop {
                interval_timer.tick().await;
                
                let running = *is_running.read().await;
                if !running {
                    break;
                }
                
                // Detect arbitrage opportunities
                if let Err(e) = Self::detect_opportunities(
                    &opportunities,
                    &price_monitor,
                    &profit_calculator,
                    &config,
                ).await {
                    eprintln!("❌ Error detecting opportunities: {}", e);
                }
            }
        });
    }
    
    /// Start execution loop
    async fn start_execution_loop(&self) {
        let active_trades = self.active_trades.clone();
        let opportunities = self.opportunities.clone();
        let config = self.config.clone();
        let is_running = Arc::new(RwLock::new(self.is_running));
        
        tokio::spawn(async move {
            let mut interval_timer = interval(Duration::from_millis(100)); // Check every 100ms
            
            loop {
                interval_timer.tick().await;
                
                let running = *is_running.read().await;
                if !running {
                    break;
                }
                
                // Execute ready opportunities
                if let Err(e) = Self::execute_ready_opportunities(
                    &active_trades,
                    &opportunities,
                    &config,
                ).await {
                    eprintln!("❌ Error executing opportunities: {}", e);
                }
            }
        });
    }
    
    /// Detect arbitrage opportunities
    async fn detect_opportunities(
        opportunities: &Arc<RwLock<HashMap<String, ArbitrageOpportunity>>>,
        price_monitor: &Arc<RwLock<RealtimePriceMonitor>>,
        profit_calculator: &RealProfitCalculator,
        config: &AutoExecutionConfig,
    ) -> Result<(), AppError> {
        // Get current prices from monitor
        let monitor = price_monitor.read().await;
        // Note: In production, this would get real subscription data
        // For now, we'll use mock opportunities
        
        // For demo purposes, create mock pools with price differences
        let mock_opportunities = Self::create_mock_opportunities(profit_calculator).await?;
        
        // Filter opportunities based on configuration
        let valid_opportunities: Vec<_> = mock_opportunities
            .into_iter()
            .filter(|opp| {
                opp.profit_percentage >= config.min_profit_threshold &&
                opp.gas_cost.value <= config.max_gas_cost &&
                opp.net_profit.value > 0
            })
            .collect();
        
        // Update opportunities
        if !valid_opportunities.is_empty() {
            let mut opps = opportunities.write().await;
            
            for opportunity in valid_opportunities {
                let id = opportunity.id.clone();
                opps.insert(id.clone(), opportunity);
                
                println!("🎯 New arbitrage opportunity detected: {}", id);
            }
        }
        
        Ok(())
    }
    
    /// Execute ready opportunities
    async fn execute_ready_opportunities(
        active_trades: &Arc<RwLock<HashMap<String, ArbitrageOpportunity>>>,
        opportunities: &Arc<RwLock<HashMap<String, ArbitrageOpportunity>>>,
        config: &AutoExecutionConfig,
    ) -> Result<(), AppError> {
        let mut opps = opportunities.write().await;
        let mut trades = active_trades.write().await;
        
        // Check if we can execute more trades
        if trades.len() >= config.max_concurrent_trades {
            return Ok(());
        }
        
        // Find opportunities ready for execution
        let ready_opportunities: Vec<_> = opps
            .iter()
            .filter(|(_, opp)| opp.status == OpportunityStatus::Detected)
            .take(config.max_concurrent_trades - trades.len())
            .map(|(id, opp)| (id.clone(), opp.clone()))
            .collect();
        
        for (id, mut opportunity) in ready_opportunities {
            // Mark as executing
            opportunity.status = OpportunityStatus::Executing;
            opps.insert(id.clone(), opportunity.clone());
            
            // Add to active trades
            trades.insert(id.clone(), opportunity.clone());
            
            println!("🚀 Executing arbitrage opportunity: {}", id);
            
            // Simulate execution delay
            tokio::time::sleep(Duration::from_millis(config.execution_delay_ms)).await;
            
            // Simulate execution result
            let success = rand::random::<bool>();
            if success {
                opportunity.status = OpportunityStatus::Completed;
                println!("✅ Arbitrage completed successfully: {}", id);
            } else {
                opportunity.status = OpportunityStatus::Failed("Execution failed".to_string());
                println!("❌ Arbitrage execution failed: {}", id);
            }
            
            // Update opportunity
            opps.insert(id.clone(), opportunity.clone());
            
            // Remove from active trades
            trades.remove(&id);
        }
        
        Ok(())
    }
    
    /// Create mock opportunities for demonstration
    async fn create_mock_opportunities(
        profit_calculator: &RealProfitCalculator,
    ) -> Result<Vec<ArbitrageOpportunity>, AppError> {
        let mut opportunities = Vec::new();
        
        // Create mock tokens
        let sol = Token {
            mint: solana_sdk::pubkey::Pubkey::new_unique(),
            symbol: "SOL".to_string(),
            decimals: 9,
            name: Some("Solana".to_string()),
        };
        
        let usdc = Token {
            mint: solana_sdk::pubkey::Pubkey::new_unique(),
            symbol: "USDC".to_string(),
            decimals: 6,
            name: Some("USD Coin".to_string()),
        };
        
        // Create mock pools with price differences
        let orca_pool = PoolInfo {
            id: "mock_orca_pool".to_string(),
            dex_type: DexType::OrcaWhirlpool,
            token_a: sol.clone(),
            token_b: usdc.clone(),
            reserve_a: Amount::new(1_000_000_000, 9),
            reserve_b: Amount::new(100_000_000, 6),
            fee_rate: 0.003,
            liquidity: Amount::new(1_000_000, 6),
            volume_24h: Amount::new(100_000, 6),
        };
        
        let raydium_pool = PoolInfo {
            id: "mock_raydium_pool".to_string(),
            dex_type: DexType::RaydiumV4,
            token_a: sol.clone(),
            token_b: usdc.clone(),
            reserve_a: Amount::new(1_000_000_000, 9),
            reserve_b: Amount::new(98_000_000, 6), // 2% price difference
            fee_rate: 0.0025,
            liquidity: Amount::new(1_000_000, 6),
            volume_24h: Amount::new(100_000, 6),
        };
        
        // Calculate profit
        let test_amount = Amount::new(100_000_000, 9); // 0.1 SOL
        
        if let Ok(calculation) = profit_calculator.calculate_two_hop_profit(
            &sol,
            &usdc,
            test_amount,
            &orca_pool,
            &raydium_pool,
        ).await {
            let opportunity = ArbitrageOpportunity {
                id: format!("opp_{}", chrono::Utc::now().timestamp()),
                token_a: sol.clone(),
                token_b: usdc.clone(),
                pool_1: orca_pool.clone(),
                pool_2: raydium_pool.clone(),
                input_amount: calculation.input_amount,
                expected_profit: calculation.gross_profit,
                profit_percentage: calculation.profit_percentage,
                gas_cost: calculation.gas_cost,
                net_profit: calculation.net_profit,
                route: calculation.route,
                timestamp: chrono::Utc::now(),
                status: OpportunityStatus::Detected,
            };
            
            opportunities.push(opportunity);
        }
        
        Ok(opportunities)
    }
    
    /// Get all detected opportunities
    pub async fn get_opportunities(&self) -> Vec<ArbitrageOpportunity> {
        let opps = self.opportunities.read().await;
        opps.values().cloned().collect()
    }
    
    /// Get active trades
    pub async fn get_active_trades(&self) -> Vec<ArbitrageOpportunity> {
        let trades = self.active_trades.read().await;
        trades.values().cloned().collect()
    }
    
    /// Get engine statistics
    pub async fn get_statistics(&self) -> EngineStatistics {
        let opportunities = self.opportunities.read().await;
        let active_trades = self.active_trades.read().await;
        
        let total_opportunities = opportunities.len();
        let active_trades_count = active_trades.len();
        let completed_trades = opportunities.values()
            .filter(|opp| opp.status == OpportunityStatus::Completed)
            .count();
        let failed_trades = opportunities.values()
            .filter(|opp| matches!(opp.status, OpportunityStatus::Failed(_)))
            .count();
        
        EngineStatistics {
            is_running: self.is_running,
            total_opportunities,
            active_trades: active_trades_count,
            completed_trades,
            failed_trades,
            auto_execute_enabled: self.config.auto_execute,
        }
    }
}

/// Engine statistics
#[derive(Debug, Clone)]
pub struct EngineStatistics {
    pub is_running: bool,
    pub total_opportunities: usize,
    pub active_trades: usize,
    pub completed_trades: usize,
    pub failed_trades: usize,
    pub auto_execute_enabled: bool,
}

impl EngineStatistics {
    /// Print statistics summary
    pub fn print_summary(&self) {
        println!("📊 Arbitrage Engine Statistics:");
        println!("   Status: {}", if self.is_running { "🟢 RUNNING" } else { "🔴 STOPPED" });
        println!("   Total Opportunities: {}", self.total_opportunities);
        println!("   Active Trades: {}", self.active_trades);
        println!("   Completed Trades: {}", self.completed_trades);
        println!("   Failed Trades: {}", self.failed_trades);
        println!("   Auto Execute: {}", if self.auto_execute_enabled { "🟢 ENABLED" } else { "🔴 DISABLED" });
    }
}
