//! Binance API Connector
//!
//! High-performance Rust client for Binance cryptocurrency exchange.
//! Supports REST API and WebSocket streaming for real-time data.

pub mod client;
pub mod config;
pub mod endpoints;
pub mod error;
pub mod models;
pub mod rate_limiter;
pub mod websocket;

// Re-export main types
pub use client::BinanceClient;
pub use config::BinanceConfig;
pub use error::{Error, Result};
pub use models::{Interval, Kline, OrderBook, Symbol, Ticker, Trade};
pub use websocket::BinanceWebSocket;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_library_exports() {
        // Ensure main types are accessible
        let _ = std::any::type_name::<BinanceClient>();
        let _ = std::any::type_name::<BinanceConfig>();
        let _ = std::any::type_name::<Error>();
    }
}
