use solana_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rpc_url = "https://mainnet.helius-rpc.com/?api-key=b5939e95-d595-4e01-9401-da85b5c720af";
    let client = RpcClient::new(rpc_url.to_string());
    
    println!("🔍 Searching for pools via Helius RPC...");
    
    // Raydium V4 program ID
    let raydium_v4_program = "675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSSt1Mp8";
    println!("📊 Looking for Raydium V4 pools (program: {})", raydium_v4_program);
    
    // Orca Whirlpool program ID
    let orca_program = "whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc";
    println!("🐋 Looking for Orca Whirlpool pools (program: {})", orca_program);
    
    // SOL mint
    let sol_mint = "So11111111111111111111111111111111111111112";
    // USDC mint
    let usdc_mint = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";
    
    println!("🪙 Looking for SOL-USDC pools");
    println!("   SOL: {}", sol_mint);
    println!("   USDC: {}", usdc_mint);
    
    // Попробуем найти пулы по RPC
    println!("\n🔍 Searching via RPC...");
    
    // Ищем аккаунты по program ID
    let raydium_program_pubkey = Pubkey::from_str(raydium_v4_program)?;
    let orca_program_pubkey = Pubkey::from_str(orca_program)?;
    
    println!("📊 Searching for Raydium V4 accounts...");
    match client.get_program_accounts(&raydium_program_pubkey) {
        Ok(accounts) => {
            println!("✅ Found {} Raydium V4 accounts", accounts.len());
            if accounts.len() > 0 {
                println!("   First account: {}", accounts[0].0);
                println!("   Data size: {} bytes", accounts[0].1.data.len());
            }
        },
        Err(e) => println!("❌ Error searching Raydium V4: {}", e),
    }
    
    println!("\n🐋 Searching for Orca Whirlpool accounts...");
    match client.get_program_accounts(&orca_program_pubkey) {
        Ok(accounts) => {
            println!("✅ Found {} Orca Whirlpool accounts", accounts.len());
            if accounts.len() > 0 {
                println!("   First account: {}", accounts[0].0);
                println!("   Data size: {} bytes", accounts[0].1.data.len());
            }
        },
        Err(e) => println!("❌ Error searching Orca: {}", e),
    }
    
    // Поиск по Solscan или другим API
    println!("\n💡 Для поиска пулов используйте:");
    println!("   - Solscan: https://solscan.io/account/{}", raydium_v4_program);
    println!("   - Solana Explorer: https://explorer.solana.com/address/{}", raydium_v4_program);
    println!("   - Orca: https://www.orca.so/");
    println!("   - Raydium: https://raydium.io/liquidity-pools/");
    
    println!("\n🔍 Ищите пулы с:");
    println!("   - Program ID: {}", raydium_v4_program);
    println!("   - Токены: SOL ↔ USDC");
    println!("   - Размер данных: ~752 байт для Raydium V4, ~653 для Orca");
    
    Ok(())
}
