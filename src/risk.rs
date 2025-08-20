// src/risk.rs
pub fn min_out(amount_out_ui: f64, slippage_bps: u32) -> f64 {
    let sl = slippage_bps as f64 / 10_000.0;
    amount_out_ui * (1.0 - sl)
}
