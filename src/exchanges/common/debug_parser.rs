use anyhow::Result;
use solana_sdk::pubkey::Pubkey;
use tracing::debug;

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
