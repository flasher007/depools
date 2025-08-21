pub mod cross_dex_scanner;
pub mod base_scanner;

pub use cross_dex_scanner::CrossDexScanner;

use anyhow::Result;
use crate::exchanges::types::ArbitrageOpportunity;
use std::any::Any;

pub trait OpportunityScanner: Send + Sync {
    fn scan_opportunities(&self, pool_addresses: &[String]) -> Result<Vec<ArbitrageOpportunity>>;
    fn calculate_profitability(&self, quote_a: &crate::exchanges::types::SwapQuote, quote_b: &crate::exchanges::types::SwapQuote) -> Result<f64>;
    fn as_any(&self) -> &dyn Any;
}
