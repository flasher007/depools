use tracing::info;
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
use crate::exchanges::api_clients::{QuoteApiClient, orca_quote_client::OrcaQuoteApiClient};
use super::OrcaWhirlpoolParser;
use crate::exchanges::common::spl_token_balance;

pub struct OrcaWhirlpoolAdapter {
    rpc_client: Arc<RpcClient>,
    config: Config,
    api_client: OrcaQuoteApiClient,
}

impl OrcaWhirlpoolAdapter {
    pub fn new(config: Config) -> Result<Self> {
        Ok(Self {
            rpc_client: Arc::new(RpcClient::new(config.rpc.url.clone())),
            config,
            api_client: OrcaQuoteApiClient::new(),
        })
    }

    /// Попытаться получить котировку через API, если не получилось - использовать AMM логику
    async fn get_quote_with_fallback(&self, pool_pubkey: &Pubkey, amount_in: u64) -> Result<SwapQuote> {
        // Сначала пытаемся API
        if self.api_client.is_available().await {
            match self.api_client.get_quote(pool_pubkey, amount_in).await {
                Ok(quote) => {
                    info!("✅ Получена котировка через Orca API");
                    return Ok(quote);
                }
                Err(e) => {
                    info!("⚠️ Orca API недоступен: {}, используем AMM логику", e);
                }
            }
        }
        
        // Fallback на AMM логику
        info!("🔄 Используем AMM логику для расчета котировки");
        self.get_quote_from_amm(pool_pubkey, amount_in).await
    }
    
    /// Получить котировку используя AMM формулу (старая логика)
    async fn get_quote_from_amm(&self, pool_pubkey: &Pubkey, amount_in: u64) -> Result<SwapQuote> {
        let pool_info = self.get_pool_info_from_chain(pool_pubkey).await?;
        
        // Используем AMM формулу для concentrated liquidity
        let reserve_in = pool_info.reserves.token_a_reserve;
        let reserve_out = pool_info.reserves.token_b_reserve;
        
        if reserve_in == 0 || reserve_out == 0 {
            return Err(anyhow::anyhow!("Нулевые резервы в пуле"));
        }
        
        let fee_bps = pool_info.fees.trade_fee_bps;
        let fee_multiplier = 1.0 - (fee_bps as f64 / 10000.0);
        
        // Упрощенная AMM формула для concentrated liquidity
        let amount_out = ((amount_in as f64 * reserve_out as f64 * fee_multiplier) / 
                        (reserve_in as f64 + amount_in as f64)) as u64;
        
        let route = SwapRoute {
            hops: vec![crate::exchanges::types::SwapHop {
                pool_address: *pool_pubkey,
                dex_label: crate::exchanges::types::DexLabel::OrcaWhirlpool,
                token_in: pool_info.token_a.mint,
                token_out: pool_info.token_b.mint,
                amount_in,
                amount_out,
                fee_bps: fee_bps as u32,
            }],
            total_fee_bps: fee_bps as u32,
        };
        
        Ok(SwapQuote {
            pool_address: *pool_pubkey,
            dex_label: crate::exchanges::types::DexLabel::OrcaWhirlpool,
            token_in: pool_info.token_a.mint,
            token_out: pool_info.token_b.mint,
            amount_in,
            amount_out,
            min_amount_out: 0,
            price_impact_bps: 0,
            fee_amount: (amount_in as u128 * fee_bps as u128 / 10000) as u64,
            route,
        })
    }
    
    /// Получить информацию о пуле с блокчейна (старая логика)
    async fn get_pool_info_from_chain(&self, pool_address: &Pubkey) -> Result<PoolInfo> {
        let data = self.fetch_pool_data(pool_address).await?;
        let (token_a, token_b, mut reserves, fees) = self.parse_pool_data(&data)?;
        
        // Используем реальные vault адреса из парсера
        let balance_a = self.get_token_account_balance(&token_a.vault).await?;
        let balance_b = self.get_token_account_balance(&token_b.vault).await?;
        
        reserves.token_a_reserve = balance_a;
        reserves.token_b_reserve = balance_b;
        
        info!("💰 Orca Whirlpool резервы: {} {} ↔ {} {}", 
              format_sol(balance_a as f64), token_a.symbol, 
              format_usdc(balance_b as f64), token_b.symbol);
        
        Ok(PoolInfo {
            pool_address: *pool_address,
            dex_label: crate::exchanges::types::DexLabel::OrcaWhirlpool,
            token_a,
            token_b,
            reserves,
            fees,
            pool_state: crate::exchanges::types::PoolState::Active,
        })
    }

    async fn fetch_pool_data(&self, pool_address: &Pubkey) -> Result<Vec<u8>> {
        use tracing::{info, error};
        
        info!("Fetching Orca Whirlpool data for: {}", pool_address);
        
        match self.rpc_client.get_account(pool_address) {
            Ok(account) => {
                info!("✅ Fetched {} bytes from Orca Whirlpool", account.data.len());
                
                let expected_program = &self.config.programs.orca_whirlpool;
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
                error!("Failed to fetch Orca Whirlpool data: {}", e);
                Err(anyhow::anyhow!("Failed to fetch pool data: {}", e))
            }
        }
    }

    /// Parse pool data and return token info and fees
    fn parse_pool_data(&self, data: &[u8]) -> Result<(TokenInfo, TokenInfo, PoolReserves, PoolFees)> {
        use tracing::info;
        
        info!("📊 Parsing Orca Whirlpool data ({} bytes)", data.len());
        
        // Используем парсер для реальных данных
        let parser = OrcaWhirlpoolParser;
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
impl DexAdapter for OrcaWhirlpoolAdapter {
    async fn get_pool_info(&self, pool_address: &Pubkey) -> Result<PoolInfo> {
        // Сначала пытаемся API
        if self.api_client.is_available().await {
            match self.api_client.get_pool_info(pool_address).await {
                Ok(pool_info) => {
                    info!("✅ Получена информация о пуле через Orca API");
                    return Ok(pool_info);
                }
                Err(e) => {
                    info!("⚠️ Orca API недоступен: {}, используем блокчейн", e);
                }
            }
        }
        
        // Fallback на блокчейн
        self.get_pool_info_from_chain(pool_address).await
    }

    async fn get_swap_quote(&self, pool_address: &Pubkey, amount_in: u64) -> Result<SwapQuote> {
        self.get_quote_with_fallback(pool_address, amount_in).await
    }

    async fn create_swap_instruction(&self, pool_pubkey: &Pubkey, amount_in: u64, min_amount_out: u64) -> Result<Instruction> {
        // Orca Whirlpool swap instruction structure
        // This is a simplified implementation - real Orca instructions are more complex
        
        use tracing::info;
        info!("Creating Orca Whirlpool swap instruction: {} -> {}, min_out: {}", 
              amount_in, min_amount_out, min_amount_out);
        
        // Get pool info to access vault accounts
        let pool_address = *pool_pubkey;
        
        // Create instruction data for Orca Whirlpool swap
        // Instruction discriminator + amount_in + min_amount_out + sqrt_price_limit + a_to_b
        let mut data = Vec::new();
        
        // Orca Whirlpool swap instruction discriminator (8 bytes)
        data.extend_from_slice(&[0xf8, 0xc6, 0x9e, 0x91, 0xe1, 0x7f, 0x18, 0xd8]);
        
        // Amount in (8 bytes)
        data.extend_from_slice(&amount_in.to_le_bytes());
        
        // Min amount out (8 bytes) 
        data.extend_from_slice(&min_amount_out.to_le_bytes());
        
        // Sqrt price limit (16 bytes) - use max for no limit
        data.extend_from_slice(&u128::MAX.to_le_bytes());
        
        // Amount specified is input (1 byte)
        data.push(1);
        
        // a_to_b direction (1 byte) - determine from token order
        // For now, assume we're swapping from token A to token B
        let a_to_b = true; // Simplified assumption
        data.push(if a_to_b { 1 } else { 0 });
        
        let accounts = vec![
            // User accounts
            AccountMeta::new(Pubkey::default(), true), // User wallet (signer) - placeholder
            AccountMeta::new(Pubkey::default(), false), // User source token account - placeholder
            AccountMeta::new(Pubkey::default(), false), // User destination token account - placeholder
            
            // Pool accounts
            AccountMeta::new(pool_address, false), // AMM pool
            AccountMeta::new(Pubkey::default(), false), // Token vault A - placeholder
            AccountMeta::new(Pubkey::default(), false), // Token vault B - placeholder
            
            // Program accounts
            AccountMeta::new_readonly(spl_token::id(), false), // SPL Token program
            AccountMeta::new_readonly(Pubkey::from_str("whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc")?, false), // Orca program ID
        ];
        
        Ok(Instruction {
            program_id: Pubkey::from_str("whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc")?,
            accounts,
            data,
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
