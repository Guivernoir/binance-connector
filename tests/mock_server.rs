//! Mock server tests (no real API calls needed)

use binance_connector::{BinanceClient, BinanceConfig};
use mockito::{Server, Matcher};

async fn create_mock_client(server: &Server) -> BinanceClient {
    let mut config = BinanceConfig::new(false);
    config.base_url = Some(server.url());
    config.enable_retries = false;
    
    BinanceClient::new(config).unwrap()
}

#[tokio::test]
async fn test_mock_ticker_price() {
    let mut server = Server::new_async().await;
    
    let mock = server.mock("GET", "/api/v3/ticker/price")
        .match_query(Matcher::UrlEncoded("symbol".into(), "BTCUSDT".into()))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{
            "symbol": "BTCUSDT",
            "price": "43250.50"
        }"#)
        .create_async()
        .await;
    
    let client = create_mock_client(&server).await;
    let ticker = client.get_ticker_price("BTCUSDT").await.unwrap();
    
    assert_eq!(ticker.symbol, "BTCUSDT");
    assert_eq!(ticker.price, 43250.50);
    
    mock.assert_async().await;
}

#[tokio::test]
async fn test_mock_24h_ticker() {
    let mut server = Server::new_async().await;
    
    let mock = server.mock("GET", "/api/v3/ticker/24hr")
        .match_query(Matcher::Any)
        .with_status(200)
        .with_body(r#"{
            "symbol": "BTCUSDT",
            "priceChange": "1000.00",
            "priceChangePercent": "2.5",
            "weightedAvgPrice": "43000.00",
            "prevClosePrice": "42000.00",
            "lastPrice": "43000.00",
            "bidPrice": "42999.00",
            "askPrice": "43001.00",
            "openPrice": "42000.00",
            "highPrice": "43500.00",
            "lowPrice": "41500.00",
            "volume": "1000.0",
            "quoteVolume": "43000000.0",
            "openTime": 1640000000000,
            "closeTime": 1640086400000,
            "firstId": 1,
            "lastId": 1000,
            "count": 1000
        }"#)
        .create_async()
        .await;
    
    let client = create_mock_client(&server).await;
    let ticker = client.get_ticker_24h("BTCUSDT").await.unwrap();
    
    assert_eq!(ticker.symbol, "BTCUSDT");
    assert_eq!(ticker.last_price, 43000.0);
    assert_eq!(ticker.price_change_percent, 2.5);
    
    mock.assert_async().await;
}

#[tokio::test]
async fn test_mock_klines() {
    let mut server = Server::new_async().await;
    
    let mock = server.mock("GET", "/api/v3/klines")
        .match_query(Matcher::Any)
        .with_status(200)
        .with_body(r#"[
            [
                1640000000000,
                "43000.00",
                "43100.00",
                "42900.00",
                "43050.00",
                "100.5",
                1640000299999,
                "4320000.00",
                1000,
                "50.25",
                "2160000.00",
                "0"
            ]
        ]"#)
        .create_async()
        .await;
    
    let client = create_mock_client(&server).await;
    let klines = client.get_klines(
        "BTCUSDT",
        binance_connector::Interval::Minutes5,
        1
    ).await.unwrap();
    
    assert_eq!(klines.len(), 1);
    assert_eq!(klines[0].symbol, "BTCUSDT");
    assert_eq!(klines[0].open, 43000.0);
    assert_eq!(klines[0].close, 43050.0);
    
    mock.assert_async().await;
}

#[tokio::test]
async fn test_mock_rate_limit_error() {
    let mut server = Server::new_async().await;
    
    let mock = server.mock("GET", Matcher::Any)
        .with_status(429)
        .with_header("Retry-After", "60")
        .with_body(r#"{"code":-1003,"msg":"Too many requests"}"#)
        .create_async()
        .await;
    
    let client = create_mock_client(&server).await;
    let result = client.get_ticker_price("BTCUSDT").await;
    
    assert!(result.is_err());
    match result.unwrap_err() {
        binance_connector::Error::RateLimitExceeded { retry_after_seconds } => {
            assert_eq!(retry_after_seconds, 60);
        }
        _ => panic!("Expected RateLimitExceeded error"),
    }
    
    mock.assert_async().await;
}

#[tokio::test]
async fn test_mock_invalid_symbol() {
    let mut server = Server::new_async().await;
    
    let mock = server.mock("GET", "/api/v3/ticker/price")
        .match_query(Matcher::UrlEncoded("symbol".into(), "INVALID".into()))
        .with_status(400)
        .with_body(r#"{"code":-1121,"msg":"Invalid symbol."}"#)
        .create_async()
        .await;
    
    let client = create_mock_client(&server).await;
    let result = client.get_ticker_price("INVALID").await;
    
    assert!(result.is_err());
    match result.unwrap_err() {
        binance_connector::Error::ApiError { code, msg } => {
            assert_eq!(code, -1121);
            assert!(msg.contains("Invalid symbol"));
        }
        _ => panic!("Expected ApiError"),
    }
    
    mock.assert_async().await;
}

#[tokio::test]
async fn test_mock_depth() {
    let mut server = Server::new_async().await;
    
    let mock = server.mock("GET", "/api/v3/depth")
        .match_query(Matcher::Any)
        .with_status(200)
        .with_body(r#"{
            "lastUpdateId": 12345,
            "bids": [
                ["43000.00", "1.5"],
                ["42999.00", "2.0"]
            ],
            "asks": [
                ["43001.00", "1.2"],
                ["43002.00", "1.8"]
            ]
        }"#)
        .create_async()
        .await;
    
    let client = create_mock_client(&server).await;
    let order_book = client.get_depth("BTCUSDT", 5).await.unwrap();
    
    assert_eq!(order_book.symbol, "BTCUSDT");
    assert_eq!(order_book.bids.len(), 2);
    assert_eq!(order_book.asks.len(), 2);
    assert_eq!(order_book.bids[0].price, 43000.0);
    assert_eq!(order_book.asks[0].price, 43001.0);
    
    mock.assert_async().await;
}