//! Example: Advanced WebSocket streaming with reconnection and error handling
//! 
//! Usage:
//!   cargo run --example advanced_stream

use binance_connector::{BinanceWebSocket, BinanceConfig, Interval};
use std::collections::HashMap;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Advanced WebSocket Streaming Example\n");
    
    let config = BinanceConfig::new(false);
    let ws = BinanceWebSocket::new(config)?;
    
    // Example: Monitor multiple kline streams with statistics
    println!("📊 Monitoring BTC/USDT 1m candles with statistics\n");
    
    monitor_klines_with_stats(&ws).await?;
    
    Ok(())
}

async fn monitor_klines_with_stats(ws: &BinanceWebSocket) -> Result<(), Box<dyn std::error::Error>> {
    let mut stream = ws.kline_stream("BTCUSDT", Interval::Minutes1).await?;
    
    let mut stats = KlineStats::new();
    let mut error_count = 0;
    let start_time = tokio::time::Instant::now();
    
    println!("Streaming started... (Ctrl+C to stop)\n");
    
    loop {
        match stream.recv().await {
            Some(Ok(kline)) => {
                error_count = 0; // Reset error count on success
                stats.update(&kline);
                
                if kline.is_closed {
                    println!("✅ Candle CLOSED at {}:", kline.close_time.format("%H:%M:%S"));
                    println!("   O: ${:.2} | H: ${:.2} | L: ${:.2} | C: ${:.2}",
                        kline.open, kline.high, kline.low, kline.close);
                    println!("   Volume: {:.4} BTC | Trades: {}", kline.volume, kline.trades);
                    println!("   Change: ${:.2} ({:.2}%)\n",
                        kline.close - kline.open,
                        ((kline.close - kline.open) / kline.open) * 100.0
                    );
                    
                    stats.print_summary();
                } else {
                    // Print periodic updates for current candle
                    if stats.update_count % 10 == 0 {
                        println!("📈 Current candle (updating): C=${:.2} | V={:.4} BTC",
                            kline.close, kline.volume);
                    }
                }
            }
            Some(Err(e)) => {
                error_count += 1;
                eprintln!("❌ Stream error ({}): {}", error_count, e);
                
                if error_count > 5 {
                    eprintln!("Too many consecutive errors, attempting reconnection...");
                    sleep(Duration::from_secs(5)).await;
                    
                    // Reconnect
                    match ws.kline_stream("BTCUSDT", Interval::Minutes1).await {
                        Ok(new_stream) => {
                            stream = new_stream;
                            error_count = 0;
                            println!("✅ Reconnected successfully");
                        }
                        Err(e) => {
                            eprintln!("❌ Reconnection failed: {}", e);
                            break;
                        }
                    }
                }
            }
            None => {
                println!("Stream ended");
                break;
            }
        }
        
        // Safety: Stop after 5 minutes
        if start_time.elapsed() > Duration::from_secs(300) {
            println!("\n⏰ 5-minute limit reached, stopping...");
            break;
        }
    }
    
    println!("\n📊 Final Statistics:");
    stats.print_summary();
    
    Ok(())
}

struct KlineStats {
    closed_candles: usize,
    update_count: usize,
    total_volume: f64,
    highest_price: f64,
    lowest_price: f64,
    price_changes: Vec<f64>,
}

impl KlineStats {
    fn new() -> Self {
        Self {
            closed_candles: 0,
            update_count: 0,
            total_volume: 0.0,
            highest_price: 0.0,
            lowest_price: f64::MAX,
            price_changes: Vec::new(),
        }
    }
    
    fn update(&mut self, kline: &binance_connector::Kline) {
        self.update_count += 1;
        
        if kline.is_closed {
            self.closed_candles += 1;
            self.total_volume += kline.volume;
            
            let change_pct = ((kline.close - kline.open) / kline.open) * 100.0;
            self.price_changes.push(change_pct);
        }
        
        if kline.high > self.highest_price {
            self.highest_price = kline.high;
        }
        if kline.low < self.lowest_price {
            self.lowest_price = kline.low;
        }
    }
    
    fn print_summary(&self) {
        println!("───────────────────────────────────");
        println!("Total updates received: {}", self.update_count);
        println!("Closed candles: {}", self.closed_candles);
        println!("Total volume: {:.4} BTC", self.total_volume);
        println!("Highest price: ${:.2}", self.highest_price);
        println!("Lowest price: ${:.2}", self.lowest_price);
        
        if !self.price_changes.is_empty() {
            let avg_change: f64 = self.price_changes.iter().sum::<f64>() / self.price_changes.len() as f64;
            let positive = self.price_changes.iter().filter(|&&x| x > 0.0).count();
            let negative = self.price_changes.iter().filter(|&&x| x < 0.0).count();
            
            println!("Average change: {:.2}%", avg_change);
            println!("Bullish candles: {} | Bearish candles: {}", positive, negative);
        }
        println!("───────────────────────────────────\n");
    }
}