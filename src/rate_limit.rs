//! Rate limiting to respect Salesforce API limits
//!
//! Prevents exceeding API rate limits and handles 429 responses gracefully.

use crate::error::{SfError, SfResult};
use governor::{Quota, RateLimiter as GovernorRateLimiter};
use std::num::NonZeroU32;
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, warn};

/// Configuration for rate limiting
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    /// Maximum requests per second
    pub requests_per_second: u32,

    /// Burst capacity (how many requests can be made at once)
    pub burst_size: u32,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        // Salesforce default: 100 API calls per 20 seconds per user
        // Conservative default: 4 requests per second
        Self {
            requests_per_second: 4,
            burst_size: 10,
        }
    }
}

impl RateLimitConfig {
    /// Create a new rate limit config with defaults
    pub fn new() -> Self {
        Self::default()
    }

    /// Set requests per second
    pub fn requests_per_second(mut self, rps: u32) -> Self {
        self.requests_per_second = rps;
        self
    }

    /// Set burst size
    pub fn burst_size(mut self, size: u32) -> Self {
        self.burst_size = size;
        self
    }

    /// No rate limiting (for testing or when using a dedicated API user)
    pub fn unlimited() -> Self {
        Self {
            requests_per_second: u32::MAX,
            burst_size: u32::MAX,
        }
    }
}

/// Rate limiter wrapper
pub struct RateLimiter {
    limiter: Arc<
        GovernorRateLimiter<
            governor::state::NotKeyed,
            governor::state::InMemoryState,
            governor::clock::DefaultClock,
        >,
    >,
    enabled: bool,
}

impl RateLimiter {
    /// Create a new rate limiter
    pub fn new(config: RateLimitConfig) -> Self {
        let enabled = config.requests_per_second < u32::MAX;

        if !enabled {
            debug!("Rate limiting disabled");
            return Self {
                limiter: Arc::new(GovernorRateLimiter::direct(Quota::per_second(
                    NonZeroU32::new(1).unwrap(),
                ))),
                enabled: false,
            };
        }

        // Create quota: X requests per second with burst capacity
        let quota = Quota::per_second(
            NonZeroU32::new(config.requests_per_second).unwrap_or(NonZeroU32::new(1).unwrap()),
        )
        .allow_burst(NonZeroU32::new(config.burst_size).unwrap_or(NonZeroU32::new(1).unwrap()));

        let limiter = GovernorRateLimiter::direct(quota);

        debug!(
            "Rate limiter initialized: {} req/s, burst {}",
            config.requests_per_second, config.burst_size
        );

        Self {
            limiter: Arc::new(limiter),
            enabled: true,
        }
    }

    /// Wait until a request can be made
    ///
    /// This method blocks (async) until the rate limit allows another request.
    pub async fn acquire(&self) -> SfResult<()> {
        if !self.enabled {
            return Ok(());
        }

        // until_ready() returns InsufficientCapacity if it fails
        self.limiter.until_ready().await;
        debug!("Rate limit check passed");
        Ok(())
    }

    /// Try to acquire without waiting
    ///
    /// Returns an error if rate limit is exceeded.
    pub fn try_acquire(&self) -> SfResult<()> {
        if !self.enabled {
            return Ok(());
        }

        match self.limiter.check() {
            Ok(_) => Ok(()),
            Err(not_until) => {
                let wait_time = not_until.wait_time_from(governor::clock::Clock::now(
                    &governor::clock::DefaultClock::default(),
                ));

                warn!("Rate limit exceeded, need to wait {:?}", wait_time);

                Err(SfError::RateLimit {
                    retry_after: Some(wait_time.as_secs()),
                })
            }
        }
    }

    /// Get current rate limit status
    pub fn status(&self) -> RateLimitStatus {
        if !self.enabled {
            return RateLimitStatus {
                available: true,
                wait_time: None,
            };
        }

        match self.limiter.check() {
            Ok(_) => RateLimitStatus {
                available: true,
                wait_time: None,
            },
            Err(not_until) => {
                let wait_time = not_until.wait_time_from(governor::clock::Clock::now(
                    &governor::clock::DefaultClock::default(),
                ));

                RateLimitStatus {
                    available: false,
                    wait_time: Some(wait_time),
                }
            }
        }
    }
}

/// Rate limit status information
#[derive(Debug, Clone)]
pub struct RateLimitStatus {
    /// Whether a request can be made immediately
    pub available: bool,

    /// Time to wait before next request (if not available)
    pub wait_time: Option<Duration>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limit_config() {
        let config = RateLimitConfig::new()
            .requests_per_second(10)
            .burst_size(20);

        assert_eq!(config.requests_per_second, 10);
        assert_eq!(config.burst_size, 20);
    }

    #[tokio::test]
    async fn test_rate_limiter_acquire() {
        let config = RateLimitConfig::new()
            .requests_per_second(100) // High limit for test
            .burst_size(10);

        let limiter = RateLimiter::new(config);

        // Should succeed immediately
        assert!(limiter.acquire().await.is_ok());
    }

    #[test]
    fn test_rate_limiter_disabled() {
        let config = RateLimitConfig::unlimited();
        let limiter = RateLimiter::new(config);

        assert!(!limiter.enabled);
        assert!(limiter.try_acquire().is_ok());
    }
}
