use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use futures_util::{StreamExt, SinkExt};
use bs58;

use yellowstone_grpc_client::{GeyserGrpcClient, GeyserGrpcClientError, Interceptor};
use yellowstone_grpc_proto::prelude::*;
use yellowstone_grpc_proto::geyser::{
    SubscribeRequest, SubscribeRequestFilterTransactions, SubscribeUpdateTransaction,
    CommitmentLevel,
};

use crate::domain::dex::DexType;
use crate::shared::errors::AppError;
use crate::shared::types::YellowstoneGrpcConfig;
use tracing::{info, warn, error, debug};

/// Информация о транзакции DEX
#[derive(Debug, Clone)]
pub struct DexTransactionInfo {
    pub signature: String,
    pub slot: u64,
    pub dex_type: DexType,
    pub program_id: String,
    pub timestamp: Instant,
    pub logs: Vec<String>,
}

/// Кэш цен для арбитража
#[derive(Debug, Clone)]
pub struct PriceCache {
    pub token_prices: HashMap<String, f64>,
    pub last_update: Instant,
    pub update_count: u64,
}

impl PriceCache {
    pub fn new() -> Self {
        Self {
            token_prices: HashMap::new(),
            last_update: Instant::now(),
            update_count: 0,
        }
    }

    pub fn update_price(&mut self, token: String, price: f64) {
        self.token_prices.insert(token, price);
        self.last_update = Instant::now();
        self.update_count += 1;
    }

    pub fn get_price(&self, token: &str) -> Option<f64> {
        self.token_prices.get(token).copied()
    }

    pub fn is_stale(&self, max_age: Duration) -> bool {
        self.last_update.elapsed() > max_age
    }
}

/// Арбитражный gRPC клиент для мониторинга DEX транзакций
pub struct ArbitrageGrpcClient {
    builder: Option<yellowstone_grpc_client::GeyserGrpcBuilder>,
    config: YellowstoneGrpcConfig,
    price_cache: Arc<RwLock<PriceCache>>,
    connection_status: Arc<RwLock<ConnectionStatus>>,
    subscription_handle: Option<tokio::task::JoinHandle<()>>,
}

#[derive(Debug, Clone)]
pub enum ConnectionStatus {
    Disconnected,
    Connecting,
    Connected,
    Error(String),
}

impl ArbitrageGrpcClient {
    /// Создать новый арбитражный gRPC клиент
    pub fn new(config: YellowstoneGrpcConfig) -> Self {
        Self {
            builder: None,
            config,
            price_cache: Arc::new(RwLock::new(PriceCache::new())),
            connection_status: Arc::new(RwLock::new(ConnectionStatus::Disconnected)),
            subscription_handle: None,
        }
    }

    /// Подключиться к gRPC серверу
    pub async fn connect(&mut self) -> Result<(), AppError> {
        info!("🔌 Подключение к Yellowstone gRPC: {}", self.config.endpoint);
        
        {
            let mut status = self.connection_status.write().await;
            *status = ConnectionStatus::Connecting;
        }

        // Создаем builder для gRPC клиента
        let builder = GeyserGrpcClient::build_from_shared(self.config.endpoint.clone())
            .map_err(|e| AppError::BlockchainError(format!("Failed to build client: {}", e)))?
            .x_token::<String>(self.config.token.clone())
            .map_err(|e| AppError::BlockchainError(format!("Failed to set x_token: {}", e)))?;
        
        self.builder = Some(builder);
        
        {
            let mut status = self.connection_status.write().await;
            *status = ConnectionStatus::Connected;
        }

        info!("✅ Успешно подготовили Yellowstone gRPC builder!");
        Ok(())
    }

    /// Подписаться на транзакции DEX программ
    pub async fn subscribe_to_dex_transactions(&mut self) -> Result<(), AppError> {
        info!("📡 Подписка на транзакции DEX программ...");
        
        let builder = self.builder.take()
            .ok_or_else(|| AppError::BlockchainError("gRPC builder not prepared".to_string()))?;

        // Создаем клиент и подключаемся
        let mut client = builder.connect().await
            .map_err(|e| AppError::BlockchainError(format!("Failed to connect: {}", e)))?;

        // Создаем фильтр для подписки на DEX транзакции
        let dex_programs = vec![
            "whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc".to_string(), // Orca Whirlpool
            "675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8".to_string(), // Raydium AMM
        ];

        // Создаем запрос на подписку
        let subscribe_request = SubscribeRequest {
            slots: std::collections::HashMap::new(),
            accounts: std::collections::HashMap::new(),
            transactions: std::collections::HashMap::from([
                ("All".to_string(), SubscribeRequestFilterTransactions {
                    vote: None,
                    failed: Some(false),
                    signature: None,
                    account_include: dex_programs,
                    account_exclude: vec![],
                    account_required: vec![]
                })
            ]),
            transactions_status: std::collections::HashMap::new(),
            entry: std::collections::HashMap::new(),
            blocks: std::collections::HashMap::new(),
            blocks_meta: std::collections::HashMap::new(),
            commitment: Some(CommitmentLevel::Processed as i32),
            accounts_data_slice: vec![],
            ping: None,
            from_slot: None,
        };

        // Подписываемся на обновления используя правильный метод
        let (mut subscribe_tx, mut stream) = client.subscribe().await
            .map_err(|e| AppError::BlockchainError(format!("Failed to subscribe: {}", e)))?;
        
        // Отправляем запрос на подписку
        subscribe_tx.send(subscribe_request).await
            .map_err(|e| AppError::BlockchainError(format!("Failed to send subscribe request: {}", e)))?;

        info!("✅ Подписка на DEX транзакции установлена!");
        info!("🔄 Начинаем обработку потока транзакций...");

        // Запускаем обработку потока в отдельной задаче
        let price_cache = self.price_cache.clone();
        let connection_status = self.connection_status.clone();
        
        let handle = tokio::spawn(async move {
            while let Some(update) = stream.next().await {
                match update {
                    Ok(update) => {
                        // Обрабатываем разные типы обновлений
                        match update.update_oneof {
                            Some(yellowstone_grpc_proto::geyser::subscribe_update::UpdateOneof::Transaction(transaction)) => {
                                if let Err(e) = Self::process_transaction_update_static(&transaction, &price_cache).await {
                                    error!("⚠️ Ошибка обработки транзакции: {}", e);
                                }
                            }
                            Some(yellowstone_grpc_proto::geyser::subscribe_update::UpdateOneof::Slot(slot)) => {
                                debug!("📊 Получен слот: {}", slot.slot);
                            }
                            Some(yellowstone_grpc_proto::geyser::subscribe_update::UpdateOneof::Account(account)) => {
                                if let Some(account_info) = &account.account {
                                    let pubkey_str = bs58::encode(&account_info.pubkey).into_string();
                                    info!("🏦 Получен аккаунт: {}", pubkey_str);
                                } else {
                                    debug!("🏦 Получен аккаунт без информации");
                                }
                            }
                            Some(yellowstone_grpc_proto::geyser::subscribe_update::UpdateOneof::Block(block)) => {
                                debug!("🧱 Получен блок: {}", block.slot);
                            }
                            Some(yellowstone_grpc_proto::geyser::subscribe_update::UpdateOneof::TransactionStatus(status)) => {
                                debug!("📋 Статус транзакции: {:?}", status);
                            }
                            Some(yellowstone_grpc_proto::geyser::subscribe_update::UpdateOneof::Ping(ping)) => {
                                debug!("🏓 Получен ping: {:?}", ping);
                            }
                            Some(yellowstone_grpc_proto::geyser::subscribe_update::UpdateOneof::Pong(pong)) => {
                                debug!("🏓 Получен pong: {:?}", pong);
                            }
                            Some(yellowstone_grpc_proto::geyser::subscribe_update::UpdateOneof::Entry(entry)) => {
                                debug!("📝 Получена запись: {:?}", entry);
                            }
                            Some(yellowstone_grpc_proto::geyser::subscribe_update::UpdateOneof::BlockMeta(block_meta)) => {
                                debug!("🧱 Мета блока: {:?}", block_meta);
                            }
                            None => {
                                debug!("❓ Неизвестный тип обновления");
                            }
                        }
                    }
                    Err(e) => {
                        error!("⚠️ Ошибка в gRPC потоке: {}", e);
                        let mut status = connection_status.write().await;
                        *status = ConnectionStatus::Error(format!("gRPC stream error: {}", e));
                    }
                }
            }
        });

        self.subscription_handle = Some(handle);
        Ok(())
    }

    /// Обработка обновления транзакции из gRPC потока (статический метод для использования в spawn)
    async fn process_transaction_update_static(
        transaction: &SubscribeUpdateTransaction,
        price_cache: &Arc<RwLock<PriceCache>>
    ) -> Result<(), AppError> {
        // Определяем DEX по program_id из логов
        let dex_type = Self::get_dex_type_from_logs_static(&transaction.transaction.as_ref().and_then(|tx| tx.meta.as_ref()).map(|meta| meta.log_messages.clone()));
        
        // Создаем информацию о транзакции
        let transaction_info = DexTransactionInfo {
            signature: bs58::encode(&transaction.transaction.as_ref().and_then(|tx| Some(tx.signature.clone())).unwrap_or_default()).into_string(),
            slot: transaction.slot,
            dex_type,
            program_id: Self::get_program_id_from_logs_static(&transaction.transaction.as_ref().and_then(|tx| tx.meta.as_ref()).map(|meta| meta.log_messages.clone())),
            timestamp: Instant::now(),
            logs: transaction.transaction.as_ref().and_then(|tx| tx.meta.as_ref()).map(|meta| meta.log_messages.clone()).unwrap_or_default(),
        };

        debug!("🔍 DEX транзакция: {} на {} (слот: {})", 
            transaction_info.signature, dex_type.as_str(), transaction_info.slot);

        // Анализируем транзакцию для поиска арбитража
        Self::analyze_transaction_for_arbitrage_static(&transaction_info, price_cache).await?;
        
        Ok(())
    }

    /// Обработка обновления транзакции из gRPC потока
    async fn process_transaction_update(&self, transaction: &SubscribeUpdateTransaction) -> Result<(), AppError> {
        Self::process_transaction_update_static(transaction, &self.price_cache).await
    }

    /// Определяем тип DEX по логам транзакции (статический метод)
    fn get_dex_type_from_logs_static(logs: &Option<Vec<String>>) -> DexType {
        let empty_vec = vec![];
        let logs = logs.as_ref().unwrap_or(&empty_vec);
        for log in logs {
            if log.contains("whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc") {
                return DexType::OrcaWhirlpool;
            }
            if log.contains("675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8") {
                return DexType::RaydiumAMM;
            }
        }
        DexType::OrcaWhirlpool // По умолчанию
    }

    /// Определяем тип DEX по логам транзакции
    fn get_dex_type_from_logs(&self, logs: &Option<Vec<String>>) -> DexType {
        Self::get_dex_type_from_logs_static(logs)
    }

    /// Извлекаем program_id из логов транзакции (статический метод)
    fn get_program_id_from_logs_static(logs: &Option<Vec<String>>) -> String {
        let empty_vec = vec![];
        let logs = logs.as_ref().unwrap_or(&empty_vec);
        for log in logs {
            if log.contains("whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc") {
                return "whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc".to_string();
            }
            if log.contains("675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8") {
                return "675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8".to_string();
            }
        }
        "unknown".to_string()
    }

    /// Извлекаем program_id из логов транзакции
    fn get_program_id_from_logs(&self, logs: &Option<Vec<String>>) -> String {
        Self::get_program_id_from_logs_static(logs)
    }

    /// Анализ транзакции для поиска арбитража (статический метод)
    async fn analyze_transaction_for_arbitrage_static(
        transaction: &DexTransactionInfo,
        price_cache: &Arc<RwLock<PriceCache>>
    ) -> Result<(), AppError> {
        debug!("🔍 Анализируем транзакцию {} для арбитража...", transaction.signature);
        
        // Извлекаем информацию о свопе из логов
        let swap_info = Self::extract_swap_info_from_logs(&transaction.logs);
        
        // Обновляем кэш цен на основе транзакции
        if let Some((token, price)) = swap_info {
            let mut cache = price_cache.write().await;
            cache.update_price(token.clone(), price);
            debug!("💰 Обновлена цена токена: {} = ${:.6}", token, price);
        }
        
        // Поиск арбитража между DEX
        Self::find_arbitrage_opportunities(transaction, price_cache).await?;
        
        Ok(())
    }

    /// Анализ транзакции для поиска арбитража
    async fn analyze_transaction_for_arbitrage(&self, transaction: &DexTransactionInfo) -> Result<(), AppError> {
        Self::analyze_transaction_for_arbitrage_static(transaction, &self.price_cache).await
    }

    /// Извлечение информации о свопе из логов транзакции
    fn extract_swap_info_from_logs(logs: &[String]) -> Option<(String, f64)> {
        // Простая логика извлечения информации о свопе
        // В реальном приложении здесь должна быть более сложная логика парсинга
        for log in logs {
            if log.contains("Swap") || log.contains("swap") {
                // Извлекаем базовую информацию о свопе
                // Это упрощенная версия - в реальности нужно парсить конкретные DEX логи
                return Some(("SOL".to_string(), 100.0)); // Заглушка
            }
        }
        None
    }

    /// Поиск арбитража между DEX
    async fn find_arbitrage_opportunities(
        _transaction: &DexTransactionInfo,
        price_cache: &Arc<RwLock<PriceCache>>
    ) -> Result<(), AppError> {
        let cache = price_cache.read().await;
        
        // Проверяем цены на разных DEX
        if let (Some(price_orca), Some(price_raydium)) = (
            cache.get_price("SOL"),
            cache.get_price("SOL")
        ) {
            let price_diff = (price_orca - price_raydium).abs();
            let price_diff_percent = (price_diff / price_orca) * 100.0;
            
            if price_diff_percent > 0.5 { // 0.5% разница
                debug!("🚀 Найдена арбитражная возможность!");
                debug!("   Orca: ${:.6}", price_orca);
                debug!("   Raydium: ${:.6}", price_raydium);
                debug!("   Разница: {:.2}%", price_diff_percent);
            }
        }
        
        Ok(())
    }

    /// Получить статус подключения
    pub async fn get_connection_status(&self) -> ConnectionStatus {
        self.connection_status.read().await.clone()
    }

    /// Получить кэш цен
    pub async fn get_price_cache(&self) -> PriceCache {
        self.price_cache.read().await.clone()
    }

    /// Обновить цену токена
    pub async fn update_token_price(&self, token: String, price: f64) {
        let mut cache = self.price_cache.write().await;
        cache.update_price(token, price);
    }

    /// Отключиться от gRPC сервера
    pub async fn disconnect(&mut self) -> Result<(), AppError> {
        info!("🔌 Отключение от Yellowstone gRPC...");
        
        // Останавливаем подписку
        if let Some(handle) = self.subscription_handle.take() {
            handle.abort();
        }
        
        // Очищаем клиент
        self.builder = None;
        
        // Обновляем статус
        {
            let mut status = self.connection_status.write().await;
            *status = ConnectionStatus::Disconnected;
        }
        
        info!("✅ Отключились от Yellowstone gRPC");
        Ok(())
    }

    /// Проверить, подключены ли мы
    pub fn is_connected(&self) -> bool {
        self.builder.is_some()
    }
}

/// Получить название DEX по program_id
fn get_dex_name(program_id: &str) -> &'static str {
    match program_id {
        "whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc" => "Orca Whirlpool",
        "675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8" => "Raydium V4",
        "CAMMCzo5YL8w4VFF8KVHrK22GGUsp5VTaW7grrKgrWqK" => "Raydium CLMM",
        "LBUZKhRxPF3XUpBCjp4YzTKgLccjZhTSDM9YuVaPwxo" => "Meteora DLMM",
        "Eo7WjKq67rjJQSZxS6z3YkapzY3eMj6Xy8X5EQVn5UaB" => "Meteora Pools",
        _ => "Unknown DEX",
    }
}

/// Получить program_id для DEX типа
fn get_program_id_for_dex(dex_type: &DexType) -> String {
    match dex_type {
        DexType::OrcaWhirlpool => "whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc".to_string(),
        DexType::RaydiumAMM => "675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8".to_string(),
    }
}
