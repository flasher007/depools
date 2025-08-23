//! Common types used across the application

use solana_sdk::pubkey::Pubkey;
use serde::{Deserialize, Serialize};

/// Token representation
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Token {
    pub mint: Pubkey,
    pub symbol: String,
    pub decimals: u8,
    pub name: Option<String>,
}

/// Amount representation with precision
#[derive(Debug, Clone, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Amount {
    pub value: u64,
    pub decimals: u8,
}

impl Amount {
    pub fn new(value: u64, decimals: u8) -> Self {
        Self { value, decimals }
    }

    pub fn from_lamports(value: u64) -> Self {
        Self { value, decimals: 9 }
    }

    pub fn from_sol(value: f64) -> Self {
        Self {
            value: (value * 1_000_000_000.0) as u64,
            decimals: 9,
        }
    }

    pub fn to_sol(&self) -> f64 {
        self.value as f64 / 10_f64.powi(self.decimals as i32)
    }

    pub fn to_lamports(&self) -> u64 {
        self.value
    }
}

/// Price representation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Price {
    pub value: f64,
    pub base_token: Token,
    pub quote_token: Token,
}

impl Price {
    pub fn new(value: f64, base_token: Token, quote_token: Token) -> Self {
        Self {
            value,
            base_token,
            quote_token,
        }
    }
}

/// Network configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    pub rpc_url: String,
    pub ws_url: Option<String>,
    pub commitment: String,
    pub timeout_ms: u64,
}

/// Bot configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BotConfig {
    pub network: NetworkConfig,
    pub min_profit_threshold: f64,
    pub max_slippage: f64,
    pub max_gas_price: u64,
    pub execution_delay_ms: u64,
    pub retry_attempts: u32,
}

impl Default for BotConfig {
    fn default() -> Self {
        Self {
            network: NetworkConfig {
                rpc_url: "https://api.mainnet-beta.solana.com".to_string(),
                ws_url: None,
                commitment: "confirmed".to_string(),
                timeout_ms: 30000,
            },
            min_profit_threshold: 0.5,
            max_slippage: 1.0,
            max_gas_price: 1000,
            execution_delay_ms: 100,
            retry_attempts: 3,
        }
    }
}
