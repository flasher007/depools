use async_trait::async_trait;
use anyhow::{Result, anyhow};
use reqwest::Client;
use serde::Deserialize;
use solana_sdk::pubkey::Pubkey;
use tracing::{info, warn};

use crate::exchanges::types::{SwapQuote, PoolInfo, SwapRoute, SwapHop, PoolReserves, PoolFees, TokenInfo, DexLabel};
use super::QuoteApiClient;

/// Структура ответа от Raydium API для информации о пуле
#[derive(Debug, Deserialize)]
struct RaydiumPoolResponse {
    #[serde(rename = "type")]
    pool_type: String,
    #[serde(rename = "programId")]
    program_id: String,
    id: String,
    #[serde(rename = "mintA")]
    mint_a: RaydiumToken,
    #[serde(rename = "mintB")]
    mint_b: RaydiumToken,
    price: f64,
    #[serde(rename = "mintAmountA")]
    mint_amount_a: f64,
    #[serde(rename = "mintAmountB")]
    mint_amount_b: f64,
    #[serde(rename = "feeRate")]
    fee_rate: f64,
    #[serde(rename = "openTime")]
    open_time: String,
    tvl: f64,
    day: RaydiumDayStats,
    week: RaydiumWeekStats,
    month: RaydiumMonthStats,
    pooltype: Vec<String>,
    #[serde(rename = "rewardDefaultPoolInfos")]
    reward_default_pool_infos: String,
    #[serde(rename = "rewardDefaultInfos")]
    reward_default_infos: Vec<RaydiumRewardInfo>,
    #[serde(rename = "farmUpcomingCount")]
    farm_upcoming_count: u32,
    #[serde(rename = "farmOngoingCount")]
    farm_ongoing_count: u32,
    #[serde(rename = "farmFinishedCount")]
    farm_finished_count: u32,
    #[serde(rename = "marketId")]
    market_id: String,
    #[serde(rename = "lpMint")]
    lp_mint: RaydiumToken,
    #[serde(rename = "lpPrice")]
    lp_price: f64,
    #[serde(rename = "lpAmount")]
    lp_amount: f64,
    #[serde(rename = "burnPercent")]
    burn_percent: f64,
    #[serde(rename = "launchMigratePool")]
    launch_migrate_pool: bool,
}

#[derive(Debug, Deserialize)]
struct RaydiumToken {
    #[serde(rename = "chainId")]
    chain_id: u32,
    address: String,
    #[serde(rename = "programId")]
    program_id: String,
    #[serde(rename = "logoURI")]
    logo_uri: String,
    symbol: String,
    name: String,
    decimals: u8,
    tags: Vec<String>,
    extensions: serde_json::Value,
}

#[derive(Debug, Deserialize)]
struct RaydiumDayStats {
    volume: f64,
    #[serde(rename = "volumeQuote")]
    volume_quote: f64,
    #[serde(rename = "volumeFee")]
    volume_fee: f64,
    apr: f64,
    #[serde(rename = "feeApr")]
    fee_apr: f64,
    #[serde(rename = "priceMin")]
    price_min: f64,
    #[serde(rename = "priceMax")]
    price_max: f64,
    #[serde(rename = "rewardApr")]
    reward_apr: Vec<f64>,
}

#[derive(Debug, Deserialize)]
struct RaydiumWeekStats {
    volume: f64,
    #[serde(rename = "volumeQuote")]
    volume_quote: f64,
    #[serde(rename = "volumeFee")]
    volume_fee: f64,
    apr: f64,
    #[serde(rename = "feeApr")]
    fee_apr: f64,
    #[serde(rename = "priceMin")]
    price_min: f64,
    #[serde(rename = "priceMax")]
    price_max: f64,
    #[serde(rename = "rewardApr")]
    reward_apr: Vec<f64>,
}

#[derive(Debug, Deserialize)]
struct RaydiumMonthStats {
    volume: f64,
    #[serde(rename = "volumeQuote")]
    volume_quote: f64,
    #[serde(rename = "volumeFee")]
    volume_fee: f64,
    apr: f64,
    #[serde(rename = "feeApr")]
    fee_apr: f64,
    #[serde(rename = "priceMin")]
    price_min: f64,
    #[serde(rename = "priceMax")]
    price_max: f64,
    #[serde(rename = "rewardApr")]
    reward_apr: Vec<f64>,
}

#[derive(Debug, Deserialize)]
struct RaydiumRewardInfo {
    mint: RaydiumToken,
    #[serde(rename = "perSecond")]
    per_second: String,
    #[serde(rename = "startTime")]
    start_time: String,
    #[serde(rename = "endTime")]
    end_time: String,
}

/// Raydium Quote API клиент
pub struct RaydiumQuoteApiClient {
    http_client: Client,
    base_url: String,
}

impl RaydiumQuoteApiClient {
    pub fn new() -> Self {
        Self {
            http_client: Client::new(),
            base_url: "https://api-v3.raydium.io".to_string(),
        }
    }

    /// Получить информацию о пуле через Raydium API
    async fn get_pool_info_from_api(&self, pool_pubkey: &Pubkey) -> Result<RaydiumPoolResponse> {
        let url = format!("{}/pools/info/ids?ids={}", self.base_url, pool_pubkey);
        
        info!("🔍 Fetching Raydium pool info from: {}", url);
        
        let response = self.http_client.get(&url).send().await?;
        
        if !response.status().is_success() {
            return Err(anyhow!("Raydium API request failed with status: {}", response.status()));
        }
        
        let data: serde_json::Value = response.json().await?;
        
        // Raydium API возвращает массив пулов
        if let Some(pools) = data.get("data").and_then(|v| v.as_array()) {
            if let Some(pool_data) = pools.first() {
                let pool: RaydiumPoolResponse = serde_json::from_value(pool_data.clone())?;
                info!("✅ Successfully parsed Raydium pool data for {}", pool_pubkey);
                return Ok(pool);
            }
        }
        
        Err(anyhow!("No pool data found in Raydium API response"))
    }

    /// Получить адреса vault'ов из блокчейна (так как API их не предоставляет)
    async fn get_vault_addresses(&self, pool_pubkey: &Pubkey) -> Result<(Pubkey, Pubkey)> {
        // Используем RPC клиент для получения vault адресов
        let rpc_client = solana_client::rpc_client::RpcClient::new("https://api.mainnet-beta.solana.com".to_string());
        
        // Получаем данные аккаунта пула
        let account_data = rpc_client.get_account_data(pool_pubkey)?;
        
        // Парсим vault адреса из данных пула
        // Raydium V4 структура: baseVault (offset 8), quoteVault (offset 40)
        if account_data.len() >= 48 {
            let base_vault = Pubkey::try_from(&account_data[8..40])?;
            let quote_vault = Pubkey::try_from(&account_data[40..72])?;
            Ok((base_vault, quote_vault))
        } else {
            warn!("⚠️ Pool account data too short, using fallback vault addresses");
            Ok((Pubkey::default(), Pubkey::default()))
        }
    }
}

#[async_trait]
impl QuoteApiClient for RaydiumQuoteApiClient {
    async fn get_quote(&self, pool_pubkey: &Pubkey, amount_in: u64) -> Result<SwapQuote> {
        let pool = self.get_pool_info_from_api(pool_pubkey).await?;
        
        // Рассчитываем amount_out используя текущие резервы и комиссии
        let trade_fee_bps = (pool.fee_rate * 10000.0) as u32;
        
        // Простая формула AMM для расчета (можно заменить на более точную)
        // API возвращает значения в правильных единицах (lamports для SOL, USDC units для USDC)
        let reserve_in = (pool.mint_amount_a * 1_000_000_000.0) as u64; // SOL -> lamports
        let reserve_out = (pool.mint_amount_b * 1_000_000.0) as u64; // USDC -> USDC units
        
        info!("🔍 Raydium AMM calculation: amount_in={}, reserve_in={}, reserve_out={}, fee={} bps", 
              amount_in, reserve_in, reserve_out, trade_fee_bps);
        
        let amount_out = if amount_in <= reserve_in {
            let fee_multiplier = 1.0 - (trade_fee_bps as f64 / 10000.0);
            let adjusted_amount_in = (amount_in as f64 * fee_multiplier) as u64;
            
            // Constant product formula: (x + dx) * (y - dy) = x * y
            // dy = (y * dx) / (x + dx)
            // Используем u128 для избежания переполнения
            let dy = (reserve_out as u128 * adjusted_amount_in as u128) / (reserve_in as u128 + adjusted_amount_in as u128);
            dy as u64
        } else {
            0
        };
        
        let fee_amount = amount_in - (amount_in as f64 * (1.0 - trade_fee_bps as f64 / 10000.0)) as u64;
        
        let route = SwapRoute {
            hops: vec![SwapHop {
                pool_address: *pool_pubkey,
                dex_label: DexLabel::RaydiumV4,
                token_in: pool.mint_a.address.parse()?,
                token_out: pool.mint_b.address.parse()?,
                amount_in,
                amount_out,
                fee_bps: trade_fee_bps,
            }],
            total_fee_bps: trade_fee_bps,
        };
        
        Ok(SwapQuote {
            pool_address: *pool_pubkey,
            dex_label: DexLabel::RaydiumV4,
            token_in: pool.mint_a.address.parse()?,
            token_out: pool.mint_b.address.parse()?,
            amount_in,
            amount_out,
            min_amount_out: amount_out, // Без slippage protection пока
            price_impact_bps: 0, // Нужно рассчитать
            fee_amount,
            route,
        })
    }
    
    async fn get_pool_info(&self, pool_pubkey: &Pubkey) -> Result<PoolInfo> {
        let pool = self.get_pool_info_from_api(pool_pubkey).await?;
        
        // Получаем vault адреса (пока заглушки)
        let (base_vault, quote_vault) = self.get_vault_addresses(pool_pubkey).await?;
        
        let token_a = TokenInfo {
            mint: pool.mint_a.address.parse()?,
            symbol: pool.mint_a.symbol,
            decimals: pool.mint_a.decimals,
            vault: base_vault,
        };
        
        let token_b = TokenInfo {
            mint: pool.mint_b.address.parse()?,
            symbol: pool.mint_b.symbol,
            decimals: pool.mint_b.decimals,
            vault: quote_vault,
        };
        
        let fees = PoolFees {
            trade_fee_bps: (pool.fee_rate * 10000.0) as u32,
            owner_trade_fee_bps: 0, // API не предоставляет owner fee
            owner_withdraw_fee_bps: 0,
        };
        
        // API возвращает значения в правильных единицах
        // mint_amount_a уже в SOL, mint_amount_b уже в USDC
        // Используем u128 для избежания переполнения
        let reserves = PoolReserves {
            token_a_reserve: (pool.mint_amount_a as u128 * 1_000_000_000u128) as u64, // SOL -> lamports
            token_b_reserve: (pool.mint_amount_b as u128 * 1_000_000u128) as u64, // USDC -> USDC units
            lp_supply: Some((pool.lp_amount as u128 * 1_000_000_000u128) as u64), // LP tokens
        };
        
        Ok(PoolInfo {
            pool_address: *pool_pubkey,
            dex_label: DexLabel::RaydiumV4,
            token_a,
            token_b,
            reserves,
            fees,
            pool_state: crate::exchanges::types::PoolState::Active,
        })
    }
    
    async fn is_available(&self) -> bool {
        // Проверяем доступность API
        match self.http_client.get(&format!("{}/pools/info/ids?ids=test", self.base_url)).send().await {
            Ok(response) => {
                let is_available = response.status().is_success() || response.status().as_u16() == 400; // 400 означает что API работает, но ID неверный
                if is_available {
                    info!("✅ Raydium API is available");
                } else {
                    warn!("⚠️ Raydium API returned status: {}", response.status());
                }
                is_available
            }
            Err(e) => {
                warn!("⚠️ Raydium API is not available: {}", e);
                false
            }
        }
    }
}
