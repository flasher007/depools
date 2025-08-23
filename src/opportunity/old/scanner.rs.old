use anyhow::Result;
use solana_sdk::pubkey::Pubkey;
use tokio::time::{Duration, sleep};
use tracing::{info, warn, error};

use crate::exchanges::{DexAdapter, factory, types::{ArbitrageOpportunity, SwapQuote, DexLabel, RiskScore, PoolInfo}};
use crate::report::{ArbitrageReport, ArbitrageDetails, RouteDetails, TokenDetails, FeesBreakdown, SlippageProtection, ExecutionPlan};
use crate::config::Config;
use super::OpportunityScanner;

pub struct CrossDexScanner {
    adapters: Vec<Box<dyn DexAdapter>>,
    config: Config,
}

impl CrossDexScanner {
    pub fn new(config: Config) -> Result<Self> {
        let mut adapters = Vec::new();
        
        // Инициализируем адаптеры для всех поддерживаемых DEX
        adapters.push(factory::create_adapter(DexLabel::RaydiumV4, config.clone())?);
        adapters.push(factory::create_adapter(DexLabel::OrcaWhirlpool, config.clone())?);
        
        Ok(Self {
            adapters,
            config,
        })
    }
    
    /// Логирует отладочную информацию о пулах
    async fn log_pool_debug_info(&self, pool_a: &Pubkey, pool_b: &Pubkey, dex_a: DexLabel, dex_b: DexLabel) {
        info!("🔍 DEBUG: Pool A: {} -> {:?}", pool_a, dex_a);
        info!("🔍 DEBUG: Pool B: {} -> {:?}", pool_b, dex_b);
        
        // Проверяем, что пулы правильно распределены по DEX
        if dex_a == dex_b {
            warn!("⚠️  WARNING: Both pools assigned to same DEX: {:?}", dex_a);
        }
    }

    async fn scan_pool_pair(&self, pool_a: &str, pool_b: &str, amount_in: u64) -> Result<Option<ArbitrageOpportunity>> {
        let pool_a_pubkey: Pubkey = pool_a.parse()?;
        let pool_b_pubkey: Pubkey = pool_b.parse()?;
        
        // Определяем DEX для каждого пула
        let dex_a = self.detect_dex(&pool_a_pubkey).await?;
        let dex_b = self.detect_dex(&pool_b_pubkey).await?;
        
        if dex_a == dex_b {
            // Пропускаем пулы одного DEX
            return Ok(None);
        }
        
        // Получаем pool info для определения правильных направлений swap
        let pool_a_info = self.get_pool_info_cached(&pool_a_pubkey, dex_a).await?;
        let pool_b_info = self.get_pool_info_cached(&pool_b_pubkey, dex_b).await?;
        
        // Логируем информацию о пулах для отладки
        self.log_pool_debug_info(&pool_a_pubkey, &pool_b_pubkey, dex_a, dex_b).await;
        
        // Проверяем, что у пулов одинаковые токены для арбитража
        // Сравниваем по символам токенов, а не по адресам
        let pools_compatible = (pool_a_info.token_a.symbol == pool_b_info.token_a.symbol && 
                               pool_a_info.token_b.symbol == pool_b_info.token_b.symbol) ||
                              (pool_a_info.token_a.symbol == pool_b_info.token_b.symbol && 
                               pool_a_info.token_b.symbol == pool_a_info.token_a.symbol);
        
        if pools_compatible {
            info!("✅ Pools are compatible for arbitrage: {} ↔ {}", 
                  pool_a_info.token_a.symbol, pool_a_info.token_b.symbol);
        } else {
            info!("⚠️  Pools don't share common tokens for arbitrage");
            info!("   Pool A: {} ↔ {}", pool_a_info.token_a.symbol, pool_a_info.token_b.symbol);
            info!("   Pool B: {} ↔ {}", pool_b_info.token_a.symbol, pool_b_info.token_b.symbol);
            return Ok(None);
        }
        
        // Для арбитража используем одинаковое направление swap на обоих пулах
        // например: SOL->USDC на пуле A, SOL->USDC на пуле B
        let base_token = pool_a_info.token_a.mint; // SOL
        
        // Получаем quotes для обоих пулов в одном направлении
        let quote_a = self.get_quote_for_pool(&pool_a_pubkey, amount_in, dex_a).await?;
        let quote_b = self.get_quote_for_pool(&pool_b_pubkey, amount_in, dex_b).await?;
        
        info!("📊 Pool A ({:?}): {} {} → {} {}, fee={} bps", 
              dex_a, amount_in, pool_a_info.token_a.symbol, 
              quote_a.amount_out, pool_a_info.token_b.symbol, quote_a.route.total_fee_bps);
        info!("📊 Pool B ({:?}): {} {} → {} {}, fee={} bps", 
              dex_b, amount_in, pool_b_info.token_a.symbol,
              quote_b.amount_out, pool_b_info.token_b.symbol, quote_b.route.total_fee_bps);
        
        // Для арбитража проверяем разность цен между пулами
        let price_a = quote_a.amount_out as f64 / amount_in as f64;
        let price_b = quote_b.amount_out as f64 / amount_in as f64;
        
        info!("💱 Prices: Pool A = {:.6}, Pool B = {:.6}", price_a, price_b);
        
        // Рассчитываем прибыльность
        let profit_bps = self.calculate_profitability(&quote_a, &quote_b)?;
        
        // Создаем детальный JSON отчёт
        if let Ok(detailed_report) = self.create_detailed_report(
            &quote_a, &quote_b, &pool_a_info, &pool_b_info, profit_bps, 0.0
        ) {
            if let Ok(json_report) = detailed_report.to_json() {
                info!("📄 Detailed JSON Report:\n{}", json_report);
            }
        }
        
        info!("💰 Profit calculation: {} bps ({}%)", profit_bps, profit_bps / 100.0);
        
        if profit_bps <= 0.0 {
            info!("❌ No profitable opportunity (profit <= 0 bps)");
            return Ok(None);
        }
        
        // Создаем арбитражную возможность
        let opportunity = ArbitrageOpportunity {
            id: format!("{}-{}-{}", pool_a, pool_b, chrono::Utc::now().timestamp()),
            timestamp: chrono::Utc::now().timestamp() as u64,
            route_a: quote_a.route.clone(),
            route_b: quote_b.route.clone(),
            profit_bps: profit_bps as i32,
            profit_amount: (amount_in as f64 * profit_bps / 10000.0) as u64,
            risk_score: RiskScore::from_profit_bps(profit_bps as i32),
        };
        
        Ok(Some(opportunity))
    }

    async fn detect_dex(&self, pool_address: &Pubkey) -> Result<DexLabel> {
        // Определяем DEX по конкретным адресам пулов
        let address_str = pool_address.to_string();
        
        match address_str.as_str() {
            // Raydium V4 pools
            "58oQChx4yWmvKdwLLZzBi4ChoCc2fqCUWBkwMihLYQo2" => Ok(DexLabel::RaydiumV4), // SOL-USDC
            "7XawhbbxtsRcQA8KTkHT9f9nc6d69UwqCDh6U5EEbEmX" => Ok(DexLabel::RaydiumV4), // RAY-USDC  
            "DVa7Qmb5ct9RCpaU7UTpSaf3GVMYz17vNVU67XpdCRut" => Ok(DexLabel::RaydiumV4), // SOL-USDT
            
            // Orca Whirlpool pools  
            "HJPjoWUrhoZzkNfRpHuieeFk9WcZWjwy6PBjZ81ngndJ" => Ok(DexLabel::OrcaWhirlpool), // SOL-USDC
            "9vqYJjDUFecLL2xPUC4Rc7hyCtZ6iJ4mDiVZX7aFXoAe" => Ok(DexLabel::OrcaWhirlpool), // SOL-USDT
            "Czfq3xZZDmsdGdUyrNLtRhGc47cXcZtLG4crryfu44zE" => Ok(DexLabel::OrcaWhirlpool), // SOL-USDC (новый)
            
            // Default fallback - try to fetch account and check owner
            _ => {
                // Use first available adapter to check account owner
                for adapter in &self.adapters {
                    // Try to get pool info, which will validate the account owner
                    if let Ok(_pool_info) = adapter.get_pool_info(pool_address).await {
                        return Ok(adapter.get_label());
                    }
                }
                
                // If all fail, return error
                Err(anyhow::anyhow!("Unable to detect DEX for pool: {}", pool_address))
            }
        }
    }

    async fn get_pool_info_cached(&self, pool_address: &Pubkey, dex: DexLabel) -> Result<PoolInfo> {
        let adapter = factory::create_adapter(dex, self.config.clone())?;
        adapter.get_pool_info(pool_address).await
    }

    async fn get_quote_for_pool(&self, pool_address: &Pubkey, amount_in: u64, dex: DexLabel) -> Result<SwapQuote> {
        let adapter = factory::create_adapter(dex, self.config.clone())?;
        
        // Получаем информацию о пуле для определения токенов
        let pool_info = adapter.get_pool_info(pool_address).await?;
        
        // Используем первый токен как входной (в реальности нужно определить направление)
        let token_in = pool_info.token_a.mint;
        
        info!("🪙 Pool {}: token_in={}, token_a={}, token_b={}", 
              pool_address, token_in, pool_info.token_a.symbol, pool_info.token_b.symbol);
        
        adapter.get_swap_quote(pool_address, amount_in, &token_in).await
    }
}

#[async_trait::async_trait]
impl OpportunityScanner for CrossDexScanner {
    fn scan_opportunities(&self, pool_addresses: &[String]) -> Result<Vec<ArbitrageOpportunity>> {
        // В async контексте просто вызываем async версию
        // Это будет обработано в app.rs через .await
        tokio::runtime::Handle::current().block_on(self.scan_opportunities_async(pool_addresses))
    }

    fn calculate_profitability(&self, quote_a: &SwapQuote, quote_b: &SwapQuote) -> Result<f64> {
        // Рассчитываем арбитражную прибыльность между пулами
        let amount_out_a = quote_a.amount_out as f64;
        let amount_out_b = quote_b.amount_out as f64;
        
        if amount_out_a <= 0.0 || amount_out_b <= 0.0 {
            return Ok(0.0);
        }
        
        // Правильный расчет арбитража:
        // 1. SOL → USDC в Pool A
        // 2. USDC → SOL в Pool B
        // 3. Сравниваем итоговое количество SOL
        
        // Комиссии пулов
        let fee_a = (quote_a.amount_in as f64 * quote_a.route.total_fee_bps as f64) / 10000.0;
        let fee_b = (quote_b.amount_in as f64 * quote_b.route.total_fee_bps as f64) / 10000.0;
        
        // Приоритетная комиссия и аренда
        let priority_fee = 1000.0; // lamports
        let rent = 2039280.0; // lamports
        let total_fees = fee_a + fee_b + priority_fee + rent;
        
        // Расчет арбитража:
        // Начинаем с amount_in SOL
        let sol_start = quote_a.amount_in as f64;
        
        // Шаг 1: SOL → USDC в Pool A (после комиссии)
        let usdc_after_pool_a = amount_out_a as f64 - fee_a;
        
        // Шаг 2: USDC → SOL в Pool B (после комиссии)
        // Используем обратную формулу: amount_out_b = k / (reserve_b + usdc_in)
        // Для упрощения используем пропорцию
        let sol_after_pool_b = (usdc_after_pool_a / amount_out_a as f64) * quote_b.amount_in as f64;
        let sol_after_pool_b = sol_after_pool_b - (sol_after_pool_b * quote_b.route.total_fee_bps as f64 / 10000.0);
        
        // Gross profit: сколько SOL мы получили в итоге
        let gross_profit = sol_after_pool_b - sol_start;
        
        // Net profit после вычета всех комиссий
        let net_profit = gross_profit - total_fees;
        
        // Spread между пулами в базисных пунктах
        let spread_bps = if gross_profit > 0.0 {
            (gross_profit / sol_start) * 10000.0
        } else {
            0.0
        };
        
        // Прибыльность в базисных пунктах относительно входной суммы
        let profit_bps = if quote_a.amount_in > 0 {
            (net_profit / quote_a.amount_in as f64) * 10000.0
        } else {
            0.0
        };
        
        info!("📊 Spread: {:.2} bps", spread_bps);
        info!("💸 Fees: Pool A={:.2}, Pool B={:.2}, Priority={:.2}, Rent={:.2}", 
              fee_a, fee_b, priority_fee, rent);
        info!("💰 Gross profit: {:.2}, Net profit: {:.2} lamports", 
              gross_profit, net_profit);
        
        // Создаем детальный JSON отчёт для каждой пары
        // Note: pool_a_info и pool_b_info нужно передать из scan_pool_pair
        // Пока что создаем простой отчёт
        info!("📄 JSON Report: {{\"profitable\": {}, \"spread_bps\": {:.2}, \"profit_bps\": {:.2}, \"gross_profit\": {:.2}, \"net_profit\": {:.2}}}", 
              profit_bps > 0.0, spread_bps, profit_bps, gross_profit, net_profit);
        
        Ok(profit_bps)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl CrossDexScanner {
    pub fn create_detailed_report(
        &self,
        quote_a: &SwapQuote,
        quote_b: &SwapQuote,
        pool_a_info: &PoolInfo,
        pool_b_info: &PoolInfo,
        profit_bps: f64,
        spread_bps: f64,
    ) -> Result<ArbitrageReport> {
        // Создаем детали маршрутов
        let route_a = RouteDetails {
            dex: format!("{:?}", quote_a.dex_label),
            pool_address: quote_a.pool_address.to_string(),
            token_in: TokenDetails {
                mint: quote_a.token_in.to_string(),
                symbol: pool_a_info.token_a.symbol.clone(),
                decimals: pool_a_info.token_a.decimals,
                amount_ui: quote_a.amount_in as f64 / 10_f64.powi(pool_a_info.token_a.decimals as i32),
            },
            token_out: TokenDetails {
                mint: quote_a.token_out.to_string(),
                symbol: pool_a_info.token_b.symbol.clone(),
                decimals: pool_a_info.token_b.decimals,
                amount_ui: quote_a.amount_out as f64 / 10_f64.powi(pool_a_info.token_b.decimals as i32),
            },
            amount_in: quote_a.amount_in,
            amount_out: quote_a.amount_out,
            price: quote_a.amount_out as f64 / quote_a.amount_in as f64,
            fee_bps: quote_a.route.total_fee_bps,
            fee_amount: quote_a.fee_amount,
        };

        let route_b = RouteDetails {
            dex: format!("{:?}", quote_b.dex_label),
            pool_address: quote_b.pool_address.to_string(),
            token_in: TokenDetails {
                mint: quote_b.token_in.to_string(),
                symbol: pool_b_info.token_a.symbol.clone(),
                decimals: pool_b_info.token_a.decimals,
                amount_ui: quote_b.amount_in as f64 / 10_f64.powi(pool_b_info.token_a.decimals as i32),
            },
            token_out: TokenDetails {
                mint: quote_b.token_out.to_string(),
                symbol: pool_b_info.token_b.symbol.clone(),
                decimals: pool_b_info.token_b.decimals,
                amount_ui: quote_b.amount_out as f64 / 10_f64.powi(pool_b_info.token_b.decimals as i32),
            },
            amount_in: quote_b.amount_in,
            amount_out: quote_b.amount_out,
            price: quote_b.amount_out as f64 / quote_b.amount_in as f64,
            fee_bps: quote_b.route.total_fee_bps,
            fee_amount: quote_b.fee_amount,
        };

        // Разбивка комиссий
        let priority_fee = 1000u64;
        let rent = 2039280u64;
        let fees_breakdown = FeesBreakdown {
            pool_a_fee: quote_a.fee_amount,
            pool_b_fee: quote_b.fee_amount,
            priority_fee,
            rent,
            total_fees: quote_a.fee_amount + quote_b.fee_amount + priority_fee + rent,
        };

        // Защита от slippage
        let slippage_protection = SlippageProtection {
            slippage_bps: 100, // 1%
            min_amount_out_a: quote_a.min_amount_out,
            min_amount_out_b: quote_b.min_amount_out,
            slippage_buffer: (quote_a.amount_out + quote_b.amount_out) / 100, // 1% buffer
        };

        // План исполнения
        let execution_plan = ExecutionPlan {
            instructions_count: 2, // 2 swap instructions
            estimated_compute_units: 200000, // Приблизительная оценка
            priority_fee_microlamports: priority_fee,
            simulate_only: true,
            recommended_action: if profit_bps > 0.0 {
                "EXECUTE".to_string()
            } else {
                "SKIP".to_string()
            },
            risk_assessment: if profit_bps > 100.0 {
                "LOW".to_string()
            } else if profit_bps > 0.0 {
                "MEDIUM".to_string()
            } else {
                "HIGH".to_string()
            },
        };

        let arbitrage_details = ArbitrageDetails {
            route_a,
            route_b,
            fees_breakdown,
            slippage_protection,
            execution_plan,
        };

        let report = ArbitrageReport::new(
            profit_bps > 0.0,
            spread_bps,
            profit_bps,
            quote_a.min_amount_out as f64,
            vec![pool_a_info.clone(), pool_b_info.clone()],
            arbitrage_details,
        );

        Ok(report)
    }
}

impl CrossDexScanner {
    pub async fn scan_opportunities_async(&self, pool_addresses: &[String]) -> Result<Vec<ArbitrageOpportunity>> {
        let mut opportunities = Vec::new();
        
        info!("Scanning {} pools for arbitrage opportunities...", pool_addresses.len());
        
        // Простой подход - последовательное сканирование пар
        for i in 0..pool_addresses.len() {
            for j in i + 1..pool_addresses.len() {
                let pool_a = &pool_addresses[i];
                let pool_b = &pool_addresses[j];
                let amount_in = 1000000; // 1 SOL в lamports
                
                info!("Checking pair: {} vs {}", pool_a, pool_b);
                
                match self.scan_pool_pair(pool_a, pool_b, amount_in).await {
                    Ok(Some(opportunity)) => {
                        info!("✅ Found opportunity: profit {} bps", opportunity.profit_bps);
                        opportunities.push(opportunity);
                    }
                    Ok(None) => {
                        info!("No opportunity found for this pair");
                    }
                    Err(e) => {
                        error!("Error scanning pair {}/{}: {}", pool_a, pool_b, e);
                    }
                }
                
                // Небольшая задержка между сканированиями
                sleep(Duration::from_millis(100)).await;
            }
        }
        
        // Сортируем по прибыльности (убывающая)
        opportunities.sort_by(|a, b| b.profit_bps.cmp(&a.profit_bps));
        
        info!("Found {} arbitrage opportunities", opportunities.len());
        Ok(opportunities)
    }
}
