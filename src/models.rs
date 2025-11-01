//! Data models for Binance API

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// OHLCV candlestick data (called "Kline" in Binance)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Kline {
    pub symbol: String,
    pub open_time: DateTime<Utc>,
    pub close_time: DateTime<Utc>,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
    pub quote_volume: f64,    // Volume in quote asset (e.g., USDT)
    pub trades: i64,          // Number of trades
    pub taker_buy_base: f64,  // Taker buy volume (base)
    pub taker_buy_quote: f64, // Taker buy volume (quote)
    pub is_closed: bool,      // Is this candle finalized?
}

/// Real-time ticker (price info)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Ticker {
    pub symbol: String,
    pub price: f64,
    pub timestamp: DateTime<Utc>,
}

/// 24-hour ticker statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ticker24h {
    pub symbol: String,
    pub price_change: f64,
    pub price_change_percent: f64,
    pub weighted_avg_price: f64,
    pub prev_close_price: f64,
    pub last_price: f64,
    pub bid_price: f64,
    pub ask_price: f64,
    pub open_price: f64,
    pub high_price: f64,
    pub low_price: f64,
    pub volume: f64,
    pub quote_volume: f64,
    pub open_time: DateTime<Utc>,
    pub close_time: DateTime<Utc>,
    pub first_id: i64,
    pub last_id: i64,
    pub count: i64,
}

impl Ticker24h {
    pub fn spread(&self) -> f64 {
        self.ask_price - self.bid_price
    }

    pub fn mid(&self) -> f64 {
        (self.bid_price + self.ask_price) / 2.0
    }
}

/// Order book (market depth)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderBook {
    pub symbol: String,
    pub last_update_id: i64,
    pub bids: Vec<PriceLevel>,
    pub asks: Vec<PriceLevel>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceLevel {
    pub price: f64,
    pub quantity: f64,
}

/// Recent trade
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trade {
    pub id: i64,
    pub symbol: String,
    pub price: f64,
    pub quantity: f64,
    pub quote_quantity: f64,
    pub time: DateTime<Utc>,
    pub is_buyer_maker: bool,
}

/// Symbol information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Symbol {
    pub symbol: String,
    pub status: String,
    pub base_asset: String,
    pub quote_asset: String,
    pub base_asset_precision: i32,
    pub quote_asset_precision: i32,
    pub order_types: Vec<String>,
}

/// Candlestick interval
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum Interval {
    #[serde(rename = "1s")]
    Seconds1,
    #[serde(rename = "1m")]
    Minutes1,
    #[serde(rename = "3m")]
    Minutes3,
    #[serde(rename = "5m")]
    Minutes5,
    #[serde(rename = "15m")]
    Minutes15,
    #[serde(rename = "30m")]
    Minutes30,
    #[serde(rename = "1h")]
    Hours1,
    #[serde(rename = "2h")]
    Hours2,
    #[serde(rename = "4h")]
    Hours4,
    #[serde(rename = "6h")]
    Hours6,
    #[serde(rename = "8h")]
    Hours8,
    #[serde(rename = "12h")]
    Hours12,
    #[serde(rename = "1d")]
    Days1,
    #[serde(rename = "3d")]
    Days3,
    #[serde(rename = "1w")]
    Weeks1,
    #[serde(rename = "1M")]
    Months1,
}

impl Interval {
    /// Get duration in milliseconds
    pub fn duration_ms(&self) -> i64 {
        match self {
            Interval::Seconds1 => 1_000,
            Interval::Minutes1 => 60_000,
            Interval::Minutes3 => 180_000,
            Interval::Minutes5 => 300_000,
            Interval::Minutes15 => 900_000,
            Interval::Minutes30 => 1_800_000,
            Interval::Hours1 => 3_600_000,
            Interval::Hours2 => 7_200_000,
            Interval::Hours4 => 14_400_000,
            Interval::Hours6 => 21_600_000,
            Interval::Hours8 => 28_800_000,
            Interval::Hours12 => 43_200_000,
            Interval::Days1 => 86_400_000,
            Interval::Days3 => 259_200_000,
            Interval::Weeks1 => 604_800_000,
            Interval::Months1 => 2_592_000_000,
        }
    }
}

impl std::fmt::Display for Interval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Interval::Seconds1 => "1s",
            Interval::Minutes1 => "1m",
            Interval::Minutes3 => "3m",
            Interval::Minutes5 => "5m",
            Interval::Minutes15 => "15m",
            Interval::Minutes30 => "30m",
            Interval::Hours1 => "1h",
            Interval::Hours2 => "2h",
            Interval::Hours4 => "4h",
            Interval::Hours6 => "6h",
            Interval::Hours8 => "8h",
            Interval::Hours12 => "12h",
            Interval::Days1 => "1d",
            Interval::Days3 => "3d",
            Interval::Weeks1 => "1w",
            Interval::Months1 => "1M",
        };
        write!(f, "{}", s)
    }
}

impl std::str::FromStr for Interval {
    type Err = crate::error::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "1s" => Ok(Interval::Seconds1),
            "1m" => Ok(Interval::Minutes1),
            "3m" => Ok(Interval::Minutes3),
            "5m" => Ok(Interval::Minutes5),
            "15m" => Ok(Interval::Minutes15),
            "30m" => Ok(Interval::Minutes30),
            "1h" => Ok(Interval::Hours1),
            "2h" => Ok(Interval::Hours2),
            "4h" => Ok(Interval::Hours4),
            "6h" => Ok(Interval::Hours6),
            "8h" => Ok(Interval::Hours8),
            "12h" => Ok(Interval::Hours12),
            "1d" => Ok(Interval::Days1),
            "3d" => Ok(Interval::Days3),
            "1w" => Ok(Interval::Weeks1),
            "1M" => Ok(Interval::Months1),
            _ => Err(crate::Error::InvalidInterval(s.to_string())),
        }
    }
}

// Internal Binance API response structures
#[derive(Debug, Deserialize)]
pub(crate) struct BinanceKlineResponse(
    pub i64,    // Open time
    pub String, // Open
    pub String, // High
    pub String, // Low
    pub String, // Close
    pub String, // Volume
    pub i64,    // Close time
    pub String, // Quote asset volume
    pub i64,    // Number of trades
    pub String, // Taker buy base asset volume
    pub String, // Taker buy quote asset volume
    pub String, // Ignore
);

impl BinanceKlineResponse {
    pub(crate) fn to_kline(&self, symbol: String) -> crate::Result<Kline> {
        Ok(Kline {
            symbol,
            open_time: DateTime::from_timestamp_millis(self.0).ok_or_else(|| {
                crate::Error::DeserializationError("Invalid open time".to_string())
            })?,
            close_time: DateTime::from_timestamp_millis(self.6).ok_or_else(|| {
                crate::Error::DeserializationError("Invalid close time".to_string())
            })?,
            open: self.1.parse().unwrap_or(0.0),
            high: self.2.parse().unwrap_or(0.0),
            low: self.3.parse().unwrap_or(0.0),
            close: self.4.parse().unwrap_or(0.0),
            volume: self.5.parse().unwrap_or(0.0),
            quote_volume: self.7.parse().unwrap_or(0.0),
            trades: self.8,
            taker_buy_base: self.9.parse().unwrap_or(0.0),
            taker_buy_quote: self.10.parse().unwrap_or(0.0),
            is_closed: true,
        })
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct BinanceTickerResponse {
    pub symbol: String,
    pub price: String,
}

impl BinanceTickerResponse {
    pub(crate) fn to_ticker(&self) -> Ticker {
        Ticker {
            symbol: self.symbol.clone(),
            price: self.price.parse().unwrap_or(0.0),
            timestamp: Utc::now(),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Binance24hTickerResponse {
    pub symbol: String,
    pub price_change: String,
    pub price_change_percent: String,
    pub weighted_avg_price: String,
    pub prev_close_price: String,
    pub last_price: String,
    pub bid_price: String,
    pub ask_price: String,
    pub open_price: String,
    pub high_price: String,
    pub low_price: String,
    pub volume: String,
    pub quote_volume: String,
    pub open_time: i64,
    pub close_time: i64,
    pub first_id: i64,
    pub last_id: i64,
    pub count: i64,
}

impl Binance24hTickerResponse {
    pub(crate) fn to_ticker24h(&self) -> crate::Result<Ticker24h> {
        Ok(Ticker24h {
            symbol: self.symbol.clone(),
            price_change: self.price_change.parse().unwrap_or(0.0),
            price_change_percent: self.price_change_percent.parse().unwrap_or(0.0),
            weighted_avg_price: self.weighted_avg_price.parse().unwrap_or(0.0),
            prev_close_price: self.prev_close_price.parse().unwrap_or(0.0),
            last_price: self.last_price.parse().unwrap_or(0.0),
            bid_price: self.bid_price.parse().unwrap_or(0.0),
            ask_price: self.ask_price.parse().unwrap_or(0.0),
            open_price: self.open_price.parse().unwrap_or(0.0),
            high_price: self.high_price.parse().unwrap_or(0.0),
            low_price: self.low_price.parse().unwrap_or(0.0),
            volume: self.volume.parse().unwrap_or(0.0),
            quote_volume: self.quote_volume.parse().unwrap_or(0.0),
            open_time: DateTime::from_timestamp_millis(self.open_time).ok_or_else(|| {
                crate::Error::DeserializationError("Invalid open time".to_string())
            })?,
            close_time: DateTime::from_timestamp_millis(self.close_time).ok_or_else(|| {
                crate::Error::DeserializationError("Invalid close time".to_string())
            })?,
            first_id: self.first_id,
            last_id: self.last_id,
            count: self.count,
        })
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct BinanceDepthResponse {
    #[serde(rename = "lastUpdateId")]
    pub last_update_id: i64,
    pub bids: Vec<(String, String)>,
    pub asks: Vec<(String, String)>,
}

impl BinanceDepthResponse {
    pub(crate) fn to_order_book(&self, symbol: String) -> OrderBook {
        OrderBook {
            symbol,
            last_update_id: self.last_update_id,
            bids: self
                .bids
                .iter()
                .map(|(p, q)| PriceLevel {
                    price: p.parse().unwrap_or(0.0),
                    quantity: q.parse().unwrap_or(0.0),
                })
                .collect(),
            asks: self
                .asks
                .iter()
                .map(|(p, q)| PriceLevel {
                    price: p.parse().unwrap_or(0.0),
                    quantity: q.parse().unwrap_or(0.0),
                })
                .collect(),
            timestamp: Utc::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interval_from_str() {
        assert_eq!("1m".parse::<Interval>().unwrap(), Interval::Minutes1);
        assert_eq!("1h".parse::<Interval>().unwrap(), Interval::Hours1);
        assert!("invalid".parse::<Interval>().is_err());
    }

    #[test]
    fn test_interval_duration() {
        assert_eq!(Interval::Minutes1.duration_ms(), 60_000);
        assert_eq!(Interval::Hours1.duration_ms(), 3_600_000);
    }

    #[test]
    fn test_ticker24h_calculations() {
        let ticker = Ticker24h {
            symbol: "BTCUSDT".to_string(),
            price_change: 1000.0,
            price_change_percent: 2.5,
            weighted_avg_price: 43000.0,
            prev_close_price: 42000.0,
            last_price: 43000.0,
            bid_price: 42999.0,
            ask_price: 43001.0,
            open_price: 42000.0,
            high_price: 43500.0,
            low_price: 41500.0,
            volume: 1000.0,
            quote_volume: 43_000_000.0,
            open_time: Utc::now(),
            close_time: Utc::now(),
            first_id: 1,
            last_id: 1000,
            count: 1000,
        };

        assert_eq!(ticker.spread(), 2.0);
        assert_eq!(ticker.mid(), 43000.0);
    }
}
