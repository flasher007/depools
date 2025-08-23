//! Price monitoring and updates

use crate::shared::types::{Token, Price};
use super::{PriceData, PriceChangeEvent, PriceMonitorConfig};

/// Monitors price changes across multiple sources
pub struct PriceMonitor {
    config: PriceMonitorConfig,
    active: bool,
}

impl PriceMonitor {
    pub fn new(config: PriceMonitorConfig) -> Self {
        Self {
            config,
            active: false,
        }
    }

    pub fn start(&mut self) {
        self.active = true;
    }

    pub fn stop(&mut self) {
        self.active = false;
    }

    pub fn is_active(&self) -> bool {
        self.active
    }
}
