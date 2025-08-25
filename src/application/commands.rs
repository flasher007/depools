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
    /// Discover and analyze pools across all DEXes
    Pools {
        /// Save pool data to files
        #[arg(short, long)]
        save: bool,
        
        /// Minimum liquidity threshold in SOL
        #[arg(short, long, default_value_t = 10.0)]
        min_liquidity: f64,
    },
    
    /// Monitor prices and find arbitrage opportunities
    Monitor {
        /// Duration to monitor (seconds)
        #[arg(short, long, default_value_t = 30)]
        duration: u64,
        
        /// Minimum profit threshold in percentage
        #[arg(long, default_value_t = 0.5)]
        min_profit: f64,
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
            Commands::Pools { save, min_liquidity } => {
                service.discover_pools(*save, *min_liquidity).await
            }
            Commands::Monitor { duration, min_profit } => {
                service.monitor_arbitrage_opportunities(*duration, *min_profit).await
            }
            Commands::ExecuteTransaction { amount, simulate } => {
                service.execute_arbitrage_transaction(*amount, *simulate).await
            }
        }
    }
}
