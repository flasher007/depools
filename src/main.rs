mod app;
mod config;
mod report;
mod math;
mod exchanges;
mod opportunity;

use anyhow::Result;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about = "AMM Arbitrage CLI for Solana pools with advanced DEX support")]
struct Args {
    /// RPC endpoint URL
    #[arg(long, default_value = "https://api.mainnet-beta.solana.com")]
    rpc_url: Option<String>,
    
    /// Path to keypair file
    #[arg(long)]
    keypair: Option<String>,
    
    /// Amount to trade (in base token)
    #[arg(long)]
    amount_in: Option<f64>,
    
    /// Minimum spread threshold in basis points
    #[arg(long, default_value = "50")]
    spread_threshold_bps: u32,
    
    /// Slippage tolerance in basis points
    #[arg(long, default_value = "100")]
    slippage_bps: u32,
    
    /// Priority fee in microlamports
    #[arg(long, default_value = "1000")]
    priority_fee: u64,
    
    /// Only simulate transaction without executing
    #[arg(long)]
    simulate_only: bool,
    
    /// Path to config file (optional)
    #[arg(long)]
    config: Option<String>,
    

    
    /// Pool addresses to monitor (comma-separated)
    #[arg(long)]
    pools: Option<String>,
    
    /// Base token mint address (overrides config)
    #[arg(long)]
    base_token_mint: Option<String>,
    
    /// Quote token mint address (overrides config)
    #[arg(long)]
    quote_token_mint: Option<String>,
    
    /// Raydium V4 program ID (overrides config)
    #[arg(long)]
    raydium_program: Option<String>,
    
    /// Orca Whirlpool program ID (overrides config)
    #[arg(long)]
    orca_program: Option<String>,
    
    /// SPL Token program ID (overrides config)
    #[arg(long)]
    spl_token_program: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt().with_env_filter("info").init();
    let args = Args::parse();

    // Load base configuration from file if provided
    let base_config = if let Some(config_path) = &args.config {
        Some(config::Config::from_file(config_path)?)
    } else {
        None
    };

    // Create AppCfg with priority: CLI args > Config file > Defaults
    let app_cfg = if let Some(cfg) = base_config {
        // Start with config file, then override with CLI args
        let mut app_cfg = app::AppCfg::from_config(cfg, args.simulate_only)?;
        
        // Override with CLI args if provided (CLI has higher priority)
        if let Some(rpc_url) = args.rpc_url {
            app_cfg.rpc_url = rpc_url;
        }
        if let Some(keypair) = args.keypair {
            app_cfg.keypair_path = keypair;
        }
        if let Some(amount_in) = args.amount_in {
            app_cfg.amount_in = amount_in;
        }
        if args.spread_threshold_bps != 50 { // Only override if not default
            app_cfg.spread_threshold_bps = args.spread_threshold_bps;
        }
        if args.slippage_bps != 100 { // Only override if not default
            app_cfg.slippage_bps = args.slippage_bps;
        }
        if args.priority_fee != 1000 { // Only override if not default
            app_cfg.priority_fee = args.priority_fee;
        }
        
        // Override pool addresses if provided via CLI
        if let Some(pools) = args.pools {
            app_cfg.pool_addresses = pools.split(',').map(|s| s.trim().to_string()).collect();
        }
        

        
        // Override tokens and programs if provided via CLI
        if let Some(base_token_mint) = args.base_token_mint {
            app_cfg.base_token_mint = Some(base_token_mint);
        }
        if let Some(quote_token_mint) = args.quote_token_mint {
            app_cfg.quote_token_mint = Some(quote_token_mint);
        }
        if let Some(raydium_program) = args.raydium_program {
            app_cfg.raydium_program = Some(raydium_program);
        }
        if let Some(orca_program) = args.orca_program {
            app_cfg.orca_program = Some(orca_program);
        }
        if let Some(spl_token_program) = args.spl_token_program {
            app_cfg.spl_token_program = Some(spl_token_program);
        }
        
        app_cfg
    } else {
        // Use CLI args only (required fields must be provided)
        let rpc_url = args.rpc_url.ok_or_else(|| anyhow::anyhow!("--rpc-url is required when not using --config"))?;
        let keypair = args.keypair.ok_or_else(|| anyhow::anyhow!("--keypair is required when not using --config"))?;
        let amount_in = args.amount_in.ok_or_else(|| anyhow::anyhow!("--amount-in is required when not using --config"))?;
        
        // Parse pool addresses if provided
        let pool_addresses = if let Some(pools) = args.pools {
            pools.split(',').map(|s| s.trim().to_string()).collect()
        } else {
            vec!["58oQChx4yWmvKdwLLZzBi4ChoCc2fqCUWBkwMihLYQo2".to_string(), // SOL-USDC Raydium V4
                 "HJPjoWUrhoZzkNfRpHuieeFk9WcZWjwy6PBjZ81ngndJ".to_string()] // SOL-USDC Orca Whirlpool
        };
        
        let mut app_cfg = app::AppCfg::from_cli_args(
            rpc_url,
            keypair,
            amount_in,
            args.spread_threshold_bps,
            args.slippage_bps,
            args.priority_fee,
            args.simulate_only,
        )?;
        
        app_cfg.pool_addresses = pool_addresses;

        
        // Override tokens and programs if provided via CLI
        if let Some(base_token_mint) = args.base_token_mint {
            app_cfg.base_token_mint = Some(base_token_mint);
        }
        if let Some(quote_token_mint) = args.quote_token_mint {
            app_cfg.quote_token_mint = Some(quote_token_mint);
        }
        if let Some(raydium_program) = args.raydium_program {
            app_cfg.raydium_program = Some(raydium_program);
        }
        if let Some(orca_program) = args.orca_program {
            app_cfg.orca_program = Some(orca_program);
        }
        if let Some(spl_token_program) = args.spl_token_program {
            app_cfg.spl_token_program = Some(spl_token_program);
        }
        
        app_cfg
    };

    app::run(app_cfg).await
}
