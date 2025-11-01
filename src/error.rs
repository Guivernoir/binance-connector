//! Error types for Binance connector

use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("HTTP request failed: {0}")]
    HttpError(#[from] reqwest::Error),

    #[error("Binance API error {code}: {msg}")]
    ApiError { code: i32, msg: String },

    #[error("Rate limit exceeded, retry after {retry_after_seconds}s")]
    RateLimitExceeded { retry_after_seconds: u64 },

    #[error("Invalid symbol: {0}")]
    InvalidSymbol(String),

    #[error("Invalid interval: {0}")]
    InvalidInterval(String),

    #[error("Network timeout after {0}s")]
    Timeout(u64),

    #[error("Deserialization error: {0}")]
    DeserializationError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("WebSocket error: {0}")]
    WebSocketError(String),

    #[error("WebSocket connection closed")]
    WebSocketClosed,

    #[error("Invalid date range: start={start}, end={end}")]
    InvalidDateRange { start: String, end: String },
}

impl Error {
    /// Check if error is retryable
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            Error::HttpError(_)
                | Error::Timeout(_)
                | Error::RateLimitExceeded { .. }
                | Error::WebSocketClosed
        )
    }

    /// Check if error is related to rate limiting
    pub fn is_rate_limit(&self) -> bool {
        matches!(self, Error::RateLimitExceeded { .. })
    }
}
