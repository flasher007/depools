use reqwest;
use serde_json::Value;
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Searching for REAL SOL-USDC pools...");
    
    // SOL mint address
    let sol_mint = "So11111111111111111111111111111111111111112";
    // USDC mint address  
    let usdc_mint = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";
    
    println!("🎯 Looking for pools with:");
    println!("   SOL: {}", sol_mint);
    println!("   USDC: {}", usdc_mint);
    
    // 1. Search Raydium V4 API for SOL-USDC pools
    println!("\n📊 Searching Raydium V4 API...");
    let raydium_url = "https://api.raydium.io/v2/sdk/liquidity/mainnet.json";
    
    match reqwest::get(raydium_url).await {
        Ok(response) => {
            match response.json::<Value>().await {
                Ok(data) => {
                    println!("✅ Raydium API response received");
                    
                    // Look for SOL-USDC pools
                    if let Some(official) = data.get("official") {
                        if let Some(amm) = official.get("amm") {
                            if let Some(amm_data) = amm.as_array() {
                                println!("📋 Found {} AMM pools", amm_data.len());
                                
                                let mut sol_usdc_pools = Vec::new();
                                
                                for pool in amm_data {
                                    if let (Some(base_mint), Some(quote_mint)) = (
                                        pool.get("baseMint").and_then(|v| v.as_str()),
                                        pool.get("quoteMint").and_then(|v| v.as_str())
                                    ) {
                                        if (base_mint == sol_mint && quote_mint == usdc_mint) ||
                                           (base_mint == usdc_mint && quote_mint == sol_mint) {
                                            let pool_id = pool.get("ammId").and_then(|v| v.as_str()).unwrap_or("Unknown");
                                            let tvl = pool.get("tvl").and_then(|v| v.as_f64()).unwrap_or(0.0);
                                            let volume_24h = pool.get("volume24h").and_then(|v| v.as_f64()).unwrap_or(0.0);
                                            
                                            sol_usdc_pools.push((pool_id.to_string(), tvl, volume_24h));
                                        }
                                    }
                                }
                                
                                if sol_usdc_pools.is_empty() {
                                    println!("❌ No SOL-USDC pools found in Raydium V4");
                                } else {
                                    println!("✅ Found {} SOL-USDC pools in Raydium V4:", sol_usdc_pools.len());
                                    sol_usdc_pools.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
                                    
                                    for (i, (pool_id, tvl, volume)) in sol_usdc_pools.iter().take(10).enumerate() {
                                        println!("   {}. Pool: {}", i + 1, pool_id);
                                        println!("      TVL: ${:.2}", tvl);
                                        println!("      Volume 24h: ${:.2}", volume);
                                    }
                                }
                            }
                        }
                    }
                }
                Err(e) => println!("❌ Failed to parse Raydium API: {}", e),
            }
        }
        Err(e) => println!("❌ Failed to fetch Raydium API: {}", e),
    }
    
    // 2. Search Orca API for SOL-USDC Whirlpools
    println!("\n🐋 Searching Orca Whirlpool API...");
    let orca_url = "https://api.mainnet.orca.so/v1/whirlpool/list";
    
    match reqwest::get(orca_url).await {
        Ok(response) => {
            match response.json::<Value>().await {
                Ok(data) => {
                    println!("✅ Orca API response received");
                    
                    if let Some(whirlpools) = data.as_array() {
                        println!("📋 Found {} Whirlpools", whirlpools.len());
                        
                        let mut sol_usdc_whirlpools = Vec::new();
                        
                        for pool in whirlpools {
                            if let (Some(token_a), Some(token_b)) = (
                                pool.get("tokenA").and_then(|v| v.get("mint")).and_then(|v| v.as_str()),
                                pool.get("tokenB").and_then(|v| v.get("mint")).and_then(|v| v.as_str())
                            ) {
                                if (token_a == sol_mint && token_b == usdc_mint) ||
                                   (token_a == usdc_mint && token_b == sol_mint) {
                                    let pool_address = pool.get("address").and_then(|v| v.as_str()).unwrap_or("Unknown");
                                    let tick_spacing = pool.get("tickSpacing").and_then(|v| v.as_u64()).unwrap_or(0);
                                    let tvl = pool.get("tvl").and_then(|v| v.as_f64()).unwrap_or(0.0);
                                    
                                    sol_usdc_whirlpools.push((pool_address.to_string(), tick_spacing, tvl));
                                }
                            }
                        }
                        
                        if sol_usdc_whirlpools.is_empty() {
                            println!("❌ No SOL-USDC Whirlpools found");
                        } else {
                            println!("✅ Found {} SOL-USDC Whirlpools:", sol_usdc_whirlpools.len());
                            sol_usdc_whirlpools.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap());
                            
                            for (i, (pool_address, tick_spacing, tvl)) in sol_usdc_whirlpools.iter().take(10).enumerate() {
                                println!("   {}. Pool: {}", i + 1, pool_address);
                                println!("      Tick Spacing: {}", tick_spacing);
                                println!("      TVL: ${:.2}", tvl);
                            }
                        }
                    }
                }
                Err(e) => println!("❌ Failed to parse Orca API: {}", e),
            }
        }
        Err(e) => println!("❌ Failed to fetch Orca API: {}", e),
    }
    
    println!("\n🎯 Summary:");
    println!("   - Use the pool addresses above for REAL SOL-USDC arbitrage");
    println!("   - Current pools in config are USDC-USDC (not SOL-USDC)");
    println!("   - Update Config.toml with real SOL-USDC pool addresses");
    
    Ok(())
}
