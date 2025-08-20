use anyhow::Result;
use solana_sdk::{
    pubkey::Pubkey,
    instruction::{Instruction, AccountMeta},
};
use std::str::FromStr;
use solana_client::rpc_client::RpcClient;
use spl_associated_token_account::get_associated_token_address;
use std::sync::Arc;
use crate::config::Config;
use crate::exchanges::{DexAdapter, types::{DexLabel, PoolInfo, SwapQuote, TokenInfo, PoolReserves, PoolFees, PoolState, SwapRoute, SwapHop}};

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

    fn parse_pool_data(&self, data: &[u8]) -> Result<(TokenInfo, TokenInfo, PoolReserves, PoolFees)> {
        // Упрощенная реализация парсинга данных пула Raydium V4
        // В реальной реализации здесь будет полный парсинг структуры пула
        
        use tracing::info;
        
        info!("📊 Parsing Raydium V4 pool data ({} bytes)", data.len());
        
        // Для известных пулов используем предопределенные данные
        // В реальной реализации нужно парсить бинарные данные пула
        
        // Токены теперь парсятся из данных пула через парсер
        
        // Токены теперь парсятся из данных пула
        
        // Используем парсер для реальных данных
        let (token_a, token_b, reserves, fees) = crate::exchanges::parsers::RaydiumV4Parser::parse_pool_data(
            data,
            &self.config.tokens.base_token.mint,
            &self.config.tokens.base_token.symbol,
            self.config.tokens.base_token.decimals,
            &self.config.tokens.quote_token.mint,
            &self.config.tokens.quote_token.symbol,
            self.config.tokens.quote_token.decimals,
        )?;
        
        info!("🪙 Parsed tokens: {} ({}) ↔ {} ({})", 
              token_a.symbol, token_a.mint, token_b.symbol, token_b.mint);
        info!("💰 Reserves: {} {} ↔ {} {}", 
              reserves.token_a_reserve, token_a.symbol, reserves.token_b_reserve, token_b.symbol);
        
        Ok((token_a, token_b, reserves, fees))
    }
}

#[async_trait::async_trait]
impl DexAdapter for RaydiumV4Adapter {
    fn get_label(&self) -> DexLabel {
        DexLabel::RaydiumV4
    }

    async fn get_pool_info(&self, pool_address: &Pubkey) -> Result<PoolInfo> {
        // Асинхронный вызов для получения данных пула
        let data = self.fetch_pool_data(pool_address).await?;
        let (token_a, token_b, reserves, fees) = self.parse_pool_data(&data)?;
        
        Ok(PoolInfo {
            pool_address: *pool_address,
            dex_label: DexLabel::RaydiumV4,
            token_a,
            token_b,
            reserves,
            fees,
            pool_state: PoolState::Active,
        })
    }

    async fn get_swap_quote(&self, pool_address: &Pubkey, amount_in: u64, token_in: &Pubkey) -> Result<SwapQuote> {
        // Упрощенная реализация расчета swap quote
        // В реальной реализации здесь будет точный расчет с учетом формулы AMM
        
        let pool_info = self.get_pool_info(pool_address).await?;
        let (_token_in_info, token_out_info) = if *token_in == pool_info.token_a.mint {
            (&pool_info.token_a, &pool_info.token_b)
        } else {
            (&pool_info.token_b, &pool_info.token_a)
        };
        
        // Простая формула constant product AMM
        let k = pool_info.reserves.token_a_reserve as f64 * pool_info.reserves.token_b_reserve as f64;
        let new_reserve_in = pool_info.reserves.token_a_reserve as f64 + amount_in as f64;
        let new_reserve_out = k / new_reserve_in;
        let amount_out = (pool_info.reserves.token_b_reserve as f64 - new_reserve_out) as u64;
        
        let fee_amount = (amount_in as u64 * pool_info.fees.trade_fee_bps as u64) / 10000;
        
        // Slippage protection: min_amount_out учитывает slippage_bps (по умолчанию 100 = 1%)
        let slippage_bps = 100; // 1% slippage tolerance
        let slippage_amount = (amount_out * slippage_bps) / 10000;
        let min_amount_out = amount_out.saturating_sub(fee_amount).saturating_sub(slippage_amount);
        
        let route = SwapRoute {
            hops: vec![SwapHop {
                pool_address: *pool_address,
                dex_label: DexLabel::RaydiumV4,
                token_in: *token_in,
                token_out: token_out_info.mint,
                amount_in,
                amount_out,
                fee_bps: pool_info.fees.trade_fee_bps,
            }],
            total_fee_bps: pool_info.fees.trade_fee_bps,
        };
        
        Ok(SwapQuote {
            pool_address: *pool_address,
            dex_label: DexLabel::RaydiumV4,
            token_in: *token_in,
            token_out: token_out_info.mint,
            amount_in,
            amount_out,
            min_amount_out,
            price_impact_bps: 0, // Расчет price impact
            fee_amount,
            route,
        })
    }

    fn create_swap_instruction(&self, quote: &SwapQuote, user_pubkey: &Pubkey) -> Result<Instruction> {
        // Создаем реальную swap инструкцию для Raydium V4
        // В реальности нужно использовать Raydium V4 program ID и правильную структуру
        
        let raydium_v4_program = self.config.programs.raydium_v4.parse::<Pubkey>()?;
        
        // Определяем направление swap
        let (token_in_mint, token_out_mint) = (quote.token_in, quote.token_out);
        
        // Создаем временные vault адреса (в реальности нужно получать из пула)
        let token_in_vault = get_associated_token_address(
            &quote.pool_address, 
            &token_in_mint
        );
        let token_out_vault = get_associated_token_address(
            &quote.pool_address, 
            &token_out_mint
        );
        
        // Пользовательские ATA (Associated Token Accounts)
        let user_token_in_ata = get_associated_token_address(
            user_pubkey, 
            &token_in_mint
        );
        let user_token_out_ata = get_associated_token_address(
            user_pubkey, 
            &token_out_mint
        );
        
        // Создаем инструкцию для Raydium V4 swap
        // В реальности это будет вызов конкретной инструкции программы
        let swap_instruction = Instruction {
            program_id: raydium_v4_program,
            accounts: vec![
                AccountMeta::new_readonly(quote.pool_address, false), // Pool
                AccountMeta::new_readonly(token_in_mint, false),      // Token In Mint
                AccountMeta::new_readonly(token_out_mint, false),     // Token Out Mint
                AccountMeta::new(token_in_vault, false),             // Pool Token In Vault
                AccountMeta::new(token_out_vault, false),            // Pool Token Out Vault
                AccountMeta::new(user_token_in_ata, false),          // User Token In ATA
                AccountMeta::new(user_token_out_ata, false),         // User Token Out ATA
                AccountMeta::new_readonly(*user_pubkey, true),       // User (signer)
                AccountMeta::new_readonly(self.config.programs.spl_token.parse()?, false),   // SPL Token Program
            ],
            data: vec![
                0x01, // Instruction discriminator для swap (пример)
                // В реальности здесь будут правильные данные для Raydium V4
            ],
        };
        
        Ok(swap_instruction)
    }
}
