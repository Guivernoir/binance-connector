//! Example: Stream real-time data from Binance WebSocket
//! 
//! Usage:
//!   cargo run --example stream_example

use binance_connector::{BinanceWebSocket, BinanceConfig, Interval};
use futures_util::StreamExt;
use tokio::time::{timeout, Duration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🌊 Binance WebSocket Streaming Example\n");
    
    let config = BinanceConfig::new(false);
    let ws = BinanceWebSocket::new(config)?;
    
    println!("✅ WebSocket client created\n");
    
    // Example 1: Stream ticker updates
    println!("📊 Example 1: Real-time Ticker Stream (BTC/USDT)");
    println!("   Streaming for 10 seconds...\n");
    
    stream_ticker_example(&ws).await?;
    
    println!("\n---\n");
    
    // Example 2: Stream kline/candlestick updates
    println!("🕯️  Example 2: Real-time Kline Stream (ETH/USDT, 1m)");
    println!("   Streaming for 10 seconds...\n");
    
    stream_kline_example(&ws).await?;
    
    println!("\n---\n");
    
    // Example 3: Stream trades
    println!("💱 Example 3: Real-time Trade Stream (BNB/USDT)");
    println!("   Streaming for 10 seconds...\n");
    
    stream_trade_example(&ws).await?;
    
    println!("\n---\n");
    
    // Example 4: Stream order book depth
    println!("📖 Example 4: Real-time Depth Stream (SOL/USDT)");
    println!("   Streaming for 10 seconds...\n");
    
    stream_depth_example(&ws).await?;
    
    println!("\n---\n");
    
    // Example 5: Multiple symbols
    println!("🔀 Example 5: Multiple Symbol Streams");
    println!("   Streaming BTC, ETH, BNB for 10 seconds...\n");
    
    stream_multiple_example(&ws).await?;
    
    println!("\n✅ All streaming examples completed!");
    
    Ok(())
}

async fn stream_ticker_example(ws: &BinanceWebSocket) -> Result<(), Box<dyn std::error::Error>> {
    let mut stream = ws.ticker_stream("BTCUSDT").await?;
    
    let mut count = 0;
    let result = timeout(Duration::from_secs(10), async {
        while let Some(result) = stream.recv().await {
            match result {
                Ok(ticker) => {
                    count += 1;
                    println!("   [{}] ${:.2} | Change: {:.2}% | Volume: {:.2} BTC",
                        count,
                        ticker.last_price,
                        ticker.price_change_percent,
                        ticker.volume
                    );
                    
                    if count >= 5 {
                        break;
                    }
                }
                Err(e) => eprintln!("   ❌ Error: {}", e),
            }
        }
    }).await;
    
    match result {
        Ok(_) => println!("   ✅ Received {} updates", count),
        Err(_) => println!("   ⏰ Timeout (no updates in 10s)"),
    }
    
    Ok(())
}

async fn stream_kline_example(ws: &BinanceWebSocket) -> Result<(), Box<dyn std::error::Error>> {
    let mut stream = ws.kline_stream("ETHUSDT", Interval::Minutes1).await?;
    
    let mut count = 0;
    let result = timeout(Duration::from_secs(10), async {
        while let Some(result) = stream.recv().await {
            match result {
                Ok(kline) => {
                    count += 1;
                    let status = if kline.is_closed { "CLOSED" } else { "UPDATING" };
                    println!("   [{}] {} | O:${:.2} H:${:.2} L:${:.2} C:${:.2} | {}",
                        count,
                        kline.open_time.format("%H:%M:%S"),
                        kline.open,
                        kline.high,
                        kline.low,
                        kline.close,
                        status
                    );
                    
                    if count >= 5 {
                        break;
                    }
                }
                Err(e) => eprintln!("   ❌ Error: {}", e),
            }
        }
    }).await;
    
    match result {
        Ok(_) => println!("   ✅ Received {} kline updates", count),
        Err(_) => println!("   ⏰ Timeout"),
    }
    
    Ok(())
}

async fn stream_trade_example(ws: &BinanceWebSocket) -> Result<(), Box<dyn std::error::Error>> {
    let mut stream = ws.trade_stream("BNBUSDT").await?;
    
    let mut count = 0;
    let result = timeout(Duration::from_secs(10), async {
        while let Some(result) = stream.recv().await {
            match result {
                Ok(trade) => {
                    count += 1;
                    let side = if trade.is_buyer_maker { "SELL" } else { "BUY " };
                    println!("   [{}] {} ${:.2} × {:.4} BNB at {}",
                        count,
                        side,
                        trade.price,
                        trade.quantity,
                        trade.time.format("%H:%M:%S")
                    );
                    
                    if count >= 10 {
                        break;
                    }
                }
                Err(e) => eprintln!("   ❌ Error: {}", e),
            }
        }
    }).await;
    
    match result {
        Ok(_) => println!("   ✅ Received {} trades", count),
        Err(_) => println!("   ⏰ Timeout"),
    }
    
    Ok(())
}

async fn stream_depth_example(ws: &BinanceWebSocket) -> Result<(), Box<dyn std::error::Error>> {
    let mut stream = ws.depth_stream("SOLUSDT").await?;
    
    let mut count = 0;
    let result = timeout(Duration::from_secs(10), async {
        while let Some(result) = stream.recv().await {
            match result {
                Ok(order_book) => {
                    count += 1;
                    
                    let best_bid = order_book.bids.first().map(|b| b.price).unwrap_or(0.0);
                    let best_ask = order_book.asks.first().map(|a| a.price).unwrap_or(0.0);
                    let spread = best_ask - best_bid;
                    
                    println!("   [{}] Update #{} | Best Bid: ${:.4} | Best Ask: ${:.4} | Spread: ${:.4}",
                        count,
                        order_book.last_update_id,
                        best_bid,
                        best_ask,
                        spread
                    );
                    
                    if count >= 5 {
                        break;
                    }
                }
                Err(e) => eprintln!("   ❌ Error: {}", e),
            }
        }
    }).await;
    
    match result {
        Ok(_) => println!("   ✅ Received {} depth updates", count),
        Err(_) => println!("   ⏰ Timeout"),
    }
    
    Ok(())
}

async fn stream_multiple_example(ws: &BinanceWebSocket) -> Result<(), Box<dyn std::error::Error>> {
    // Stream mini tickers for multiple symbols
    let symbols = vec!["BTCUSDT", "ETHUSDT", "BNBUSDT"];
    
    let mut streams = vec![];
    for symbol in &symbols {
        let stream = ws.mini_ticker_stream(symbol).await?;
        streams.push((symbol.to_string(), stream));
    }
    
    println!("   Streaming {} symbols...", symbols.len());
    
    let mut counts = std::collections::HashMap::new();
    let start = tokio::time::Instant::now();
    
    loop {
        if start.elapsed() > Duration::from_secs(10) {
            break;
        }
        
        for (symbol, stream) in &mut streams {
            match timeout(Duration::from_millis(100), stream.recv()).await {
                Ok(Some(Ok(ticker))) => {
                    let count = counts.entry(symbol.clone()).or_insert(0);
                    *count += 1;
                    println!("   {} → ${:.2}", ticker.symbol, ticker.price);
                }
                Ok(Some(Err(e))) => eprintln!("   {} ❌ Error: {}", symbol, e),
                Ok(None) => break,
                Err(_) => continue, // Timeout, try next stream
            }
        }
    }
    
    println!("\n   ✅ Updates received:");
    for (symbol, count) in counts {
        println!("      {}: {} updates", symbol, count);
    }
    
    Ok(())
}