use crate::shared::errors::AppError;
use crate::shared::types::{Amount, Token};
use crate::domain::dex::{DexType, PoolInfo};
use crate::infrastructure::blockchain::TokenMetadataService;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{info, warn, debug, error};

/// Индекс пула в графе
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PoolIndex(pub usize);

/// Индекс токена в графе
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TokenIndex(pub usize);

/// Информация о пуле для арбитража
#[derive(Debug, Clone)]
pub struct PoolQuote {
    pub pool_id: String,
    pub dex_type: DexType,
    pub token_a: String,
    pub token_b: String,
    pub price_a_to_b: f64,
    pub price_b_to_a: f64,
    pub liquidity: Amount,
    pub fee: Amount,
}

impl PoolQuote {
    pub fn get_name(&self) -> String {
        format!("{}_{}", self.dex_type.as_str(), self.pool_id)
    }
    
    /// Получить котировку для обмена amount_in токена in на токен out
    pub fn get_quote_with_amounts_scaled(
        &self,
        amount_in: u64,
        mint_in: &str,
        mint_out: &str,
    ) -> u64 {
        if mint_in == self.token_a && mint_out == self.token_b {
            // A -> B
            (amount_in as f64 * self.price_a_to_b) as u64
        } else if mint_in == self.token_b && mint_out == self.token_a {
            // B -> A
            (amount_in as f64 * self.price_b_to_a) as u64
        } else {
            0 // Неправильная пара токенов
        }
    }
}

/// Граф пулов для поиска арбитража
#[derive(Debug, Clone)]
pub struct PoolGraph(pub HashMap<TokenIndex, HashMap<TokenIndex, Vec<PoolQuote>>>);

impl PoolGraph {
    pub fn new() -> Self {
        Self(HashMap::new())
    }
    
    /// Добавить пул в граф
    pub fn add_pool(&mut self, token_a: usize, token_b: usize, pool: PoolQuote) {
        self.0.entry(TokenIndex(token_a)).or_insert_with(HashMap::new)
            .entry(TokenIndex(token_b)).or_insert_with(Vec::new)
            .push(pool.clone());
            
        // Добавляем обратное направление
        self.0.entry(TokenIndex(token_b)).or_insert_with(HashMap::new)
            .entry(TokenIndex(token_a)).or_insert_with(Vec::new)
            .push(pool);
    }
    
    /// Получить все пулы между двумя токенами
    pub fn get_pools(&self, token_a: usize, token_b: usize) -> Option<&Vec<PoolQuote>> {
        self.0.get(&TokenIndex(token_a))
            .and_then(|edges| edges.get(&TokenIndex(token_b)))
    }
}

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
#[derive(Clone)]
pub struct ArbitrageOpportunityDetector {
    min_profit_threshold: f64,
    min_liquidity: Amount,
    price_cache: Arc<RwLock<HashMap<String, Vec<PriceData>>>>,
    max_route_length: usize,
    risk_tolerance: f64,
    token_metadata: Arc<TokenMetadataService>,
    max_slippage: f64, // Максимальное допустимое проскальзывание
    min_confidence_score: f64, // Минимальная уверенность в расчетах
    
    // Новые поля для графа пулов
    pool_graph: PoolGraph,
    token_mints: Vec<String>,
    token_to_index: HashMap<String, usize>,
}

impl ArbitrageOpportunityDetector {
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
            
            // Новые поля для графа пулов
            pool_graph: PoolGraph::new(),
            token_mints: Vec::new(),
            token_to_index: HashMap::new(),
        }
    }

    /// Поиск арбитражных маршрутов с улучшенными алгоритмами
    pub async fn find_arbitrage_routes(&self, price_data: &[PriceData]) -> Vec<ArbitrageRoute> {
        info!("🔍 Поиск арбитражных маршрутов...");
        
        // Сначала обновляем кэш цен
        for price in price_data {
            self.update_price_cache(price.clone()).await;
        }
        
        // Строим граф пулов из обновленного кэша
        let mut detector = self.clone();
        detector.build_pool_graph().await;
        
        if detector.token_mints.is_empty() {
            info!("⚠️ Граф пулов не построен, пропускаем поиск");
            return Vec::new();
        }
        
        let mut routes = Vec::new();
        let mut sent_arbs = HashSet::new();
        
        // Ищем арбитраж для каждого токена как стартовой точки
        for start_mint_idx in 0..detector.token_mints.len() {
            let init_balance = 1000000000u64; // 1 SOL в lamports
            
            detector.brute_force_search(
                start_mint_idx,
                init_balance,
                init_balance,
                vec![start_mint_idx],
                Vec::new(),
                &mut sent_arbs,
                &mut routes,
            );
        }
        
        info!("📊 Статистика поиска:");
        info!("   Всего токенов в графе: {}", detector.token_mints.len());
        info!("   Найдено маршрутов: {}", routes.len());
        
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
        
        info!("   Маршрутов после фильтрации: {} (было: {})", routes.len(), initial_count);
        info!("   Фильтры: прибыль ≥{:.2}%, риск ≤{:.2}, уверенность ≥{:.2}", 
            self.min_profit_threshold * 100.0, self.risk_tolerance, self.min_confidence_score);
        info!("✅ Найдено {} арбитражных маршрутов", routes.len());
        
        routes
    }
    
    /// Brute force поиск арбитража (по примеру рабочего кода)
    fn brute_force_search(
        &self,
        start_mint_idx: usize,
        init_balance: u64,
        curr_balance: u64,
        path: Vec<usize>,
        pool_path: Vec<PoolQuote>,
        sent_arbs: &mut HashSet<String>,
        routes: &mut Vec<ArbitrageRoute>,
    ) {
        let src_curr = path[path.len() - 1]; // последний токен в пути
        let src_mint = &self.token_mints[src_curr];
        
        // Максимум 3 шага для 2 DEX
        if path.len() >= 4 {
            return;
        }
        
        // Ищем все возможные переходы из текущего токена
        if let Some(edges) = self.pool_graph.0.get(&TokenIndex(src_curr)) {
            for (dst_mint_idx, pools) in edges {
                let dst_mint_idx = dst_mint_idx.0;
                let dst_mint = &self.token_mints[dst_mint_idx];
                
                // Пропускаем, если токен уже в пути (кроме возврата к началу)
                if path.contains(&dst_mint_idx) && dst_mint_idx != start_mint_idx {
                    continue;
                }
                
                // Выбираем лучший пул для перехода
                if let Some(best_pool) = pools.iter().min_by(|a, b| {
                    a.price_a_to_b.partial_cmp(&b.price_a_to_b).unwrap_or(std::cmp::Ordering::Equal)
                }) {
                    let new_balance = best_pool.get_quote_with_amounts_scaled(
                        curr_balance,
                        src_mint,
                        dst_mint,
                    );
                    
                    let mut new_path = path.clone();
                    new_path.push(dst_mint_idx);
                    
                    let mut new_pool_path = pool_path.clone();
                    new_pool_path.push(best_pool.clone());
                    
                    // Проверяем, вернулись ли к начальному токену (арбитраж!)
                    if dst_mint_idx == start_mint_idx && new_path.len() >= 3 {
                        if new_balance > init_balance {
                            // Нашли прибыльный арбитраж!
                            let profit = new_balance - init_balance;
                            let profit_percentage = profit as f64 / init_balance as f64;
                            
                            // Создаем уникальный ключ для арбитража
                            let mint_keys: Vec<String> = new_path.iter().map(|i| i.to_string()).collect();
                            let pool_keys: Vec<String> = new_pool_path.iter().map(|p| p.get_name()).collect();
                            let arb_key = format!("{}_{}", mint_keys.join("->"), pool_keys.join("->"));
                            
                            if !sent_arbs.contains(&arb_key) {
                                sent_arbs.insert(arb_key);
                                
                                info!("🎯 Найден арбитраж: {} -> {} (прибыль: {:.2}%)", 
                                    init_balance, new_balance, profit_percentage * 100.0);
                                
                                // Создаем маршрут арбитража
                                if let Some(route) = self.create_arbitrage_route(
                                    &new_path,
                                    &new_pool_path,
                                    profit_percentage,
                                    init_balance,
                                    new_balance,
                                ) {
                                    routes.push(route);
                                }
                            }
                        }
                    } else {
                        // Продолжаем поиск
                        self.brute_force_search(
                            start_mint_idx,
                            init_balance,
                            new_balance,
                            new_path,
                            new_pool_path,
                            sent_arbs,
                            routes,
                        );
                    }
                }
            }
        }
    }
    
    /// Создать маршрут арбитража из найденного пути
    fn create_arbitrage_route(
        &self,
        path: &[usize],
        pool_path: &[PoolQuote],
        profit_percentage: f64,
        init_balance: u64,
        final_balance: u64,
    ) -> Option<ArbitrageRoute> {
        if path.len() < 3 || pool_path.len() < 2 {
            return None;
        }
        
        let mut steps = Vec::new();
        let mut current_balance = init_balance;
        
        // Создаем шаги арбитража
        for i in 0..pool_path.len() {
            let pool = &pool_path[i];
            let token_in_idx = path[i];
            let token_out_idx = path[i + 1];
            
            let token_in_mint = &self.token_mints[token_in_idx];
            let token_out_mint = &self.token_mints[token_out_idx];
            
            let amount_out = pool.get_quote_with_amounts_scaled(
                current_balance,
                token_in_mint,
                token_out_mint,
            );
            
            let step = ArbitrageStep {
                dex_type: pool.dex_type.clone(),
                pool_id: pool.pool_id.clone(),
                token_in: Token {
                    mint: solana_sdk::pubkey::Pubkey::try_from(token_in_mint.as_bytes()).unwrap_or_default(),
                    symbol: token_in_mint.clone(), // Временно используем mint как symbol
                    decimals: 9,
                    name: Some(token_in_mint.clone()),
                },
                token_out: Token {
                    mint: solana_sdk::pubkey::Pubkey::try_from(token_out_mint.as_bytes()).unwrap_or_default(),
                    symbol: token_out_mint.clone(),
                    decimals: 9,
                    name: Some(token_out_mint.clone()),
                },
                amount_in: Amount::new(current_balance, 9),
                expected_amount_out: Amount::new(amount_out, 9),
                price_impact: 0.001,
                fee: pool.fee.clone(),
                slippage_estimate: 0.001,
                gas_estimate: Amount::new(5000000, 9),
            };
            
            steps.push(step);
            current_balance = amount_out;
        }
        
        // Создаем маршрут
        let route = ArbitrageRoute {
            id: format!("arb_{}_{}_{}", 
                self.token_mints[path[0]], 
                self.token_mints[path[1]], 
                self.token_mints[path[0]]),
            steps,
            expected_profit: (final_balance - init_balance) as f64 / 1_000_000_000.0,
            total_cost: Amount::new(10000000, 9), // Примерная стоимость
            profit_percentage,
            risk_score: self.calculate_risk_score(profit_percentage, 0.001),
            confidence_score: self.calculate_confidence_score(profit_percentage, 0.001),
            execution_time_estimate: std::time::Duration::from_millis(500),
            timestamp: std::time::Instant::now(),
        };
        
        Some(route)
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
        cache.entry(price_data.token_mint.clone()).or_insert_with(Vec::new).push(price_data);
    }

    /// Получить кэш цен
    pub async fn get_price_cache(&self) -> HashMap<String, Vec<PriceData>> {
        self.price_cache.read().await.clone()
    }

    /// Очистить устаревшие цены
    pub async fn cleanup_stale_prices(&self, max_age: Duration) {
        let mut cache = self.price_cache.write().await;
        let now = Instant::now();
        cache.retain(|_, prices| prices.iter().all(|p| now.duration_since(p.timestamp) <= max_age));
    }

    /// Получить статистику кэша цен
    pub async fn get_cache_stats(&self) -> (usize, Duration) {
        let cache = self.price_cache.read().await;
        let size = cache.len();
        let oldest_timestamp = cache.values()
            .flat_map(|prices| prices.iter())
            .map(|p| p.timestamp)
            .min()
            .unwrap_or(Instant::now());
        let age = Instant::now().duration_since(oldest_timestamp);
        (size, age)
    }

    /// Построить граф пулов из кэша цен
    pub async fn build_pool_graph(&mut self) {
        let cache = self.price_cache.read().await;
        let mut graph = PoolGraph::new();
        let mut token_mints = Vec::new();
        let mut token_to_index = HashMap::new();
        
        // Собираем все уникальные токены
        for (token_mint, prices) in cache.iter() {
            if !token_to_index.contains_key(token_mint) {
                token_to_index.insert(token_mint.clone(), token_mints.len());
                token_mints.push(token_mint.clone());
            }
        }
        
        // Строим граф пулов
        for (token_mint, prices) in cache.iter() {
            let token_a_idx = *token_to_index.get(token_mint).unwrap();
            
            // Группируем цены по DEX для поиска арбитража
            let mut dex_prices: HashMap<DexType, Vec<&PriceData>> = HashMap::new();
            for price in prices {
                dex_prices.entry(price.dex_type.clone()).or_insert_with(Vec::new).push(price);
            }
            
            // Ищем другие токены для создания пулов
            for (other_mint, other_prices) in cache.iter() {
                if token_mint == other_mint {
                    continue; // Пропускаем тот же токен
                }
                
                let token_b_idx = *token_to_index.get(other_mint).unwrap();
                
                // Создаем пул между токенами
                for (dex_type, dex_price_list) in dex_prices.iter() {
                    if let Some(price_data) = dex_price_list.first() {
                        let pool_quote = PoolQuote {
                            pool_id: price_data.pool_id.clone(),
                            dex_type: dex_type.clone(),
                            token_a: token_mint.clone(),
                            token_b: other_mint.clone(),
                            price_a_to_b: price_data.price,
                            price_b_to_a: 1.0 / price_data.price, // Обратная цена
                            liquidity: price_data.liquidity.clone(),
                            fee: Amount::new(5000000, 9), // Примерная комиссия
                        };
                        
                        graph.add_pool(token_a_idx, token_b_idx, pool_quote);
                    }
                }
            }
        }
        
        // Обновляем поля
        self.pool_graph = graph;
        self.token_mints = token_mints;
        self.token_to_index = token_to_index;
        
        info!("🏗️ Построен граф пулов: {} токенов, {} пулов", 
            self.token_mints.len(), 
            self.count_total_pools());
    }
    
    /// Подсчитать общее количество пулов в графе
    fn count_total_pools(&self) -> usize {
        let mut total = 0;
        for edges in self.pool_graph.0.values() {
            for pools in edges.values() {
                total += pools.len();
            }
        }
        total
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
        let mut detector = ArbitrageOpportunityDetector::new_default(token_metadata);
        
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

        detector.update_price_cache(price_data[0].clone()).await;
        detector.update_price_cache(price_data[1].clone()).await;
        detector.build_pool_graph().await;

        let routes = detector.find_arbitrage_routes(&price_data).await;
        assert!(!routes.is_empty());
        
        let route = &routes[0];
        assert_eq!(route.steps.len(), 2);
        assert!(route.profit_percentage > 0.0);
    }
}
