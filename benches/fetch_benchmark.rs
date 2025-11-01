//! Benchmark for Binance connector performance
//! 
//! Run with: cargo bench

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use binance_connector::{BinanceClient, BinanceConfig, Interval};
use tokio::runtime::Runtime;

fn create_client() -> BinanceClient {
    let config = BinanceConfig::new(false);
    BinanceClient::new(config).expect("Failed to create client")
}

fn benchmark_get_ticker_price(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let client = create_client();
    
    c.bench_function("get_ticker_price", |b| {
        b.to_async(&rt).iter(|| async {
            let result = client.get_ticker_price("BTCUSDT").await;
            black_box(result)
        });
    });
}

fn benchmark_get_ticker_24h(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let client = create_client();
    
    c.bench_function("get_ticker_24h", |b| {
        b.to_async(&rt).iter(|| async {
            let result = client.get_ticker_24h("BTCUSDT").await;
            black_box(result)
        });
    });
}

fn benchmark_get_klines(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let client = create_client();
    
    let mut group = c.benchmark_group("get_klines");
    
    for count in [10, 50, 100, 500].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(count),
            count,
            |b, &count| {
                b.to_async(&rt).iter(|| async {
                    let result = client.get_klines("BTCUSDT", Interval::Minutes5, count).await;
                    black_box(result)
                });
            },
        );
    }
    
    group.finish();
}

fn benchmark_get_depth(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let client = create_client();
    
    c.bench_function("get_depth", |b| {
        b.to_async(&rt).iter(|| async {
            let result = client.get_depth("BTCUSDT", 100).await;
            black_box(result)
        });
    });
}

fn benchmark_rate_limiter(c: &mut Criterion) {
    use binance_connector::rate_limiter::RateLimiter;
    let rt = Runtime::new().unwrap();
    
    c.bench_function("rate_limiter_acquire", |b| {
        let limiter = RateLimiter::new(1200);
        
        b.to_async(&rt).iter(|| async {
            let permit = limiter.acquire().await;
            black_box(permit)
        });
    });
}

criterion_group!(
    benches,
    benchmark_get_ticker_price,
    benchmark_get_ticker_24h,
    benchmark_get_klines,
    benchmark_get_depth,
    benchmark_rate_limiter
);
criterion_main!(benches);