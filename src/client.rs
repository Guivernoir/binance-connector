//! Binance REST API client implementation

use crate::{
    config::BinanceConfig,
    endpoints::Endpoints,
    error::{Error, Result},
    models::*,
    rate_limiter::RateLimiter,
};
use reqwest::{Client as HttpClient, Response, StatusCode};
use std::sync::Arc;
use tokio::time::{sleep, Duration};

/// Binance API client
#[derive(Clone)]
pub struct BinanceClient {
    http_client: HttpClient,
    config: Arc<BinanceConfig>,
    rate_limiter: Arc<RateLimiter>,
}

impl BinanceClient {
    /// Create new Binance client
    pub fn new(config: BinanceConfig) -> Result<Self> {
        config.validate()?;
        
        let http_client = HttpClient::builder()
            .timeout(config.timeout())
            .build()
            .map_err(Error::HttpError)?;
        
        let rate_limiter = Arc::new(RateLimiter::new(config.requests_per_minute));
        
        Ok(Self {
            http_client,
            config: Arc::new(config),
            rate_limiter,
        })
    }
    
    /// Get current price for a symbol
    /// 
    /// # Arguments
    /// * `symbol` - Trading pair symbol (e.g., "BTCUSDT")
    /// 
    /// # Example
    /// ```no_run
    /// use binance_connector::{BinanceClient, BinanceConfig};
    /// 
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let config = BinanceConfig::new(false);
    ///     let client = BinanceClient::new(config)?;
    ///     
    ///     let ticker = client.get_ticker_price("BTCUSDT").await?;
    ///     println!("BTC/USDT: ${}", ticker.price);
    ///     Ok(())
    /// }
    /// ```
    pub async fn get_ticker_price(&self, symbol: &str) -> Result<Ticker> {
        let endpoint = Endpoints::ticker_price();
        let url = format!("{}{}?symbol={}", self.config.get_base_url(), endpoint, symbol);
        
        let response = self.request_with_retry(|| async {
            self.rate_limiter.acquire().await;
            
            self.http_client
                .get(&url)
                .send()
                .await
        }).await?;
        
        let ticker_response: BinanceTickerResponse = self.handle_response(response).await?;
        Ok(ticker_response.to_ticker())
    }
    
    /// Get prices for all symbols
    pub async fn get_all_ticker_prices(&self) -> Result<Vec<Ticker>> {
        let endpoint = Endpoints::ticker_price();
        let url = format!("{}{}", self.config.get_base_url(), endpoint);
        
        let response = self.request_with_retry(|| async {
            self.rate_limiter.acquire().await;
            
            self.http_client
                .get(&url)
                .send()
                .await
        }).await?;
        
        let tickers: Vec<BinanceTickerResponse> = self.handle_response(response).await?;
        Ok(tickers.into_iter().map(|t| t.to_ticker()).collect())
    }
    
    /// Get 24-hour ticker statistics
    /// 
    /// # Arguments
    /// * `symbol` - Trading pair symbol (e.g., "BTCUSDT")
    pub async fn get_ticker_24h(&self, symbol: &str) -> Result<Ticker24h> {
        let endpoint = Endpoints::ticker_24h();
        let url = format!("{}{}?symbol={}", self.config.get_base_url(), endpoint, symbol);
        
        let response = self.request_with_retry(|| async {
            self.rate_limiter.acquire().await;
            
            self.http_client
                .get(&url)
                .send()
                .await
        }).await?;
        
        let ticker_response: Binance24hTickerResponse = self.handle_response(response).await?;
        ticker_response.to_ticker24h()
    }
    
    /// Get klines (candlestick data)
    /// 
    /// # Arguments
    /// * `symbol` - Trading pair symbol
    /// * `interval` - Candlestick interval
    /// * `limit` - Number of candles (max 1000, default 500)
    /// 
    /// # Example
    /// ```no_run
    /// use binance_connector::{BinanceClient, BinanceConfig, Interval};
    /// 
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let config = BinanceConfig::new(false);
    ///     let client = BinanceClient::new(config)?;
    ///     
    ///     let klines = client.get_klines("BTCUSDT", Interval::Minutes5, 100).await?;
    ///     println!("Fetched {} candles", klines.len());
    ///     Ok(())
    /// }
    /// ```
    pub async fn get_klines(
        &self,
        symbol: &str,
        interval: Interval,
        limit: usize,
    ) -> Result<Vec<Kline>> {
        if limit > 1000 {
            return Err(Error::ConfigError(
                format!("Limit {} exceeds maximum of 1000", limit)
            ));
        }
        
        let endpoint = Endpoints::klines();
        let url = format!(
            "{}{}?symbol={}&interval={}&limit={}",
            self.config.get_base_url(),
            endpoint,
            symbol,
            interval,
            limit
        );
        
        let response = self.request_with_retry(|| async {
            self.rate_limiter.acquire().await;
            
            self.http_client
                .get(&url)
                .send()
                .await
        }).await?;
        
        let klines_response: Vec<BinanceKlineResponse> = self.handle_response(response).await?;
        
        klines_response
            .into_iter()
            .map(|k| k.to_kline(symbol.to_string()))
            .collect()
    }
    
    /// Get klines with time range
    /// 
    /// # Arguments
    /// * `symbol` - Trading pair symbol
    /// * `interval` - Candlestick interval
    /// * `start_time` - Start time in milliseconds
    /// * `end_time` - End time in milliseconds
    pub async fn get_klines_range(
        &self,
        symbol: &str,
        interval: Interval,
        start_time: i64,
        end_time: i64,
    ) -> Result<Vec<Kline>> {
        let endpoint = Endpoints::klines();
        let url = format!(
            "{}{}?symbol={}&interval={}&startTime={}&endTime={}",
            self.config.get_base_url(),
            endpoint,
            symbol,
            interval,
            start_time,
            end_time
        );
        
        let response = self.request_with_retry(|| async {
            self.rate_limiter.acquire().await;
            
            self.http_client
                .get(&url)
                .send()
                .await
        }).await?;
        
        let klines_response: Vec<BinanceKlineResponse> = self.handle_response(response).await?;
        
        klines_response
            .into_iter()
            .map(|k| k.to_kline(symbol.to_string()))
            .collect()
    }
    
    /// Get order book depth
    /// 
    /// # Arguments
    /// * `symbol` - Trading pair symbol
    /// * `limit` - Depth (valid: 5, 10, 20, 50, 100, 500, 1000, 5000)
    pub async fn get_depth(&self, symbol: &str, limit: usize) -> Result<OrderBook> {
        let endpoint = Endpoints::depth();
        let url = format!(
            "{}{}?symbol={}&limit={}",
            self.config.get_base_url(),
            endpoint,
            symbol,
            limit
        );
        
        let response = self.request_with_retry(|| async {
            self.rate_limiter.acquire().await;
            
            self.http_client
                .get(&url)
                .send()
                .await
        }).await?;
        
        let depth_response: BinanceDepthResponse = self.handle_response(response).await?;
        Ok(depth_response.to_order_book(symbol.to_string()))
    }
    
    /// Get recent trades
    /// 
    /// # Arguments
    /// * `symbol` - Trading pair symbol
    /// * `limit` - Number of trades (max 1000, default 500)
    pub async fn get_recent_trades(&self, symbol: &str, limit: usize) -> Result<Vec<Trade>> {
        let endpoint = Endpoints::trades();
        let url = format!(
            "{}{}?symbol={}&limit={}",
            self.config.get_base_url(),
            endpoint,
            symbol,
            limit
        );
        
        let response = self.request_with_retry(|| async {
            self.rate_limiter.acquire().await;
            
            self.http_client
                .get(&url)
                .send()
                .await
        }).await?;
        
        #[derive(serde::Deserialize)]
        struct TradeResponse {
            id: i64,
            price: String,
            qty: String,
            #[serde(rename = "quoteQty")]
            quote_qty: String,
            time: i64,
            #[serde(rename = "isBuyerMaker")]
            is_buyer_maker: bool,
        }
        
        let trades_response: Vec<TradeResponse> = self.handle_response(response).await?;
        
        Ok(trades_response.into_iter().map(|t| Trade {
            id: t.id,
            symbol: symbol.to_string(),
            price: t.price.parse().unwrap_or(0.0),
            quantity: t.qty.parse().unwrap_or(0.0),
            quote_quantity: t.quote_qty.parse().unwrap_or(0.0),
            time: chrono::DateTime::from_timestamp_millis(t.time)
                .unwrap_or_default(),
            is_buyer_maker: t.is_buyer_maker,
        }).collect())
    }
    
    /// Get exchange information (all symbols)
    pub async fn get_exchange_info(&self) -> Result<Vec<Symbol>> {
        let endpoint = Endpoints::exchange_info();
        let url = format!("{}{}", self.config.get_base_url(), endpoint);
        
        let response = self.request_with_retry(|| async {
            self.rate_limiter.acquire().await;
            
            self.http_client
                .get(&url)
                .send()
                .await
        }).await?;
        
        #[derive(serde::Deserialize)]
        struct ExchangeInfo {
            symbols: Vec<Symbol>,
        }
        
        let info: ExchangeInfo = self.handle_response(response).await?;
        Ok(info.symbols)
    }
    
    /// Get server time
    pub async fn get_server_time(&self) -> Result<i64> {
        let endpoint = Endpoints::time();
        let url = format!("{}{}", self.config.get_base_url(), endpoint);
        
        let response = self.request_with_retry(|| async {
            self.rate_limiter.acquire().await;
            
            self.http_client
                .get(&url)
                .send()
                .await
        }).await?;
        
        #[derive(serde::Deserialize)]
        struct ServerTime {
            #[serde(rename = "serverTime")]
            server_time: i64,
        }
        
        let time: ServerTime = self.handle_response(response).await?;
        Ok(time.server_time)
    }
    
    /// Ping the server (health check)
    pub async fn ping(&self) -> Result<bool> {
        let endpoint = Endpoints::ping();
        let url = format!("{}{}", self.config.get_base_url(), endpoint);
        
        let response = self.http_client
            .get(&url)
            .send()
            .await
            .map_err(Error::HttpError)?;
        
        Ok(response.status() == StatusCode::OK)
    }
    
    /// Check if client is connected
    pub async fn health_check(&self) -> Result<bool> {
        self.ping().await
    }
    
    // ============================================================
    // PRIVATE HELPER METHODS
    // ============================================================
    
    /// Make request with automatic retry logic
    async fn request_with_retry<F, Fut>(&self, mut f: F) -> Result<Response>
    where
        F: FnMut() -> Fut,
        Fut: std::future::Future<Output = reqwest::Result<Response>>,
    {
        if !self.config.enable_retries {
            return f().await.map_err(Error::HttpError);
        }
        
        let mut attempts = 0;
        let max_attempts = self.config.max_retries + 1;
        
        loop {
            attempts += 1;
            
            match f().await {
                Ok(response) => return Ok(response),
                Err(e) if attempts >= max_attempts => {
                    return Err(Error::HttpError(e));
                }
                Err(e) if e.is_timeout() => {
                    let delay = Duration::from_millis(100 * 2u64.pow(attempts - 1));
                    sleep(delay).await;
                    continue;
                }
                Err(e) if e.is_connect() => {
                    let delay = Duration::from_millis(500 * 2u64.pow(attempts - 1));
                    sleep(delay).await;
                    continue;
                }
                Err(e) => {
                    return Err(Error::HttpError(e));
                }
            }
        }
    }
    
    /// Handle HTTP response and convert to typed result
    async fn handle_response<T>(&self, response: Response) -> Result<T>
    where
        T: serde::de::DeserializeOwned,
    {
        let status = response.status();
        
        match status {
            StatusCode::OK => {
                response.json::<T>().await.map_err(|e| Error::ApiError {
                    code: 0,
                    msg: format!("Failed to parse response: {}", e),
                })
            }
            StatusCode::BAD_REQUEST => {
                #[derive(serde::Deserialize)]
                struct BinanceError {
                    code: i32,
                    msg: String,
                }
                
                match response.json::<BinanceError>().await {
                    Ok(err) => Err(Error::ApiError {
                        code: err.code,
                        msg: err.msg,
                    }),
                    Err(_) => Err(Error::ApiError {
                        code: 400,
                        msg: "Bad request".to_string(),
                    }),
                }
            }
            StatusCode::TOO_MANY_REQUESTS => {
                let retry_after = response
                    .headers()
                    .get("Retry-After")
                    .and_then(|h| h.to_str().ok())
                    .and_then(|s| s.parse::<u64>().ok())
                    .unwrap_or(60);
                
                Err(Error::RateLimitExceeded {
                    retry_after_seconds: retry_after,
                })
            }
            _ => {
                let error_text = response.text().await.unwrap_or_default();
                Err(Error::ApiError {
                    code: status.as_u16() as i32,
                    msg: error_text,
                })
            }
        }
    }
}

// ============================================================
// BUILDER PATTERN
// ============================================================

/// Builder for BinanceClient
pub struct BinanceClientBuilder {
    config: BinanceConfig,
}

impl BinanceClientBuilder {
    /// Create new builder
    pub fn new(config: BinanceConfig) -> Self {
        Self { config }
    }
    
    /// Set timeout
    pub fn timeout(mut self, seconds: u64) -> Self {
        self.config.timeout_seconds = seconds;
        self
    }
    
    /// Set rate limit
    pub fn rate_limit(mut self, requests_per_minute: u32) -> Self {
        self.config.requests_per_minute = requests_per_minute;
        self
    }
    
    /// Enable/disable retries
    pub fn retries(mut self, enable: bool) -> Self {
        self.config.enable_retries = enable;
        self
    }
    
    /// Set max retry attempts
    pub fn max_retries(mut self, max: u32) -> Self {
        self.config.max_retries = max;
        self
    }
    
    /// Build client
    pub fn build(self) -> Result<BinanceClient> {
        BinanceClient::new(self.config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let config = BinanceConfig::new(false);
        let client = BinanceClient::new(config);
        assert!(client.is_ok());
    }

    #[test]
    fn test_client_builder() {
        let config = BinanceConfig::new(false);
        let client = BinanceClientBuilder::new(config)
            .timeout(20)
            .rate_limit(600)
            .build();
        
        assert!(client.is_ok());
    }
}