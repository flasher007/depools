use anyhow::Result;
use solana_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rpc_url = "https://api.mainnet-beta.solana.com";
    let client = RpcClient::new(rpc_url.to_string());

    let raydium_pool = "58oQChx4yWmvKdwLLZzBi4ChoCc2fqCUWBkwMihLYQo2";
    let orca_pool = "Czfq3xZZDmsdGdUyrNLtRhGc47cXcZtLG4crryfu44zE";

    println!("🔍 Analyzing vault structure in pool data...");

    // Analyze Raydium pool
    println!("\n📊 Raydium V4 pool: {}", raydium_pool);
    if let Ok(account) = client.get_account(&Pubkey::from_str(raydium_pool)?) {
        let data = account.data;
        println!("Data size: {} bytes", data.len());
        
        // Analyze vault group at positions 225-235
        println!("\n🔍 Analyzing vault group at positions 225-235:");
        for i in 225..=235 {
            if let Ok(pubkey) = Pubkey::try_from(&data[i..i+32]) {
                let pubkey_str = pubkey.to_string();
                println!("  Position {}: {}", i, pubkey_str);
                
                // Check if this account exists
                if let Ok(acc) = client.get_account(&pubkey) {
                    println!("    ✅ Account exists, size: {} bytes", acc.data.len());
                } else {
                    println!("    ❌ Account not found");
                }
            }
        }
        
        // Analyze vault group at positions 657-667
        println!("\n🔍 Analyzing vault group at positions 657-667:");
        for i in 657..=667 {
            if let Ok(pubkey) = Pubkey::try_from(&data[i..i+32]) {
                let pubkey_str = pubkey.to_string();
                println!("  Position {}: {}", i, pubkey_str);
                
                // Check if this account exists
                if let Ok(acc) = client.get_account(&pubkey) {
                    println!("    ✅ Account exists, size: {} bytes", acc.data.len());
                } else {
                    println!("    ❌ Account not found");
                }
            }
        }
    }

    // Analyze Orca pool
    println!("\n📊 Orca Whirlpool pool: {}", orca_pool);
    if let Ok(account) = client.get_account(&Pubkey::from_str(orca_pool)?) {
        let data = account.data;
        println!("Data size: {} bytes", data.len());
        
        // Analyze vault group at positions 98-99
        println!("\n🔍 Analyzing vault group at positions 98-99:");
        for i in 98..=99 {
            if let Ok(pubkey) = Pubkey::try_from(&data[i..i+32]) {
                let pubkey_str = pubkey.to_string();
                println!("  Position {}: {}", i, pubkey_str);
                
                // Check if this account exists
                if let Ok(acc) = client.get_account(&pubkey) {
                    println!("    ✅ Account exists, size: {} bytes", acc.data.len());
                } else {
                    println!("    ❌ Account not found");
                }
            }
        }
        
        // Analyze vault group at positions 430-440
        println!("\n🔍 Analyzing vault group at positions 430-440:");
        for i in 430..=440 {
            if let Ok(pubkey) = Pubkey::try_from(&data[i..i+32]) {
                let pubkey_str = pubkey.to_string();
                println!("  Position {}: {}", i, pubkey_str);
                
                // Check if this account exists
                if let Ok(acc) = client.get_account(&pubkey) {
                    println!("    ✅ Account exists, size: {} bytes", acc.data.len());
                } else {
                    println!("    ❌ Account not found");
                }
            }
        }
    }

    Ok(())
}
