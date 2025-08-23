//! Price domain - price monitoring and analysis

mod price_monitor;
mod price_feed;
mod price_analyzer;

pub use price_monitor::PriceMonitor;
pub use price_feed::PriceFeed;
pub use price_analyzer::PriceAnalyzer;

use crate::shared::types::{Token, Price};
use chrono::{DateTime, Utc};

/// Price data point
#[derive(Debug, Clone)]
pub struct PriceData {
    pub token: Token,
    pub price: Price,
    pub timestamp: DateTime<Utc>,
    pub source: String,
    pub volume_24h: Option<f64>,
}

/// Price change event
#[derive(Debug, Clone)]
pub struct PriceChangeEvent {
    pub token: Token,
    pub old_price: Price,
    pub new_price: Price,
    pub change_percentage: f64,
    pub timestamp: DateTime<Utc>,
    pub volume: Option<f64>,
}

/// Price monitoring configuration
#[derive(Debug, Clone)]
pub struct PriceMonitorConfig {
    pub update_interval_ms: u64,
    pub price_change_threshold: f64,
    pub max_price_age_ms: u64,
    pub sources: Vec<String>,
}
