# 💡 Usage Examples - Depools Arbitrage Bot

## 🚀 Практические примеры использования

### 1. **Первое знакомство с ботом** 🆕

#### Шаг 1: Сборка и проверка
```bash
# Собрать проект
cargo build --release

# Проверить доступные команды
cargo run -- --help
```

#### Шаг 2: Открытие пулов
```bash
# Найти все доступные пулы
cargo run -- discover --save

# Результат: Сканирование блокчейна и сохранение пулов в файлы
```

#### Шаг 3: Чтение данных с блокчейна
```bash
# Прочитать данные USDC токена
cargo run -- read-data --mint EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v

# Результат: Чтение реального баланса и метаданных с Solana
```

### 2. **Анализ арбитража** 💰

#### Расчет прибыли арбитража
```bash
# Расчет с 1 SOL
cargo run -- calculate-profit --amount 1.0

# Результат:
# 💰 Profit Calculation Results:
#    Gross Profit: 0.020000 SOL
#    DEX Fees: 0.005500 SOL
#    Gas Cost: 0.000200 SOL
#    Net Profit: 0.014300 SOL
#    Profit Percentage: 1.43%
```

#### Анализ арбитража с разными суммами
```bash
# Анализ с 0.5 SOL
cargo run -- analyze-arbitrage --amount 0.5

# Анализ с 2.0 SOL
cargo run -- analyze-arbitrage --amount 2.0

# Результат: Анализ прибыльности для разных сумм
```

### 3. **Мониторинг цен в реальном времени** 📊

#### Краткий мониторинг (15 секунд)
```bash
cargo run -- monitor-realtime --duration 15

# Результат:
# 📡 Real-time Price Monitoring
# ⏱️  Duration: 15 seconds
# 🚀 Real-time price monitoring started
# 📊 Update at 5s:
#    Status: 🟢 ACTIVE
#    Monitored Tokens: 3
#    Active Subscriptions: 3
#    Total Alerts: 2
```

#### Длительный мониторинг (2 минуты)
```bash
cargo run -- monitor-realtime --duration 120

# Результат: Расширенная статистика и больше алертов
```

### 4. **Запуск арбитражного движка** 🎯

#### Безопасный запуск (без авто-исполнения)
```bash
cargo run -- run-engine --duration 30

# Результат:
# 🎯 Real-time Arbitrage Engine
# ⏱️  Duration: 30 seconds
# 🚀 Auto-execute: DISABLED
# 📊 Opportunities detected: 5
# 💰 Total potential profit: 0.125 SOL
```

#### Запуск с авто-исполнением
```bash
cargo run -- run-engine --duration 45 --auto-execute

# Результат:
# 🎯 Real-time Arbitrage Engine
# ⏱️  Duration: 45 seconds
# 🚀 Auto-execute: ENABLED
# 🚀 Auto-executing profitable trades...
# ✅ Trade executed: 0.045 SOL profit
# 📊 Total executed trades: 3
```

### 5. **Исполнение транзакций** 🔧

#### Симуляция транзакций (безопасно)
```bash
cargo run -- execute-transaction --amount 0.1 --simulate

# Результат:
# 🔧 Real Transaction Execution
# 💰 Amount: 0.1 SOL
# 🧪 Mode: SIMULATION
# 🧪 Simulating arbitrage transaction...
# 📊 Simulation Results:
#    Success: ✅ YES
#    Gas Used: 150000 lamports
#    💡 Transaction simulation successful! Ready for execution.
```

#### Просмотр деталей транзакции
```bash
cargo run -- execute-transaction --amount 0.05

# Результат:
# 🔧 Testing Real Transaction Execution
# 💰 Amount: 0.05 SOL
# 🧪 Mode: REAL EXECUTION
# 📋 Transaction Details:
#    Type: TwoHop Arbitrage
#    Route: Orca Whirlpool -> Raydium V4 -> Orca Whirlpool
#    Expected Profit: ~2% (minus fees)
#    Estimated Gas: ~200k compute units
#    Priority Fee: 50k lamports
```

### 6. **Комбинированные сценарии** 🔄

#### Полный цикл тестирования
```bash
# 1. Открытие пулов
cargo run -- discover --save

# 2. Чтение данных
cargo run -- read-data --mint So11111111111111111111111111111111111111112

# 3. Расчет прибыли
cargo run -- calculate-profit --amount 1.0

# 4. Мониторинг цен
cargo run -- monitor-realtime --duration 20

# 5. Запуск движка
cargo run -- run-engine --duration 30 --auto-execute

# 6. Исполнение транзакций
cargo run -- execute-transaction --amount 0.2 --simulate
```

### 7. **Настройка параметров** ⚙️

#### Тест с кастомными настройками
```bash
# Тест с высоким порогом прибыли
cargo run -- test-arbitrage-engine --duration 60

# Тест с длительным мониторингом
cargo run -- test-realtime --duration 300

# Тест с большой суммой
cargo run -- test-transaction-execution --amount 5.0 --simulate
```

## 📊 Анализ результатов

### 1. **Интерпретация статистики**
```
📊 Real-time Monitor Statistics:
   Status: 🟢 ACTIVE
   Monitored Tokens: 3
   Active Subscriptions: 3
   Total Alerts: 5
```

**Что это означает:**
- ✅ Бот активно работает
- 📊 Отслеживает 3 токена
- 📡 3 активные подписки на обновления
- 🚨 5 сгенерированных алертов

### 2. **Анализ прибыльности**
```
💰 Profit Calculation Results:
   Gross Profit: 0.020000 SOL
   DEX Fees: 0.005500 SOL
   Gas Cost: 0.000200 SOL
   Net Profit: 0.014300 SOL
   Profit Percentage: 1.43%
```

**Что это означает:**
- 💰 Валовая прибыль: 0.02 SOL
- 💸 Комиссии DEX: 0.0055 SOL
- ⛽ Газ: 0.0002 SOL
- 💎 Чистая прибыль: 0.0143 SOL
- 📈 Процент прибыли: 1.43%

### 3. **Анализ транзакций**
```
📋 Transaction Details:
   Type: TwoHop Arbitrage
   Route: Orca Whirlpool -> Raydium V4 -> Orca Whirlpool
   Expected Profit: ~2% (minus fees)
   Estimated Gas: ~200k compute units
   Priority Fee: 50k lamports
```

**Что это означает:**
- 🎯 Тип: Двухшаговый арбитраж
- 🛣️ Маршрут: Orca → Raydium → Orca
- 💰 Ожидаемая прибыль: ~2%
- ⛽ Оценка газа: ~200k compute units
- 🚀 Приоритетная комиссия: 50k lamports

## 🚨 Типичные сценарии ошибок

### 1. **Ошибки RPC**
```bash
❌ Connection failed: RPC endpoint unavailable
💡 Решение: Проверьте интернет и RPC URL
```

### 2. **Ошибки парсинга**
```bash
❌ Failed to parse pool account: Invalid data
💡 Решение: Обновите структуры данных
```

### 3. **Ошибки симуляции**
```bash
❌ Transaction simulation failed: Insufficient funds
💡 Решение: Проверьте баланс кошелька
```

## 🔍 Отладка и логирование

### Включение детального логирования
```bash
# Запуск с debug логированием
RUST_LOG=debug cargo run -- test-realtime --duration 10

# Запуск с trace логированием
RUST_LOG=trace cargo run -- test-arbitrage-engine --duration 20
```

### Анализ логов
```bash
# Фильтрация по типу
cargo run -- test-realtime --duration 30 2>&1 | grep "ALERT"

# Поиск ошибок
cargo run -- test-arbitrage-engine --duration 45 2>&1 | grep "ERROR"
```

## 📈 Оптимизация производительности

### 1. **Настройка интервалов**
```bash
# Быстрый мониторинг
cargo run -- test-realtime --duration 60

# Медленный мониторинг (экономия ресурсов)
# Измените update_interval_ms в конфигурации
```

### 2. **Оптимизация газа**
```bash
# Тест с разными compute units
cargo run -- test-transaction-execution --amount 0.1 --simulate

# Анализируйте gas usage в результатах
```

---

**💡 Используйте эти примеры для изучения и настройки бота!**
