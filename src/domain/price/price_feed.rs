//! Price feed implementations

use crate::shared::types::{Token, Price};
use super::PriceData;

/// Price feed interface
pub trait PriceFeed {
    fn get_price(&self, token: &Token) -> Option<PriceData>;
    fn subscribe(&self, token: &Token);
    fn unsubscribe(&self, token: &Token);
}

/// Generic price feed implementation
pub struct GenericPriceFeed {
    source: String,
}

impl GenericPriceFeed {
    pub fn new(source: String) -> Self {
        Self { source }
    }
}

impl PriceFeed for GenericPriceFeed {
    fn get_price(&self, _token: &Token) -> Option<PriceData> {
        // TODO: Implement price fetching
        None
    }

    fn subscribe(&self, _token: &Token) {
        // TODO: Implement subscription
    }

    fn unsubscribe(&self, _token: &Token) {
        // TODO: Implement unsubscription
    }
}
