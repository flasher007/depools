// src/math.rs
use anyhow::Result;

/// Calculate spread between two prices in basis points
pub fn calculate_spread_bps(price_a: f64, price_b: f64) -> Result<u32> {
    if price_a <= 0.0 || price_b <= 0.0 {
        return Err(anyhow::anyhow!("Invalid prices: price_a={}, price_b={}", price_a, price_b));
    }
    
    let spread = if price_a > price_b {
        ((price_a - price_b) / price_b) * 10000.0
    } else {
        ((price_b - price_a) / price_a) * 10000.0
    };
    
    Ok(spread as u32)
}

/// Calculate minimum output amount with slippage protection
pub fn calculate_min_out(amount_out: f64, slippage_bps: u32) -> f64 {
    let slippage_multiplier = 1.0 - (slippage_bps as f64 / 10000.0);
    amount_out * slippage_multiplier
}

/// Calculate effective price after fees
pub fn calculate_effective_price(base_price: f64, fee_bps: u32) -> f64 {
    let fee_multiplier = 1.0 + (fee_bps as f64 / 10000.0);
    base_price * fee_multiplier
}

/// Calculate arbitrage profit in basis points
pub fn calculate_arbitrage_profit_bps(price_a: f64, price_b: f64, fee_a_bps: u32, fee_b_bps: u32) -> Result<i32> {
    let spread_bps = calculate_spread_bps(price_a, price_b)? as i32;
    
    // Calculate total fees
    let total_fees_bps = fee_a_bps as i32 + fee_b_bps as i32;
    
    // Net profit = spread - fees
    let net_profit_bps = spread_bps - total_fees_bps;
    
    Ok(net_profit_bps)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_calculate_spread_bps() {
        let spread = calculate_spread_bps(1.0, 1.01).unwrap();
        assert_eq!(spread, 100); // 1% = 100 bps
        
        let spread = calculate_spread_bps(1.01, 1.0).unwrap();
        assert_eq!(spread, 100); // 1% = 100 bps
    }
    
    #[test]
    fn test_calculate_min_out() {
        let min_out = calculate_min_out(100.0, 100); // 1% slippage
        assert_eq!(min_out, 99.0);
    }
    
    #[test]
    fn test_calculate_arbitrage_profit_bps() {
        let profit = calculate_arbitrage_profit_bps(1.0, 1.01, 25, 25).unwrap();
        assert_eq!(profit, 50); // 100 bps spread - 50 bps fees = 50 bps profit
    }
}
