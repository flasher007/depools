//! Orca Whirlpool account structures

use borsh::{BorshDeserialize, BorshSerialize};
use solana_sdk::pubkey::Pubkey;

/// Orca Whirlpool account discriminator
pub const WHIRLPOOL_DISCRIMINATOR: [u8; 8] = [0x3f, 0x95, 0xd1, 0x0c, 0xe1, 0x80, 0x63, 0x09];

/// Orca Whirlpool pool account structure (raw data approach)
#[derive(Debug, Clone)]
pub struct Whirlpool {
    /// Raw account data
    pub raw_data: Vec<u8>,
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
    /// Fee rate
    pub fee_rate: u16,
    /// Protocol fee rate
    pub protocol_fee_rate: u16,
    /// Tick spacing
    pub tick_spacing: u16,
    /// Current sqrt price
    pub sqrt_price: u128,
    /// Current tick index
    pub tick_current_index: i32,
    /// Liquidity
    pub liquidity: u128,
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
    
    /// Try to deserialize account data into Whirlpool using raw data approach
    pub fn try_deserialize(data: &[u8]) -> Result<Self, borsh::maybestd::io::Error> {
        if !Self::is_valid_whirlpool(data) {
            return Err(borsh::maybestd::io::Error::new(
                borsh::maybestd::io::ErrorKind::InvalidData,
                "Invalid Whirlpool account data"
            ));
        }
        
        if data.len() < 235 {
            return Err(borsh::maybestd::io::Error::new(
                borsh::maybestd::io::ErrorKind::InvalidData,
                "Insufficient data length"
            ));
        }
        
        // Extract fields from raw data
        let discriminator: [u8; 8] = data[0..8].try_into().unwrap();
        let nonce = data[8];
        let whirlpools_config = Pubkey::new_from_array(data[9..41].try_into().unwrap());
        let token_mint_a = Pubkey::new_from_array(data[41..73].try_into().unwrap());
        let token_mint_b = Pubkey::new_from_array(data[73..105].try_into().unwrap());
        let token_vault_a = Pubkey::new_from_array(data[105..137].try_into().unwrap());
        let token_vault_b = Pubkey::new_from_array(data[137..169].try_into().unwrap());
        let fee_rate = u16::from_le_bytes([data[169], data[170]]);
        let protocol_fee_rate = u16::from_le_bytes([data[171], data[172]]);
        let tick_spacing = u16::from_le_bytes([data[173], data[174]]);
        let sqrt_price = u128::from_le_bytes(data[175..191].try_into().unwrap());
        let tick_current_index = i32::from_le_bytes([data[191], data[192], data[193], data[194]]);
        let liquidity = u128::from_le_bytes(data[195..211].try_into().unwrap());
        
        Ok(Self {
            raw_data: data.to_vec(),
            discriminator,
            nonce,
            whirlpools_config,
            token_mint_a,
            token_mint_b,
            token_vault_a,
            token_vault_b,
            fee_rate,
            protocol_fee_rate,
            tick_spacing,
            sqrt_price,
            tick_current_index,
            liquidity,
        })
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
