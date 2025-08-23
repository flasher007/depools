//! Raydium V4 account structures

use borsh::{BorshDeserialize, BorshSerialize};
use solana_sdk::pubkey::Pubkey;

/// Raydium V4 account discriminator
pub const RAYDIUM_V4_DISCRIMINATOR: [u8; 8] = [0x0c, 0x0c, 0x0c, 0x0c, 0x0c, 0x0c, 0x0c, 0x0c];

/// Raydium V4 pool account structure
#[derive(BorshDeserialize, BorshSerialize, Debug, Clone)]
pub struct RaydiumV4Pool {
    /// Account discriminator
    pub discriminator: [u8; 8],
    /// Nonce used in deriving pool address
    pub nonce: u8,
    /// The Raydium program ID
    pub raydium_program: Pubkey,
    /// Token mint A
    pub token_mint_a: Pubkey,
    /// Token mint B
    pub token_mint_b: Pubkey,
    /// Token vault A
    pub token_vault_a: Pubkey,
    /// Token vault B
    pub token_vault_b: Pubkey,
    /// Fee rate (basis points)
    pub fee_rate: u16,
    /// Protocol fee rate (basis points)
    pub protocol_fee_rate: u16,
    /// Current sqrt price
    pub sqrt_price: u128,
    /// Current tick index
    pub tick_current_index: i32,
    /// Liquidity
    pub liquidity: u128,
    /// Protocol fee owed A
    pub protocol_fee_owed_a: u64,
    /// Protocol fee owed B
    pub protocol_fee_owed_b: u64,
    /// Pool bump seed
    pub pool_bump: [u8; 1],
}

impl RaydiumV4Pool {
    /// Check if account data represents a valid Raydium V4 pool
    pub fn is_valid_raydium_pool(data: &[u8]) -> bool {
        if data.len() < 8 {
            return false;
        }
        
        let discriminator: [u8; 8] = data[0..8].try_into().unwrap_or([0; 8]);
        discriminator == RAYDIUM_V4_DISCRIMINATOR
    }
    
    /// Try to deserialize account data into RaydiumV4Pool
    pub fn try_deserialize(data: &[u8]) -> Result<Self, borsh::maybestd::io::Error> {
        if !Self::is_valid_raydium_pool(data) {
            return Err(borsh::maybestd::io::Error::new(
                borsh::maybestd::io::ErrorKind::InvalidData,
                "Invalid Raydium V4 pool account data"
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
