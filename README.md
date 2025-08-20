# 🚀 Depools - AMM Arbitrage Engine

Профессиональный CLI-инструмент для арбитража между AMM пулами на Solana.

## ✨ **Возможности**

- **🔍 Сканирование пулов** - Автоматический поиск арбитражных возможностей
- **💱 Поддержка DEX** - Raydium V4, Orca Whirlpool
- **⚡ Атомарные транзакции** - Два свопа в одной транзакции
- **🛡️ Защита от slippage** - Настраиваемые лимиты
- **📊 Детальная отчетность** - JSON отчеты с расчетами
- **🎯 Гибкая конфигурация** - CLI аргументы + конфиг файлы

## 🚀 **Быстрый старт**

### **Установка**
```bash
git clone <repository>
cd depools
cargo build --release
```

### **Базовое использование**
```bash
# С конфигурационным файлом
cargo run --bin depools -- --config Config.toml --simulate-only

# Только CLI аргументы
cargo run --bin depools -- \
  --rpc-url "https://mainnet.helius-rpc.com/?api-key=YOUR_KEY" \
  --keypair "path/to/keypair.json" \
  --amount-in 1000000.0 \
  --simulate-only
```

## ⚙️ **Конфигурация**

### **Config.toml**
```toml
[rpc]
url = "https://mainnet.helius-rpc.com/?api-key=YOUR_KEY"

[wallet]
keypair = "test-keypair.json"

[tokens]
base_token = { mint = "So11111111111111111111111111111111111111112", symbol = "SOL", decimals = 9 }
quote_token = { mint = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v", symbol = "USDC", decimals = 6 }

[pools]
pool_a = "58oQChx4yWmvKdwLLZzBi4ChoCc2fqCUWBkwMihLYQo2"
pool_b = "HJPjoWUrhoZzkNfRpHuieeFk9WcZWjwy6PBjZ81ngndJ"

[trade]
amount_in = 1000000.0
spread_threshold_bps = 50
slippage_bps = 100
priority_fee_microlamports = 1000
simulate_only = true

[programs]
raydium_v4 = "675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8"
orca_whirlpool = "whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc"
spl_token = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
```

### **CLI аргументы**
```bash
--rpc-url <URL>                    # RPC эндпоинт
--keypair <PATH>                   # Путь к keypair файлу
--amount-in <AMOUNT>               # Сумма для торговли
--spread-threshold-bps <BPS>       # Минимальный спред (по умолчанию: 50)
--slippage-bps <BPS>               # Slippage tolerance (по умолчанию: 100)
--priority-fee <FEE>               # Priority fee в microlamports (по умолчанию: 1000)
--simulate-only                     # Только симуляция (без исполнения)
--config <PATH>                     # Путь к конфигурационному файлу
--pools <ADDRESSES>                # Адреса пулов (через запятую)

# Переопределение токенов и программ
--base-token-mint <ADDRESS>        # Base token mint address
--quote-token-mint <ADDRESS>       # Quote token mint address
--raydium-program <ID>             # Raydium V4 program ID
--orca-program <ID>                # Orca Whirlpool program ID
--spl-token-program <ID>           # SPL Token program ID
```

## 🔧 **Архитектура**

```
src/
├── app.rs              # Основная логика приложения
├── config.rs           # Конфигурация и CLI парсинг
├── exchanges/          # Адаптеры для DEX
│   ├── raydium_v4.rs   # Raydium V4 интеграция
│   ├── orca_whirlpool.rs # Orca Whirlpool интеграция
│   ├── types.rs        # Общие типы для DEX
│   └── mod.rs          # Factory для создания адаптеров
├── opportunity/        # Сканер арбитражных возможностей
│   └── scanner.rs      # Основная логика сканирования
└── report.rs           # Генерация отчетов
```

## 📊 **Примеры использования**

### **Тестирование на mainnet**
```bash
cargo run --bin depools -- --config Config.toml --simulate-only
```

### **Переопределение RPC**
```bash
cargo run --bin depools -- --config Config.toml \
  --rpc-url "https://api.mainnet-beta.solana.com" \
  --simulate-only
```

### **Кастомные пулы**
```bash
cargo run --bin depools -- --config Config.toml \
  --pools "POOL_A_ADDRESS,POOL_B_ADDRESS" \
  --simulate-only
```

### **Переопределение токенов**
```bash
cargo run --bin depools -- --config Config.toml \
  --base-token-mint "CUSTOM_TOKEN_MINT" \
  --quote-token-mint "CUSTOM_QUOTE_MINT" \
  --simulate-only
```

## ⚠️ **Важные замечания**

1. **Всегда используйте `--simulate-only` для тестирования**
2. **Проверяйте конфигурацию перед запуском**
3. **Используйте надежные RPC эндпоинты**
4. **Храните keypair файлы в безопасном месте**

## 🎯 **Требования**

- Rust 1.70+
- Solana CLI tools
- Доступ к Solana RPC (mainnet/devnet)

## 📝 **Лицензия**

MIT License

---

**Готово к использованию!** 🚀
