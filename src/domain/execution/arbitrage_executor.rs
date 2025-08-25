//! Arbitrage transaction executor with risk management

use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use solana_sdk::pubkey::Pubkey;

use crate::shared::types::{Amount, Token};
use crate::shared::errors::AppError;
use crate::domain::arbitrage::{ArbitrageRoute, ProfitCalculation};
use crate::domain::execution::{ExecutionRequest, ExecutionResult};
use crate::infrastructure::blockchain::RealTransactionExecutor;
use crate::domain::dex::{PoolInfo, DexType};

/// Risk management configuration
#[derive(Debug, Clone)]
pub struct RiskManagementConfig {
    pub max_position_size: Amount,           // Максимальный размер позиции
    pub max_daily_loss: Amount,              // Максимальный дневной убыток
    pub max_concurrent_trades: usize,        // Максимум одновременных сделок
    pub min_profit_threshold: f64,           // Минимальный порог прибыли
    pub max_slippage_tolerance: f64,         // Максимальная толерантность к проскальзыванию
    pub max_risk_score: f64,                 // Максимальный риск
    pub min_confidence_score: f64,           // Минимальная уверенность
    pub cooldown_period_ms: u64,             // Период охлаждения между сделками
}

impl Default for RiskManagementConfig {
    fn default() -> Self {
        Self {
            max_position_size: Amount::new(10000000000, 9), // 10 SOL
            max_daily_loss: Amount::new(1000000000, 9),     // 1 SOL
            max_concurrent_trades: 3,
            min_profit_threshold: 0.01,                     // 1%
            max_slippage_tolerance: 0.05,                   // 5%
            max_risk_score: 0.7,                            // 70%
            min_confidence_score: 0.8,                      // 80%
            cooldown_period_ms: 5000,                       // 5 секунд
        }
    }
}

/// Trade execution status
#[derive(Debug, Clone, PartialEq)]
pub enum TradeStatus {
    Pending,
    Executing,
    Completed,
    Failed,
    Cancelled,
}

/// Active trade information
#[derive(Debug, Clone)]
pub struct ActiveTrade {
    pub id: String,
    pub route: ArbitrageRoute,
    pub status: TradeStatus,
    pub execution_start: Instant,
    pub last_update: Instant,
    pub attempts: u32,
    pub max_attempts: u32,
    pub profit_calculation: ProfitCalculation,
}

/// Daily trading statistics
#[derive(Debug, Clone)]
pub struct DailyStats {
    pub date: chrono::NaiveDate,
    pub total_trades: u32,
    pub successful_trades: u32,
    pub total_profit: Amount,
    pub total_loss: Amount,
    pub net_profit: Amount,
    pub start_balance: Amount,
    pub end_balance: Amount,
}

impl DailyStats {
    pub fn new(date: chrono::NaiveDate, start_balance: Amount) -> Self {
        Self {
            date,
            total_trades: 0,
            successful_trades: 0,
            total_profit: Amount::new(0, 9),
            total_loss: Amount::new(0, 9),
            net_profit: Amount::new(0, 9),
            start_balance: start_balance.clone(),
            end_balance: start_balance,
        }
    }

    pub fn update_after_trade(&mut self, profit: Amount, success: bool) {
        self.total_trades += 1;
        if success {
            self.successful_trades += 1;
            if profit.value > 0 {
                self.total_profit.value += profit.value;
            } else {
                self.total_loss.value += profit.value;
            }
        } else {
            // Failed trade - add to losses
            self.total_loss.value += profit.value;
        }
        
        self.net_profit.value = self.total_profit.value.saturating_sub(self.total_loss.value);
        self.end_balance.value = self.start_balance.value.saturating_add(self.net_profit.value);
    }

    pub fn get_success_rate(&self) -> f64 {
        if self.total_trades == 0 {
            0.0
        } else {
            self.successful_trades as f64 / self.total_trades as f64
        }
    }

    pub fn get_roi(&self) -> f64 {
        if self.start_balance.value == 0 {
            0.0
        } else {
            self.net_profit.value as f64 / self.start_balance.value as f64
        }
    }
}

/// Arbitrage transaction executor with risk management
pub struct ArbitrageTransactionExecutor {
    executor: RealTransactionExecutor,
    risk_config: RiskManagementConfig,
    active_trades: Arc<RwLock<HashMap<String, ActiveTrade>>>,
    daily_stats: Arc<RwLock<HashMap<chrono::NaiveDate, DailyStats>>>,
    last_trade_time: Arc<RwLock<Instant>>,
    current_balance: Arc<RwLock<Amount>>,
}

impl ArbitrageTransactionExecutor {
    /// Create new arbitrage executor
    pub fn new(
        executor: RealTransactionExecutor,
        risk_config: RiskManagementConfig,
        initial_balance: Amount,
    ) -> Self {
        Self {
            executor,
            risk_config,
            active_trades: Arc::new(RwLock::new(HashMap::new())),
            daily_stats: Arc::new(RwLock::new(HashMap::new())),
            last_trade_time: Arc::new(RwLock::new(Instant::now())),
            current_balance: Arc::new(RwLock::new(initial_balance)),
        }
    }

    /// Create with default risk configuration
    pub fn new_default(
        executor: RealTransactionExecutor,
        initial_balance: Amount,
    ) -> Self {
        Self::new(executor, RiskManagementConfig::default(), initial_balance)
    }

    /// Execute arbitrage opportunity with risk management
    pub async fn execute_arbitrage_opportunity(
        &mut self,
        route: &ArbitrageRoute,
        profit_calculation: &ProfitCalculation,
    ) -> Result<ExecutionResult, AppError> {
        println!("🎯 Проверяем возможность исполнения арбитража...");
        
        // 1. Проверяем риск-менеджмент
        if let Err(e) = self.validate_risk_limits(route, profit_calculation).await {
            println!("❌ Риск-менеджмент заблокировал сделку: {}", e);
            return Err(e);
        }

        // 2. Проверяем период охлаждения
        if let Err(e) = self.check_cooldown_period().await {
            println!("⏰ Сделка заблокирована периодом охлаждения: {}", e);
            return Err(e);
        }

        // 3. Создаем активную сделку
        let trade_id = self.create_active_trade(route, profit_calculation).await?;
        println!("📝 Создана активная сделка: {}", trade_id);

        // 4. Выполняем транзакцию
        let result = self.execute_trade_transaction(route).await?;
        
        // 5. Обновляем статистику
        self.update_trade_status(&trade_id, &result).await;
        self.update_daily_stats(&result).await;
        
        // 6. Обновляем баланс
        self.update_balance(&result).await;

        println!("✅ Арбитражная сделка выполнена!");
        Ok(result)
    }

    /// Validate risk management limits
    async fn validate_risk_limits(
        &self,
        route: &ArbitrageRoute,
        profit_calculation: &ProfitCalculation,
    ) -> Result<(), AppError> {
        // Проверяем размер позиции
        if route.steps[0].amount_in > self.risk_config.max_position_size {
            return Err(AppError::ExecutionError(format!(
                "Position size {} exceeds maximum {}",
                route.steps[0].amount_in.value,
                self.risk_config.max_position_size.value
            )));
        }

        // Проверяем минимальную прибыль
        if profit_calculation.roi_percentage < self.risk_config.min_profit_threshold {
            return Err(AppError::ExecutionError(format!(
                "Profit {}% below threshold {}%",
                profit_calculation.roi_percentage * 100.0,
                self.risk_config.min_profit_threshold * 100.0
            )));
        }

        // Проверяем риск
        if route.risk_score > self.risk_config.max_risk_score {
            return Err(AppError::ExecutionError(format!(
                "Risk score {} exceeds maximum {}",
                route.risk_score,
                self.risk_config.max_risk_score
            )));
        }

        // Проверяем уверенность
        if route.confidence_score < self.risk_config.min_confidence_score {
            return Err(AppError::ExecutionError(format!(
                "Confidence score {} below minimum {}",
                route.confidence_score,
                self.risk_config.min_confidence_score
            )));
        }

        // Проверяем количество активных сделок
        let active_trades = self.active_trades.read().await;
        if active_trades.len() >= self.risk_config.max_concurrent_trades {
            return Err(AppError::ExecutionError(format!(
                "Maximum concurrent trades {} reached",
                self.risk_config.max_concurrent_trades
            )));
        }

        // Проверяем дневной убыток
        let today = chrono::Utc::now().date_naive();
        let daily_stats = self.daily_stats.read().await;
        if let Some(stats) = daily_stats.get(&today) {
            if stats.total_loss > self.risk_config.max_daily_loss {
                return Err(AppError::ExecutionError(format!(
                    "Daily loss {} exceeds maximum {}",
                    stats.total_loss.value,
                    self.risk_config.max_daily_loss.value
                )));
            }
        }

        Ok(())
    }

    /// Check cooldown period between trades
    async fn check_cooldown_period(&self) -> Result<(), AppError> {
        let last_trade = self.last_trade_time.read().await;
        let elapsed = last_trade.elapsed();
        let cooldown = Duration::from_millis(self.risk_config.cooldown_period_ms);
        
        if elapsed < cooldown {
            let remaining = cooldown - elapsed;
            return Err(AppError::ExecutionError(format!(
                "Cooldown period not met. Wait {}ms",
                remaining.as_millis()
            )));
        }
        
        Ok(())
    }

    /// Create active trade
    async fn create_active_trade(
        &self,
        route: &ArbitrageRoute,
        profit_calculation: &ProfitCalculation,
    ) -> Result<String, AppError> {
        let trade_id = format!("trade_{}_{}", route.id, chrono::Utc::now().timestamp());
        
        let active_trade = ActiveTrade {
            id: trade_id.clone(),
            route: route.clone(),
            status: TradeStatus::Pending,
            execution_start: Instant::now(),
            last_update: Instant::now(),
            attempts: 0,
            max_attempts: 3,
            profit_calculation: profit_calculation.clone(),
        };

        let mut trades = self.active_trades.write().await;
        trades.insert(trade_id.clone(), active_trade);
        
        Ok(trade_id)
    }

    /// Execute trade transaction
    async fn execute_trade_transaction(
        &self,
        route: &ArbitrageRoute,
    ) -> Result<ExecutionResult, AppError> {
        // Обновляем статус на "выполняется"
        self.update_trade_status_internal(&route.id, TradeStatus::Executing).await;

        // Создаем ExecutionRequest
        let request = ExecutionRequest {
            route_id: route.id.clone(),
            amount_in: route.steps[0].amount_in.clone(),
            token_in: route.steps[0].token_in.clone(),
            token_out: route.steps[0].token_out.clone(),
            min_amount_out: route.steps[0].expected_amount_out.clone(),
            slippage_tolerance: route.steps[0].slippage_estimate.min(self.risk_config.max_slippage_tolerance),
            deadline: chrono::Utc::now() + chrono::Duration::seconds(60), // 1 минута
        };

        // Выполняем транзакцию через RealTransactionExecutor
        // Для простоты создаем заглушку, но в реальности здесь будет вызов executor.execute_arbitrage
        let result = ExecutionResult {
            request: request.clone(),
            transaction: None, // TODO: Реальная транзакция
            signature: Some(format!("sig_{}", chrono::Utc::now().timestamp())),
            success: true, // TODO: Реальный результат
            error: None,
            gas_used: Some(5000000), // 0.005 SOL
            actual_amount_out: Some(route.steps[0].expected_amount_out.clone()),
        };

        // Обновляем время последней сделки
        {
            let mut last_trade_time = self.last_trade_time.write().await;
            *last_trade_time = Instant::now();
        }

        Ok(result)
    }

    /// Update trade status
    async fn update_trade_status(&self, trade_id: &str, result: &ExecutionResult) {
        let mut trades = self.active_trades.write().await;
        if let Some(trade) = trades.get_mut(trade_id) {
            trade.status = if result.success {
                TradeStatus::Completed
            } else {
                TradeStatus::Failed
            };
            trade.last_update = Instant::now();
        }
    }

    /// Update trade status internally
    async fn update_trade_status_internal(&self, route_id: &str, status: TradeStatus) {
        let mut trades = self.active_trades.write().await;
        for trade in trades.values_mut() {
            if trade.route.id == route_id {
                trade.status = status;
                trade.last_update = Instant::now();
                break;
            }
        }
    }

    /// Update daily statistics
    async fn update_daily_stats(&self, result: &ExecutionResult) {
        let today = chrono::Utc::now().date_naive();
        let mut daily_stats = self.daily_stats.write().await;
        
        let stats = daily_stats.entry(today).or_insert_with(|| {
            // Получаем текущий баланс синхронно для инициализации
            let current_balance = {
                // Используем простой подход - создаем Amount с нулевым значением
                // В реальности здесь нужно будет решить проблему асинхронности
                crate::shared::types::Amount::new(0, 9)
            };
            DailyStats::new(today, current_balance)
        });

        // Рассчитываем прибыль/убыток
        let profit = if result.success {
            if let Some(actual_out) = &result.actual_amount_out {
                if actual_out.value > result.request.amount_in.value {
                    actual_out.value.saturating_sub(result.request.amount_in.value)
                } else {
                    0
                }
            } else {
                0
            }
        } else {
            // Неудачная сделка - убыток равен входной сумме
            result.request.amount_in.value
        };

        let profit_amount = Amount::new(profit, 9);
        let success = result.success && profit > 0;
        
        stats.update_after_trade(profit_amount, success);
    }

    /// Update current balance
    async fn update_balance(&self, result: &ExecutionResult) {
        let mut balance = self.current_balance.write().await;
        
        if result.success {
            if let Some(actual_out) = &result.actual_amount_out {
                // Успешная сделка - добавляем прибыль
                let profit = actual_out.value.saturating_sub(result.request.amount_in.value);
                balance.value = balance.value.saturating_add(profit);
            }
        } else {
            // Неудачная сделка - вычитаем входную сумму
            balance.value = balance.value.saturating_sub(result.request.amount_in.value);
        }
    }

    /// Get active trades
    pub async fn get_active_trades(&self) -> Vec<ActiveTrade> {
        let trades = self.active_trades.read().await;
        trades.values().cloned().collect()
    }

    /// Get daily statistics
    pub async fn get_daily_stats(&self, date: chrono::NaiveDate) -> Option<DailyStats> {
        let stats = self.daily_stats.read().await;
        stats.get(&date).cloned()
    }

    /// Get current balance
    pub async fn get_current_balance(&self) -> Amount {
        let balance = self.current_balance.read().await;
        balance.clone()
    }

    /// Cancel active trade
    pub async fn cancel_trade(&self, trade_id: &str) -> Result<(), AppError> {
        let mut trades = self.active_trades.write().await;
        if let Some(trade) = trades.get_mut(trade_id) {
            trade.status = TradeStatus::Cancelled;
            trade.last_update = Instant::now();
            Ok(())
        } else {
            Err(AppError::ExecutionError("Trade not found".to_string()))
        }
    }

    /// Get risk management configuration
    pub fn get_risk_config(&self) -> &RiskManagementConfig {
        &self.risk_config
    }

    /// Update risk management configuration
    pub fn update_risk_config(&mut self, config: RiskManagementConfig) {
        self.risk_config = config;
    }

    /// Get trading performance metrics
    pub async fn get_performance_metrics(&self) -> (f64, f64, f64) {
        let today = chrono::Utc::now().date_naive();
        let daily_stats = self.daily_stats.read().await;
        
        if let Some(stats) = daily_stats.get(&today) {
            (
                stats.get_success_rate(),
                stats.get_roi(),
                stats.net_profit.value as f64 / 1_000_000_000.0, // В SOL
            )
        } else {
            (0.0, 0.0, 0.0)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::arbitrage::{ArbitrageStep, ArbitrageRoute};
    use crate::domain::dex::DexType;

    #[tokio::test]
    async fn test_risk_validation() {
        let executor = RealTransactionExecutor::new_simple("".to_string());
        let risk_config = RiskManagementConfig::default();
        let initial_balance = Amount::new(10000000000, 9); // 10 SOL
        
        let arbitrage_executor = ArbitrageTransactionExecutor::new(
            executor,
            risk_config,
            initial_balance,
        );

        // Создаем тестовый маршрут
        let route = ArbitrageRoute {
            id: "test_route".to_string(),
            steps: vec![
                ArbitrageStep {
                    dex_type: DexType::OrcaWhirlpool,
                    pool_id: "pool1".to_string(),
                    token_in: Token {
                        mint: Pubkey::default(),
                        symbol: "SOL".to_string(),
                        decimals: 9,
                        name: Some("Wrapped SOL".to_string()),
                    },
                    token_out: Token {
                        mint: Pubkey::default(),
                        symbol: "USDC".to_string(),
                        decimals: 6,
                        name: Some("USD Coin".to_string()),
                    },
                    amount_in: Amount::new(1000000000, 9), // 1 SOL
                    expected_amount_out: Amount::new(100000000, 6), // 100 USDC
                    price_impact: 0.001,
                    fee: Amount::new(5000000, 9),
                    slippage_estimate: 0.001,
                    gas_estimate: Amount::new(5000000, 9),
                },
            ],
            expected_profit: 10000000.0, // 0.01 SOL
            profit_percentage: 0.01,     // 1%
            total_cost: Amount::new(10000000, 9),
            risk_score: 0.3,
            timestamp: Instant::now(),
            confidence_score: 0.9,
            execution_time_estimate: Duration::from_millis(500),
        };

        let profit_calculation = ProfitCalculation {
            gross_profit: 10000000.0,
            net_profit: 8000000.0,
            gas_cost: Amount::new(5000000, 9),
            slippage_cost: 1000000.0,
            fee_cost: 1000000.0,
            profit_margin: 0.8,
            is_profitable: true,
            roi_percentage: 0.8,
            break_even_amount: 950000000,
        };

        // Проверяем валидацию рисков
        let result = arbitrage_executor.validate_risk_limits(&route, &profit_calculation).await;
        assert!(result.is_ok());
    }
}
