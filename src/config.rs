use anyhow::{Context, Result};
use serde::Deserialize;
use std::{fs, path::Path};
use crate::app::AppCfg;

#[derive(Debug, Clone, Deserialize)]
pub struct RpcCfg { 
    pub url: String 
}

#[derive(Debug, Clone, Deserialize)]
pub struct WalletCfg { 
    pub keypair: String 
}

#[derive(Debug, Clone, Deserialize)]
pub struct TokenCfg {
    pub base_token: TokenInfo,
    pub quote_token: TokenInfo,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TokenInfo {
    pub mint: String,
    pub symbol: String,
    pub decimals: u8,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PoolsCfg {
    pub pool_a: String,
    pub pool_b: String,
    pub user_source_ata: Option<String>,
    pub user_dest_ata: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TradeCfg {
    pub amount_in: f64,
    pub spread_threshold_bps: u32,
    pub slippage_bps: u32,
    pub priority_fee_microlamports: u64,
    pub simulate_only: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ProgramsCfg {
    pub raydium_v4: String,
    pub orca_whirlpool: String,
    pub spl_token: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct StreamCfg {
    pub backend: String,         // "yellowstone"
    pub endpoint: String,
    pub x_token: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub rpc: RpcCfg,
    pub wallet: WalletCfg,
    pub tokens: TokenCfg,
    pub pools: PoolsCfg,
    pub trade: TradeCfg,
    pub programs: ProgramsCfg,
    pub stream: StreamCfg,
}

impl Config {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let s = fs::read_to_string(path.as_ref())?;
        let cfg: Self = toml::from_str(&s).context("parse Config.toml")?;
        Ok(cfg)
    }
}

impl From<AppCfg> for Config {
    fn from(app_cfg: AppCfg) -> Self {
        Self {
            rpc: RpcCfg { url: app_cfg.rpc_url },
            wallet: WalletCfg { keypair: app_cfg.keypair_path },
            tokens: TokenCfg {
                base_token: TokenInfo {
                    mint: app_cfg.base_token_mint.unwrap_or_else(|| "So11111111111111111111111111111111111111112".to_string()),
                    symbol: "SOL".to_string(),
                    decimals: 9,
                },
                quote_token: TokenInfo {
                    mint: app_cfg.quote_token_mint.unwrap_or_else(|| "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string()),
                    symbol: "USDC".to_string(),
                    decimals: 6,
                },
            },
            pools: PoolsCfg {
                pool_a: app_cfg.pool_addresses[0].clone(),
                pool_b: app_cfg.pool_addresses[1].clone(),
                user_source_ata: None,
                user_dest_ata: None,
            },
            trade: TradeCfg {
                amount_in: app_cfg.amount_in,
                spread_threshold_bps: app_cfg.spread_threshold_bps,
                slippage_bps: app_cfg.slippage_bps,
                priority_fee_microlamports: app_cfg.priority_fee,
                simulate_only: Some(app_cfg.simulate_only),
            },
            programs: ProgramsCfg {
                raydium_v4: app_cfg.raydium_program.unwrap_or_else(|| "675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8".to_string()),
                orca_whirlpool: app_cfg.orca_program.unwrap_or_else(|| "whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc".to_string()),
                spl_token: app_cfg.spl_token_program.unwrap_or_else(|| "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA".to_string()),
            },
            stream: StreamCfg {
                backend: "yellowstone".to_string(),
                endpoint: "grpc.yellowstone.finance:443".to_string(),
                x_token: "".to_string(),
            },
        }
    }
}

// CLI-based configuration
#[derive(Debug, Clone)]
pub struct CliConfig {
    pub rpc_url: String,
    pub keypair_path: String,
    pub amount_in: f64,
    pub spread_threshold_bps: u32,
    pub slippage_bps: u32,
    pub priority_fee: u64,
    pub simulate_only: bool,
}

impl CliConfig {
    pub fn new(
        rpc_url: String,
        keypair_path: String,
        amount_in: f64,
        spread_threshold_bps: u32,
        slippage_bps: u32,
        priority_fee: u64,
        simulate_only: bool,
    ) -> Self {
        Self {
            rpc_url,
            keypair_path,
            amount_in,
            spread_threshold_bps,
            slippage_bps,
            priority_fee,
            simulate_only,
        }
    }
}
