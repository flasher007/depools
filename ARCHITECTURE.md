# 🏗️ Architecture Overview - Depools Arbitrage Bot

## 🎯 Общая архитектура

Наш арбитражный бот построен по принципам **Domain-Driven Design (DDD)** с четким разделением ответственности между слоями.

```
┌─────────────────────────────────────────────────────────────┐
│                    CLI Layer (main.rs)                     │
├─────────────────────────────────────────────────────────────┤
│                 Application Layer                          │
│  ┌─────────────────┐  ┌─────────────────┐                │
│  │   Commands      │  │   Services      │                │
│  └─────────────────┘  └─────────────────┘                │
├─────────────────────────────────────────────────────────────┤
│                 Domain Layer                               │
│  ┌─────────────────┐  ┌─────────────────┐                │
│  │   Arbitrage     │  │      DEX        │                │
│  │   Strategies    │  │   Interfaces    │                │
│  └─────────────────┘  └─────────────────┘                │
├─────────────────────────────────────────────────────────────┤
│              Infrastructure Layer                          │
│  ┌─────────────────────────────────────────────────────┐  │
│  │              Blockchain Integration                 │  │
│  │  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐  │  │
│  │  │Pool Discovery│ │Account Parser│ │Vault Reader│  │  │
│  │  └─────────────┘ └─────────────┘ └─────────────┘  │  │
│  │  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐  │  │
│  │  │Token Metadata│ │Profit Calc. │ │Yellowstone │  │  │
│  │  └─────────────┘ └─────────────┘ └─────────────┘  │  │
│  │  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐  │  │
│  │  │Real-time    │ │Arbitrage    │ │Transaction │  │  │
│  │  │Monitor      │ │Engine       │ │Executor    │  │
│  │  └─────────────┘ └─────────────┘ └─────────────┘  │  │
│  └─────────────────────────────────────────────────────┘  │
├─────────────────────────────────────────────────────────────┤
│                 Shared Layer                               │
│  ┌─────────────────┐  ┌─────────────────┐                │
│  │     Types       │  │     Errors      │                │
│  └─────────────────┘  └─────────────────┘                │
└─────────────────────────────────────────────────────────────┘
```

## 🧠 Domain Layer

### Core Entities

#### 1. **Arbitrage Domain**
```rust
// Стратегии арбитража
pub trait ArbitrageStrategy {
    async fn calculate_profit(&self, amount_in: Amount) -> Result<f64, ArbitrageError>;
    async fn execute(&self, amount_in: Amount) -> Result<ExecutionResult, ArbitrageError>;
}

// TwoHop стратегия (A -> B -> A)
pub struct TwoHopStrategy {
    pool_1: PoolInfo,
    pool_2: PoolInfo,
    min_profit_threshold: f64,
}

// Арбитражный движок
pub struct ArbitrageEngine {
    strategies: Vec<Box<dyn ArbitrageStrategy>>,
    profit_calculator: Box<dyn ProfitCalculator>,
}
```

#### 2. **DEX Domain**
```rust
// Типы DEX
pub enum DexType {
    OrcaWhirlpool,
    RaydiumV4,
}

// Информация о пуле
pub struct PoolInfo {
    pub id: String,
    pub dex_type: DexType,
    pub token_a: Token,
    pub token_b: Token,
    pub reserve_a: Amount,
    pub reserve_b: Amount,
    pub fee_rate: f64,
    pub liquidity: Amount,
    pub volume_24h: Amount,
}
```

#### 3. **Token Domain**
```rust
// Токен
pub struct Token {
    pub mint: Pubkey,
    pub symbol: String,
    pub decimals: u8,
    pub name: Option<String>,
}

// Количество
pub struct Amount {
    pub value: u64,
    pub decimals: u8,
}
```

## 🔧 Infrastructure Layer

### Blockchain Integration

#### 1. **Pool Discovery Service**
```rust
pub struct PoolDiscoveryService {
    rpc_client: SolanaRpcClient,
    orca_parser: OrcaAccountParser,
    raydium_parser: RaydiumAccountParser,
}

impl PoolDiscoveryService {
    // Сканирование блокчейна для поиска пулов
    pub async fn discover_pools(&self) -> Result<Vec<PoolInfo>, AppError>
    
    // Чтение данных конкретного пула
    pub async fn get_pool_data(&self, pool_id: &str) -> Result<PoolInfo, AppError>
}
```

#### 2. **Account Parser**
```rust
// Парсинг аккаунтов Orca
pub struct OrcaAccountParser {
    vault_reader: VaultReader,
}

impl OrcaAccountParser {
    // Парсинг Whirlpool аккаунта
    pub fn parse_whirlpool_account(&self, account_data: &[u8]) -> Result<PoolInfo, AppError>
    
    // Парсинг с реальными балансами
    pub async fn parse_pool_account_with_balances(&self, account_data: &[u8]) -> Result<PoolInfo, AppError>
}

// Парсинг аккаунтов Raydium
pub struct RaydiumAccountParser {
    vault_reader: VaultReader,
}
```

#### 3. **Vault Reader**
```rust
pub struct VaultReader {
    rpc_client: SolanaRpcClient,
    token_metadata_reader: TokenMetadataReader,
}

impl VaultReader {
    // Чтение баланса токена
    pub async fn get_token_balance(&self, token_account: &str) -> Result<Amount, AppError>
    
    // Чтение метаданных токена
    pub async fn get_token_metadata(&self, mint: &str) -> Result<TokenMetadata, AppError>
}
```

#### 4. **Profit Calculator**
```rust
pub struct RealProfitCalculator {
    vault_reader: Arc<VaultReader>,
}

impl RealProfitCalculator {
    // Расчет прибыли для TwoHop арбитража
    pub async fn calculate_twohop_profit(
        &self,
        token_a: &Token,
        token_b: &Token,
        amount_in: Amount,
        pool_1: &PoolInfo,
        pool_2: &PoolInfo,
    ) -> Result<ProfitCalculation, AppError>
}
```

#### 5. **Real-time Components**

##### Yellowstone gRPC Client
```rust
pub struct YellowstoneGrpcClient {
    config: YellowstoneConfig,
    channel: Option<Channel>,
    subscriptions: Arc<RwLock<HashMap<String, DataSubscription>>>,
    price_cache: Arc<RwLock<HashMap<String, PriceData>>>,
}

impl YellowstoneGrpcClient {
    // Подписка на обновления цен
    pub async fn subscribe_price_updates(&mut self, token_mint: &str) -> Result<String, AppError>
    
    // Получение текущей цены
    pub async fn get_current_price(&self, token_mint: &str) -> Result<Option<PriceData>, AppError>
}
```

##### Real-time Price Monitor
```rust
pub struct RealtimePriceMonitor {
    config: MonitorConfig,
    yellowstone_client: YellowstoneGrpcClient,
    profit_calculator: RealProfitCalculator,
    price_history: Arc<RwLock<HashMap<String, Vec<PriceData>>>>,
    alerts: Arc<RwLock<Vec<PriceAlert>>>,
}
```

##### Real-time Arbitrage Engine
```rust
pub struct RealtimeArbitrageEngine {
    config: AutoExecutionConfig,
    price_monitor: Arc<RwLock<RealtimePriceMonitor>>,
    profit_calculator: RealProfitCalculator,
    opportunities: Arc<RwLock<HashMap<String, ArbitrageOpportunity>>>,
    active_trades: Arc<RwLock<HashMap<String, TradeStatus>>>,
}
```

#### 6. **Transaction Executor**
```rust
pub struct RealTransactionExecutor {
    rpc_client: RpcClient,
    config: ExecutionConfig,
    wallet: Keypair,
}

impl RealTransactionExecutor {
    // Исполнение арбитражной транзакции
    pub async fn execute_arbitrage(
        &self,
        token_a: &Token,
        token_b: &Token,
        amount_in: Amount,
        pool_1: &PoolInfo,
        pool_2: &PoolInfo,
    ) -> Result<ExecutionResult, AppError>
    
    // Симуляция транзакции
    pub async fn simulate_transaction(&self, instructions: &[Instruction]) -> Result<ExecutionResult, AppError>
}
```

## 📱 Application Layer

### CLI Commands

```rust
#[derive(Subcommand)]
enum Commands {
    /// Открытие пулов
    Discover { save: bool },
    
    /// Тест реальных данных
    TestReal { mint: String },
    
    /// Тест расчетов прибыли
    TestProfit { amount: f64 },
    
    /// Тест арбитража
    TestArbitrage { amount: f64 },
    
    /// Тест мониторинга в реальном времени
    TestRealtime { duration: u64 },
    
    /// Тест арбитражного движка
    TestArbitrageEngine { duration: u64, auto_execute: bool },
    
    /// Тест исполнения транзакций
    TestTransactionExecution { amount: f64, simulate: bool },
}
```

## 🔄 Data Flow

### 1. **Pool Discovery Flow**
```
CLI Command → PoolDiscoveryService → Solana RPC → Account Parser → Pool Info
```

### 2. **Real-time Monitoring Flow**
```
Yellowstone gRPC → Price Data → Price Monitor → Alerts → Arbitrage Engine
```

### 3. **Arbitrage Execution Flow**
```
Opportunity Detection → Profit Calculation → Transaction Building → Execution → Result
```

## 🏗️ Design Patterns

### 1. **Repository Pattern**
- `PoolDiscoveryService` - репозиторий для пулов
- `VaultReader` - репозиторий для токенов

### 2. **Strategy Pattern**
- `ArbitrageStrategy` trait для разных стратегий
- `TwoHopStrategy` - конкретная реализация

### 3. **Observer Pattern**
- `RealtimePriceMonitor` - наблюдатель за ценами
- `PriceAlert` - уведомления об изменениях

### 4. **Factory Pattern**
- `ArbitrageEngine` - фабрика для создания стратегий
- `TransactionExecutor` - фабрика для транзакций

### 5. **Builder Pattern**
- `TransactionBuilder` - построение транзакций
- `InstructionBuilder` - построение инструкций

## 🔒 Error Handling

### Error Types
```rust
#[derive(Error, Debug)]
pub enum AppError {
    #[error("Blockchain error: {0}")]
    BlockchainError(String),
    
    #[error("DEX error: {0}")]
    DexError(String),
    
    #[error("Arbitrage error: {0}")]
    ArbitrageError(String),
    
    #[error("Configuration error: {0}")]
    ConfigError(String),
}
```

### Error Recovery
- **Retry Logic**: Автоматические повторы для сетевых ошибок
- **Fallback**: Переключение на devnet при недоступности mainnet
- **Circuit Breaker**: Остановка при критических ошибках

## 📊 Performance Considerations

### 1. **Async/Await**
- Все I/O операции асинхронные
- Параллельная обработка множественных пулов
- Неблокирующие операции

### 2. **Caching**
- Кэширование цен в памяти
- Кэширование метаданных токенов
- Кэширование пулов

### 3. **Connection Pooling**
- Переиспользование RPC соединений
- Управление жизненным циклом gRPC каналов

### 4. **Batch Processing**
- Группировка запросов к RPC
- Пакетное чтение аккаунтов

## 🚀 Scalability

### 1. **Horizontal Scaling**
- Множественные экземпляры бота
- Разделение по токенам/DEX
- Load balancing

### 2. **Vertical Scaling**
- Увеличение лимитов compute units
- Оптимизация алгоритмов
- Кэширование на уровне приложения

### 3. **Database Integration**
- PostgreSQL для исторических данных
- Redis для кэширования
- TimescaleDB для временных рядов

---

**🏗️ Архитектура готова для масштабирования и продакшена!**
