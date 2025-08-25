use crate::shared::errors::AppError;
use crate::shared::types::{Amount, Token};
use crate::domain::dex::{DexType, PoolInfo};
use crate::infrastructure::blockchain::TokenMetadataService;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{info, warn, debug, error};

/// Информация о цене токена
#[derive(Debug, Clone)]
pub struct PriceData {
    pub token_mint: String,
    pub price: f64,
    pub dex_type: DexType,
    pub pool_id: String,
    pub timestamp: Instant,
    pub liquidity: Amount,
    pub volume_24h: Option<f64>, // 24-часовой объем торгов
    pub price_change_24h: Option<f64>, // Изменение цены за 24 часа
}

/// Арбитражный маршрут
#[derive(Debug, Clone)]
pub struct ArbitrageRoute {
    pub id: String,
    pub steps: Vec<ArbitrageStep>,
    pub expected_profit: f64,
    pub profit_percentage: f64,
    pub total_cost: Amount,
    pub risk_score: f64,
    pub timestamp: Instant,
    pub confidence_score: f64, // Уверенность в расчетах (0.0 - 1.0)
    pub execution_time_estimate: Duration, // Оценка времени выполнения
}

/// Шаг арбитража
#[derive(Debug, Clone)]
pub struct ArbitrageStep {
    pub dex_type: DexType,
    pub pool_id: String,
    pub token_in: Token,
    pub token_out: Token,
    pub amount_in: Amount,
    pub expected_amount_out: Amount,
    pub price_impact: f64,
    pub fee: Amount,
    pub slippage_estimate: f64, // Оценка проскальзывания
    pub gas_estimate: Amount, // Оценка газа для этого шага
}

/// Расчет прибыли
#[derive(Debug, Clone)]
pub struct ProfitCalculation {
    pub gross_profit: f64,
    pub net_profit: f64,
    pub gas_cost: Amount,
    pub slippage_cost: f64,
    pub fee_cost: f64,
    pub profit_margin: f64,
    pub is_profitable: bool,
    pub roi_percentage: f64, // ROI в процентах
    pub break_even_amount: f64, // Минимальная сумма для безубыточности
}

/// Детектор арбитражных возможностей
pub struct ArbitrageOpportunityDetector {
    min_profit_threshold: f64,
    min_liquidity: Amount,
    price_cache: Arc<RwLock<HashMap<String, PriceData>>>,
    max_route_length: usize,
    risk_tolerance: f64,
    token_metadata: Arc<TokenMetadataService>,
    max_slippage: f64, // Максимальное допустимое проскальзывание
    min_confidence_score: f64, // Минимальная уверенность в расчетах
}

impl ArbitrageOpportunityDetector {
    /// Создать новый детектор
    pub fn new(
        min_profit_threshold: f64,
        min_liquidity: Amount,
        max_route_length: usize,
        risk_tolerance: f64,
        token_metadata: Arc<TokenMetadataService>,
    ) -> Self {
        Self {
            min_profit_threshold,
            min_liquidity,
            price_cache: Arc::new(RwLock::new(HashMap::new())),
            max_route_length,
            risk_tolerance,
            token_metadata,
            max_slippage: 0.10, // 10% максимальное проскальзывание (было 5%)
            min_confidence_score: 0.4, // 40% минимальная уверенность (было 70%)
        }
    }

    /// Создать с настройками по умолчанию
    pub fn new_default(token_metadata: Arc<TokenMetadataService>) -> Self {
        Self::new(
            0.001, // 0.1% минимальная прибыль (было 0.5%)
            Amount::new(1000000000, 9), // 1 SOL минимальная ликвидность
            4, // Максимум 4 шага в маршруте
            0.5, // Высокий риск (было 0.3)
            token_metadata,
        )
    }

    /// Поиск арбитражных маршрутов с улучшенными алгоритмами
    pub async fn find_arbitrage_routes(&self, price_data: &[PriceData]) -> Vec<ArbitrageRoute> {
        info!("🔍 Поиск арбитражных маршрутов...");
        
        let mut routes = Vec::new();
        let mut processed_pairs: HashSet<String> = HashSet::new();

        // Группируем цены по токенам
        let mut token_prices: HashMap<String, Vec<&PriceData>> = HashMap::new();
        for price in price_data {
            token_prices.entry(price.token_mint.clone()).or_insert_with(Vec::new).push(price);
        }

        info!("📊 Анализируем {} токенов с ценами", token_prices.len());

        // Ищем арбитражные возможности для каждого токена
        let mut total_pairs_checked = 0;
        let mut profitable_pairs_found = 0;
        
        for (token_mint, prices) in &token_prices {
            if prices.len() < 2 {
                debug!("⏭️  Токен {}: недостаточно DEX ({}), пропускаем", token_mint, prices.len());
                continue; // Нужно минимум 2 DEX для арбитража
            }

            info!("🔍 Анализируем токен {}: {} DEX найдено", token_mint, prices.len());
            
            // Сортируем цены по убыванию
            let mut sorted_prices: Vec<_> = prices.iter().collect();
            sorted_prices.sort_by(|a, b| b.price.partial_cmp(&a.price).unwrap_or(std::cmp::Ordering::Equal));

            // Показываем диапазон цен
            let highest_price = sorted_prices[0];
            let lowest_price = sorted_prices[sorted_prices.len() - 1];
            let price_spread = ((highest_price.price - lowest_price.price) / lowest_price.price) * 100.0;
            
            info!("   💰 Диапазон цен: {:.6} - {:.6} (спред: {:.2}%)", 
                lowest_price.price, highest_price.price, price_spread);

            // Ищем Two-Hop арбитраж (A -> B -> A)
            if let Some(route) = self.find_two_hop_arbitrage(token_mint, &sorted_prices.iter().map(|&&p| p).collect::<Vec<_>>()).await {
                routes.push(route);
                profitable_pairs_found += 1;
                info!("   ✅ Two-Hop арбитраж найден!");
            } else {
                debug!("   ❌ Two-Hop арбитраж не найден");
            }

            // Ищем Triangle арбитраж (A -> B -> C -> A)
            if let Some(route) = self.find_triangle_arbitrage(token_mint, &sorted_prices.iter().map(|&&p| p).collect::<Vec<_>>(), &token_prices).await {
                routes.push(route);
                profitable_pairs_found += 1;
                info!("   ✅ Triangle арбитраж найден!");
            } else {
                debug!("   ❌ Triangle арбитраж не найден");
            }

            // Ищем Multi-Hop арбитраж (A -> B -> C -> D -> A)
            if let Some(route) = self.find_multi_hop_arbitrage(token_mint, &sorted_prices.iter().map(|&&p| p).collect::<Vec<_>>(), &token_prices).await {
                routes.push(route);
                profitable_pairs_found += 1;
                info!("   ✅ Multi-Hop арбитраж найден!");
            } else {
                debug!("   ❌ Multi-Hop арбитраж не найден");
            }

            total_pairs_checked += 1;
        }

        // Фильтруем по прибыльности, риску и уверенности
        let initial_count = routes.len();
        routes.retain(|route| {
            let profitable = route.profit_percentage >= self.min_profit_threshold;
            let risk_acceptable = route.risk_score <= self.risk_tolerance;
            let confident = route.confidence_score >= self.min_confidence_score;
            
            if !profitable {
                debug!("   🚫 Маршрут {} отфильтрован: недостаточная прибыль {:.2}% < {:.2}%", 
                    route.id, route.profit_percentage * 100.0, self.min_profit_threshold * 100.0);
            }
            if !risk_acceptable {
                debug!("   🚫 Маршрут {} отфильтрован: высокий риск {:.2} > {:.2}", 
                    route.id, route.risk_score, self.risk_tolerance);
            }
            if !confident {
                debug!("   🚫 Маршрут {} отфильтрован: низкая уверенность {:.2} < {:.2}", 
                    route.id, route.confidence_score, self.min_confidence_score);
            }
            
            profitable && risk_acceptable && confident
        });

        // Показываем краткую статистику только если есть что показать
        if total_pairs_checked > 0 {
            if profitable_pairs_found > 0 {
                info!("📊 Найдено {} прибыльных пар из {} проверенных токенов", profitable_pairs_found, total_pairs_checked);
            } else {
                info!("📊 Проверено {} токенов, прибыльных пар не найдено", total_pairs_checked);
            }
        }

        info!("✅ Найдено {} арбитражных маршрутов", routes.len());
        routes
    }

    /// Поиск Two-Hop арбитража (A -> B -> A) с улучшенными расчетами
    async fn find_two_hop_arbitrage(
        &self,
        token_mint: &str,
        prices: &[&PriceData],
    ) -> Option<ArbitrageRoute> {
        if prices.len() < 2 {
            return None;
        }

        // Ищем арбитраж между разными токенами, а не внутри одного
        // Например: SOL -> USDC -> SOL через разные DEX
        
        // Получаем все доступные токены для арбитража
        let available_tokens = self.get_available_trading_pairs(token_mint).await;
        
        for (other_token, other_prices) in available_tokens {
            if other_prices.len() < 2 {
                continue;
            }
            
            // Ищем лучшие цены для обмена token_mint <-> other_token
            let best_buy_other = other_prices.iter()
                .min_by(|a, b| a.price.partial_cmp(&b.price).unwrap_or(std::cmp::Ordering::Equal))?;
            let best_sell_other = other_prices.iter()
                .max_by(|a, b| a.price.partial_cmp(&b.price).unwrap_or(std::cmp::Ordering::Equal))?;
                
            // Проверяем, что это разные DEX
            if best_buy_other.dex_type == best_sell_other.dex_type {
                continue;
            }
            
            // Рассчитываем прибыльность арбитража
            let base_amount = 1000000000u64; // 1 SOL в lamports
            
            // SOL -> USDC (покупаем USDC за SOL)
            let usdc_amount = (base_amount as f64 / best_buy_other.price) as u64;
            
            // USDC -> SOL (продаем USDC за SOL)
            let sol_final = (usdc_amount as f64 * best_sell_other.price) as u64;
            
            let profit = if sol_final > base_amount {
                sol_final - base_amount
            } else {
                continue; // Убыточно
            };
            
            let profit_percentage = profit as f64 / base_amount as f64;
            
            if profit_percentage < self.min_profit_threshold {
                continue;
            }
            
            // Получаем метаданные токенов
            let sol_metadata = match self.token_metadata.get_token_metadata(token_mint).await {
                Ok(m) => m,
                Err(_) => continue,
            };
            
            let usdc_metadata = match self.token_metadata.get_token_metadata(&other_token).await {
                Ok(m) => m,
                Err(_) => continue,
            };
            
            // Рассчитываем реальные суммы с учетом проскальзывания
            let amount_in = Amount::new(base_amount, 9);
            let slippage_estimate = self.calculate_slippage_estimate(amount_in.value, best_buy_other.liquidity.value);
            let effective_price = best_buy_other.price * (1.0 - slippage_estimate);
            let expected_amount_out = Amount::new((base_amount as f64 * effective_price) as u64, 9);
            
            // Рассчитываем комиссии
            let fee_estimate = self.calculate_fee_estimate(&best_buy_other.dex_type, amount_in.value);
            let gas_estimate = self.calculate_gas_estimate(&best_buy_other.dex_type);
            
            // Создаем маршрут
            let route = ArbitrageRoute {
                id: format!("two_hop_{}_{}_{}_{}", 
                    sol_metadata.symbol, 
                    usdc_metadata.symbol,
                    best_buy_other.dex_type.as_str(), 
                    best_sell_other.dex_type.as_str()),
                steps: vec![
                    ArbitrageStep {
                        dex_type: best_buy_other.dex_type.clone(),
                        pool_id: best_buy_other.pool_id.clone(),
                        token_in: Token {
                            mint: solana_sdk::pubkey::Pubkey::try_from(token_mint.as_bytes()).unwrap_or_default(),
                            symbol: sol_metadata.symbol.clone(),
                            decimals: sol_metadata.decimals,
                            name: Some(sol_metadata.name.clone()),
                        },
                        token_out: Token {
                            mint: solana_sdk::pubkey::Pubkey::try_from(other_token.as_bytes()).unwrap_or_default(),
                            symbol: usdc_metadata.symbol.clone(),
                            decimals: usdc_metadata.decimals,
                            name: Some(usdc_metadata.name.clone()),
                        },
                        amount_in: amount_in.clone(),
                        expected_amount_out: expected_amount_out.clone(),
                        price_impact: slippage_estimate,
                        fee: fee_estimate.clone(),
                        slippage_estimate,
                        gas_estimate: gas_estimate.clone(),
                    },
                    ArbitrageStep {
                        dex_type: best_sell_other.dex_type.clone(),
                        pool_id: best_sell_other.pool_id.clone(),
                        token_in: Token {
                            mint: solana_sdk::pubkey::Pubkey::try_from(other_token.as_bytes()).unwrap_or_default(),
                            symbol: usdc_metadata.symbol.clone(),
                            decimals: usdc_metadata.decimals,
                            name: Some(usdc_metadata.name.clone()),
                        },
                        token_out: Token {
                            mint: solana_sdk::pubkey::Pubkey::try_from(token_mint.as_bytes()).unwrap_or_default(),
                            symbol: sol_metadata.symbol.clone(),
                            decimals: sol_metadata.decimals,
                            name: Some(sol_metadata.name.clone()),
                        },
                        amount_in: expected_amount_out.clone(),
                        expected_amount_out: amount_in.clone(),
                        price_impact: slippage_estimate,
                        fee: fee_estimate.clone(),
                        slippage_estimate,
                        gas_estimate: gas_estimate.clone(),
                    },
                ],
                expected_profit: profit as f64 / 1_000_000_000.0, // Конвертируем в SOL
                total_cost: Amount::new(fee_estimate.value + gas_estimate.value, 9),
                profit_percentage,
                risk_score: self.calculate_risk_score(profit_percentage, slippage_estimate),
                confidence_score: self.calculate_confidence_score(profit_percentage, slippage_estimate),
                execution_time_estimate: std::time::Duration::from_millis(500),
                timestamp: std::time::Instant::now(),
            };
            
            return Some(route);
        }
        
        None
    }
    
    /// Получить доступные торговые пары для токена
    async fn get_available_trading_pairs(&self, base_token: &str) -> HashMap<String, Vec<PriceData>> {
        // В реальной системе здесь нужно получить все доступные пары
        // Пока используем демо-данные для SOL/USDC
        let mut pairs = HashMap::new();
        
        if base_token == "111111111111111111111111111111111111111111111111111111111111111111" {
            // SOL -> USDC
            let usdc_mint = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"; // USDC mint
            let demo_prices = vec![
                PriceData {
                    token_mint: usdc_mint.to_string(),
                    dex_type: DexType::OrcaWhirlpool,
                    pool_id: "orca_sol_usdc_pool".to_string(),
                    price: 0.00098, // 1 SOL = 0.00098 USDC (примерно 98 USDC за 1 SOL)
                    liquidity: Amount::new(1000000000000, 9), // 1000 SOL
                    volume_24h: Some(1000000.0),
                    price_change_24h: Some(0.01),
                    timestamp: std::time::Instant::now(),
                },
                PriceData {
                    token_mint: usdc_mint.to_string(),
                    dex_type: DexType::RaydiumAMM,
                    pool_id: "raydium_sol_usdc_pool".to_string(),
                    price: 0.00100, // 1 SOL = 0.00100 USDC (100 USDC за 1 SOL)
                    liquidity: Amount::new(800000000000, 9), // 800 SOL
                    volume_24h: Some(800000.0),
                    price_change_24h: Some(0.02),
                    timestamp: std::time::Instant::now(),
                },
            ];
            pairs.insert(usdc_mint.to_string(), demo_prices);
        }
        
        pairs
    }

    /// Поиск Triangle арбитража (A -> B -> C -> A)
    async fn find_triangle_arbitrage(
        &self,
        token_mint: &str,
        prices: &[&PriceData],
        all_token_prices: &HashMap<String, Vec<&PriceData>>,
    ) -> Option<ArbitrageRoute> {
        // Ищем токены, которые торгуются с базовым токеном
        let mut triangle_routes = Vec::new();
        
        for (other_token, other_prices) in all_token_prices {
            if other_token == token_mint {
                continue;
            }

            // Ищем маршрут: A -> B -> C -> A
            if let Some(route) = self.find_triangle_route(token_mint, other_token, prices, other_prices).await {
                triangle_routes.push(route);
            }
        }

        // Возвращаем самый прибыльный маршрут
        triangle_routes.into_iter()
            .max_by(|a, b| a.profit_percentage.partial_cmp(&b.profit_percentage).unwrap_or(std::cmp::Ordering::Equal))
    }

    /// Поиск конкретного треугольного маршрута
    async fn find_triangle_route(
        &self,
        token_a: &str,
        token_b: &str,
        prices_a: &[&PriceData],
        prices_b: &[&PriceData],
    ) -> Option<ArbitrageRoute> {
        if prices_a.len() < 2 || prices_b.len() < 2 {
            return None;
        }

        // Находим лучшие цены для каждого направления
        let best_buy_a = prices_a.iter().min_by(|a, b| a.price.partial_cmp(&b.price).unwrap_or(std::cmp::Ordering::Equal))?;
        let best_sell_a = prices_a.iter().max_by(|a, b| a.price.partial_cmp(&b.price).unwrap_or(std::cmp::Ordering::Equal))?;
        let best_buy_b = prices_b.iter().min_by(|a, b| a.price.partial_cmp(&b.price).unwrap_or(std::cmp::Ordering::Equal))?;
        let best_sell_b = prices_b.iter().max_by(|a, b| a.price.partial_cmp(&b.price).unwrap_or(std::cmp::Ordering::Equal))?;

        // Проверяем, что это разные DEX
        if best_buy_a.dex_type == best_sell_a.dex_type || 
           best_buy_b.dex_type == best_sell_b.dex_type {
            return None;
        }

        // Рассчитываем прибыльность треугольника
        let base_amount = 1000000000u64; // 1 SOL
        
        // A -> B (покупаем B за A)
        let amount_b = (base_amount as f64 / best_buy_b.price) as u64;
        
        // B -> A (продаем B за A)
        let amount_a_final = (amount_b as f64 * best_sell_b.price) as u64;
        
        let profit = if amount_a_final > base_amount {
            amount_a_final - base_amount
        } else {
            return None; // Убыточно
        };

        let profit_percentage = profit as f64 / base_amount as f64;
        
        if profit_percentage < self.min_profit_threshold {
            return None;
        }

        // Получаем метаданные токенов
        let metadata_a = match self.token_metadata.get_token_metadata(token_a).await {
            Ok(m) => m,
            Err(_) => return None,
        };
        
        let metadata_b = match self.token_metadata.get_token_metadata(token_b).await {
            Ok(m) => m,
            Err(_) => return None,
        };

        // Создаем треугольный маршрут
        let route = ArbitrageRoute {
            id: format!("triangle_{}_{}_{}", metadata_a.symbol, metadata_b.symbol, metadata_a.symbol),
            steps: vec![
                // A -> B
                ArbitrageStep {
                    dex_type: best_buy_b.dex_type.clone(),
                    pool_id: best_buy_b.pool_id.clone(),
                    token_in: Token {
                        mint: solana_sdk::pubkey::Pubkey::try_from(token_a.as_bytes()).unwrap_or_default(),
                        symbol: metadata_a.symbol.clone(),
                        decimals: metadata_a.decimals,
                        name: Some(metadata_a.name.clone()),
                    },
                    token_out: Token {
                        mint: solana_sdk::pubkey::Pubkey::try_from(token_b.as_bytes()).unwrap_or_default(),
                        symbol: metadata_b.symbol.clone(),
                        decimals: metadata_b.decimals,
                        name: Some(metadata_b.name.clone()),
                    },
                    amount_in: Amount::new(base_amount, 9),
                    expected_amount_out: Amount::new(amount_b, 9),
                    price_impact: 0.001,
                    fee: Amount::new(5000000, 9),
                    slippage_estimate: 0.001,
                    gas_estimate: Amount::new(5000000, 9),
                },
                // B -> A
                ArbitrageStep {
                    dex_type: best_sell_b.dex_type.clone(),
                    pool_id: best_sell_b.pool_id.clone(),
                    token_in: Token {
                        mint: solana_sdk::pubkey::Pubkey::try_from(token_b.as_bytes()).unwrap_or_default(),
                        symbol: metadata_b.symbol.clone(),
                        decimals: metadata_b.decimals,
                        name: Some(metadata_b.name.clone()),
                    },
                    token_out: Token {
                        mint: solana_sdk::pubkey::Pubkey::try_from(token_a.as_bytes()).unwrap_or_default(),
                        symbol: metadata_a.symbol.clone(),
                        decimals: metadata_a.decimals,
                        name: Some(metadata_a.name.clone()),
                    },
                    amount_in: Amount::new(amount_b, 9),
                    expected_amount_out: Amount::new(amount_a_final, 9),
                    price_impact: 0.001,
                    fee: Amount::new(5000000, 9),
                    slippage_estimate: 0.001,
                    gas_estimate: Amount::new(5000000, 9),
                },
            ],
            expected_profit: profit as f64,
            profit_percentage,
            total_cost: Amount::new(20000000, 9), // 0.02 SOL
            risk_score: 0.4, // Средний риск для треугольника
            timestamp: Instant::now(),
            confidence_score: 0.8,
            execution_time_estimate: Duration::from_millis(800),
        };

        info!("🎯 Triangle арбитраж: {} -> {} -> {} (прибыль: {:.2}%)", 
            metadata_a.symbol, metadata_b.symbol, metadata_a.symbol, profit_percentage * 100.0);

        Some(route)
    }

    /// Поиск Multi-Hop арбитража (A -> B -> C -> D -> A)
    async fn find_multi_hop_arbitrage(
        &self,
        token_mint: &str,
        prices: &[&PriceData],
        all_token_prices: &HashMap<String, Vec<&PriceData>>,
    ) -> Option<ArbitrageRoute> {
        // TODO: Реализовать поиск многошагового арбитража
        // Это требует построения графа токенов и поиска циклов
        None
    }

    /// Расчет оценки проскальзывания на основе ликвидности
    fn calculate_slippage_estimate(&self, amount_in: u64, pool_liquidity: u64) -> f64 {
        let liquidity_ratio = amount_in as f64 / pool_liquidity as f64;
        
        // Простая модель проскальзывания: квадратичная зависимость от размера сделки
        let slippage = liquidity_ratio * liquidity_ratio * 0.1; // 10% базовый коэффициент
        
        // Ограничиваем максимальное проскальзывание
        slippage.min(self.max_slippage)
    }

    /// Расчет комиссий для DEX
    fn calculate_fee_estimate(&self, dex_type: &DexType, amount: u64) -> Amount {
        let fee_rate = match dex_type {
            DexType::OrcaWhirlpool => 0.0003, // 0.03%
            DexType::RaydiumAMM => 0.0025,    // 0.25%
        };

        Amount::new((amount as f64 * fee_rate) as u64, 9)
    }

    /// Расчет газа для DEX
    fn calculate_gas_estimate(&self, dex_type: &DexType) -> Amount {
        let gas_estimate = match dex_type {
            DexType::OrcaWhirlpool => 5000000,  // 0.005 SOL
            DexType::RaydiumAMM => 6000000,     // 0.006 SOL
        };

        Amount::new(gas_estimate, 9)
    }

    /// Расчет оценки риска для маршрута
    fn calculate_risk_score(&self, profit_percentage: f64, slippage_estimate: f64) -> f64 {
        let mut risk_score: f64 = 0.0;

        // Риск на основе прибыли
        if profit_percentage < 0.001 { // 0.1% минимальная прибыль
            risk_score += 0.5; // Низкая прибыль = высокий риск
        } else if profit_percentage < 0.01 { // 1% прибыль
            risk_score += 0.3; // Средняя прибыль = средний риск
        } else {
            risk_score += 0.1; // Высокая прибыль = низкий риск
        }

        // Риск на основе проскальзывания
        if slippage_estimate > 0.05 { // 5% максимальное проскальзывание
            risk_score += 0.4; // Высокое проскальзывание = высокий риск
        } else if slippage_estimate > 0.01 { // 1% проскальзывание
            risk_score += 0.2; // Среднее проскальзывание = средний риск
        } else {
            risk_score += 0.1; // Низкое проскальзывание = низкий риск
        }

        // Риск на основе ликвидности (упрощенно)
        // Этот параметр не используется в текущей логике Two-Hop, но может быть добавлен
        // если будет реализована проверка ликвидности для каждого шага
        risk_score.min(1.0) // Ограничиваем максимальный риск
    }

    /// Расчет уверенности в расчетах
    fn calculate_confidence_score(&self, profit_percentage: f64, slippage_estimate: f64) -> f64 {
        let mut confidence: f64 = 1.0;

        // Уверенность на основе прибыли
        if profit_percentage < 0.001 { // 0.1% минимальная прибыль
            confidence *= 0.6; // Низкая прибыль снижает уверенность
        } else if profit_percentage < 0.01 { // 1% прибыль
            confidence *= 0.8; // Средняя прибыль = средняя уверенность
        } else {
            confidence *= 0.9; // Высокая прибыль = высокая уверенность
        }

        // Уверенность на основе проскальзывания
        if slippage_estimate > 0.05 { // 5% максимальное проскальзывание
            confidence *= 0.7; // Высокое проскальзывание снижает уверенность
        } else if slippage_estimate > 0.01 { // 1% проскальзывание
            confidence *= 0.9; // Среднее проскальзывание = средняя уверенность
        } else {
            confidence *= 1.0; // Низкое проскальзывание = высокая уверенность
        }

        // Уверенность на основе ликвидности (упрощенно)
        // Этот параметр не используется в текущей логике Two-Hop, но может быть добавлен
        confidence.max(0.1) // Минимальная уверенность 10%
    }

    /// Проверка прибыльности маршрута с улучшенными расчетами
    pub fn calculate_route_profit(&self, route: &ArbitrageRoute) -> ProfitCalculation {
        let gross_profit = route.expected_profit;
        
        // Суммируем все комиссии
        let total_fees: u64 = route.steps.iter().map(|step| step.fee.value).sum();
        let total_gas: u64 = route.steps.iter().map(|step| step.gas_estimate.value).sum();
        
        // Рассчитываем проскальзывание
        let total_slippage_cost = route.steps.iter()
            .map(|step| step.expected_amount_out.value as f64 * step.slippage_estimate)
            .sum::<f64>();

        let net_profit = gross_profit - total_fees as f64 - total_gas as f64 - total_slippage_cost;
        let profit_margin = if gross_profit > 0.0 { net_profit / gross_profit } else { 0.0 };
        let is_profitable = net_profit > 0.0 && profit_margin >= self.min_profit_threshold;
        
        // Рассчитываем ROI
        let total_investment = route.steps[0].amount_in.value as f64;
        let roi_percentage = if total_investment > 0.0 { (net_profit / total_investment) * 100.0 } else { 0.0 };
        
        // Рассчитываем точку безубыточности
        let break_even_amount = if roi_percentage > 0.0 { 
            total_investment * (1.0 + (total_fees as f64 + total_gas as f64 + total_slippage_cost) / total_investment)
        } else { 
            0.0 
        };

        ProfitCalculation {
            gross_profit,
            net_profit,
            gas_cost: Amount::new(total_gas, 9),
            slippage_cost: total_slippage_cost,
            fee_cost: total_fees as f64,
            profit_margin,
            is_profitable,
            roi_percentage,
            break_even_amount,
        }
    }

    /// Валидация ликвидности маршрута с улучшенными проверками
    pub fn validate_route_liquidity(&self, route: &ArbitrageRoute) -> bool {
        for step in &route.steps {
            // Проверяем, что сумма сделки не превышает ликвидность пула
            if step.amount_in.value > step.expected_amount_out.value {
                // Если вход больше выхода, проверяем ликвидность входа
                if step.amount_in.value < self.min_liquidity.value {
                    return false;
                }
            } else {
                // Если выход больше входа, проверяем ликвидность выхода
                if step.expected_amount_out.value < self.min_liquidity.value {
                    return false;
                }
            }

            // Проверяем, что проскальзывание не превышает допустимое
            if step.slippage_estimate > self.max_slippage {
                return false;
            }
        }
        true
    }

    /// Обновить кэш цен с дополнительной информацией
    pub async fn update_price_cache(&self, price_data: PriceData) {
        let mut cache = self.price_cache.write().await;
        cache.insert(price_data.token_mint.clone(), price_data);
    }

    /// Получить кэш цен
    pub async fn get_price_cache(&self) -> HashMap<String, PriceData> {
        self.price_cache.read().await.clone()
    }

    /// Очистить устаревшие цены
    pub async fn cleanup_stale_prices(&self, max_age: Duration) {
        let mut cache = self.price_cache.write().await;
        let now = Instant::now();
        cache.retain(|_, price| now.duration_since(price.timestamp) <= max_age);
    }

    /// Получить статистику кэша цен
    pub async fn get_cache_stats(&self) -> (usize, Duration) {
        let cache = self.price_cache.read().await;
        let size = cache.len();
        let oldest_timestamp = cache.values()
            .map(|p| p.timestamp)
            .min()
            .unwrap_or(Instant::now());
        let age = Instant::now().duration_since(oldest_timestamp);
        (size, age)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::dex::DexType;

    #[tokio::test]
    async fn test_find_two_hop_arbitrage() {
        let rpc_client = Arc::new(solana_client::rpc_client::RpcClient::new("".to_string()));
        let token_metadata = Arc::new(TokenMetadataService::new(rpc_client));
        let detector = ArbitrageOpportunityDetector::new_default(token_metadata);
        
        let price_data = vec![
            PriceData {
                token_mint: "SOL".to_string(),
                price: 100.0,
                dex_type: DexType::OrcaWhirlpool,
                pool_id: "pool1".to_string(),
                timestamp: Instant::now(),
                liquidity: Amount::new(1000000000, 9),
                volume_24h: Some(1000000000.0),
                price_change_24h: Some(0.01),
            },
            PriceData {
                token_mint: "SOL".to_string(),
                price: 98.0,
                dex_type: DexType::RaydiumAMM,
                pool_id: "pool2".to_string(),
                timestamp: Instant::now(),
                liquidity: Amount::new(1000000000, 9),
                volume_24h: Some(1000000000.0),
                price_change_24h: Some(0.01),
            },
        ];

        let routes = detector.find_arbitrage_routes(&price_data).await;
        assert!(!routes.is_empty());
        
        let route = &routes[0];
        assert_eq!(route.steps.len(), 2);
        assert!(route.profit_percentage > 0.0);
    }
}
