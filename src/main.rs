//! Depools - Solana Arbitrage Bot v2
//! Main entry point for production use on Solana mainnet

use depools::shared::types::BotConfig;
use depools::shared::errors::AppError;
use depools::application::{Cli, CommandExecutor, ConfigService};
use clap::Parser;
use tracing_subscriber::{fmt, EnvFilter};

#[tokio::main]
async fn main() -> Result<(), AppError> {
    // Initialize logging with proper configuration
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));
    
    fmt()
        .with_env_filter(env_filter)
        .with_target(false)
        .with_thread_ids(false)
        .with_thread_names(false)
        .init();
    
    println!("🚀 Depools - Solana Arbitrage Bot v2");
    println!("🌐 Connecting to Solana mainnet...");
    
    let cli = Cli::parse();
    
    // Load configuration
    let config = load_config(&cli.config)?;
    
    // Validate configuration for mainnet
    validate_mainnet_config(&config)?;
    
    println!("✅ Configuration loaded successfully");
    println!("🔗 RPC Endpoint: {}", config.network.rpc_url);
    println!("💰 Min Profit Threshold: {}%", config.min_profit_threshold);
    println!("⚠️  Max Slippage: {}%", config.max_slippage);
    
    // Execute command using the production architecture
    CommandExecutor::execute(&cli.command, &config).await?;
    
    Ok(())
}

fn load_config(path: &std::path::PathBuf) -> Result<BotConfig, AppError> {
    ConfigService::load_config(path)
}

fn validate_mainnet_config(config: &BotConfig) -> Result<(), AppError> {
    // Ensure we're using mainnet RPC
    if config.network.rpc_url.contains("devnet") {
        return Err(AppError::ConfigError(
            "Devnet RPC detected! This bot is configured for mainnet only.".to_string()
        ));
    }
    
    // Validate profit threshold
    if config.min_profit_threshold <= 0.0 {
        return Err(AppError::ConfigError(
            "Invalid profit threshold! Must be greater than 0.".to_string()
        ));
    }
    
    // Validate slippage
    if config.max_slippage <= 0.0 || config.max_slippage > 100.0 {
        return Err(AppError::ConfigError(
            "Invalid slippage! Must be between 0 and 100.".to_string()
        ));
    }
    
    Ok(())
}