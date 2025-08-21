use anyhow::Result;
use solana_sdk::{
    instruction::Instruction,
    transaction::Transaction,
    message::Message,
    signature::{Keypair, Signer},
    pubkey::Pubkey,
    hash::Hash,
};
use crate::exchanges::{
    types::{ArbitrageOpportunity, SwapQuote},
    compute_budget::{create_compute_budget_instructions},
    DexAdapter,
};
use crate::math::calculate_min_out;
use tracing::{info, warn};
use crate::exchanges::utils::{lamports_to_sol, format_sol};

pub struct TransactionBuilder;

impl TransactionBuilder {
    /// Build atomic arbitrage transaction with ComputeBudget and two swap instructions
    pub fn build_arbitrage_transaction(
        &self,
        opportunity: &ArbitrageOpportunity,
        user_keypair: &Keypair,
        recent_blockhash: Hash,
        adapters: &[Box<dyn DexAdapter>],
        slippage_bps: u32,
        priority_fee: u64,
    ) -> Result<Transaction> {
        info!("🔨 Building atomic arbitrage transaction");
        let amount_in_sol = lamports_to_sol(opportunity.route_a.hops[0].amount_in);
        let amount_out_sol = lamports_to_sol(opportunity.route_b.hops[0].amount_out);
        info!("💰 Opportunity: {} -> {}", format_sol(amount_in_sol), format_sol(amount_out_sol));
        
        let mut instructions = Vec::new();
        
        // 1. Add ComputeBudget instructions
        let compute_units = 400_000; // Conservative estimate for arbitrage
        let compute_budget_instructions = create_compute_budget_instructions(compute_units, priority_fee);
        instructions.extend(compute_budget_instructions);
        let priority_fee_sol = lamports_to_sol(priority_fee);
        info!("✅ Added ComputeBudget instructions: {} CU, {} priority fee", compute_units, format_sol(priority_fee_sol));
        
        // 2. Find adapters for each route
        let route_a_adapter = self.find_adapter_for_dex(adapters, opportunity.route_a.hops[0].dex_label)?;
        let route_b_adapter = self.find_adapter_for_dex(adapters, opportunity.route_b.hops[0].dex_label)?;
        
        // 3. Create first swap instruction (Route A)
        let quote_a = SwapQuote {
            pool_address: opportunity.route_a.hops[0].pool_address,
            dex_label: opportunity.route_a.hops[0].dex_label,
            token_in: opportunity.route_a.hops[0].token_in,
            token_out: opportunity.route_a.hops[0].token_out,
            amount_in: opportunity.route_a.hops[0].amount_in,
            amount_out: opportunity.route_a.hops[0].amount_out,
            min_amount_out: opportunity.min_out_a,
            price_impact_bps: 0, // Simplified
            fee_amount: 0, // Simplified
            route: opportunity.route_a.clone(),
        };
        
        let min_out_a = calculate_min_out(quote_a.amount_out, slippage_bps);
        let swap_instruction_a = route_a_adapter.create_swap_instruction(
            &quote_a,
            &user_keypair.pubkey(),
            min_out_a,
        )?;
        instructions.push(swap_instruction_a);
        let amount_in_sol = lamports_to_sol(quote_a.amount_in);
        let amount_out_sol = lamports_to_sol(quote_a.amount_out);
        let min_out_sol = lamports_to_sol(min_out_a);
        info!("✅ Added first swap instruction: {} -> {} (min_out: {})", 
              format_sol(amount_in_sol), format_sol(amount_out_sol), format_sol(min_out_sol));
        
        // 4. Create second swap instruction (Route B)
        let quote_b = SwapQuote {
            pool_address: opportunity.route_b.hops[0].pool_address,
            dex_label: opportunity.route_b.hops[0].dex_label,
            token_in: opportunity.route_b.hops[0].token_in,
            token_out: opportunity.route_b.hops[0].token_out,
            amount_in: opportunity.route_b.hops[0].amount_in,
            amount_out: opportunity.route_b.hops[0].amount_out,
            min_amount_out: opportunity.min_out_b,
            price_impact_bps: 0, // Simplified
            fee_amount: 0, // Simplified
            route: opportunity.route_b.clone(),
        };
        
        let min_out_b = calculate_min_out(quote_b.amount_out, slippage_bps);
        let swap_instruction_b = route_b_adapter.create_swap_instruction(
            &quote_b,
            &user_keypair.pubkey(),
            min_out_b,
        )?;
        instructions.push(swap_instruction_b);
        let amount_in_sol = lamports_to_sol(quote_b.amount_in);
        let amount_out_sol = lamports_to_sol(quote_b.amount_out);
        let min_out_sol = lamports_to_sol(min_out_b);
        info!("✅ Added second swap instruction: {} -> {} (min_out: {})", 
              format_sol(amount_in_sol), format_sol(amount_out_sol), format_sol(min_out_sol));
        
        // 5. Build transaction
        let message = Message::new(&instructions, Some(&user_keypair.pubkey()));
        let mut transaction = Transaction::new_unsigned(message);
        transaction.partial_sign(&[user_keypair], recent_blockhash);
        
        info!("🎯 Built atomic transaction with {} instructions", instructions.len());
        info!("📝 Transaction size: {} bytes", transaction.message_data().len());
        
        Ok(transaction)
    }
    
    /// Find adapter for specific DEX label
    fn find_adapter_for_dex<'a>(
        &self,
        adapters: &'a [Box<dyn DexAdapter>],
        dex_label: crate::exchanges::types::DexLabel,
    ) -> Result<&'a Box<dyn DexAdapter>> {
        for adapter in adapters {
            if adapter.get_label() == dex_label {
                return Ok(adapter);
            }
        }
        Err(anyhow::anyhow!("No adapter found for DEX: {:?}", dex_label))
    }
    
    /// Estimate transaction size for fee calculation
    pub fn estimate_transaction_size(instruction_count: usize) -> usize {
        // Base transaction overhead + instructions + signatures
        let base_size = 64; // Message header
        let signature_size = 64; // Per signature
        let instruction_overhead = 32; // Per instruction overhead
        let estimated_instruction_size = 200; // Average instruction size
        
        base_size + signature_size + (instruction_count * (instruction_overhead + estimated_instruction_size))
    }
    
    /// Validate transaction before execution
    pub fn validate_transaction(&self, transaction: &Transaction) -> Result<()> {
        if transaction.message.instructions.is_empty() {
            return Err(anyhow::anyhow!("Transaction has no instructions"));
        }
        
        if transaction.message.instructions.len() < 3 {
            warn!("⚠️ Transaction has fewer than 3 instructions (expected: ComputeBudget + 2 swaps)");
        }
        
        let tx_size = transaction.message_data().len();
        if tx_size > 1232 { // Solana transaction size limit
            return Err(anyhow::anyhow!("Transaction too large: {} bytes", tx_size));
        }
        
        info!("✅ Transaction validation passed: {} instructions, {} bytes", 
              transaction.message.instructions.len(), tx_size);
        
        Ok(())
    }
}
