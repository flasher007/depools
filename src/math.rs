// src/math.rs
use anyhow::Result;
use crate::exchanges::types::{PnlBreakdown, SwapQuote, SwapRoute};
use crate::exchanges::utils::{lamports_to_sol, format_sol};

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
pub fn calculate_min_out(amount_out: u64, slippage_bps: u32) -> u64 {
    let slippage_multiplier = (10000 - slippage_bps) as f64 / 10000.0;
    (amount_out as f64 * slippage_multiplier) as u64
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

/// Calculate gross profit from arbitrage opportunity
/// For arbitrage: SOL → USDC (quote_a) → SOL (quote_b)
/// Profit = final_sol_amount - initial_sol_amount
pub fn calculate_gross_profit(quote_a: &SwapQuote, quote_b: &SwapQuote) -> u64 {
    // quote_a.amount_in = initial SOL amount
    // quote_b.amount_out = final SOL amount
    if quote_b.amount_out > quote_a.amount_in {
        quote_b.amount_out - quote_a.amount_in
    } else {
        0
    }
}



/// Calculate priority fee for the transaction
pub fn calculate_priority_fee(priority_fee_lamports: u64, compute_units: u32) -> u64 {
    (priority_fee_lamports * compute_units as u64) / 1_000_000
}

/// Calculate rent fee for temporary accounts (approximate)
pub fn calculate_rent_fee(account_count: u32) -> u64 {
    // Solana rent is ~0.00203928 SOL per account per epoch
    // For arbitrage, we typically need 2-4 temporary accounts
    let rent_per_account = 2_039_280; // in lamports
    (account_count as u64 * rent_per_account) / 1_000_000_000
}

/// Calculate complete PnL breakdown for arbitrage opportunity
/// Pool fees are already accounted for within the AMM swap formula (via dx')
/// and should not be subtracted again here
pub fn calculate_pnl_breakdown(
    quote_a: &SwapQuote,
    quote_b: &SwapQuote,
    priority_fee_lamports: u64,
    slippage_bps: u32,
) -> PnlBreakdown {
    let gross_profit = calculate_gross_profit(quote_a, quote_b);
    
    // Estimate compute units for arbitrage transaction
    let estimated_compute_units = 200_000; // Typical for complex swaps
    let priority_fee = calculate_priority_fee(priority_fee_lamports, estimated_compute_units);
    
    // Estimate account count for arbitrage
    let estimated_accounts = 4; // 2 pools + 2 temporary accounts
    let rent_fee = calculate_rent_fee(estimated_accounts);
    
    let net_profit = if gross_profit > (priority_fee + rent_fee) {
        gross_profit - priority_fee - rent_fee
    } else {
        0
    };
    
    let is_profitable = net_profit > 0;
    
    PnlBreakdown {
        gross_profit,
        priority_fee,
        rent_fee,
        net_profit,
        is_profitable,
    }
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
