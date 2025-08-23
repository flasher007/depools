//! Arbitrage domain - core arbitrage logic and strategies

mod arbitrage_engine;
pub mod arbitrage_strategy;
mod arbitrage_route;
mod profit_calculator;

pub use arbitrage_engine::ArbitrageEngine;
pub use arbitrage_strategy::{ArbitrageStrategy, StrategyType};
pub use arbitrage_route::{ArbitrageRoute, RouteStep};
pub use profit_calculator::ProfitCalculator;

use crate::shared::types::{Token, Amount, Price};
use crate::shared::errors::ArbitrageError;

/// Arbitrage opportunity representation
#[derive(Debug, Clone)]
pub struct ArbitrageOpportunity {
    pub id: String,
    pub route: ArbitrageRoute,
    pub expected_profit: Amount,
    pub profit_percentage: f64,
    pub risk_score: f64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Arbitrage execution result
#[derive(Debug, Clone)]
pub struct ArbitrageResult {
    pub opportunity: ArbitrageOpportunity,
    pub executed: bool,
    pub actual_profit: Option<Amount>,
    pub transaction_signature: Option<String>,
    pub error: Option<ArbitrageError>,
}
