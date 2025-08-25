//! Depools - Solana Arbitrage Bot v2
//! Main entry point for production use on Solana mainnet

use clap::Parser;
use depools::application::{Cli, CommandExecutor};
use depools::shared::config::ConfigLoader;
use depools::shared::errors::AppError;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<(), AppError> {
    // Initialize logging
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "depools=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Parse CLI arguments
    let cli = Cli::parse();
    
    // Load configuration
    let config = ConfigLoader::load_config()?;
    
    tracing::info!("🚀 Solana DEX Pool Discovery and Arbitrage Bot v{}", env!("CARGO_PKG_VERSION"));
    tracing::info!("📁 Конфигурация загружена из: {}", config.network.rpc_url);
    
    // Execute the selected command
    CommandExecutor::execute(cli.command, config).await?;
    
    Ok(())
}