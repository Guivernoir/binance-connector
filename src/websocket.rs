//! WebSocket streaming client for real-time Binance data
//! 
//! Provides async streams for:
//! - Real-time ticker updates
//! - Live kline/candlestick updates
//! - Trade stream
//! - Order book depth updates
//! - Aggregate trade stream

use crate::{
    config::BinanceConfig,
    endpoints::WebSocketStreams,
    error::{Error, Result},
    models::{Interval, Kline, OrderBook, PriceLevel, Ticker, Ticker24h, Trade},
};
use chrono::{DateTime, Utc};
use futures_util::{SinkExt, StreamExt};
use serde::Deserialize;
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tokio::time::{sleep, Duration};
use tokio_tungstenite::{
    connect_async, tungstenite::Message, MaybeTlsStream, WebSocketStream,
};

type WsStream = WebSocketStream<MaybeTlsStream<TcpStream>>;

/// WebSocket connection manager
#[derive(Clone)]
pub struct BinanceWebSocket {
    config: Arc<BinanceConfig>,
}

impl BinanceWebSocket {
    /// Create new WebSocket client
    pub fn new(config: BinanceConfig) -> Result<Self> {
        config.validate()?;
        Ok(Self {
            config: Arc::new(config),
        })
    }

    /// Stream real-time ticker updates for a symbol
    /// 
    /// # Arguments
    /// * `symbol` - Trading pair (e.g., "BTCUSDT")
    /// 
    /// # Example
    /// ```no_run
    /// use binance_connector::{BinanceWebSocket, BinanceConfig};
    /// use futures_util::StreamExt;
    /// 
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let config = BinanceConfig::new(false);
    ///     let ws = BinanceWebSocket::new(config)?;
    ///     
    ///     let mut stream = ws.ticker_stream("BTCUSDT").await?;
    ///     
    ///     while let Some(result) = stream.recv().await {
    ///         match result {
    ///             Ok(ticker) => println!("BTC: ${}", ticker.last_price),
    ///             Err(e) => eprintln!("Error: {}", e),
    ///         }
    ///     }
    ///     
    ///     Ok(())
    /// }
    /// ```
    pub async fn ticker_stream(
        &self,
        symbol: &str,
    ) -> Result<mpsc::Receiver<Result<Ticker24h>>> {
        let stream_name = WebSocketStreams::ticker(symbol);
        let url = format!("{}/{}", self.config.get_ws_url(), stream_name);
        
        let (tx, rx) = mpsc::channel(100);
        let symbol = symbol.to_string();
        
        tokio::spawn(async move {
            if let Err(e) = Self::ticker_stream_handler(url, symbol, tx.clone()).await {
                let _ = tx.send(Err(e)).await;
            }
        });
        
        Ok(rx)
    }

    /// Stream real-time kline/candlestick updates
    /// 
    /// # Arguments
    /// * `symbol` - Trading pair (e.g., "BTCUSDT")
    /// * `interval` - Candlestick interval
    /// 
    /// # Example
    /// ```no_run
    /// use binance_connector::{BinanceWebSocket, BinanceConfig, Interval};
    /// use futures_util::StreamExt;
    /// 
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let config = BinanceConfig::new(false);
    ///     let ws = BinanceWebSocket::new(config)?;
    ///     
    ///     let mut stream = ws.kline_stream("BTCUSDT", Interval::Minutes1).await?;
    ///     
    ///     while let Some(result) = stream.recv().await {
    ///         match result {
    ///             Ok(kline) => {
    ///                 if kline.is_closed {
    ///                     println!("Candle closed: O={} C={}", kline.open, kline.close);
    ///                 }
    ///             }
    ///             Err(e) => eprintln!("Error: {}", e),
    ///         }
    ///     }
    ///     
    ///     Ok(())
    /// }
    /// ```
    pub async fn kline_stream(
        &self,
        symbol: &str,
        interval: Interval,
    ) -> Result<mpsc::Receiver<Result<Kline>>> {
        let stream_name = WebSocketStreams::kline(symbol, &interval.to_string());
        let url = format!("{}/{}", self.config.get_ws_url(), stream_name);
        
        let (tx, rx) = mpsc::channel(100);
        let symbol = symbol.to_string();
        
        tokio::spawn(async move {
            if let Err(e) = Self::kline_stream_handler(url, symbol, tx.clone()).await {
                let _ = tx.send(Err(e)).await;
            }
        });
        
        Ok(rx)
    }

    /// Stream real-time trade updates
    /// 
    /// # Arguments
    /// * `symbol` - Trading pair (e.g., "BTCUSDT")
    pub async fn trade_stream(&self, symbol: &str) -> Result<mpsc::Receiver<Result<Trade>>> {
        let stream_name = WebSocketStreams::trade(symbol);
        let url = format!("{}/{}", self.config.get_ws_url(), stream_name);
        
        let (tx, rx) = mpsc::channel(100);
        let symbol = symbol.to_string();
        
        tokio::spawn(async move {
            if let Err(e) = Self::trade_stream_handler(url, symbol, tx.clone()).await {
                let _ = tx.send(Err(e)).await;
            }
        });
        
        Ok(rx)
    }

    /// Stream order book depth updates
    /// 
    /// # Arguments
    /// * `symbol` - Trading pair (e.g., "BTCUSDT")
    pub async fn depth_stream(&self, symbol: &str) -> Result<mpsc::Receiver<Result<OrderBook>>> {
        let stream_name = WebSocketStreams::depth(symbol);
        let url = format!("{}/{}", self.config.get_ws_url(), stream_name);
        
        let (tx, rx) = mpsc::channel(100);
        let symbol = symbol.to_string();
        
        tokio::spawn(async move {
            if let Err(e) = Self::depth_stream_handler(url, symbol, tx.clone()).await {
                let _ = tx.send(Err(e)).await;
            }
        });
        
        Ok(rx)
    }

    /// Stream mini ticker (lightweight ticker updates)
    /// 
    /// # Arguments
    /// * `symbol` - Trading pair (e.g., "BTCUSDT")
    pub async fn mini_ticker_stream(
        &self,
        symbol: &str,
    ) -> Result<mpsc::Receiver<Result<Ticker>>> {
        let stream_name = WebSocketStreams::mini_ticker(symbol);
        let url = format!("{}/{}", self.config.get_ws_url(), stream_name);
        
        let (tx, rx) = mpsc::channel(100);
        let symbol = symbol.to_string();
        
        tokio::spawn(async move {
            if let Err(e) = Self::mini_ticker_stream_handler(url, symbol, tx.clone()).await {
                let _ = tx.send(Err(e)).await;
            }
        });
        
        Ok(rx)
    }

    /// Stream multiple symbols combined
    /// 
    /// # Arguments
    /// * `streams` - List of stream names (e.g., ["btcusdt@ticker", "ethusdt@ticker"])
    /// 
    /// # Example
    /// ```no_run
    /// use binance_connector::{BinanceWebSocket, BinanceConfig};
    /// 
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let config = BinanceConfig::new(false);
    ///     let ws = BinanceWebSocket::new(config)?;
    ///     
    ///     let streams = vec!["btcusdt@ticker", "ethusdt@ticker", "bnbusdt@ticker"];
    ///     let mut stream = ws.combined_stream(&streams).await?;
    ///     
    ///     // Handle messages from multiple streams
    ///     while let Some(result) = stream.recv().await {
    ///         match result {
    ///             Ok(msg) => println!("Message: {}", msg),
    ///             Err(e) => eprintln!("Error: {}", e),
    ///         }
    ///     }
    ///     
    ///     Ok(())
    /// }
    /// ```
    pub async fn combined_stream(
        &self,
        streams: &[&str],
    ) -> Result<mpsc::Receiver<Result<String>>> {
        let streams_param = streams.join("/");
        let url = format!("{}/stream?streams={}", self.config.get_ws_url(), streams_param);
        
        let (tx, rx) = mpsc::channel(100);
        
        tokio::spawn(async move {
            if let Err(e) = Self::raw_stream_handler(url, tx.clone()).await {
                let _ = tx.send(Err(e)).await;
            }
        });
        
        Ok(rx)
    }

    // ============================================================
    // PRIVATE STREAM HANDLERS
    // ============================================================

    async fn ticker_stream_handler(
        url: String,
        symbol: String,
        tx: mpsc::Sender<Result<Ticker24h>>,
    ) -> Result<()> {
        loop {
            match Self::connect_with_retry(&url).await {
                Ok(ws_stream) => {
                    if let Err(e) = Self::handle_ticker_messages(ws_stream, &symbol, &tx).await {
                        let _ = tx.send(Err(e)).await;
                    }
                }
                Err(e) => {
                    let _ = tx.send(Err(e)).await;
                }
            }
            
            // Reconnect after delay
            sleep(Duration::from_secs(5)).await;
        }
    }

    async fn handle_ticker_messages(
        mut ws_stream: WsStream,
        symbol: &str,
        tx: &mpsc::Sender<Result<Ticker24h>>,
    ) -> Result<()> {
        while let Some(msg) = ws_stream.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    match serde_json::from_str::<WsTickerData>(&text) {
                        Ok(data) => {
                            let ticker = data.to_ticker24h()?;
                            if tx.send(Ok(ticker)).await.is_err() {
                                return Ok(()); // Channel closed
                            }
                        }
                        Err(e) => {
                            let _ = tx.send(Err(Error::DeserializationError(e.to_string()))).await;
                        }
                    }
                }
                Ok(Message::Ping(data)) => {
                    ws_stream.send(Message::Pong(data)).await
                        .map_err(|e| Error::WebSocketError(e.to_string()))?;
                }
                Ok(Message::Close(_)) => {
                    return Err(Error::WebSocketClosed);
                }
                Err(e) => {
                    return Err(Error::WebSocketError(e.to_string()));
                }
                _ => {}
            }
        }
        
        Err(Error::WebSocketClosed)
    }

    async fn kline_stream_handler(
        url: String,
        symbol: String,
        tx: mpsc::Sender<Result<Kline>>,
    ) -> Result<()> {
        loop {
            match Self::connect_with_retry(&url).await {
                Ok(ws_stream) => {
                    if let Err(e) = Self::handle_kline_messages(ws_stream, &symbol, &tx).await {
                        let _ = tx.send(Err(e)).await;
                    }
                }
                Err(e) => {
                    let _ = tx.send(Err(e)).await;
                }
            }
            
            sleep(Duration::from_secs(5)).await;
        }
    }

    async fn handle_kline_messages(
        mut ws_stream: WsStream,
        symbol: &str,
        tx: &mpsc::Sender<Result<Kline>>,
    ) -> Result<()> {
        while let Some(msg) = ws_stream.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    match serde_json::from_str::<WsKlineData>(&text) {
                        Ok(data) => {
                            let kline = data.to_kline(symbol.to_string())?;
                            if tx.send(Ok(kline)).await.is_err() {
                                return Ok(());
                            }
                        }
                        Err(e) => {
                            let _ = tx.send(Err(Error::DeserializationError(e.to_string()))).await;
                        }
                    }
                }
                Ok(Message::Ping(data)) => {
                    ws_stream.send(Message::Pong(data)).await
                        .map_err(|e| Error::WebSocketError(e.to_string()))?;
                }
                Ok(Message::Close(_)) => {
                    return Err(Error::WebSocketClosed);
                }
                Err(e) => {
                    return Err(Error::WebSocketError(e.to_string()));
                }
                _ => {}
            }
        }
        
        Err(Error::WebSocketClosed)
    }

    async fn trade_stream_handler(
        url: String,
        symbol: String,
        tx: mpsc::Sender<Result<Trade>>,
    ) -> Result<()> {
        loop {
            match Self::connect_with_retry(&url).await {
                Ok(ws_stream) => {
                    if let Err(e) = Self::handle_trade_messages(ws_stream, &symbol, &tx).await {
                        let _ = tx.send(Err(e)).await;
                    }
                }
                Err(e) => {
                    let _ = tx.send(Err(e)).await;
                }
            }
            
            sleep(Duration::from_secs(5)).await;
        }
    }

    async fn handle_trade_messages(
        mut ws_stream: WsStream,
        symbol: &str,
        tx: &mpsc::Sender<Result<Trade>>,
    ) -> Result<()> {
        while let Some(msg) = ws_stream.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    match serde_json::from_str::<WsTradeData>(&text) {
                        Ok(data) => {
                            let trade = data.to_trade(symbol.to_string())?;
                            if tx.send(Ok(trade)).await.is_err() {
                                return Ok(());
                            }
                        }
                        Err(e) => {
                            let _ = tx.send(Err(Error::DeserializationError(e.to_string()))).await;
                        }
                    }
                }
                Ok(Message::Ping(data)) => {
                    ws_stream.send(Message::Pong(data)).await
                        .map_err(|e| Error::WebSocketError(e.to_string()))?;
                }
                Ok(Message::Close(_)) => {
                    return Err(Error::WebSocketClosed);
                }
                Err(e) => {
                    return Err(Error::WebSocketError(e.to_string()));
                }
                _ => {}
            }
        }
        
        Err(Error::WebSocketClosed)
    }

    async fn depth_stream_handler(
        url: String,
        symbol: String,
        tx: mpsc::Sender<Result<OrderBook>>,
    ) -> Result<()> {
        loop {
            match Self::connect_with_retry(&url).await {
                Ok(ws_stream) => {
                    if let Err(e) = Self::handle_depth_messages(ws_stream, &symbol, &tx).await {
                        let _ = tx.send(Err(e)).await;
                    }
                }
                Err(e) => {
                    let _ = tx.send(Err(e)).await;
                }
            }
            
            sleep(Duration::from_secs(5)).await;
        }
    }

    async fn handle_depth_messages(
        mut ws_stream: WsStream,
        symbol: &str,
        tx: &mpsc::Sender<Result<OrderBook>>,
    ) -> Result<()> {
        while let Some(msg) = ws_stream.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    match serde_json::from_str::<WsDepthData>(&text) {
                        Ok(data) => {
                            let order_book = data.to_order_book(symbol.to_string())?;
                            if tx.send(Ok(order_book)).await.is_err() {
                                return Ok(());
                            }
                        }
                        Err(e) => {
                            let _ = tx.send(Err(Error::DeserializationError(e.to_string()))).await;
                        }
                    }
                }
                Ok(Message::Ping(data)) => {
                    ws_stream.send(Message::Pong(data)).await
                        .map_err(|e| Error::WebSocketError(e.to_string()))?;
                }
                Ok(Message::Close(_)) => {
                    return Err(Error::WebSocketClosed);
                }
                Err(e) => {
                    return Err(Error::WebSocketError(e.to_string()));
                }
                _ => {}
            }
        }
        
        Err(Error::WebSocketClosed)
    }

    async fn mini_ticker_stream_handler(
        url: String,
        symbol: String,
        tx: mpsc::Sender<Result<Ticker>>,
    ) -> Result<()> {
        loop {
            match Self::connect_with_retry(&url).await {
                Ok(ws_stream) => {
                    if let Err(e) = Self::handle_mini_ticker_messages(ws_stream, &symbol, &tx).await {
                        let _ = tx.send(Err(e)).await;
                    }
                }
                Err(e) => {
                    let _ = tx.send(Err(e)).await;
                }
            }
            
            sleep(Duration::from_secs(5)).await;
        }
    }

    async fn handle_mini_ticker_messages(
        mut ws_stream: WsStream,
        symbol: &str,
        tx: &mpsc::Sender<Result<Ticker>>,
    ) -> Result<()> {
        while let Some(msg) = ws_stream.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    match serde_json::from_str::<WsMiniTickerData>(&text) {
                        Ok(data) => {
                            let ticker = data.to_ticker();
                            if tx.send(Ok(ticker)).await.is_err() {
                                return Ok(());
                            }
                        }
                        Err(e) => {
                            let _ = tx.send(Err(Error::DeserializationError(e.to_string()))).await;
                        }
                    }
                }
                Ok(Message::Ping(data)) => {
                    ws_stream.send(Message::Pong(data)).await
                        .map_err(|e| Error::WebSocketError(e.to_string()))?;
                }
                Ok(Message::Close(_)) => {
                    return Err(Error::WebSocketClosed);
                }
                Err(e) => {
                    return Err(Error::WebSocketError(e.to_string()));
                }
                _ => {}
            }
        }
        
        Err(Error::WebSocketClosed)
    }

    async fn raw_stream_handler(
        url: String,
        tx: mpsc::Sender<Result<String>>,
    ) -> Result<()> {
        loop {
            match Self::connect_with_retry(&url).await {
                Ok(mut ws_stream) => {
                    while let Some(msg) = ws_stream.next().await {
                        match msg {
                            Ok(Message::Text(text)) => {
                                if tx.send(Ok(text)).await.is_err() {
                                    return Ok(());
                                }
                            }
                            Ok(Message::Ping(data)) => {
                                ws_stream.send(Message::Pong(data)).await
                                    .map_err(|e| Error::WebSocketError(e.to_string()))?;
                            }
                            Ok(Message::Close(_)) => {
                                let _ = tx.send(Err(Error::WebSocketClosed)).await;
                                break;
                            }
                            Err(e) => {
                                let _ = tx.send(Err(Error::WebSocketError(e.to_string()))).await;
                                break;
                            }
                            _ => {}
                        }
                    }
                }
                Err(e) => {
                    let _ = tx.send(Err(e)).await;
                }
            }
            
            sleep(Duration::from_secs(5)).await;
        }
    }

    // ============================================================
    // CONNECTION HELPERS
    // ============================================================

    async fn connect_with_retry(url: &str) -> Result<WsStream> {
        let max_retries = 5;
        let mut attempts = 0;
        
        loop {
            attempts += 1;
            
            match connect_async(url).await {
                Ok((ws_stream, _)) => return Ok(ws_stream),
                Err(e) if attempts >= max_retries => {
                    return Err(Error::WebSocketError(format!(
                        "Failed to connect after {} attempts: {}",
                        max_retries, e
                    )));
                }
                Err(_) => {
                    let delay = Duration::from_secs(2u64.pow(attempts - 1));
                    sleep(delay).await;
                }
            }
        }
    }
}

// ============================================================
// WEBSOCKET DATA STRUCTURES
// ============================================================

#[derive(Debug, Deserialize)]
struct WsTickerData {
    #[serde(rename = "e")]
    event_type: String,
    #[serde(rename = "s")]
    symbol: String,
    #[serde(rename = "p")]
    price_change: String,
    #[serde(rename = "P")]
    price_change_percent: String,
    #[serde(rename = "w")]
    weighted_avg_price: String,
    #[serde(rename = "x")]
    prev_close: String,
    #[serde(rename = "c")]
    last_price: String,
    #[serde(rename = "b")]
    bid_price: String,
    #[serde(rename = "a")]
    ask_price: String,
    #[serde(rename = "o")]
    open_price: String,
    #[serde(rename = "h")]
    high_price: String,
    #[serde(rename = "l")]
    low_price: String,
    #[serde(rename = "v")]
    volume: String,
    #[serde(rename = "q")]
    quote_volume: String,
    #[serde(rename = "O")]
    open_time: i64,
    #[serde(rename = "C")]
    close_time: i64,
    #[serde(rename = "F")]
    first_trade_id: i64,
    #[serde(rename = "L")]
    last_trade_id: i64,
    #[serde(rename = "n")]
    trade_count: i64,
}

impl WsTickerData {
    fn to_ticker24h(&self) -> Result<Ticker24h> {
        Ok(Ticker24h {
            symbol: self.symbol.clone(),
            price_change: self.price_change.parse().unwrap_or(0.0),
            price_change_percent: self.price_change_percent.parse().unwrap_or(0.0),
            weighted_avg_price: self.weighted_avg_price.parse().unwrap_or(0.0),
            prev_close_price: self.prev_close.parse().unwrap_or(0.0),
            last_price: self.last_price.parse().unwrap_or(0.0),
            bid_price: self.bid_price.parse().unwrap_or(0.0),
            ask_price: self.ask_price.parse().unwrap_or(0.0),
            open_price: self.open_price.parse().unwrap_or(0.0),
            high_price: self.high_price.parse().unwrap_or(0.0),
            low_price: self.low_price.parse().unwrap_or(0.0),
            volume: self.volume.parse().unwrap_or(0.0),
            quote_volume: self.quote_volume.parse().unwrap_or(0.0),
            open_time: DateTime::from_timestamp_millis(self.open_time).unwrap_or_default(),
            close_time: DateTime::from_timestamp_millis(self.close_time).unwrap_or_default(),
            first_id: self.first_trade_id,
            last_id: self.last_trade_id,
            count: self.trade_count,
        })
    }
}

#[derive(Debug, Deserialize)]
struct WsKlineData {
    #[serde(rename = "e")]
    event_type: String,
    #[serde(rename = "s")]
    symbol: String,
    #[serde(rename = "k")]
    kline: WsKline,
}

#[derive(Debug, Deserialize)]
struct WsKline {
    #[serde(rename = "t")]
    open_time: i64,
    #[serde(rename = "T")]
    close_time: i64,
    #[serde(rename = "o")]
    open: String,
    #[serde(rename = "h")]
    high: String,
    #[serde(rename = "l")]
    low: String,
    #[serde(rename = "c")]
    close: String,
    #[serde(rename = "v")]
    volume: String,
    #[serde(rename = "q")]
    quote_volume: String,
    #[serde(rename = "n")]
    trades: i64,
    #[serde(rename = "V")]
    taker_buy_base: String,
    #[serde(rename = "Q")]
    taker_buy_quote: String,
    #[serde(rename = "x")]
    is_closed: bool,
}

impl WsKlineData {
    fn to_kline(&self, symbol: String) -> Result<Kline> {
        Ok(Kline {
            symbol,
            open_time: DateTime::from_timestamp_millis(self.kline.open_time).unwrap_or_default(),
            close_time: DateTime::from_timestamp_millis(self.kline.close_time).unwrap_or_default(),
            open: self.kline.open.parse().unwrap_or(0.0),
            high: self.kline.high.parse().unwrap_or(0.0),
            low: self.kline.low.parse().unwrap_or(0.0),
            close: self.kline.close.parse().unwrap_or(0.0),
            volume: self.kline.volume.parse().unwrap_or(0.0),
            quote_volume: self.kline.quote_volume.parse().unwrap_or(0.0),
            trades: self.kline.trades,
            taker_buy_base: self.kline.taker_buy_base.parse().unwrap_or(0.0),
            taker_buy_quote: self.kline.taker_buy_quote.parse().unwrap_or(0.0),
            is_closed: self.kline.is_closed,
        })
    }
}

#[derive(Debug, Deserialize)]
struct WsTradeData {
    #[serde(rename = "e")]
    event_type: String,
    #[serde(rename = "t")]
    trade_id: i64,
    #[serde(rename = "p")]
    price: String,
    #[serde(rename = "q")]
    quantity: String,
    #[serde(rename = "T")]
    trade_time: i64,
    #[serde(rename = "m")]
    is_buyer_maker: bool,
}

impl WsTradeData {
    fn to_trade(&self, symbol: String) -> Result<Trade> {
        let price: f64 = self.price.parse().unwrap_or(0.0);
        let quantity: f64 = self.quantity.parse().unwrap_or(0.0);
        
        Ok(Trade {
            id: self.trade_id,
            symbol,
            price,
            quantity,
            quote_quantity: price * quantity,
            time: DateTime::from_timestamp_millis(self.trade_time).unwrap_or_default(),
            is_buyer_maker: self.is_buyer_maker,
        })
    }
}

#[derive(Debug, Deserialize)]
struct WsDepthData {
    #[serde(rename = "e")]
    event_type: String,
    #[serde(rename = "s")]
    symbol: String,
    #[serde(rename = "U")]
    first_update_id: i64,
    #[serde(rename = "u")]
    last_update_id: i64,
    #[serde(rename = "b")]
    bids: Vec<(String, String)>,
    #[serde(rename = "a")]
    asks: Vec<(String, String)>,
}

impl WsDepthData {
    fn to_order_book(&self, symbol: String) -> Result<OrderBook> {
        Ok(OrderBook {
            symbol,
            last_update_id: self.last_update_id,
            bids: self.bids.iter().map(|(p, q)| PriceLevel {
                price: p.parse().unwrap_or(0.0),
                quantity: q.parse().unwrap_or(0.0),
            }).collect(),
            asks: self.asks.iter().map(|(p, q)| PriceLevel {
                price: p.parse().unwrap_or(0.0),
                quantity: q.parse().unwrap_or(0.0),
            }).collect(),
            timestamp: Utc::now(),
        })
    }
}

#[derive(Debug, Deserialize)]
struct WsMiniTickerData {
    #[serde(rename = "e")]
    event_type: String,
    #[serde(rename = "s")]
    symbol: String,
    #[serde(rename = "c")]
    close_price: String,
    #[serde(rename = "E")]
    event_time: i64,
}

impl WsMiniTickerData {
    fn to_ticker(&self) -> Ticker {
        Ticker {
            symbol: self.symbol.clone(),
            price: self.close_price.parse().unwrap_or(0.0),
            timestamp: DateTime::from_timestamp_millis(self.event_time).unwrap_or_default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_websocket_creation() {
        let config = BinanceConfig::new(false);
        let ws = BinanceWebSocket::new(config);
        assert!(ws.is_ok());
    }

    #[tokio::test]
    #[ignore] // Only run manually (connects to real WebSocket)
    async fn test_ticker_stream() {
        let config = BinanceConfig::new(false);
        let ws = BinanceWebSocket::new(config).unwrap();
        
        let mut stream = ws.ticker_stream("BTCUSDT").await.unwrap();
        
        // Get at least one message
        if let Some(result) = stream.recv().await {
            assert!(result.is_ok());
            let ticker = result.unwrap();
            assert_eq!(ticker.symbol, "BTCUSDT");
            assert!(ticker.last_price > 0.0);
        }
    }

    #[tokio::test]
    #[ignore]
    async fn test_kline_stream() {
        let config = BinanceConfig::new(false);
        let ws = BinanceWebSocket::new(config).unwrap();
        
        let mut stream = ws.kline_stream("BTCUSDT", Interval::Minutes1).await.unwrap();
        
        if let Some(result) = stream.recv().await {
            assert!(result.is_ok());
            let kline = result.unwrap();
            assert_eq!(kline.symbol, "BTCUSDT");
            assert!(kline.open > 0.0);
        }
    }
}