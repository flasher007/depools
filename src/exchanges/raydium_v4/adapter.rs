use anyhow::Result;
use solana_sdk::{
    pubkey::Pubkey,
    instruction::{Instruction, AccountMeta},
};
use std::str::FromStr;
use solana_client::rpc_client::RpcClient;
use std::sync::Arc;
use crate::config::Config;
use crate::exchanges::{DexAdapter, types::{DexLabel, PoolInfo, SwapQuote, TokenInfo, PoolReserves, PoolFees, PoolState, SwapRoute, SwapHop}};
use crate::exchanges::utils::{lamports_to_sol, lamports_to_usdc, format_sol, format_usdc, format_large_number};
use super::RaydiumV4Parser;
use tracing::info;
use crate::exchanges::common::spl_token_balance;

pub struct RaydiumV4Adapter {
    rpc_client: Arc<RpcClient>,
    config: Config,
}

impl RaydiumV4Adapter {
    pub fn new(config: Config) -> Result<Self> {
        Ok(Self {
            rpc_client: Arc::new(RpcClient::new(config.rpc.url.clone())),
            config,
        })
    }

    async fn fetch_pool_data(&self, pool_address: &Pubkey) -> Result<Vec<u8>> {
        use tracing::{info, error};
        
        info!("Fetching Raydium V4 pool data for: {}", pool_address);
        
        match self.rpc_client.get_account(pool_address) {
            Ok(account) => {
                info!("✅ Fetched {} bytes from Raydium V4 pool", account.data.len());
                
                // Verify account owner is Raydium V4 program
                let expected_program = &self.config.programs.raydium_v4;
                if account.owner.to_string() != *expected_program {
                    error!("❌ Invalid pool owner. Expected: {}, Got: {}", expected_program, account.owner);
                    return Err(anyhow::anyhow!(
                        "Invalid pool owner. Expected: {}, Got: {}", 
                        expected_program, account.owner
                    ));
                }
                
                Ok(account.data)
            }
            Err(e) => {
                error!("Failed to fetch Raydium V4 pool data: {}", e);
                Err(anyhow::anyhow!("Failed to fetch pool data: {}", e))
            }
        }
    }

    /// Parse pool data and return token info and fees
    fn parse_pool_data(&self, data: &[u8]) -> Result<(TokenInfo, TokenInfo, PoolReserves, PoolFees)> {
        use tracing::info;
        
        info!("📊 Parsing Raydium V4 pool data ({} bytes)", data.len());
        
        // Используем парсер для реальных данных
        let parser = RaydiumV4Parser;
        let (token_a, token_b, reserves, fees) = parser.parse_pool_data(data)?;
        
        info!("🪙 Parsed tokens: {} ({}) ↔ {} ({})", 
              token_a.symbol, token_a.mint, token_b.symbol, token_b.mint);
        
        // Note: Reserves will be fetched from vault accounts later
        // Initial parse shows 0, real values come from RPC calls
        
        Ok((token_a, token_b, reserves, fees))
    }

    /// Get token account balance
    async fn get_token_account_balance(&self, vault: &Pubkey) -> Result<u64> {
        let account = self.rpc_client.get_account(vault)?;
        
        // Parse token account balance (position 64-71 in token account data)
        if account.data.len() >= 72 {
            let balance = u64::from_le_bytes([
                account.data[64], account.data[65], account.data[66], account.data[67],
                account.data[68], account.data[69], account.data[70], account.data[71]
            ]);
            Ok(balance)
        } else {
            Ok(0)
        }
    }
}

#[async_trait::async_trait]
impl DexAdapter for RaydiumV4Adapter {
    fn get_label(&self) -> DexLabel {
        DexLabel::RaydiumV4
    }

    async fn get_pool_info(&self, pool_address: &Pubkey) -> Result<PoolInfo> {
        let data = self.fetch_pool_data(pool_address).await?;
        let (token_a, token_b, mut reserves, fees) = self.parse_pool_data(&data)?;
        
        // Fetch real-time reserves from vault accounts
        if let Ok(base_reserve) = spl_token_balance(&self.rpc_client, &token_a.vault).await {
            reserves.token_a_reserve = base_reserve;
            info!("✅ Fetched base reserve: {} {}", base_reserve, token_a.symbol);
        } else {
            info!("⚠️ Failed to fetch base reserve for vault: {}", token_a.vault);
        }
        
        if let Ok(quote_reserve) = spl_token_balance(&self.rpc_client, &token_b.vault).await {
            reserves.token_b_reserve = quote_reserve;
            info!("✅ Fetched quote reserve: {} {}", quote_reserve, token_b.symbol);
        } else {
            info!("⚠️ Failed to fetch quote reserve for vault: {}", token_b.vault);
        }
        
        Ok(PoolInfo {
            pool_address: *pool_address,
            dex_label: self.get_label(),
            token_a,
            token_b,
            reserves,
            fees,
            pool_state: crate::exchanges::types::PoolState::Active,
        })
    }

    async fn get_swap_quote(&self, pool_address: &Pubkey, amount_in: u64, token_in: &Pubkey) -> Result<SwapQuote> {
        let pool_info = self.get_pool_info(pool_address).await?;
        
        let (token_in_info, token_out_info, _amount_out) = if token_in == &pool_info.token_a.mint {
            (&pool_info.token_a, &pool_info.token_b, pool_info.reserves.token_b_reserve)
        } else if token_in == &pool_info.token_b.mint {
            (&pool_info.token_b, &pool_info.token_a, pool_info.reserves.token_a_reserve)
        } else {
            return Err(anyhow::anyhow!("Token {} not found in pool", token_in));
        };
        
        // Correct AMM calculation using Constant Product Formula: (x + dx) * (y - dy) = x * y
        let (reserve_in, reserve_out) = if token_in == &pool_info.token_a.mint {
            (pool_info.reserves.token_a_reserve, pool_info.reserves.token_b_reserve)
        } else {
            (pool_info.reserves.token_b_reserve, pool_info.reserves.token_a_reserve)
        };
        
        // Add detailed logging for debugging
        info!("🔍 AMM Calculation Debug:");
        
        // Convert to readable units
        let amount_in_sol = lamports_to_sol(amount_in);
        let reserve_in_sol = lamports_to_sol(reserve_in);
        let reserve_out_usdc = lamports_to_usdc(reserve_out);
        
        info!("  Token In: {} ({})", token_in_info.symbol, format_sol(amount_in_sol));
        info!("  Reserve In: {} ({})", format_sol(reserve_in_sol), format_large_number(reserve_in));
        info!("  Reserve Out: {} ({})", format_usdc(reserve_out_usdc), format_large_number(reserve_out));
        info!("  Token In Decimals: {}", token_in_info.decimals);
        info!("  Token Out Decimals: {}", token_out_info.decimals);
        
        let reserve_in_u128 = reserve_in as u128;
        let reserve_out_u128 = reserve_out as u128;
        let amount_in_u128 = amount_in as u128;
        
        // AMM formula: dy = (y * dx) / (x + dx)
        let amount_out_raw = (reserve_out_u128 * amount_in_u128) / (reserve_in_u128 + amount_in_u128);
        
        let amount_out = amount_out_raw as u64; // Convert back to u64
        let amount_out_usdc = lamports_to_usdc(amount_out);
        
        info!("  AMM Formula: dy = ({} * {}) / ({} + {}) = {}", 
              format_usdc(reserve_out_usdc), format_sol(amount_in_sol), 
              format_sol(reserve_in_sol), format_sol(amount_in_sol), 
              format_usdc(amount_out_usdc));
        info!("  Final Amount Out: {} ({})", format_usdc(amount_out_usdc), format_large_number(amount_out));
        
        let fee_bps = pool_info.fees.trade_fee_bps;
        let fee_amount = (amount_in as u128 * fee_bps as u128 / 10000) as u64;
        let fee_sol = lamports_to_sol(fee_amount);
        
        info!("  Fee: {} bps = {} ({})", fee_bps, format_sol(fee_sol), format_large_number(fee_amount));
        
        Ok(SwapQuote {
            pool_address: *pool_address,
            dex_label: self.get_label(),
            token_in: token_in_info.mint, // Use token_in_info.mint
            token_out: token_out_info.mint,
            amount_in,
            amount_out,
            min_amount_out: 0, // Placeholder, will be calculated later
            price_impact_bps: 0, // Placeholder
            fee_amount,
            route: SwapRoute {
                hops: vec![SwapHop {
                    pool_address: *pool_address,
                    dex_label: self.get_label(),
                    token_in: token_in_info.mint, // Use token_in_info.mint
                    token_out: token_out_info.mint,
                    amount_in,
                    amount_out,
                    fee_bps,
                }],
                total_fee_bps: fee_bps,
            },
        })
    }

    async fn get_pool_reserves(&self, pool_address: &Pubkey) -> Result<crate::exchanges::types::PoolReserves> {
        // Get pool info to access vault addresses
        let pool_info = self.get_pool_info(pool_address).await?;
        
        // Fetch balances from vault accounts
        let base_balance = self.get_token_account_balance(&pool_info.token_a.vault).await?;
        let quote_balance = self.get_token_account_balance(&pool_info.token_b.vault).await?;
        
        Ok(crate::exchanges::types::PoolReserves {
            token_a_reserve: base_balance,
            token_b_reserve: quote_balance,
            lp_supply: None, // TODO: Implement LP supply fetching
        })
    }

    fn create_swap_instruction(
        &self, 
        quote: &SwapQuote, 
        user_pubkey: &Pubkey,
        min_amount_out: u64,
    ) -> Result<Instruction> {
        // Raydium V4 swap instruction structure
        // This is a simplified implementation - real Raydium instructions are more complex
        
        use tracing::info;
        info!("Creating Raydium V4 swap instruction: {} -> {}, min_out: {}", 
              quote.amount_in, quote.amount_out, min_amount_out);
        
        // Get pool info to access vault accounts
        let pool_address = quote.pool_address;
        
        // Create instruction data for Raydium V4 swap
        // Instruction: 0x09 (swap), amount_in (8 bytes), min_amount_out (8 bytes)
        let mut data = Vec::new();
        data.push(0x09); // Swap instruction discriminator
        data.extend_from_slice(&quote.amount_in.to_le_bytes());
        data.extend_from_slice(&min_amount_out.to_le_bytes());
        
        let accounts = vec![
            // User accounts
            AccountMeta::new(*user_pubkey, true), // User wallet (signer)
            AccountMeta::new(*user_pubkey, false), // User source token account
            AccountMeta::new(*user_pubkey, false), // User destination token account
            
            // Pool accounts
            AccountMeta::new(pool_address, false), // AMM pool
            AccountMeta::new(quote.token_in, false), // Pool token A vault
            AccountMeta::new(quote.token_out, false), // Pool token B vault
            
            // Program accounts
            AccountMeta::new_readonly(spl_token::id(), false), // SPL Token program
            AccountMeta::new_readonly(Pubkey::from_str("675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8")?, false), // Raydium program ID
        ];
        
        Ok(Instruction {
            program_id: Pubkey::from_str("675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8")?, // Raydium V4 program
            accounts,
            data,
        })
    }
}
