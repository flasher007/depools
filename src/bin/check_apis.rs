use reqwest;
use serde_json::Value;
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Checking Raydium and Orca APIs for SOL-USDC pools...");
    
    // Проверяем Raydium API
    println!("\n📊 Checking Raydium V4 API...");
    let raydium_url = "https://api.raydium.io/v2/sdk/liquidity/mainnet.json";
    
    match reqwest::get(raydium_url).await {
        Ok(response) => {
            match response.json::<Value>().await {
                Ok(data) => {
                    println!("✅ Raydium API response received");
                    
                    // Ищем SOL-USDC пулы
                    if let Some(official) = data.get("official") {
                        if let Some(amm) = official.get("amm") {
                            if let Some(amm_data) = amm.as_array() {
                                println!("📋 Found {} AMM pools", amm_data.len());
                                
                                // Ищем SOL-USDC пулы
                                let mut sol_usdc_pools = Vec::new();
                                for pool in amm_data {
                                    if let (Some(base_mint), Some(quote_mint)) = (
                                        pool.get("baseMint").and_then(|v| v.as_str()),
                                        pool.get("quoteMint").and_then(|v| v.as_str())
                                    ) {
                                        if (base_mint == "So11111111111111111111111111111111111111112" && 
                                             quote_mint == "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v") ||
                                           (base_mint == "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v" && 
                                             quote_mint == "So11111111111111111111111111111111111111112") {
                                            
                                            let amm_id = pool.get("ammId").and_then(|v| v.as_str()).unwrap_or("N/A");
                                            let lp_mint = pool.get("lpMint").and_then(|v| v.as_str()).unwrap_or("N/A");
                                            let base_decimals = pool.get("baseDecimals").and_then(|v| v.as_u64()).unwrap_or(0);
                                            let quote_decimals = pool.get("quoteDecimals").and_then(|v| v.as_u64()).unwrap_or(0);
                                            
                                            sol_usdc_pools.push((amm_id, lp_mint, base_decimals, quote_decimals));
                                        }
                                    }
                                }
                                
                                println!("🪙 Found {} SOL-USDC pools:", sol_usdc_pools.len());
                                for (i, (amm_id, lp_mint, base_dec, quote_dec)) in sol_usdc_pools.iter().enumerate() {
                                    println!("   {}. AMM ID: {}", i + 1, amm_id);
                                    println!("      LP Mint: {}", lp_mint);
                                    println!("      Base Decimals: {}", base_dec);
                                    println!("      Quote Decimals: {}", quote_dec);
                                    println!();
                                }
                            }
                        }
                    }
                },
                Err(e) => println!("❌ Failed to parse Raydium JSON: {}", e),
            }
        },
        Err(e) => println!("❌ Failed to fetch Raydium API: {}", e),
    }
    
    // Проверяем Orca API
    println!("\n🐋 Checking Orca Whirlpool API...");
    let orca_url = "https://api.mainnet.orca.so/v1/whirlpool/list";
    
    match reqwest::get(orca_url).await {
        Ok(response) => {
            match response.json::<Value>().await {
                Ok(data) => {
                    println!("✅ Orca API response received");
                    
                    if let Some(whirlpools) = data.get("whirlpools").and_then(|v| v.as_array()) {
                        println!("📋 Found {} Whirlpools", whirlpools.len());
                        
                        // Ищем SOL-USDC пулы
                        let mut sol_usdc_whirlpools = Vec::new();
                        for pool in whirlpools {
                            if let (Some(token_a), Some(token_b)) = (
                                pool.get("tokenA").and_then(|v| v.get("mint")).and_then(|v| v.as_str()),
                                pool.get("tokenB").and_then(|v| v.get("mint")).and_then(|v| v.as_str())
                            ) {
                                if (token_a == "So11111111111111111111111111111111111111112" && 
                                     token_b == "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v") ||
                                   (token_a == "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v" && 
                                     token_b == "So11111111111111111111111111111111111111112") {
                                    
                                    let address = pool.get("address").and_then(|v| v.as_str()).unwrap_or("N/A");
                                    let tick_spacing = pool.get("tickSpacing").and_then(|v| v.as_u64()).unwrap_or(0);
                                    
                                    sol_usdc_whirlpools.push((address, tick_spacing));
                                }
                            }
                        }
                        
                        println!("🪙 Found {} SOL-USDC Whirlpools:", sol_usdc_whirlpools.len());
                        for (i, (address, tick_spacing)) in sol_usdc_whirlpools.iter().enumerate() {
                            println!("   {}. Address: {}", i + 1, address);
                            println!("      Tick Spacing: {}", tick_spacing);
                            println!();
                        }
                    }
                },
                Err(e) => println!("❌ Failed to parse Orca JSON: {}", e),
            }
        },
        Err(e) => println!("❌ Failed to fetch Orca API: {}", e),
    }
    
    Ok(())
}
