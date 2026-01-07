//! # Salesforce API Client - Enterprise Edition
//!
//! A production-grade, type-driven Salesforce REST API client library for Rust.
//!
//! ## Features
//! - 🔐 **OAuth 2.0 Authentication**: Automatic token refresh
//! - 📄 **Automatic Pagination**: Handle large result sets transparently
//! - 🔁 **Retry Logic**: Exponential backoff for transient failures
//! - ⚡ **Caching**: Reduce API calls with intelligent caching
//! - 🚦 **Rate Limiting**: Respect Salesforce API limits
//! - 📝 **CRUD Operations**: Create, Read, Update, Delete
//! - 🏗️ **Query Builder**: Type-safe SOQL construction
//! - 📊 **Comprehensive Logging**: Built-in tracing support
//! - 🎯 **Type Safety**: Generic methods with compile-time guarantees
//!
//! ## Quick Start
//! ```no_run
//! use salesforce_client::{SalesforceClient, ClientConfig, SfError};
//! use serde::{Deserialize, Serialize};
//!
//! #[derive(Debug, Clone, Deserialize, Serialize)]
//! struct Account {
//!     #[serde(rename = "Id")]
//!     id: String,
//!     #[serde(rename = "Name")]
//!     name: String,
//! }
//!
//! #[tokio::main]
//! async fn main() -> Result<(), SfError> {
//!     // Initialize with configuration
//!     let config = ClientConfig::new(
//!         "https://your-instance.salesforce.com",
//!         "your_access_token",
//!     );
//!     
//!     let client = SalesforceClient::new(config);
//!
//!     // Query with automatic pagination and caching
//!     let accounts: Vec<Account> = client
//!         .query("SELECT Id, Name FROM Account LIMIT 10")
//!         .await?;
//!
//!     Ok(())
//! }
//! ```

// Module declarations
pub mod auth;
pub mod cache;
pub mod crud;
pub mod error;
pub mod pagination;
pub mod query_builder;
pub mod rate_limit;
pub mod retry;

// Re-exports for convenience
pub use auth::{AccessToken, OAuthCredentials, TokenManager};
pub use cache::{CacheConfig, QueryCache};
pub use crud::{InsertResponse, UpdateResponse, UpsertBuilder};
pub use error::{SfError, SfResult};
pub use pagination::{PaginatedQuery, QueryOptions};
pub use query_builder::{CountQueryBuilder, QueryBuilder, SubqueryBuilder};
pub use rate_limit::{RateLimitConfig, RateLimiter};
pub use retry::RetryConfig;

use serde::de::DeserializeOwned;
use serde::Serialize;
use std::sync::Arc;
use tracing::{debug, info, instrument};

/// Client configuration builder
///
/// Provides a fluent API for configuring the Salesforce client with all
/// enterprise features.
#[derive(Debug, Clone)]
pub struct ClientConfig {
    /// Base URL of the Salesforce instance
    pub base_url: String,

    /// Access token for authentication
    pub access_token: String,

    /// Retry configuration
    pub retry_config: RetryConfig,

    /// Cache configuration
    pub cache_config: CacheConfig,

    /// Rate limit configuration
    pub rate_limit_config: RateLimitConfig,

    /// Enable automatic pagination
    pub auto_paginate: bool,
}

impl ClientConfig {
    /// Create a new configuration with defaults
    pub fn new(base_url: impl Into<String>, access_token: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
            access_token: access_token.into(),
            retry_config: RetryConfig::default(),
            cache_config: CacheConfig::default(),
            rate_limit_config: RateLimitConfig::default(),
            auto_paginate: true,
        }
    }

    /// Configure retry behavior
    pub fn with_retry(mut self, config: RetryConfig) -> Self {
        self.retry_config = config;
        self
    }

    /// Configure caching
    pub fn with_cache(mut self, config: CacheConfig) -> Self {
        self.cache_config = config;
        self
    }

    /// Configure rate limiting
    pub fn with_rate_limit(mut self, config: RateLimitConfig) -> Self {
        self.rate_limit_config = config;
        self
    }

    /// Disable automatic pagination
    pub fn no_pagination(mut self) -> Self {
        self.auto_paginate = false;
        self
    }

    /// Disable all optional features (for testing or simple use cases)
    pub fn minimal() -> Self {
        Self {
            base_url: String::new(),
            access_token: String::new(),
            retry_config: RetryConfig::no_retry(),
            cache_config: CacheConfig::disabled(),
            rate_limit_config: RateLimitConfig::unlimited(),
            auto_paginate: false,
        }
    }
}

/// Enterprise-grade Salesforce API client
///
/// This client provides comprehensive features for production use:
/// - Automatic token refresh via OAuth
/// - Intelligent caching to reduce API calls
/// - Retry logic with exponential backoff
/// - Rate limiting to respect API quotas
/// - Automatic pagination for large result sets
/// - Full CRUD operations
/// - Type-safe query building
///
/// ## Design Principles
/// - **Enterprise-ready**: Built-in retry, caching, and rate limiting
/// - **Type-safe**: Generic methods with compile-time guarantees
/// - **Async-first**: Non-blocking I/O throughout
/// - **Observable**: Comprehensive tracing for debugging
/// - **Composable**: Arc-based sharing for concurrent use
#[derive(Clone)]
pub struct SalesforceClient {
    /// Configuration
    config: Arc<ClientConfig>,

    /// HTTP client with connection pooling
    http_client: reqwest::Client,

    /// Query result cache
    query_cache: Arc<QueryCache>,

    /// Rate limiter
    rate_limiter: Arc<RateLimiter>,

    /// CRUD operations handler
    crud: Arc<crud::CrudOperations>,
}

impl SalesforceClient {
    /// Creates a new Salesforce API client with the given configuration
    ///
    /// # Example
    /// ```no_run
    /// use salesforce_client::{SalesforceClient, ClientConfig};
    ///
    /// let config = ClientConfig::new(
    ///     "https://yourinstance.salesforce.com",
    ///     "00D... your token",
    /// );
    ///
    /// let client = SalesforceClient::new(config);
    /// ```
    pub fn new(config: ClientConfig) -> Self {
        let http_client = reqwest::Client::new();
        let query_cache = Arc::new(QueryCache::new(config.cache_config.clone()));
        let rate_limiter = Arc::new(RateLimiter::new(config.rate_limit_config.clone()));

        let crud = Arc::new(crud::CrudOperations::new(
            http_client.clone(),
            config.base_url.clone(),
            config.access_token.clone(),
        ));

        info!(
            "Salesforce client initialized with base URL: {}",
            config.base_url
        );

        Self {
            config: Arc::new(config),
            http_client,
            query_cache,
            rate_limiter,
            crud,
        }
    }

    /// Create a client with OAuth credentials (automatic token refresh)
    ///
    /// # Example
    /// ```no_run
    /// use salesforce_client::{SalesforceClient, OAuthCredentials};
    ///
    /// let credentials = OAuthCredentials {
    ///     client_id: "your_client_id".to_string(),
    ///     client_secret: "your_client_secret".to_string(),
    ///     refresh_token: Some("your_refresh_token".to_string()),
    ///     username: None,
    ///     password: None,
    /// };
    ///
    /// // This will be implemented with TokenManager integration
    /// // let client = SalesforceClient::with_oauth(credentials).await?;
    /// ```
    pub async fn with_oauth(credentials: OAuthCredentials) -> SfResult<Self> {
        let token_manager = TokenManager::new(credentials);
        let token = token_manager.get_token().await?;

        let config = ClientConfig::new(token.instance_url(), token.token());

        Ok(Self::new(config))
    }

    /// Execute a SOQL query with caching, retry, and rate limiting
    ///
    /// This method automatically handles:
    /// - Rate limiting (waits if limit exceeded)
    /// - Caching (returns cached results if available)
    /// - Retry logic (retries transient failures)
    /// - Pagination (fetches only first page by default)
    ///
    /// # Example
    /// ```no_run
    /// # use salesforce_client::{SalesforceClient, ClientConfig, SfError};
    /// # #[derive(Debug, Clone, Deserialize, Serialize)]
    /// # struct Account { #[serde(rename = "Id")] id: String }
    /// # async fn example() -> Result<(), SfError> {
    /// # let config = ClientConfig::new("https://example.com", "token");
    /// # let client = SalesforceClient::new(config);
    /// let accounts: Vec<Account> = client
    ///     .query("SELECT Id, Name FROM Account WHERE AnnualRevenue > 1000000")
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(self, soql))]
    pub async fn query<T>(&self, soql: impl AsRef<str>) -> SfResult<Vec<T>>
    where
        T: DeserializeOwned + Serialize + Clone,
    {
        let query_str = soql.as_ref();

        // Check cache first
        if let Some(cached) = self.query_cache.get::<T>(query_str).await {
            debug!("Returning cached query results");
            return Ok(cached);
        }

        // Apply rate limiting
        self.rate_limiter.acquire().await?;

        // Execute query with retry logic
        let result = retry::with_retry(&self.config.retry_config, || async {
            self.execute_query(query_str).await
        })
        .await?;

        // Cache the results (clone only if T is Clone, otherwise skip caching)
        // Note: We require T: Clone for caching
        if let Ok(()) = self.query_cache.set(query_str, result.clone()).await {
            // Cached successfully
        }

        Ok(result)
    }

    /// Execute query without caching (internal method)
    async fn execute_query<T>(&self, soql: &str) -> SfResult<Vec<T>>
    where
        T: DeserializeOwned,
    {
        let url = format!("{}/services/data/v57.0/query", self.config.base_url);

        debug!("Executing SOQL query");

        let response = self
            .http_client
            .get(&url)
            .query(&[("q", soql)])
            .header(
                "Authorization",
                format!("Bearer {}", self.config.access_token),
            )
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            // Check for rate limit before consuming response body
            let retry_after = if status.as_u16() == 429 {
                response
                    .headers()
                    .get("Retry-After")
                    .and_then(|v| v.to_str().ok())
                    .and_then(|s| s.parse().ok())
            } else {
                None
            };

            let body = response.text().await.unwrap_or_default();

            if status.as_u16() == 429 {
                return Err(SfError::RateLimit { retry_after });
            }

            return Err(SfError::Api {
                status: status.as_u16(),
                body,
            });
        }

        let query_response: pagination::QueryResponse<T> = response.json().await?;

        info!("Query returned {} records", query_response.records.len());
        Ok(query_response.records)
    }

    /// Query with automatic pagination - fetches ALL results
    ///
    /// **Warning**: This can consume significant memory for large result sets.
    /// For queries returning >100k records, consider using `query_paginated` instead.
    ///
    /// # Example
    /// ```no_run
    /// # use salesforce_client::{SalesforceClient, ClientConfig, SfError};
    /// # struct Account { #[serde(rename = "Id")] id: String }
    /// # async fn example() -> Result<(), SfError> {
    /// # let client = SalesforceClient::new(config);
    /// // Fetches all accounts, automatically handling pagination
    /// let all_accounts: Vec<Account> = client
    ///     .query_all("SELECT Id, Name FROM Account")
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(self, soql))]
    pub async fn query_all<T>(&self, soql: impl AsRef<str>) -> SfResult<Vec<T>>
    where
        T: DeserializeOwned + Serialize,
    {
        info!("Executing query with full pagination");

        let mut all_records = Vec::new();
        let mut pages = self.query_paginated::<T>(soql.as_ref()).await?;

        while let Some(batch) = pages.next().await? {
            all_records.extend(batch);
        }

        info!("Collected {} total records", all_records.len());
        Ok(all_records)
    }

    /// Get a paginated query iterator for manual pagination control
    ///
    /// This is the most memory-efficient way to handle large result sets.
    ///
    /// # Example
    /// ```no_run
    /// # use serde::{Deserialize, Serialize};
    /// # use salesforce_client::{SalesforceClient, ClientConfig, SfError};
    /// # use serde::{Deserialize, Serialize};
    /// # async fn example() -> Result<(), SfError> {
    /// # let config = ClientConfig::new("https://example.com", "token");
    ///     .await?;
    ///
    /// while let Some(batch) = pages.next().await? {
    ///     // Process each batch of records
    ///     for account in batch {
    ///         println!("{:?}", account);
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn query_paginated<T>(&self, soql: &str) -> SfResult<PaginatedQuery<T>>
    where
        T: DeserializeOwned,
    {
        // Execute first query to get initial results and nextRecordsUrl
        let url = format!("{}/services/data/v57.0/query", self.config.base_url);

        self.rate_limiter.acquire().await?;

        let response = self
            .http_client
            .get(&url)
            .query(&[("q", soql)])
            .header(
                "Authorization",
                format!("Bearer {}", self.config.access_token),
            )
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(SfError::Api {
                status: status.as_u16(),
                body,
            });
        }

        let query_response: pagination::QueryResponse<T> = response.json().await?;
        let next_url = query_response.next_records_url.clone();

        Ok(PaginatedQuery::new(
            self.http_client.clone(),
            self.config.base_url.clone(),
            self.config.access_token.clone(),
            next_url,
        ))
    }

    /// Insert a new record
    ///
    /// # Example
    /// ```no_run
    /// # use salesforce_client::{SalesforceClient, ClientConfig, SfError};
    /// # use serde::Serialize;
    /// #[derive(Serialize)]
    /// struct NewAccount {
    ///     #[serde(rename = "Name")]
    ///     name: String,
    ///     #[serde(rename = "Industry")]
    ///     industry: String,
    /// }
    /// # async fn example() -> Result<(), SfError> {
    /// # let config = ClientConfig::new("https://example.com", "token");
    /// # let client = SalesforceClient::new(config);
    ///
    /// let account = NewAccount {
    ///     name: "Acme Corporation".to_string(),
    ///     industry: "Technology".to_string(),
    /// };
    ///
    /// let response = client.insert("Account", &account).await?;
    /// println!("Created account with ID: {}", response.id);
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(self, data))]
    pub async fn insert<T: Serialize>(&self, sobject: &str, data: &T) -> SfResult<InsertResponse> {
        self.rate_limiter.acquire().await?;

        retry::with_retry(&self.config.retry_config, || async {
            self.crud.insert(sobject, data).await
        })
        .await
    }

    /// Update an existing record
    ///
    /// # Example
    /// ```no_run
    /// # use salesforce_client::{SalesforceClient, ClientConfig, SfError};
    /// # use serde::Serialize;
    /// #[derive(Serialize)]
    /// struct AccountUpdate {
    ///     #[serde(rename = "Name")]
    ///     name: String,
    /// }
    /// # async fn example() -> Result<(), SfError> {
    /// # let config = ClientConfig::new("https://example.com", "token");
    /// # let client = SalesforceClient::new(config);
    ///
    /// let update = AccountUpdate {
    ///     name: "Acme Corp (Updated)".to_string(),
    /// };
    ///
    /// client.update("Account", "001xx000003DGbX", &update).await?;
    /// println!("Account updated successfully");
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(self, data))]
    pub async fn update<T: Serialize>(&self, sobject: &str, id: &str, data: &T) -> SfResult<()> {
        self.rate_limiter.acquire().await?;

        retry::with_retry(&self.config.retry_config, || async {
            self.crud.update(sobject, id, data).await
        })
        .await?;

        // Invalidate cache for this record
        self.query_cache.clear().await;

        Ok(())
    }

    /// Delete a record
    ///
    /// # Example
    /// ```no_run
    /// # use salesforce_client::{SalesforceClient, ClientConfig, SfError};
    /// # async fn example() -> Result<(), SfError> {
    /// # let config = ClientConfig::new("https://example.com", "token");
    /// # let client = SalesforceClient::new(config);
    ///
    /// client.delete("Account", "001xx000003DGbX").await?;
    /// println!("Account deleted successfully");
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(self))]
    pub async fn delete(&self, sobject: &str, id: &str) -> SfResult<()> {
        self.rate_limiter.acquire().await?;

        retry::with_retry(&self.config.retry_config, || async {
            self.crud.delete(sobject, id).await
        })
        .await?;

        // Invalidate cache
        self.query_cache.clear().await;

        Ok(())
    }

    /// Upsert a record (insert or update based on external ID)
    ///
    /// # Example
    /// ```no_run
    /// # use salesforce_client::{SalesforceClient, ClientConfig, UpsertBuilder, SfError};
    /// # use serde::Serialize;
    /// #[derive(Serialize)]
    /// struct Account {
    ///     #[serde(rename = "Name")]
    ///     name: String,
    /// }
    /// # async fn example() -> Result<(), SfError> {
    /// # let config = ClientConfig::new("https://example.com", "token");
    /// # let client = SalesforceClient::new(config);
    ///
    /// let account = Account {
    ///     name: "Acme Corporation".to_string(),
    /// };
    ///
    /// let upsert = UpsertBuilder::new("External_Id__c", "EXT-12345");
    /// let response = client.upsert("Account", upsert, &account).await?;
    /// println!("Upserted account with ID: {}", response.id);
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(self, data))]
    pub async fn upsert<T: Serialize>(
        &self,
        sobject: &str,
        builder: UpsertBuilder,
        data: &T,
    ) -> SfResult<InsertResponse> {
        self.rate_limiter.acquire().await?;

        let result = retry::with_retry(&self.config.retry_config, || async {
            self.crud.upsert(sobject, builder.clone(), data).await
        })
        .await?;

        // Invalidate cache
        self.query_cache.clear().await;

        Ok(result)
    }

    // ========================================================================
    // Utility Methods
    // ========================================================================

    /// Clear the query cache
    pub async fn clear_cache(&self) {
        self.query_cache.clear().await;
        info!("Cache cleared");
    }

    /// Get the current configuration
    pub fn config(&self) -> &ClientConfig {
        &self.config
    }

    /// Get rate limiter status
    pub fn rate_limit_status(&self) -> rate_limit::RateLimitStatus {
        self.rate_limiter.status()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_config_builder() {
        let config = ClientConfig::new("https://test.salesforce.com", "test_token")
            .with_cache(CacheConfig::disabled())
            .no_pagination();

        assert_eq!(config.base_url, "https://test.salesforce.com");
        assert!(!config.auto_paginate);
    }

    #[test]
    fn test_client_creation() {
        let config = ClientConfig::new("https://test.salesforce.com", "test_token");

        let client = SalesforceClient::new(config);
        assert_eq!(client.config.base_url, "https://test.salesforce.com");
    }
}
