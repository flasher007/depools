use anyhow::Result;
use solana_client::rpc_client::RpcClient;
use solana_sdk::commitment_config::CommitmentConfig;
use tracing::{info, error};

pub async fn test_helius_connection() -> Result<()> {
    info!("Testing Helius RPC connection...");
    
    // Create RPC client with Helius endpoint
    let rpc_url = "https://mainnet.helius-rpc.com/?api-key=b5939e95-d595-4e01-9401-da85b5c720af";
    let rpc_client = RpcClient::new_with_commitment(rpc_url.to_string(), CommitmentConfig::confirmed());
    
    // Test basic connection
    match rpc_client.get_version().await {
        Ok(version) => {
            info!("✅ Successfully connected to Helius RPC!");
            info!("Solana version: {:?}", version);
        }
        Err(e) => {
            error!("❌ Failed to connect to Helius RPC: {}", e);
            return Err(e.into());
        }
    }
    
    // Test getting latest blockhash
    match rpc_client.get_latest_blockhash().await {
        Ok(blockhash) => {
            info!("✅ Latest blockhash: {}", blockhash);
        }
        Err(e) => {
            error!("❌ Failed to get latest blockhash: {}", e);
            return Err(e.into());
        }
    }
    
    // Test getting recent performance samples
    match rpc_client.get_recent_performance_samples(Some(5)).await {
        Ok(samples) => {
            info!("✅ Got {} performance samples", samples.len());
            if let Some(sample) = samples.first() {
                info!("Latest sample - num_transactions: {}, num_slots: {}", 
                       sample.num_transactions, sample.num_slots);
            }
        }
        Err(e) => {
            error!("❌ Failed to get performance samples: {}", e);
            // Don't return error here as this is optional
        }
    }
    
    // Test getting slot info
    match rpc_client.get_slot().await {
        Ok(slot) => {
            info!("✅ Current slot: {}", slot);
        }
        Err(e) => {
            error!("❌ Failed to get current slot: {}", e);
            return Err(e.into());
        }
    }
    
    info!("🎉 Helius RPC connection test completed successfully!");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_helius_connection_works() {
        let result = test_helius_connection().await;
        assert!(result.is_ok(), "Helius connection test failed: {:?}", result);
    }
}
