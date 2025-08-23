//! Price analysis and calculations

use crate::shared::types::{Token, Price};
use super::PriceData;

/// Analyzes price data and trends
pub struct PriceAnalyzer;

impl PriceAnalyzer {
    pub fn calculate_price_change(
        &self,
        old_price: &PriceData,
        new_price: &PriceData,
    ) -> f64 {
        if old_price.price.value > 0.0 {
            ((new_price.price.value - old_price.price.value) / old_price.price.value) * 100.0
        } else {
            0.0
        }
    }

    pub fn is_significant_change(&self, change_percentage: f64, threshold: f64) -> bool {
        change_percentage.abs() >= threshold
    }
}
