//! Integration tests for Binance connector
//! 
//! These tests connect to the real Binance API (no auth required)
//! Run with: cargo test --test integration_tests -- --ignored --nocapture

use binance_connector::{BinanceClient, BinanceConfig, Interval};
use std::time::Duration;

fn get_test_client() -> BinanceClient {
    // Use mainnet for tests (market data is free)
    let config = BinanceConfig::new(false);
    BinanceClient::new(config).expect("Failed to create client")
}

#[tokio::test]
#[ignore]
async fn test_health_check() {
    let client = get_test_client();
    
    let result = client.health_check().await;
    assert!(result.is_ok(), "Health check failed: {:?}", result);
    assert!(result.unwrap(), "Health check returned false");
}

#[tokio::test]
#[ignore]
async fn test_get_server_time() {
    let client = get_test_client();
    
    let time = client.get_server_time().await
        .expect("Failed to get server time");
    
    assert!(time > 0, "Server time should be positive");
    
    // Check it's reasonably current (within last hour)
    let now = chrono::Utc::now().timestamp_millis();
    let diff = (now - time).abs();
    assert!(diff < 3_600_000, "Server time seems wrong: diff={}ms", diff);
}

#[tokio::test]
#[ignore]
async fn test_get_ticker_price() {
    let client = get_test_client();
    
    let ticker = client.get_ticker_price("BTCUSDT").await
        .expect("Failed to get ticker price");
    
    assert_eq!(ticker.symbol, "BTCUSDT");
    assert!(ticker.price > 0.0, "Price should be positive");
    
    println!("BTC/USDT: ${}", ticker.price);
}

#[tokio::test]
#[ignore]
async fn test_get_all_ticker_prices() {
    let client = get_test_client();
    
    let tickers = client.get_all_ticker_prices().await
        .expect("Failed to get all tickers");
    
    assert!(!tickers.is_empty(), "Should have at least one ticker");
    assert!(tickers.len() > 100, "Binance has 100+ trading pairs");
    
    // Check BTCUSDT is in the list
    let btc = tickers.iter().find(|t| t.symbol == "BTCUSDT");
    assert!(btc.is_some(), "BTCUSDT should be in the list");
    
    println!("Total symbols: {}", tickers.len());
}

#[tokio::test]
#[ignore]
async fn test_get_ticker_24h() {
    let client = get_test_client();
    
    let ticker = client.get_ticker_24h("ETHUSDT").await
        .expect("Failed to get 24h ticker");
    
    assert_eq!(ticker.symbol, "ETHUSDT");
    assert!(ticker.last_price > 0.0);
    assert!(ticker.volume > 0.0);
    assert!(ticker.high_price >= ticker.low_price);
    assert!(ticker.ask_price >= ticker.bid_price);
    assert!(ticker.spread() >= 0.0);
    
    println!("ETH/USDT: ${} (24h change: {:.2}%)",
        ticker.last_price,
        ticker.price_change_percent
    );
}

#[tokio::test]
#[ignore]
async fn test_get_klines() {
    let client = get_test_client();
    
    let klines = client.get_klines("BTCUSDT", Interval::Minutes5, 10).await
        .expect("Failed to get klines");
    
    assert_eq!(klines.len(), 10);
    
    for kline in &klines {
        assert_eq!(kline.symbol, "BTCUSDT");
        assert!(kline.open > 0.0);
        assert!(kline.high >= kline.low);
        assert!(kline.high >= kline.open);
        assert!(kline.high >= kline.close);
        assert!(kline.low <= kline.open);
        assert!(kline.low <= kline.close);
        assert!(kline.volume >= 0.0);
    }
    
    // Check chronological order
    for i in 1..klines.len() {
        assert!(klines[i].open_time > klines[i-1].open_time,
            "Klines should be in chronological order");
    }
    
    println!("Latest kline: O=${:.2} H=${:.2} L=${:.2} C=${:.2}",
        klines.last().unwrap().open,
        klines.last().unwrap().high,
        klines.last().unwrap().low,
        klines.last().unwrap().close
    );
}

#[tokio::test]
#[ignore]
async fn test_get_klines_max_limit() {
    let client = get_test_client();
    
    // Binance allows max 1000 klines
    let result = client.get_klines("BTCUSDT", Interval::Minutes1, 1000).await;
    assert!(result.is_ok());
    
    // Should fail with limit > 1000
    let result = client.get_klines("BTCUSDT", Interval::Minutes1, 1001).await;
    assert!(result.is_err());
}

#[tokio::test]
#[ignore]
async fn test_get_depth() {
    let client = get_test_client();
    
    let order_book = client.get_depth("BTCUSDT", 10).await
        .expect("Failed to get order book");
    
    assert_eq!(order_book.symbol, "BTCUSDT");
    assert_eq!(order_book.bids.len(), 10);
    assert_eq!(order_book.asks.len(), 10);
    
    // Bids should be descending (highest first)
    for i in 1..order_book.bids.len() {
        assert!(order_book.bids[i].price < order_book.bids[i-1].price,
            "Bids should be in descending order");
    }
    
    // Asks should be ascending (lowest first)
    for i in 1..order_book.asks.len() {
        assert!(order_book.asks[i].price > order_book.asks[i-1].price,
            "Asks should be in ascending order");
    }
    
    // Best bid should be less than best ask
    let best_bid = order_book.bids.first().unwrap().price;
    let best_ask = order_book.asks.first().unwrap().price;
    assert!(best_bid < best_ask, "Best bid should be less than best ask");
    
    println!("Spread: ${:.2}", best_ask - best_bid);
}

#[tokio::test]
#[ignore]
async fn test_get_recent_trades() {
    let client = get_test_client();
    
    let trades = client.get_recent_trades("BTCUSDT", 10).await
        .expect("Failed to get recent trades");
    
    assert_eq!(trades.len(), 10);
    
    for trade in &trades {
        assert_eq!(trade.symbol, "BTCUSDT");
        assert!(trade.price > 0.0);
        assert!(trade.quantity > 0.0);
        assert!(trade.quote_quantity > 0.0);
    }
    
    // Check chronological order
    for i in 1..trades.len() {
        assert!(trades[i].id > trades[i-1].id,
            "Trades should be in chronological order");
    }
}

#[tokio::test]
#[ignore]
async fn test_get_exchange_info() {
    let client = get_test_client();
    
    let symbols = client.get_exchange_info().await
        .expect("Failed to get exchange info");
    
    assert!(!symbols.is_empty());
    
    // Check BTCUSDT exists
    let btcusdt = symbols.iter().find(|s| s.symbol == "BTCUSDT");
    assert!(btcusdt.is_some(), "BTCUSDT should exist");
    
    let btcusdt = btcusdt.unwrap();
    assert_eq!(btcusdt.base_asset, "BTC");
    assert_eq!(btcusdt.quote_asset, "USDT");
    assert_eq!(btcusdt.status, "TRADING");
    
    println!("Total symbols: {}", symbols.len());
}

#[tokio::test]
#[ignore]
async fn test_invalid_symbol() {
    let client = get_test_client();
    
    let result = client.get_ticker_price("INVALID_SYMBOL").await;
    assert!(result.is_err(), "Should fail with invalid symbol");
}

#[tokio::test]
#[ignore]
async fn test_rate_limiting() {
    let client = get_test_client();
    
    let start = std::time::Instant::now();
    
    // Make 30 rapid requests
    for _ in 0..30 {
        let _ = client.get_ticker_price("BTCUSDT").await;
    }
    
    let elapsed = start.elapsed();
    
    // With 1200 req/min limit, 30 requests should take < 2 seconds
    assert!(elapsed < Duration::from_secs(2),
        "Rate limiting seems too restrictive: took {:?}", elapsed);
    
    println!("30 requests completed in {:?}", elapsed);
}

#[tokio::test]
#[ignore]
async fn test_interval_parsing() {
    use std::str::FromStr;
    
    assert_eq!(Interval::from_str("1m").unwrap(), Interval::Minutes1);
    assert_eq!(Interval::from_str("5m").unwrap(), Interval::Minutes5);
    assert_eq!(Interval::from_str("1h").unwrap(), Interval::Hours1);
    assert_eq!(Interval::from_str("1d").unwrap(), Interval::Days1);
    assert!(Interval::from_str("invalid").is_err());
}

#[tokio::test]
#[ignore]
async fn test_concurrent_requests() {
    let client = get_test_client();
    
    // Make 5 concurrent requests
    let futures: Vec<_> = (0..5)
        .map(|_| client.get_ticker_price("BTCUSDT"))
        .collect();
    
    let results = futures::future::join_all(futures).await;
    
    // All should succeed
    for result in results {
        assert!(result.is_ok(), "Concurrent request failed");
    }
}