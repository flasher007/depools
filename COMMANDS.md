# 🎮 Commands Reference - Depools Arbitrage Bot

## 📋 Полный список команд

### 🚀 **Основные команды**

#### `discover` - Открытие пулов
```bash
cargo run -- discover [OPTIONS]

OPTIONS:
    --save    Сохранить найденные пулы в файлы
```

**Описание**: Сканирует Solana блокчейн для поиска доступных пулов Orca и Raydium.

**Примеры**:
```bash
# Найти пулы без сохранения
cargo run -- discover

# Найти пулы и сохранить в файлы
cargo run -- discover --save
```

**Результат**:
```
🔍 Discovering pools on Solana blockchain...
📊 Found 15 Orca Whirlpool pools
📊 Found 23 Raydium V4 pools
💾 Saving pool data to files...
✅ Discovery completed: 38 pools found
```

---

#### `read-data` - Чтение данных с блокчейна
```bash
cargo run -- read-data --mint <TOKEN_MINT>

OPTIONS:
    --mint <TOKEN_MINT>    Mint адрес токена для чтения
```

**Описание**: Читает реальные данные токена с Solana блокчейна.

**Примеры**:
```bash
# Чтение USDC токена
cargo run -- read-data --mint EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v

# Чтение SOL токена
cargo run -- read-data --mint So11111111111111111111111111111111111111112

# Чтение USDT токена
cargo run -- read-data --mint Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB
```

**Результат**:
```
📊 Reading Real Blockchain Data
🔍 Token Mint: EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v
📊 Token Metadata:
   Symbol: USDC
   Decimals: 6
   Name: USD Coin
💰 Token Balance: 1,000,000 USDC
✅ Real data reading completed successfully!
```

---

#### `calculate-profit` - Расчет прибыли арбитража
```bash
cargo run -- calculate-profit --amount <AMOUNT>

OPTIONS:
    --amount <AMOUNT>    Сумма в SOL для расчета
```

**Описание**: Создает тестовые пулы с разницей цен и рассчитывает потенциальную прибыль от арбитража.

**Примеры**:
```bash
# Расчет с 1 SOL
cargo run -- calculate-profit --amount 1.0

# Расчет с 0.5 SOL
cargo run -- calculate-profit --amount 0.5

# Расчет с 2.0 SOL
cargo run -- calculate-profit --amount 2.0
```

**Результат**:
```
💰 Calculating Arbitrage Profit
📊 Setup:
   Token A: SOL (9 decimals)
   Token B: USDC (6 decimals)
   Amount: 1.0 SOL
   Pool 1: Orca Whirlpool
   Pool 2: Raydium V4

📈 Profit Calculation Results:
   Gross Profit: 0.020000 SOL
   DEX Fees: 0.005500 SOL
   Gas Cost: 0.000200 SOL
   Net Profit: 0.014300 SOL
   Profit Percentage: 1.43%
```

---

#### `analyze-arbitrage` - Анализ арбитража
```bash
cargo run -- analyze-arbitrage --amount <AMOUNT>

OPTIONS:
    --amount <AMOUNT>    Сумма в SOL для анализа
```

**Описание**: Анализирует арбитражные возможности с расчетом маршрутов и прибыльности.

**Примеры**:
```bash
# Анализ с 0.5 SOL
cargo run -- analyze-arbitrage --amount 0.5

# Анализ с 1.5 SOL
cargo run -- analyze-arbitrage --amount 1.5
```

**Результат**:
```
🎯 Analyzing Arbitrage Opportunities
📊 Arbitrage Analysis:
   Route: Orca → Raydium → Orca
   Input Amount: 0.5 SOL
   Expected Output: 0.5075 SOL
   Gross Profit: 0.0075 SOL
   Net Profit: 0.0058 SOL
   Profit Percentage: 1.16%
```

---

#### `monitor-realtime` - Мониторинг цен в реальном времени
```bash
cargo run -- monitor-realtime --duration <SECONDS>

OPTIONS:
    --duration <SECONDS>    Длительность мониторинга в секундах
```

**Описание**: Запускает мониторинг цен в реальном времени с генерацией алертов и статистики.

**Примеры**:
```bash
# Краткий мониторинг (15 секунд)
cargo run -- monitor-realtime --duration 15

# Средний мониторинг (1 минута)
cargo run -- monitor-realtime --duration 60

# Длительный мониторинг (5 минут)
cargo run -- monitor-realtime --duration 300
```

**Результат**:
```
📡 Starting Real-time Price Monitoring
⏱️  Duration: 15 seconds
🌐 Using RPC: https://api.mainnet-beta.solana.com

📊 Adding tokens to monitoring:
   ✅ EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v
   ✅ So11111111111111111111111111111111111111112
   ✅ Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB

🚀 Starting real-time monitoring...
📊 Update at 5s:
   Status: 🟢 ACTIVE
   Monitored Tokens: 3
   Active Subscriptions: 3
   Total Alerts: 2

🚨 Recent Alerts:
   📈 USDC: 0.12% change
   📉 SOL: 0.08% change
```

---

#### `run-engine` - Запуск арбитражного движка
```bash
cargo run -- run-engine --duration <SECONDS> [OPTIONS]

OPTIONS:
    --duration <SECONDS>    Длительность работы движка в секундах
    --auto-execute          Включить автоматическое исполнение сделок
```

**Описание**: Запускает полный арбитражный движок с мониторингом возможностей и автоматическим исполнением.

**Примеры**:
```bash
# Безопасный запуск (без авто-исполнения)
cargo run -- run-engine --duration 30

# Запуск с авто-исполнением
cargo run -- run-engine --duration 45 --auto-execute

# Длительный запуск
cargo run -- run-engine --duration 120 --auto-execute
```

**Результат**:
```
🎯 Starting Real-time Arbitrage Engine
⏱️  Duration: 45 seconds
🚀 Auto-execute: ENABLED
🌐 Using RPC: https://api.mainnet-beta.solana.com

📊 Adding tokens to monitoring:
   ✅ USDC, SOL, USDT

🚀 Starting real-time monitoring...
🚀 Starting arbitrage engine...
📊 Opportunities detected: 3
💰 Total potential profit: 0.125 SOL

🚀 Auto-executing profitable trades...
✅ Trade executed: 0.045 SOL profit
✅ Trade executed: 0.032 SOL profit
✅ Trade executed: 0.048 SOL profit

📊 Final Statistics:
   Total Opportunities: 8
   Executed Trades: 3
   Total Profit: 0.125 SOL
   Success Rate: 100%
```

---

#### `execute-transaction` - Исполнение арбитражных транзакций
```bash
cargo run -- execute-transaction --amount <AMOUNT> [OPTIONS]

OPTIONS:
    --amount <AMOUNT>    Сумма в SOL для исполнения
    --simulate           Режим симуляции (безопасно)
```

**Описание**: Строит и исполняет реальные Solana транзакции для арбитража.

**Примеры**:
```bash
# Безопасная симуляция
cargo run -- execute-transaction --amount 0.1 --simulate

# Просмотр деталей транзакции
cargo run -- execute-transaction --amount 0.05

# Симуляция с большой суммой
cargo run -- execute-transaction --amount 2.0 --simulate
```

**Результат**:
```
🔧 Executing Arbitrage Transaction
💰 Amount: 0.1 SOL
🧪 Mode: SIMULATION
🌐 Using RPC: https://api.mainnet-beta.solana.com
👛 Test wallet created: GaJmTya5sYtwLXxbZz5ZmMYfAPKvFoMEt6dSnQc67qab

📊 Setup:
   Token A: SOL (9 decimals)
   Token B: USDC (6 decimals)
   Amount: 100000000 lamports
   Pool 1: Orca Whirlpool
   Pool 2: Raydium V4

🧪 Simulating arbitrage transaction...
🧪 Simulating transaction...
   Simulation result: ✅ SUCCESS
   Estimated gas: 150000 lamports

📊 Simulation Results:
   Success: ✅ YES
   Gas Used: 150000 lamports
   💡 Transaction simulation successful! Ready for execution.

📈 Market Analysis:
   Orca Price: 1 SOL = 100.00 USDC
   Raydium Price: 1 SOL = 98.00 USDC
   Price Difference: 2.00%
   🎯 Significant arbitrage opportunity detected!
```

---

## 🔧 **Дополнительные опции**

### Глобальные флаги
```bash
--config <PATH>    Путь к файлу конфигурации (по умолчанию: Config.toml)
--help             Показать справку по команде
--version          Показать версию
```

### Переменные окружения
```bash
SOLANA_RPC_URL     URL Solana RPC endpoint
SOLANA_WS_URL      URL Solana WebSocket endpoint
RUST_LOG           Уровень логирования (debug, info, warn, error)
```

## 📊 **Анализ результатов команд**

### 1. **Успешное выполнение**
```
✅ Command completed successfully
📊 Statistics: ...
💰 Results: ...
```

### 2. **Предупреждения**
```
⚠️  Warning: ...
💡 Suggestion: ...
```

### 3. **Ошибки**
```
❌ Error: ...
🔍 Debug info: ...
💡 Solution: ...
```

## 🚨 **Безопасность команд**

### Безопасные команды (только чтение)
- ✅ `discover` - Сканирование пулов
- ✅ `test-real` - Чтение данных
- ✅ `test-profit` - Расчеты
- ✅ `test-arbitrage` - Анализ
- ✅ `test-realtime` - Мониторинг

### Команды с авто-исполнением
- ⚠️ `test-arbitrage-engine --auto-execute` - Автоматическое исполнение
- ⚠️ `test-transaction-execution` (без --simulate) - Реальные транзакции

### Рекомендации по безопасности
1. **Всегда начинайте с симуляции**
2. **Используйте малые суммы для тестирования**
3. **Мониторьте логи выполнения**
4. **Проверяйте конфигурацию перед запуском**

## 🔍 **Отладка команд**

### Включение детального логирования
```bash
# Debug уровень
RUST_LOG=debug cargo run -- <COMMAND>

# Trace уровень
RUST_LOG=trace cargo run -- <COMMAND>

# Только ошибки
RUST_LOG=error cargo run -- <COMMAND>
```

### Анализ логов
```bash
# Фильтрация по типу
cargo run -- test-realtime --duration 30 2>&1 | grep "ALERT"

# Поиск ошибок
cargo run -- test-arbitrage-engine --duration 45 2>&1 | grep "ERROR"

# Анализ производительности
cargo run -- test-transaction-execution --amount 0.1 --simulate 2>&1 | grep "Gas"
```

---

**🎮 Используйте эти команды для полного тестирования и настройки бота!**
