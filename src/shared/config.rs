use std::fs;
use toml;
use crate::shared::types::BotConfig;
use crate::shared::errors::AppError;

/// Загрузчик конфигурации
pub struct ConfigLoader;

impl ConfigLoader {
    /// Загрузить конфигурацию из файла Config.toml
    pub fn load_config() -> Result<BotConfig, AppError> {
        let config_content = fs::read_to_string("Config.toml")
            .map_err(|e| AppError::ConfigError(format!("Failed to read config file: {}", e)))?;
        
        let config: BotConfig = toml::from_str(&config_content)
            .map_err(|e| AppError::ConfigError(format!("Failed to parse config file: {}", e)))?;
        
        Ok(config)
    }
}
