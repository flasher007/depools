use async_trait::async_trait;
use anyhow::{Result, anyhow};
use reqwest::Client;
use serde::Deserialize;
use solana_sdk::pubkey::Pubkey;
use tracing::{info, warn};

use crate::exchanges::types::{SwapQuote, PoolInfo, SwapRoute, SwapHop, PoolReserves, PoolFees, TokenInfo, DexLabel, PoolState};
use super::QuoteApiClient;

/// Структура ответа от Orca API v2
#[derive(Debug, Deserialize)]
struct OrcaApiResponse {
    data: OrcaPoolData,
    meta: OrcaMeta,
}

/// Основные данные пула от Orca API
#[derive(Debug, Deserialize)]
struct OrcaPoolData {
    address: String,
    #[serde(rename = "whirlpoolsConfig")]
    whirlpools_config: String,
    #[serde(rename = "whirlpoolBump")]
    whirlpool_bump: Vec<u8>,
    #[serde(rename = "tickSpacing")]
    tick_spacing: u32,
    #[serde(rename = "tickSpacingSeed")]
    tick_spacing_seed: Vec<u8>,
    #[serde(rename = "feeRate")]
    fee_rate: u32,
    #[serde(rename = "protocolFeeRate")]
    protocol_fee_rate: u32,
    liquidity: String,
    #[serde(rename = "sqrtPrice")]
    sqrt_price: String,
    #[serde(rename = "tickCurrentIndex")]
    tick_current_index: i32,
    #[serde(rename = "protocolFeeOwedA")]
    protocol_fee_owed_a: String,
    #[serde(rename = "protocolFeeOwedB")]
    protocol_fee_owed_b: String,
    #[serde(rename = "tokenMintA")]
    token_mint_a: String,
    #[serde(rename = "tokenVaultA")]
    token_vault_a: String,
    #[serde(rename = "feeGrowthGlobalA")]
    fee_growth_global_a: String,
    #[serde(rename = "tokenMintB")]
    token_mint_b: String,
    #[serde(rename = "tokenVaultB")]
    token_vault_b: String,
    #[serde(rename = "feeGrowthGlobalB")]
    fee_growth_global_b: String,
    #[serde(rename = "rewardLastUpdatedTimestamp")]
    reward_last_updated_timestamp: String,
    #[serde(rename = "updatedAt")]
    updated_at: String,
    #[serde(rename = "updatedSlot")]
    updated_slot: u64,
    #[serde(rename = "writeVersion")]
    write_version: u64,
    #[serde(rename = "hasWarning")]
    has_warning: bool,
    #[serde(rename = "poolType")]
    pool_type: String,
    #[serde(rename = "tokenA")]
    token_a: OrcaToken,
    #[serde(rename = "tokenB")]
    token_b: OrcaToken,
    price: String,
    #[serde(rename = "tvlUsdc")]
    tvl_usdc: String,
    #[serde(rename = "yieldOverTvl")]
    yield_over_tvl: String,
    #[serde(rename = "tokenBalanceA")]
    token_balance_a: String,
    #[serde(rename = "tokenBalanceB")]
    token_balance_b: String,
    stats: OrcaStats,
    rewards: Vec<OrcaReward>,
    #[serde(rename = "lockedLiquidityPercent")]
    locked_liquidity_percent: Vec<OrcaLockedLiquidity>,
    #[serde(rename = "feeTierIndex")]
    fee_tier_index: u32,
    #[serde(rename = "adaptiveFeeEnabled")]
    adaptive_fee_enabled: bool,
    #[serde(rename = "adaptiveFee")]
    adaptive_fee: Option<serde_json::Value>,
    #[serde(rename = "tradeEnableTimestamp")]
    trade_enable_timestamp: String,
}

/// Метаданные ответа
#[derive(Debug, Deserialize)]
struct OrcaMeta {
    cursor: Option<String>,
}

/// Информация о токене
#[derive(Debug, Deserialize)]
struct OrcaToken {
    address: String,
    #[serde(rename = "programId")]
    program_id: String,
    #[serde(rename = "imageUrl")]
    image_url: String,
    name: String,
    symbol: String,
    decimals: u8,
    tags: Vec<String>,
}

/// Статистика пула
#[derive(Debug, Deserialize)]
struct OrcaStats {
    #[serde(rename = "24h")]
    day: OrcaStatsItem,
    #[serde(rename = "7d")]
    week: OrcaStatsItem,
    #[serde(rename = "30d")]
    month: OrcaStatsItem,
}

/// Элемент статистики
#[derive(Debug, Deserialize)]
struct OrcaStatsItem {
    volume: String,
    fees: String,
    rewards: String,
    #[serde(rename = "yieldOverTvl")]
    yield_over_tvl: String,
}

/// Информация о наградах
#[derive(Debug, Deserialize)]
struct OrcaReward {
    mint: String,
    vault: String,
    authority: String,
    #[serde(rename = "emissions_per_second_x64")]
    emissions_per_second_x64: String,
    #[serde(rename = "growth_global_x64")]
    growth_global_x64: String,
    active: bool,
    #[serde(rename = "emissionsPerSecond")]
    emissions_per_second: String,
}

/// Информация о заблокированной ликвидности
#[derive(Debug, Deserialize)]
struct OrcaLockedLiquidity {
    name: String,
    #[serde(rename = "locked_percentage")]
    locked_percentage: String,
    #[serde(rename = "lockedPercentage")]
    locked_percentage_alt: String,
}

/// Orca Quote API клиент
pub struct OrcaQuoteApiClient {
    http_client: Client,
    base_url: String,
}

impl OrcaQuoteApiClient {
    pub fn new() -> Self {
        Self {
            http_client: Client::new(),
            base_url: "https://api.orca.so/v2/solana".to_string(),
        }
    }

    /// Найти пул по адресу через Orca API
    async fn find_pool(&self, pool_pubkey: &Pubkey) -> Result<OrcaPoolData> {
        let url = format!("{}/pools/{}", self.base_url, pool_pubkey);
        
        info!("🔍 Fetching Orca pool info from: {}", url);
        
        let response = self.http_client.get(&url).send().await?;
        
        if !response.status().is_success() {
            return Err(anyhow!("Orca API request failed with status: {}", response.status()));
        }
        
        let orca_response: OrcaApiResponse = response.json().await?;
        
        info!("✅ Successfully parsed Orca pool data for {}", pool_pubkey);
        Ok(orca_response.data)
    }
}

#[async_trait]
impl QuoteApiClient for OrcaQuoteApiClient {
    async fn get_quote(&self, pool_pubkey: &Pubkey, amount_in: u64) -> Result<SwapQuote> {
        let pool = self.find_pool(pool_pubkey).await?;
        
        // Определяем, какой токен является входным (SOL) и выходным (USDC)
        // SOL всегда имеет 9 decimals, USDC имеет 6 decimals
        // ВАЖНО: API возвращает значения в lamports/units, поэтому конвертируем
        let (sol_reserve, usdc_reserve) = if pool.token_a.decimals == 9 {
            // token_a = SOL, token_b = USDC
            let sol_balance = pool.token_balance_a.parse::<f64>()?; // в lamports
            let usdc_balance = pool.token_balance_b.parse::<f64>()?; // в USDC units
            (sol_balance as u64, usdc_balance as u64)
        } else {
            // token_b = SOL, token_a = USDC
            let sol_balance = pool.token_balance_b.parse::<f64>()?; // в lamports  
            let usdc_balance = pool.token_balance_a.parse::<f64>()?; // в USDC units
            (sol_balance as u64, usdc_balance as u64)
        };
        
        // fee_rate в API уже в базисных пунктах деленных на 100 (400 = 4 bps = 0.04%)
        let trade_fee_bps = pool.fee_rate / 100; // 400 -> 4 bps
        
        info!("🔍 Orca AMM calculation: amount_in={}, sol_reserve={}, usdc_reserve={}, fee={} bps", 
              amount_in, sol_reserve, usdc_reserve, trade_fee_bps);
        
        // Для арбитража: USDC → SOL
        // Используем готовую цену из API вместо расчета AMM для точности
        let price_sol_usdc = pool.price.parse::<f64>()?; // цена SOL в USDC
        let fee_multiplier = 1.0 - (trade_fee_bps as f64 / 10000.0);
        
        // Простой расчет: amount_out_sol = amount_in_usdc / price_sol_usdc * fee_multiplier
        let amount_in_usdc = amount_in as f64 / 1_000_000.0; // конвертируем в USDC
        let amount_out_sol = (amount_in_usdc / price_sol_usdc) * fee_multiplier;
        let amount_out = (amount_out_sol * 1_000_000_000.0) as u64; // конвертируем в lamports
        
        info!("🔍 Orca price-based calculation: {} USDC → {} SOL (price={}, fee={}%)", 
              amount_in_usdc, amount_out_sol, price_sol_usdc, trade_fee_bps as f64 / 100.0);
        
        let fee_amount = amount_in - (amount_in as f64 * (1.0 - trade_fee_bps as f64 / 10000.0)) as u64;
        
        let route = SwapRoute {
            hops: vec![SwapHop {
                pool_address: *pool_pubkey,
                dex_label: DexLabel::OrcaWhirlpool,
                token_in: pool.token_mint_a.parse()?,
                token_out: pool.token_mint_b.parse()?,
                amount_in,
                amount_out,
                fee_bps: trade_fee_bps,
            }],
            total_fee_bps: trade_fee_bps,
        };
        
        Ok(SwapQuote {
            pool_address: *pool_pubkey,
            dex_label: DexLabel::OrcaWhirlpool,
            token_in: pool.token_mint_a.parse()?,
            token_out: pool.token_mint_b.parse()?,
            amount_in,
            amount_out,
            min_amount_out: amount_out,
            price_impact_bps: 0,
            fee_amount,
            route,
        })
    }
    
    async fn get_pool_info(&self, pool_pubkey: &Pubkey) -> Result<PoolInfo> {
        let pool = self.find_pool(pool_pubkey).await?;
        
        let token_a = TokenInfo {
            mint: pool.token_mint_a.parse()?,
            symbol: pool.token_a.symbol,
            decimals: pool.token_a.decimals,
            vault: pool.token_vault_a.parse()?,
        };
        
        let token_b = TokenInfo {
            mint: pool.token_mint_b.parse()?,
            symbol: pool.token_b.symbol,
            decimals: pool.token_b.decimals,
            vault: pool.token_vault_b.parse()?,
        };
        
        // fee_rate в API уже в базисных пунктах деленных на 100 (400 = 4 bps = 0.04%)
        let fees = PoolFees {
            trade_fee_bps: pool.fee_rate / 100, // 400 -> 4 bps
            owner_trade_fee_bps: pool.protocol_fee_rate / 100, // протокольная комиссия
            owner_withdraw_fee_bps: 0,
        };
        
        // Определяем правильные резервы для SOL и USDC
        // ВАЖНО: API возвращает значения в lamports/units, поэтому конвертируем
        let (sol_reserve, usdc_reserve) = if pool.token_a.decimals == 9 {
            // token_a = SOL, token_b = USDC
            let sol_balance = pool.token_balance_a.parse::<f64>()?; // в lamports
            let usdc_balance = pool.token_balance_b.parse::<f64>()?; // в USDC units
            (sol_balance as u64, usdc_balance as u64)
        } else {
            // token_b = SOL, token_a = USDC
            let sol_balance = pool.token_balance_b.parse::<f64>()?; // в lamports  
            let usdc_balance = pool.token_balance_a.parse::<f64>()?; // в USDC units
            (sol_balance as u64, usdc_balance as u64)
        };
        
        let reserves = PoolReserves {
            token_a_reserve: sol_reserve,  // SOL всегда token_a
            token_b_reserve: usdc_reserve, // USDC всегда token_b
            lp_supply: None, // API не предоставляет LP supply
        };
        
        Ok(PoolInfo {
            pool_address: *pool_pubkey,
            dex_label: DexLabel::OrcaWhirlpool,
            token_a,
            token_b,
            reserves,
            fees,
            pool_state: PoolState::Active,
        })
    }
    
    async fn is_available(&self) -> bool {
        // Проверяем доступность API с тестовым запросом
        match self.http_client.get(&format!("{}/pools", self.base_url)).send().await {
            Ok(response) => {
                let is_available = response.status().is_success();
                if is_available {
                    info!("✅ Orca API is available");
                } else {
                    warn!("⚠️ Orca API returned status: {}", response.status());
                }
                is_available
            }
            Err(e) => {
                warn!("⚠️ Orca API is not available: {}", e);
                false
            }
        }
    }
}