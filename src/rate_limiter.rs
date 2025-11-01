//! Rate limiter implementation for Binance API
//! 
//! Binance uses weight-based rate limiting, but we'll start with simple request counting

use std::sync::Arc;
use tokio::sync::Semaphore;
use tokio::time::{sleep, Duration, Instant};

/// Rate limiter (simplified version - Binance uses weight-based)
#[derive(Clone)]
pub struct RateLimiter {
    semaphore: Arc<Semaphore>,
    requests_per_minute: u32,
    refill_interval: Duration,
}

impl RateLimiter {
    /// Create new rate limiter
    /// 
    /// # Arguments
    /// * `requests_per_minute` - Maximum requests allowed per minute
    pub fn new(requests_per_minute: u32) -> Self {
        Self {
            semaphore: Arc::new(Semaphore::new(requests_per_minute as usize)),
            requests_per_minute,
            refill_interval: Duration::from_millis(60_000 / requests_per_minute as u64),
        }
    }
    
    /// Acquire permission to make a request
    pub async fn acquire(&self) -> RateLimitPermit {
        let permit = self.semaphore.clone().acquire_owned().await
            .expect("Semaphore should never be closed");
        
        let semaphore = self.semaphore.clone();
        let refill_interval = self.refill_interval;
        
        tokio::spawn(async move {
            sleep(refill_interval).await;
            drop(permit);
        });
        
        RateLimitPermit {
            _acquired_at: Instant::now(),
        }
    }
    
    /// Try to acquire permission immediately (non-blocking)
    pub fn try_acquire(&self) -> Option<RateLimitPermit> {
        let permit = self.semaphore.clone().try_acquire_owned().ok()?;
        
        let semaphore = self.semaphore.clone();
        let refill_interval = self.refill_interval;
        
        tokio::spawn(async move {
            sleep(refill_interval).await;
            drop(permit);
        });
        
        Some(RateLimitPermit {
            _acquired_at: Instant::now(),
        })
    }
    
    /// Get current available permits
    pub fn available_permits(&self) -> usize {
        self.semaphore.available_permits()
    }
}

/// RAII guard for rate limit permit
pub struct RateLimitPermit {
    _acquired_at: Instant,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_rate_limiter_basic() {
        let limiter = RateLimiter::new(60); // 60 req/min
        
        // Should immediately acquire some permits
        for _ in 0..10 {
            limiter.acquire().await;
        }
        
        assert!(limiter.available_permits() < 60);
    }

    #[tokio::test]
    async fn test_rate_limiter_refill() {
        let limiter = RateLimiter::new(60);
        
        // Exhaust some permits
        for _ in 0..30 {
            limiter.acquire().await;
        }
        
        let initial = limiter.available_permits();
        
        // Wait for some refills
        sleep(Duration::from_millis(2000)).await;
        
        // Should have more permits now
        assert!(limiter.available_permits() > initial);
    }

    #[tokio::test]
    async fn test_try_acquire() {
        let limiter = RateLimiter::new(5);
        
        // Should succeed 5 times
        for _ in 0..5 {
            assert!(limiter.try_acquire().is_some());
        }
        
        // Should fail on 6th attempt
        assert!(limiter.try_acquire().is_none());
    }
}