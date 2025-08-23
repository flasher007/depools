use anyhow::Result;
use solana_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rpc_url = "https://api.mainnet-beta.solana.com";
    let client = RpcClient::new(rpc_url.to_string());

    println!("🔍 Getting vault balances for pools...");

    // Raydium V4 vaults from our parser
    let raydium_base_vault = "FnXVNHzripKNTs5Hs2qBL3cbNbC85ZPKbVdXSsmNNtTq";
    let raydium_quote_vault = "CfD3HGk5hWbSexSx5emfhCSVZmATyN4NLDKAiLKGWYaT";

    // Orca Whirlpool vaults from our parser
    let orca_base_vault = "FnXVNHzripKNTs5Hs2qBL3cbNbC85ZPKbVdXSi52PqwA";
    let orca_quote_vault = "CfD3HGk5hWbSexSx5emfhCSVZmATyN4NLDK9yYF4jPBE";

    println!("\n📊 Raydium V4 vaults:");
    println!("  Base vault (WSOL): {}", raydium_base_vault);
    if let Ok(account) = client.get_account(&Pubkey::from_str(raydium_base_vault)?) {
        println!("    Account data size: {} bytes", account.data.len());
        println!("    Owner: {}", account.owner);
        // Parse token account balance
        if account.data.len() >= 72 {
            let balance = u64::from_le_bytes([
                account.data[64], account.data[65], account.data[66], account.data[67],
                account.data[68], account.data[69], account.data[70], account.data[71]
            ]);
            println!("    Balance: {} WSOL ({} lamports)", balance as f64 / 1_000_000_000.0, balance);
        }
    }

    println!("  Quote vault (USDC): {}", raydium_quote_vault);
    if let Ok(account) = client.get_account(&Pubkey::from_str(raydium_quote_vault)?) {
        println!("    Account data size: {} bytes", account.data.len());
        println!("    Owner: {}", account.owner);
        // Parse token account balance
        if account.data.len() >= 72 {
            let balance = u64::from_le_bytes([
                account.data[64], account.data[65], account.data[66], account.data[67],
                account.data[68], account.data[69], account.data[70], account.data[71]
            ]);
            println!("    Balance: {} USDC ({} units)", balance as f64 / 1_000_000.0, balance);
        }
    }

    println!("\n📊 Orca Whirlpool vaults:");
    println!("  Base vault (WSOL): {}", orca_base_vault);
    if let Ok(account) = client.get_account(&Pubkey::from_str(orca_base_vault)?) {
        println!("    Account data size: {} bytes", account.data.len());
        println!("    Owner: {}", account.owner);
        // Parse token account balance
        if account.data.len() >= 72 {
            let balance = u64::from_le_bytes([
                account.data[64], account.data[65], account.data[66], account.data[67],
                account.data[68], account.data[69], account.data[70], account.data[71]
            ]);
            println!("    Balance: {} WSOL ({} lamports)", balance as f64 / 1_000_000_000.0, balance);
        }
    }

    println!("  Quote vault (USDC): {}", orca_quote_vault);
    if let Ok(account) = client.get_account(&Pubkey::from_str(orca_quote_vault)?) {
        println!("    Account data size: {} bytes", account.data.len());
        println!("    Owner: {}", account.owner);
        // Parse token account balance
        if account.data.len() >= 72 {
            let balance = u64::from_le_bytes([
                account.data[64], account.data[65], account.data[66], account.data[67],
                account.data[68], account.data[69], account.data[70], account.data[71]
            ]);
            println!("    Balance: {} USDC ({} units)", balance as f64 / 1_000_000.0, balance);
        }
    }

    Ok(())
}
