use solana_sdk::{
    instruction::Instruction,
    compute_budget::ComputeBudgetInstruction,
};

/// Create ComputeBudget instruction to set priority fee
pub fn create_priority_fee_instruction(priority_fee_lamports: u64) -> Instruction {
    ComputeBudgetInstruction::set_compute_unit_price(priority_fee_lamports)
}

/// Create ComputeBudget instruction to set compute unit limit
pub fn create_compute_unit_limit_instruction(compute_units: u32) -> Instruction {
    ComputeBudgetInstruction::set_compute_unit_limit(compute_units)
}

/// Create both ComputeBudget instructions for arbitrage transaction
pub fn create_compute_budget_instructions(
    compute_units: u32,
    priority_fee_lamports: u64,
) -> Vec<Instruction> {
    vec![
        create_compute_unit_limit_instruction(compute_units),
        create_priority_fee_instruction(priority_fee_lamports),
    ]
}
