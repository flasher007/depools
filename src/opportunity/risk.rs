use anyhow::Result;
use crate::exchanges::types::{SwapQuote, RiskScore};

pub struct RiskAssessor;

impl RiskAssessor {
    pub fn assess_opportunity_risk(
        quote_a: &SwapQuote,
        quote_b: &SwapQuote,
        market_volatility: f64,
    ) -> Result<RiskScore> {
        let price_impact_a = quote_a.price_impact_bps.abs() as f64;
        let price_impact_b = quote_b.price_impact_bps.abs() as f64;
        
        let total_price_impact = price_impact_a + price_impact_b;
        let volatility_factor = market_volatility * 100.0; // Convert to basis points
        
        let risk_score = if total_price_impact > 1000.0 || volatility_factor > 500.0 {
            RiskScore::Extreme
        } else if total_price_impact > 500.0 || volatility_factor > 200.0 {
            RiskScore::High
        } else if total_price_impact > 200.0 || volatility_factor > 100.0 {
            RiskScore::Medium
        } else {
            RiskScore::Low
        };
        
        Ok(risk_score)
    }
    
    pub fn calculate_slippage_risk(quote: &SwapQuote, max_slippage_bps: u32) -> f64 {
        let current_slippage = quote.price_impact_bps.abs() as u32;
        if current_slippage > max_slippage_bps {
            (current_slippage - max_slippage_bps) as f64 / max_slippage_bps as f64
        } else {
            0.0
        }
    }
    
    pub fn is_acceptable_risk(risk_score: &RiskScore, max_acceptable: &RiskScore) -> bool {
        (*risk_score) as u8 <= (*max_acceptable) as u8
    }
}
