//! Arbitrage route representation and management

use crate::shared::types::{Amount, Token};

/// Single step in an arbitrage route
#[derive(Debug, Clone)]
pub struct RouteStep {
    pub dex_name: String,
    pub token_in: Token,
    pub token_out: Token,
    pub amount_in: Amount,
    pub expected_amount_out: Amount,
    pub pool_id: String,
}

/// Complete arbitrage route
#[derive(Debug, Clone)]
pub struct ArbitrageRoute {
    pub id: String,
    pub steps: Vec<RouteStep>,
    pub total_profit: Amount,
    pub profit_percentage: f64,
    pub estimated_gas: u64,
    pub risk_score: f64,
}

impl ArbitrageRoute {
    pub fn new(id: String) -> Self {
        Self {
            id,
            steps: Vec::new(),
            total_profit: Amount::new(0, 9),
            profit_percentage: 0.0,
            estimated_gas: 0,
            risk_score: 0.0,
        }
    }

    pub fn add_step(&mut self, step: RouteStep) {
        self.steps.push(step);
    }

    pub fn calculate_total_profit(&mut self) -> Result<(), String> {
        // TODO: Implement profit calculation
        Ok(())
    }

    pub fn validate_route(&self) -> Result<bool, String> {
        if self.steps.is_empty() {
            return Err("Route has no steps".to_string());
        }
        
        // Check if route forms a cycle
        if self.steps.first().unwrap().token_in != self.steps.last().unwrap().token_out {
            return Err("Route does not form a cycle".to_string());
        }
        
        Ok(true)
    }
}
