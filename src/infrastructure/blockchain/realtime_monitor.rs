//! Real-time price monitoring service

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{Duration, interval};
use crate::shared::errors::AppError;
use super::yellowstone_grpc::{YellowstoneGrpcClient, PriceData};
use super::profit_calculator::RealProfitCalculator;

/// Price monitoring configuration
#[derive(Debug, Clone)]
pub struct MonitorConfig {
    pub update_interval_ms: u64,
    pub price_change_threshold: f64,
    pub max_price_history: usize,
    pub enable_alerts: bool,
    pub alert_threshold: f64,
}

impl Default for MonitorConfig {
    fn default() -> Self {
        Self {
            update_interval_ms: 1000, // 1 second
            price_change_threshold: 0.01, // 1%
            max_price_history: 1000,
            enable_alerts: true,
            alert_threshold: 0.05, // 5%
        }
    }
}

/// Price alert
#[derive(Debug, Clone)]
pub struct PriceAlert {
    pub token_mint: String,
    pub alert_type: AlertType,
    pub old_price: f64,
    pub new_price: f64,
    pub change_percentage: f64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Types of price alerts
#[derive(Debug, Clone)]
pub enum AlertType {
    PriceIncrease,
    PriceDecrease,
    Volatility,
    ArbitrageOpportunity,
}

/// Real-time price monitor
pub struct RealtimePriceMonitor {
    config: MonitorConfig,
    yellowstone_client: YellowstoneGrpcClient,
    profit_calculator: RealProfitCalculator,
    price_history: Arc<RwLock<HashMap<String, Vec<PriceData>>>>,
    alerts: Arc<RwLock<Vec<PriceAlert>>>,
    active: bool,
}

impl RealtimePriceMonitor {
    /// Create new price monitor
    pub fn new(
        config: MonitorConfig,
        yellowstone_config: super::yellowstone_grpc::YellowstoneConfig,
        rpc_url: String,
    ) -> Self {
        Self {
            config,
            yellowstone_client: YellowstoneGrpcClient::new(yellowstone_config),
            profit_calculator: RealProfitCalculator::new(rpc_url),
            price_history: Arc::new(RwLock::new(HashMap::new())),
            alerts: Arc::new(RwLock::new(Vec::new())),
            active: false,
        }
    }
    
    /// Create with default configuration
    pub fn new_default(rpc_url: String) -> Self {
        Self::new(
            MonitorConfig::default(),
            super::yellowstone_grpc::YellowstoneConfig::default(),
            rpc_url,
        )
    }
    
    /// Create with just RPC URL
    pub fn new(rpc_url: String) -> Self {
        Self::new_default(rpc_url)
    }
    
    /// Start monitoring
    pub async fn start(&mut self) -> Result<(), AppError> {
        if self.active {
            return Ok(());
        }
        
        // Connect to Yellowstone gRPC
        self.yellowstone_client.connect().await?;
        
        // Start monitoring
        self.yellowstone_client.start_monitoring().await?;
        
        self.active = true;
        println!("🚀 Real-time price monitoring started");
        
        // Start monitoring loop
        self.start_monitoring_loop().await;
        
        Ok(())
    }
    
    /// Stop monitoring
    pub async fn stop(&mut self) -> Result<(), AppError> {
        self.active = false;
        println!("🛑 Real-time price monitoring stopped");
        Ok(())
    }
    
    /// Add token to monitor
    pub async fn add_token(&mut self, token_mint: &str) -> Result<(), AppError> {
        // Subscribe to price updates
        self.yellowstone_client.subscribe_price_updates(token_mint).await?;
        
        // Initialize price history
        {
            let mut history = self.price_history.write().await;
            history.insert(token_mint.to_string(), Vec::new());
        }
        
        println!("📊 Added token to monitoring: {}", token_mint);
        Ok(())
    }
    
    /// Remove token from monitoring
    pub async fn remove_token(&mut self, token_mint: &str) -> Result<(), AppError> {
        // Unsubscribe from updates
        let subscription_id = format!("price_{}", token_mint);
        self.yellowstone_client.unsubscribe(&subscription_id).await?;
        
        // Remove from price history
        {
            let mut history = self.price_history.write().await;
            history.remove(token_mint);
        }
        
        println!("📊 Removed token from monitoring: {}", token_mint);
        Ok(())
    }
    
    /// Get current price for token
    pub async fn get_current_price(&self, token_mint: &str) -> Result<Option<PriceData>, AppError> {
        self.yellowstone_client.get_current_price(token_mint).await
    }
    
    /// Get price history for token
    pub async fn get_price_history(&self, token_mint: &str) -> Result<Vec<PriceData>, AppError> {
        let history = self.price_history.read().await;
        Ok(history.get(token_mint).cloned().unwrap_or_default())
    }
    
    /// Get all active alerts
    pub async fn get_alerts(&self) -> Vec<PriceAlert> {
        let alerts = self.alerts.read().await;
        alerts.clone()
    }
    
    /// Start monitoring loop
    async fn start_monitoring_loop(&self) {
        let price_history = self.price_history.clone();
        let alerts = self.alerts.clone();
        let config = self.config.clone();
        let yellowstone_client = self.yellowstone_client.clone();
        
        tokio::spawn(async move {
            let mut interval_timer = interval(Duration::from_millis(config.update_interval_ms));
            
            loop {
                interval_timer.tick().await;
                
                // Get all active subscriptions
                let subscriptions = yellowstone_client.get_active_subscriptions().await;
                
                for subscription in subscriptions {
                    if let super::yellowstone_grpc::SubscriptionType::PriceUpdates(token_mint) = subscription.subscription_type {
                        // Get current price
                        if let Ok(Some(price_data)) = yellowstone_client.get_current_price(&token_mint).await {
                            // Update price history
                            {
                                let mut history = price_history.write().await;
                                if let Some(token_history) = history.get_mut(&token_mint) {
                                    token_history.push(price_data.clone());
                                    
                                    // Trim history if too long
                                    if token_history.len() > config.max_price_history {
                                        token_history.remove(0);
                                    }
                                }
                            }
                            
                            // Check for price changes and generate alerts
                            if config.enable_alerts {
                                Self::check_price_alerts(&price_data, &price_history, &alerts, &config).await;
                            }
                        }
                    }
                }
            }
        });
    }
    
    /// Check for price alerts
    async fn check_price_alerts(
        current_price: &PriceData,
        price_history: &Arc<RwLock<HashMap<String, Vec<PriceData>>>>,
        alerts: &Arc<RwLock<Vec<PriceAlert>>>,
        config: &MonitorConfig,
    ) {
        let history = price_history.read().await;
        
        if let Some(token_history) = history.get(&current_price.token_mint) {
            if let Some(previous_price) = token_history.iter().rev().nth(1) {
                let price_change = (current_price.price_usd - previous_price.price_usd) / previous_price.price_usd;
                let change_percentage = price_change.abs() * 100.0;
                
                // Check if change exceeds threshold
                if change_percentage >= config.alert_threshold {
                    let alert_type = if price_change > 0.0 {
                        AlertType::PriceIncrease
                    } else {
                        AlertType::PriceDecrease
                    };
                    
                    let alert = PriceAlert {
                        token_mint: current_price.token_mint.clone(),
                        alert_type,
                        old_price: previous_price.price_usd,
                        new_price: current_price.price_usd,
                        change_percentage,
                        timestamp: chrono::Utc::now(),
                    };
                    
                    // Add alert
                    {
                        let mut alerts_list = alerts.write().await;
                        alerts_list.push(alert.clone());
                        
                        // Keep only recent alerts
                        if alerts_list.len() > 100 {
                            alerts_list.remove(0);
                        }
                    }
                    
                    // Print alert
                    let direction = if price_change > 0.0 { "📈" } else { "📉" };
                    println!("{} PRICE ALERT: {} changed by {:.2}% (${:.4} -> ${:.4})", 
                        direction, 
                        current_price.token_mint, 
                        change_percentage, 
                        previous_price.price_usd, 
                        current_price.price_usd
                    );
                }
            }
        }
    }
    
    /// Get monitoring statistics
    pub async fn get_statistics(&self) -> MonitorStatistics {
        let history = self.price_history.read().await;
        let alerts = self.alerts.read().await;
        let subscriptions = self.yellowstone_client.get_active_subscriptions().await;
        
        let total_tokens = history.len();
        let total_alerts = alerts.len();
        let active_subscriptions = subscriptions.len();
        
        MonitorStatistics {
            total_tokens,
            total_alerts,
            active_subscriptions,
            is_active: self.active,
        }
    }
}

/// Monitoring statistics
#[derive(Debug, Clone)]
pub struct MonitorStatistics {
    pub total_tokens: usize,
    pub total_alerts: usize,
    pub active_subscriptions: usize,
    pub is_active: bool,
}

impl MonitorStatistics {
    /// Print statistics
    pub fn print_summary(&self) {
        println!("📊 Real-time Monitor Statistics:");
        println!("   Status: {}", if self.is_active { "🟢 ACTIVE" } else { "🔴 INACTIVE" });
        println!("   Monitored Tokens: {}", self.total_tokens);
        println!("   Active Subscriptions: {}", self.active_subscriptions);
        println!("   Total Alerts: {}", self.total_alerts);
    }
}
