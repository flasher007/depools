use anyhow::Result;
use solana_sdk::pubkey::Pubkey;
use tokio::time::{Duration, sleep};
use tracing::{info, warn};
use std::any::Any;

use crate::exchanges::{DexAdapter, types::{ArbitrageOpportunity, SwapQuote, DexLabel, RiskScore, PoolInfo, SwapRoute, SwapHop}};
use crate::config::Config;
use super::OpportunityScanner;
use crate::exchanges::factory;

pub struct CrossDexScanner {
    adapters: Vec<Box<dyn DexAdapter>>,
    config: Config,
}

impl CrossDexScanner {
    pub fn new(config: Config) -> Result<Self> {
        let mut adapters = Vec::new();
        
        info!("🔧 Creating Raydium V4 adapter...");
        let raydium_adapter = factory::create_adapter(DexLabel::RaydiumV4, config.clone())?;
        info!("✅ Raydium V4 adapter created successfully");
        adapters.push(raydium_adapter);
        
        info!("🔧 Creating Orca Whirlpool adapter...");
        let orca_adapter = factory::create_adapter(DexLabel::OrcaWhirlpool, config.clone())?;
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
            warn!("⚠️  WARNING: Both pools assigned to same DEX: {:?}", dex_a);
        }
    }

    async fn scan_pool_pair(&self, pool_a: &str, pool_b: &str, amount_in: u64) -> Result<Option<ArbitrageOpportunity>> {
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
              quote_a.amount_out, pool_a_info.token_b.symbol, quote_a.fee_amount);
        info!("📊 Pool B ({:?}): {} {} → {} {}, fee={} bps", 
              dex_b, amount_in, pool_b_info.token_a.symbol,
              quote_b.amount_out, pool_b_info.token_b.symbol, quote_b.fee_amount);
        
        // Для арбитража проверяем разность цен между пулами
        let price_a = quote_a.amount_out as f64 / amount_in as f64;
        let price_b = quote_b.amount_out as f64 / amount_in as f64;
        
        info!("💱 Prices: Pool A = {:.6}, Pool B = {:.6}", price_a, price_b);
        
        // Рассчитываем прибыльность
        let profit_bps = self.calculate_profitability(&quote_a, &quote_b)?;
        
        // Создаем арбитражную возможность
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
                    token_in: pool_b_info.token_a.mint,
                    token_out: pool_b_info.token_b.mint,
                    amount_in,
                    amount_out: quote_b.amount_out,
                    fee_bps: quote_b.route.total_fee_bps,
                }],
                total_fee_bps: quote_b.route.total_fee_bps,
            },
            profit_bps: profit_bps as i32,
            profit_amount: 0, // Упрощенно
            risk_score: RiskScore::Low, // Упрощенно
        };
        
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
            if adapter.get_label() == dex_label {
                return adapter.get_pool_info(pool_address).await;
            }
        }
        
        Err(anyhow::anyhow!("No adapter found for DEX: {:?}", dex_label))
    }

    async fn get_quote_for_pool(&self, pool_address: &Pubkey, amount_in: u64, dex_label: DexLabel) -> Result<SwapQuote> {
        // Получаем quote через соответствующий адаптер
        for adapter in &self.adapters {
            if adapter.get_label() == dex_label {
                // Упрощенно: используем первый токен как входной
                let pool_info = adapter.get_pool_info(pool_address).await?;
                return adapter.get_swap_quote(pool_address, amount_in, &pool_info.token_a.mint).await;
            }
        }
        
        Err(anyhow::anyhow!("No adapter found for DEX: {:?}", dex_label))
    }
}

#[async_trait::async_trait]
impl OpportunityScanner for CrossDexScanner {
    fn scan_opportunities(&self, _pool_addresses: &[String]) -> Result<Vec<ArbitrageOpportunity>> {
        // Синхронная версия (пока не реализована)
        Ok(vec![])
    }

    fn calculate_profitability(&self, quote_a: &SwapQuote, quote_b: &SwapQuote) -> Result<f64> {
        // Простой расчет прибыльности
        let price_a = quote_a.amount_out as f64 / quote_a.amount_in as f64;
        let price_b = quote_b.amount_out as f64 / quote_b.amount_in as f64;
        
        let profit_ratio = (price_a - price_b).abs() / price_a.min(price_b);
        let profit_bps = (profit_ratio * 10000.0) as i32;
        
        Ok(profit_bps as f64)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl CrossDexScanner {
    pub async fn scan_opportunities_async(&self, pool_addresses: &[String]) -> Result<Vec<ArbitrageOpportunity>> {
        info!("🔍 Starting async scan of {} pools", pool_addresses.len());
        let mut opportunities = Vec::new();
        
        // Сканируем все возможные пары пулов
        for i in 0..pool_addresses.len() {
            for j in (i + 1)..pool_addresses.len() {
                let pool_a = &pool_addresses[i];
                let pool_b = &pool_addresses[j];
                
                info!("🔍 Scanning pair: {} vs {}", pool_a, pool_b);
                
                // Используем фиксированную сумму для тестирования
                let amount_in = 1_000_000; // 1 SOL в lamports
                
                if let Some(opportunity) = self.scan_pool_pair(pool_a, pool_b, amount_in).await? {
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
