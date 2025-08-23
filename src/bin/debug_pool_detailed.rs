use anyhow::Result;
use solana_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;

#[tokio::main]
async fn main() -> Result<()> {
    let rpc_url = "https://api.mainnet-beta.solana.com";
    let client = RpcClient::new(rpc_url.to_string());
    
    // Анализируем Raydium V4 пул
    let raydium_pool = "58oQChx4yWmvKdwLLZzBi4ChoCc2fqCUWBkwMihLYQo2";
    println!("🔍 Analyzing Raydium V4 pool: {}", raydium_pool);
    
    let pubkey = Pubkey::from_str(raydium_pool)?;
    let account = client.get_account(&pubkey)?;
    
    println!("📊 Account data size: {} bytes", account.data.len());
    println!("📊 Owner: {}", account.owner);
    
    // Ищем все возможные Pubkey в данных
    println!("\n🔍 Searching for all Pubkeys in data:");
    let mut found_pubkeys = Vec::new();
    
    for (i, chunk) in account.data.chunks(32).enumerate() {
        if chunk.len() == 32 {
            if let Ok(pubkey) = Pubkey::try_from(chunk) {
                let pubkey_str = pubkey.to_string();
                
                // Проверяем, является ли это известным токеном
                let token_type = if pubkey_str == "So11111111111111111111111111111111111111112" {
                    "WSOL"
                } else if pubkey_str == "11111111111111111111111111111111" {
                    "SOL"
                } else if pubkey_str == "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v" {
                    "USDC"
                } else if pubkey_str == "4k3Dyjzvzp8eMZWUXbBCjEvwSkkk59S5iCNLY3QrkX6R" {
                    "RAY"
                } else {
                    "UNKNOWN"
                };
                
                found_pubkeys.push((i * 32, pubkey, token_type));
                println!("  {:3}: {} -> {}", i * 32, pubkey, token_type);
            }
        }
    }
    
    // Анализируем первые 300 байт детально
    println!("\n🔍 Detailed analysis of first 300 bytes:");
    for (i, chunk) in account.data.chunks(32).enumerate().take(10) {
        println!("  {:3}: {}", i * 32, hex::encode(chunk));
        
        // Пытаемся интерпретировать как разные типы
        if chunk.len() >= 8 {
            let u64_value = u64::from_le_bytes([
                chunk[0], chunk[1], chunk[2], chunk[3],
                chunk[4], chunk[5], chunk[6], chunk[7]
            ]);
            if u64_value > 0 && u64_value < 1_000_000_000_000_000_000 {
                println!("       -> u64: {}", u64_value);
            }
        }
        
        if chunk.len() >= 4 {
            let u32_value = u32::from_le_bytes([
                chunk[0], chunk[1], chunk[2], chunk[3]
            ]);
            if u32_value > 0 && u32_value < 1_000_000 {
                println!("       -> u32: {}", u32_value);
            }
        }
        
        if chunk.len() >= 2 {
            let u16_value = u16::from_le_bytes([chunk[0], chunk[1]]);
            if u16_value > 0 && u16_value < 65_535 {
                println!("       -> u16: {}", u16_value);
            }
        }
        
        if chunk.len() >= 1 {
            let u8_value = chunk[0];
            if u8_value > 0 && u8_value < 255 {
                println!("       -> u8: {}", u8_value);
            }
        }
    }
    
    // Анализируем Orca Whirlpool пул
    let orca_pool = "Czfq3xZZDmsdGdUyrNLtRhGc47cXcZtLG4crryfu44zE";
    println!("\n🔍 Analyzing Orca Whirlpool pool: {}", orca_pool);
    
    let pubkey = Pubkey::from_str(orca_pool)?;
    let account = client.get_account(&pubkey)?;
    
    println!("📊 Account data size: {} bytes", account.data.len());
    println!("📊 Owner: {}", account.owner);
    
    // Ищем все возможные Pubkey в данных
    println!("\n🔍 Searching for all Pubkeys in data:");
    let mut found_pubkeys = Vec::new();
    
    for (i, chunk) in account.data.chunks(32).enumerate() {
        if chunk.len() == 32 {
            if let Ok(pubkey) = Pubkey::try_from(chunk) {
                let pubkey_str = pubkey.to_string();
                
                // Проверяем, является ли это известным токеном
                let token_type = if pubkey_str == "So11111111111111111111111111111111111111112" {
                    "WSOL"
                } else if pubkey_str == "11111111111111111111111111111111" {
                    "SOL"
                } else if pubkey_str == "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v" {
                    "USDC"
                } else if pubkey_str == "4k3Dyjzvzp8eMZWUXbBCjEvwSkkk59S5iCNLY3QrkX6R" {
                    "RAY"
                } else {
                    "UNKNOWN"
                };
                
                found_pubkeys.push((i * 32, pubkey, token_type));
                println!("  {:3}: {} -> {}", i * 32, pubkey, token_type);
            }
        }
    }
    
    // Анализируем первые 300 байт детально
    println!("\n🔍 Detailed analysis of first 300 bytes:");
    for (i, chunk) in account.data.chunks(32).enumerate().take(10) {
        println!("  {:3}: {}", i * 32, hex::encode(chunk));
        
        // Пытаемся интерпретировать как разные типы
        if chunk.len() >= 8 {
            let u64_value = u64::from_le_bytes([
                chunk[0], chunk[1], chunk[2], chunk[3],
                chunk[4], chunk[5], chunk[6], chunk[7]
            ]);
            if u64_value > 0 && u64_value < 1_000_000_000_000_000_000 {
                println!("       -> u64: {}", u64_value);
            }
        }
        
        if chunk.len() >= 4 {
            let u32_value = u32::from_le_bytes([
                chunk[0], chunk[1], chunk[2], chunk[3]
            ]);
            if u32_value > 0 && u32_value < 1_000_000 {
                println!("       -> u32: {}", u32_value);
            }
        }
        
        if chunk.len() >= 2 {
            let u16_value = u16::from_le_bytes([chunk[0], chunk[1]]);
            if u16_value > 0 && u16_value < 65_535 {
                println!("       -> u16: {}", u16_value);
            }
        }
        
        if chunk.len() >= 1 {
            let u8_value = chunk[0];
            if u8_value > 0 && u8_value < 255 {
                println!("       -> u8: {}", u8_value);
            }
        }
    }
    
    Ok(())
}
