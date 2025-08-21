use anyhow::Result;
use solana_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;

#[tokio::main]
async fn main() -> Result<()> {
    let rpc_url = "https://api.mainnet-beta.solana.com";
    let client = RpcClient::new(rpc_url.to_string());
    
    println!("Testing vault addresses...");
    
    let vaults = vec![
        "BGoJdfAA39yRerqpj76PV9uzdQiXnD4cnfR4mhYXLNoU",
        "9PaNcpedM6QCZB91Ly6jfCvfsoKzfwVriSpec1ZYKMMP",
    ];
    
    for vault_str in vaults {
        println!("\nTesting vault: {}", vault_str);
        
        let vault = Pubkey::from_str(vault_str)?;
        
        // Check if account exists
        match client.get_account(&vault) {
            Ok(account) => {
                println!("  ✅ Account exists, size: {} bytes", account.data.len());
                
                // Try to get token account balance
                match client.get_token_account_balance(&vault) {
                    Ok(balance) => {
                        println!("  💰 Token balance: {} (decimals: {})", 
                                balance.ui_amount.unwrap_or(0.0), 
                                balance.decimals);
                    }
                    Err(e) => {
                        println!("  ❌ Failed to get token balance: {}", e);
                    }
                }
            }
            Err(e) => {
                println!("  ❌ Account not found: {}", e);
            }
        }
    }
    
    Ok(())
}
