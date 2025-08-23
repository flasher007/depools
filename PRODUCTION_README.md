# Depools - Solana Arbitrage Bot v2

**Production-ready Solana arbitrage bot for mainnet with real blockchain integration**

## 🚀 Overview

Depools is a production-ready Solana arbitrage bot that discovers and executes profitable trading opportunities across multiple DEXes on Solana mainnet. Built with Domain-Driven Design principles, it provides real-time monitoring, pool discovery, and arbitrage execution capabilities.

## ✨ Features

- **Real Blockchain Integration**: Direct reading from Solana mainnet
- **Multi-DEX Support**: Orca Whirlpool, Raydium V4/AMM/CLMM, Meteora DLMM/Pools, PumpSwap, Jupiter
- **Live Price Monitoring**: Real-time price feeds via Yellowstone gRPC
- **Pool Discovery**: Automatic discovery of liquidity pools across all supported DEXes
- **Arbitrage Engine**: Advanced arbitrage detection and execution
- **Risk Management**: Configurable slippage, profit thresholds, and position limits
- **Production Ready**: No mocks, demo data, or test configurations

## 🏗️ Architecture

```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│   Application   │    │     Domain       │    │ Infrastructure  │
│   Layer         │◄──►│     Layer        │◄──►│     Layer       │
│                 │    │                  │    │                 │
│ • Commands      │    │ • Arbitrage      │    │ • Blockchain    │
│ • Services      │    │ • DEX            │    │ • RPC Client    │
│ • Config        │    │ • Pool           │    │ • gRPC Client   │
└─────────────────┘    │ • Price         │    │ • Storage       │
                       └──────────────────┘    └─────────────────┘
```

## 🚦 Prerequisites

- Rust 1.70+ 
- Solana CLI tools
- Access to Solana mainnet RPC (Helius, QuickNode, etc.)
- Yellowstone gRPC access (optional, for real-time data)

## 📦 Installation

1. **Clone the repository**
   ```bash
   git clone <repository-url>
   cd depools
   ```

2. **Install dependencies**
   ```bash
   cargo build --release
   ```

3. **Configure the bot**
   ```bash
   cp Config.toml.example Config.toml
   # Edit Config.toml with your settings
   ```

## ⚙️ Configuration

### Essential Settings

```toml
[network]
rpc_url = "https://mainnet.helius-rpc.com/?api-key=YOUR_API_KEY"
ws_url = "wss://mainnet.helius-rpc.com/?api-key=YOUR_API_KEY"

# Arbitrage Settings
min_profit_threshold = 0.5  # Minimum profit percentage
max_slippage = 1.0          # Maximum slippage tolerance

# Development Settings (MUST be false for production)
[development]
use_devnet = false
simulation_mode = false
use_mock_data = false
fast_mode = false
```

### RPC Endpoints

**Recommended (Paid):**
- Helius: `https://mainnet.helius-rpc.com/?api-key=YOUR_KEY`
- QuickNode: `https://your-endpoint.solana-mainnet.quiknode.pro/YOUR_KEY/`
- Alchemy: `https://solana-mainnet.g.alchemy.com/v2/YOUR_KEY`

**Free (Limited):**
- Solana: `https://api.mainnet-beta.solana.com`
- GenesysGo: `https://ssc-dao.genesysgo.net`

## 🎯 Usage

### 1. Pool Discovery

Discover all available pools across supported DEXes:

```bash
# Discover pools and save to files
cargo run -- pools --save --min-liquidity 10.0

# Discover pools using blockchain reading
cargo run -- discover --save
```

### 2. Price Monitoring

Monitor prices in real-time:

```bash
# Monitor prices with custom interval
cargo run -- monitor --interval 1000 --threshold 0.1

# Real-time price monitoring
cargo run -- monitor-realtime --duration 300
```

### 3. Arbitrage Detection

Run the arbitrage engine:

```bash
# Start arbitrage bot
cargo run -- start --min-profit 0.5 --max-slippage 1.0

# Run arbitrage engine
cargo run -- run-engine --duration 300 --auto-execute
```

### 4. Transaction Execution

Execute arbitrage transactions:

```bash
# Simulate transaction first
cargo run -- execute-transaction --amount 0.1 --simulate

# Execute real transaction
cargo run -- execute-transaction --amount 0.1
```

## 🔍 Pool Discovery

The bot automatically discovers pools by:

1. **Reading Program Accounts**: Direct blockchain queries to DEX program IDs
2. **Parsing Pool Data**: Real-time parsing of pool account structures
3. **Validating Liquidity**: Filtering pools by minimum liquidity requirements
4. **Cross-Reference**: Matching pools across different DEXes for arbitrage opportunities

### Supported DEXes

| DEX | Program ID | Type | Status |
|-----|------------|------|--------|
| Orca Whirlpool | `whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc` | CLMM | ✅ Active |
| Raydium V4 | `675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8` | Legacy AMM | ✅ Active |
| Raydium AMM | `9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM` | AMM | ✅ Active |
| Raydium CLMM | `CAMMCzo5YL8w4VFF8KVHrK22GGUQp5VhH5bMKM3p9bkt` | CLMM | ✅ Active |
| Meteora DLMM | `LBUZKhRxPF3XUpBCjp4YzTKgLccjZhTSDM9YuVaPwxo` | DLMM | ✅ Active |
| Meteora Pools | `Eo7WjKq67rjJQSZxS6z3YkapzY3eMj6Xy8X5EQVn5UaB` | Traditional | ✅ Active |

## 💰 Arbitrage Strategies

### Two-Hop Arbitrage
```
Token A → Token B [DEX 1] → Token A [DEX 2]
```

### Triangle Arbitrage
```
Token A → Token B [DEX 1]
Token B → Token C [DEX 2]  
Token C → Token A [DEX 3]
```

### Multi-DEX Arbitrage
```
Token A → Token B [Multiple DEXes] → Token A
```

## 🛡️ Risk Management

- **Profit Thresholds**: Minimum profit requirements before execution
- **Slippage Protection**: Maximum acceptable slippage per trade
- **Liquidity Requirements**: Minimum pool liquidity thresholds
- **Position Limits**: Maximum position sizes per trade
- **Stop Loss**: Automatic stop-loss mechanisms

## 📊 Monitoring & Alerts

- **Real-time Price Feeds**: Live price updates via WebSocket/gRPC
- **Pool Health Monitoring**: Liquidity and volume tracking
- **Arbitrage Opportunity Alerts**: Instant notifications for profitable trades
- **Performance Metrics**: Trade success rates and profitability tracking

## 🚨 Production Checklist

Before running in production:

- [ ] `fast_mode = false` in Config.toml
- [ ] `simulation_mode = false` in Config.toml
- [ ] `use_mock_data = false` in Config.toml
- [ ] Mainnet RPC endpoint configured
- [ ] Proper API keys and rate limits
- [ ] Risk parameters configured
- [ ] Monitoring and alerting set up
- [ ] Backup RPC endpoints configured

## 🔧 Troubleshooting

### Common Issues

1. **RPC Rate Limiting**
   - Use paid RPC services for production
   - Implement rate limiting in your configuration

2. **Pool Discovery Failures**
   - Check RPC endpoint connectivity
   - Verify program IDs are correct
   - Check account data parsing

3. **Transaction Failures**
   - Verify wallet has sufficient SOL for fees
   - Check slippage settings
   - Ensure pool has sufficient liquidity

### Debug Mode

Enable debug logging:

```bash
export RUST_LOG=debug
cargo run -- <command>
```

## 📈 Performance Optimization

- **Concurrent Pool Discovery**: Parallel processing of multiple DEXes
- **Smart Caching**: Cache frequently accessed pool data
- **Connection Pooling**: Reuse RPC connections
- **Batch Processing**: Group multiple operations together

## 🔐 Security Considerations

- **Private Key Management**: Never hardcode private keys
- **RPC Security**: Use HTTPS endpoints only
- **Rate Limiting**: Respect API rate limits
- **Error Handling**: Graceful handling of network failures

## 📞 Support

For production support and issues:

- Check logs for detailed error messages
- Verify configuration settings
- Test with small amounts first
- Monitor RPC endpoint health

## 📄 License

This project is licensed under the MIT License - see the LICENSE file for details.

---

**⚠️ DISCLAIMER: This bot is for educational and production use. Always test thoroughly and use at your own risk. Cryptocurrency trading involves substantial risk of loss.**
