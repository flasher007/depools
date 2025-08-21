use anyhow::Result;
use crate::exchanges::types::{ArbitrageOpportunity, RiskScore};
use crate::opportunity::scanner::OpportunityScanner;
use std::sync::Arc;
use std::any::Any;

pub struct ArbitrageEngine {
    scanner: Arc<dyn OpportunityScanner>,
    min_profit_bps: i32,
    max_risk_score: RiskScore,
}

impl ArbitrageEngine {
    pub fn new(scanner: Arc<dyn OpportunityScanner>, min_profit_bps: i32) -> Self {
        Self {
            scanner,
            min_profit_bps,
            max_risk_score: RiskScore::Medium,
        }
    }

    pub async fn find_opportunities(&self, pool_addresses: &[String]) -> Result<Vec<ArbitrageOpportunity>> {
        let opportunities = self.scanner.scan_opportunities(pool_addresses)?;
        
        // Фильтруем по минимальной прибыльности и риску
        let filtered = opportunities
            .into_iter()
            .filter(|opp| {
                opp.profit_bps >= self.min_profit_bps &&
                opp.risk_score as u8 <= self.max_risk_score as u8
            })
            .collect();
        
        Ok(filtered)
    }

    pub async fn find_opportunities_async(&self, pool_addresses: &[String]) -> Result<Vec<ArbitrageOpportunity>> {
        // Используем async версию сканера
        if let Some(scanner) = self.scanner.as_any().downcast_ref::<crate::opportunity::scanner::CrossDexScanner>() {
            let opportunities = scanner.scan_opportunities_async(pool_addresses).await?;
            
            // Фильтруем по минимальной прибыльности и риску
            let filtered = opportunities
                .into_iter()
                .filter(|opp| {
                    opp.profit_bps >= self.min_profit_bps &&
                    opp.risk_score as u8 <= self.max_risk_score as u8
                })
                .collect();
            
            Ok(filtered)
        } else {
            // Fallback к синхронной версии
            self.find_opportunities(pool_addresses).await
        }
    }

    pub fn set_min_profit_threshold(&mut self, bps: i32) {
        self.min_profit_bps = bps;
    }

    pub fn set_max_risk_score(&mut self, risk_score: RiskScore) {
        self.max_risk_score = risk_score;
    }
}
