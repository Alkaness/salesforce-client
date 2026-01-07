//! Automatic pagination support for large query results
//!
//! Salesforce limits query results to 2000 records per request.
//! This module handles automatic pagination transparently.

use crate::error::{SfError, SfResult};
use serde::de::DeserializeOwned;
use serde::Deserialize;
use tracing::{debug, info};

/// Response from Salesforce query with pagination info
#[derive(Debug, Deserialize)]
pub(crate) struct QueryResponse<T> {
    /// The list of records returned by the query
    pub records: Vec<T>,

    /// Whether all results were returned or if pagination is needed
    pub done: bool,

    /// Total number of records (optional, may not always be present)
    #[serde(rename = "totalSize")]
    pub total_size: Option<i32>,

    /// URL for fetching next batch of records
    #[serde(rename = "nextRecordsUrl")]
    pub next_records_url: Option<String>,
}

impl<T> QueryResponse<T> {
    /// Check if there are more records to fetch
    pub fn has_more(&self) -> bool {
        !self.done && self.next_records_url.is_some()
    }
}

/// Iterator for paginated query results
///
/// This allows users to iterate over all results without worrying about pagination:
/// ```ignore
/// let mut pages = client.query_paginated::<Account>("SELECT Id FROM Account").await?;
///
/// while let Some(batch) = pages.next().await? {
///     for account in batch {
///         println!("{:?}", account);
///     }
/// }
/// ```
pub struct PaginatedQuery<T> {
    client: reqwest::Client,
    base_url: String,
    access_token: String,
    next_url: Option<String>,
    finished: bool,
    _phantom: std::marker::PhantomData<T>,
}

impl<T: DeserializeOwned> PaginatedQuery<T> {
    /// Create a new paginated query iterator
    pub(crate) fn new(
        client: reqwest::Client,
        base_url: String,
        access_token: String,
        initial_url: Option<String>,
    ) -> Self {
        let finished = initial_url.is_none();
        Self {
            client,
            base_url,
            access_token,
            next_url: initial_url,
            finished,
            _phantom: std::marker::PhantomData,
        }
    }

    /// Fetch the next page of results
    pub async fn next(&mut self) -> SfResult<Option<Vec<T>>> {
        if self.finished {
            return Ok(None);
        }

        let url = match &self.next_url {
            Some(path) => {
                // nextRecordsUrl is a relative path, prepend base URL
                if path.starts_with("http") {
                    path.clone()
                } else {
                    format!("{}{}", self.base_url, path)
                }
            }
            None => {
                self.finished = true;
                return Ok(None);
            }
        };

        debug!("Fetching paginated results from: {}", url);

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.access_token))
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().await?;
            return Err(SfError::Api {
                status: status.as_u16(),
                body,
            });
        }

        let query_response: QueryResponse<T> = response.json().await?;

        if query_response.done {
            self.finished = true;
            self.next_url = None;
            info!("Pagination complete");
        } else {
            self.next_url = query_response.next_records_url;
            debug!("More records available, next URL: {:?}", self.next_url);
        }

        Ok(Some(query_response.records))
    }

    /// Collect all remaining pages into a single vector
    ///
    /// **Warning:** This loads all results into memory. For very large
    /// result sets (>100k records), consider processing pages individually.
    pub async fn collect_all(mut self) -> SfResult<Vec<T>> {
        let mut all_records = Vec::new();

        while let Some(batch) = self.next().await? {
            all_records.extend(batch);
        }

        info!(
            "Collected {} total records across all pages",
            all_records.len()
        );
        Ok(all_records)
    }
}

/// Builder for query options
#[derive(Debug, Clone)]
pub struct QueryOptions {
    /// Maximum number of records to fetch (None = fetch all)
    pub limit: Option<usize>,

    /// Batch size per request (max 2000)
    pub batch_size: usize,

    /// Enable automatic pagination
    pub auto_paginate: bool,
}

impl Default for QueryOptions {
    fn default() -> Self {
        Self {
            limit: None,
            batch_size: 2000,
            auto_paginate: true,
        }
    }
}

impl QueryOptions {
    /// Create a new QueryOptions with defaults
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the maximum number of records to fetch
    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Set the batch size (how many records per API call)
    pub fn batch_size(mut self, size: usize) -> Self {
        self.batch_size = size.min(2000); // Salesforce max is 2000
        self
    }

    /// Disable automatic pagination (single request only)
    pub fn no_pagination(mut self) -> Self {
        self.auto_paginate = false;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_options_builder() {
        let opts = QueryOptions::new().limit(1000).batch_size(500);

        assert_eq!(opts.limit, Some(1000));
        assert_eq!(opts.batch_size, 500);
        assert!(opts.auto_paginate);
    }

    #[test]
    fn test_query_options_max_batch_size() {
        let opts = QueryOptions::new().batch_size(5000);

        // Should be clamped to 2000
        assert_eq!(opts.batch_size, 2000);
    }
}
