//! Configuration for Binance connector

use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BinanceConfig {
    /// API key (optional - not needed for market data)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,

    /// Secret key (optional - not needed for market data)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub secret_key: Option<String>,

    /// Use testnet (true) or mainnet (false)
    pub testnet: bool,

    /// Base URL (auto-set based on testnet flag)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_url: Option<String>,

    /// WebSocket URL (auto-set based on testnet flag)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ws_url: Option<String>,

    /// Request timeout in seconds
    #[serde(default = "default_timeout")]
    pub timeout_seconds: u64,

    /// Maximum requests per minute
    #[serde(default = "default_rate_limit")]
    pub requests_per_minute: u32,

    /// Enable automatic retries
    #[serde(default = "default_true")]
    pub enable_retries: bool,

    /// Maximum retry attempts
    #[serde(default = "default_max_retries")]
    pub max_retries: u32,
}

fn default_timeout() -> u64 {
    10
}
fn default_rate_limit() -> u32 {
    1200
} // Binance limit
fn default_true() -> bool {
    true
}
fn default_max_retries() -> u32 {
    3
}

impl BinanceConfig {
    /// Create new configuration (no auth needed for market data)
    pub fn new(testnet: bool) -> Self {
        Self {
            api_key: None,
            secret_key: None,
            testnet,
            base_url: None,
            ws_url: None,
            timeout_seconds: default_timeout(),
            requests_per_minute: default_rate_limit(),
            enable_retries: default_true(),
            max_retries: default_max_retries(),
        }
    }

    /// Create config with API credentials (for trading)
    pub fn with_auth(api_key: String, secret_key: String, testnet: bool) -> Self {
        Self {
            api_key: Some(api_key),
            secret_key: Some(secret_key),
            testnet,
            base_url: None,
            ws_url: None,
            timeout_seconds: default_timeout(),
            requests_per_minute: default_rate_limit(),
            enable_retries: default_true(),
            max_retries: default_max_retries(),
        }
    }

    /// Load configuration from environment variables (optional auth)
    ///
    /// Expected env vars:
    /// - BINANCE_API_KEY (optional)
    /// - BINANCE_SECRET_KEY (optional)
    /// - BINANCE_TESTNET (optional, default: false)
    pub fn from_env() -> crate::Result<Self> {
        let api_key = std::env::var("BINANCE_API_KEY").ok();
        let secret_key = std::env::var("BINANCE_SECRET_KEY").ok();

        let testnet = std::env::var("BINANCE_TESTNET")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(false);

        let timeout_seconds = std::env::var("BINANCE_TIMEOUT_SECONDS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(default_timeout());

        let requests_per_minute = std::env::var("BINANCE_REQUESTS_PER_MINUTE")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(default_rate_limit());

        Ok(Self {
            api_key,
            secret_key,
            testnet,
            base_url: None,
            ws_url: None,
            timeout_seconds,
            requests_per_minute,
            enable_retries: default_true(),
            max_retries: default_max_retries(),
        })
    }

    /// Get REST API base URL
    pub fn get_base_url(&self) -> String {
        self.base_url.clone().unwrap_or_else(|| {
            if self.testnet {
                "https://testnet.binance.vision".to_string()
            } else {
                "https://api.binance.com".to_string()
            }
        })
    }

    /// Get WebSocket URL
    pub fn get_ws_url(&self) -> String {
        self.ws_url.clone().unwrap_or_else(|| {
            if self.testnet {
                "wss://testnet.binance.vision/ws".to_string()
            } else {
                "wss://stream.binance.com:9443/ws".to_string()
            }
        })
    }

    /// Get timeout as Duration
    pub fn timeout(&self) -> Duration {
        Duration::from_secs(self.timeout_seconds)
    }

    /// Check if authenticated
    pub fn is_authenticated(&self) -> bool {
        self.api_key.is_some() && self.secret_key.is_some()
    }

    /// Validate configuration
    pub fn validate(&self) -> crate::Result<()> {
        if self.timeout_seconds == 0 {
            return Err(crate::Error::ConfigError(
                "Timeout must be greater than 0".to_string(),
            ));
        }

        if self.requests_per_minute == 0 {
            return Err(crate::Error::ConfigError(
                "Requests per minute must be greater than 0".to_string(),
            ));
        }

        Ok(())
    }
}

impl Default for BinanceConfig {
    fn default() -> Self {
        Self::new(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_urls() {
        let config_mainnet = BinanceConfig::new(false);
        assert!(config_mainnet.get_base_url().contains("api.binance.com"));
        assert!(config_mainnet.get_ws_url().contains("stream.binance.com"));

        let config_testnet = BinanceConfig::new(true);
        assert!(config_testnet.get_base_url().contains("testnet"));
        assert!(config_testnet.get_ws_url().contains("testnet"));
    }

    #[test]
    fn test_config_auth() {
        let config_noauth = BinanceConfig::new(false);
        assert!(!config_noauth.is_authenticated());

        let config_auth =
            BinanceConfig::with_auth("test_key".to_string(), "test_secret".to_string(), false);
        assert!(config_auth.is_authenticated());
    }

    #[test]
    fn test_config_validation() {
        let mut config = BinanceConfig::default();
        assert!(config.validate().is_ok());

        config.timeout_seconds = 0;
        assert!(config.validate().is_err());
    }
}
