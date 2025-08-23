use anyhow::Result;
use solana_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::program_pack::Pack;
use spl_token::state::Account as SplAccount;

// Константы для известных токенов
const KNOWN_TOKENS: &[(&str, &str, u8)] = &[
    ("So11111111111111111111111111111111111111112", "WSOL", 9),
    ("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v", "USDC", 6),
];

// Функция для определения токена по mint адресу
pub fn get_token_info(mint: &str) -> Option<(&str, u8)> {
    KNOWN_TOKENS
        .iter()
        .find(|(known_mint, _, _)| *known_mint == mint)
        .map(|(_, symbol, decimals)| (*symbol, *decimals))
}

/// Read SPL token account balance
pub async fn spl_token_balance(rpc: &RpcClient, token_account: &Pubkey) -> Result<u64> {
    let acc = rpc.get_account(token_account)?;
    let ta = SplAccount::unpack(&acc.data)?;
    Ok(ta.amount)
}
