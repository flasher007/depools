use solana_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;

const USDC_MINT: &str = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rpc_url = "https://api.mainnet-beta.solana.com";
    let client = RpcClient::new(rpc_url.to_string());

    let raydium_pool = "58oQChx4yWmvKdwLLZzBi4ChoCc2fqCUWBkwMihLYQo2";
    let orca_pool = "Czfq3xZZDmsdGdUyrNLtRhGc47cXcZtLG4crryfu44zE";

    println!("🔍 Searching for USDC token in pools...");
    println!("USDC mint: {}", USDC_MINT);

    // Check Raydium pool
    println!("\n📊 Raydium V4 pool: {}", raydium_pool);
    if let Ok(account) = client.get_account(&Pubkey::from_str(raydium_pool)?) {
        let data = account.data;
        println!("Data size: {} bytes", data.len());
        
        // Search for USDC mint
        let usdc_bytes = Pubkey::from_str(USDC_MINT)?.to_bytes();
        if let Some(pos) = find_byte_sequence(&data, &usdc_bytes) {
            println!("✅ Found USDC at position: {}", pos);
        } else {
            println!("❌ USDC not found in Raydium pool");
        }
    }

    // Check Orca pool
    println!("\n📊 Orca Whirlpool pool: {}", orca_pool);
    if let Ok(account) = client.get_account(&Pubkey::from_str(orca_pool)?) {
        let data = account.data;
        println!("Data size: {} bytes", data.len());
        
        // Search for USDC mint
        let usdc_bytes = Pubkey::from_str(USDC_MINT)?.to_bytes();
        if let Some(pos) = find_byte_sequence(&data, &usdc_bytes) {
            println!("✅ Found USDC at position: {}", pos);
        } else {
            println!("❌ USDC not found in Orca pool");
        }
    }

    Ok(())
}

fn find_byte_sequence(data: &[u8], sequence: &[u8]) -> Option<usize> {
    data.windows(sequence.len()).position(|window| window == sequence)
}
