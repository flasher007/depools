# ⚙️ Configuration Examples - Depools Arbitrage Bot

## 🔧 Основная конфигурация

### `Config.toml` - Основной файл
```toml
[network]
rpc_url = "https://api.mainnet-beta.solana.com"
ws_url = "wss://api.mainnet-beta.solana.com"
commitment = "confirmed"
timeout_ms = 30000

[bot]
min_profit_threshold = 0.5
max_slippage = 1.0
max_gas_price = 1000
execution_delay_ms = 100
retry_attempts = 3

[monitoring]
update_interval_ms = 1000
price_change_threshold = 0.01
max_price_history = 1000
enable_alerts = true
alert_threshold = 0.05

[execution]
max_concurrent_trades = 5
max_priority_fee = 50000
compute_units = 200000
```

## 🧪 Конфигурация для разработки

### `Config.dev.toml` - Разработка
```toml
[network]
rpc_url = "https://api.devnet.solana.com"
ws_url = "wss://api.devnet.solana.com"
commitment = "confirmed"
timeout_ms = 60000

[bot]
min_profit_threshold = 0.1
max_slippage = 2.0
max_gas_price = 100
execution_delay_ms = 500
retry_attempts = 5

[monitoring]
update_interval_ms = 2000
price_change_threshold = 0.005
max_price_history = 500
enable_alerts = true
alert_threshold = 0.02

[execution]
max_concurrent_trades = 2
max_priority_fee = 10000
compute_units = 100000
```

## 🚀 Конфигурация для продакшена

### `Config.prod.toml` - Продакшен
```toml
[network]
rpc_url = "https://your-private-rpc.solana.com"
ws_url = "wss://your-private-rpc.solana.com"
commitment = "finalized"
timeout_ms = 15000

[bot]
min_profit_threshold = 1.0
max_slippage = 0.5
max_gas_price = 2000
execution_delay_ms = 50
retry_attempts = 2

[monitoring]
update_interval_ms = 500
price_change_threshold = 0.005
max_price_history = 2000
enable_alerts = true
alert_threshold = 0.02

[execution]
max_concurrent_trades = 10
max_priority_fee = 100000
compute_units = 300000

[risk]
max_position_size = 100.0
max_daily_loss = 1000.0
stop_loss_threshold = 0.1
circuit_breaker_threshold = 5
```

## 📊 Конфигурация мониторинга

### `Config.monitor.toml` - Только мониторинг
```toml
[network]
rpc_url = "https://api.mainnet-beta.solana.com"
commitment = "confirmed"
timeout_ms = 30000

[monitoring]
update_interval_ms = 1000
price_change_threshold = 0.01
max_price_history = 1000
enable_alerts = true
alert_threshold = 0.05

[execution]
max_concurrent_trades = 0  # Отключить исполнение
```

## 🎯 Конфигурация для тестирования

### `Config.test.toml` - Тестирование
```toml
[network]
rpc_url = "https://api.mainnet-beta.solana.com"
commitment = "confirmed"
timeout_ms = 30000

[bot]
min_profit_threshold = 0.1
max_slippage = 5.0
max_gas_price = 100
execution_delay_ms = 1000
retry_attempts = 10

[monitoring]
update_interval_ms = 5000
price_change_threshold = 0.02
max_price_history = 100
enable_alerts = false

[execution]
max_concurrent_trades = 1
max_priority_fee = 5000
compute_units = 50000
```

## 🔒 Конфигурация безопасности

### `Config.safe.toml` - Максимальная безопасность
```toml
[network]
rpc_url = "https://api.mainnet-beta.solana.com"
commitment = "finalized"
timeout_ms = 60000

[bot]
min_profit_threshold = 2.0
max_slippage = 0.3
max_gas_price = 500
execution_delay_ms = 2000
retry_attempts = 1

[monitoring]
update_interval_ms = 2000
price_change_threshold = 0.005
max_price_history = 5000
enable_alerts = true
alert_threshold = 0.01

[execution]
max_concurrent_trades = 1
max_priority_fee = 25000
compute_units = 150000

[risk]
max_position_size = 10.0
max_daily_loss = 100.0
stop_loss_threshold = 0.05
circuit_breaker_threshold = 2
```

## 📝 Переменные окружения

### `.env` файл
```bash
# Сеть
SOLANA_RPC_URL=https://api.mainnet-beta.solana.com
SOLANA_WS_URL=wss://api.mainnet-beta.solana.com
SOLANA_COMMITMENT=confirmed

# Бот
MIN_PROFIT_THRESHOLD=0.5
MAX_SLIPPAGE=1.0
MAX_GAS_PRICE=1000

# Мониторинг
UPDATE_INTERVAL_MS=1000
PRICE_CHANGE_THRESHOLD=0.01
ENABLE_ALERTS=true

# Исполнение
MAX_CONCURRENT_TRADES=5
MAX_PRIORITY_FEE=50000
COMPUTE_UNITS=200000

# Безопасность
MAX_POSITION_SIZE=50.0
MAX_DAILY_LOSS=500.0
STOP_LOSS_THRESHOLD=0.1
```

## 🎛️ Параметры командной строки

### Переопределение конфигурации
```bash
# Запуск с кастомными параметрами
cargo run -- test-arbitrage-engine \
  --duration 30 \
  --auto-execute \
  --min-profit 1.5 \
  --max-slippage 0.5

# Тест с другими настройками
cargo run -- test-transaction-execution \
  --amount 0.2 \
  --simulate \
  --priority-fee 100000 \
  --compute-units 250000
```

## 🔍 Мониторинг конфигурации

### Проверка текущих настроек
```bash
# Вывести текущую конфигурацию
cargo run -- config --show

# Валидация конфигурации
cargo run -- config --validate

# Тест подключения
cargo run -- config --test-connection
```

## 📊 Рекомендуемые настройки

### Для начинающих
- `min_profit_threshold`: 1.0%
- `max_slippage`: 1.0%
- `execution_delay_ms`: 1000ms
- `max_concurrent_trades`: 1

### Для опытных
- `min_profit_threshold`: 0.5%
- `max_slippage`: 0.5%
- `execution_delay_ms`: 100ms
- `max_concurrent_trades`: 5

### Для агрессивных
- `min_profit_threshold`: 0.2%
- `max_slippage`: 0.3%
- `execution_delay_ms`: 50ms
- `max_concurrent_trades`: 10

---

**⚙️ Настройте конфигурацию под ваши нужды!**
