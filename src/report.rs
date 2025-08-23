// src/report.rs
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use crate::exchanges::types::PoolInfo;

#[derive(Debug, Serialize, Deserialize)]
pub struct ArbitrageReport {
    // Основные результаты
    pub profitable: bool,
    pub spread_bps: f64,
    pub pnl: f64,
    pub min_out: f64,
    pub transaction_signature: Option<String>,
    
    // Детали пулов
    pub pool_states: Vec<PoolInfo>,
    
    // Детали арбитража
    pub arbitrage_details: ArbitrageDetails,
    
    // Метаданные
    pub timestamp: DateTime<Utc>,
    pub simulation_logs: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ArbitrageDetails {
    pub route_a: RouteDetails,
    pub route_b: RouteDetails,
    pub fees_breakdown: FeesBreakdown,
    pub slippage_protection: SlippageProtection,
    pub execution_plan: ExecutionPlan,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RouteDetails {
    pub dex: String,
    pub pool_address: String,
    pub token_in: TokenDetails,
    pub token_out: TokenDetails,
    pub amount_in: u64,
    pub amount_out: u64,
    pub price: f64,
    pub fee_bps: u32,
    pub fee_amount: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenDetails {
    pub mint: String,
    pub symbol: String,
    pub decimals: u8,
    pub amount_ui: f64, // Amount in UI format (considering decimals)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FeesBreakdown {
    pub pool_a_fee: u64,
    pub pool_b_fee: u64,
    pub priority_fee: u64,
    pub rent: u64,
    pub total_fees: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SlippageProtection {
    pub slippage_bps: u32,
    pub min_amount_out_a: u64,
    pub min_amount_out_b: u64,
    pub slippage_buffer: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExecutionPlan {
    pub instructions_count: u32,
    pub estimated_compute_units: u32,
    pub priority_fee_microlamports: u64,
    pub simulate_only: bool,
    pub recommended_action: String,
    pub risk_assessment: String,
}

impl ArbitrageReport {
    pub fn new(
        profitable: bool,
        spread_bps: f64,
        pnl: f64,
        min_out: f64,
        pool_states: Vec<PoolInfo>,
        arbitrage_details: ArbitrageDetails,
    ) -> Self {
        Self {
            profitable,
            spread_bps,
            pnl,
            min_out,
            transaction_signature: None,
            pool_states,
            arbitrage_details,
            timestamp: Utc::now(),
            simulation_logs: None,
        }
    }

    pub fn with_transaction_signature(mut self, signature: String) -> Self {
        self.transaction_signature = Some(signature);
        self
    }

    pub fn with_simulation_logs(mut self, logs: Vec<String>) -> Self {
        self.simulation_logs = Some(logs);
        self
    }

    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::exchanges::types::PoolInfo;
    use solana_sdk::pubkey::Pubkey;
    
    #[test]
    fn test_arbitrage_report_creation() {
        use crate::exchanges::types::{DexLabel, TokenInfo, PoolReserves, PoolFees, PoolState};
        
        let pool_state = PoolInfo {
            pool_address: Pubkey::default(),
            dex_label: DexLabel::RaydiumV4,
            token_a: TokenInfo {
                mint: Pubkey::default(),
                symbol: "TOKEN_A".to_string(),
                decimals: 6,
                vault: Pubkey::default(),
            },
            token_b: TokenInfo {
                mint: Pubkey::default(),
                symbol: "TOKEN_B".to_string(),
                decimals: 6,
                vault: Pubkey::default(),
            },
            reserves: PoolReserves {
                token_a_reserve: 1000,
                token_b_reserve: 1000,
                lp_supply: Some(1000),
            },
            fees: PoolFees {
                trade_fee_bps: 25,
                owner_trade_fee_bps: 0,
                owner_withdraw_fee_bps: 0,
            },
            pool_state: PoolState::Active,
        };
        
        let arbitrage_details = ArbitrageDetails {
            route_a: RouteDetails {
                dex: "Test".to_string(),
                pool_address: "test".to_string(),
                token_in: TokenDetails {
                    mint: "test".to_string(),
                    symbol: "TEST".to_string(),
                    decimals: 6,
                    amount_ui: 1.0,
                },
                token_out: TokenDetails {
                    mint: "test".to_string(),
                    symbol: "TEST".to_string(),
                    decimals: 6,
                    amount_ui: 1.0,
                },
                amount_in: 1000,
                amount_out: 1000,
                price: 1.0,
                fee_bps: 25,
                fee_amount: 25,
            },
            route_b: RouteDetails {
                dex: "Test".to_string(),
                pool_address: "test".to_string(),
                token_in: TokenDetails {
                    mint: "test".to_string(),
                    symbol: "TEST".to_string(),
                    decimals: 6,
                    amount_ui: 1.0,
                },
                token_out: TokenDetails {
                    mint: "test".to_string(),
                    symbol: "TEST".to_string(),
                    decimals: 6,
                    amount_ui: 1.0,
                },
                amount_in: 1000,
                amount_out: 1000,
                price: 1.0,
                fee_bps: 25,
                fee_amount: 25,
            },
            fees_breakdown: FeesBreakdown {
                pool_a_fee: 25,
                pool_b_fee: 25,
                priority_fee: 1000,
                rent: 2039280,
                total_fees: 2039330,
            },
            slippage_protection: SlippageProtection {
                slippage_bps: 100,
                min_amount_out_a: 990,
                min_amount_out_b: 990,
                slippage_buffer: 10,
            },
            execution_plan: ExecutionPlan {
                instructions_count: 3,
                estimated_compute_units: 400000,
                priority_fee_microlamports: 1000,
                simulate_only: true,
                recommended_action: "EXECUTE".to_string(),
                risk_assessment: "Low".to_string(),
            },
        };

        let report = ArbitrageReport::new(
            true,
            100.0,
            0.5,
            99.5,
            vec![pool_state],
            arbitrage_details,
        );
        
        assert!(report.profitable);
        assert_eq!(report.spread_bps, 100);
        assert_eq!(report.pnl, 0.5);
        assert_eq!(report.min_out, 99.5);
        assert_eq!(report.pool_states.len(), 1);
        assert!(report.timestamp > Utc::now() - chrono::Duration::seconds(1));
    }
    
    #[test]
    fn test_arbitrage_report_serialization() {
        use crate::exchanges::types::{DexLabel, TokenInfo, PoolReserves, PoolFees, PoolState};
        
        let pool_state = PoolInfo {
            pool_address: Pubkey::default(),
            dex_label: DexLabel::RaydiumV4,
            token_a: TokenInfo {
                mint: Pubkey::default(),
                symbol: "TOKEN_A".to_string(),
                decimals: 6,
                vault: Pubkey::default(),
            },
            token_b: TokenInfo {
                mint: Pubkey::default(),
                symbol: "TOKEN_B".to_string(),
                decimals: 6,
                vault: Pubkey::default(),
            },
            reserves: PoolReserves {
                token_a_reserve: 1000,
                token_b_reserve: 1000,
                lp_supply: Some(1000),
            },
            fees: PoolFees {
                trade_fee_bps: 25,
                owner_trade_fee_bps: 0,
                owner_withdraw_fee_bps: 0,
            },
            pool_state: PoolState::Active,
        };
        
        let arbitrage_details = ArbitrageDetails {
            route_a: RouteDetails {
                dex: "Test".to_string(),
                pool_address: "test".to_string(),
                token_in: TokenDetails {
                    mint: "test".to_string(),
                    symbol: "TEST".to_string(),
                    decimals: 6,
                    amount_ui: 1.0,
                },
                token_out: TokenDetails {
                    mint: "test".to_string(),
                    symbol: "TEST".to_string(),
                    decimals: 6,
                    amount_ui: 1.0,
                },
                amount_in: 1000,
                amount_out: 1000,
                price: 1.0,
                fee_bps: 25,
                fee_amount: 25,
            },
            route_b: RouteDetails {
                dex: "Test".to_string(),
                pool_address: "test".to_string(),
                token_in: TokenDetails {
                    mint: "test".to_string(),
                    symbol: "TEST".to_string(),
                    decimals: 6,
                    amount_ui: 1.0,
                },
                token_out: TokenDetails {
                    mint: "test".to_string(),
                    symbol: "TEST".to_string(),
                    decimals: 6,
                    amount_ui: 1.0,
                },
                amount_in: 1000,
                amount_out: 1000,
                price: 1.0,
                fee_bps: 25,
                fee_amount: 25,
            },
            fees_breakdown: FeesBreakdown {
                pool_a_fee: 25,
                pool_b_fee: 25,
                priority_fee: 1000,
                rent: 2039280,
                total_fees: 2039330,
            },
            slippage_protection: SlippageProtection {
                slippage_bps: 100,
                min_amount_out_a: 990,
                min_amount_out_b: 990,
                slippage_buffer: 10,
            },
            execution_plan: ExecutionPlan {
                instructions_count: 3,
                estimated_compute_units: 400000,
                priority_fee_microlamports: 1000,
                simulate_only: true,
                recommended_action: "EXECUTE".to_string(),
                risk_assessment: "Low".to_string(),
            },
        };

        let report = ArbitrageReport::new(
            true,
            100.0,
            0.5,
            99.5,
            vec![pool_state],
            arbitrage_details,
        );
        
        let json = serde_json::to_string(&report).unwrap();
        let deserialized: ArbitrageReport = serde_json::from_str(&json).unwrap();
        
        assert_eq!(report.profitable, deserialized.profitable);
        assert_eq!(report.spread_bps, deserialized.spread_bps);
        assert_eq!(report.pnl, deserialized.pnl);
    }
}
