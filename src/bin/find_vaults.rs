use solana_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rpc_url = "https://api.mainnet-beta.solana.com";
    let client = RpcClient::new(rpc_url.to_string());

    let raydium_pool = "58oQChx4yWmvKdwLLZzBi4ChoCc2fqCUWBkwMihLYQo2";
    let orca_pool = "Czfq3xZZDmsdGdUyrNLtRhGc47cXcZtLG4crryfu44zE";

    println!("🔍 Searching for vault addresses in pools...");

    // Check Raydium pool
    println!("\n📊 Raydium V4 pool: {}", raydium_pool);
    if let Ok(account) = client.get_account(&Pubkey::from_str(raydium_pool)?) {
        let data = account.data;
        println!("Data size: {} bytes", data.len());
        
        // Search for vaults after token mints
        // WSOL is at 400, USDC at 432, so vaults should be after
        println!("🔍 Searching for vaults after position 432...");
        
        for i in 432..data.len() - 32 {
            if let Ok(pubkey) = Pubkey::try_from(&data[i..i+32]) {
                // Check if this looks like a vault (not a known token mint)
                let pubkey_str = pubkey.to_string();
                if !pubkey_str.contains("11111111111111111111111111111111") && 
                   !pubkey_str.contains("So11111111111111111111111111111111111111112") &&
                   !pubkey_str.contains("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v") {
                    println!("Potential vault at position {}: {}", i, pubkey_str);
                }
            }
        }
    }

    // Check Orca pool
    println!("\n📊 Orca Whirlpool pool: {}", orca_pool);
    if let Ok(account) = client.get_account(&Pubkey::from_str(orca_pool)?) {
        let data = account.data;
        println!("Data size: {} bytes", data.len());
        
        // Search for vaults after token mints
        // WSOL is at 101, USDC at 181, so vaults should be after
        println!("🔍 Searching for vaults after position 181...");
        
        for i in 181..data.len() - 32 {
            if let Ok(pubkey) = Pubkey::try_from(&data[i..i+32]) {
                // Check if this looks like a vault (not a known token mint)
                let pubkey_str = pubkey.to_string();
                if !pubkey_str.contains("11111111111111111111111111111111") && 
                   !pubkey_str.contains("So11111111111111111111111111111111111111112") &&
                   !pubkey_str.contains("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v") {
                    println!("Potential vault at position {}: {}", i, pubkey_str);
                }
            }
        }
    }

    Ok(())
}
