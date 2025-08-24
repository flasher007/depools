//! Direct blockchain reading for Solana DEX data

pub mod pool_discovery;
pub mod account_parser;
pub mod rpc_client;
pub mod orca_structures;
pub mod raydium_structures;
pub mod dex_structures;
pub mod vault_reader;
pub mod token_metadata;
pub mod profit_calculator;
pub mod yellowstone_grpc;
pub mod realtime_monitor;
pub mod realtime_arbitrage;
pub mod transaction_executor;

pub use pool_discovery::PoolDiscoveryService;
pub use account_parser::{OrcaAccountParser, RaydiumAccountParser};
pub use rpc_client::SolanaRpcClient;
pub use orca_structures::Whirlpool;
pub use raydium_structures::RaydiumV4Pool;
pub use vault_reader::VaultReader;
pub use token_metadata::TokenMetadataService;
pub use profit_calculator::RealProfitCalculator;
pub use yellowstone_grpc::{YellowstoneGrpcClient, PriceData};
pub use realtime_monitor::{RealtimePriceMonitor, MonitorConfig, PriceAlert, AlertType};
pub use realtime_arbitrage::{RealtimeArbitrageEngine, AutoExecutionConfig, ArbitrageOpportunity, OpportunityStatus};
pub use transaction_executor::{RealTransactionExecutor, ExecutionConfig, ExecutionResult};
