//! Error types for the Salesforce client
//!
//! Provides comprehensive error handling with detailed context.

use thiserror::Error;

/// Custom error type for Salesforce API operations.
///
/// This enum uses `thiserror` to provide ergonomic error handling with automatic
/// `Display`, `Error`, and `From` implementations.
#[derive(Debug, Error)]
pub enum SfError {
    /// Network-level errors from the HTTP client (connection failures, timeouts, etc.)
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    /// JSON serialization/deserialization errors
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Salesforce API returned a non-success status code
    ///
    /// Includes the status code and response body for debugging
    #[error("API error (status {status}): {body}")]
    Api { status: u16, body: String },

    /// Authentication errors (OAuth, token refresh, etc.)
    #[error("Authentication error: {0}")]
    Auth(String),

    /// Rate limit exceeded
    #[error("Rate limit exceeded. Retry after {retry_after:?} seconds")]
    RateLimit { retry_after: Option<u64> },

    /// Record not found
    #[error("Record not found: {sobject} with id {id}")]
    NotFound { sobject: String, id: String },

    /// Invalid query or SOQL syntax
    #[error("Invalid query: {0}")]
    InvalidQuery(String),

    /// Configuration error
    #[error("Configuration error: {0}")]
    Config(String),

    /// Cache error
    #[error("Cache error: {0}")]
    Cache(String),

    /// Timeout error
    #[error("Operation timed out after {seconds} seconds")]
    Timeout { seconds: u64 },
}

/// Result type alias for Salesforce operations
pub type SfResult<T> = Result<T, SfError>;
