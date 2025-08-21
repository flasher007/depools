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
    
    // Анализируем все данные и ищем известные токены
    println!("\n🔍 Searching for known tokens:");
    let known_tokens = [
        ("So11111111111111111111111111111111111111112", "WSOL"),
        ("11111111111111111111111111111111", "SOL"),
        ("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v", "USDC"),
        ("Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB", "USDT"),
        ("4k3Dyjzvzp8eMZWUXbBCjEvwSkkk59S5iCNLY3QrkX6R", "RAY"),
    ];
    
    for (i, chunk) in account.data.chunks(32).enumerate() {
        if chunk.len() == 32 {
            if let Ok(pubkey) = Pubkey::try_from(chunk) {
                let pubkey_str = pubkey.to_string();
                
                // Проверяем, является ли это известным токеном
                for (known_mint, symbol) in &known_tokens {
                    if &pubkey_str == known_mint {
                        println!("  {:3}: {} -> {}", i * 32, pubkey, symbol);
                    }
                }
            }
        }
    }
    
    // Анализируем первые 200 байт
    println!("\n🔍 First 200 bytes:");
    for (i, chunk) in account.data.chunks(32).enumerate().take(7) {
        println!("  {:3}: {}", i * 32, hex::encode(chunk));
        
        // Пытаемся интерпретировать как Pubkey
        if chunk.len() == 32 {
            if let Ok(pubkey) = Pubkey::try_from(chunk) {
                println!("       -> Pubkey: {}", pubkey);
            }
        }
        
        // Пытаемся интерпретировать как u64
        if chunk.len() >= 8 {
            let value = u64::from_le_bytes([
                chunk[0], chunk[1], chunk[2], chunk[3],
                chunk[4], chunk[5], chunk[6], chunk[7]
            ]);
            if value > 0 && value < 1_000_000_000_000_000_000 {
                println!("       -> u64: {}", value);
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
    
    // Анализируем все данные и ищем известные токены
    println!("\n🔍 Searching for known tokens:");
    for (i, chunk) in account.data.chunks(32).enumerate() {
        if chunk.len() == 32 {
            if let Ok(pubkey) = Pubkey::try_from(chunk) {
                let pubkey_str = pubkey.to_string();
                
                // Проверяем, является ли это известным токеном
                for (known_mint, symbol) in &known_tokens {
                    if &pubkey_str == known_mint {
                        println!("  {:3}: {} -> {}", i * 32, pubkey, symbol);
                    }
                }
            }
        }
    }
    
    // Анализируем первые 200 байт
    println!("\n🔍 First 200 bytes:");
    for (i, chunk) in account.data.chunks(32).enumerate().take(7) {
        println!("  {:3}: {}", i * 32, hex::encode(chunk));
        
        // Пытаемся интерпретировать как Pubkey
        if chunk.len() == 32 {
            if let Ok(pubkey) = Pubkey::try_from(chunk) {
                println!("       -> Pubkey: {}", pubkey);
            }
        }
        
        // Пытаемся интерпретировать как u64
        if chunk.len() >= 8 {
            let value = u64::from_le_bytes([
                chunk[0], chunk[1], chunk[2], chunk[3],
                chunk[4], chunk[5], chunk[6], chunk[7]
            ]);
            if value > 0 && value < 1_000_000_000_000_000_000 {
                println!("       -> u64: {}", value);
            }
        }
    }
    
    Ok(())
}
