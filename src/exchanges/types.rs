use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DexLabel {
    RaydiumV4,
    OrcaWhirlpool,
}

impl DexLabel {
    pub fn as_str(&self) -> &'static str {
        match self {
            DexLabel::RaydiumV4 => "Raydium V4",
            DexLabel::OrcaWhirlpool => "Orca Whirlpool",
        }
    }

    pub fn program_id(&self) -> &'static str {
        match self {
            DexLabel::RaydiumV4 => "675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8",
            DexLabel::OrcaWhirlpool => "whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc",
        }
    }

    pub fn from_program_id(program_id: &str) -> Option<Self> {
        match program_id {
            "675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8" => Some(DexLabel::RaydiumV4),
            "whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc" => Some(DexLabel::OrcaWhirlpool),
            _ => None,
        }
    }
}

impl FromStr for DexLabel {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "raydium_v4" | "raydium" => Ok(DexLabel::RaydiumV4),
            "orca_whirlpool" | "orca" | "whirlpool" => Ok(DexLabel::OrcaWhirlpool),
            _ => Err(anyhow::anyhow!("Unknown DEX label: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolInfo {
    pub pool_address: Pubkey,
    pub dex_label: DexLabel,
    pub token_a: TokenInfo,
    pub token_b: TokenInfo,
    pub reserves: PoolReserves,
    pub fees: PoolFees,
    pub pool_state: PoolState,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenInfo {
    pub mint: Pubkey,
    pub symbol: String,
    pub decimals: u8,
    pub vault: Pubkey,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolReserves {
    pub token_a_reserve: u64,
    pub token_b_reserve: u64,
    pub lp_supply: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolFees {
    pub trade_fee_bps: u32,
    pub owner_trade_fee_bps: u32,
    pub owner_withdraw_fee_bps: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PoolState {
    Active,
    Paused,
    Closed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwapQuote {
    pub pool_address: Pubkey,
    pub dex_label: DexLabel,
    pub token_in: Pubkey,
    pub token_out: Pubkey,
    pub amount_in: u64,
    pub amount_out: u64,
    pub min_amount_out: u64,
    pub price_impact_bps: i32,
    pub fee_amount: u64,
    pub route: SwapRoute,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwapRoute {
    pub hops: Vec<SwapHop>,
    pub total_fee_bps: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwapHop {
    pub pool_address: Pubkey,
    pub dex_label: DexLabel,
    pub token_in: Pubkey,
    pub token_out: Pubkey,
    pub amount_in: u64,
    pub amount_out: u64,
    pub fee_bps: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PnlBreakdown {
    pub gross_profit: u64,
    pub priority_fee: u64,
    pub rent_fee: u64,
    pub net_profit: u64,
    pub is_profitable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArbitrageOpportunity {
    pub id: String,
    pub timestamp: u64,
    pub route_a: SwapRoute,
    pub route_b: SwapRoute,
    pub profit_bps: i32,
    pub profit_amount: u64,
    pub risk_score: RiskScore,
    pub pnl_breakdown: PnlBreakdown,
    pub min_out_a: u64,
    pub min_out_b: u64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum RiskScore {
    Low,
    Medium,
    High,
    Extreme,
}

impl RiskScore {
    pub fn from_profit_bps(profit_bps: i32) -> Self {
        match profit_bps {
            bps if bps < 50 => RiskScore::Low,
            bps if bps < 200 => RiskScore::Medium,
            bps if bps < 500 => RiskScore::High,
            _ => RiskScore::Extreme,
        }
    }
}
