use tracing::{info, error};
use anyhow::Result;
use solana_sdk::pubkey::Pubkey;
use std::time::Duration;
use std::any::Any;
use tokio::time::sleep;
use async_trait::async_trait;
use reqwest;
use serde_json;
use crate::exchanges::{DexAdapter, types::{ArbitrageOpportunity, SwapQuote, DexLabel, RiskScore, PoolInfo, SwapRoute, SwapHop, PnlBreakdown}};
use crate::exchanges::utils::{lamports_to_sol, lamports_to_usdc, format_sol, format_usdc, format_large_number};
use crate::opportunity::scanner::{OpportunityScanner, AsyncOpportunityScanner};
use crate::exchanges;
use crate::math::calculate_pnl_breakdown;
use crate::report::{ArbitrageReport, ArbitrageDetails, RouteDetails, TokenDetails, FeesBreakdown, SlippageProtection, ExecutionPlan};

pub struct CrossDexScanner {
    adapters: Vec<Box<dyn DexAdapter>>,
    config: crate::config::Config,
}

impl CrossDexScanner {
    pub fn new(config: crate::config::Config) -> Result<Self> {
        let mut adapters = Vec::new();
        
        info!("🔧 Creating Raydium V4 adapter...");
        let raydium_adapter = exchanges::create_adapter(DexLabel::RaydiumV4, config.clone())?;
        info!("✅ Raydium V4 adapter created successfully");
        adapters.push(raydium_adapter);
        
        info!("🔧 Creating Orca Whirlpool adapter...");
        let orca_adapter = exchanges::create_adapter(DexLabel::OrcaWhirlpool, config.clone())?;
        info!("✅ Orca Whirlpool adapter created successfully");
        adapters.push(orca_adapter);
        
        info!("🎯 Created {} adapters", adapters.len());
        
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
            error!("⚠️  WARNING: Both pools assigned to same DEX: {:?}", dex_a);
        }
    }

    async fn scan_pool_pair(
        &self,
        pool_a: &str,
        pool_b: &str,
        amount_in: u64,
        spread_threshold_bps: u32,
        slippage_bps: u32,
        priority_fee: u64,
    ) -> Result<Option<ArbitrageOpportunity>> {
        info!("🔍 Starting scan_pool_pair for {} vs {}", pool_a, pool_b);
        
        let pool_a_pubkey: Pubkey = pool_a.parse()?;
        let pool_b_pubkey: Pubkey = pool_b.parse()?;
        
        info!("🔍 Parsed pubkeys: {} and {}", pool_a_pubkey, pool_b_pubkey);
        
        // Определяем DEX для каждого пула
        let dex_a = self.detect_dex(&pool_a_pubkey).await?;
        let dex_b = self.detect_dex(&pool_b_pubkey).await?;
        
        info!("🔍 Detected DEX: Pool A -> {:?}, Pool B -> {:?}", dex_a, dex_b);
        
        if dex_a == dex_b {
            // Пропускаем пулы одного DEX
            info!("⚠️  Skipping pools from same DEX: {:?}", dex_a);
            return Ok(None);
        }
        
        // Получаем pool info для определения правильных направлений swap
        let pool_a_info = self.get_pool_info_cached(&pool_a_pubkey, dex_a).await?;
        let pool_b_info = self.get_pool_info_cached(&pool_b_pubkey, dex_b).await?;
        
        // Логируем информацию о пулах для отладки
        self.log_pool_debug_info(&pool_a_pubkey, &pool_b_pubkey, dex_a, dex_b).await;
        
        // Проверяем, что у пулов одинаковые токены для арбитража
        // Сравниваем по символам токенов, а не по адресам
        // Нормализуем SOL/WSOL как один токен
        fn normalize_symbol(symbol: &str) -> String {
            match symbol {
                "SOL" | "WSOL" => "SOL".to_string(),
                other => other.to_string(),
            }
        }
        
        let pool_a_token_a_norm = normalize_symbol(&pool_a_info.token_a.symbol);
        let pool_a_token_b_norm = normalize_symbol(&pool_a_info.token_b.symbol);
        let pool_b_token_a_norm = normalize_symbol(&pool_b_info.token_a.symbol);
        let pool_b_token_b_norm = normalize_symbol(&pool_b_info.token_b.symbol);
        
        let pools_compatible = (pool_a_token_a_norm == pool_b_token_a_norm && 
                               pool_a_token_b_norm == pool_b_token_b_norm) ||
                              (pool_a_token_a_norm == pool_b_token_b_norm && 
                               pool_a_token_b_norm == pool_b_token_a_norm);
        
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
        
        // Для арбитража: Pool A: SOL → USDC, Pool B: USDC → SOL
        let quote_a = self.get_quote_for_pool(&pool_a_pubkey, amount_in, dex_a).await?;
        let amount_usdc = quote_a.amount_out; // Выход из Pool A станет входом для Pool B
        
        // Для Pool B используем quote в обратном направлении (USDC → SOL)
        let quote_b = self.get_quote_for_pool_reverse(&pool_b_pubkey, amount_usdc, dex_b).await?;
        
        info!("📊 Pool A ({:?}): {} {} → {} {}, fee={} bps", 
              dex_a, amount_in, pool_a_info.token_a.symbol, 
              quote_a.amount_out, pool_a_info.token_b.symbol, quote_a.route.total_fee_bps);
        info!("📊 Pool B ({:?}): {} {} → {} {}, fee={} bps", 
              dex_b, amount_usdc, pool_b_info.token_b.symbol,
              quote_b.amount_out, pool_b_info.token_a.symbol, quote_b.route.total_fee_bps);
        
        // Для арбитража считаем итоговую прибыль
        let sol_out = quote_b.amount_out; // Сколько SOL получим в итоге
        let profit_lamports = if sol_out > amount_in { sol_out - amount_in } else { 0 };
        
        // Convert to readable units
        let amount_in_sol = lamports_to_sol(amount_in);
        let amount_usdc_formatted = lamports_to_usdc(amount_usdc);
        let sol_out_formatted = lamports_to_sol(sol_out);
        let profit_sol = lamports_to_sol(profit_lamports);
        
        info!("💱 Arbitrage: {} → {} → {}, profit: {}", 
              format_sol(amount_in_sol), format_usdc(amount_usdc_formatted), 
              format_sol(sol_out_formatted), format_sol(profit_sol));
        
        // 🔍 ДЕТАЛЬНЫЙ АНАЛИЗ ЦЕН И СПРЕДОВ
        info!("🔍 === ДЕТАЛЬНЫЙ АНАЛИЗ АРБИТРАЖА ===");
        
        // Цены в пулах (без учета комиссий)
        // Используем готовые цены из API вместо расчета из резервов
        let price_a_sol_usdc = self.get_pool_price_from_api(&pool_a_pubkey, dex_a).await?;
        let price_b_sol_usdc = self.get_pool_price_from_api(&pool_b_pubkey, dex_b).await?;
        
        info!("💰 Цены в пулах:");
        info!("  Raydium V4: 1 SOL = {:.2} USDC (reserves: {:.0} SOL ↔ {:.0} USDC)", 
              price_a_sol_usdc, 
              lamports_to_sol(pool_a_info.reserves.token_a_reserve),
              lamports_to_usdc(pool_a_info.reserves.token_b_reserve));
        info!("  Orca Whirlpool: 1 SOL = {:.2} USDC (reserves: {:.0} SOL ↔ {:.0} USDC)", 
              price_b_sol_usdc,
              lamports_to_sol(pool_b_info.reserves.token_a_reserve),
              lamports_to_usdc(pool_b_info.reserves.token_b_reserve));
        
        // Спред между пулами
        let spread_bps = if price_a_sol_usdc > price_b_sol_usdc {
            ((price_a_sol_usdc - price_b_sol_usdc) / price_b_sol_usdc * 10000.0) as u32
        } else {
            ((price_b_sol_usdc - price_a_sol_usdc) / price_a_sol_usdc * 10000.0) as u32
        };
        
        info!("📊 Спред между пулами: {} bps ({:.4}%)", spread_bps, spread_bps as f64 / 100.0);
        
        // Эффективные цены с учетом комиссий
        let effective_price_a = price_a_sol_usdc * (1.0 + pool_a_info.fees.trade_fee_bps as f64 / 10000.0);
        let effective_price_b = price_b_sol_usdc * (1.0 + pool_b_info.fees.trade_fee_bps as f64 / 10000.0);
        
        info!("💸 Эффективные цены с комиссиями:");
        info!("  Raydium V4: 1 SOL = {:.6} USDC (fee: {} bps)", effective_price_a, pool_a_info.fees.trade_fee_bps);
        info!("  Orca Whirlpool: 1 SOL = {:.6} USDC (fee: {} bps)", effective_price_b, pool_b_info.fees.trade_fee_bps);
        
        // Анализ AMM расчетов
        info!("🧮 Анализ AMM расчетов:");
        info!("  Raydium V4: {} SOL → {} USDC (fee: {} bps)", 
              lamports_to_sol(amount_in), lamports_to_usdc(quote_a.amount_out), quote_a.route.total_fee_bps);
        info!("  Orca Whirlpool: {} USDC → {} SOL (fee: {} bps)", 
              lamports_to_usdc(amount_usdc), lamports_to_sol(quote_b.amount_out), quote_b.route.total_fee_bps);
        
        // Рассчитываем прибыльность
        let profit_bps = self.calculate_profitability(&quote_a, &quote_b)?;
        
        // Проверка реалистичности прибыли
        let profit_percentage = (profit_lamports as f64 / amount_in as f64) * 100.0;
        if profit_percentage > 10.0 {
            info!("⚠️  ВНИМАНИЕ: Очень высокая прибыль: {:.2}% ({:.2} bps)", profit_percentage, profit_bps);
            info!("   Возможные причины:");
            info!("   - Временный дисбаланс в пулах");
            info!("   - Неправильная AMM формула для concentrated liquidity");
            info!("   - Устаревшие данные резервов");
            info!("   - Ошибка в расчете комиссий");
        }
        
        info!("🔍 === КОНЕЦ АНАЛИЗА ===");
        
        // Calculate PnL breakdown
        let pnl_breakdown = calculate_pnl_breakdown(&quote_a, &quote_b, priority_fee, slippage_bps);
        
        // Calculate minimum output amounts with slippage protection
        let min_out_a = crate::math::calculate_min_out(quote_a.amount_out, slippage_bps);
        let min_out_b = crate::math::calculate_min_out(quote_b.amount_out, slippage_bps);
        
        // Детальный анализ PnL будет показан только для прибыльных сделок
        
        // Gross profit (до комиссий)
        let gross_profit_sol = lamports_to_sol(pnl_breakdown.gross_profit);
        let gross_profit_usdc = lamports_to_usdc(pnl_breakdown.gross_profit);
        info!("📈 Gross Profit: {} SOL ({} USDC)", format_sol(gross_profit_sol), format_usdc(gross_profit_usdc));
        
        // Комиссии пулов (показываем только если значимые)
        let pool_a_fee_usdc = lamports_to_usdc(quote_a.fee_amount);
        let pool_b_fee_sol = lamports_to_sol(quote_b.fee_amount);
        
        if pool_a_fee_usdc > 0.0 || pool_b_fee_sol > 0.0 {
            info!("🏦 Pool Fees:");
            if pool_a_fee_usdc > 0.0 {
                info!("   Raydium V4: {} USDC ({} bps)", format_usdc(pool_a_fee_usdc), quote_a.route.total_fee_bps);
            }
            if pool_b_fee_sol > 0.0 {
                info!("   Orca Whirlpool: {} SOL ({} bps)", format_sol(pool_b_fee_sol), quote_b.route.total_fee_bps);
            }
        }
        
        // Сетевые комиссии (показываем только если значимые)
        let priority_fee_sol = lamports_to_sol(pnl_breakdown.priority_fee);
        let rent_fee_sol = lamports_to_sol(pnl_breakdown.rent_fee);
        
        if priority_fee_sol > 0.0 || rent_fee_sol > 0.0 {
            info!("🌐 Network Fees:");
            if priority_fee_sol > 0.0 {
                info!("   Priority Fee: {} SOL", format_sol(priority_fee_sol));
            }
            if rent_fee_sol > 0.0 {
                info!("   Rent Fee: {} SOL", format_sol(rent_fee_sol));
            }
        }
        
        // Net profit (после всех комиссий)
        let net_profit_sol = lamports_to_sol(pnl_breakdown.net_profit);
        let net_profit_usdc = lamports_to_usdc(pnl_breakdown.net_profit);
        info!("💵 Net Profit: {} SOL ({} USDC)", format_sol(net_profit_sol), format_usdc(net_profit_usdc));
        
        // ROI и прибыльность (показываем только если есть прибыль)
        if pnl_breakdown.net_profit > 0 {
            let roi_percentage = (pnl_breakdown.net_profit as f64 / amount_in as f64) * 100.0;
            let roi_bps = (pnl_breakdown.net_profit as f64 / amount_in as f64) * 10000.0;
            info!("📊 ROI: {:.4}% ({:.2} bps)", roi_percentage, roi_bps);
        }
        
        // Статус прибыльности
        if pnl_breakdown.is_profitable {
            info!("✅ Arbitrage is PROFITABLE");
        } else {
            info!("❌ Arbitrage is NOT profitable");
        }
        
        info!("💰 === КОНЕЦ АНАЛИЗА PnL ===");
        
        // Check if arbitrage is profitable
        if profit_lamports == 0 || profit_bps < spread_threshold_bps as f64 {
            let profit_sol = lamports_to_sol(profit_lamports);
            info!("❌ Opportunity not profitable: profit = {} ({:.2} bps)", format_sol(profit_sol), profit_bps);
            
                    // Краткий PnL для неприбыльных сделок
        if pnl_breakdown.gross_profit > 0 {
            info!("💰 PnL Summary: Gross: {} SOL, Net: {} SOL", 
                  format_sol(lamports_to_sol(pnl_breakdown.gross_profit)),
                  format_sol(lamports_to_sol(pnl_breakdown.net_profit)));
        }
            
            return Ok(None);
        }

        // 🔍 ДЕТАЛЬНЫЙ АНАЛИЗ PnL (только для прибыльных сделок)
        info!("💰 === ДЕТАЛЬНЫЙ АНАЛИЗ PnL ===");
        
        // Gross profit (до комиссий)
        let gross_profit_sol = lamports_to_sol(pnl_breakdown.gross_profit);
        let gross_profit_usdc = lamports_to_usdc(pnl_breakdown.gross_profit);
        info!("📈 Gross Profit: {} SOL ({} USDC)", format_sol(gross_profit_sol), format_usdc(gross_profit_usdc));
        
        // Комиссии пулов (показываем только если значимые)
        let pool_a_fee_usdc = lamports_to_usdc(quote_a.fee_amount);
        let pool_b_fee_sol = lamports_to_usdc(quote_b.fee_amount);
        
        if pool_a_fee_usdc > 0.0 || pool_b_fee_sol > 0.0 {
            info!("🏦 Pool Fees:");
            if pool_a_fee_usdc > 0.0 {
                info!("   Raydium V4: {} USDC ({} bps)", format_usdc(pool_a_fee_usdc), quote_a.route.total_fee_bps);
            }
            if pool_b_fee_sol > 0.0 {
                info!("   Orca Whirlpool: {} SOL ({} bps)", format_sol(pool_b_fee_sol), quote_b.route.total_fee_bps);
            }
        }
        
        // Сетевые комиссии (показываем только если значимые)
        let priority_fee_sol = lamports_to_sol(pnl_breakdown.priority_fee);
        let rent_fee_sol = lamports_to_sol(pnl_breakdown.rent_fee);
        
        if priority_fee_sol > 0.0 || rent_fee_sol > 0.0 {
            info!("🌐 Network Fees:");
            if priority_fee_sol > 0.0 {
                info!("   Priority Fee: {} SOL", format_sol(priority_fee_sol));
            }
            if rent_fee_sol > 0.0 {
                info!("   Rent Fee: {} SOL", format_sol(rent_fee_sol));
            }
        }
        
        // Net profit (после всех комиссий)
        let net_profit_sol = lamports_to_sol(pnl_breakdown.net_profit);
        let net_profit_usdc = lamports_to_usdc(pnl_breakdown.net_profit);
        info!("💵 Net Profit: {} SOL ({} USDC)", format_sol(net_profit_sol), format_usdc(net_profit_usdc));
        
        // ROI и прибыльность (показываем только если есть прибыль)
        if pnl_breakdown.net_profit > 0 {
            let roi_percentage = (pnl_breakdown.net_profit as f64 / amount_in as f64) * 100.0;
            let roi_bps = (pnl_breakdown.net_profit as f64 / amount_in as f64) * 10000.0;
            info!("📊 ROI: {:.4}% ({:.2} bps)", roi_percentage, roi_bps);
        }
        
        // Статус прибыльности
        if pnl_breakdown.is_profitable {
            info!("✅ Arbitrage is PROFITABLE");
        } else {
            info!("❌ Arbitrage is NOT profitable");
        }
        
        info!("💰 === КОНЕЦ АНАЛИЗА PnL ===");
        
        let opportunity = ArbitrageOpportunity {
            id: format!("{}-{}", pool_a, pool_b),
            timestamp: chrono::Utc::now().timestamp() as u64,
            route_a: SwapRoute {
                hops: vec![SwapHop {
                    pool_address: pool_a_pubkey,
                    dex_label: dex_a,
                    token_in: pool_a_info.token_a.mint,
                    token_out: pool_a_info.token_b.mint,
                    amount_in,
                    amount_out: quote_a.amount_out,
                    fee_bps: quote_a.route.total_fee_bps,
                }],
                total_fee_bps: quote_a.route.total_fee_bps,
            },
            route_b: SwapRoute {
                hops: vec![SwapHop {
                    pool_address: pool_b_pubkey,
                    dex_label: dex_b,
                    token_in: pool_b_info.token_b.mint, // USDC (token_b for reverse direction)
                    token_out: pool_b_info.token_a.mint, // WSOL (token_a for reverse direction)
                    amount_in: quote_a.amount_out, // Use the USDC amount from first swap
                    amount_out: quote_b.amount_out,
                    fee_bps: quote_b.route.total_fee_bps,
                }],
                total_fee_bps: quote_b.route.total_fee_bps,
            },
            profit_bps: profit_bps as i32,
            profit_amount: pnl_breakdown.net_profit,
            risk_score: RiskScore::Low, // Упрощенно
            pnl_breakdown,
            min_out_a,
            min_out_b,
        };
        
        // 🎯 ФИНАЛЬНЫЙ РЕЗУЛЬТАТ АРБИТРАЖА
        info!("🎯 === ФИНАЛЬНЫЙ РЕЗУЛЬТАТ ===");
        info!("📊 Profit: {} SOL ({:.2} bps)", format_sol(lamports_to_sol(opportunity.profit_amount)), opportunity.profit_bps);
        info!("💰 PnL Summary:");
        info!("   Gross: {} SOL", format_sol(lamports_to_sol(opportunity.pnl_breakdown.gross_profit)));
        info!("   Net: {} SOL", format_sol(lamports_to_sol(opportunity.pnl_breakdown.net_profit)));
        info!("   Risk Score: {:?}", opportunity.risk_score);
        info!("🎯 === КОНЕЦ РЕЗУЛЬТАТА ===");
        
        // Генерируем JSON-отчет
        let json_report = self.generate_json_report(&opportunity, &pool_a_info, &pool_b_info, &quote_a, &quote_b).await?;
        info!("📄 JSON Report:");
        info!("{}", json_report.to_json()?);
        
        Ok(Some(opportunity))
    }

    async fn detect_dex(&self, pool_address: &Pubkey) -> Result<DexLabel> {
        // Умная логика определения DEX по адресу пула
        let address_str = pool_address.to_string();
        
        info!("🔍 Detecting DEX for address: {} (length: {})", address_str, address_str.len());
        
        // Используем известные адреса пулов для определения DEX
        // В реальной реализации здесь будет проверка через RPC или базу данных
        
        let dex_label = match address_str.as_str() {
            // Raydium V4 пулы (известные адреса)
            "58oQChx4yWmvKdwLLZzBi4ChoCc2fqCUWBkwMihLYQo2" => DexLabel::RaydiumV4,
            
            // Orca Whirlpool пулы (известные адреса)
            "Czfq3xZZDmsdGdUyrNLtRhGc47cXcZtLG4crryfu44zE" => DexLabel::OrcaWhirlpool,
            "HJPjoWUrhoZzkNfRpHuieeFk9WcZWjwy6PBjZ81ngndJ" => DexLabel::OrcaWhirlpool,
            "7qbRF6YsyGuLUVs6Y1q64bdVrfe4ZcUUz1JRdoVNUJnm" => DexLabel::OrcaWhirlpool,
            "83v8iPyZihDEjDdY8RdZddyZNyUtXngz69Lgo9Kt5d6d" => DexLabel::OrcaWhirlpool,
            "21gTfxAnhUDjJGZJDkTXctGFKT8TeiXx6pN1CEg9K1uW" => DexLabel::OrcaWhirlpool,
            "DFVTutNYXD8z4T5cRdgpso1G3sZqQvMHWpW2N99E4DvE" => DexLabel::OrcaWhirlpool,
            "7xuPLn8Bun4ZGHeD95xYLnPKReKtSe7zfVRzRJWJZVZW" => DexLabel::OrcaWhirlpool,
            "6d4UYGAEs4Akq6py8Vb3Qv5PvMkecPLS1Z9bBCcip2R7" => DexLabel::OrcaWhirlpool,
            "CWjGo5jkduSW5LN5rxgiQ18vGnJJEKWPCXkpJGxKSQTH" => DexLabel::OrcaWhirlpool,
            
            // Fallback логика для неизвестных адресов
            _ => {
                if address_str.len() > 40 {
                    DexLabel::RaydiumV4
                } else {
                    DexLabel::OrcaWhirlpool
                }
            }
        };
        
        info!("🔍 Determined DEX: {:?} for address: {}", dex_label, address_str);
        Ok(dex_label)
    }

    async fn get_pool_info_cached(&self, pool_address: &Pubkey, dex_label: DexLabel) -> Result<PoolInfo> {
        // Получаем информацию о пуле через соответствующий адаптер
        for adapter in &self.adapters {
            // Проверяем тип адаптера по его реализации
            if let Some(_raydium_adapter) = adapter.as_any().downcast_ref::<crate::exchanges::raydium_v4::adapter::RaydiumV4Adapter>() {
                if dex_label == DexLabel::RaydiumV4 {
                    return adapter.get_pool_info(pool_address).await;
                }
            } else if let Some(_orca_adapter) = adapter.as_any().downcast_ref::<crate::exchanges::orca_whirlpool::adapter::OrcaWhirlpoolAdapter>() {
                if dex_label == DexLabel::OrcaWhirlpool {
                    return adapter.get_pool_info(pool_address).await;
                }
            }
        }
        
        Err(anyhow::anyhow!("No suitable adapter found for DEX: {:?}", dex_label))
    }

    async fn get_quote_for_pool(&self, pool_address: &Pubkey, amount_in: u64, dex_label: DexLabel) -> Result<SwapQuote> {
        // Получаем quote через соответствующий адаптер
        for adapter in &self.adapters {
            // Проверяем тип адаптера по его реализации
            if let Some(_raydium_adapter) = adapter.as_any().downcast_ref::<crate::exchanges::raydium_v4::adapter::RaydiumV4Adapter>() {
                if dex_label == DexLabel::RaydiumV4 {
                    return adapter.get_swap_quote(pool_address, amount_in).await;
                }
            } else if let Some(_orca_adapter) = adapter.as_any().downcast_ref::<crate::exchanges::orca_whirlpool::adapter::OrcaWhirlpoolAdapter>() {
                if dex_label == DexLabel::OrcaWhirlpool {
                    return adapter.get_swap_quote(pool_address, amount_in).await;
                }
            }
        }
        
        Err(anyhow::anyhow!("No suitable adapter found for DEX: {:?}", dex_label))
    }
    
    async fn get_quote_for_pool_reverse(&self, pool_address: &Pubkey, amount_in: u64, dex_label: DexLabel) -> Result<SwapQuote> {
        // Получаем quote в обратном направлении (USDC → SOL)
        for adapter in &self.adapters {
            // Проверяем тип адаптера по его реализации
            if let Some(_raydium_adapter) = adapter.as_any().downcast_ref::<crate::exchanges::raydium_v4::adapter::RaydiumV4Adapter>() {
                if dex_label == DexLabel::RaydiumV4 {
                    return adapter.get_swap_quote(pool_address, amount_in).await;
                }
            } else if let Some(_orca_adapter) = adapter.as_any().downcast_ref::<crate::exchanges::orca_whirlpool::adapter::OrcaWhirlpoolAdapter>() {
                if dex_label == DexLabel::OrcaWhirlpool {
                    return adapter.get_swap_quote(pool_address, amount_in).await;
                }
            }
        }
        
        Err(anyhow::anyhow!("No suitable adapter found for DEX: {:?}", dex_label))
    }

    /// Получить цену пула из API
    async fn get_pool_price_from_api(&self, pool_address: &Pubkey, dex_label: DexLabel) -> Result<f64> {
        match dex_label {
            DexLabel::RaydiumV4 => {
                // Получаем цену через Raydium API
                let url = format!("https://api-v3.raydium.io/pools/info/ids?ids={}", pool_address);
                let response = reqwest::get(&url).await?;
                let data: serde_json::Value = response.json().await?;
                
                if let Some(pools) = data.get("data").and_then(|v| v.as_array()) {
                    if let Some(pool_data) = pools.first() {
                        if let Some(price) = pool_data.get("price").and_then(|v| v.as_f64()) {
                            return Ok(price);
                        }
                    }
                }
                
                Err(anyhow::anyhow!("Failed to get price from Raydium API"))
            }
            DexLabel::OrcaWhirlpool => {
                // Получаем цену через Orca API
                let url = format!("https://api.orca.so/v2/solana/pools/{}", pool_address);
                let response = reqwest::get(&url).await?;
                let data: serde_json::Value = response.json().await?;
                
                if let Some(pool_data) = data.get("data") {
                    if let Some(price_str) = pool_data.get("price").and_then(|v| v.as_str()) {
                        if let Ok(price) = price_str.parse::<f64>() {
                            return Ok(price);
                        }
                    }
                }
                
                Err(anyhow::anyhow!("Failed to get price from Orca API"))
            }
            _ => Err(anyhow::anyhow!("Unsupported DEX: {:?}", dex_label))
        }
    }
}

#[async_trait]
impl AsyncOpportunityScanner for CrossDexScanner {
    async fn scan_opportunities_async(
        &self,
        pool_addresses: &[String],
        amount_in: u64,
        spread_threshold_bps: u32,
        slippage_bps: u32,
        priority_fee: u64,
    ) -> Result<Vec<ArbitrageOpportunity>> {
        info!("🔍 Starting async scan of {} pools", pool_addresses.len());
        info!("🔧 Scan parameters: amount_in={}, spread_threshold={}, slippage={}, priority_fee={}", 
              amount_in, spread_threshold_bps, slippage_bps, priority_fee);
        
        let mut opportunities = Vec::new();
        
        // Сканируем все возможные пары пулов
        for i in 0..pool_addresses.len() {
            for j in (i + 1)..pool_addresses.len() {
                let pool_a = &pool_addresses[i];
                let pool_b = &pool_addresses[j];
                
                info!("🔍 Scanning pair: {} vs {}", pool_a, pool_b);
                
                if let Some(opportunity) = self.scan_pool_pair(
                    pool_a, 
                    pool_b, 
                    amount_in, 
                    spread_threshold_bps, 
                    slippage_bps, 
                    priority_fee
                ).await? {
                    info!("💰 Found opportunity: {:?}", opportunity);
                    opportunities.push(opportunity);
                } else {
                    info!("❌ No opportunity found for this pair");
                }
                
                // Небольшая задержка между запросами
                sleep(Duration::from_millis(100)).await;
            }
        }
        
        info!("🎯 Found {} arbitrage opportunities", opportunities.len());
        Ok(opportunities)
    }
}

#[async_trait]
impl OpportunityScanner for CrossDexScanner {
    fn scan_opportunities(&self, _pool_addresses: &[String]) -> Result<Vec<ArbitrageOpportunity>> {
        // Синхронная версия (пока не реализована)
        Ok(vec![])
    }

    fn calculate_profitability(&self, quote_a: &SwapQuote, quote_b: &SwapQuote) -> Result<f64> {
        // Для арбитража: SOL → USDC → SOL
        // Прибыльность = (final_sol - initial_sol) / initial_sol * 10000
        let initial_sol = quote_a.amount_in as f64;
        let final_sol = quote_b.amount_out as f64;
        
        let profit_ratio = (final_sol - initial_sol) / initial_sol;
        let profit_bps = (profit_ratio * 10000.0) as i32;
        
        Ok(profit_bps as f64)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl CrossDexScanner {
    /// Генерирует JSON-отчет об арбитраже
    async fn generate_json_report(
        &self,
        opportunity: &ArbitrageOpportunity,
        pool_a_info: &PoolInfo,
        pool_b_info: &PoolInfo,
        quote_a: &SwapQuote,
        quote_b: &SwapQuote,
    ) -> Result<ArbitrageReport> {
        let route_a = RouteDetails {
            dex: format!("{:?}", pool_a_info.dex_label),
            pool_address: pool_a_info.pool_address.to_string(),
            token_in: TokenDetails {
                mint: pool_a_info.token_a.mint.to_string(),
                symbol: pool_a_info.token_a.symbol.clone(),
                decimals: pool_a_info.token_a.decimals,
                amount_ui: lamports_to_sol(quote_a.amount_in),
            },
            token_out: TokenDetails {
                mint: pool_a_info.token_b.mint.to_string(),
                symbol: pool_a_info.token_b.symbol.clone(),
                decimals: pool_a_info.token_b.decimals,
                amount_ui: lamports_to_usdc(quote_a.amount_out),
            },
            amount_in: quote_a.amount_in,
            amount_out: quote_a.amount_out,
            price: lamports_to_usdc(quote_a.amount_out) / lamports_to_sol(quote_a.amount_in),
            fee_bps: quote_a.route.total_fee_bps,
            fee_amount: quote_a.fee_amount,
        };

        let route_b = RouteDetails {
            dex: format!("{:?}", pool_b_info.dex_label),
            pool_address: pool_b_info.pool_address.to_string(),
            token_in: TokenDetails {
                mint: pool_b_info.token_b.mint.to_string(),
                symbol: pool_b_info.token_b.symbol.clone(),
                decimals: pool_b_info.token_b.decimals,
                amount_ui: lamports_to_usdc(quote_b.amount_in),
            },
            token_out: TokenDetails {
                mint: pool_b_info.token_a.mint.to_string(),
                symbol: pool_b_info.token_a.symbol.clone(),
                decimals: pool_b_info.token_a.decimals,
                amount_ui: lamports_to_sol(quote_b.amount_out),
            },
            amount_in: quote_b.amount_in,
            amount_out: quote_b.amount_out,
            price: lamports_to_sol(quote_b.amount_out) / lamports_to_usdc(quote_b.amount_in),
            fee_bps: quote_b.route.total_fee_bps,
            fee_amount: quote_b.fee_amount,
        };

        let fees_breakdown = FeesBreakdown {
            pool_a_fee: quote_a.fee_amount,
            pool_b_fee: quote_b.fee_amount,
            priority_fee: opportunity.pnl_breakdown.priority_fee,
            rent: opportunity.pnl_breakdown.rent_fee,
            total_fees: opportunity.pnl_breakdown.priority_fee + opportunity.pnl_breakdown.rent_fee + quote_a.fee_amount + quote_b.fee_amount,
        };

        let slippage_protection = SlippageProtection {
            slippage_bps: 150, // Hardcoded for now
            min_amount_out_a: opportunity.min_out_a,
            min_amount_out_b: opportunity.min_out_b,
            slippage_buffer: opportunity.min_out_a.saturating_sub(quote_a.amount_out),
        };

        let execution_plan = ExecutionPlan {
            instructions_count: 3, // ComputeBudget + 2 swaps
            estimated_compute_units: 400_000,
            priority_fee_microlamports: opportunity.pnl_breakdown.priority_fee * 1_000_000,
            simulate_only: true, // From config
            recommended_action: if opportunity.profit_bps > 50 {
                "EXECUTE".to_string()
            } else {
                "SKIP".to_string()
            },
            risk_assessment: format!("{:?}", opportunity.risk_score),
        };

        let arbitrage_details = ArbitrageDetails {
            route_a,
            route_b,
            fees_breakdown,
            slippage_protection,
            execution_plan,
        };

        let report = ArbitrageReport::new(
            opportunity.profit_bps > 0,
            opportunity.profit_bps as f64,
            lamports_to_sol(opportunity.profit_amount),
            lamports_to_sol(opportunity.min_out_b),
            vec![pool_a_info.clone(), pool_b_info.clone()],
            arbitrage_details,
        );

        Ok(report)
    }
}
