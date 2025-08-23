use anyhow::Result;
use solana_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rpc_url = "https://api.mainnet-beta.solana.com";
    let client = RpcClient::new(rpc_url.to_string());

    let raydium_pool = "58oQChx4yWmvKdwLLZzBi4ChoCc2fqCUWBkwMihLYQo2";
    let orca_pool = "Czfq3xZZDmsdGdUyrNLtRhGc47cXcZtLG4crryfu44zE";

    println!("🔍 Searching for reserve positions in pool data...");

    // Analyze Raydium pool
    println!("\n📊 Raydium V4 pool: {}", raydium_pool);
    if let Ok(account) = client.get_account(&Pubkey::from_str(raydium_pool)?) {
        let data = account.data;
        println!("Data size: {} bytes", data.len());
        
        // Search for reserve-related values
        println!("\n🔍 Searching for reserve-related values...");
        
        // Search for swap amounts from API
        search_for_u64(&data, 12525690016865650, "swapBaseInAmount");
        search_for_u64(&data, 1159911068613210, "swapQuoteOutAmount");
        search_for_u64(&data, 1174450762128042, "swapQuoteInAmount");
        search_for_u64(&data, 12729904466017689, "swapBaseOutAmount");
        
        // Search for PnL values
        search_for_u64(&data, 64285653, "baseNeedTakePnl");
        search_for_u64(&data, 12027407, "quoteNeedTakePnl");
        search_for_u64(&data, 3181802036811, "quoteTotalPnl");
        search_for_u64(&data, 49557593921531, "baseTotalPnl");
        
        // Search for LP supply
        search_for_u64(&data, 88287903869163, "lpReserve");
    }

    // Analyze Orca pool
    println!("\n📊 Orca Whirlpool pool: {}", orca_pool);
    if let Ok(account) = client.get_account(&Pubkey::from_str(orca_pool)?) {
        let data = account.data;
        println!("Data size: {} bytes", data.len());
        
        // Search for reserve-related values
        println!("\n🔍 Searching for reserve-related values...");
        
        // Search for protocol fees
        search_for_u64(&data, 390305397, "protocolFeeOwedA");
        search_for_u64(&data, 51436657, "protocolFeeOwedB");
        
        // Search for fee growth
        search_for_u128(&data, 10248334973510346052, "feeGrowthGlobalA");
        search_for_u128(&data, 1344234359596287297, "feeGrowthGlobalB");
        
        // Search for reward timestamp
        search_for_u64(&data, 1755759182, "rewardLastUpdatedTimestamp");
    }

    Ok(())
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
