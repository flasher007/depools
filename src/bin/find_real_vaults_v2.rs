use anyhow::Result;
use solana_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rpc_url = "https://api.mainnet-beta.solana.com";
    let client = RpcClient::new(rpc_url.to_string());

    let raydium_pool = "58oQChx4yWmvKdwLLZzBi4ChoCc2fqCUWBkwMihLYQo2";
    let orca_pool = "Czfq3xZZDmsdGdUyrNLtRhGc47cXcZtLG4crryfu44zE";

    println!("🔍 Searching for real vault addresses in pool data...");

    // Analyze Raydium pool
    println!("\n📊 Raydium V4 pool: {}", raydium_pool);
    if let Ok(account) = client.get_account(&Pubkey::from_str(raydium_pool)?) {
        let data = account.data;
        println!("Data size: {} bytes", data.len());
        
        // Search for potential vault addresses
        println!("\n🔍 Searching for potential vault addresses...");
        
        // Look for Pubkeys that might be vaults
        for i in 0..data.len() - 31 {
            if let Ok(pubkey) = Pubkey::try_from(&data[i..i+32]) {
                let pubkey_str = pubkey.to_string();
                
                // Filter out known token mints and other known addresses
                if !pubkey_str.contains("11111111111111111111111111111111") && 
                   pubkey_str != "So11111111111111111111111111111111111111112" &&
                   pubkey_str != "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v" {
                    
                    // Check if this might be a vault by looking for patterns
                    // Vault addresses often have many '1's and '2's
                    let ones = pubkey_str.chars().filter(|&c| c == '1').count();
                    let twos = pubkey_str.chars().filter(|&c| c == '2').count();
                    
                    if ones > 20 || twos > 15 {
                        println!("  Position {}: {} (ones: {}, twos: {})", i, pubkey_str, ones, twos);
                    }
                }
            }
        }
    }

    // Analyze Orca pool
    println!("\n📊 Orca Whirlpool pool: {}", orca_pool);
    if let Ok(account) = client.get_account(&Pubkey::from_str(orca_pool)?) {
        let data = account.data;
        println!("Data size: {} bytes", data.len());
        
        // Search for potential vault addresses
        println!("\n🔍 Searching for potential vault addresses...");
        
        // Look for Pubkeys that might be vaults
        for i in 0..data.len() - 31 {
            if let Ok(pubkey) = Pubkey::try_from(&data[i..i+32]) {
                let pubkey_str = pubkey.to_string();
                
                // Filter out known token mints and other known addresses
                if !pubkey_str.contains("11111111111111111111111111111111") && 
                   pubkey_str != "So11111111111111111111111111111111111111112" &&
                   pubkey_str != "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v" {
                    
                    // Check if this might be a vault by looking for patterns
                    // Vault addresses often have many '1's and '2's
                    let ones = pubkey_str.chars().filter(|&c| c == '1').count();
                    let twos = pubkey_str.chars().filter(|&c| c == '2').count();
                    
                    if ones > 20 || twos > 15 {
                        println!("  Position {}: {} (ones: {}, twos: {})", i, pubkey_str, ones, twos);
                    }
                }
            }
        }
    }

    Ok(())
}
