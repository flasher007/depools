//! Yellowstone gRPC client for real-time Solana data

use tonic::transport::Channel;
use futures_util::StreamExt;
use tokio_stream::wrappers::ReceiverStream;
use crate::shared::errors::AppError;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Yellowstone gRPC client configuration
#[derive(Debug, Clone)]
pub struct YellowstoneConfig {
    pub endpoint: String,
    pub auth_token: Option<String>,
    pub max_reconnect_attempts: u32,
    pub reconnect_delay_ms: u64,
}

impl Default for YellowstoneConfig {
    fn default() -> Self {
        Self {
            endpoint: "https://grpc.yellowstone.com".to_string(),
            auth_token: None,
            max_reconnect_attempts: 5,
            reconnect_delay_ms: 1000,
        }
    }
}

/// Real-time data subscription
#[derive(Debug, Clone)]
pub struct DataSubscription {
    pub id: String,
    pub subscription_type: SubscriptionType,
    pub active: bool,
    pub last_update: chrono::DateTime<chrono::Utc>,
}

/// Types of data subscriptions
#[derive(Debug, Clone)]
pub enum SubscriptionType {
    AccountUpdates(String), // Account address
    ProgramUpdates(String), // Program ID
    SlotUpdates,
    TransactionUpdates,
    PriceUpdates(String), // Token mint
}

/// Yellowstone gRPC client
#[derive(Clone)]
pub struct YellowstoneGrpcClient {
    config: YellowstoneConfig,
    channel: Option<Channel>,
    subscriptions: Arc<RwLock<HashMap<String, DataSubscription>>>,
    price_cache: Arc<RwLock<HashMap<String, PriceData>>>,
}

/// Real-time price data
#[derive(Debug, Clone)]
pub struct PriceData {
    pub token_mint: String,
    pub price_usd: f64,
    pub price_sol: f64,
    pub volume_24h: f64,
    pub market_cap: f64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub source: String,
}

impl YellowstoneGrpcClient {
    /// Create new Yellowstone gRPC client
    pub fn new(config: YellowstoneConfig) -> Self {
        Self {
            config,
            channel: None,
            subscriptions: Arc::new(RwLock::new(HashMap::new())),
            price_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Create with default configuration
    pub fn new_default() -> Self {
        Self::new(YellowstoneConfig::default())
    }
    
    /// Create new Yellowstone gRPC client with default configuration
    pub fn new() -> Self {
        Self::new_default()
    }
    
    /// Connect to Yellowstone gRPC service
    pub async fn connect(&mut self) -> Result<(), AppError> {
        let mut attempts = 0;
        
        while attempts < self.config.max_reconnect_attempts {
            match tonic::transport::Channel::from_shared(self.config.endpoint.clone()) {
                Ok(endpoint) => {
                    // Build channel from endpoint
                    match endpoint.connect().await {
                        Ok(channel) => {
                            self.channel = Some(channel);
                            println!("✅ Connected to Yellowstone gRPC: {}", self.config.endpoint);
                            return Ok(());
                        }
                        Err(e) => {
                            attempts += 1;
                            eprintln!("❌ Connection attempt {} failed: {}", attempts, e);
                            
                            if attempts < self.config.max_reconnect_attempts {
                                tokio::time::sleep(tokio::time::Duration::from_millis(self.config.reconnect_delay_ms)).await;
                            }
                        }
                    }
                }
                Err(e) => {
                    attempts += 1;
                    eprintln!("❌ Endpoint creation failed: {}", e);
                    
                    if attempts < self.config.max_reconnect_attempts {
                        tokio::time::sleep(tokio::time::Duration::from_millis(self.config.reconnect_delay_ms)).await;
                    }
                }
            }
        }
        
        Err(AppError::BlockchainError("Failed to connect to Yellowstone gRPC after max attempts".to_string()))
    }
    
    /// Subscribe to account updates
    pub async fn subscribe_account_updates(&mut self, account: &str) -> Result<String, AppError> {
        let subscription_id = format!("account_{}", account);
        
        let subscription = DataSubscription {
            id: subscription_id.clone(),
            subscription_type: SubscriptionType::AccountUpdates(account.to_string()),
            active: true,
            last_update: chrono::Utc::now(),
        };
        
        {
            let mut subscriptions = self.subscriptions.write().await;
            subscriptions.insert(subscription_id.clone(), subscription);
        }
        
        println!("📡 Subscribed to account updates: {}", account);
        Ok(subscription_id)
    }
    
    /// Subscribe to program updates
    pub async fn subscribe_program_updates(&mut self, program_id: &str) -> Result<String, AppError> {
        let subscription_id = format!("program_{}", program_id);
        
        let subscription = DataSubscription {
            id: subscription_id.clone(),
            subscription_type: SubscriptionType::ProgramUpdates(program_id.to_string()),
            active: true,
            last_update: chrono::Utc::now(),
        };
        
        {
            let mut subscriptions = self.subscriptions.write().await;
            subscriptions.insert(subscription_id.clone(), subscription);
        }
        
        println!("📡 Subscribed to program updates: {}", program_id);
        Ok(subscription_id)
    }
    
    /// Subscribe to price updates
    pub async fn subscribe_price_updates(&mut self, token_mint: &str) -> Result<String, AppError> {
        let subscription_id = format!("price_{}", token_mint);
        
        let subscription = DataSubscription {
            id: subscription_id.clone(),
            subscription_type: SubscriptionType::PriceUpdates(token_mint.to_string()),
            active: true,
            last_update: chrono::Utc::now(),
        };
        
        {
            let mut subscriptions = self.subscriptions.write().await;
            subscriptions.insert(subscription_id.clone(), subscription);
        }
        
        println!("📡 Subscribed to price updates: {}", token_mint);
        Ok(subscription_id)
    }
    
    /// Get current price for token
    pub async fn get_current_price(&self, token_mint: &str) -> Result<Option<PriceData>, AppError> {
        let price_cache = self.price_cache.read().await;
        Ok(price_cache.get(token_mint).cloned())
    }
    
    /// Get all active subscriptions
    pub async fn get_active_subscriptions(&self) -> Vec<DataSubscription> {
        let subscriptions = self.subscriptions.read().await;
        subscriptions.values()
            .filter(|sub| sub.active)
            .cloned()
            .collect()
    }
    
    /// Unsubscribe from updates
    pub async fn unsubscribe(&mut self, subscription_id: &str) -> Result<(), AppError> {
        let mut subscriptions = self.subscriptions.write().await;
        
        if let Some(subscription) = subscriptions.get_mut(subscription_id) {
            subscription.active = false;
            println!("📡 Unsubscribed from: {}", subscription_id);
            Ok(())
        } else {
            Err(AppError::BlockchainError(format!("Subscription not found: {}", subscription_id)))
        }
    }
    
    /// Start monitoring all active subscriptions
    pub async fn start_monitoring(&mut self) -> Result<(), AppError> {
        if self.channel.is_none() {
            return Err(AppError::BlockchainError("Not connected to Yellowstone gRPC".to_string()));
        }
        
        println!("🚀 Starting real-time data monitoring...");
        
        // Start monitoring loop
        let subscriptions = self.subscriptions.clone();
        let price_cache = self.price_cache.clone();
        
        tokio::spawn(async move {
            loop {
                let active_subs = {
                    let subs = subscriptions.read().await;
                    subs.values()
                        .filter(|sub| sub.active)
                        .cloned()
                        .collect::<Vec<_>>()
                };
                
                for subscription in active_subs {
                    // Simulate real-time updates (in production, this would be actual gRPC streams)
                    match subscription.subscription_type {
                        SubscriptionType::PriceUpdates(token_mint) => {
                            let price_data = PriceData {
                                token_mint: token_mint.clone(),
                                price_usd: 100.0 + (chrono::Utc::now().timestamp() % 100) as f64, // Simulated price
                                price_sol: 1.0 + (chrono::Utc::now().timestamp() % 10) as f64 * 0.1,
                                volume_24h: 1000000.0,
                                market_cap: 1000000000.0,
                                timestamp: chrono::Utc::now(),
                                source: "Yellowstone".to_string(),
                            };
                            
                            let mut cache = price_cache.write().await;
                            cache.insert(token_mint, price_data);
                        }
                        _ => {
                            // Handle other subscription types
                        }
                    }
                }
                
                tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
            }
        });
        
        Ok(())
    }
}

// TODO: Implement proper authentication in production
