use solana_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;

const KNOWN_TOKENS: &[(&str, &str)] = &[
    ("So11111111111111111111111111111111111111112", "WSOL"),
    ("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v", "USDC"),
    ("11111111111111111111111111111111", "SOL"),
];

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rpc_url = "https://api.mainnet-beta.solana.com";
    let client = RpcClient::new(rpc_url.to_string());

    let raydium_pool = "58oQChx4yWmvKdwLLZzBi4ChoCc2fqCUWBkwMihLYQo2";
    let orca_pool = "Czfq3xZZDmsdGdUyrNLtRhGc47cXcZtLG4crryfu44zE";

    println!("🔍 Searching for all known tokens in pools...");

    // Check Raydium pool
    println!("\n📊 Raydium V4 pool: {}", raydium_pool);
    if let Ok(account) = client.get_account(&Pubkey::from_str(raydium_pool)?) {
        let data = account.data;
        println!("Data size: {} bytes", data.len());
        
        for (mint, symbol) in KNOWN_TOKENS {
            let pubkey = Pubkey::from_str(mint)?;
            let pubkey_bytes = pubkey.to_bytes();
            
            if let Some(pos) = find_byte_sequence(&data, &pubkey_bytes) {
                println!("✅ Found {} ({}) at position: {}", symbol, mint, pos);
            }
        }
    }

    // Check Orca pool
    println!("\n📊 Orca Whirlpool pool: {}", orca_pool);
    if let Ok(account) = client.get_account(&Pubkey::from_str(orca_pool)?) {
        let data = account.data;
        println!("Data size: {} bytes", data.len());
        
        for (mint, symbol) in KNOWN_TOKENS {
            let pubkey = Pubkey::from_str(mint)?;
            let pubkey_bytes = pubkey.to_bytes();
            
            if let Some(pos) = find_byte_sequence(&data, &pubkey_bytes) {
                println!("✅ Found {} ({}) at position: {}", symbol, mint, pos);
            }
        }
    }

    Ok(())
}

fn find_byte_sequence(data: &[u8], sequence: &[u8]) -> Option<usize> {
    data.windows(sequence.len()).position(|window| window == sequence)
}
