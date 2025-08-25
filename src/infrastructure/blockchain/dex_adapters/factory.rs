use std::sync::Arc;
use solana_client::rpc_client::RpcClient;

use super::traits::DexAdapter;
use super::orca_adapter::OrcaAdapter;
use super::raydium_adapter::RaydiumAMMAdapter;
use crate::domain::dex::DexType;
use crate::infrastructure::blockchain::{
    OrcaAccountParser, RaydiumAccountParser, VaultReader
};

/// Factory for creating DEX adapters
pub struct DexAdapterFactory {
    rpc_client: Arc<RpcClient>,
    vault_reader: Arc<VaultReader>,
    orca_parser: OrcaAccountParser,
    raydium_parser: RaydiumAccountParser,
}

impl DexAdapterFactory {
    pub fn new(
        rpc_client: Arc<RpcClient>,
        vault_reader: Arc<VaultReader>,
        orca_parser: OrcaAccountParser,
        raydium_parser: RaydiumAccountParser,
    ) -> Self {
        Self {
            rpc_client,
            vault_reader,
            orca_parser,
            raydium_parser,
        }
    }
    
    /// Create a DEX adapter for the specified DEX type
    pub fn create_adapter(&self, dex_type: &DexType) -> Box<dyn DexAdapter> {
        match dex_type {
            DexType::OrcaWhirlpool => {
                Box::new(OrcaAdapter::new(
                    Arc::clone(&self.rpc_client),
                    Arc::clone(&self.vault_reader),
                    self.orca_parser.clone(),
                ))
            },
            DexType::RaydiumAMM => {
                Box::new(RaydiumAMMAdapter::new(
                    Arc::clone(&self.vault_reader),
                ))
            },
        }
    }
    
    /// Get all available DEX types
    pub fn get_available_dex_types() -> Vec<DexType> {
        vec![
            DexType::OrcaWhirlpool,
            DexType::RaydiumAMM,
        ]
    }
}
