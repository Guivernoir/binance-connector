//! Binance API endpoint definitions

/// API endpoint paths
pub struct Endpoints;

impl Endpoints {
    /// Get ticker price
    /// GET /api/v3/ticker/price
    pub fn ticker_price() -> &'static str {
        "/api/v3/ticker/price"
    }

    /// Get 24h ticker statistics
    /// GET /api/v3/ticker/24hr
    pub fn ticker_24h() -> &'static str {
        "/api/v3/ticker/24hr"
    }

    /// Get klines (candlestick data)
    /// GET /api/v3/klines
    pub fn klines() -> &'static str {
        "/api/v3/klines"
    }

    /// Get order book depth
    /// GET /api/v3/depth
    pub fn depth() -> &'static str {
        "/api/v3/depth"
    }

    /// Get recent trades
    /// GET /api/v3/trades
    pub fn trades() -> &'static str {
        "/api/v3/trades"
    }

    /// Get exchange info
    /// GET /api/v3/exchangeInfo
    pub fn exchange_info() -> &'static str {
        "/api/v3/exchangeInfo"
    }

    /// Server time
    /// GET /api/v3/time
    pub fn time() -> &'static str {
        "/api/v3/time"
    }

    /// Ping
    /// GET /api/v3/ping
    pub fn ping() -> &'static str {
        "/api/v3/ping"
    }
}

/// WebSocket streams
pub struct WebSocketStreams;

impl WebSocketStreams {
    /// Individual symbol ticker stream
    /// wss://stream.binance.com:9443/ws/<symbol>@ticker
    pub fn ticker(symbol: &str) -> String {
        format!("{}@ticker", symbol.to_lowercase())
    }

    /// Individual symbol kline/candlestick stream
    /// wss://stream.binance.com:9443/ws/<symbol>@kline_<interval>
    pub fn kline(symbol: &str, interval: &str) -> String {
        format!("{}@kline_{}", symbol.to_lowercase(), interval)
    }

    /// Individual symbol trade stream
    /// wss://stream.binance.com:9443/ws/<symbol>@trade
    pub fn trade(symbol: &str) -> String {
        format!("{}@trade", symbol.to_lowercase())
    }

    /// Individual symbol mini ticker stream
    /// wss://stream.binance.com:9443/ws/<symbol>@miniTicker
    pub fn mini_ticker(symbol: &str) -> String {
        format!("{}@miniTicker", symbol.to_lowercase())
    }

    /// Partial book depth stream
    /// wss://stream.binance.com:9443/ws/<symbol>@depth
    pub fn depth(symbol: &str) -> String {
        format!("{}@depth", symbol.to_lowercase())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_endpoint_paths() {
        assert_eq!(Endpoints::ticker_price(), "/api/v3/ticker/price");
        assert_eq!(Endpoints::klines(), "/api/v3/klines");
    }

    #[test]
    fn test_websocket_streams() {
        assert_eq!(WebSocketStreams::ticker("BTCUSDT"), "btcusdt@ticker");
        assert_eq!(WebSocketStreams::kline("ETHUSDT", "1m"), "ethusdt@kline_1m");
        assert_eq!(WebSocketStreams::trade("BTCUSDT"), "btcusdt@trade");
    }
}
