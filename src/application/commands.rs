//! CLI commands and handlers
use clap::{Parser, Subcommand};
use crate::shared::types::BotConfig;
use crate::shared::errors::AppError;
use crate::application::services::ArbitrageService;

/// Main CLI application
#[derive(Parser)]
#[command(name = "depools")]
#[command(about = "Solana Arbitrage Bot v2 - Built with DDD principles")]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
    
    /// Configuration file path
    #[arg(short, long, default_value = "Config.toml")]
    pub config: std::path::PathBuf,
}

/// Available commands
#[derive(Subcommand)]
pub enum Commands {
    /// Start the arbitrage bot
    Start {
        /// Minimum profit threshold in percentage
        #[arg(long, default_value_t = 0.5)]
        min_profit: f64,
        
        /// Maximum slippage tolerance in percentage
        #[arg(long, default_value_t = 1.0)]
        max_slippage: f64,
    },
    
    /// Discover and analyze pools
    Pools {
        /// Save pool data to files
        #[arg(short, long)]
        save: bool,
        
        /// Minimum liquidity threshold in SOL
        #[arg(short, long, default_value_t = 10.0)]
        min_liquidity: f64,
    },
    
    /// Monitor prices across DEXes
    Monitor {
        /// Update interval in milliseconds
        #[arg(short, long, default_value_t = 1000)]
        interval: u64,
        
        /// Price change threshold in percentage
        #[arg(short, long, default_value_t = 0.1)]
        threshold: f64,
    },
    
    /// Discover pools using blockchain reading
    Discover {
        /// Save discovered pools to file
        #[arg(long, default_value_t = false)]
        save: bool,
    },
    
    /// Read real blockchain data
    ReadData {
        /// Token mint to read
        #[arg(short, long)]
        mint: String,
    },
    
    /// Calculate arbitrage profit
    CalculateProfit {
        /// Amount to calculate with
        #[arg(short, long, default_value_t = 1.0)]
        amount: f64,
    },
    
    /// Analyze arbitrage opportunities
    AnalyzeArbitrage {
        /// Amount to analyze with
        #[arg(short, long, default_value_t = 1.0)]
        amount: f64,
    },
    
    /// Monitor prices in real-time
    MonitorRealtime {
        /// Duration to monitor (seconds)
        #[arg(short, long, default_value_t = 30)]
        duration: u64,
    },
    
    /// Run arbitrage engine
    RunEngine {
        /// Duration to run (seconds)
        #[arg(short, long, default_value_t = 30)]
        duration: u64,
        /// Enable auto-execution
        #[arg(long, default_value_t = false)]
        auto_execute: bool,
    },
    
    /// Execute arbitrage transaction
    ExecuteTransaction {
        /// Amount to execute with
        #[arg(short, long, default_value_t = 0.1)]
        amount: f64,
        /// Simulate only (don't execute)
        #[arg(long, default_value_t = false)]
        simulate: bool,
    },
}

/// Command execution service
pub struct CommandExecutor;

impl CommandExecutor {
    /// Execute a command using the arbitrage service
    pub async fn execute(command: &Commands, config: &BotConfig) -> Result<(), AppError> {
        let service = ArbitrageService::new(config)?;
        
        match command {
            Commands::Start { min_profit, max_slippage } => {
                service.start_arbitrage_bot(*min_profit, *max_slippage).await
            }
            Commands::Pools { save, min_liquidity } => {
                service.discover_pools(*save, *min_liquidity).await
            }
            Commands::Monitor { interval, threshold } => {
                service.monitor_prices(*interval, *threshold).await
            }
            Commands::Discover { save } => {
                service.discover_pools_blockchain(*save).await
            }
            Commands::ReadData { mint } => {
                service.read_blockchain_data(mint).await
            }
            Commands::CalculateProfit { amount } => {
                service.calculate_arbitrage_profit(*amount).await
            }
            Commands::AnalyzeArbitrage { amount } => {
                service.analyze_arbitrage_opportunities(*amount).await
            }
            Commands::MonitorRealtime { duration } => {
                service.monitor_realtime_prices(*duration).await
            }
            Commands::RunEngine { duration, auto_execute } => {
                service.run_arbitrage_engine(*duration, *auto_execute).await
            }
            Commands::ExecuteTransaction { amount, simulate } => {
                service.execute_arbitrage_transaction(*amount, *simulate).await
            }
        }
    }
}
