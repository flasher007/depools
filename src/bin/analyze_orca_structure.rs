use anyhow::Result;
use solana_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;

#[tokio::main]
async fn main() -> Result<()> {
    let rpc_url = "https://api.mainnet-beta.solana.com";
    let client = RpcClient::new(rpc_url.to_string());
    
    let pool_address = "Czfq3xZZDmsdGdUyrNLtRhGc47cXcZtLG4crryfu44zE";
    let pool_pubkey = Pubkey::from_str(pool_address)?;
    
    println!("Analyzing Orca Whirlpool structure for: {}", pool_address);
    
    // Get pool data
    let account = client.get_account(&pool_pubkey)?;
    let data = &account.data;
    
    println!("Pool data size: {} bytes", data.len());
    
    // Search for known token addresses
    let known_tokens = vec![
        ("WSOL", "So11111111111111111111111111111111111111112"),
        ("USDC", "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"),
    ];
    
    for (symbol, address) in known_tokens {
        let token_bytes = Pubkey::from_str(address)?.to_bytes();
        
        // Search for this token in the data
        for (i, window) in data.windows(32).enumerate() {
            if window == token_bytes {
                println!("  Found {} at position {}: {}", symbol, i, address);
            }
        }
    }
    
    // Search for potential vault addresses (look for sequences that could be Pubkeys)
    println!("\nSearching for potential vault addresses...");
    
    // Look for sequences of 32 bytes that could be valid Pubkeys
    for i in (0..data.len()).step_by(32) {
        if i + 32 <= data.len() {
            let potential_pubkey = &data[i..i+32];
            
            // Check if this looks like a valid vault (not all zeros, not all ones)
            let all_zeros = potential_pubkey.iter().all(|&b| b == 0);
            let all_ones = potential_pubkey.iter().all(|&b| b == 0xFF);
            
            if !all_zeros && !all_ones {
                let pubkey = Pubkey::new(potential_pubkey);
                println!("  Position {}: {}", i, pubkey);
                
                // Try to check if this account exists
                match client.get_account(&pubkey) {
                    Ok(acc) => {
                        println!("    ✅ Account exists, size: {} bytes", acc.data.len());
                        
                        // Try to get token balance
                        match client.get_token_account_balance(&pubkey) {
                            Ok(balance) => {
                                println!("    💰 Token balance: {} (decimals: {})", 
                                        balance.ui_amount.unwrap_or(0.0), 
                                        balance.decimals);
                            }
                            Err(_) => {
                                println!("    ❌ Not a token account");
                            }
                        }
                    }
                    Err(_) => {
                        println!("    ❌ Account not found");
                    }
                }
            }
        }
    }
    
    Ok(())
}
