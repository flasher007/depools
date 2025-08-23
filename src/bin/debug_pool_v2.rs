use anyhow::Result;
use solana_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rpc_url = "https://api.mainnet-beta.solana.com";
    let client = RpcClient::new(rpc_url.to_string());

    let raydium_pool = "58oQChx4yWmvKdwLLZzBi4ChoCc2fqCUWBkwMihLYQo2";
    let orca_pool = "Czfq3xZZDmsdGdUyrNLtRhGc47cXcZtLG4crryfu44zE";

    println!("🔍 Detailed analysis of pool data structures...");

    // Analyze Raydium pool
    println!("\n📊 Raydium V4 pool: {}", raydium_pool);
    if let Ok(account) = client.get_account(&Pubkey::from_str(raydium_pool)?) {
        let data = account.data;
        println!("Data size: {} bytes", data.len());
        
        // Search for known values from the API response
        println!("\n🔍 Searching for known values from Raydium API...");
        
        // Search for trade fee: 25/10000 = 0.0025
        search_for_u32(&data, 25, "tradeFeeNumerator");
        search_for_u32(&data, 10000, "tradeFeeDenominator");
        
        // Search for swap fee: 25/10000 = 0.0025
        search_for_u32(&data, 25, "swapFeeNumerator");
        search_for_u32(&data, 10000, "swapFeeDenominator");
        
        // Search for base lot size: 1000000
        search_for_u64(&data, 1000000, "baseLotSize");
        
        // Search for quote lot size: 1000000
        search_for_u64(&data, 1000000, "quoteLotSize");
        
        // Search for state: 2
        search_for_u8(&data, 2, "state");
        
        // Search for nonce: 254
        search_for_u8(&data, 254, "nonce");
        
        // Search for maxOrder: 7
        search_for_u8(&data, 7, "maxOrder");
        
        // Search for depth: 3
        search_for_u8(&data, 3, "depth");
        
        // Search for baseDecimal: 9
        search_for_u8(&data, 9, "baseDecimal");
        
        // Search for quoteDecimal: 6
        search_for_u8(&data, 6, "quoteDecimal");
    }

    // Analyze Orca pool
    println!("\n📊 Orca Whirlpool pool: {}", orca_pool);
    if let Ok(account) = client.get_account(&Pubkey::from_str(orca_pool)?) {
        let data = account.data;
        println!("Data size: {} bytes", data.len());
        
        // Search for known values from the API response
        println!("\n🔍 Searching for known values from Orca API...");
        
        // Search for feeRate: 400 (0.04%)
        search_for_u16(&data, 400, "feeRate");
        
        // Search for protocolFeeRate: 1300 (0.13%)
        search_for_u16(&data, 1300, "protocolFeeRate");
        
        // Search for tickSpacing: 4
        search_for_u16(&data, 4, "tickSpacing");
        
        // Search for tickCurrentIndex: -16783
        search_for_i32(&data, -16783, "tickCurrentIndex");
        
        // Search for liquidity: 971770922408936
        search_for_u128(&data, 971770922408936, "liquidity");
        
        // Search for sqrtPrice: 7971149281086566658
        search_for_u128(&data, 7971149281086566658, "sqrtPrice");
    }

    Ok(())
}

fn search_for_u8(data: &[u8], value: u8, name: &str) {
    for (i, &byte) in data.iter().enumerate() {
        if byte == value {
            println!("  Found {} = {} at position {}", name, value, i);
        }
    }
}

fn search_for_u16(data: &[u8], value: u16, name: &str) {
    for i in 0..data.len() - 1 {
        let found = u16::from_le_bytes([data[i], data[i+1]]);
        if found == value {
            println!("  Found {} = {} at position {}", name, value, i);
        }
    }
}

fn search_for_u32(data: &[u8], value: u32, name: &str) {
    for i in 0..data.len() - 3 {
        let found = u32::from_le_bytes([data[i], data[i+1], data[i+2], data[i+3]]);
        if found == value {
            println!("  Found {} = {} at position {}", name, value, i);
        }
    }
}

fn search_for_u64(data: &[u8], value: u64, name: &str) {
    for i in 0..data.len() - 7 {
        let found = u64::from_le_bytes([
            data[i], data[i+1], data[i+2], data[i+3],
            data[i+4], data[i+5], data[i+6], data[i+7]
        ]);
        if found == value {
            println!("  Found {} = {} at position {}", name, value, i);
        }
    }
}

fn search_for_i32(data: &[u8], value: i32, name: &str) {
    for i in 0..data.len() - 3 {
        let found = i32::from_le_bytes([data[i], data[i+1], data[i+2], data[i+3]]);
        if found == value {
            println!("  Found {} = {} at position {}", name, value, i);
        }
    }
}

fn search_for_u128(data: &[u8], value: u128, name: &str) {
    for i in 0..data.len() - 15 {
        let found = u128::from_le_bytes([
            data[i], data[i+1], data[i+2], data[i+3],
            data[i+4], data[i+5], data[i+6], data[i+7],
            data[i+8], data[i+9], data[i+10], data[i+11],
            data[i+12], data[i+13], data[i+14], data[i+15]
        ]);
        if found == value {
            println!("  Found {} = {} at position {}", name, value, i);
        }
    }
}
