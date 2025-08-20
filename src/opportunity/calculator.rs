use anyhow::Result;
use crate::exchanges::types::{SwapQuote, ArbitrageOpportunity};

pub struct ProfitabilityCalculator;

impl ProfitabilityCalculator {
    pub fn calculate_profit_bps(quote_a: &SwapQuote, quote_b: &SwapQuote) -> Result<i32> {
        if quote_a.token_in != quote_b.token_out || quote_a.token_out != quote_b.token_in {
            return Err(anyhow::anyhow!("Incompatible token pairs"));
        }
        
        let amount_in = quote_a.amount_in as f64;
        let amount_out_a = quote_a.amount_out as f64;
        let amount_out_b = quote_b.amount_out as f64;
        
        if amount_out_a <= 0.0 || amount_out_b <= 0.0 {
            return Ok(0);
        }
        
        // Calculate profit in basis points
        let profit_bps = ((amount_out_b - amount_out_a) / amount_out_a * 10000.0) as i32;
        
        Ok(profit_bps)
    }
    
    pub fn calculate_net_profit(quote_a: &SwapQuote, quote_b: &SwapQuote, priority_fee: u64) -> Result<u64> {
        let profit_bps = Self::calculate_profit_bps(quote_a, quote_b)?;
        
        if profit_bps <= 0 {
            return Ok(0);
        }
        
        let gross_profit = (quote_a.amount_in as f64 * profit_bps as f64 / 10000.0) as u64;
        let net_profit = gross_profit.saturating_sub(priority_fee);
        
        Ok(net_profit)
    }
    
    pub fn is_profitable(quote_a: &SwapQuote, quote_b: &SwapQuote, min_profit_bps: i32) -> Result<bool> {
        let profit_bps = Self::calculate_profit_bps(quote_a, quote_b)?;
        Ok(profit_bps >= min_profit_bps)
    }
}
