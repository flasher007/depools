use anyhow::Result;
use solana_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rpc_url = "https://api.mainnet-beta.solana.com";
    let client = RpcClient::new(rpc_url.to_string());

    let raydium_pool = "58oQChx4yWmvKdwLLZzBi4ChoCc2fqCUWBkwMihLYQo2";

    println!("🔍 Analyzing Raydium V4 pool structure...");
    println!("Pool: {}", raydium_pool);

    if let Ok(account) = client.get_account(&Pubkey::from_str(raydium_pool)?) {
        let data = account.data;
        println!("Data size: {} bytes", data.len());
        
        // Analyze first 100 bytes in detail
        println!("\n📊 First 100 bytes analysis:");
        for i in 0..100 {
            if i % 16 == 0 {
                println!("\n{:04X}:", i);
            }
            print!("{:02X} ", data[i]);
        }
        
        // Look for known patterns
        println!("\n\n🔍 Looking for known patterns:");
        
        // Search for "Raydium" string
        if let Some(pos) = find_string(&data, "Raydium") {
            println!("  Found 'Raydium' at position {}", pos);
        }
        
        // Search for "Serum" string
        if let Some(pos) = find_string(&data, "Serum") {
            println!("  Found 'Serum' at position {}", pos);
        }
        
        // Search for "OpenBook" string
        if let Some(pos) = find_string(&data, "OpenBook") {
            println!("  Found 'OpenBook' at position {}", pos);
        }
        
        // Look for potential program IDs
        println!("\n🔍 Potential program IDs:");
        for i in 0..data.len() - 31 {
            if let Ok(pubkey) = Pubkey::try_from(&data[i..i+32]) {
                let pubkey_str = pubkey.to_string();
                
                // Check if this looks like a program ID
                if pubkey_str.contains("11111111111111111111111111111111") {
                    continue; // Skip System Program
                }
                
                // Check if this might be a known program
                if pubkey_str.contains("9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM") || // Raydium V4
                   pubkey_str.contains("srmqPvymJeFKQ4zGQed1GFppgkRHL9kaELCbyksJtPX") || // Serum/OpenBook
                   pubkey_str.contains("675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8") {  // Raydium AMM
                    println!("  Position {}: {} (Known program)", i, pubkey_str);
                }
            }
        }
        
        // Look for potential vault addresses (different approach)
        println!("\n🔍 Looking for potential vault addresses (different approach):");
        
        // Search for addresses that might be vaults by looking at their structure
        for i in 0..data.len() - 31 {
            if let Ok(pubkey) = Pubkey::try_from(&data[i..i+32]) {
                let pubkey_str = pubkey.to_string();
                
                // Skip known token mints
                if pubkey_str == "So11111111111111111111111111111111111111112" ||
                   pubkey_str == "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v" {
                    continue;
                }
                
                // Skip System Program
                if pubkey_str.contains("11111111111111111111111111111111") {
                    continue;
                }
                
                // Look for addresses that might be vaults
                // Vault addresses often have specific patterns
                let ones = pubkey_str.chars().filter(|&c| c == '1').count();
                let twos = pubkey_str.chars().filter(|&c| c == '2').count();
                let threes = pubkey_str.chars().filter(|&c| c == '3').count();
                let fours = pubkey_str.chars().filter(|&c| c == '4').count();
                let fives = pubkey_str.chars().filter(|&c| c == '5').count();
                let sixes = pubkey_str.chars().filter(|&c| c == '6').count();
                let sevens = pubkey_str.chars().filter(|&c| c == '7').count();
                let eights = pubkey_str.chars().filter(|&c| c == '8').count();
                let nines = pubkey_str.chars().filter(|&c| c == '9').count();
                let zeros = pubkey_str.chars().filter(|&c| c == '0').count();
                let as_count = pubkey_str.chars().filter(|&c| c == 'A').count();
                let bs = pubkey_str.chars().filter(|&c| c == 'B').count();
                let cs = pubkey_str.chars().filter(|&c| c == 'C').count();
                let ds = pubkey_str.chars().filter(|&c| c == 'D').count();
                let es = pubkey_str.chars().filter(|&c| c == 'E').count();
                let fs = pubkey_str.chars().filter(|&c| c == 'F').count();
                
                // Vault addresses often have a mix of characters
                if ones > 15 && (twos > 5 || threes > 5 || fours > 5 || fives > 5 || 
                                 sixes > 5 || sevens > 5 || eights > 5 || nines > 5 ||
                                 zeros > 5 || as_count > 5 || bs > 5 || cs > 5 || 
                                 ds > 5 || es > 5 || fs > 5) {
                    println!("  Position {}: {} (Potential vault - mixed chars)", i, pubkey_str);
                }
            }
        }
    }

    Ok(())
}

fn find_string(data: &[u8], search: &str) -> Option<usize> {
    let search_bytes = search.as_bytes();
    for i in 0..data.len() - search_bytes.len() + 1 {
        if data[i..i + search_bytes.len()] == *search_bytes {
            return Some(i);
        }
    }
    None
}
