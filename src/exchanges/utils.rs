use anyhow::Result;
use solana_sdk::pubkey::Pubkey;

/// Format pool address for display
pub fn format_pool_address(address: &Pubkey) -> String {
    format!("{}", address)
}

/// Validate pool address string
pub fn validate_pool_address(address: &str) -> anyhow::Result<Pubkey> {
    address.parse().map_err(|e| anyhow::anyhow!("Invalid pool address: {}", e))
}

/// Calculate price impact for a swap
pub fn calculate_price_impact(amount_in: u64, reserve_in: u64, reserve_out: u64) -> f64 {
    let amount_in_f = amount_in as f64;
    let reserve_in_f = reserve_in as f64;
    (amount_in_f / (reserve_in_f + amount_in_f)) * 100.0
}

/// Convert lamports to SOL (9 decimals)
pub fn lamports_to_sol(lamports: u64) -> f64 {
    lamports as f64 / 1_000_000_000.0
}

/// Convert lamports to USDC (6 decimals)
pub fn lamports_to_usdc(lamports: u64) -> f64 {
    lamports as f64 / 1_000_000.0
}

/// Format SOL amount with 6 decimal places
pub fn format_sol(amount: f64) -> String {
    format!("{:.6} SOL", amount)
}

/// Format USDC amount with 3 decimal places
pub fn format_usdc(amount: f64) -> String {
    format!("{:.3} USDC", amount)
}

/// Format large numbers with commas for readability
pub fn format_large_number(num: u64) -> String {
    let num_str = num.to_string();
    let mut result = String::new();
    let mut count = 0;
    
    for (i, ch) in num_str.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.push(',');
        }
        result.push(ch);
        count += 1;
    }
    
    result.chars().rev().collect()
}
