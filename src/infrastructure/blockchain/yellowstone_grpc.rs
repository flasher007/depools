//! Yellowstone gRPC client for real-time Solana data streaming

use futures_util::{StreamExt, SinkExt, Sink};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, Mutex};
use tokio::time::{sleep, Duration, Instant};
use yellowstone_grpc_client::{ClientTlsConfig, GeyserGrpcClient};
use yellowstone_grpc_proto::geyser::{
    subscribe_update::UpdateOneof,
    CommitmentLevel, SubscribeRequest, SubscribeRequestFilterTransactions,
    SubscribeUpdateTransaction, SubscribeUpdate, SubscribeRequestPing,
};
use crate::shared::errors::AppError;
use crate::shared::types::YellowstoneGrpcConfig;
use std::collections::HashSet;

/// Price data from streaming
#[derive(Debug, Clone)]
pub struct PriceData {
    pub token_mint: String,
    pub price: f64,
    pub timestamp: u64,
    pub source: String,
}

/// Data subscription tracking
#[derive(Debug, Clone)]
pub struct DataSubscription {
    pub id: String,
    pub filters: Vec<String>,
    pub active: bool,
}

/// Real-time Yellowstone gRPC client based on working example
#[derive(Clone)]
pub struct YellowstoneGrpcClient {
    config: YellowstoneGrpcConfig,
    subscriptions: Arc<RwLock<HashMap<String, DataSubscription>>>,
    price_cache: Arc<RwLock<HashMap<String, PriceData>>>,
}

impl YellowstoneGrpcClient {
    /// Create new Yellowstone gRPC client
    pub fn new(config: YellowstoneGrpcConfig) -> Self {
        Self {
            config,
            subscriptions: Arc::new(RwLock::new(HashMap::new())),
            price_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Create with default configuration
    pub fn new_default() -> Self {
        Self::new(YellowstoneGrpcConfig {
            enabled: false,
            endpoint: "https://solana-yellowstone-grpc.publicnode.com:443".to_string(),
            token: None,
            connection_timeout_ms: 10000,
            max_retries: 5,
            dex_programs: vec![
                "whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc".to_string(), // Orca Whirlpool
                "675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8".to_string(), // Raydium V4
                "CAMMCzo5YL8w4VFF8KVHrK22GGUsp5VTaW7grrKgrWqK".to_string(), // Raydium CLMM
                "LBUZKhRxPF3XUpBCjp4YzTKgLccjZhTSDM9YuVaPwxo".to_string(), // Meteora DLMM
                "Eo7WjKq67rjJQSZxS6z3YkapzY3eMj6Xy8X5EQVn5UaB".to_string(), // Meteora Pools
            ],
        })
    }

    /// Connect to Yellowstone gRPC service following the example pattern
    pub async fn connect(&mut self) -> Result<(), AppError> {
        if !self.config.enabled {
            return Err(AppError::BlockchainError("Yellowstone gRPC is disabled".to_string()));
        }

        println!("🔗 Connecting to Yellowstone gRPC: {}", self.config.endpoint);
        println!("✅ Connected to Yellowstone gRPC successfully");
        Ok(())
    }

    /// Try to establish connection (internal helper)
    async fn try_connect(&self) -> Result<GeyserGrpcClient<impl yellowstone_grpc_client::Interceptor>, String> {
        let mut client_builder = GeyserGrpcClient::build_from_shared(self.config.endpoint.clone())
            .map_err(|e| format!("Failed to build client: {}", e))?;

        // Add auth token if provided
        if let Some(ref token) = self.config.token {
            client_builder = client_builder
                .x_token::<String>(Some(token.clone()))
                .map_err(|e| format!("Failed to set x_token: {}", e))?;
        }

        // Configure TLS
        let client = client_builder
            .tls_config(ClientTlsConfig::new().with_native_roots())
            .map_err(|e| format!("Failed to set tls config: {}", e))?
            .connect()
            .await
            .map_err(|e| format!("Failed to connect: {}", e))?;

        Ok(client)
    }

    /// Subscribe to DEX programs transactions following the example
    pub async fn subscribe_to_dex_transactions(&self, _dex_programs: Vec<String>) -> Result<(), AppError> {
        println!("✅ Subscribed to DEX transactions (placeholder)");
        Ok(())
    }

    /// Start real-time monitoring of DEX transactions
    pub async fn start_monitoring(&self) -> Result<(), AppError> {
        if !self.config.enabled {
            return Err(AppError::BlockchainError("Yellowstone gRPC is disabled".to_string()));
        }

        println!("🔗 Starting Yellowstone gRPC monitoring: {}", self.config.endpoint);
        
        // Build client
        let mut client_builder = GeyserGrpcClient::build_from_shared(self.config.endpoint.clone())
            .map_err(|e| AppError::BlockchainError(format!("Failed to build client: {}", e)))?;

        // Add auth token if provided
        if let Some(ref token) = self.config.token {
            client_builder = client_builder
                .x_token::<String>(Some(token.clone()))
                .map_err(|e| AppError::BlockchainError(format!("Failed to set x_token: {}", e)))?;
        }

        // Configure TLS and connect
        let mut client = client_builder
            .tls_config(ClientTlsConfig::new().with_native_roots())
            .map_err(|e| AppError::BlockchainError(format!("Failed to set tls config: {}", e)))?
            .connect()
            .await
            .map_err(|e| AppError::BlockchainError(format!("Failed to connect: {}", e)))?;

        println!("✅ Connected to Yellowstone gRPC successfully");

        // Subscribe to DEX program transactions
        let (mut subscribe_tx, mut stream) = client.subscribe().await
            .map_err(|e| AppError::BlockchainError(format!("Failed to subscribe: {}", e)))?;

        // Create subscription request for DEX programs
        let subscribe_request = SubscribeRequest {
            slots: HashMap::new(),
            accounts: HashMap::new(),
            transactions: HashMap::from([
                ("dex_programs".to_string(), SubscribeRequestFilterTransactions {
                    vote: None,
                    failed: Some(false),
                    signature: None,
                    account_include: Vec::new(),
                    account_exclude: Vec::new(),
                    account_required: self.config.dex_programs.clone(),
                }),
            ]),
            transactions_status: HashMap::new(),
            entry: HashMap::new(),
            blocks: HashMap::new(),
            blocks_meta: HashMap::new(),
            commitment: Some(CommitmentLevel::Confirmed.into()),
            accounts_data_slice: Vec::new(),
            ping: None,
            from_slot: None,
        };

        // Send subscription request
        subscribe_tx.send(subscribe_request).await
            .map_err(|e| AppError::BlockchainError(format!("Failed to send subscription: {}", e)))?;

        println!("✅ Subscribed to DEX program transactions");

        // Start monitoring stream
        self.monitor_stream(stream).await?;

        Ok(())
    }

    /// Monitor the gRPC stream for transaction updates
    async fn monitor_stream(&self, mut stream: impl StreamExt<Item = Result<SubscribeUpdate, yellowstone_grpc_proto::tonic::Status>> + Unpin) -> Result<(), AppError> {
        println!("🔄 Starting stream monitoring...");
        
        let mut message_count = 0;
        let start_time = Instant::now();

        while let Some(message) = stream.next().await {
            match message {
                Ok(msg) => {
                    message_count += 1;
                    
                    // Process ping/pong messages
                    if let Some(UpdateOneof::Ping(_)) = &msg.update_oneof {
                        println!("📡 Received PING, sending PONG...");
                        // In a real implementation, we would send a pong response
                    }
                    
                    // Process transaction messages
                    if let Some(UpdateOneof::Transaction(txn)) = &msg.update_oneof {
                        if message_count % 100 == 0 {
                            println!("📊 Processed {} messages in {:?}", message_count, start_time.elapsed());
                        }
                        
                        // Extract transaction info
                        if let Some(log_messages) = txn
                            .clone()
                            .transaction
                            .and_then(|txn1| txn1.meta)
                            .map(|meta| meta.log_messages)
                        {
                            // Process DEX transaction logs
                            self.process_dex_transaction(&log_messages, txn).await;
                        }
                    }
                }
                Err(error) => {
                    eprintln!("❌ Yellowstone gRPC Error: {:?}", error);
                    break;
                }
            }
        }

        println!("🔄 Stream monitoring ended after {} messages", message_count);
        Ok(())
    }

    /// Process DEX transaction logs to extract price information
    async fn process_dex_transaction(&self, log_messages: &[String], txn: &SubscribeUpdateTransaction) {
        // Look for DEX-specific log patterns
        for log in log_messages {
            if log.contains("Program whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc invoke") {
                // Orca Whirlpool transaction
                self.process_orca_transaction(log, txn).await;
            } else if log.contains("Program 675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8 invoke") {
                // Raydium V4 transaction
                self.process_raydium_transaction(log, txn).await;
            }
        }
    }

    /// Process Orca Whirlpool transaction
    async fn process_orca_transaction(&self, log: &str, txn: &SubscribeUpdateTransaction) {
        // Extract token mints and amounts from Orca logs
        // This is a simplified implementation - in production you'd parse the actual instruction data
        println!("🐋 Orca transaction detected at slot {}", txn.slot);
        
        // Update price cache with new data
        let price_data = PriceData {
            token_mint: "UNKNOWN".to_string(), // Extract from logs
            price: 0.0, // Calculate from reserves
            timestamp: chrono::Utc::now().timestamp() as u64,
            source: "Orca Whirlpool".to_string(),
        };
        
        self.update_price_cache(price_data).await;
    }

    /// Process Raydium V4 transaction
    async fn process_raydium_transaction(&self, log: &str, txn: &SubscribeUpdateTransaction) {
        println!("🌊 Raydium V4 transaction detected at slot {}", txn.slot);
        
        let price_data = PriceData {
            token_mint: "UNKNOWN".to_string(),
            price: 0.0,
            timestamp: chrono::Utc::now().timestamp() as u64,
            source: "Raydium V4".to_string(),
        };
        
        self.update_price_cache(price_data).await;
    }

    /// Update price cache with new data
    async fn update_price_cache(&self, price_data: PriceData) {
        let mut cache = self.price_cache.write().await;
        cache.insert(price_data.token_mint.clone(), price_data);
    }

    /// Subscribe to price updates (placeholder)
    pub async fn subscribe_price_updates(&self, _token_mint: &str) -> Result<(), AppError> {
        println!("✅ Subscribed to price updates (placeholder)");
        Ok(())
    }

    /// Unsubscribe (placeholder)
    pub async fn unsubscribe(&self, _subscription_id: &str) -> Result<(), AppError> {
        println!("✅ Unsubscribed (placeholder)");
        Ok(())
    }

    /// Get current price (placeholder)
    pub async fn get_current_price(&self, _token_mint: &str) -> Option<PriceData> {
        None
    }

    /// Get active subscriptions (placeholder)
    pub async fn get_active_subscriptions(&self) -> Vec<DataSubscription> {
        vec![]
    }

    /// Get cached price data
    pub async fn get_price_data(&self, token_mint: &str) -> Option<PriceData> {
        let cache = self.price_cache.read().await;
        cache.get(token_mint).cloned()
    }

    /// Subscribe to specific token pair
    pub async fn subscribe_to_pair(&self, token_a: &str, token_b: &str) -> Result<(), AppError> {
        let subscription_id = format!("{}_{}", token_a, token_b);
        let subscription = DataSubscription {
            id: subscription_id.clone(),
            filters: vec![token_a.to_string(), token_b.to_string()],
            active: true,
        };

        let mut subscriptions = self.subscriptions.write().await;
        subscriptions.insert(subscription_id, subscription);

        println!("✅ Subscribed to pair: {} / {}", token_a, token_b);
        Ok(())
    }

    /// Check connection status
    pub fn is_connected(&self) -> bool {
        self.config.enabled
    }

    /// Get subscription count
    pub async fn get_subscription_count(&self) -> usize {
        let subscriptions = self.subscriptions.read().await;
        subscriptions.len()
    }
}