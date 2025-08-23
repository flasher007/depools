//! Depools - Solana Arbitrage Bot v2
//! Built with Domain-Driven Design principles

pub mod domain;
pub mod infrastructure;
pub mod application;
pub mod shared;

// Re-export main types for convenience
pub use domain::arbitrage::ArbitrageEngine;
pub use domain::dex::DexRegistry;
pub use domain::pool::PoolManager;
pub use domain::price::PriceMonitor;
pub use domain::execution::TransactionExecutor;
