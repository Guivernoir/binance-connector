//! Example: Fetch cryptocurrency prices and data from Binance
//! 
//! Usage:
//!   cargo run --example fetch_example

use binance_connector::{BinanceClient, BinanceConfig, Interval};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Binance Connector Example\n");
    
    // Create client (no API key needed for market data)
    let config = BinanceConfig::new(false); // false = mainnet, true = testnet
    println!("✅ Configuration:");
    println!("   Network: {}", if config.testnet { "Testnet" } else { "Mainnet" });
    println!("   REST URL: {}", config.get_base_url());
    println!("   WebSocket URL: {}\n", config.get_ws_url());
    
    let client = BinanceClient::new(config)?;
    println!("✅ Client created\n");
    
    // 1. Health check
    println!("🏥 Health Check:");
    match client.health_check().await {
        Ok(true) => println!("   ✅ Connected to Binance\n"),
        Ok(false) => {
            println!("   ❌ Connection failed\n");
            return Ok(());
        }
        Err(e) => {
            println!("   ❌ Error: {}\n", e);
            return Ok(());
        }
    }
    
    // 2. Get server time
    println!("🕐 Server Time:");
    match client.get_server_time().await {
        Ok(time) => {
            let dt = chrono::DateTime::from_timestamp_millis(time)
                .unwrap_or_default();
            println!("   Server time: {} ({})\n", dt, time);
        }
        Err(e) => println!("   ❌ Error: {}\n", e),
    }
    
    // 3. Get BTC/USDT price
    println!("💰 Current Price (BTC/USDT):");
    match client.get_ticker_price("BTCUSDT").await {
        Ok(ticker) => {
            println!("   Price: ${}", ticker.price);
            println!("   Time: {}\n", ticker.timestamp);
        }
        Err(e) => println!("   ❌ Error: {}\n", e),
    }
    
    // 4. Get 24h statistics
    println!("📊 24-Hour Statistics (BTC/USDT):");
    match client.get_ticker_24h("BTCUSDT").await {
        Ok(ticker) => {
            println!("   Price: ${}", ticker.last_price);
            println!("   Change: ${} ({:.2}%)",
                ticker.price_change,
                ticker.price_change_percent
            );
            println!("   High: ${}", ticker.high_price);
            println!("   Low: ${}", ticker.low_price);
            println!("   Volume: {} BTC", ticker.volume);
            println!("   Quote Volume: ${}", ticker.quote_volume);
            println!("   Bid: ${}", ticker.bid_price);
            println!("   Ask: ${}", ticker.ask_price);
            println!("   Spread: ${:.2}", ticker.spread());
            println!("   Trades: {}\n", ticker.count);
        }
        Err(e) => println!("   ❌ Error: {}\n", e),
    }
    
    // 5. Get multiple prices
    println!("💱 Multiple Prices:");
    let symbols = vec!["BTCUSDT", "ETHUSDT", "BNBUSDT"];
    for symbol in &symbols {
        match client.get_ticker_price(symbol).await {
            Ok(ticker) => {
                println!("   {}: ${:.2}", ticker.symbol, ticker.price);
            }
            Err(e) => println!("   {} ❌ Error: {}", symbol, e),
        }
    }
    println!();
    
    // 6. Get historical klines
    println!("📈 Historical Klines (BTC/USDT, 5m, last 10):");
    match client.get_klines("BTCUSDT", Interval::Minutes5, 10).await {
        Ok(klines) => {
            println!("   Fetched {} klines", klines.len());
            for (i, kline) in klines.iter().take(5).enumerate() {
                println!("   [{}] {} | O:${:.2} H:${:.2} L:${:.2} C:${:.2} V:{:.4}",
                    i + 1,
                    kline.open_time.format("%Y-%m-%d %H:%M"),
                    kline.open,
                    kline.high,
                    kline.low,
                    kline.close,
                    kline.volume
                );
            }
            if klines.len() > 5 {
                println!("   ... ({} more)", klines.len() - 5);
            }
            println!();
        }
        Err(e) => println!("   ❌ Error: {}\n", e),
    }
    
    // 7. Get order book depth
    println!("📖 Order Book Depth (BTC/USDT, top 5):");
    match client.get_depth("BTCUSDT", 5).await {
        Ok(order_book) => {
            println!("   Last Update ID: {}", order_book.last_update_id);
            println!("   Bids (top 5):");
            for (i, bid) in order_book.bids.iter().take(5).enumerate() {
                println!("     [{}] ${:.2} × {:.6}", i + 1, bid.price, bid.quantity);
            }
            println!("   Asks (top 5):");
            for (i, ask) in order_book.asks.iter().take(5).enumerate() {
                println!("     [{}] ${:.2} × {:.6}", i + 1, ask.price, ask.quantity);
            }
            println!();
        }
        Err(e) => println!("   ❌ Error: {}\n", e),
    }
    
    // 8. Get recent trades
    println!("📜 Recent Trades (BTC/USDT, last 5):");
    match client.get_recent_trades("BTCUSDT", 5).await {
        Ok(trades) => {
            for (i, trade) in trades.iter().enumerate() {
                let side = if trade.is_buyer_maker { "SELL" } else { "BUY " };
                println!("   [{}] {} ${:.2} × {:.6} at {}",
                    i + 1,
                    side,
                    trade.price,
                    trade.quantity,
                    trade.time.format("%H:%M:%S")
                );
            }
            println!();
        }
        Err(e) => println!("   ❌ Error: {}\n", e),
    }
    
    // 9. Get exchange info (first 10 symbols)
    println!("🎯 Available Symbols (first 10):");
    match client.get_exchange_info().await {
        Ok(symbols) => {
            for (i, symbol) in symbols.iter().take(10).enumerate() {
                println!("   [{}] {} ({}/{})",
                    i + 1,
                    symbol.symbol,
                    symbol.base_asset,
                    symbol.quote_asset
                );
            }
            println!("   ... (total: {} symbols)\n", symbols.len());
        }
        Err(e) => println!("   ❌ Error: {}\n", e),
    }
    
    println!("✅ Example completed successfully!");
    
    Ok(())
}