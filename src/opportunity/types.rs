use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpportunityConfig {
    pub min_profit_bps: i32,
    pub max_risk_score: RiskLevel,
    pub max_slippage_bps: u32,
    pub enable_flash_loans: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Extreme,
}

impl RiskLevel {
    pub fn as_u8(&self) -> u8 {
        match self {
            RiskLevel::Low => 0,
            RiskLevel::Medium => 1,
            RiskLevel::High => 2,
            RiskLevel::Extreme => 3,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpportunityMetrics {
    pub total_opportunities: u64,
    pub profitable_opportunities: u64,
    pub total_profit: u64,
    pub average_profit_bps: f64,
    pub success_rate: f64,
}
