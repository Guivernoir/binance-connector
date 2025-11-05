//! Rate limiter implementation for Binance API using Governor
//! 
//! Binance uses weight-based rate limiting, but this implementation provides
//! simple request-per-minute rate limiting. Weight-based limiting can be added later.

use governor::{
    clock::DefaultClock,
    state::{InMemoryState, NotKeyed},
    Quota, RateLimiter as GovernorRateLimiter,
};
use std::num::NonZeroU32;
use std::sync::Arc;

/// Token bucket rate limiter using Governor's GCRA algorithm
#[derive(Clone)]
pub struct RateLimiter {
    governor: Arc<GovernorRateLimiter<NotKeyed, InMemoryState, DefaultClock>>,
}

impl RateLimiter {
    /// Create new rate limiter
    /// 
    /// # Arguments
    /// * `requests_per_minute` - Maximum requests allowed per minute
    /// 
    /// # Panics
    /// Panics if requests_per_minute is 0
    /// 
    /// # Example
    /// ```
    /// use binance_connector::rate_limiter::RateLimiter;
    /// 
    /// // Binance default: 1200 requests per minute
    /// let limiter = RateLimiter::new(1200);
    /// ```
    pub fn new(requests_per_minute: u32) -> Self {
        let burst: u32 = ((requests_per_minute + 59) / 60).max(1);
        let quota = Quota::per_minute(
            NonZeroU32::new(requests_per_minute)
                .expect("requests_per_minute must be greater than 0")
        ).allow_burst(NonZeroU32::new(burst).expect("Burst must be greater than 0."));
        
        Self {
            governor: Arc::new(GovernorRateLimiter::direct(quota)),
        }
    }
    
    /// Create rate limiter with custom quota per second
    /// 
    /// Useful for stricter local rate limiting or testing.
    /// 
    /// # Arguments
    /// * `requests_per_second` - Maximum requests per second
    pub fn per_second(requests_per_second: u32) -> Self {
        let quota = Quota::per_second(
            NonZeroU32::new(requests_per_second)
                .expect("requests_per_second must be greater than 0")
        );
        
        Self {
            governor: Arc::new(GovernorRateLimiter::direct(quota)),
        }
    }
    
    /// Acquire permission to make a request (async, will wait if needed)
    /// 
    /// Uses GCRA (Generic Cell Rate Algorithm) for smooth rate limiting.
    /// This method blocks until a permit becomes available according to the rate limit.
    /// 
    /// # Example
    /// ```no_run
    /// # use binance_connector::rate_limiter::RateLimiter;
    /// # async fn example() {
    /// let limiter = RateLimiter::new(1200);
    /// 
    /// // This will wait if rate limit is exceeded
    /// limiter.acquire().await;
    /// // Make your API call here
    /// # }
    /// ```
    pub async fn acquire(&self) -> RateLimitPermit {
        // Wait until we're allowed to proceed
        self.governor.until_ready().await;
        
        RateLimitPermit {
            _private: (),
        }
    }
    
    /// Try to acquire permission immediately (non-blocking)
    /// 
    /// Returns Some(permit) if the rate limit allows the request, None if exceeded.
    /// Useful for implementing custom backoff strategies or request queuing.
    /// 
    /// # Example
    /// ```no_run
    /// # use binance_connector::rate_limiter::RateLimiter;
    /// # async fn example() {
    /// let limiter = RateLimiter::new(1200);
    /// 
    /// if let Some(_permit) = limiter.try_acquire() {
    ///     // Rate limit OK, make request
    /// } else {
    ///     // Rate limit exceeded, handle accordingly
    ///     println!("Rate limit exceeded, backing off");
    /// }
    /// # }
    /// ```
    pub fn try_acquire(&self) -> Option<RateLimitPermit> {
        self.governor.check().is_ok().then_some(RateLimitPermit {
            _private: (),
        })
    }
}

/// RAII guard for rate limit permit
/// 
/// Governor handles permit lifecycle internally through GCRA state,
/// so this is primarily a marker type for API consistency and future extensions
/// (e.g., weight-based rate limiting where permit would track consumed weight).
pub struct RateLimitPermit {
    _private: (),
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{sleep, Duration, Instant};

    #[tokio::test]
    async fn test_rate_limiter_basic() {
        let limiter = RateLimiter::new(60); // 60 req/min = 1 req/sec
        
        let start = Instant::now();
        
        // Should immediately acquire up to burst capacity
        for _ in 0..10 {
            limiter.acquire().await;
        }
        
        // Should take around 10 secs
        let elapsed = start.elapsed();
        assert!(elapsed >= Duration::from_millis(8500));
        assert!(elapsed <= Duration::from_secs(11));
    }

    #[tokio::test]
    async fn test_rate_limiter_enforcement() {
        let limiter = RateLimiter::per_second(10); // 10 req/sec for faster testing
        
        let start = Instant::now();
        
        // Make 20 requests (double the per-second rate)
        for _ in 0..20 {
            limiter.acquire().await;
        }
        
        // Should take at least 1 second (20 requests at 10/sec)
        let elapsed = start.elapsed();
        assert!(elapsed >= Duration::from_millis(900)); // Small tolerance
        assert!(elapsed <= Duration::from_millis(2500)); // Upper bound
    }

    #[tokio::test]
    async fn test_try_acquire() {
        let limiter = RateLimiter::per_second(5);
        
        // Should succeed up to burst capacity
        let mut successful = 0;
        for _ in 0..10 {
            if limiter.try_acquire().is_some() {
                successful += 1;
            }
        }
        
        // Should get at least burst capacity (typically 5)
        assert!(successful >= 5);
        assert!(successful < 10); // But not all 10
        
        // Wait for rate window to recover
        sleep(Duration::from_millis(300)).await;
        
        // Should be able to acquire again
        assert!(limiter.try_acquire().is_some());
    }

    #[tokio::test]
    async fn test_rate_limiter_smooth_distribution() {
        let limiter = RateLimiter::per_second(10); // 10 req/sec
        
        let start = Instant::now();
        let mut timestamps = Vec::new();
        
        // Make significantly more requests to exceed burst capacity
        for _ in 0..50 {
            limiter.acquire().await;
            timestamps.push(Instant::now());
        }
        
        let total_duration = start.elapsed();
        
        // 50 requests at 10/sec = ~5 seconds
        // Account for burst capacity (first ~10 are instant)
        assert!(total_duration >= Duration::from_millis(3500));
        assert!(total_duration <= Duration::from_millis(6000));
        
        // Verify smooth distribution after burst
        // Skip first 15 timestamps to get past burst capacity
        if timestamps.len() > 25 {
            let post_burst = &timestamps[15..];
            for window in post_burst.windows(10) {
                let window_duration = window.last().unwrap()
                    .duration_since(*window.first().unwrap());
                
                // 10 requests should take roughly 1 second at 10/sec rate
                // Allow some tolerance for scheduling
                assert!(window_duration <= Duration::from_millis(1500));
            }
        }
    }

    #[tokio::test]
    #[ignore]
    async fn test_per_minute_limiter() {
        // Test with Binance's actual rate limit
        let limiter = RateLimiter::new(1200); // 1200 req/min = 20 req/sec
        
        let start = Instant::now();
        
        // Make significantly more requests than burst capacity
        // to force rate limiting behavior
        for _ in 0..1221 {
            limiter.acquire().await;
        }
        
        let elapsed = start.elapsed();
        
        // 100 requests at 20/sec = ~5 seconds
        // Allow for burst capacity reducing initial delay
        assert!(elapsed >= Duration::from_millis(60000));
    }

    #[tokio::test]
    async fn test_concurrent_access() {
        let limiter = Arc::new(RateLimiter::per_second(10));
        let mut handles = vec![];
        
        // Spawn multiple tasks competing for rate limit
        // Use more requests to exceed burst capacity
        for _ in 0..5 {
            let limiter_clone = Arc::clone(&limiter);
            let handle = tokio::spawn(async move {
                for _ in 0..20 {
                    limiter_clone.acquire().await;
                }
            });
            handles.push(handle);
        }
        
        let start = Instant::now();
        
        // Wait for all to complete
        for handle in handles {
            handle.await.unwrap();
        }
        
        let elapsed = start.elapsed();
        
        // 100 total requests at 10/sec = ~10 seconds
        // Account for burst capacity
        assert!(elapsed >= Duration::from_millis(7000));
        assert!(elapsed <= Duration::from_millis(12000));
    }

    #[test]
    #[should_panic(expected = "requests_per_minute must be greater than 0")]
    fn test_zero_rate_panics() {
        let _ = RateLimiter::new(0);
    }

    #[test]
    #[should_panic(expected = "requests_per_second must be greater than 0")]
    fn test_zero_per_second_panics() {
        let _ = RateLimiter::per_second(0);
    }
}