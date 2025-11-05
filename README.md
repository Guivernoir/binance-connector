# Binance Connector for Rust

[![Rust](https://img.shields.io/badge/rust-1.91%2B-orange.svg)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

High-performance Rust client for the Binance cryptocurrency exchange API. Built for speed, reliability, and ease of use.

## Features

- ✅ **Full Binance Spot API support**

  - Real-time price tickers
  - Historical candlestick (kline) data
  - Order book depth
  - Recent trades
  - Exchange information

- 🚀 **Production-grade reliability**

  - Automatic rate limiting (1200 req/min)
  - Exponential backoff retry logic
  - Comprehensive error handling
  - Request timeout management

- 🔒 **Type-safe & well-tested**

  - Strongly typed API responses
  - 90%+ test coverage
  - Integration tests with real API
  - Mock server tests for CI/CD

- ⚡ **Performance optimized**

  - Async/await using Tokio
  - Efficient connection pooling
  - Minimal allocations
  - Benchmarked operations

- 🆓 **No API key required**
  - Market data is completely free
  - No authentication needed for prices/candles
  - Optional auth for trading (future feature)

## Quick Start

### 1. No Setup Required!

Unlike most exchange APIs, Binance allows **free access to all market data** without any API keys or registration. Just install and start fetching data!

### 2. Install

Add to your `Cargo.toml`:

```toml
[dependencies]
binance-connector = { path = "path/to/binance-connector" }
tokio = { version = "1", features = ["full"] }
```

### 3. Run Example

```bash
cargo run --example fetch_example
```

Expected output:

```
🚀 Binance Connector Example

✅ Configuration:
   Network: Mainnet
   REST URL: https://api.binance.com

🏥 Health Check:
   ✅ Connected to Binance

💰 Current Price (BTC/USDT):
   Price: $43,245.67
   ...

📊 24-Hour Statistics (BTC/USDT):
   Price: $43,245.67
   Change: $1,234.56 (2.94%)
   High: $44,000.00
   Low: $42,000.00
   ...
```

## Usage Examples

### Get Current Price

```rust
use binance_connector::{BinanceClient, BinanceConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create client (no API key needed!)
    let config = BinanceConfig::new(false); // false = mainnet
    let client = BinanceClient::new(config)?;

    // Fetch BTC/USDT price
    let ticker = client.get_ticker_price("BTCUSDT").await?;
    println!("BTC/USDT: ${}", ticker.price);

    Ok(())
}
```

### Get 24-Hour Statistics

```rust
let ticker = client.get_ticker_24h("ETHUSDT").await?;

println!("ETH/USDT:");
println!("  Price: ${}", ticker.last_price);
println!("  24h Change: {:.2}%", ticker.price_change_percent);
println!("  24h High: ${}", ticker.high_price);
println!("  24h Low: ${}", ticker.low_price);
println!("  24h Volume: {} ETH", ticker.volume);
println!("  Spread: ${:.2}", ticker.spread());
```

### Get Historical Candles

```rust
use binance_connector::Interval;

// Get last 100 5-minute candles
let klines = client.get_klines("BTCUSDT", Interval::Minutes5, 100).await?;

for kline in klines.iter().take(5) {
    println!("{}: O=${:.2} H=${:.2} L=${:.2} C=${:.2}",
        kline.open_time.format("%Y-%m-%d %H:%M"),
        kline.open,
        kline.high,
        kline.low,
        kline.close
    );
}
```

### Get Order Book Depth

```rust
// Get top 20 bids and asks
let order_book = client.get_depth("BTCUSDT", 20).await?;

println!("Top 5 Bids:");
for bid in order_book.bids.iter().take(5) {
    println!("  ${:.2} × {:.6} BTC", bid.price, bid.quantity);
}

println!("Top 5 Asks:");
for ask in order_book.asks.iter().take(5) {
    println!("  ${:.2} × {:.6} BTC", ask.price, ask.quantity);
}

let best_bid = order_book.bids[0].price;
let best_ask = order_book.asks[0].price;
println!("Spread: ${:.2}", best_ask - best_bid);
```

### Get Recent Trades

```rust
let trades = client.get_recent_trades("BTCUSDT", 10).await?;

for trade in trades {
    let side = if trade.is_buyer_maker { "SELL" } else { "BUY" };
    println!("{} ${:.2} × {:.6} at {}",
        side,
        trade.price,
        trade.quantity,
        trade.time.format("%H:%M:%S")
    );
}
```

### Get All Available Symbols

```rust
let symbols = client.get_exchange_info().await?;

for symbol in symbols.iter().filter(|s| s.quote_asset == "USDT").take(10) {
    println!("{} ({}/{})",
        symbol.symbol,
        symbol.base_asset,
        symbol.quote_asset
    );
}

println!("Total symbols: {}", symbols.len());
```

## Available Intervals

```rust
Interval::Seconds1   // 1 second
Interval::Minutes1   // 1 minute
Interval::Minutes3   // 3 minutes
Interval::Minutes5   // 5 minutes
Interval::Minutes15  // 15 minutes
Interval::Minutes30  // 30 minutes
Interval::Hours1     // 1 hour
Interval::Hours2     // 2 hours
Interval::Hours4     // 4 hours
Interval::Hours6     // 6 hours
Interval::Hours8     // 8 hours
Interval::Hours12    // 12 hours
Interval::Days1      // 1 day
Interval::Days3      // 3 days
Interval::Weeks1     // 1 week
Interval::Months1    // 1 month
```

## Popular Trading Pairs

### Stablecoins (USDT pairs)

- `BTCUSDT` - Bitcoin / Tether
- `ETHUSDT` - Ethereum / Tether
- `BNBUSDT` - Binance Coin / Tether
- `ADAUSDT` - Cardano / Tether
- `SOLUSDT` - Solana / Tether
- `XRPUSDT` - Ripple / Tether
- `DOTUSDT` - Polkadot / Tether
- `DOGEUSDT` - Dogecoin / Tether
- `MATICUSDT` - Polygon / Tether
- `AVAXUSDT` - Avalanche / Tether

### BTC pairs

- `ETHBTC` - Ethereum / Bitcoin
- `BNBBTC` - Binance Coin / Bitcoin
- `ADABTC` - Cardano / Bitcoin
- `SOLBTC` - Solana / Bitcoin

### BUSD pairs (Binance USD)

- `BTCBUSD` - Bitcoin / BUSD
- `ETHBUSD` - Ethereum / BUSD
- `BNBBUSD` - Binance Coin / BUSD

**Get full list**: Run `client.get_exchange_info().await` to see all 600+ trading pairs.

## Error Handling

```rust
use binance_connector::Error;

match client.get_ticker_price("BTCUSDT").await {
    Ok(ticker) => println!("Price: ${}", ticker.price),
    Err(Error::RateLimitExceeded { retry_after_seconds }) => {
        eprintln!("Rate limited, retry after {} seconds", retry_after_seconds);
    }
    Err(Error::InvalidSymbol(symbol)) => {
        eprintln!("Invalid symbol: {}", symbol);
    }
    Err(Error::Timeout(secs)) => {
        eprintln!("Request timed out after {} seconds", secs);
    }
    Err(Error::ApiError { code, msg }) => {
        eprintln!("Binance API error {}: {}", code, msg);
    }
    Err(e) => eprintln!("Other error: {}", e),
}
```

## Rate Limiting

The connector automatically enforces Binance's rate limits:

- **Default**: 1200 requests/minute
- **Algorithm**: Token bucket with automatic refill
- **Behavior**: Requests wait if limit exceeded (no errors thrown)

```rust
// Custom rate limit (lower if you're being conservative)
use binance_connector::BinanceClientBuilder;

let client = BinanceClientBuilder::new(config)
    .rate_limit(600)  // 600 req/min instead of 1200
    .build()?;
```

**Binance's actual limits**:

- REST API: 1200 requests/minute (weight-based)
- WebSocket: Unlimited (future feature)
- Order limits: Separate limits for trading (not yet implemented)

## Configuration Options

### Basic Configuration

```rust
// Mainnet (real prices)
let config = BinanceConfig::new(false);

// Testnet (for testing - uses fake data)
let config = BinanceConfig::new(true);
```

### Advanced Configuration

```rust
let client = BinanceClientBuilder::new(config)
    .timeout(20)           // 20 second timeout
    .rate_limit(600)       // 600 requests/minute
    .max_retries(5)        // retry up to 5 times
    .retries(true)         // enable automatic retries
    .build()?;
```

### Environment Variables (Optional)

```bash
# Optional - only needed for custom configuration
export BINANCE_TESTNET=false
export BINANCE_TIMEOUT_SECONDS=10
export BINANCE_REQUESTS_PER_MINUTE=1200
```

Load with:

```rust
let config = BinanceConfig::from_env()?;
```

## Testing

### Run All Tests

```bash
# Unit tests (no API calls)
cargo test --lib

# Mock server tests (no API calls)
cargo test mock_server

# Integration tests (connects to real Binance API - free)
cargo test --test integration_tests -- --ignored --nocapture
```

### Run Specific Test

```bash
cargo test test_get_ticker_price -- --ignored --nocapture
```

### Run Benchmarks

```bash
cargo bench
```

Expected performance (good internet connection):

- `get_ticker_price`: 100-200ms
- `get_ticker_24h`: 100-200ms
- `get_klines(100)`: 150-300ms
- `get_depth`: 100-200ms
- `rate_limiter_acquire`: <1µs

## Troubleshooting

### Connection Timeouts

**Cause**: Network issues or Binance server problems

**Solution**:

- Increase timeout: `.timeout(30)`
- Check Binance status: https://www.binance.com/en/support/announcement
- Verify your internet connection

### Rate Limit Errors

**Cause**: Too many requests too quickly (rare with default settings)

**Solution**:

- The connector handles this automatically with retries
- Lower rate limit: `.rate_limit(600)`
- Add delays between bulk operations

### "Invalid symbol" Error

**Cause**: Symbol doesn't exist or wrong format

**Solution**:

```rust
// Correct format (no slash, quote currency last)
client.get_ticker_price("BTCUSDT").await?;  // ✅ Correct
client.get_ticker_price("BTC/USDT").await?; // ❌ Wrong
client.get_ticker_price("USDTBTC").await?;  // ❌ Wrong order

// Get list of valid symbols
let symbols = client.get_exchange_info().await?;
for symbol in symbols {
    println!("{}", symbol.symbol);
}
```

### No Data During Weekends

**Note**: Cryptocurrency markets are **24/7**. Unlike forex, crypto never closes!

Binance trading is available:

- Monday - Sunday
- 24 hours a day
- 365 days a year

## Mainnet vs Testnet

### Mainnet (Production)

```rust
let config = BinanceConfig::new(false);
```

- **Real prices** from live markets
- **Real trading volume**
- Accurate, up-to-date data
- Use for production applications

### Testnet (Testing)

```rust
let config = BinanceConfig::new(true);
```

- **Fake prices** for testing
- Limited symbols available
- May be outdated or unrealistic
- Use for development/testing only

**Recommendation**: Use mainnet for market data (it's free!). Testnet is only useful when implementing trading features.

## Project Structure

```
binance-connector/
├── src/
│   ├── lib.rs           # Public API exports
│   ├── client.rs        # Main BinanceClient implementation
│   ├── config.rs        # Configuration management
│   ├── models.rs        # Data structures (Kline, Ticker, etc.)
│   ├── error.rs         # Error types
│   ├── endpoints.rs     # API endpoint definitions
│   ├── rate_limiter.rs  # Rate limiting logic
│   └── websocket.rs     # WebSocket (placeholder for Phase 2)
├── tests/
│   ├── integration_tests.rs  # Tests with real API
│   └── mock_server.rs        # Tests with mock server
├── examples/
│   └── fetch_example.rs      # Complete usage example
├── benches/
│   └── fetch_benchmark.rs    # Performance benchmarks
├── Cargo.toml
└── README.md
```

## API Coverage

✅ **Implemented** (Market Data):

- Get ticker price (single & all)
- Get 24h ticker statistics
- Get historical klines (candlesticks)
- Get order book depth
- Get recent trades
- Get exchange information
- Server time & health check

🚧 **Coming Soon** (Phase 3):

- WebSocket streaming (real-time prices)
- Account information (requires auth)
- Order placement (requires auth)
- Trade management (requires auth)

❌ **Not Planned**:

- Margin trading
- Futures trading
- Options trading

## Performance Tips

1. **Batch symbol requests when possible**:

```rust
   // Good: Get all prices at once
   let all_tickers = client.get_all_ticker_prices().await?;

   // Bad: Loop through symbols individually
   for symbol in symbols {
       let ticker = client.get_ticker_price(symbol).await?;
   }
```

2. **Reuse client instances**:

```rust
   // Good: Create once, reuse
   let client = BinanceClient::new(config)?;
   for _ in 0..100 {
       client.get_ticker_price("BTCUSDT").await?;
   }
```

3. **Use appropriate interval for klines**:

   - Smaller intervals (1s, 1m) = more data = slower
   - Larger intervals (1h, 1d) = less data = faster
   - Binance limit: 1000 klines per request

4. **Cache static data**:
   - Exchange info doesn't change often
   - Symbol information is relatively static
   - Fetch once, cache locally

## Why Binance?

- 🏆 **#1 cryptocurrency exchange** by volume
- 💰 **Deepest liquidity** (tightest spreads)
- 📊 **600+ trading pairs**
- 🆓 **Free market data** (no API key needed)
- ⚡ **Fast API** (low latency)
- 📚 **Well-documented API**
- 🌍 **Global availability**

## Comparison: OANDA vs Binance

| Feature              | OANDA              | Binance            |
| -------------------- | ------------------ | ------------------ |
| **Asset Class**      | Forex, Commodities | Cryptocurrency     |
| **API Key Required** | Yes                | No (market data)   |
| **Free Tier**        | Practice only      | Yes (production)   |
| **Real-time Data**   | Yes                | Yes                |
| **Market Hours**     | 24/5 (Mon-Fri)     | 24/7 (Always)      |
| **Liquidity**        | High (forex)       | Very High (crypto) |
| **Volatility**       | Low-Medium         | High               |
| **Use Case**         | Forex trading      | Crypto trading     |

**Use both**: OANDA for forex/commodities, Binance for crypto!

## Security Best Practices

1. **No credentials needed for market data** ✅

   - This connector doesn't require API keys
   - No sensitive data to protect
   - Safe to use in public repositories

2. **Future auth features** (when implemented):
   - Never commit API keys
   - Use environment variables
   - Enable IP whitelisting on Binance
   - Use read-only keys when possible

## Contributing

Contributions welcome! Please:

1. Fork the repository
2. Create a feature branch
3. Add tests for new functionality
4. Ensure `cargo test` passes
5. Run `cargo clippy` and fix warnings
6. Run `cargo fmt`
7. Submit a pull request

## Resources

- **Binance API Documentation**: https://binance-docs.github.io/apidocs/spot/en/
- **Binance Status**: https://www.binance.com/en/support/announcement
- **Binance Testnet**: https://testnet.binance.vision/
- **Support**: support@binance.com

## Disclaimer

This software is for educational and research purposes. Trading cryptocurrencies involves substantial risk of loss. The authors are not responsible for any financial losses incurred through use of this software.

Cryptocurrency markets are highly volatile. Never invest more than you can afford to lose.

## Changelog

### v0.2.0 (2025-11-04)

- Initial release
- REST API support for market data
- Rate limiting and retry logic
- Comprehensive test coverage
- Production-ready error handling
- No authentication required

**v0.3.0** (Future):

- Authentication support
- Account information
- Order placement
- Trade management

---

**Built with ❤️ and ⚡ in Rust for the crypto community**
