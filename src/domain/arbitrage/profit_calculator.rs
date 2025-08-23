//! Profit calculation and analysis for arbitrage opportunities

use crate::shared::types::{Amount, Token};

/// Profit calculation result
#[derive(Debug, Clone)]
pub struct ProfitCalculation {
    pub gross_profit: Amount,
    pub net_profit: Amount,
    pub fees: Amount,
    pub gas_cost: Amount,
    pub profit_percentage: f64,
    pub roi: f64,
}

/// Profit calculator for arbitrage opportunities
pub struct ProfitCalculator {
    pub gas_price: u64,
    pub fee_multiplier: f64,
}

impl ProfitCalculator {
    pub fn new(gas_price: u64, fee_multiplier: f64) -> Self {
        Self {
            gas_price,
            fee_multiplier,
        }
    }

    pub fn calculate_profit(
        &self,
        amount_in: Amount,
        amount_out: Amount,
        gas_used: u64,
        fees: Vec<Amount>,
    ) -> ProfitCalculation {
        let gross_profit = if amount_out.value > amount_in.value {
            Amount::new(amount_out.value - amount_in.value, amount_out.decimals)
        } else {
            Amount::new(0, amount_in.decimals)
        };

        let total_fees = fees.iter().fold(Amount::new(0, 6), |acc, fee| {
            Amount::new(acc.value + fee.value, acc.decimals)
        });

        let gas_cost = Amount::new(self.gas_price * gas_used, 9);
        let net_profit = if gross_profit.value > (total_fees.value + gas_cost.value) {
            Amount::new(
                gross_profit.value - total_fees.value - gas_cost.value,
                gross_profit.decimals,
            )
        } else {
            Amount::new(0, gross_profit.decimals)
        };

        let profit_percentage = if amount_in.value > 0 {
            (gross_profit.value as f64 / amount_in.value as f64) * 100.0
        } else {
            0.0
        };

        let roi = if amount_in.value > 0 {
            (net_profit.value as f64 / amount_in.value as f64) * 100.0
        } else {
            0.0
        };

        ProfitCalculation {
            gross_profit,
            net_profit,
            fees: total_fees,
            gas_cost,
            profit_percentage,
            roi,
        }
    }

    pub fn is_profitable(&self, calculation: &ProfitCalculation, min_threshold: f64) -> bool {
        calculation.roi >= min_threshold
    }
}
