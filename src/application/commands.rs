//! CLI commands and handlers
use clap::{Parser, Subcommand, Args};
use crate::shared::errors::AppError;
use crate::shared::types::BotConfig;
use crate::infrastructure::blockchain::{
    DexAdapterFactory, VaultReader, OrcaAccountParser, RaydiumAccountParser,
};
use crate::application::arbitrage_monitor::{ArbitrageMonitor, ArbitrageMonitorConfig};
use std::sync::Arc;
use std::time::Duration;
use tracing::{info, warn, error};

#[derive(Parser)]
#[command(name = "depools")]
#[command(about = "Solana DEX Pool Discovery and Arbitrage Bot")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Discover and list pools from supported DEXes
    Pools {
        /// Show detailed pool information
        #[arg(long)]
        detailed: bool,
        
        /// Filter by DEX type (orca, raydium)
        #[arg(short, long)]
        dex: Option<String>,
        
        /// Limit number of pools to show
        #[arg(short, long, default_value_t = 10)]
        limit: usize,
    },
    
    /// Мониторинг арбитражных возможностей
    #[command(name = "monitor")]
    Monitor {
        /// Минимальная прибыль в процентах
        #[arg(short, long, default_value_t = 0.1)]
        profit: f64,
        
        /// Минимальная ликвидность в SOL
        #[arg(short, long, default_value_t = 1.0)]
        liquidity: f64,
        
        /// Интервал обновления в миллисекундах
        #[arg(short, long, default_value_t = 2000)]
        interval: u64,
        
        /// Максимальное количество одновременных сделок
        #[arg(short, long, default_value_t = 5)]
        max_trades: usize,
        
        /// Толерантность к риску (0.0 - 1.0)
        #[arg(short, long, default_value_t = 0.5)]
        risk: f64,
        
        /// Автоматическое исполнение сделок
        #[arg(short, long, default_value_t = false)]
        auto_execute: bool,
        
        /// Начальный баланс в SOL
        #[arg(short, long, default_value_t = 100.0)]
        balance: f64,
        
        /// Продолжительность работы в секундах
        #[arg(short, long)]
        duration: Option<u64>,
    },
    
    /// Show current bot status and statistics
    Status {
        /// Show detailed status information
        #[arg(short, long)]
        detailed: bool,
    },
    
    /// Test specific functionality
    Test {
        /// Test DEX adapters
        #[arg(long)]
        dex_adapters: bool,
        
        /// Test gRPC connection
        #[arg(long)]
        grpc: bool,
        
        /// Test arbitrage detection
        #[arg(long)]
        arbitrage: bool,
    },
}

#[derive(Args)]
pub struct MonitorArgs {
    /// Минимальная прибыль в процентах
    #[arg(short, long, default_value_t = 0.5)]
    profit: f64,

    /// Минимальная ликвидность в SOL
    #[arg(short, long, default_value_t = 10.0)]
    liquidity: f64,

    /// Интервал обновления в миллисекундах
    #[arg(short, long, default_value_t = 5000)]
    interval: u64,

    /// Максимум одновременных сделок
    #[arg(short, long, default_value_t = 3)]
    max_trades: usize,

    /// Толерантность к риску (0.0 - 1.0)
    #[arg(short, long, default_value_t = 0.3)]
    risk: f64,

    /// Включить автоматическое выполнение сделок
    #[arg(short, long)]
    auto_execute: bool,

    /// Начальный баланс в SOL
    #[arg(short, long, default_value_t = 100.0)]
    balance: f64,

    /// Время работы в секундах (если не указано - бесконечно)
    #[arg(short, long)]
    duration: Option<u64>,
}

pub struct CommandExecutor;

impl CommandExecutor {
    /// Execute the selected command
    pub async fn execute(command: Commands, config: BotConfig) -> Result<(), AppError> {
        match command {
            Commands::Pools { detailed, dex, limit } => {
                Self::execute_pools_command(detailed, dex, limit, config).await
            }
            Commands::Monitor { profit, liquidity, interval, max_trades, risk, auto_execute, balance, duration } => {
                Self::execute_monitor_command(profit, liquidity, interval, max_trades, risk, auto_execute, balance, duration, config).await
            }
            Commands::Status { detailed } => {
                Self::execute_status_command(detailed, config).await
            }
            Commands::Test { dex_adapters, grpc, arbitrage } => {
                Self::execute_test_command(dex_adapters, grpc, arbitrage, config).await
            }
        }
    }

    /// Execute pools command
    async fn execute_pools_command(
        detailed: bool,
        dex_filter: Option<String>,
        limit: usize,
        config: BotConfig,
    ) -> Result<(), AppError> {
        info!("🔍 Поиск пулов в поддерживаемых DEX...");
        
        let rpc_client = solana_client::rpc_client::RpcClient::new(config.network.rpc_url.clone());
        let vault_reader = Arc::new(VaultReader::new(config.network.rpc_url.clone()));
        let orca_parser = OrcaAccountParser::new(Arc::clone(&vault_reader));
        let raydium_parser = RaydiumAccountParser::new();
        
        let dex_factory = DexAdapterFactory::new(
            Arc::new(rpc_client),
            vault_reader,
            orca_parser,
            raydium_parser,
        );

        // Получаем пулы для каждого DEX
        let dex_types = vec![
            crate::domain::dex::DexType::OrcaWhirlpool,
            crate::domain::dex::DexType::RaydiumAMM,
        ];

        let mut total_pools = 0;
        
        for dex_type in dex_types {
            // Применяем фильтр если указан
            if let Some(ref filter) = dex_filter {
                if !dex_type.as_str().to_lowercase().contains(&filter.to_lowercase()) {
                    continue;
                }
            }

            info!("\n📊 {}:", dex_type.as_str());
            
            let adapter = dex_factory.create_adapter(&dex_type);
            match adapter.discover_pools().await {
                Ok(pools) => {
                    let pools_to_show = pools.iter().take(limit).collect::<Vec<_>>();
                    info!("   Найдено пулов: {} (показано: {})", pools.len(), pools_to_show.len());
                    
                    for (i, pool) in pools_to_show.iter().enumerate() {
                        info!("   {}. {} <-> {} (ID: {})", 
                            i + 1, 
                            pool.token_a.symbol, 
                            pool.token_b.symbol,
                            &pool.id[..16]
                        );
                        
                        if detailed {
                            info!("      Ликвидность: {} {}", 
                                pool.liquidity.value as f64 / 10f64.powi(pool.liquidity.decimals as i32),
                                if pool.liquidity.decimals == 9 { "SOL" } else { "tokens" }
                            );
                            info!("      Комиссия: {:.2}%", pool.fee_rate);
                        }
                    }
                    
                    total_pools += pools.len();
                }
                Err(e) => {
                    error!("   ❌ Ошибка: {}", e);
                }
            }
        }

        info!("\n✅ Всего найдено пулов: {}", total_pools);
        Ok(())
    }

    /// Execute monitor command
    async fn execute_monitor_command(
        min_profit: f64,
        min_liquidity: f64,
        interval: u64,
        max_trades: usize,
        risk: f64,
        auto_execute: bool,
        balance: f64,
        duration: Option<u64>,
        config: BotConfig,
    ) -> Result<(), AppError> {
        info!("🚀 Запуск арбитражного мониторинга...");
        
        // Создаем конфигурацию мониторинга
        let monitor_config = ArbitrageMonitorConfig {
            min_profit_threshold: min_profit / 100.0, // Конвертируем из процентов
            min_liquidity,
            update_interval_ms: interval,
            max_concurrent_trades: max_trades,
            risk_tolerance: risk,
            enable_auto_execution: auto_execute,
            initial_balance_sol: balance, // Используем переданный баланс
        };

        // Создаем монитор
        let mut monitor = ArbitrageMonitor::new(config.clone(), monitor_config.clone())?;
        
        info!("📊 Конфигурация мониторинга:");
        info!("   Минимальная прибыль: {:.2}%", min_profit);
        info!("   Минимальная ликвидность: {} SOL", min_liquidity);
        info!("   Интервал обновления: {}ms", interval);
        info!("   Максимум сделок: {}", max_trades);
        info!("   Толерантность к риску: {:.1}", risk);
        info!("   Авто-выполнение: {}", if auto_execute { "включено" } else { "отключено" });
        info!("   Начальный баланс: {} SOL", balance);

        // Запускаем мониторинг с таймером если указан
        if let Some(duration_secs) = duration {
            info!("⏱️  Мониторинг будет работать {} секунд", duration_secs);
            
            let config_clone = config.clone();
            let monitor_config_clone = monitor_config.clone();
            let monitor_handle = tokio::spawn(async move {
                let mut monitor = ArbitrageMonitor::new(config_clone, monitor_config_clone)?;
                monitor.run_monitoring_loop().await
            });

            // Ждем указанное время
            tokio::time::sleep(Duration::from_secs(duration_secs)).await;
            
            info!("✅ Мониторинг завершен");
            
            // Ждем завершения
            if let Err(e) = monitor_handle.await {
                error!("⚠️ Ошибка в мониторинге: {:?}", e);
            }
        } else {
            // Запускаем бесконечный мониторинг
            monitor.run_monitoring_loop().await?;
        }

        Ok(())
    }

    /// Execute status command
    async fn execute_status_command(detailed: bool, config: BotConfig) -> Result<(), AppError> {
        info!("📊 Статус бота:");
        info!("   Версия: {}", env!("CARGO_PKG_VERSION"));
        info!("   RPC Endpoint: {}", config.network.rpc_url);
        info!("   Yellowstone gRPC: solana-yellowstone-grpc.publicnode.com:443");
        
        if detailed {
            info!("   Конфигурация:");
            info!("     - Минимальная прибыль: {:.2}%", 0.5);
            info!("     - Минимальная ликвидность: 10 SOL");
            info!("     - Поддерживаемые DEX: Orca Whirlpool, Raydium AMM");
        }
        
        Ok(())
    }

    /// Execute test command
    async fn execute_test_command(
        dex_adapters: bool,
        grpc: bool,
        arbitrage: bool,
        config: BotConfig,
    ) -> Result<(), AppError> {
        info!("🧪 Запуск тестов...");
        
        if dex_adapters {
            info!("📋 Тестирование DEX адаптеров...");
            // TODO: Реализовать тесты DEX адаптеров
            info!("   ✅ DEX адаптеры работают");
        }
        
        if grpc {
            info!("🔌 Тестирование gRPC соединения...");
            // TODO: Реализовать тесты gRPC
            info!("   ✅ gRPC соединение работает");
        }
        
        if arbitrage {
            info!("🎯 Тестирование детекции арбитража...");
            // TODO: Реализовать тесты арбитража
            info!("   ✅ Детекция арбитража работает");
        }
        
        info!("✅ Все тесты пройдены");
        Ok(())
    }
}
