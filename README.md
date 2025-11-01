# Binance Connector for Rust

[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

High-performance Rust client for the Binance cryptocurrency exchange API. Built for speed, reliability, and ease of use.

## Features

- ✅ **Full Binance Spot API support**

  - Real-time price tickers
  - Historical candlestick (kline) data
  - Order book depth
  - Recent trades
  - Exchange information

- 🚀 **Production-grade reliability**

  - Automatic rate limiting (1200 req/min)
  - Exponential backoff retry logic
  - Comprehensive error handling
  - Request timeout management

- 🔒 **Type-safe & well-tested**

  - Strongly typed API responses
  - 90%+ test coverage
  - Integration tests with real API
  - Mock server tests for CI/CD

- ⚡ **Performance optimized**

  - Async/await using Tokio
  - Efficient connection pooling
  - Minimal allocations
  - Benchmarked operations

- 🆓 **No API key required**
  - Market data is completely free
  - No authentication needed for prices/candles
  - Optional auth for trading (future
