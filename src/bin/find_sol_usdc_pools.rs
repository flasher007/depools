use anyhow::Result;
use solana_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;

#[tokio::main]
async fn main() -> Result<()> {
    let rpc_url = "https://api.mainnet-beta.solana.com";
    let client = RpcClient::new(rpc_url.to_string());
    
    // Известные SOL-USDC пулы
    let known_sol_usdc_pools = [
        // Raydium V4 SOL-USDC (из их документации)
        "58oQChx4yWmvKdwLLZzBi4ChoCc2fqCUWBkwMihLYQo2",
        // Orca Whirlpool SOL-USDC (из их документации)
        "HJPjoWUrhoZzkNfRpHuieeFk9WcZWjwy6PBjZ81ngndJ",
        // Другие известные пулы
        "8HoQnePLqPj4M7PUDzfw8e3qmdLacxYC6K7QddiPy4mS", // Raydium V4
        "7qbRF6YsyGuLUVs6Y1q64bdVrfe4ZcUUz1JRdoVNUJnm", // Orca Whirlpool
    ];
    
    println!("🔍 Searching for real SOL-USDC pools...");
    
    for pool_address in &known_sol_usdc_pools {
        println!("\n🔍 Checking pool: {}", pool_address);
        
        if let Ok(pubkey) = Pubkey::from_str(pool_address) {
            if let Ok(account) = client.get_account(&pubkey) {
                println!("  📊 Size: {} bytes", account.data.len());
                println!("  📊 Owner: {}", account.owner);
                
                // Ищем известные токены
                let known_tokens = [
                    ("So11111111111111111111111111111111111111112", "WSOL"),
                    ("11111111111111111111111111111111", "SOL"),
                    ("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v", "USDC"),
                ];
                
                let mut found_tokens = Vec::new();
                for (i, chunk) in account.data.chunks(32).enumerate() {
                    if chunk.len() == 32 {
                        if let Ok(pubkey) = Pubkey::try_from(chunk) {
                            let pubkey_str = pubkey.to_string();
                            
                            for (known_mint, symbol) in &known_tokens {
                                if &pubkey_str == known_mint {
                                    found_tokens.push((i * 32, pubkey, symbol));
                                }
                            }
                        }
                    }
                }
                
                if found_tokens.is_empty() {
                    println!("  ❌ No known tokens found");
                } else {
                    println!("  ✅ Found tokens:");
                    for (pos, pubkey, symbol) in &found_tokens {
                        println!("    {:3}: {} -> {}", pos, pubkey, symbol);
                    }
                    
                    // Проверяем, есть ли и SOL, и USDC
                    let has_sol = found_tokens.iter().any(|(_, _, symbol)| **symbol == "SOL" || **symbol == "WSOL");
                    let has_usdc = found_tokens.iter().any(|(_, _, symbol)| **symbol == "USDC");
                    
                    if has_sol && has_usdc {
                        println!("  🎯 This is a SOL-USDC pool!");
                    } else {
                        println!("  ⚠️  This is NOT a SOL-USDC pool");
                    }
                }
            } else {
                println!("  ❌ Failed to fetch account");
            }
        } else {
            println!("  ❌ Invalid pool address");
        }
    }
    
    Ok(())
}
