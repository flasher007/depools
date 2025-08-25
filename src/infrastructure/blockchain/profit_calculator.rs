//! Real profit calculator using blockchain data

use std::sync::Arc;
use crate::shared::types::{Amount, Token};
use crate::domain::dex::{PoolInfo, DexType};
use crate::shared::errors::AppError;
use super::vault_reader::VaultReader;

/// Real profit calculator
#[derive(Clone)]
pub struct RealProfitCalculator {
    vault_reader: Arc<VaultReader>,
}

impl RealProfitCalculator {
    /// Create new profit calculator
    pub fn new(rpc_url: String) -> Self {
        Self {
            vault_reader: Arc::new(VaultReader::new(rpc_url)),
        }
    }
    

    
    /// Calculate real profit for TwoHop arbitrage
    pub async fn calculate_two_hop_profit(
        &self,
        token_a: &Token,
        token_b: &Token,
        amount_in: Amount,
        pool_1: &PoolInfo,
        pool_2: &PoolInfo,
    ) -> Result<ProfitCalculation, AppError> {
        // Validate pools
        if pool_1.dex_type == pool_2.dex_type {
            return Err(AppError::BlockchainError("Pools must be on different DEXes".to_string()));
        }
        
        // Calculate optimal amount based on liquidity
        let optimal_amount = self.calculate_optimal_amount(pool_1, pool_2, amount_in);
        
        // Calculate amount out from first swap
        let amount_out_1 = self.calculate_swap_output(
            optimal_amount.value,
            pool_1.reserve_a.value,
            pool_1.reserve_b.value,
            pool_1.fee_rate,
        )?;
        
        // Calculate amount out from second swap
        let amount_out_2 = self.calculate_swap_output(
            amount_out_1,
            pool_2.reserve_b.value,
            pool_2.reserve_a.value,
            pool_2.fee_rate,
        )?;
        
        // Calculate profit
        let profit = if amount_out_2 > optimal_amount.value {
            amount_out_2 - optimal_amount.value
        } else {
            0
        };
        
        let profit_percentage = if optimal_amount.value > 0 {
            (profit as f64 / optimal_amount.value as f64) * 100.0
        } else {
            0.0
        };
        
        // Calculate gas costs (estimated)
        let gas_cost = self.estimate_gas_cost(pool_1.dex_type, pool_2.dex_type);
        
        // Calculate net profit
        let net_profit = if profit > gas_cost {
            profit - gas_cost
        } else {
            0
        };
        
        Ok(ProfitCalculation {
            input_amount: optimal_amount,
            intermediate_amount: Amount::new(amount_out_1, token_b.decimals),
            output_amount: Amount::new(amount_out_2, token_a.decimals),
            gross_profit: Amount::new(profit, token_a.decimals),
            gas_cost: Amount::new(gas_cost, 9), // SOL
            net_profit: Amount::new(net_profit, token_a.decimals),
            profit_percentage,
            is_profitable: net_profit > 0,
            route: format!("{} -> {} -> {}", 
                pool_1.dex_type.as_str(), 
                pool_2.dex_type.as_str(),
                pool_1.dex_type.as_str()
            ),
        })
    }
    
    /// Calculate optimal amount for arbitrage
    fn calculate_optimal_amount(&self, pool_1: &PoolInfo, pool_2: &PoolInfo, max_amount: Amount) -> Amount {
        // Use the smaller reserve to avoid excessive price impact
        let reserve_1 = pool_1.reserve_a.value.min(pool_1.reserve_b.value);
        let reserve_2 = pool_2.reserve_a.value.min(pool_2.reserve_b.value);
        
        let optimal = reserve_1.min(reserve_2) as f64 * 0.01; // 1% of smaller reserve
        
        // Cap at max amount
        let optimal = optimal.min(max_amount.value as f64);
        
        Amount::new(optimal as u64, max_amount.decimals)
    }
    
    /// Calculate swap output using constant product formula
    fn calculate_swap_output(
        &self,
        amount_in: u64,
        reserve_in: u64,
        reserve_out: u64,
        fee_rate: f64,
    ) -> Result<u64, AppError> {
        if reserve_in == 0 || reserve_out == 0 {
            return Err(AppError::BlockchainError("Invalid reserves".to_string()));
        }
        
        // Apply fee
        let amount_in_with_fee = (amount_in as f64) * (1.0 - fee_rate);
        
        // Constant product formula: (x + dx) * (y - dy) = x * y
        // dy = (y * dx) / (x + dx)
        let amount_out = (reserve_out as f64 * amount_in_with_fee) / (reserve_in as f64 + amount_in_with_fee);
        
        Ok(amount_out as u64)
    }
    
    /// Estimate gas cost for arbitrage
    fn estimate_gas_cost(&self, dex_1: DexType, dex_2: DexType) -> u64 {
        // Base cost for two swaps
        let mut total_cost = 200_000; // 200k lamports base
        
        // Add DEX-specific costs
        match dex_1 {
            DexType::OrcaWhirlpool => total_cost += 50_000,
            DexType::RaydiumAMM => total_cost += 40_000,
            _ => total_cost += 60_000,
        }
        
        match dex_2 {
            DexType::OrcaWhirlpool => total_cost += 50_000,
            DexType::RaydiumAMM => total_cost += 40_000,
            _ => total_cost += 60_000,
        }
        
        total_cost
    }
    
    /// Get real-time pool data for calculation
    pub async fn get_real_pool_data(&self, pool_address: &str) -> Result<PoolInfo, AppError> {
        // This would integrate with real-time data sources
        // For now, return mock data
        Err(AppError::BlockchainError("Real-time pool data not yet implemented".to_string()))
    }
    
    /// Calculate arbitrage profit between two pools
    pub async fn calculate_arbitrage_profit(
        &self,
        pool_1: &PoolInfo,
        pool_2: &PoolInfo,
        amount_in: Amount,
    ) -> Result<ProfitCalculation, AppError> {
        // Ensure pools have the same token pair
        if !self.is_same_token_pair(pool_1, pool_2) {
            return Err(AppError::BlockchainError("Pools must have the same token pair".to_string()));
        }
        
        // Calculate optimal amount
        let optimal_amount = self.calculate_optimal_amount(pool_1, pool_2, amount_in);
        
        // First swap: Token A -> Token B on pool_1
        let intermediate_amount = self.calculate_swap_output(
            optimal_amount.value,
            pool_1.reserve_a.value,
            pool_1.reserve_b.value,
            pool_1.fee_rate,
        )?;
        
        // Second swap: Token B -> Token A on pool_2
        let output_amount = self.calculate_swap_output(
            intermediate_amount,
            pool_2.reserve_b.value,
            pool_2.reserve_a.value,
            pool_2.fee_rate,
        )?;
        
        // Calculate profits
        let gross_profit = if output_amount > optimal_amount.value {
            Amount::new(output_amount - optimal_amount.value, optimal_amount.decimals)
        } else {
            Amount::new(0, optimal_amount.decimals)
        };
        
        // Estimate gas cost
        let gas_cost_lamports = self.estimate_gas_cost(pool_1.dex_type.clone(), pool_2.dex_type.clone());
        let gas_cost = Amount::new(gas_cost_lamports, 9); // SOL has 9 decimals
        
        // Convert gas cost to token A equivalent
        let gas_cost_token_a = (gas_cost_lamports as f64) / 1_000_000_000.0; // Convert lamports to SOL
        let gas_cost_token_a_amount = Amount::new((gas_cost_token_a * 1_000_000_000.0) as u64, 9);
        
        // Calculate net profit
        let net_profit = if gross_profit.value > gas_cost_token_a_amount.value {
            Amount::new(gross_profit.value - gas_cost_token_a_amount.value, gross_profit.decimals)
        } else {
            Amount::new(0, gross_profit.decimals)
        };
        
        // Calculate profit percentage
        let profit_percentage = if optimal_amount.value > 0 {
            (net_profit.value as f64 / optimal_amount.value as f64) * 100.0
        } else {
            0.0
        };
        
        // Determine if profitable
        let is_profitable = net_profit.value > 0;
        
        // Create route description
        let route = format!(
            "{} -> {} -> {} (via {} -> {})",
            pool_1.token_a.symbol,
            pool_1.token_b.symbol,
            pool_1.token_a.symbol,
            pool_1.dex_type.as_str(),
            pool_2.dex_type.as_str()
        );
        
        Ok(ProfitCalculation {
            input_amount: optimal_amount.clone(),
            intermediate_amount: Amount::new(intermediate_amount, pool_1.token_b.decimals),
            output_amount: Amount::new(output_amount, optimal_amount.decimals),
            gross_profit,
            gas_cost: gas_cost_token_a_amount,
            net_profit,
            profit_percentage,
            is_profitable,
            route,
        })
    }
    
    /// Check if two pools have the same token pair
    fn is_same_token_pair(&self, pool_1: &PoolInfo, pool_2: &PoolInfo) -> bool {
        (pool_1.token_a.mint == pool_2.token_a.mint && pool_1.token_b.mint == pool_2.token_b.mint) ||
        (pool_1.token_a.mint == pool_2.token_b.mint && pool_1.token_b.mint == pool_2.token_a.mint)
    }
}

/// Profit calculation result
#[derive(Debug, Clone)]
pub struct ProfitCalculation {
    pub input_amount: Amount,
    pub intermediate_amount: Amount,
    pub output_amount: Amount,
    pub gross_profit: Amount,
    pub gas_cost: Amount,
    pub net_profit: Amount,
    pub profit_percentage: f64,
    pub is_profitable: bool,
    pub route: String,
}

impl ProfitCalculation {
    /// Print profit calculation summary
    pub fn print_summary(&self) {
        println!("💰 Profit Calculation Summary:");
        println!("   Route: {}", self.route);
        println!("   Input: {} tokens", self.input_amount.value);
        println!("   Intermediate: {} tokens", self.intermediate_amount.value);
        println!("   Output: {} tokens", self.output_amount.value);
        println!("   Gross Profit: {} tokens", self.gross_profit.value);
        println!("   Gas Cost: {} lamports", self.gas_cost.value);
        println!("   Net Profit: {} tokens", self.net_profit.value);
        println!("   Profit %: {:.4}%", self.profit_percentage);
        println!("   Profitable: {}", if self.is_profitable { "✅ YES" } else { "❌ NO" });
    }
}
