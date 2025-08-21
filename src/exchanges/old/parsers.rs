use anyhow::Result;
use solana_sdk::pubkey::Pubkey;
use crate::exchanges::types::{TokenInfo, PoolReserves, PoolFees};
use tracing::debug;

// Константы для известных токенов
const KNOWN_TOKENS: &[(&str, &str, u8)] = &[
    // SOL variants
    ("So11111111111111111111111111111111111111112", "SOL", 9),
    ("11111111111111111111111111111111", "SOL", 9),
    
    // USDC
    ("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v", "USDC", 6),
    
    // USDT
    ("Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB", "USDT", 6),
    
    // RAY
    ("4k3Dyjzvzp8eMZWUXbBCjEvwSkkk59S5iCNLY3QrkX6R", "RAY", 6),
];

// Функция для определения токена по mint адресу
fn get_token_info(mint: &str) -> Option<(&str, u8)> {
    KNOWN_TOKENS.iter()
        .find(|(addr, _, _)| *addr == mint)
        .map(|(_, symbol, decimals)| (*symbol, *decimals))
}

/// Отладочный парсер для анализа структуры данных пула
pub struct DebugParser;

impl DebugParser {
    /// Анализирует структуру данных пула и показывает все возможные токены
    pub fn analyze_pool_structure(data: &[u8]) -> Result<()> {
        debug!("🔍 Analyzing pool data structure ({} bytes)", data.len());
        
        // Ищем все возможные Pubkey (32 байта)
        let mut pubkeys = Vec::new();
        for i in (0..data.len()).step_by(32) {
            if i + 32 <= data.len() {
                let pubkey = Pubkey::try_from(&data[i..i + 32])?;
                pubkeys.push((i, pubkey));
            }
        }
        
        debug!("📋 Found {} potential Pubkeys:", pubkeys.len());
        for (pos, pubkey) in pubkeys.iter().take(10) { // Показываем первые 10
            debug!("  Position {}: {}", pos, pubkey);
        }
        
        // Ищем возможные резервы (u64 в little-endian)
        let mut reserves = Vec::new();
        for i in (0..data.len()).step_by(8) {
            if i + 8 <= data.len() {
                let reserve = u64::from_le_bytes([
                    data[i], data[i + 1], data[i + 2], data[i + 3],
                    data[i + 4], data[i + 5], data[i + 6], data[i + 7]
                ]);
                if reserve > 0 && reserve < 1_000_000_000_000_000_000 { // Разумные значения
                    reserves.push((i, reserve));
                }
            }
        }
        
        debug!("💰 Found {} potential reserves:", reserves.len());
        for (pos, reserve) in reserves.iter().take(10) { // Показываем первые 10
            debug!("  Position {}: {}", pos, reserve);
        }
        
        // Ищем возможные комиссии (u32 в little-endian)
        let mut fees = Vec::new();
        for i in (0..data.len()).step_by(4) {
            if i + 4 <= data.len() {
                let fee = u32::from_le_bytes([
                    data[i], data[i + 1], data[i + 2], data[i + 3]
                ]);
                if fee > 0 && fee < 10000 { // Разумные значения для bps
                    fees.push((i, fee));
                }
            }
        }
        
        debug!("💸 Found {} potential fees:", fees.len());
        for (pos, fee) in fees.iter().take(10) { // Показываем первые 10
            debug!("  Position {}: {} bps", pos, fee);
        }
        
        // Анализируем структуру комиссий более детально
        debug!("🔍 Analyzing fee structure:");
        for i in (0..data.len()).step_by(4) {
            if i + 4 <= data.len() {
                let fee = u32::from_le_bytes([
                    data[i], data[i + 1], data[i + 2], data[i + 3]
                ]);
                // Ищем разумные значения комиссий (0.01% - 10%)
                if fee >= 1 && fee <= 1000 {
                    debug!("  Position {}: {} bps (0.{}%)", i, fee, fee as f64 / 100.0);
                }
            }
        }
        
        Ok(())
    }
}

/// Парсер для данных пула Raydium V4
pub struct RaydiumV4Parser;

impl RaydiumV4Parser {
    /// Парсит данные пула Raydium V4
    /// Использует фиксированные позиции для SOL-USDC пулов
    pub fn parse_pool_data(
        data: &[u8],
        base_token_mint: &str,
        base_token_symbol: &str,
        base_token_decimals: u8,
        quote_token_mint: &str,
        quote_token_symbol: &str,
        quote_token_decimals: u8,
    ) -> Result<(TokenInfo, TokenInfo, PoolReserves, PoolFees)> {
        // Проверяем минимальный размер данных
        if data.len() < 752 {
            return Err(anyhow::anyhow!("Invalid Raydium V4 pool data size: {} bytes", data.len()));
        }
        
        // Анализируем структуру данных для отладки
        DebugParser::analyze_pool_structure(data)?;
        
        // Динамически ищем токены в данных Raydium V4
        let mut token_positions: Vec<(usize, String, u8)> = Vec::new();
        
        // Ищем известные токены во всех позициях
        
        for i in (0..data.len()).step_by(32) {
            if i + 32 <= data.len() {
                if let Ok(pubkey) = Pubkey::try_from(&data[i..i + 32]) {
                    let pubkey_str = pubkey.to_string();
                    if let Some((symbol, decimals)) = get_token_info(&pubkey_str) {
                        token_positions.push((i, symbol.to_string(), decimals));
                        debug!("🔍 Found {} at position {} in Raydium V4", symbol, i);
                    }
                }
            }
        }
        
        // Ищем разные токены для разнообразия
        let mut token_a_mint_start = 0;
        let mut token_b_mint_start = 32;
        
        if let Some((pos, symbol, _)) = token_positions.get(0) {
            token_a_mint_start = *pos;
            
            // Ищем токен с другим символом
            if let Some((pos2, symbol2, _)) = token_positions.iter()
                .find(|(_, sym, _)| sym != symbol) {
                token_b_mint_start = *pos2;
                debug!("🔍 Using different tokens: {} at pos {}, {} at pos {}", 
                        symbol, token_a_mint_start, symbol2, token_b_mint_start);
            } else {
                debug!("⚠️  Only found one token type: {}", symbol);
            }
        }
        let token_a_vault_start = 64; // USDC vault
        let token_b_vault_start = 96; // SOL vault
        let reserve_a_start = 128;     // USDC reserve
        let reserve_b_start = 136;     // SOL reserve
        let lp_supply_start = 144;     // LP supply
        let fee_start = 144;           // Fees (исправлено с 152 на 144)
        
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
        
        // Отладочная информация для проверки
        debug!("🔍 DEBUG: token_a_mint_str = '{}'", token_a_mint_str);
        debug!("🔍 DEBUG: token_b_mint_str = '{}'", token_b_mint_str);
        debug!("🔍 DEBUG: SOL mint = 'So11111111111111111111111111111111111111112'");
        debug!("🔍 DEBUG: USDC mint = 'EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v'");
        debug!("🔍 DEBUG: token_a = {} ({}), token_b = {} ({})", 
                token_a_symbol, token_a_mint, token_b_symbol, token_b_mint);
        
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
        quote_token_mint: &str,
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
