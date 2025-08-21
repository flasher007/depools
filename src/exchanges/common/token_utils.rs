// Константы для известных токенов
const KNOWN_TOKENS: &[(&str, &str, u8)] = &[
    // SOL (Wrapped SOL) - правильный mint для пулов
    ("So11111111111111111111111111111111111111112", "SOL", 9),
    
    // USDC - правильный mint
    ("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v", "USDC", 6),
    
    // USDT
    ("Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB", "USDT", 6),
    
    // RAY
    ("4k3Dyjzvzp8eMZWUXbBCjEvwSkkk59S5iCNLY3QrkX6R", "RAY", 6),
];

// Функция для определения токена по mint адресу
pub fn get_token_info(mint: &str) -> Option<(&str, u8)> {
    KNOWN_TOKENS.iter()
        .find(|(addr, _, _)| *addr == mint)
        .map(|(_, symbol, decimals)| (*symbol, *decimals))
}
