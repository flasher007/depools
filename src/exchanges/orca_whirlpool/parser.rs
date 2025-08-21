use anyhow::Result;
use solana_sdk::pubkey::Pubkey;
use crate::exchanges::types::{TokenInfo, PoolReserves, PoolFees};
use crate::exchanges::common::DebugParser;
use tracing::debug;

/// Парсер для данных пула Orca Whirlpool
pub struct OrcaWhirlpoolParser;

impl OrcaWhirlpoolParser {
    /// Парсит данные пула Orca Whirlpool
    /// Использует фиксированные позиции для SOL-USDC пулов
    pub fn parse_pool_data(
        data: &[u8],
        base_token_mint: &str,
        base_token_symbol: &str,
        base_token_decimals: u8,
        _quote_token_mint: &str,
        quote_token_symbol: &str,
        quote_token_decimals: u8,
    ) -> Result<(TokenInfo, TokenInfo, PoolReserves, PoolFees)> {
        // Проверяем минимальный размер данных
        if data.len() < 653 {
            return Err(anyhow::anyhow!("Invalid Orca Whirlpool data size: {} bytes", data.len()));
        }
        
        // Анализируем структуру данных для отладки
        DebugParser::analyze_pool_structure(data)?;
        
        // Динамически ищем токены в данных Orca Whirlpool
        let mut sol_position = None;
        let mut usdc_position = None;
        
        // Ищем SOL и USDC во всех позициях
        for i in (0..data.len()).step_by(32) {
            if i + 32 <= data.len() {
                if let Ok(pubkey) = Pubkey::try_from(&data[i..i + 32]) {
                    let pubkey_str = pubkey.to_string();
                    if pubkey_str == "So11111111111111111111111111111111111111112" ||
                       pubkey_str == "11111111111111111111111111111111" ||
                       pubkey_str == "11111111111111112GUYsqKrFZU73SvLFXevgF" {
                        sol_position = Some(i);
                        debug!("🔍 Found SOL at position {} in Orca Whirlpool", i);
                    } else if pubkey_str == "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v" {
                        usdc_position = Some(i);
                        debug!("🔍 Found USDC at position {} in Orca Whirlpool", i);
                    }
                }
            }
        }
        
        // Нормализуем порядок: всегда SOL первый, USDC второй
        let token_a_mint_start = sol_position.unwrap_or(0);    // SOL всегда первый
        let token_b_mint_start = usdc_position.unwrap_or(32); // USDC всегда второй
        let token_a_vault_start = 64; // SOL vault
        let token_b_vault_start = 96; // USDC vault
        let reserve_a_start = 128;    // SOL reserve
        let reserve_b_start = 136;    // USDC reserve
        let lp_supply_start = 144;    // LP supply
        let fee_start = 72;           // Fees (исправлено с 152 на 72)
        
        let token_a_mint = Pubkey::try_from(&data[token_a_mint_start..token_a_mint_start + 32])?;
        let token_b_mint = Pubkey::try_from(&data[token_b_mint_start..token_b_mint_start + 32])?;
        
        // Отладочная информация
        debug!("🔍 DEBUG: token_a_mint = {} (position {})", token_a_mint, token_a_mint_start);
        debug!("🔍 DEBUG: token_b_mint = {} (position {})", token_b_mint, token_b_mint_start);
        
        let token_a_vault = Pubkey::try_from(&data[token_a_vault_start..token_a_vault_start + 32])?;
        let token_b_vault = Pubkey::try_from(&data[token_b_vault_start..token_b_vault_start + 32])?;
        
        let token_a_reserve = u64::from_le_bytes([
            data[reserve_a_start], data[reserve_a_start + 1], data[reserve_a_start + 2], data[reserve_a_start + 3],
            data[reserve_a_start + 4], data[reserve_a_start + 5], data[reserve_a_start + 6], data[reserve_a_start + 7]
        ]);
        let token_b_reserve = u64::from_le_bytes([
            data[reserve_b_start], data[reserve_b_start + 1], data[reserve_b_start + 2], data[reserve_b_start + 3],
            data[reserve_b_start + 4], data[reserve_b_start + 5], data[reserve_b_start + 6], data[reserve_b_start + 7]
        ]);
        
        let lp_supply = u64::from_le_bytes([
            data[lp_supply_start], data[lp_supply_start + 1], data[lp_supply_start + 2], data[lp_supply_start + 3],
            data[lp_supply_start + 4], data[lp_supply_start + 5], data[lp_supply_start + 6], data[lp_supply_start + 7]
        ]);
        
        let trade_fee_bps = u32::from_le_bytes([
            data[fee_start], data[fee_start + 1], data[fee_start + 2], data[fee_start + 3]
        ]);
        let owner_trade_fee_bps = u32::from_le_bytes([
            data[fee_start + 4], data[fee_start + 5], data[fee_start + 6], data[fee_start + 7]
        ]);
        let owner_withdraw_fee_bps = u32::from_le_bytes([
            data[fee_start + 8], data[fee_start + 9], data[fee_start + 10], data[fee_start + 11]
        ]);
        
        // Определяем какой токен SOL, какой USDC
        // Сравниваем mint адреса с известными токенами
        let token_a_mint_str = token_a_mint.to_string();
        let token_b_mint_str = token_b_mint.to_string();
        
        let (token_a_symbol, token_a_decimals) = if token_a_mint_str == "So11111111111111111111111111111111111111112" || 
                                                   token_a_mint_str == "11111111111111111111111111111111" ||
                                                   token_a_mint_str == "11111111111111112GUYsqKrFZU73SvLFXevgF" {
            ("SOL".to_string(), 9)
        } else if token_a_mint_str == "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v" {
            ("USDC".to_string(), 6)
        } else {
            // Если не знаем токен, используем переданные значения
            if token_a_mint_str == base_token_mint {
                (base_token_symbol.to_string(), base_token_decimals)
            } else {
                (quote_token_symbol.to_string(), quote_token_decimals)
            }
        };
        
        let (token_b_symbol, token_b_decimals) = if token_b_mint_str == "So11111111111111111111111111111111111111112" || 
                                                   token_b_mint_str == "11111111111111111111111111111111" ||
                                                   token_b_mint_str == "11111111111111112GUYsqKrFZU73SvLFXevgF" {
            ("SOL".to_string(), 9)
        } else if token_b_mint_str == "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v" {
            ("USDC".to_string(), 6)
        } else {
            // Если не знаем токен, используем переданные значения
            if token_b_mint_str == base_token_mint {
                (base_token_symbol.to_string(), base_token_decimals)
            } else {
                (quote_token_symbol.to_string(), quote_token_decimals)
            }
        };
        
        let token_a = TokenInfo {
            mint: token_a_mint,
            symbol: token_a_symbol,
            decimals: token_a_decimals,
            vault: token_a_vault,
        };
        
        let token_b = TokenInfo {
            mint: token_b_mint,
            symbol: token_b_symbol,
            decimals: token_b_decimals,
            vault: token_b_vault,
        };
        
        let reserves = PoolReserves {
            token_a_reserve,
            token_b_reserve,
            lp_supply: Some(lp_supply),
        };
        
        let fees = PoolFees {
            trade_fee_bps,
            owner_trade_fee_bps,
            owner_withdraw_fee_bps,
        };
        
        Ok((token_a, token_b, reserves, fees))
    }
}
