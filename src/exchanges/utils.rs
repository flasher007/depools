use anyhow::Result;
use solana_sdk::pubkey::Pubkey;

pub fn format_pool_address(address: &Pubkey) -> String {
    format!("{}...{}", &address.to_string()[..8], &address.to_string()[address.to_string().len()-8..])
}

pub fn validate_pool_address(address: &str) -> Result<Pubkey> {
    address.parse().map_err(|e| anyhow::anyhow!("Invalid pool address: {}", e))
}

pub fn calculate_price_impact(amount_in: u64, reserve_in: u64, reserve_out: u64) -> f64 {
    if reserve_in == 0 || reserve_out == 0 {
        return 0.0;
    }
    
    let k = reserve_in as f64 * reserve_out as f64;
    let new_reserve_in = reserve_in as f64 + amount_in as f64;
    let new_reserve_out = k / new_reserve_in;
    let price_impact = (reserve_out as f64 - new_reserve_out) / reserve_out as f64;
    
    price_impact * 10000.0 // Convert to basis points
}
