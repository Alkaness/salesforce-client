//! Retry logic with exponential backoff
//!
//! Automatically retries failed requests with intelligent backoff strategies.

use crate::error::{SfError, SfResult};
// Retry logic implementation without backoff crate due to lifetime issues
use std::time::Duration;
use tracing::{debug, warn};

/// Configuration for retry behavior
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Maximum number of retry attempts
    pub max_retries: u32,

    /// Initial backoff duration
    pub initial_interval: Duration,

    /// Maximum backoff duration
    pub max_interval: Duration,

    /// Multiplier for exponential backoff
    pub multiplier: f64,

    /// Maximum elapsed time before giving up
    pub max_elapsed_time: Option<Duration>,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_interval: Duration::from_millis(500),
            max_interval: Duration::from_secs(30),
            multiplier: 2.0,
            max_elapsed_time: Some(Duration::from_secs(300)), // 5 minutes
        }
    }
}

impl RetryConfig {
    /// Create a new retry config with defaults
    pub fn new() -> Self {
        Self::default()
    }

    /// Set maximum number of retries
    pub fn max_retries(mut self, max: u32) -> Self {
        self.max_retries = max;
        self
    }

    /// Set initial backoff interval
    pub fn initial_interval(mut self, duration: Duration) -> Self {
        self.initial_interval = duration;
        self
    }

    /// Set maximum backoff interval
    pub fn max_interval(mut self, duration: Duration) -> Self {
        self.max_interval = duration;
        self
    }

    /// Disable retry (for testing)
    pub fn no_retry() -> Self {
        Self {
            max_retries: 0,
            ..Default::default()
        }
    }
}

/// Determines if an error is retryable
pub(crate) fn is_retryable(error: &SfError) -> bool {
    match error {
        // Network errors are retryable
        SfError::Network(_) => true,

        // Rate limits are retryable (with backoff)
        SfError::RateLimit { .. } => true,

        // Timeout is retryable
        SfError::Timeout { .. } => true,

        // API errors: only retry on specific status codes
        SfError::Api { status, .. } => {
            matches!(
                *status,
                // 408 Request Timeout
                408 |
                // 429 Too Many Requests
                429 |
                // 500 Internal Server Error
                500 |
                // 502 Bad Gateway
                502 |
                // 503 Service Unavailable
                503 |
                // 504 Gateway Timeout
                504
            )
        }

        // Other errors are not retryable
        _ => false,
    }
}

/// Execute an async operation with retry logic
///
/// # Example
/// ```ignore
/// let result = with_retry(config, || async {
///     client.query::<Account>("SELECT Id FROM Account").await
/// }).await?;
/// ```
pub async fn with_retry<F, Fut, T>(config: &RetryConfig, operation: F) -> SfResult<T>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = SfResult<T>>,
{
    if config.max_retries == 0 {
        // No retry, execute once
        return operation().await;
    }

    let mut attempt = 0;
    let mut delay = config.initial_interval;

    loop {
        attempt += 1;

        match operation().await {
            Ok(result) => {
                if attempt > 1 {
                    debug!("Operation succeeded after {} attempts", attempt);
                }
                return Ok(result);
            }
            Err(e) => {
                if is_retryable(&e) && attempt <= config.max_retries {
                    warn!(
                        "Attempt {} failed: {}. Retrying in {:?}...",
                        attempt, e, delay
                    );
                    tokio::time::sleep(delay).await;

                    // Exponential backoff
                    delay = Duration::min(
                        Duration::from_secs_f64(delay.as_secs_f64() * config.multiplier),
                        config.max_interval,
                    );
                } else {
                    if attempt > config.max_retries {
                        warn!("Max retries ({}) exceeded. Giving up.", config.max_retries);
                    } else {
                        debug!("Error is not retryable: {}", e);
                    }
                    return Err(e);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_retry_config_builder() {
        let config = RetryConfig::new()
            .max_retries(5)
            .initial_interval(Duration::from_millis(100));

        assert_eq!(config.max_retries, 5);
        assert_eq!(config.initial_interval, Duration::from_millis(100));
    }

    #[test]
    fn test_is_retryable() {
        // Retryable errors
        assert!(is_retryable(&SfError::RateLimit { retry_after: None }));
        assert!(is_retryable(&SfError::Timeout { seconds: 30 }));
        assert!(is_retryable(&SfError::Api {
            status: 503,
            body: "Service Unavailable".to_string()
        }));

        // Non-retryable errors
        assert!(!is_retryable(&SfError::Auth("Invalid token".to_string())));
        assert!(!is_retryable(&SfError::NotFound {
            sobject: "Account".to_string(),
            id: "123".to_string()
        }));
        assert!(!is_retryable(&SfError::Api {
            status: 400,
            body: "Bad Request".to_string()
        }));
    }

    #[tokio::test]
    async fn test_with_retry_success() {
        let config = RetryConfig::no_retry();

        let result = with_retry(&config, || async { Ok::<i32, SfError>(42) }).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
    }

    #[tokio::test]
    async fn test_with_retry_non_retryable_error() {
        let config = RetryConfig::new().max_retries(3);

        let result = with_retry(&config, || async {
            Err::<i32, SfError>(SfError::Auth("Invalid token".to_string()))
        })
        .await;

        assert!(result.is_err());
        // Should not retry non-retryable errors
    }
}
