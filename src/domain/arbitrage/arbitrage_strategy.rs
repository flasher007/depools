//! Arbitrage strategies and their implementations

use crate::shared::types::{Amount, Token};
use crate::domain::dex::{DexType, SwapQuote, PoolInfo};
use crate::shared::errors::ArbitrageError;
use std::collections::HashMap;

/// Types of arbitrage strategies
#[derive(Debug, Clone, PartialEq)]
pub enum StrategyType {
    TwoHop,      // A -> B -> A
    Triangle,    // A -> B -> C -> A
    MultiDex,   // Same pair across different DEXes
    FlashLoan,  // Using flash loans for arbitrage
}

impl StrategyType {
    pub fn as_str(&self) -> &'static str {
        match self {
            StrategyType::TwoHop => "Two-Hop",
            StrategyType::Triangle => "Triangle",
            StrategyType::MultiDex => "Multi-DEX",
            StrategyType::FlashLoan => "Flash Loan",
        }
    }
}

/// Trait for arbitrage strategies
pub trait ArbitrageStrategy {
    fn strategy_type(&self) -> StrategyType;
    async fn calculate_profit(&self, amount_in: Amount) -> Result<f64, ArbitrageError>;
    fn validate_route(&self) -> Result<bool, ArbitrageError>;
    fn get_route_description(&self) -> String;
    fn get_required_tokens(&self) -> Vec<Token>;
}

/// Two-hop arbitrage strategy implementation
/// A -> B -> A (same token pair, different DEXes)
pub struct TwoHopStrategy {
    token_a: Token,
    token_b: Token,
    dex_1: DexType,
    dex_2: DexType,
    route: Vec<String>,
    min_profit_threshold: f64,
}

impl TwoHopStrategy {
    pub fn new(
        token_a: Token,
        token_b: Token,
        dex_1: DexType,
        dex_2: DexType,
        min_profit_threshold: f64,
    ) -> Self {
        let route = vec![
            format!("{} -> {} (via {})", token_a.symbol, token_b.symbol, dex_1.as_str()),
            format!("{} -> {} (via {})", token_b.symbol, token_a.symbol, dex_2.as_str()),
        ];
        
        Self {
            token_a,
            token_b,
            dex_1,
            dex_2,
            route,
            min_profit_threshold,
        }
    }
    
    /// Calculate optimal amount for arbitrage
    pub fn calculate_optimal_amount(&self, pool_1: &PoolInfo, pool_2: &PoolInfo) -> Amount {
        // Use the smaller reserve to avoid excessive price impact
        let reserve_1 = pool_1.reserve_a.value.min(pool_1.reserve_b.value);
        let reserve_2 = pool_2.reserve_a.value.min(pool_2.reserve_b.value);
        
        let optimal = reserve_1.min(reserve_2) as f64 * 0.01; // 1% of smaller reserve
        
        Amount::new(optimal as u64, self.token_a.decimals)
    }
    
    /// Calculate price impact for given amount
    pub fn calculate_price_impact(&self, amount: Amount, pool: &PoolInfo) -> f64 {
        let reserve_a = pool.reserve_a.value as f64;
        let reserve_b = pool.reserve_b.value as f64;
        
        // Simple constant product formula price impact
        let k = reserve_a * reserve_b;
        let new_reserve_a = reserve_a + amount.value as f64;
        let new_reserve_b = k / new_reserve_a;
        let price_impact = (reserve_b - new_reserve_b) / reserve_b;
        
        price_impact * 100.0 // Convert to percentage
    }
}

impl ArbitrageStrategy for TwoHopStrategy {
    fn strategy_type(&self) -> StrategyType {
        StrategyType::TwoHop
    }

    async fn calculate_profit(&self, amount_in: Amount) -> Result<f64, ArbitrageError> {
        // TODO: Implement actual profit calculation with real DEX data
        // For now, return a placeholder calculation
        let base_profit = amount_in.value as f64 * 0.001; // 0.1% base profit
        Ok(base_profit)
    }

    fn validate_route(&self) -> Result<bool, ArbitrageError> {
        // Validate that we have different DEXes
        if self.dex_1 == self.dex_2 {
            return Err(ArbitrageError::InvalidRoute);
        }
        
        // Validate that tokens are different
        if self.token_a.mint == self.token_b.mint {
            return Err(ArbitrageError::InvalidRoute);
        }
        
        Ok(true)
    }
    
    fn get_route_description(&self) -> String {
        self.route.join(" -> ")
    }

    fn get_required_tokens(&self) -> Vec<Token> {
        vec![self.token_a.clone(), self.token_b.clone()]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared::types::Token;
    use solana_sdk::pubkey::Pubkey;

    fn create_test_tokens() -> (Token, Token) {
        let sol = Token {
            mint: Pubkey::new_unique(),
            symbol: "SOL".to_string(),
            decimals: 9,
            name: Some("Solana".to_string()),
        };
        
        let usdc = Token {
            mint: Pubkey::new_unique(),
            symbol: "USDC".to_string(),
            decimals: 6,
            name: Some("USD Coin".to_string()),
        };
        
        (sol, usdc)
    }

    #[test]
    fn test_two_hop_strategy_creation() {
        let (sol, usdc) = create_test_tokens();
        let strategy = TwoHopStrategy::new(
            sol.clone(),
            usdc.clone(),
            DexType::OrcaWhirlpool,
            DexType::RaydiumV4,
            0.001, // 0.1% min profit
        );
        
        assert_eq!(strategy.strategy_type(), StrategyType::TwoHop);
        assert_eq!(strategy.token_a.symbol, "SOL");
        assert_eq!(strategy.token_b.symbol, "USDC");
        assert_eq!(strategy.dex_1, DexType::OrcaWhirlpool);
        assert_eq!(strategy.dex_2, DexType::RaydiumV4);
        assert_eq!(strategy.min_profit_threshold, 0.001);
    }

    #[test]
    fn test_two_hop_route_validation() {
        let (sol, usdc) = create_test_tokens();
        
        // Valid route: different DEXes, different tokens
        let valid_strategy = TwoHopStrategy::new(
            sol.clone(),
            usdc.clone(),
            DexType::OrcaWhirlpool,
            DexType::RaydiumV4,
            0.001,
        );
        assert!(valid_strategy.validate_route().is_ok());
        
        // Invalid route: same DEX
        let invalid_strategy = TwoHopStrategy::new(
            sol.clone(),
            usdc.clone(),
            DexType::OrcaWhirlpool,
            DexType::OrcaWhirlpool,
            0.001,
        );
        assert!(invalid_strategy.validate_route().is_err());
    }

    #[test]
    fn test_two_hop_route_description() {
        let (sol, usdc) = create_test_tokens();
        let strategy = TwoHopStrategy::new(
            sol.clone(),
            usdc.clone(),
            DexType::OrcaWhirlpool,
            DexType::RaydiumV4,
            0.001,
        );
        
        let description = strategy.get_route_description();
        assert!(description.contains("SOL -> USDC"));
        assert!(description.contains("USDC -> SOL"));
        assert!(description.contains("Orca Whirlpool"));
        assert!(description.contains("Raydium V4"));
    }

    #[test]
    fn test_two_hop_required_tokens() {
        let (sol, usdc) = create_test_tokens();
        let strategy = TwoHopStrategy::new(
            sol.clone(),
            usdc.clone(),
            DexType::OrcaWhirlpool,
            DexType::RaydiumV4,
            0.001,
        );
        
        let required_tokens = strategy.get_required_tokens();
        assert_eq!(required_tokens.len(), 2);
        assert_eq!(required_tokens[0].symbol, "SOL");
        assert_eq!(required_tokens[1].symbol, "USDC");
    }
}
