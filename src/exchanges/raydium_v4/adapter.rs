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
use super::RaydiumV4Parser;

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
        let (token_a, token_b, reserves, fees) = RaydiumV4Parser::parse_pool_data(
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
            dex_label: self.get_label(),
            token_a,
            token_b,
            reserves,
            fees,
            pool_state: PoolState::Active,
        })
    }

    async fn get_swap_quote(&self, pool_address: &Pubkey, amount_in: u64, token_in: &Pubkey) -> Result<SwapQuote> {
        // Получаем информацию о пуле
        let pool_info = self.get_pool_info(pool_address).await?;
        
        // Определяем направление свапа
        let (token_in_info, token_out_info, amount_out) = if token_in == &pool_info.token_a.mint {
            (&pool_info.token_a, &pool_info.token_b, pool_info.reserves.token_b_reserve)
        } else if token_in == &pool_info.token_b.mint {
            (&pool_info.token_b, &pool_info.token_a, pool_info.reserves.token_a_reserve)
        } else {
            return Err(anyhow::anyhow!("Token {} not found in pool", token_in));
        };
        
        // Простой расчет свапа (без учета комиссий и slippage)
        // В реальной реализации здесь будет сложная формула AMM
        let amount_out = if token_in == &pool_info.token_a.mint {
            pool_info.reserves.token_b_reserve
        } else {
            pool_info.reserves.token_a_reserve
        };
        
        Ok(SwapQuote {
            pool_address: *pool_address,
            dex_label: self.get_label(),
            token_in: *token_in,
            token_out: token_out_info.mint,
            amount_in,
            amount_out,
            min_amount_out: 0, // Упрощенно
            price_impact_bps: 0, // Упрощенно
            fee_amount: 0, // Упрощенно
            route: SwapRoute {
                hops: vec![SwapHop {
                    pool_address: *pool_address,
                    dex_label: self.get_label(),
                    token_in: *token_in,
                    token_out: token_out_info.mint,
                    amount_in,
                    amount_out,
                    fee_bps: 0, // Упрощенно
                }],
                total_fee_bps: 0, // Упрощенно
            },
        })
    }

    fn create_swap_instruction(&self, quote: &SwapQuote, user_pubkey: &Pubkey) -> Result<Instruction> {
        // Создаем инструкцию для свапа
        // В реальной реализации здесь будет создание инструкции для Raydium V4
        
        let accounts = vec![
            AccountMeta::new(*user_pubkey, true),
            AccountMeta::new(quote.pool_address, false),
            // Добавить другие необходимые аккаунты
        ];
        
        let data = vec![
            // Данные инструкции для свапа
        ];
        
        Ok(Instruction {
            program_id: Pubkey::from_str(&self.config.programs.raydium_v4)?,
            accounts,
            data,
        })
    }
}
