//! Orca Whirlpool account structures

use borsh::{BorshDeserialize, BorshSerialize};
use solana_sdk::pubkey::Pubkey;

/// Orca Whirlpool account discriminator
pub const WHIRLPOOL_DISCRIMINATOR: [u8; 8] = [0x0b, 0x0b, 0x0b, 0x0b, 0x0b, 0x0b, 0x0b, 0x0b];

/// Orca Whirlpool pool account structure
#[derive(BorshDeserialize, BorshSerialize, Debug, Clone)]
pub struct Whirlpool {
    /// Account discriminator
    pub discriminator: [u8; 8],
    /// Nonce used in deriving whirlpool address
    pub nonce: u8,
    /// The whirlpool program ID
    pub whirlpools_config: Pubkey,
    /// Token mint A
    pub token_mint_a: Pubkey,
    /// Token mint B
    pub token_mint_b: Pubkey,
    /// Token vault A
    pub token_vault_a: Pubkey,
    /// Token vault B
    pub token_vault_b: Pubkey,
    /// Tick array bitmap
    pub tick_array_bitmap: [u64; 16],
    /// Fee growth global A
    pub fee_growth_global_a: u128,
    /// Fee growth global B
    pub fee_growth_global_b: u128,
    /// Protocol fee rate
    pub protocol_fee_rate: u16,
    /// Fee rate
    pub fee_rate: u16,
    /// Tick spacing
    pub tick_spacing: u16,
    /// Current sqrt price
    pub sqrt_price: u128,
    /// Current tick index
    pub tick_current_index: i32,
    /// Observation index
    pub observation_index: u16,
    /// Observation cardinality
    pub observation_cardinality: u16,
    /// Observation cardinality next
    pub observation_cardinality_next: u16,
    /// Maximum observation cardinality
    pub max_observation_cardinality: u16,
    /// Protocol fee owed A
    pub protocol_fee_owed_a: u64,
    /// Protocol fee owed B
    pub protocol_fee_owed_b: u64,
    /// Liquidity
    pub liquidity: u128,
    /// Fee growth checkpoint A
    pub fee_growth_checkpoint_a: u128,
    /// Fee growth checkpoint B
    pub fee_growth_checkpoint_b: u128,
    /// Reward infos
    pub reward_infos: [WhirlpoolRewardInfo; 3],
    /// Whirlpool bump seed
    pub whirlpool_bump: [u8; 1],
}

/// Orca Whirlpool reward info
#[derive(BorshDeserialize, BorshSerialize, Debug, Clone)]
pub struct WhirlpoolRewardInfo {
    /// Reward mint
    pub mint: Pubkey,
    /// Reward vault
    pub vault: Pubkey,
    /// Authority account that has permission to initialize the reward and set emissions
    pub authority: Pubkey,
    /// Q64.64 number that indicates how many tokens per second are earned per unit of liquidity
    pub emissions_per_second_x64: u128,
    /// Q64.64 number that tracks the total tokens earned per unit of liquidity since the reward emissions were turned on
    pub growth_global_x64: u128,
}

impl Whirlpool {
    /// Check if account data represents a valid Whirlpool
    pub fn is_valid_whirlpool(data: &[u8]) -> bool {
        if data.len() < 8 {
            return false;
        }
        
        let discriminator: [u8; 8] = data[0..8].try_into().unwrap_or([0; 8]);
        discriminator == WHIRLPOOL_DISCRIMINATOR
    }
    
    /// Try to deserialize account data into Whirlpool
    pub fn try_deserialize(data: &[u8]) -> Result<Self, borsh::maybestd::io::Error> {
        if !Self::is_valid_whirlpool(data) {
            return Err(borsh::maybestd::io::Error::new(
                borsh::maybestd::io::ErrorKind::InvalidData,
                "Invalid Whirlpool account data"
            ));
        }
        
        BorshDeserialize::try_from_slice(data)
    }
    
    /// Get token reserves from vault balances
    pub fn get_reserves(&self, vault_a_balance: u64, vault_b_balance: u64) -> (u64, u64) {
        (vault_a_balance, vault_b_balance)
    }
    
    /// Calculate fee rate as percentage
    pub fn get_fee_rate_percentage(&self) -> f64 {
        self.fee_rate as f64 / 10000.0
    }
    
    /// Calculate protocol fee rate as percentage
    pub fn get_protocol_fee_rate_percentage(&self) -> f64 {
        self.protocol_fee_rate as f64 / 10000.0
    }
}
