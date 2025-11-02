//! WebSocket integration tests
//! 
//! Run with: cargo test --test websocket_tests -- --ignored --nocapture

use binance_connector::{BinanceWebSocket, BinanceConfig, Interval};
use tokio::time::{timeout, Duration};

fn get_test_ws() -> BinanceWebSocket {
    let config = BinanceConfig::new(false);
    BinanceWebSocket::new(config).expect("Failed to create WebSocket client")
}

#[tokio::test]
#[ignore]
async fn test_ticker_stream_connection() {
    let ws = get_test_ws();
    
    let mut stream = ws.ticker_stream("BTCUSDT").await
        .expect("Failed to connect to ticker stream");
    
    // Should receive at least one message within 10 seconds
    let result = timeout(Duration::from_secs(10), stream.recv()).await;
    
    assert!(result.is_ok(), "Should receive message within timeout");
    
    let message = result.unwrap();
    assert!(message.is_some(), "Should receive a message");
    
    let ticker = message.unwrap().expect("Should be Ok result");
    assert_eq!(ticker.symbol, "BTCUSDT");
    assert!(ticker.last_price > 0.0);
    
    println!("✅ Received ticker: ${}", ticker.last_price);
}

#[tokio::test]
#[ignore]
async fn test_kline_stream_connection() {
    let ws = get_test_ws();
    
    let mut stream = ws.kline_stream("ETHUSDT", Interval::Minutes1).await
        .expect("Failed to connect to kline stream");
    
    let result = timeout(Duration::from_secs(10), stream.recv()).await;
    
    assert!(result.is_ok());
    
    let message = result.unwrap().unwrap();
    let kline = message.expect("Should be Ok result");
    
    assert_eq!(kline.symbol, "ETHUSDT");
    assert!(kline.open > 0.0);
    assert!(kline.high >= kline.low);
    
    println!("✅ Received kline: O={} C={} (closed={})",
        kline.open, kline.close, kline.is_closed);
}

#[tokio::test]
#[ignore]
async fn test_trade_stream_connection() {
    let ws = get_test_ws();
    
    let mut stream = ws.trade_stream("BTCUSDT").await
        .expect("Failed to connect to trade stream");
    
    let result = timeout(Duration::from_secs(10), stream.recv()).await;
    
    assert!(result.is_ok());
    
    let message = result.unwrap().unwrap();
    let trade = message.expect("Should be Ok result");
    
    assert_eq!(trade.symbol, "BTCUSDT");
    assert!(trade.price > 0.0);
    assert!(trade.quantity > 0.0);
    
    println!("✅ Received trade: {} ${} × {}",
        if trade.is_buyer_maker { "SELL" } else { "BUY" },
        trade.price,
        trade.quantity
    );
}

#[tokio::test]
#[ignore]
async fn test_depth_stream_connection() {
    let ws = get_test_ws();
    
    let mut stream = ws.depth_stream("BTCUSDT").await
        .expect("Failed to connect to depth stream");
    
    let result = timeout(Duration::from_secs(10), stream.recv()).await;
    
    assert!(result.is_ok());
    
    let message = result.unwrap().unwrap();
    let order_book = message.expect("Should be Ok result");
    
    assert_eq!(order_book.symbol, "BTCUSDT");
    assert!(!order_book.bids.is_empty());
    assert!(!order_book.asks.is_empty());
    
    let best_bid = order_book.bids[0].price;
    let best_ask = order_book.asks[0].price;
    
    assert!(best_ask > best_bid, "Ask should be higher than bid");
    
    println!("✅ Received depth: bid=${} ask=${} spread=${}",
        best_bid, best_ask, best_ask - best_bid);
}

#[tokio::test]
#[ignore]
async fn test_mini_ticker_stream() {
    let ws = get_test_ws();
    
    let mut stream = ws.mini_ticker_stream("BNBUSDT").await
        .expect("Failed to connect to mini ticker stream");
    
    let result = timeout(Duration::from_secs(10), stream.recv()).await;
    
    assert!(result.is_ok());
    
    let message = result.unwrap().unwrap();
    let ticker = message.expect("Should be Ok result");
    
    assert_eq!(ticker.symbol, "BNBUSDT");
    assert!(ticker.price > 0.0);
    
    println!("✅ Received mini ticker: ${}", ticker.price);
}

#[tokio::test]
#[ignore]
async fn test_multiple_messages() {
    let ws = get_test_ws();
    
    let mut stream = ws.ticker_stream("BTCUSDT").await.unwrap();
    
    let mut count = 0;
    let start = tokio::time::Instant::now();
    
    while count < 5 && start.elapsed() < Duration::from_secs(30) {
        if let Some(Ok(ticker)) = stream.recv().await {
            count += 1;
            println!("Message {}: ${}", count, ticker.last_price);
        }
    }
    
    assert_eq!(count, 5, "Should receive 5 messages");
}

#[tokio::test]
#[ignore]
async fn test_stream_closed_candle_detection() {
    let ws = get_test_ws();
    
    let mut stream = ws.kline_stream("BTCUSDT", Interval::Minutes1).await.unwrap();
    
    // Wait for a closed candle (max 2 minutes)
    let mut found_closed = false;
    let start = tokio::time::Instant::now();
    
    while !found_closed && start.elapsed() < Duration::from_secs(120) {
        if let Some(Ok(kline)) = stream.recv().await {
            if kline.is_closed {
                found_closed = true;
                println!("✅ Found closed candle at {}", kline.close_time);
                break;
            }
        }
    }
    
    assert!(found_closed, "Should find at least one closed candle within 2 minutes");
}

#[tokio::test]
#[ignore]
async fn test_combined_stream() {
    let ws = get_test_ws();
    
    let streams = vec!["btcusdt@ticker", "ethusdt@ticker"];
    let mut stream = ws.combined_stream(&streams).await.unwrap();
    
    // Should receive messages from either stream
    let result = timeout(Duration::from_secs(10), stream.recv()).await;
    
    assert!(result.is_ok());
    
    let message = result.unwrap().unwrap().expect("Should be Ok result");
    
    // Message should contain data from one of the streams
    assert!(message.contains("BTCUSDT") || message.contains("ETHUSDT"));
    
    println!("✅ Received combined stream message");
}