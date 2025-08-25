use crate::domain::arbitrage::{
    ArbitrageOpportunityDetector, ArbitrageRoute, PriceData,
};
use crate::infrastructure::blockchain::{
    ArbitrageGrpcClient, DexAdapterFactory,
};
use crate::domain::execution::{
    ArbitrageTransactionExecutor, RiskManagementConfig,
};
use crate::shared::errors::AppError;
use crate::shared::types::{Amount, BotConfig, YellowstoneGrpcConfig};
use crate::domain::arbitrage::ProfitCalculation;
use crate::domain::dex::DexType;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{info, warn, error, debug};

/// Конфигурация мониторинга арбитража
#[derive(Debug, Clone)]
pub struct ArbitrageMonitorConfig {
    pub min_profit_threshold: f64,
    pub min_liquidity: f64,
    pub update_interval_ms: u64,
    pub max_concurrent_trades: usize,
    pub risk_tolerance: f64,
    pub enable_auto_execution: bool,
    pub initial_balance_sol: f64, // Начальный баланс в SOL
}

impl Default for ArbitrageMonitorConfig {
    fn default() -> Self {
        Self {
            min_profit_threshold: 0.005, // 0.5%
            min_liquidity: 10.0, // 10 SOL
            update_interval_ms: 100, // 100ms
            max_concurrent_trades: 5,
            risk_tolerance: 0.3,
            enable_auto_execution: true, // Включаем автоматическое исполнение
            initial_balance_sol: 100.0, // 100 SOL начальный баланс
        }
    }
}

/// Статистика мониторинга
#[derive(Debug, Clone)]
pub struct MonitorStats {
    pub start_time: Instant,
    pub transactions_processed: u64,
    pub opportunities_found: u64,
    pub trades_executed: u64,
    pub successful_trades: u64,
    pub failed_trades: u64,
    pub total_profit: f64,
    pub last_update: Instant,
    pub success_rate: f64,
    pub current_balance_sol: f64,
}

impl MonitorStats {
    pub fn new() -> Self {
        Self {
            start_time: Instant::now(),
            transactions_processed: 0,
            opportunities_found: 0,
            trades_executed: 0,
            successful_trades: 0,
            failed_trades: 0,
            total_profit: 0.0,
            last_update: Instant::now(),
            success_rate: 0.0,
            current_balance_sol: 0.0,
        }
    }

    pub fn update(&mut self) {
        self.last_update = Instant::now();
    }

    pub fn get_uptime(&self) -> Duration {
        self.start_time.elapsed()
    }

    pub fn get_opportunities_per_minute(&self) -> f64 {
        let uptime_minutes = self.get_uptime().as_secs_f64() / 60.0;
        if uptime_minutes > 0.0 {
            self.opportunities_found as f64 / uptime_minutes
        } else {
            0.0
        }
    }
}

/// Основной монитор арбитража
pub struct ArbitrageMonitor {
    config: ArbitrageMonitorConfig,
    grpc_client: ArbitrageGrpcClient,
    opportunity_detector: ArbitrageOpportunityDetector,
    dex_factory: DexAdapterFactory,
    rpc_client: Arc<solana_client::rpc_client::RpcClient>,
    arbitrage_executor: ArbitrageTransactionExecutor,
    token_metadata: Arc<crate::infrastructure::blockchain::TokenMetadataService>,
    stats: Arc<RwLock<MonitorStats>>,
    active_trades: Arc<RwLock<HashMap<String, ArbitrageRoute>>>,
    price_cache: Arc<RwLock<HashMap<String, PriceData>>>,
}

impl ArbitrageMonitor {
    /// Создать новый монитор арбитража
    pub fn new(
        config: BotConfig,
        monitor_config: ArbitrageMonitorConfig,
    ) -> Result<Self, AppError> {
        let yellowstone_config = YellowstoneGrpcConfig {
            enabled: true,
            endpoint: "solana-yellowstone-grpc.publicnode.com:443".to_string(), // Временная заглушка
            token: None, // Временная заглушка
            connection_timeout_ms: 10000,
            max_retries: 5,
            dex_programs: vec![
                "whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc".to_string(), // Orca Whirlpool
                "675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8".to_string(), // Raydium V4
            ],
        };

        let grpc_client = ArbitrageGrpcClient::new(yellowstone_config);
        let rpc_client = Arc::new(solana_client::rpc_client::RpcClient::new(config.network.rpc_url.clone()));
        let dex_factory = DexAdapterFactory::new(
            Arc::clone(&rpc_client),
            Arc::new(crate::infrastructure::blockchain::VaultReader::new(config.network.rpc_url.clone())),
            crate::infrastructure::blockchain::OrcaAccountParser::new(Arc::new(crate::infrastructure::blockchain::VaultReader::new(config.network.rpc_url.clone()))),
            crate::infrastructure::blockchain::RaydiumAccountParser::new(),
        );

        let opportunity_detector = ArbitrageOpportunityDetector::new(
            monitor_config.min_profit_threshold,
            crate::shared::types::Amount::new(
                (monitor_config.min_liquidity * 1_000_000_000.0) as u64, 
                9
            ),
            4, // max_route_length
            monitor_config.risk_tolerance,
            Arc::new(crate::infrastructure::blockchain::TokenMetadataService::new(Arc::clone(&rpc_client))),
        );

        // Создаем RealTransactionExecutor
        let real_executor = crate::infrastructure::blockchain::RealTransactionExecutor::new_simple(
            config.network.rpc_url.clone()
        );

        // Создаем конфигурацию риск-менеджмента
        let risk_config = RiskManagementConfig {
            max_position_size: crate::shared::types::Amount::new(
                (monitor_config.max_concurrent_trades as f64 * monitor_config.min_liquidity * 1_000_000_000.0) as u64,
                9
            ),
            max_daily_loss: crate::shared::types::Amount::new(
                (monitor_config.initial_balance_sol * 0.2 * 1_000_000_000.0) as u64, // 20% от начального баланса (было 10%)
                9
            ),
            max_concurrent_trades: monitor_config.max_concurrent_trades,
            min_profit_threshold: monitor_config.min_profit_threshold,
            max_slippage_tolerance: 0.10, // 10% (было 5%)
            max_risk_score: 0.8, // 80% (было 70%)
            min_confidence_score: 0.4, // 40% (было 60%)
            cooldown_period_ms: 500, // 0.5 секунды (было 1 секунда)
        };

        let arbitrage_executor = ArbitrageTransactionExecutor::new(
            real_executor,
            risk_config,
            crate::shared::types::Amount::new(
                (monitor_config.initial_balance_sol * 1_000_000_000.0) as u64,
                9
            ),
        );

        Ok(Self {
            config: monitor_config,
            grpc_client,
            opportunity_detector,
            dex_factory,
            rpc_client: rpc_client.clone(),
            arbitrage_executor,
            token_metadata: Arc::new(crate::infrastructure::blockchain::TokenMetadataService::new(rpc_client)),
            stats: Arc::new(RwLock::new(MonitorStats::new())),
            active_trades: Arc::new(RwLock::new(HashMap::new())),
            price_cache: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// Запустить основной цикл мониторинга
    pub async fn run_monitoring_loop(&mut self) -> Result<(), AppError> {
        info!("🚀 Запуск цикла мониторинга арбитража...");
        
        // Создаем тестовые данные для проверки
        self.create_test_pools().await;
        
        let start_time = std::time::Instant::now();
        let mut last_stats_print = start_time;
        
        loop {
            let cycle_start = std::time::Instant::now();
            
            // Выполняем цикл мониторинга
            self.perform_monitoring_cycle().await?;
            
            // Обновляем статистику
            {
                let mut stats = self.stats.write().await;
                stats.transactions_processed += 1;
                stats.last_update = std::time::Instant::now();
            }
            
            // Показываем статистику каждые 30 секунд
            if cycle_start.duration_since(last_stats_print).as_secs() >= 30 {
                self.print_monitor_stats().await;
                last_stats_print = cycle_start;
            }
            
            // Ждем до следующего цикла
            let cycle_duration = cycle_start.elapsed();
            if cycle_duration < std::time::Duration::from_millis(self.config.update_interval_ms) {
                let sleep_duration = std::time::Duration::from_millis(self.config.update_interval_ms) - cycle_duration;
                tokio::time::sleep(sleep_duration).await;
            }
        }
    }

    /// Выполнить один цикл мониторинга
    async fn perform_monitoring_cycle(&mut self) -> Result<(), AppError> {
        // Ищем арбитражные возможности в тестовых данных
        let test_price_data = vec![
            // SOL -> USDC на Orca
            PriceData {
                token_mint: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(), // USDC
                dex_type: crate::domain::dex::DexType::OrcaWhirlpool,
                pool_id: "orca_sol_usdc_test".to_string(),
                price: 0.00098, // 1 SOL = 98 USDC
                liquidity: crate::shared::types::Amount::new(1000000000000, 9), // 1000 SOL
                volume_24h: Some(1000000.0),
                price_change_24h: Some(0.01),
                timestamp: std::time::Instant::now(),
            },
            // SOL -> USDC на Raydium
            PriceData {
                token_mint: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(), // USDC
                dex_type: crate::domain::dex::DexType::RaydiumAMM,
                pool_id: "raydium_sol_usdc_test".to_string(),
                price: 0.00100, // 1 SOL = 100 USDC
                liquidity: crate::shared::types::Amount::new(800000000000, 9), // 800 SOL
                volume_24h: Some(800000.0),
                price_change_24h: Some(0.02),
                timestamp: std::time::Instant::now(),
            },
        ];
        
        // Ищем арбитражные маршруты
        let routes = self.opportunity_detector.find_arbitrage_routes(&test_price_data).await;
        
        // Обновляем статистику
        {
            let mut stats = self.stats.write().await;
            stats.opportunities_found += routes.len() as u64;
        }
        
        // Обрабатываем найденные возможности
        for route in routes {
            info!("🎯 Обрабатываем арбитражную возможность: {}", route.id);
            
            if let Err(e) = self.process_arbitrage_opportunity(route).await {
                error!("❌ Ошибка обработки арбитража: {}", e);
            }
        }
        
        Ok(())
    }

    /// Получить актуальные данные о ценах
    async fn get_current_price_data(&self) -> Result<Vec<PriceData>, AppError> {
        let mut price_data = Vec::new();
        
        // Получаем цены из кэша
        let cache = self.price_cache.read().await;
        for (_token_mint, price) in cache.iter() {
            price_data.push(price.clone());
        }

        // Если кэш пуст, симулируем некоторые цены для демонстрации
        if price_data.is_empty() {
            price_data = self.generate_demo_price_data().await;
        }

        Ok(price_data)
    }

    /// Генерировать демо данные о ценах
    async fn generate_demo_price_data(&self) -> Vec<PriceData> {
        let mut price_data = Vec::new();
        
        // Симулируем цены SOL на разных DEX
        let sol_prices = vec![
            ("Orca Whirlpool", 100.0, "whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc"),
            ("Raydium V4", 98.5, "675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8"),
        ];

        for (dex_name, price, program_id) in sol_prices {
            let dex_type = if dex_name.contains("Orca") { 
                DexType::OrcaWhirlpool 
            } else { 
                DexType::RaydiumAMM 
            };

            price_data.push(PriceData {
                token_mint: "SOL".to_string(),
                price,
                dex_type,
                pool_id: program_id.to_string(),
                timestamp: Instant::now(),
                liquidity: crate::shared::types::Amount::new(1000000000, 9), // 1 SOL
                volume_24h: Some(1000000.0), // 1M USD
                price_change_24h: Some(0.01), // 1%
            });
        }

        price_data
    }

    /// Создать тестовые данные для проверки арбитража
    async fn create_test_pools(&self) {
        info!("🧪 Создание тестовых пулов для проверки арбитража...");
        
        // SOL -> USDC на Orca
        let sol_usdc_orca = PriceData {
            token_mint: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(), // USDC
            dex_type: crate::domain::dex::DexType::OrcaWhirlpool,
            pool_id: "orca_sol_usdc_test".to_string(),
            price: 0.00098, // 1 SOL = 98 USDC
            liquidity: crate::shared::types::Amount::new(1000000000000, 9), // 1000 SOL
            volume_24h: Some(1000000.0),
            price_change_24h: Some(0.01),
            timestamp: std::time::Instant::now(),
        };
        
        // SOL -> USDC на Raydium
        let sol_usdc_raydium = PriceData {
            token_mint: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(), // USDC
            dex_type: crate::domain::dex::DexType::RaydiumAMM,
            pool_id: "raydium_sol_usdc_test".to_string(),
            price: 0.00100, // 1 SOL = 100 USDC (немного дороже)
            liquidity: crate::shared::types::Amount::new(800000000000, 9), // 800 SOL
            volume_24h: Some(800000.0),
            price_change_24h: Some(0.02),
            timestamp: std::time::Instant::now(),
        };
        
        // USDC -> SOL на Orca (обратная цена)
        let usdc_sol_orca = PriceData {
            token_mint: "111111111111111111111111111111111111111111111111111111111111111111".to_string(), // SOL
            dex_type: crate::domain::dex::DexType::OrcaWhirlpool,
            pool_id: "orca_usdc_sol_test".to_string(),
            price: 1.0 / 0.00098, // 1 USDC = 1/98 SOL
            liquidity: crate::shared::types::Amount::new(98000000000, 6), // 98M USDC
            volume_24h: Some(1000000.0),
            price_change_24h: Some(0.01),
            timestamp: std::time::Instant::now(),
        };
        
        // USDC -> SOL на Raydium (обратная цена)
        let usdc_sol_raydium = PriceData {
            token_mint: "111111111111111111111111111111111111111111111111111111111111111111".to_string(), // SOL
            dex_type: crate::domain::dex::DexType::RaydiumAMM,
            pool_id: "raydium_usdc_sol_test".to_string(),
            price: 1.0 / 0.00100, // 1 USDC = 1/100 SOL
            liquidity: crate::shared::types::Amount::new(100000000000, 6), // 100M USDC
            volume_24h: Some(800000.0),
            price_change_24h: Some(0.02),
            timestamp: std::time::Instant::now(),
        };
        
        // Добавляем в детектор
        self.opportunity_detector.update_price_cache(sol_usdc_orca).await;
        self.opportunity_detector.update_price_cache(sol_usdc_raydium).await;
        self.opportunity_detector.update_price_cache(usdc_sol_orca).await;
        self.opportunity_detector.update_price_cache(usdc_sol_raydium).await;
        
        info!("✅ Создано 4 тестовых пула: SOL↔USDC на Orca и Raydium");
        info!("   💰 Цены: Orca: 1 SOL = 98 USDC, Raydium: 1 SOL = 100 USDC");
        info!("   🎯 Ожидаемый арбитраж: SOL → USDC на Orca → USDC → SOL на Raydium");
    }

    /// Обработать найденную арбитражную возможность
    async fn process_arbitrage_opportunity(&mut self, route: ArbitrageRoute) -> Result<(), AppError> {
        // Формируем понятное описание маршрута
        let route_description = if route.steps.len() >= 2 {
            let mut description = String::new();
            
            for (i, step) in route.steps.iter().enumerate() {
                let token_in_symbol = self.token_metadata.get_token_symbol(&step.token_in.mint.to_string()).await;
                let token_out_symbol = self.token_metadata.get_token_symbol(&step.token_out.mint.to_string()).await;
                let dex_name = match step.dex_type {
                    crate::domain::dex::DexType::OrcaWhirlpool => "Orca Whirlpool",
                    crate::domain::dex::DexType::RaydiumAMM => "Raydium AMM",
                    _ => "Unknown DEX",
                };
                
                if i > 0 {
                    description.push_str(" → ");
                }
                description.push_str(&format!("{}/{} на {}", token_in_symbol, token_out_symbol, dex_name));
            }
            
            // Добавляем возврат к исходному токену
            if let Some(first_step) = route.steps.first() {
                let first_token_symbol = self.token_metadata.get_token_symbol(&first_step.token_in.mint.to_string()).await;
                description.push_str(&format!(" → {}", first_token_symbol));
            }
            
            description
        } else {
            "Недостаточно шагов для отображения".to_string()
        };
        
        info!("\n🎯 Найдена арбитражная возможность!");
        info!("   Маршрут: {}", route_description);
        info!("   Прибыль: {:.2}%", route.profit_percentage * 100.0);
        info!("   Оценка риска: {:.2}", route.risk_score);
        info!("   Уверенность: {:.2}", route.confidence_score);
        info!("   Время выполнения: {:.0} мс", route.execution_time_estimate.as_millis());
        
        // Проверяем, что такая сделка уже не выполняется
        let active_trades = self.active_trades.read().await;
        if active_trades.contains_key(&route.id) {
            warn!("   ⚠️  Сделка уже выполняется, пропускаем");
            return Ok(());
        }
        drop(active_trades);

        // Обновляем статистику
        {
            let mut stats = self.stats.write().await;
            stats.opportunities_found += 1;
        }

        // Если включено автоматическое исполнение, выполняем сделку
        if self.config.enable_auto_execution {
            info!("   🚀 Автоматическое исполнение...");
            
            // Получаем расчет прибыли
            let profit_calc = ProfitCalculation {
                gross_profit: route.expected_profit,
                net_profit: route.expected_profit - (route.total_cost.value as f64 / 1_000_000_000.0),
                gas_cost: route.total_cost.clone(),
                slippage_cost: route.steps.iter().map(|s| s.slippage_estimate).sum(),
                fee_cost: route.steps.iter().map(|s| s.fee.value as f64 / 1_000_000_000.0).sum(),
                profit_margin: route.profit_percentage,
                is_profitable: route.profit_percentage > 0.0,
                roi_percentage: route.profit_percentage,
                break_even_amount: 0.0,
            };

            match self.arbitrage_executor.execute_arbitrage_opportunity(&route, &profit_calc).await {
                Ok(result) => {
                    info!("   ✅ Сделка выполнена успешно!");
                    info!("      ID: {}", route.id);
                    info!("      Успех: {}", result.success);
                    if let Some(signature) = &result.signature {
                        info!("      Подпись: {}", signature);
                    }
                    
                    // Обновляем статистику успешной сделки
                    {
                        let mut stats = self.stats.write().await;
                        stats.trades_executed += 1;
                        stats.successful_trades += 1;
                        stats.total_profit += profit_calc.net_profit;
                    }
                    
                    // Добавляем в активные сделки
                    {
                        let mut active_trades = self.active_trades.write().await;
                        active_trades.insert(route.id.clone(), route.clone());
                    }
                }
                Err(e) => {
                    error!("   ❌ Ошибка выполнения сделки: {}", e);
                    
                    // Обновляем статистику неудачной сделки
                    {
                        let mut stats = self.stats.write().await;
                        stats.failed_trades += 1;
                    }
                }
            }
        } else {
            warn!("   ⏸️  Автоматическое исполнение отключено");
            
            // Добавляем в активные сделки для ручного исполнения
            {
                let mut active_trades = self.active_trades.write().await;
                active_trades.insert(route.id.clone(), route.clone());
            }
        }

        Ok(())
    }

    /// Проверить, можно ли выполнить сделку
    async fn can_execute_trade(&self, route: &ArbitrageRoute) -> bool {
        let active_trades = self.active_trades.read().await;
        
        // Проверяем лимит одновременных сделок
        if active_trades.len() >= self.config.max_concurrent_trades {
            return false;
        }

        // Проверяем, что такая сделка уже не выполняется
        if active_trades.contains_key(&route.id) {
            return false;
        }

        true
    }

    /// Вывести статистику мониторинга
    async fn print_monitor_stats(&self) {
        let stats = self.stats.read().await;
        let (success_rate, roi, net_profit_sol) = self.arbitrage_executor.get_performance_metrics().await;
        
        info!("📊 Статистика мониторинга:");
        info!("   Время работы: {:.1} мин", stats.get_uptime().as_secs_f64() / 60.0);
        
        // Показываем только ненулевые значения
        if stats.transactions_processed > 0 {
            info!("   Обработано транзакций: {}", stats.transactions_processed);
        }
        if stats.opportunities_found > 0 {
            info!("   Найдено возможностей: {}", stats.opportunities_found);
        }
        if stats.trades_executed > 0 {
            info!("   Выполнено сделок: {}", stats.trades_executed);
        }
        if stats.total_profit > 0.0 {
            info!("   Общая прибыль: {:.4} SOL", stats.total_profit);
        }
        if stats.get_opportunities_per_minute() > 0.0 {
            info!("   Возможностей/мин: {:.1}", stats.get_opportunities_per_minute());
        }
        
        info!("   Текущий баланс: {:.4} SOL", stats.current_balance_sol);
        
        if success_rate > 0.0 {
            info!("   Успешность сделок: {:.1}%", success_rate * 100.0);
        }
        if roi > 0.0 {
            info!("   ROI: {:.2}%", roi * 100.0);
        }
        if net_profit_sol > 0.0 {
            info!("   Чистая прибыль сегодня: {:.4} SOL", net_profit_sol);
        }
        
        // Статистика активных сделок
        let active_trades = self.active_trades.read().await;
        if active_trades.len() > 0 {
            info!("   Активных сделок: {}", active_trades.len());
        }
        
        // Статистика кэша цен
        let price_cache = self.price_cache.read().await;
        if price_cache.len() > 0 {
            info!("   Токенов в кэше: {}", price_cache.len());
        }
        
        // Статистика арбитражного исполнителя
        let executor_trades = self.arbitrage_executor.get_active_trades().await;
        if executor_trades.len() > 0 {
            info!("   Сделок в исполнении: {}", executor_trades.len());
        }
        
        // Статистика токен-метаданных
        let token_cache_size = self.token_metadata.get_cache_size().await;
        if token_cache_size > 0 {
            info!("   Токенов в кэше метаданных: {}", token_cache_size);
        }
        
        // Получаем сегодняшнюю статистику
        let today = chrono::Utc::now().date_naive();
        if let Some(daily_stats) = self.arbitrage_executor.get_daily_stats(today).await {
            if daily_stats.total_trades > 0 {
                info!("   📈 Дневная статистика:");
                info!("      Всего сделок: {}", daily_stats.total_trades);
                if daily_stats.successful_trades > 0 {
                    info!("      Успешных: {}", daily_stats.successful_trades);
                }
                if daily_stats.total_profit.value > 0 {
                    info!("      Прибыль: {:.4} SOL", daily_stats.total_profit.value as f64 / 1_000_000_000.0);
                }
                if daily_stats.total_loss.value > 0 {
                    info!("      Убытки: {:.4} SOL", daily_stats.total_loss.value as f64 / 1_000_000_000.0);
                }
                if daily_stats.net_profit.value != 0 {
                    info!("      Чистая прибыль: {:.4} SOL", daily_stats.net_profit.value as f64 / 1_000_000_000.0);
                }
            }
        }
        
        info!("");
    }

    /// Получить статистику мониторинга
    pub async fn get_stats(&self) -> MonitorStats {
        self.stats.read().await.clone()
    }

    /// Получить активные сделки
    pub async fn get_active_trades(&self) -> Vec<ArbitrageRoute> {
        let active_trades = self.active_trades.read().await;
        active_trades.values().cloned().collect()
    }

    /// Остановить мониторинг
    pub async fn stop(&self) -> Result<(), AppError> {
        info!("🛑 Остановка арбитражного монитора...");
        
        // TODO: Корректно закрыть gRPC соединение
        
        info!("✅ Мониторинг остановлен");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared::types::BotConfig;

    #[tokio::test]
    async fn test_arbitrage_monitor_creation() {
        let config = BotConfig::default();
        let monitor_config = ArbitrageMonitorConfig::default();
        
        let monitor = ArbitrageMonitor::new(config, monitor_config);
        assert!(monitor.is_ok());
    }

    #[tokio::test]
    async fn test_monitor_stats() {
        let stats = MonitorStats::new();
        assert_eq!(stats.transactions_processed, 0);
        assert_eq!(stats.opportunities_found, 0);
        assert_eq!(stats.trades_executed, 0);
        assert_eq!(stats.total_profit, 0.0);
    }
}
