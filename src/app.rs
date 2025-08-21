// src/app.rs
use anyhow::Result;
use tracing::{info, error};
use solana_sdk::{
    signature::{read_keypair_file, Signer},
    transaction::Transaction,
};
use solana_client::rpc_client::RpcClient;
use std::sync::Arc;

use crate::config::Config;
use crate::opportunity::scanner::CrossDexScanner;
use crate::opportunity::scanner::AsyncOpportunityScanner;
use crate::opportunity::arbitrage::ArbitrageEngine;

use crate::exchanges::factory;

#[derive(Debug, Clone)]
pub struct AppCfg {
    pub simulate_only: bool,
    pub rpc_url: String,
    pub keypair_path: String,
    pub amount_in: f64,
    pub spread_threshold_bps: u32,
    pub slippage_bps: u32,
    pub priority_fee: u64,
    pub pool_addresses: Vec<String>,

    
    // Token and program overrides
    pub base_token_mint: Option<String>,
    pub quote_token_mint: Option<String>,
    pub raydium_program: Option<String>,
    pub orca_program: Option<String>,
    pub spl_token_program: Option<String>,
}

impl AppCfg {
    pub fn from_config(cfg: Config, override_simulate: bool) -> Result<Self> {


        let pool_addresses = vec![
            cfg.pools.pool_a.clone(),
            cfg.pools.pool_b.clone(),
        ];

        Ok(Self {
            simulate_only: if override_simulate { true } else { cfg.trade.simulate_only.unwrap_or(false) },
            rpc_url: cfg.rpc.url,
            keypair_path: cfg.wallet.keypair,
            amount_in: cfg.trade.amount_in,
            spread_threshold_bps: cfg.trade.spread_threshold_bps,
            slippage_bps: cfg.trade.slippage_bps,
            priority_fee: cfg.trade.priority_fee_microlamports,
            pool_addresses,
            
            // Token and program overrides (None = use config defaults)
            base_token_mint: None,
            quote_token_mint: None,
            raydium_program: None,
            orca_program: None,
            spl_token_program: None,
        })
    }

    pub fn from_cli_args(
        rpc_url: String,
        keypair: String,
        amount_in: f64,
        spread_threshold_bps: u32,
        slippage_bps: u32,
        priority_fee: u64,
        simulate_only: bool,
    ) -> Result<Self> {
        Ok(Self {
            simulate_only,
            rpc_url,
            keypair_path: keypair,
            amount_in,
            spread_threshold_bps,
            slippage_bps,
            priority_fee,
            pool_addresses: vec![
                "58oQChx4yWmvKdwLLZzBi4ChoCc2fqCUWBkwMihLYQo2".to_string(), // SOL-USDC Raydium V4
                "HJPjoWUrhoZzkNfRpHuieeFk9WcZWjwy6PBjZ81ngndJ".to_string(), // SOL-USDC Orca Whirlpool
            ],

            
            // Token and program overrides (None = use defaults)
            base_token_mint: None,
            quote_token_mint: None,
            raydium_program: None,
            orca_program: None,
            spl_token_program: None,
        })
    }
}

pub async fn run(app_cfg: AppCfg) -> Result<()> {
    info!("Starting AMM arbitrage engine with new architecture");
    info!("Configuration: {:?}", app_cfg);

    // Validate configuration
    info!("Validating pool addresses...");
    for (i, pool_addr) in app_cfg.pool_addresses.iter().enumerate() {
        match pool_addr.parse::<solana_sdk::pubkey::Pubkey>() {
            Ok(_) => info!("✅ Pool {} address is valid: {}", i + 1, pool_addr),
            Err(e) => {
                error!("❌ Invalid pool {} address {}: {}", i + 1, pool_addr, e);
                return Err(anyhow::anyhow!("Invalid pool address: {}", pool_addr));
            }
        }
    }

    // Initialize RPC client
    let rpc_client = Arc::new(RpcClient::new(app_cfg.rpc_url.clone()));

    // Initialize arbitrage engine
    let scanner = Arc::new(CrossDexScanner::new(app_cfg.clone().into())?);
    let arbitrage_engine = ArbitrageEngine::new(
        scanner.clone(),
        app_cfg.spread_threshold_bps as i32,
    );


    
    // Load keypair
    let keypair = read_keypair_file(&app_cfg.keypair_path)
        .map_err(|e| anyhow::anyhow!("Failed to load keypair: {}", e))?;
    
    info!("Loaded keypair: {}", keypair.pubkey());

    // Main arbitrage loop
    run_polling_mode(
        app_cfg,
        rpc_client,
        arbitrage_engine,
        keypair,
    ).await?;

    Ok(())
}



async fn run_polling_mode(
    app_cfg: AppCfg,
    rpc_client: Arc<RpcClient>,
    arbitrage_engine: ArbitrageEngine,
    keypair: solana_sdk::signature::Keypair,
) -> Result<()> {
    info!("Running in polling mode");
    
    let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(5));
    
    loop {
        interval.tick().await;
        
        info!("Scanning for arbitrage opportunities...");
        
        // Get scanner from arbitrage engine
        let scanner = arbitrage_engine.get_scanner();
        
        // Scan for opportunities using async scanner with configuration parameters
        let opportunities = scanner.scan_opportunities_async(
            &app_cfg.pool_addresses,
            app_cfg.amount_in as u64,
            app_cfg.spread_threshold_bps,
            app_cfg.slippage_bps,
            app_cfg.priority_fee,
        ).await?;
        
        for opportunity in opportunities {
            info!("Found arbitrage opportunity: {:?}", opportunity);
            
            if !app_cfg.simulate_only {
                // Execute arbitrage
                execute_arbitrage(&rpc_client, &keypair, &opportunity, app_cfg.clone()).await?;
            }
        }
        
        if app_cfg.simulate_only {
            info!("Simulation mode - not executing real transactions");
            break;
        }
    }

    Ok(())
}

async fn execute_arbitrage(
    rpc_client: &Arc<RpcClient>,
    keypair: &solana_sdk::signature::Keypair,
    opportunity: &crate::exchanges::types::ArbitrageOpportunity,
    app_cfg: AppCfg,
) -> Result<()> {
    info!("Executing arbitrage opportunity: {}", opportunity.id);
    info!("Route A: {:?}", opportunity.route_a);
    info!("Route B: {:?}", opportunity.route_b);
    
    // 1. Создаем swap инструкцию для Route A
    let dex_a = opportunity.route_a.hops[0].dex_label;
            let adapter_a = factory::create_adapter(dex_a, app_cfg.clone().into())?;
    let min_out_a = opportunity.route_a.hops[0].amount_out.saturating_sub(
        (opportunity.route_a.hops[0].amount_in * opportunity.route_a.hops[0].fee_bps as u64) / 10000
    );
    let swap_instruction_a = adapter_a.create_swap_instruction(
        &crate::exchanges::types::SwapQuote {
            pool_address: opportunity.route_a.hops[0].pool_address,
            dex_label: dex_a,
            token_in: opportunity.route_a.hops[0].token_in,
            token_out: opportunity.route_a.hops[0].token_out,
            amount_in: opportunity.route_a.hops[0].amount_in,
            amount_out: opportunity.route_a.hops[0].amount_out,
            min_amount_out: min_out_a,
            price_impact_bps: 0,
            fee_amount: (opportunity.route_a.hops[0].amount_in * opportunity.route_a.hops[0].fee_bps as u64) / 10000,
            route: opportunity.route_a.clone(),
        },
        &keypair.pubkey(),
        min_out_a,
    )?;
    
    // 2. Создаем swap инструкцию для Route B (обратное направление)
    let dex_b = opportunity.route_b.hops[0].dex_label;
            let adapter_b = factory::create_adapter(dex_b, app_cfg.clone().into())?;
    let min_out_b = opportunity.route_a.hops[0].amount_in.saturating_sub(
        (opportunity.route_b.hops[0].amount_out * opportunity.route_b.hops[0].fee_bps as u64) / 10000
    );
    let swap_instruction_b = adapter_b.create_swap_instruction(
        &crate::exchanges::types::SwapQuote {
            pool_address: opportunity.route_b.hops[0].pool_address,
            dex_label: dex_b,
            token_in: opportunity.route_b.hops[0].token_out, // Обратное направление
            token_out: opportunity.route_b.hops[0].token_in, // Обратное направление
            amount_in: opportunity.route_a.hops[0].amount_out, // Используем выход из A как вход в B
            amount_out: opportunity.route_a.hops[0].amount_in, // Ожидаем вернуть исходный токен
            min_amount_out: min_out_b,
            price_impact_bps: 0,
            fee_amount: (opportunity.route_b.hops[0].amount_out * opportunity.route_b.hops[0].fee_bps as u64) / 10000,
            route: opportunity.route_b.clone(),
        },
        &keypair.pubkey(),
        min_out_b,
    )?;
    
    // 3. Создаем транзакцию с обеими инструкциями
    let mut transaction = Transaction::new_with_payer(
        &[swap_instruction_a, swap_instruction_b],
        Some(&keypair.pubkey()),
    );
    
    info!("✅ Created atomic transaction with 2 swap instructions");
    info!("📝 Transaction size: {} instructions", transaction.message.instructions.len());
    

    
    // Get latest blockhash and sign transaction
    let blockhash = rpc_client.get_latest_blockhash()?;
    transaction.sign(&[keypair], blockhash);
    
    // Simulate transaction first
    info!("🧪 Simulating transaction...");
    match rpc_client.simulate_transaction(&transaction) {
        Ok(simulation) => {
            info!("✅ Simulation successful:");
            info!("   - Compute units used: {:?}", simulation.value.units_consumed);
            info!("   - Logs: {:?}", simulation.value.logs);
            
            if let Some(err) = simulation.value.err {
                error!("❌ Simulation failed: {:?}", err);
                return Err(anyhow::anyhow!("Transaction simulation failed: {:?}", err));
            }
        }
        Err(e) => {
            error!("❌ Failed to simulate transaction: {}", e);
            return Err(anyhow::anyhow!("Simulation error: {}", e));
        }
    }
    
    // Send transaction (only if simulation succeeded)
    let signature = rpc_client.send_transaction(&transaction)?;
    info!("🚀 Arbitrage transaction sent: {}", signature);
    
    // Wait for confirmation
    rpc_client.confirm_transaction(&signature)?;
    info!("✅ Arbitrage transaction confirmed!");
    
    Ok(())
}


